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

use crate::catalog;
use crate::core::{Card, CardId, Error, Game, Result};
use crate::settings;
use crate::storage::Database;
use async_trait::async_trait;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

// ---------- constants ----------

const SCRYFALL_BULK_INDEX_URL: &str = "https://api.scryfall.com/bulk-data";
const PTCGAPI_BASE: &str = "https://api.pokemontcg.io/v2";
const USER_AGENT: &str = concat!("Binderbase/", env!("CARGO_PKG_VERSION"));

/// Tauri event name the UI subscribes to via `listen()`.
pub const PROGRESS_EVENT: &str = "catalog:import:progress";

/// How many cards to upsert per DB transaction. Tuned for 1000-card batches:
/// small enough to let cancellation respond within a second or so, large
/// enough to keep the per-batch transaction overhead under control.
const BATCH_SIZE: usize = 1000;

/// Max page size PTCGAPI accepts. Keeping it at the ceiling minimizes round
/// trips and therefore rate-limit pressure.
const PTCGAPI_PAGE_SIZE: u32 = 250;

// ---------- progress model ----------

/// Snapshot of where an import currently is. Emitted frequently — keep fields
/// cheap to serialize.
///
/// `stage` is a stable short string the UI can branch on:
///   - "fetching_bulk_index" — Scryfall metadata roundtrip
///   - "downloading"         — pulling the bulk JSON / next PTCGAPI page
///   - "parsing"             — JSON → native types
///   - "importing"           — batched upserts in flight
///   - "finished"            — success terminal
///   - "cancelled"           — cancellation terminal
///   - "failed"              — error terminal (UI should also surface the err)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ImportProgress {
    pub game: Option<Game>,
    pub stage: String,
    pub processed: u64,
    pub total: Option<u64>,
    pub message: Option<String>,
}

/// Public status handed back by `catalog_import_status` — combines live
/// progress with a summary of the last completed run.
#[derive(Debug, Clone, Serialize)]
pub struct ImportStatus {
    pub in_progress: bool,
    pub progress: Option<ImportProgress>,
    pub last_mtg: Option<ImportRunSummary>,
    pub last_pokemon: Option<ImportRunSummary>,
}

/// Slice of a row from `catalog_imports`.
#[derive(Debug, Clone, Serialize)]
pub struct ImportRunSummary {
    pub game: Game,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub status: String, // 'running' | 'completed' | 'cancelled' | 'failed'
    pub cards_imported: u64,
    pub error_message: Option<String>,
}

// ---------- controller ----------

/// Shared process-wide handle to the current (or most recent) import.
///
/// Only one import runs at a time — [`ImportController::try_start`] enforces
/// this with an `AtomicBool` race that is safe to call from any thread.
///
/// Stored in `AppState` behind `Arc<ImportController>` so the command layer,
/// the background task, and the cancel command all see the same flags.
pub struct ImportController {
    cancel_flag: AtomicBool,
    in_progress: AtomicBool,
    latest_progress: Mutex<Option<ImportProgress>>,
}

impl ImportController {
    pub fn new() -> Self {
        Self {
            cancel_flag: AtomicBool::new(false),
            in_progress: AtomicBool::new(false),
            latest_progress: Mutex::new(None),
        }
    }

    /// Atomically flip `in_progress` from false→true. Returns `true` if the
    /// caller now owns the import slot; `false` if another import is already
    /// running.
    pub fn try_start(&self) -> bool {
        let acquired = self
            .in_progress
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok();
        if acquired {
            self.cancel_flag.store(false, Ordering::Release);
            if let Ok(mut slot) = self.latest_progress.lock() {
                *slot = None;
            }
        }
        acquired
    }

    /// Release the import slot. Called from the `finally` of the spawned task.
    pub fn finish(&self) {
        self.in_progress.store(false, Ordering::Release);
    }

    pub fn is_in_progress(&self) -> bool {
        self.in_progress.load(Ordering::Acquire)
    }

    pub fn cancel(&self) {
        self.cancel_flag.store(true, Ordering::Release);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancel_flag.load(Ordering::Acquire)
    }

    pub fn record_progress(&self, progress: ImportProgress) {
        if let Ok(mut slot) = self.latest_progress.lock() {
            *slot = Some(progress);
        }
    }

    pub fn snapshot(&self) -> Option<ImportProgress> {
        self.latest_progress.lock().ok().and_then(|g| g.clone())
    }
}

impl Default for ImportController {
    fn default() -> Self {
        Self::new()
    }
}

// ---------- progress sink ----------

/// Where progress updates go. Production fans out to both the controller
/// (for polling fallback) and a Tauri event (for push updates).
pub trait ProgressSink: Send + Sync {
    fn emit(&self, progress: ImportProgress);
}

/// Ties the Tauri runtime to the controller. Cloning `AppHandle` is cheap.
pub struct TauriProgressSink {
    app: AppHandle,
    controller: std::sync::Arc<ImportController>,
}

impl TauriProgressSink {
    pub fn new(app: AppHandle, controller: std::sync::Arc<ImportController>) -> Self {
        Self { app, controller }
    }
}

impl ProgressSink for TauriProgressSink {
    fn emit(&self, progress: ImportProgress) {
        self.controller.record_progress(progress.clone());
        if let Err(e) = self.app.emit(PROGRESS_EVENT, &progress) {
            // Emit failures are non-fatal — the poll-status command is still
            // a working escape hatch. Log and move on.
            tracing::warn!(error = %e, "emit {PROGRESS_EVENT} failed");
        }
    }
}

// ---------- HTTP layer (Scryfall / PTCGAPI wire format) ----------

#[derive(Debug, Deserialize)]
struct ScryfallBulkIndex {
    data: Vec<ScryfallBulkItem>,
}

#[derive(Debug, Deserialize)]
struct ScryfallBulkItem {
    #[serde(rename = "type")]
    kind: String,
    download_uri: String,
}

#[derive(Debug, Deserialize)]
struct ScryfallCard {
    id: String,
    name: String,
    set: String,
    set_name: String,
    collector_number: String,
    rarity: Option<String>,
    image_uris: Option<ScryfallImageUris>,
}

#[derive(Debug, Deserialize)]
struct ScryfallImageUris {
    small: Option<String>,
    normal: Option<String>,
}

impl ScryfallCard {
    fn into_card(self) -> Card {
        Card {
            game: Game::Mtg,
            id: CardId(self.id),
            name: self.name,
            set_code: self.set,
            set_name: self.set_name,
            collector_number: self.collector_number,
            image_url: self.image_uris.and_then(|u| u.small.or(u.normal)),
            rarity: self.rarity,
        }
    }
}

#[derive(Debug, Deserialize)]
struct PtcgPageResponse {
    data: Vec<PtcgCard>,
    #[serde(rename = "totalCount", default)]
    total_count: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct PtcgCard {
    id: String,
    name: String,
    number: String,
    rarity: Option<String>,
    set: PtcgSet,
    images: Option<PtcgImages>,
}

#[derive(Debug, Deserialize)]
struct PtcgSet {
    id: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct PtcgImages {
    small: Option<String>,
    large: Option<String>,
}

impl PtcgCard {
    fn into_card(self) -> Card {
        Card {
            game: Game::Pokemon,
            id: CardId(self.id),
            name: self.name,
            set_code: self.set.id,
            set_name: self.set.name,
            collector_number: self.number,
            image_url: self.images.and_then(|i| i.small.or(i.large)),
            rarity: self.rarity,
        }
    }
}

/// HTTP layer behind a trait so tests can swap in a canned source without
/// spinning up a mock server.
#[async_trait]
pub trait BulkSource: Send + Sync {
    async fn fetch_mtg_cards(&self) -> Result<Vec<Card>>;
    async fn fetch_pokemon_page(
        &self,
        page: u32,
        api_key: Option<&str>,
    ) -> Result<(Vec<Card>, Option<u64>)>;
}

/// Production HTTP source — real network.
pub struct HttpBulkSource {
    client: reqwest::Client,
}

impl HttpBulkSource {
    pub fn new() -> Result<Self> {
        let client = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .map_err(Error::from)?;
        Ok(Self { client })
    }
}

#[async_trait]
impl BulkSource for HttpBulkSource {
    async fn fetch_mtg_cards(&self) -> Result<Vec<Card>> {
        let index: ScryfallBulkIndex = self
            .client
            .get(SCRYFALL_BULK_INDEX_URL)
            .header("Accept", "application/json")
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let default_cards = index
            .data
            .iter()
            .find(|d| d.kind == "default_cards")
            .ok_or_else(|| {
                Error::Internal("no default_cards entry in Scryfall bulk index".into())
            })?;

        let bytes = self
            .client
            .get(&default_cards.download_uri)
            .header("Accept", "application/json")
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?;

        let raw: Vec<ScryfallCard> = serde_json::from_slice(&bytes)?;
        Ok(raw.into_iter().map(ScryfallCard::into_card).collect())
    }

    async fn fetch_pokemon_page(
        &self,
        page: u32,
        api_key: Option<&str>,
    ) -> Result<(Vec<Card>, Option<u64>)> {
        let url = format!("{PTCGAPI_BASE}/cards?page={page}&pageSize={PTCGAPI_PAGE_SIZE}");
        let mut req = self.client.get(url).header("Accept", "application/json");
        if let Some(k) = api_key {
            if !k.is_empty() {
                req = req.header("X-Api-Key", k);
            }
        }
        let body: PtcgPageResponse = req.send().await?.error_for_status()?.json().await?;
        let total = body.total_count;
        let cards = body.data.into_iter().map(PtcgCard::into_card).collect();
        Ok((cards, total))
    }
}

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
    let mtg_id = Uuid::new_v4().to_string();
    log_import_start(db, &mtg_id, Game::Mtg)?;
    let mtg_result = run_mtg_import(db, source, sink, controller).await;
    finalize_import_log(db, &mtg_id, &mtg_result, controller)?;
    if mtg_result.is_err() || controller.is_cancelled() {
        emit_terminal(sink, Some(Game::Mtg), controller, mtg_result.as_ref().err());
        return mtg_result.map(|_| ());
    }

    // Pokémon
    let poke_id = Uuid::new_v4().to_string();
    log_import_start(db, &poke_id, Game::Pokemon)?;
    let poke_result = run_pokemon_import(db, source, sink, controller, ptcgapi_key).await;
    finalize_import_log(db, &poke_id, &poke_result, controller)?;
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

    stamp_last_imported(&conn, Game::Pokemon)?;
    Ok(total_imported)
}

/// Batched upsert into `cards`. Pure — takes an explicit Vec so tests can
/// feed fixture data without an HTTP layer.
pub fn persist_cards(
    db: &Database,
    game: Game,
    cards: &[Card],
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

fn stamp_last_imported(conn: &Connection, game: Game) -> Result<()> {
    settings::set(
        conn,
        &settings::last_imported_at_key(game.slug()),
        &chrono::Utc::now().to_rfc3339(),
    )
}

// ---------- catalog_imports table helpers ----------

fn log_import_start(db: &Database, import_id: &str, game: Game) -> Result<()> {
    let conn = db.connect()?;
    conn.execute(
        "INSERT INTO catalog_imports (import_id, game, status) VALUES (?1, ?2, 'running')",
        params![import_id, game.slug()],
    )?;
    Ok(())
}

fn finalize_import_log(
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Card, CardId};

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
        events: Mutex<Vec<ImportProgress>>,
    }
    impl SpySink {
        fn new() -> Self {
            Self {
                events: Mutex::new(Vec::new()),
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
        // Cancel before calling, so the first cancellation check after the
        // initial emit fires before any batch commits.
        controller.cancel();

        let sink = SpySink::new();
        let err = persist_cards(&db, Game::Mtg, &cards, &sink, &controller).unwrap_err();
        assert!(matches!(err, Error::Internal(ref m) if m.contains("cancelled")));

        // Nothing committed — the cancel check is before the first tx.
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
        // A sink that flips the controller's cancel flag after N emits. This
        // exercises the "cancelled mid-import" path — rows from batches
        // committed before the flag was seen must survive.
        struct CancelAfterSink {
            after_n_emits: usize,
            controller: std::sync::Arc<ImportController>,
            seen: Mutex<usize>,
            inner: Mutex<Vec<ImportProgress>>,
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
        // Emit order for 2500 cards at BATCH_SIZE=1000:
        //   emit #1: initial (0/2500, no commit yet)
        //   commit batch 1 → emit #2 (1000/2500)
        //   commit batch 2 → emit #3 (2000/2500)
        // Cancel after emit #2, so batch 1 survives and batch 2 is aborted
        // before its commit attempt.
        let sink = CancelAfterSink {
            after_n_emits: 2,
            controller: controller.clone(),
            seen: Mutex::new(0),
            inner: Mutex::new(Vec::new()),
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

    #[async_trait]
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

        // Two full pages, then a short page → terminates.
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

    // ---- catalog_imports logging ----

    #[test]
    fn finalize_import_log_marks_completed_on_ok() {
        let dir = tempfile::TempDir::new().unwrap();
        let db = Database::at(dir.path().join("t.sqlite3"));
        let id = Uuid::new_v4().to_string();
        log_import_start(&db, &id, Game::Mtg).unwrap();
        finalize_import_log(&db, &id, &Ok(42), &ImportController::new()).unwrap();

        let conn = db.connect().unwrap();
        let summary = latest_run(&conn, Game::Mtg).unwrap().unwrap();
        assert_eq!(summary.status, "completed");
        assert_eq!(summary.cards_imported, 42);
        assert!(summary.finished_at.is_some());
    }

    #[test]
    fn finalize_import_log_marks_cancelled_when_flag_set() {
        let dir = tempfile::TempDir::new().unwrap();
        let db = Database::at(dir.path().join("t.sqlite3"));
        let id = Uuid::new_v4().to_string();
        log_import_start(&db, &id, Game::Pokemon).unwrap();

        let controller = ImportController::new();
        controller.cancel();
        let err: Result<u64> = Err(Error::Internal("import cancelled".into()));
        finalize_import_log(&db, &id, &err, &controller).unwrap();

        let conn = db.connect().unwrap();
        let summary = latest_run(&conn, Game::Pokemon).unwrap().unwrap();
        assert_eq!(summary.status, "cancelled");
        assert!(summary.error_message.unwrap().contains("cancelled"));
    }

    #[test]
    fn finalize_import_log_marks_failed_on_other_error() {
        let dir = tempfile::TempDir::new().unwrap();
        let db = Database::at(dir.path().join("t.sqlite3"));
        let id = Uuid::new_v4().to_string();
        log_import_start(&db, &id, Game::Mtg).unwrap();

        let err: Result<u64> = Err(Error::Network("boom".into()));
        finalize_import_log(&db, &id, &err, &ImportController::new()).unwrap();

        let conn = db.connect().unwrap();
        let summary = latest_run(&conn, Game::Mtg).unwrap().unwrap();
        assert_eq!(summary.status, "failed");
        assert!(summary.error_message.unwrap().contains("boom"));
    }

    #[test]
    fn current_status_reflects_no_runs() {
        let dir = tempfile::TempDir::new().unwrap();
        let db = Database::at(dir.path().join("t.sqlite3"));
        let conn = db.connect().unwrap();
        let controller = ImportController::new();

        let status = current_status(&conn, &controller).unwrap();
        assert!(!status.in_progress);
        assert!(status.progress.is_none());
        assert!(status.last_mtg.is_none());
        assert!(status.last_pokemon.is_none());
    }
}
