//! Local collection — CRUD over `collection_entries`.
//!
//! Every method takes a `&Connection` so callers can batch operations into a
//! transaction when they need atomicity. Nothing here talks to the network.

use crate::core::{CardCondition, CardId, Error, Game, Result};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionEntry {
    pub entry_id: String,
    pub game: Game,
    pub card_id: CardId,
    pub condition: CardCondition,
    pub foil: bool,
    pub quantity: u32,
    pub notes: Option<String>,
    pub acquired_at: Option<String>,
    pub acquired_price_cents: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewEntry {
    pub game: Game,
    pub card_id: CardId,
    pub condition: CardCondition,
    #[serde(default)]
    pub foil: bool,
    pub quantity: u32,
    pub notes: Option<String>,
    pub acquired_at: Option<String>,
    pub acquired_price_cents: Option<u64>,
}

pub fn add(conn: &Connection, entry: NewEntry) -> Result<CollectionEntry> {
    if entry.quantity == 0 {
        return Err(Error::InvalidInput("quantity must be > 0".into()));
    }

    let entry_id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO collection_entries
            (entry_id, game, card_id, condition, foil, quantity, notes, acquired_at, acquired_price_cents)
         VALUES
            (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            entry_id,
            entry.game.slug(),
            entry.card_id.0,
            entry.condition.code(),
            entry.foil as i64,
            entry.quantity as i64,
            entry.notes,
            entry.acquired_at,
            entry.acquired_price_cents.map(|c| c as i64),
        ],
    )?;

    Ok(CollectionEntry {
        entry_id,
        game: entry.game,
        card_id: entry.card_id,
        condition: entry.condition,
        foil: entry.foil,
        quantity: entry.quantity,
        notes: entry.notes,
        acquired_at: entry.acquired_at,
        acquired_price_cents: entry.acquired_price_cents,
    })
}

pub fn remove(conn: &Connection, entry_id: &str) -> Result<()> {
    let n = conn.execute(
        "DELETE FROM collection_entries WHERE entry_id = ?1",
        params![entry_id],
    )?;
    if n == 0 {
        return Err(Error::CardNotFound(format!("entry {entry_id}")));
    }
    Ok(())
}

pub fn list(conn: &Connection, game: Option<Game>) -> Result<Vec<CollectionEntry>> {
    let mut rows = Vec::new();
    let query = match game {
        Some(_) => {
            "SELECT entry_id, game, card_id, condition, foil, quantity, notes, acquired_at, acquired_price_cents
             FROM collection_entries
             WHERE game = ?1
             ORDER BY created_at DESC"
        }
        None => {
            "SELECT entry_id, game, card_id, condition, foil, quantity, notes, acquired_at, acquired_price_cents
             FROM collection_entries
             ORDER BY created_at DESC"
        }
    };

    let mut stmt = conn.prepare(query)?;
    let map_row = |row: &rusqlite::Row| -> rusqlite::Result<CollectionEntry> {
        let game_slug: String = row.get(1)?;
        let condition_code: String = row.get(3)?;
        Ok(CollectionEntry {
            entry_id: row.get(0)?,
            game: Game::from_slug(&game_slug).unwrap_or(Game::Mtg),
            card_id: CardId(row.get(2)?),
            condition: condition_from_code(&condition_code),
            foil: row.get::<_, i64>(4)? != 0,
            quantity: row.get::<_, i64>(5)? as u32,
            notes: row.get(6)?,
            acquired_at: row.get(7)?,
            acquired_price_cents: row
                .get::<_, Option<i64>>(8)?
                .map(|v| v.max(0) as u64),
        })
    };

    match game {
        Some(g) => {
            let iter = stmt.query_map(params![g.slug()], map_row)?;
            for entry in iter {
                rows.push(entry?);
            }
        }
        None => {
            let iter = stmt.query_map([], map_row)?;
            for entry in iter {
                rows.push(entry?);
            }
        }
    }
    Ok(rows)
}

fn condition_from_code(code: &str) -> CardCondition {
    match code {
        "NM" => CardCondition::NearMint,
        "LP" => CardCondition::LightlyPlayed,
        "MP" => CardCondition::ModeratelyPlayed,
        "HP" => CardCondition::HeavilyPlayed,
        "DMG" => CardCondition::Damaged,
        // Unknown condition codes fall back to NM — safest default because it
        // doesn't silently mark cards as damaged. We still log above.
        _ => CardCondition::NearMint,
    }
}
