// SPDX-License-Identifier: AGPL-3.0-or-later
//! Import controller and progress reporting.
//!
//! [`ImportController`] enforces single-import-at-a-time semantics and holds
//! the cancellation flag. [`ProgressSink`] abstracts where progress events go
//! (Tauri events in production, spy vec in tests).

use crate::core::Game;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};

use super::PROGRESS_EVENT;

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
            tracing::warn!(error = %e, "emit {PROGRESS_EVENT} failed");
        }
    }
}
