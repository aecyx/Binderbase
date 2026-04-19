// SPDX-License-Identifier: AGPL-3.0-or-later
//! Scanning commands — card identification and hash index management.

use crate::core::{Error, Game, Result};
use crate::scanning::hashing;
use crate::scanning::index::{IndexProgress, IndexStatus};
use crate::scanning::{self, ScanResult};
use crate::storage::Database;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};

use super::AppState;

/// Identify a card from an uploaded image.
#[tauri::command]
pub fn scan_identify(
    state: State<'_, AppState>,
    image: Vec<u8>,
    game_hint: Option<Game>,
) -> Result<ScanResult> {
    state.with_conn(|c| scanning::identify(&image, game_hint, c))
}

/// Start building the scan hash index for a game.
#[tauri::command]
pub async fn scan_build_index(
    state: State<'_, AppState>,
    app: AppHandle,
    game: Game,
) -> Result<()> {
    if !state.index_controller.try_start() {
        return Err(Error::InvalidInput(
            "scan index build is already running".into(),
        ));
    }

    let controller = Arc::clone(&state.index_controller);
    let db = Database::at(state.db.path());

    tauri::async_runtime::spawn(async move {
        let result = scanning::index::build_index(&db, game, &controller, &app).await;
        if let Err(e) = &result {
            tracing::warn!(error = %e, "scan index build finished with error");
            let progress = IndexProgress {
                game,
                processed: 0,
                total: 0,
                stage: if controller.is_cancelled() {
                    "cancelled"
                } else {
                    "failed"
                }
                .into(),
                message: Some(e.to_string()),
            };
            controller.record_progress(progress.clone());
            if let Err(emit_err) = app.emit(scanning::index::INDEX_PROGRESS_EVENT, &progress) {
                tracing::warn!(error = %emit_err, "emit index progress failed");
            }
        }
        controller.finish();
    });

    Ok(())
}

/// Cancel a running scan index build.
#[tauri::command]
pub fn scan_build_index_cancel(state: State<'_, AppState>) -> Result<()> {
    state.index_controller.cancel();
    Ok(())
}

/// Get scan index status: coverage per game and in-progress state.
#[tauri::command]
pub fn scan_index_status(state: State<'_, AppState>) -> Result<IndexStatus> {
    state.with_conn(|c| {
        let (mtg_hashed, mtg_total) = hashing::index_coverage(c, Game::Mtg)?;
        let (pokemon_hashed, pokemon_total) = hashing::index_coverage(c, Game::Pokemon)?;
        Ok(IndexStatus {
            in_progress: state.index_controller.is_in_progress(),
            progress: state.index_controller.snapshot(),
            mtg_hashed,
            mtg_total,
            pokemon_hashed,
            pokemon_total,
        })
    })
}
