// SPDX-License-Identifier: AGPL-3.0-or-later
//! Scanning pipeline.
//!
//! Inputs:
//!  * A still image (JPEG/PNG/WebP bytes) from a webcam capture, file load,
//!    or — eventually — a phone camera on mobile.
//!
//! Outputs:
//!  * Candidate matches (`Match`) ordered by confidence.
//!
//! The pipeline:
//!  1. Decode bytes → `image::DynamicImage`.
//!  2. Compute a 256-bit perceptual hash (dHash).
//!  3. Brute-force nearest-neighbour search against pre-computed hashes in the
//!     local `card_hashes` table, using Hamming distance.
//!  4. Return the top candidates with confidence scores and card details.
//!
//! The hash index is built separately via [`index::build_index`] which
//! downloads card art thumbnails from the CDN and computes dHash for each.

pub mod hashing;
pub mod index;

use crate::catalog;
use crate::core::{CardId, Error, Game, Result};
use image::{ImageReader, Limits};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::io::Cursor;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Match {
    pub game: Game,
    pub card_id: CardId,
    pub name: String,
    pub set_name: String,
    pub image_url: Option<String>,
    /// Model confidence on [0.0, 1.0].
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    /// Best candidates, best-first. May be empty.
    pub matches: Vec<Match>,
    pub width: u32,
    pub height: u32,
}

/// Minimum confidence to include in results.
const MIN_CONFIDENCE: f32 = 0.70;

/// Maximum number of match results to return.
const MAX_RESULTS: usize = 5;

/// Maximum raw image bytes accepted (10 MB).
const MAX_IMAGE_BYTES: usize = 10 * 1024 * 1024;

/// Maximum decoded pixel count (50 megapixels).
const MAX_PIXEL_COUNT: u64 = 50_000_000;

/// Identify a card from raw image bytes by comparing its perceptual hash
/// against the local hash index.
///
/// If the hash index is empty (not yet built), returns an empty match list
/// gracefully — the UI shows guidance to build the index first.
pub fn identify(bytes: &[u8], game_hint: Option<Game>, conn: &Connection) -> Result<ScanResult> {
    if bytes.is_empty() {
        return Err(Error::InvalidInput("empty image buffer".into()));
    }

    // Guard: reject oversized payloads before decoding.
    if bytes.len() > MAX_IMAGE_BYTES {
        return Err(Error::InputTooLarge {
            message: format!(
                "image file exceeds the {} MB size limit",
                MAX_IMAGE_BYTES / (1024 * 1024)
            ),
            limit: format!("{MAX_IMAGE_BYTES}"),
            actual: format!("{}", bytes.len()),
        });
    }

    let mut reader = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|e| Error::ImageDecode(e.to_string()))?;

    // Set decode limits *before* calling decode() to prevent
    // decompression bombs from allocating unbounded memory.
    let mut limits = Limits::default();
    // 4 bytes per pixel (RGBA) × MAX_PIXEL_COUNT
    limits.max_alloc = Some(MAX_PIXEL_COUNT * 4);
    reader.limits(limits);

    let img = reader.decode()?;
    let width = img.width();
    let height = img.height();

    // Secondary guard: reject on pixel count even if decoder
    // allocation tracking was imprecise.
    let pixel_count = u64::from(width) * u64::from(height);
    if pixel_count > MAX_PIXEL_COUNT {
        return Err(Error::InputTooLarge {
            message: format!(
                "decoded image exceeds the {} megapixel limit",
                MAX_PIXEL_COUNT / 1_000_000
            ),
            limit: format!("{MAX_PIXEL_COUNT}"),
            actual: format!("{pixel_count}"),
        });
    }

    let query_hash = hashing::compute_dhash(&img);

    let games = match game_hint {
        Some(g) => vec![g],
        None => Game::all().to_vec(),
    };

    let mut all_matches = Vec::new();
    for game in games {
        let candidates =
            hashing::find_nearest(conn, game, &query_hash, MAX_RESULTS, MIN_CONFIDENCE)?;
        for (card_id, confidence) in candidates {
            let (name, set_name, image_url) = match catalog::get(conn, game, &card_id)? {
                Some(card) => (card.name, card.set_name, card.image_url),
                None => (card_id.0.clone(), String::new(), None),
            };
            all_matches.push(Match {
                game,
                card_id,
                name,
                set_name,
                image_url,
                confidence,
            });
        }
    }

    all_matches.sort_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    all_matches.truncate(MAX_RESULTS);

    Ok(ScanResult {
        matches: all_matches,
        width,
        height,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_identify_rejects_oversized_image_bytes() {
        // 11 MB of zeros — exceeds the 10 MB limit.
        let huge = vec![0u8; 11 * 1024 * 1024];
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        let err = identify(&huge, None, &conn).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("10 MB"),
            "error should mention size limit: {msg}"
        );
    }

    #[test]
    fn scan_identify_rejects_empty_buffer() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        let err = identify(&[], None, &conn).unwrap_err();
        assert!(err.to_string().contains("empty"));
    }
}
