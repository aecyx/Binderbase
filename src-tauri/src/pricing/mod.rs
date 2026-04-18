// SPDX-License-Identifier: AGPL-3.0-or-later
//! Local price cache + lookup.
//!
//! Philosophy: we never block on a live price fetch for UI reads. Reads go
//! straight to the `prices` table. A background refresh task (to be wired in
//! later) populates the cache from public sources.
//!
//! Currencies stay as integer cents to avoid floating-point sadness.

use crate::core::{CardId, Error, Game, Result};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Price {
    pub game: Game,
    pub card_id: CardId,
    pub currency: String,
    pub source: String,
    pub cents: u64,
    pub foil: bool,
    pub fetched_at: String,
}

/// Return the latest cached prices for a card, across all sources and foil
/// variants.
pub fn get_cached(conn: &Connection, game: Game, card_id: &CardId) -> Result<Vec<Price>> {
    let mut stmt = conn.prepare(
        "SELECT game, card_id, currency, source, cents, foil, fetched_at
         FROM prices
         WHERE game = ?1 AND card_id = ?2
         ORDER BY fetched_at DESC",
    )?;
    let rows = stmt
        .query_map(params![game.slug(), &card_id.0], |r| {
            let game_slug: String = r.get(0)?;
            Ok(Price {
                game: Game::from_slug(&game_slug).unwrap_or(Game::Mtg),
                card_id: CardId(r.get(1)?),
                currency: r.get(2)?,
                source: r.get(3)?,
                cents: r.get::<_, i64>(4)?.max(0) as u64,
                foil: r.get::<_, i64>(5)? != 0,
                fetched_at: r.get(6)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| Error::Storage(e.to_string()))?;
    Ok(rows)
}

/// Upsert a price row. `(game, card_id, currency, source, foil)` is the
/// natural key; `fetched_at` is bumped on conflict.
pub fn upsert(conn: &Connection, price: &Price) -> Result<()> {
    conn.execute(
        "INSERT INTO prices (game, card_id, currency, source, cents, foil, fetched_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, datetime('now'))
         ON CONFLICT (game, card_id, currency, source, foil)
         DO UPDATE SET cents = excluded.cents, fetched_at = datetime('now')",
        params![
            price.game.slug(),
            price.card_id.0,
            price.currency,
            price.source,
            price.cents as i64,
            price.foil as i64,
        ],
    )?;
    Ok(())
}
