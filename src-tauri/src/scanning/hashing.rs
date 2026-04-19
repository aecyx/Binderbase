// SPDX-License-Identifier: AGPL-3.0-or-later
//! Perceptual hashing for card image matching.
//!
//! Uses a 256-bit difference hash (dHash):
//! 1. Convert to grayscale.
//! 2. Resize to 17×16 pixels (one extra column for gradient computation).
//! 3. For each row, compare adjacent pixels: bit = 1 if left pixel is
//!    brighter than right pixel.
//! 4. Produces 16 comparisons × 16 rows = 256 bits = 32 bytes.
//!
//! dHash is robust against brightness/contrast changes and minor resizing
//! artifacts, making it well-suited for matching photos of physical cards
//! against clean catalog thumbnails.

use crate::core::{CardId, Error, Game, Result};
use image::{imageops::FilterType, DynamicImage, ImageReader};
use rusqlite::{params, Connection};
use std::io::Cursor;

/// Hash size in bytes (256 bits = 32 bytes).
pub const HASH_SIZE: usize = 32;

/// Compute a 256-bit difference hash (dHash) from raw image bytes.
pub fn compute_dhash_from_bytes(bytes: &[u8]) -> Result<[u8; HASH_SIZE]> {
    let reader = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|e| Error::ImageDecode(e.to_string()))?;
    let img = reader.decode()?;
    Ok(compute_dhash(&img))
}

/// Compute a 256-bit difference hash (dHash) from a decoded image.
pub fn compute_dhash(img: &DynamicImage) -> [u8; HASH_SIZE] {
    // Resize to 17 wide × 16 tall so we get 16 horizontal comparisons per row.
    let gray = img.resize_exact(17, 16, FilterType::Lanczos3).to_luma8();
    let mut hash = [0u8; HASH_SIZE];
    let mut bit_idx = 0usize;

    for y in 0..16u32 {
        for x in 0..16u32 {
            let left = gray.get_pixel(x, y).0[0];
            let right = gray.get_pixel(x + 1, y).0[0];
            if left > right {
                hash[bit_idx / 8] |= 1 << (7 - (bit_idx % 8));
            }
            bit_idx += 1;
        }
    }

    hash
}

/// Hamming distance between two 256-bit hashes (number of differing bits).
pub fn hamming_distance(a: &[u8; HASH_SIZE], b: &[u8; HASH_SIZE]) -> u32 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x ^ y).count_ones())
        .sum()
}

/// Store a hash for a card. Upserts on conflict.
pub fn store_hash(
    conn: &Connection,
    game: Game,
    card_id: &CardId,
    hash: &[u8; HASH_SIZE],
) -> Result<()> {
    conn.execute(
        "INSERT INTO card_hashes (game, card_id, hash)
         VALUES (?1, ?2, ?3)
         ON CONFLICT (game, card_id)
         DO UPDATE SET hash = excluded.hash, created_at = datetime('now')",
        params![game.slug(), &card_id.0, &hash[..]],
    )?;
    Ok(())
}

/// Load all hashes for a given game. Returns (card_id, hash) pairs.
pub fn load_hashes(conn: &Connection, game: Game) -> Result<Vec<(CardId, [u8; HASH_SIZE])>> {
    let mut stmt = conn.prepare_cached("SELECT card_id, hash FROM card_hashes WHERE game = ?1")?;
    let rows = stmt.query_map(params![game.slug()], |r| {
        let card_id: String = r.get(0)?;
        let hash_blob: Vec<u8> = r.get(1)?;
        Ok((card_id, hash_blob))
    })?;
    let mut result = Vec::new();
    for row in rows {
        let (card_id, hash_blob) = row?;
        if hash_blob.len() == HASH_SIZE {
            let mut hash = [0u8; HASH_SIZE];
            hash.copy_from_slice(&hash_blob);
            result.push((CardId(card_id), hash));
        }
    }
    Ok(result)
}

/// Count how many cards have hashes vs total cards with image URLs for a game.
pub fn index_coverage(conn: &Connection, game: Game) -> Result<(u64, u64)> {
    let hashed: u64 = conn.query_row(
        "SELECT COUNT(*) FROM card_hashes WHERE game = ?1",
        params![game.slug()],
        |r| r.get(0),
    )?;
    let total: u64 = conn.query_row(
        "SELECT COUNT(*) FROM cards WHERE game = ?1 AND image_url IS NOT NULL",
        params![game.slug()],
        |r| r.get(0),
    )?;
    Ok((hashed, total))
}

/// Find the nearest matches for a query hash among stored hashes.
///
/// Returns matches with confidence ≥ `min_confidence`, sorted by confidence
/// descending, capped at `limit`.
pub fn find_nearest(
    conn: &Connection,
    game: Game,
    query_hash: &[u8; HASH_SIZE],
    limit: usize,
    min_confidence: f32,
) -> Result<Vec<(CardId, f32)>> {
    let all_hashes = load_hashes(conn, game)?;
    let max_bits = (HASH_SIZE * 8) as f32;
    let mut candidates: Vec<(CardId, f32)> = all_hashes
        .iter()
        .filter_map(|(card_id, stored_hash)| {
            let dist = hamming_distance(query_hash, stored_hash);
            let confidence = 1.0 - (dist as f32 / max_bits);
            if confidence >= min_confidence {
                Some((card_id.clone(), confidence))
            } else {
                None
            }
        })
        .collect();
    candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    candidates.truncate(limit);
    Ok(candidates)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_images_have_zero_distance() {
        let img = DynamicImage::new_rgb8(100, 100);
        let hash1 = compute_dhash(&img);
        let hash2 = compute_dhash(&img);
        assert_eq!(hamming_distance(&hash1, &hash2), 0);
    }

    #[test]
    fn hash_is_correct_size() {
        let img = DynamicImage::new_rgb8(50, 50);
        let hash = compute_dhash(&img);
        assert_eq!(hash.len(), HASH_SIZE);
    }

    #[test]
    fn hamming_distance_is_symmetric() {
        let a = [0xAA; HASH_SIZE];
        let b = [0x55; HASH_SIZE];
        assert_eq!(hamming_distance(&a, &b), hamming_distance(&b, &a));
    }

    #[test]
    fn hamming_distance_max_is_256() {
        let a = [0xFF; HASH_SIZE];
        let b = [0x00; HASH_SIZE];
        assert_eq!(hamming_distance(&a, &b), 256);
    }

    #[test]
    fn store_and_load_round_trips() {
        let conn = crate::storage::test_support::memory_conn();
        let game = Game::Mtg;
        let card_id = CardId("test-card-123".into());
        let hash = [0xAB; HASH_SIZE];

        conn.execute(
            "INSERT INTO cards (game, card_id, name, set_code, set_name, collector_number, image_url)
             VALUES ('mtg', 'test-card-123', 'Test Card', 'TST', 'Test Set', '1', 'https://example.com/img.jpg')",
            [],
        )
        .unwrap();

        store_hash(&conn, game, &card_id, &hash).unwrap();
        let loaded = load_hashes(&conn, game).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].0, card_id);
        assert_eq!(loaded[0].1, hash);
    }

    #[test]
    fn store_hash_upserts_on_conflict() {
        let conn = crate::storage::test_support::memory_conn();
        let game = Game::Mtg;
        let card_id = CardId("test-card-123".into());

        conn.execute(
            "INSERT INTO cards (game, card_id, name, set_code, set_name, collector_number, image_url)
             VALUES ('mtg', 'test-card-123', 'Test Card', 'TST', 'Test Set', '1', 'https://example.com/img.jpg')",
            [],
        )
        .unwrap();

        let hash1 = [0xAA; HASH_SIZE];
        let hash2 = [0xBB; HASH_SIZE];
        store_hash(&conn, game, &card_id, &hash1).unwrap();
        store_hash(&conn, game, &card_id, &hash2).unwrap();

        let loaded = load_hashes(&conn, game).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].1, hash2);
    }

    #[test]
    fn index_coverage_counts_correctly() {
        let conn = crate::storage::test_support::memory_conn();

        conn.execute(
            "INSERT INTO cards (game, card_id, name, set_code, set_name, collector_number, image_url)
             VALUES ('mtg', 'c1', 'Card 1', 'S', 'Set', '1', 'https://example.com/1.jpg')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO cards (game, card_id, name, set_code, set_name, collector_number, image_url)
             VALUES ('mtg', 'c2', 'Card 2', 'S', 'Set', '2', 'https://example.com/2.jpg')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO cards (game, card_id, name, set_code, set_name, collector_number)
             VALUES ('mtg', 'c3', 'Card 3', 'S', 'Set', '3')",
            [],
        )
        .unwrap();

        store_hash(&conn, Game::Mtg, &CardId("c1".into()), &[0xAA; HASH_SIZE]).unwrap();

        let (hashed, total) = index_coverage(&conn, Game::Mtg).unwrap();
        assert_eq!(hashed, 1);
        assert_eq!(total, 2); // Only cards with image_url
    }

    #[test]
    fn find_nearest_returns_sorted_by_confidence() {
        let conn = crate::storage::test_support::memory_conn();

        for (id, hash_byte) in [("c1", 0x00u8), ("c2", 0xFFu8)] {
            conn.execute(
                "INSERT INTO cards (game, card_id, name, set_code, set_name, collector_number, image_url)
                 VALUES ('mtg', ?1, 'Card', 'S', 'Set', '1', 'https://example.com/img.jpg')",
                params![id],
            )
            .unwrap();
            store_hash(
                &conn,
                Game::Mtg,
                &CardId(id.into()),
                &[hash_byte; HASH_SIZE],
            )
            .unwrap();
        }

        let query = [0x00; HASH_SIZE];
        let matches = find_nearest(&conn, Game::Mtg, &query, 5, 0.0).unwrap();
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].0, CardId("c1".into()));
        assert_eq!(matches[0].1, 1.0);
        assert_eq!(matches[1].0, CardId("c2".into()));
        assert_eq!(matches[1].1, 0.0);
    }
}
