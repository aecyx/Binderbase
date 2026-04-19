// SPDX-License-Identifier: AGPL-3.0-or-later
//! Persistence layer for bulk catalog imports.
//!
//! Batched upserts into the `cards` table, import-log bookkeeping in
//! `catalog_imports`, and the `last_imported_at` settings stamp.

use crate::catalog;
use crate::core::{Error, Game, Result};
use crate::pricing::{self, Price};
use crate::settings;
use crate::storage::Database;
use rusqlite::{params, Connection};
use uuid::Uuid;

use super::controller::{
    ImportController, ImportProgress, ImportRunSummary, ImportStatus, ProgressSink,
};
use super::BATCH_SIZE;

/// Batched upsert into `cards`. Pure — takes an explicit Vec so tests can
/// feed fixture data without an HTTP layer.
pub fn persist_cards(
    db: &Database,
    game: Game,
    cards: &[crate::core::Card],
    sink: &dyn ProgressSink,
    controller: &ImportController,
) -> Result<u64> {
    let total = cards.len() as u64;
    sink.emit(ImportProgress {
        game: Some(game),
        stage: "importing".into(),
        processed: 0,
        total: Some(total),
        message: None,
    });

    let mut conn = db.connect()?;
    let mut processed = 0u64;

    for chunk in cards.chunks(BATCH_SIZE) {
        if controller.is_cancelled() {
            return Err(Error::Internal("import cancelled".into()));
        }
        let tx = conn.transaction()?;
        for card in chunk {
            catalog::upsert(&tx, card)?;
        }
        tx.commit()?;
        processed += chunk.len() as u64;

        sink.emit(ImportProgress {
            game: Some(game),
            stage: "importing".into(),
            processed,
            total: Some(total),
            message: None,
        });
    }

    stamp_last_imported(&conn, game)?;
    Ok(processed)
}

pub fn stamp_last_imported(conn: &Connection, game: Game) -> Result<()> {
    settings::set(
        conn,
        &settings::last_imported_at_key(game.slug()),
        &chrono::Utc::now().to_rfc3339(),
    )
}

// ---------- catalog_imports table helpers ----------

pub fn log_import_start(db: &Database, import_id: &str, game: Game) -> Result<()> {
    let conn = db.connect()?;
    conn.execute(
        "INSERT INTO catalog_imports (import_id, game, status) VALUES (?1, ?2, 'running')",
        params![import_id, game.slug()],
    )?;
    Ok(())
}

pub fn finalize_import_log(
    db: &Database,
    import_id: &str,
    result: &Result<u64>,
    controller: &ImportController,
) -> Result<()> {
    let (status, cards_imported, error_message): (&str, u64, Option<String>) = match result {
        Ok(n) => ("completed", *n, None),
        Err(e) if controller.is_cancelled() => ("cancelled", 0, Some(e.to_string())),
        Err(e) => ("failed", 0, Some(e.to_string())),
    };
    let conn = db.connect()?;
    conn.execute(
        "UPDATE catalog_imports
         SET finished_at = datetime('now'),
             status = ?2,
             cards_imported = ?3,
             error_message = ?4
         WHERE import_id = ?1",
        params![import_id, status, cards_imported as i64, error_message],
    )?;
    Ok(())
}

/// Most recent `catalog_imports` row for each game.
pub fn latest_run(conn: &Connection, game: Game) -> Result<Option<ImportRunSummary>> {
    let mut stmt = conn.prepare_cached(
        "SELECT game, started_at, finished_at, status, cards_imported, error_message
         FROM catalog_imports
         WHERE game = ?1
         ORDER BY started_at DESC
         LIMIT 1",
    )?;
    let mut rows = stmt.query_map(params![game.slug()], |r| {
        let game_slug: String = r.get(0)?;
        Ok(ImportRunSummary {
            game: Game::from_slug(&game_slug).unwrap_or(Game::Mtg),
            started_at: r.get(1)?,
            finished_at: r.get(2)?,
            status: r.get(3)?,
            cards_imported: r.get::<_, i64>(4)? as u64,
            error_message: r.get(5)?,
        })
    })?;
    match rows.next() {
        Some(r) => Ok(Some(r?)),
        None => Ok(None),
    }
}

/// Fan-in for ImportStatus: reads both games' latest runs + controller state.
pub fn current_status(conn: &Connection, controller: &ImportController) -> Result<ImportStatus> {
    Ok(ImportStatus {
        in_progress: controller.is_in_progress(),
        progress: controller.snapshot(),
        last_mtg: latest_run(conn, Game::Mtg)?,
        last_pokemon: latest_run(conn, Game::Pokemon)?,
    })
}

/// Generates a new UUID for an import run.
pub fn new_import_id() -> String {
    Uuid::new_v4().to_string()
}

/// Batched upsert of prices extracted during bulk import.
pub fn persist_prices(db: &Database, prices: &[Price]) -> Result<()> {
    if prices.is_empty() {
        return Ok(());
    }
    let mut conn = db.connect()?;
    for chunk in prices.chunks(BATCH_SIZE) {
        let tx = conn.transaction()?;
        for price in chunk {
            pricing::upsert(&tx, price)?;
        }
        tx.commit()?;
    }
    Ok(())
}
