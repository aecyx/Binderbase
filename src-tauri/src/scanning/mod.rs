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
//! The 1.0 pipeline is simple and local:
//!  1. Decode bytes → `image::DynamicImage`.
//!  2. Preprocess (orient, crop to card bounds, normalize).
//!  3. Feature-match against the local catalog's perceptual hashes.
//!
//! Steps 2 and 3 arrive incrementally. For now we decode + validate; the rest
//! is a typed stub so the UI can call it end-to-end.

use crate::core::{CardId, Error, Game, Result};
use image::ImageReader;
use serde::{Deserialize, Serialize};
use std::io::Cursor;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Match {
    pub game: Game,
    pub card_id: CardId,
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

/// Attempt to identify a card from raw image bytes.
///
/// Today this only decodes and reports the image dimensions; it returns no
/// matches. The signature is stable so UI and tests can wire against it.
pub fn identify(bytes: &[u8], _game_hint: Option<Game>) -> Result<ScanResult> {
    if bytes.is_empty() {
        return Err(Error::InvalidInput("empty image buffer".into()));
    }
    let reader = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|e| Error::ImageDecode(e.to_string()))?;
    let img = reader.decode()?;

    Ok(ScanResult {
        matches: Vec::new(),
        width: img.width(),
        height: img.height(),
    })
}
