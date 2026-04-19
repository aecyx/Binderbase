// SPDX-License-Identifier: AGPL-3.0-or-later
//! Scan index builder — downloads card thumbnails and computes perceptual
//! hashes to power the card identification pipeline.
//!
//! The builder is structured like the catalog bulk import: long-running async
//! task with progress events and cooperative cancellation.

use crate::core::{CardId, Error, Game, Result};
use crate::scanning::hashing;
use crate::storage::Database;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};

/// Tauri event name for index build progress.
pub const INDEX_PROGRESS_EVENT: &str = "scan:index:progress";

/// How many images to download concurrently per batch.
const CONCURRENT_DOWNLOADS: usize = 10;

// ---------- progress model ----------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexProgress {
    pub game: Game,
    pub processed: u64,
    pub total: u64,
    /// `"downloading"` | `"finished"` | `"cancelled"` | `"failed"`
    pub stage: String,
    pub message: Option<String>,
}

/// Status snapshot returned by `scan_index_status`.
#[derive(Debug, Clone, Serialize)]
pub struct IndexStatus {
    pub in_progress: bool,
    pub progress: Option<IndexProgress>,
    pub mtg_hashed: u64,
    pub mtg_total: u64,
    pub pokemon_hashed: u64,
    pub pokemon_total: u64,
}

// ---------- controller ----------

/// Shared process-wide handle to the current (or most recent) index build.
///
/// Structurally identical to `ImportController` — enforces single-operation
/// semantics with atomic flags.
pub struct IndexController {
    cancel_flag: AtomicBool,
    in_progress: AtomicBool,
    latest_progress: Mutex<Option<IndexProgress>>,
}

impl IndexController {
    pub fn new() -> Self {
        Self {
            cancel_flag: AtomicBool::new(false),
            in_progress: AtomicBool::new(false),
            latest_progress: Mutex::new(None),
        }
    }

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

    pub fn record_progress(&self, progress: IndexProgress) {
        if let Ok(mut slot) = self.latest_progress.lock() {
            *slot = Some(progress);
        }
    }

    pub fn snapshot(&self) -> Option<IndexProgress> {
        self.latest_progress.lock().ok().and_then(|g| g.clone())
    }
}

impl Default for IndexController {
    fn default() -> Self {
        Self::new()
    }
}

// ---------- internal types ----------

struct UnhashedCard {
    game: Game,
    card_id: CardId,
    image_url: String,
}

fn get_unhashed_cards(db: &Database, game: Game) -> Result<Vec<UnhashedCard>> {
    let conn = db.connect()?;
    let mut stmt = conn.prepare(
        "SELECT c.game, c.card_id, c.image_url
         FROM cards c
         LEFT JOIN card_hashes h ON c.game = h.game AND c.card_id = h.card_id
         WHERE c.game = ?1 AND c.image_url IS NOT NULL AND h.card_id IS NULL",
    )?;
    let rows = stmt.query_map(params![game.slug()], |r| {
        let game_slug: String = r.get(0)?;
        Ok(UnhashedCard {
            game: Game::from_slug(&game_slug).unwrap_or(Game::Mtg),
            card_id: CardId(r.get(1)?),
            image_url: r.get(2)?,
        })
    })?;
    rows.collect::<std::result::Result<Vec<_>, _>>()
        .map_err(Error::from)
}

// ---------- builder ----------

/// Build the scan index for a game by downloading card thumbnails and
/// computing perceptual hashes.
///
/// Only processes cards that don't already have a hash, so the operation is
/// resumable — cancel and restart will pick up where it left off.
pub async fn build_index(
    db: &Database,
    game: Game,
    controller: &IndexController,
    app: &AppHandle,
) -> Result<u64> {
    let cards = get_unhashed_cards(db, game)?;
    let total = cards.len() as u64;
    if total == 0 {
        emit_progress(app, controller, game, 0, 0, "finished", None);
        return Ok(0);
    }

    let client = reqwest::Client::builder()
        .user_agent(concat!("Binderbase/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(Error::from)?;

    let mut processed = 0u64;
    emit_progress(app, controller, game, 0, total, "downloading", None);

    for chunk in cards.chunks(CONCURRENT_DOWNLOADS) {
        if controller.is_cancelled() {
            return Err(Error::Internal("index build cancelled".into()));
        }

        // Download and hash concurrently within the chunk.
        let mut set = tokio::task::JoinSet::new();
        for card in chunk {
            let client = client.clone();
            let url = card.image_url.clone();
            let card_id = card.card_id.clone();
            let card_game = card.game;
            set.spawn(async move {
                let resp = client.get(&url).send().await.map_err(Error::from)?;
                let bytes = resp
                    .error_for_status()
                    .map_err(Error::from)?
                    .bytes()
                    .await
                    .map_err(Error::from)?;
                let hash = hashing::compute_dhash_from_bytes(&bytes)?;
                Ok::<_, Error>((card_game, card_id, hash))
            });
        }

        // Collect results from the concurrent tasks.
        let mut batch: Vec<(Game, CardId, [u8; hashing::HASH_SIZE])> = Vec::new();
        while let Some(result) = set.join_next().await {
            processed += 1;
            match result {
                Ok(Ok((g, id, hash))) => batch.push((g, id, hash)),
                Ok(Err(e)) => tracing::debug!("hash computation failed: {e}"),
                Err(e) => tracing::debug!("task join failed: {e}"),
            }
        }

        // Persist this batch.
        if !batch.is_empty() {
            persist_batch(db, &batch)?;
        }

        emit_progress(app, controller, game, processed, total, "downloading", None);

        // Small delay between chunks to be respectful of CDN rate limits.
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }

    emit_progress(app, controller, game, processed, total, "finished", None);
    Ok(processed)
}

fn persist_batch(db: &Database, batch: &[(Game, CardId, [u8; hashing::HASH_SIZE])]) -> Result<()> {
    let mut conn = db.connect()?;
    let tx = conn.transaction()?;
    for (game, card_id, hash) in batch {
        hashing::store_hash(&tx, *game, card_id, hash)?;
    }
    tx.commit()?;
    Ok(())
}

fn emit_progress(
    app: &AppHandle,
    controller: &IndexController,
    game: Game,
    processed: u64,
    total: u64,
    stage: &str,
    message: Option<String>,
) {
    let progress = IndexProgress {
        game,
        processed,
        total,
        stage: stage.into(),
        message,
    };
    controller.record_progress(progress.clone());
    if let Err(e) = app.emit(INDEX_PROGRESS_EVENT, &progress) {
        tracing::warn!(error = %e, "emit {INDEX_PROGRESS_EVENT} failed");
    }
}
