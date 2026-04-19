// SPDX-License-Identifier: AGPL-3.0-or-later
//! Bulk catalog import.
//!
//! Pulls the full card list for each supported game into the local `cards`
//! table so autocomplete, collection lookups, and offline browsing all work
//! without a network round-trip.
//!
//! Data sources:
//!
//! * **MTG / Scryfall** — single bulk download (`default_cards`, ~200 MB of
//!   JSON) linked from `GET /bulk-data`. One shot, no pagination.
//! * **Pokémon / PTCGAPI** — paginated REST (`GET /cards?pageSize=250`).
//!   An API key is optional; without one we're rate-limited and slow but
//!   functional.
//!
//! Design:
//!
//! * HTTP and persistence are separated. [`run_mtg_import`] and
//!   [`run_pokemon_import`] own HTTP + parse; [`persist_cards`] is a pure
//!   DB-batched upsert and is the real unit-test target.
//! * Progress is reported via a [`ProgressSink`] trait. Production wires this
//!   to a Tauri event; tests wire it to a `Vec` for assertion.
//! * Cancellation: each batch checks [`ImportController::is_cancelled`] and
//!   returns `Error::Internal("import cancelled")` cleanly. In-flight rows
//!   that were already committed stay committed — partial catalog is still
//!   more useful than no catalog.
//! * Logging: each run opens a `catalog_imports` row at start and updates it
//!   (`status`, `finished_at`, `cards_imported`, `error_message`) at the end.
//!   UI reads the newest row per game for the "last updated" display.

mod controller;
mod http;
mod persist;

// Re-export the public surface so external `use crate::catalog::bulk::*`
// paths keep working unchanged.
pub use controller::{
    ImportController, ImportProgress, ImportRunSummary, ImportStatus, ProgressSink,
    TauriProgressSink,
};
pub use http::{BulkSource, HttpBulkSource};
pub use persist::{current_status, latest_run, persist_cards};

use crate::catalog;
use crate::core::{Error, Game, Result};
use crate::storage::Database;

// ---------- constants ----------

const SCRYFALL_BULK_INDEX_URL: &str = "https://api.scryfall.com/bulk-data";
const PTCGAPI_BASE: &str = "https://api.pokemontcg.io/v2";
const USER_AGENT: &str = concat!("Binderbase/", env!("CARGO_PKG_VERSION"));

/// Tauri event name the UI subscribes to via `listen()`.
pub const PROGRESS_EVENT: &str = "catalog:import:progress";

/// How many cards to upsert per DB transaction.
const BATCH_SIZE: usize = 1000;

/// Max page size PTCGAPI accepts.
const PTCGAPI_PAGE_SIZE: u32 = 250;

// ---------- orchestrator ----------

/// Import everything — MTG first, then Pokémon. Called by the spawned task
/// in the command layer.
pub async fn run_import_all(
    db: &Database,
    source: &dyn BulkSource,
    sink: &dyn ProgressSink,
    controller: &ImportController,
    ptcgapi_key: Option<&str>,
) -> Result<()> {
    // MTG
    let mtg_id = persist::new_import_id();
    persist::log_import_start(db, &mtg_id, Game::Mtg)?;
    let mtg_result = run_mtg_import(db, source, sink, controller).await;
    persist::finalize_import_log(db, &mtg_id, &mtg_result, controller)?;
    if mtg_result.is_err() || controller.is_cancelled() {
        emit_terminal(sink, Some(Game::Mtg), controller, mtg_result.as_ref().err());
        return mtg_result.map(|_| ());
    }

    // Pokémon
    let poke_id = persist::new_import_id();
    persist::log_import_start(db, &poke_id, Game::Pokemon)?;
    let poke_result = run_pokemon_import(db, source, sink, controller, ptcgapi_key).await;
    persist::finalize_import_log(db, &poke_id, &poke_result, controller)?;
    emit_terminal(
        sink,
        Some(Game::Pokemon),
        controller,
        poke_result.as_ref().err(),
    );
    poke_result.map(|_| ())
}

/// Full MTG pipeline: fetch → parse → persist in batches.
pub async fn run_mtg_import(
    db: &Database,
    source: &dyn BulkSource,
    sink: &dyn ProgressSink,
    controller: &ImportController,
) -> Result<u64> {
    sink.emit(ImportProgress {
        game: Some(Game::Mtg),
        stage: "fetching_bulk_index".into(),
        processed: 0,
        total: None,
        message: Some("Fetching Scryfall bulk index".into()),
    });
    if controller.is_cancelled() {
        return Err(Error::Internal("import cancelled".into()));
    }

    sink.emit(ImportProgress {
        game: Some(Game::Mtg),
        stage: "downloading".into(),
        processed: 0,
        total: None,
        message: Some("Downloading MTG catalog (≈200 MB)".into()),
    });
    let cards = source.fetch_mtg_cards().await?;

    persist_cards(db, Game::Mtg, &cards, sink, controller)
}

/// Full Pokémon pipeline: paginate through PTCGAPI, persist each page's
/// cards as we go (no single 200 MB buffer to hold).
pub async fn run_pokemon_import(
    db: &Database,
    source: &dyn BulkSource,
    sink: &dyn ProgressSink,
    controller: &ImportController,
    api_key: Option<&str>,
) -> Result<u64> {
    sink.emit(ImportProgress {
        game: Some(Game::Pokemon),
        stage: "downloading".into(),
        processed: 0,
        total: None,
        message: Some("Fetching Pokémon TCG catalog (paginated)".into()),
    });

    let mut conn = db.connect()?;
    let mut total_imported = 0u64;
    let mut announced_total: Option<u64> = None;
    let mut page: u32 = 1;

    loop {
        if controller.is_cancelled() {
            return Err(Error::Internal("import cancelled".into()));
        }
        let (cards, maybe_total) = source.fetch_pokemon_page(page, api_key).await?;
        if cards.is_empty() {
            break;
        }
        if announced_total.is_none() {
            announced_total = maybe_total;
        }

        let tx = conn.transaction()?;
        for card in &cards {
            catalog::upsert(&tx, card)?;
        }
        tx.commit()?;
        total_imported += cards.len() as u64;

        sink.emit(ImportProgress {
            game: Some(Game::Pokemon),
            stage: "importing".into(),
            processed: total_imported,
            total: announced_total,
            message: None,
        });

        // Last page is typically < page size.
        if cards.len() < PTCGAPI_PAGE_SIZE as usize {
            break;
        }
        page += 1;
    }

    persist::stamp_last_imported(&conn, Game::Pokemon)?;
    Ok(total_imported)
}

// ---------- helpers ----------

fn emit_terminal(
    sink: &dyn ProgressSink,
    game: Option<Game>,
    controller: &ImportController,
    err: Option<&Error>,
) {
    let stage = match err {
        Some(_) if controller.is_cancelled() => "cancelled",
        Some(_) => "failed",
        None => "finished",
    };
    sink.emit(ImportProgress {
        game,
        stage: stage.into(),
        processed: 0,
        total: None,
        message: err.map(|e| e.to_string()),
    });
}

// =========================================================================
// Tests
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Card, CardId};
    use crate::settings;

    fn c(game: Game, id: &str, name: &str) -> Card {
        Card {
            game,
            id: CardId(id.into()),
            name: name.into(),
            set_code: "X".into(),
            set_name: "X".into(),
            collector_number: "1".into(),
            image_url: None,
            rarity: None,
        }
    }

    /// Collects every emitted progress event for later assertion.
    struct SpySink {
        events: std::sync::Mutex<Vec<ImportProgress>>,
    }
    impl SpySink {
        fn new() -> Self {
            Self {
                events: std::sync::Mutex::new(Vec::new()),
            }
        }
        fn events(&self) -> Vec<ImportProgress> {
            self.events.lock().unwrap().clone()
        }
    }
    impl ProgressSink for SpySink {
        fn emit(&self, progress: ImportProgress) {
            self.events.lock().unwrap().push(progress);
        }
    }

    // ---- ImportController ----

    #[test]
    fn try_start_is_exclusive() {
        let c = ImportController::new();
        assert!(c.try_start(), "first try_start should succeed");
        assert!(!c.try_start(), "second try_start while running should fail");
        c.finish();
        assert!(c.try_start(), "try_start after finish should succeed");
    }

    #[test]
    fn try_start_clears_stale_cancel_flag() {
        let c = ImportController::new();
        c.cancel();
        assert!(c.is_cancelled());
        c.finish();
        assert!(c.try_start(), "try_start acquires the slot");
        assert!(
            !c.is_cancelled(),
            "try_start must reset cancel_flag for the new run"
        );
    }

    #[test]
    fn snapshot_reflects_latest_recorded_progress() {
        let c = ImportController::new();
        assert!(c.snapshot().is_none());
        c.record_progress(ImportProgress {
            game: Some(Game::Mtg),
            stage: "importing".into(),
            processed: 10,
            total: Some(100),
            message: None,
        });
        let s = c.snapshot().unwrap();
        assert_eq!(s.processed, 10);
        assert_eq!(s.total, Some(100));
    }

    // ---- persist_cards ----

    #[test]
    fn persist_cards_writes_every_card_and_emits_progress() {
        let dir = tempfile::TempDir::new().unwrap();
        let db = Database::at(dir.path().join("t.sqlite3"));
        db.connect().unwrap(); // run migrations

        let cards: Vec<Card> = (0..2500)
            .map(|i| c(Game::Mtg, &format!("mtg-{i}"), &format!("Card {i}")))
            .collect();

        let sink = SpySink::new();
        let controller = ImportController::new();
        let written = persist_cards(&db, Game::Mtg, &cards, &sink, &controller).unwrap();
        assert_eq!(written, 2500);

        let conn = db.connect().unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM cards WHERE game = 'mtg'", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(count, 2500);

        // First event is the initial "starting at 0 of 2500"; remaining are
        // batch-completion updates. With BATCH_SIZE=1000 we expect 4 emits
        // total: init + three batches (1000, 1000, 500).
        let events = sink.events();
        assert_eq!(events.len(), 4, "events: {events:#?}");
        assert_eq!(events.first().unwrap().processed, 0);
        assert_eq!(events.last().unwrap().processed, 2500);
        assert_eq!(events.last().unwrap().total, Some(2500));
    }

    #[test]
    fn persist_cards_aborts_cleanly_when_pre_cancelled() {
        let dir = tempfile::TempDir::new().unwrap();
        let db = Database::at(dir.path().join("t.sqlite3"));
        db.connect().unwrap();

        let cards: Vec<Card> = (0..2500)
            .map(|i| c(Game::Mtg, &format!("mtg-{i}"), &format!("Card {i}")))
            .collect();

        let controller = ImportController::new();
        controller.cancel();

        let sink = SpySink::new();
        let err = persist_cards(&db, Game::Mtg, &cards, &sink, &controller).unwrap_err();
        assert!(matches!(err, Error::Internal(ref m) if m.contains("cancelled")));

        let conn = db.connect().unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM cards WHERE game = 'mtg'", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn persist_cards_preserves_batches_committed_before_mid_run_cancel() {
        struct CancelAfterSink {
            after_n_emits: usize,
            controller: std::sync::Arc<ImportController>,
            seen: std::sync::Mutex<usize>,
            inner: std::sync::Mutex<Vec<ImportProgress>>,
        }
        impl ProgressSink for CancelAfterSink {
            fn emit(&self, progress: ImportProgress) {
                self.inner.lock().unwrap().push(progress);
                let mut seen = self.seen.lock().unwrap();
                *seen += 1;
                if *seen == self.after_n_emits {
                    self.controller.cancel();
                }
            }
        }

        let dir = tempfile::TempDir::new().unwrap();
        let db = Database::at(dir.path().join("t.sqlite3"));
        db.connect().unwrap();

        let cards: Vec<Card> = (0..2500)
            .map(|i| c(Game::Mtg, &format!("mtg-{i}"), &format!("Card {i}")))
            .collect();

        let controller = std::sync::Arc::new(ImportController::new());
        let sink = CancelAfterSink {
            after_n_emits: 2,
            controller: controller.clone(),
            seen: std::sync::Mutex::new(0),
            inner: std::sync::Mutex::new(Vec::new()),
        };

        let err = persist_cards(&db, Game::Mtg, &cards, &sink, &controller).unwrap_err();
        assert!(matches!(err, Error::Internal(ref m) if m.contains("cancelled")));

        let conn = db.connect().unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM cards WHERE game = 'mtg'", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(
            count, 1000,
            "first batch should be committed before the mid-run cancel"
        );
    }

    #[test]
    fn persist_cards_stamps_last_imported_at() {
        let dir = tempfile::TempDir::new().unwrap();
        let db = Database::at(dir.path().join("t.sqlite3"));
        db.connect().unwrap();

        let cards = vec![c(Game::Mtg, "mtg-1", "A")];
        let sink = SpySink::new();
        let controller = ImportController::new();
        persist_cards(&db, Game::Mtg, &cards, &sink, &controller).unwrap();

        let conn = db.connect().unwrap();
        let stamp = settings::get(&conn, &settings::last_imported_at_key("mtg"))
            .unwrap()
            .expect("last_imported_at set");
        assert!(
            stamp.len() >= 10,
            "expected ISO 8601 timestamp, got {stamp}"
        );
    }

    // ---- run_pokemon_import (via fake source) ----

    struct FakeSource {
        mtg: Vec<Card>,
        pokemon_pages: Vec<Vec<Card>>,
        pokemon_total: Option<u64>,
    }

    #[async_trait::async_trait]
    impl BulkSource for FakeSource {
        async fn fetch_mtg_cards(&self) -> Result<Vec<Card>> {
            Ok(self.mtg.clone())
        }
        async fn fetch_pokemon_page(
            &self,
            page: u32,
            _api_key: Option<&str>,
        ) -> Result<(Vec<Card>, Option<u64>)> {
            let idx = (page as usize).saturating_sub(1);
            let cards = self.pokemon_pages.get(idx).cloned().unwrap_or_default();
            Ok((cards, self.pokemon_total))
        }
    }

    #[tokio::test]
    async fn run_pokemon_import_paginates_until_short_page() {
        let dir = tempfile::TempDir::new().unwrap();
        let db = Database::at(dir.path().join("t.sqlite3"));
        db.connect().unwrap();

        let page1: Vec<Card> = (0..PTCGAPI_PAGE_SIZE)
            .map(|i| c(Game::Pokemon, &format!("poke-a-{i}"), &format!("P1 {i}")))
            .collect();
        let page2: Vec<Card> = (0..PTCGAPI_PAGE_SIZE)
            .map(|i| c(Game::Pokemon, &format!("poke-b-{i}"), &format!("P2 {i}")))
            .collect();
        let short_page: Vec<Card> = (0..50)
            .map(|i| c(Game::Pokemon, &format!("poke-tail-{i}"), &format!("PT {i}")))
            .collect();
        let source = FakeSource {
            mtg: vec![],
            pokemon_pages: vec![page1, page2, short_page],
            pokemon_total: Some(550),
        };

        let controller = ImportController::new();
        let sink = SpySink::new();
        let n = run_pokemon_import(&db, &source, &sink, &controller, None)
            .await
            .unwrap();
        assert_eq!(n, (PTCGAPI_PAGE_SIZE as u64) * 2 + 50);

        let conn = db.connect().unwrap();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM cards WHERE game = 'pokemon'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count as u64, (PTCGAPI_PAGE_SIZE as u64) * 2 + 50);
    }

    #[tokio::test]
    async fn run_pokemon_import_honors_cancellation_between_pages() {
        let dir = tempfile::TempDir::new().unwrap();
        let db = Database::at(dir.path().join("t.sqlite3"));
        db.connect().unwrap();

        let full_page: Vec<Card> = (0..PTCGAPI_PAGE_SIZE)
            .map(|i| c(Game::Pokemon, &format!("poke-{i}"), &format!("P {i}")))
            .collect();
        let source = FakeSource {
            mtg: vec![],
            pokemon_pages: vec![full_page.clone(); 10],
            pokemon_total: Some(10_000),
        };

        let controller = ImportController::new();
        controller.cancel();
        let sink = SpySink::new();
        let err = run_pokemon_import(&db, &source, &sink, &controller, None)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Internal(ref m) if m.contains("cancelled")));
    }
}
