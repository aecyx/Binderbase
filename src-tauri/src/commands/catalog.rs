// SPDX-License-Identifier: AGPL-3.0-or-later
//! Catalog commands — local lookups, search, and bulk-import lifecycle.

use crate::catalog;
use crate::catalog::bulk::{self, HttpBulkSource, ImportStatus, TauriProgressSink};
use crate::core::{Card, CardId, Error, Game, Result};
use crate::games;
use crate::settings;
use crate::storage::Database;
use std::sync::Arc;
use tauri::{AppHandle, State};

use super::AppState;

/// Default page size for `catalog_search`.
const DEFAULT_CATALOG_SEARCH_LIMIT: u32 = 25;
/// Hard cap to protect the UI thread from a runaway request.
const MAX_CATALOG_SEARCH_LIMIT: u32 = 200;

/// Fetch a card, local-first.
///
/// Policy:
/// 1. Look up `(game, id)` in the local catalog. Hit → return immediately,
///    no network.
/// 2. Miss → hit the live game adapter (`games::fetch_card`).
/// 3. On a successful live fetch, upsert into the catalog so the next call
///    is a hit. Upsert failures are logged but do not fail the command —
///    the user still gets their card.
#[tauri::command]
pub async fn fetch_card(state: State<'_, AppState>, game: Game, id: String) -> Result<Card> {
    let card_id = CardId(id);

    if let Some(cached) = state.with_conn(|c| catalog::get(c, game, &card_id))? {
        return Ok(cached);
    }

    let card = games::fetch_card(game, &card_id).await?;

    if let Err(e) = state.with_conn(|c| catalog::upsert(c, &card)) {
        tracing::warn!(error = %e, game = ?game, card_id = %card.id.0,
            "catalog upsert failed after live fetch; user got card but cache is cold");
    }

    Ok(card)
}

/// Read a card straight from the catalog without a network fallthrough.
/// Returns `None` if the catalog hasn't heard of it yet.
#[tauri::command]
pub fn catalog_get(
    state: State<'_, AppState>,
    game: Game,
    card_id: String,
) -> Result<Option<Card>> {
    state.with_conn(|c| catalog::get(c, game, &CardId(card_id)))
}

/// Substring-search the local catalog. Used for autocomplete in the
/// "add to collection" flow. Clamps `limit` to `MAX_CATALOG_SEARCH_LIMIT`.
#[tauri::command]
pub fn catalog_search(
    state: State<'_, AppState>,
    game: Option<Game>,
    query: String,
    limit: Option<u32>,
) -> Result<Vec<Card>> {
    let effective_limit = limit
        .unwrap_or(DEFAULT_CATALOG_SEARCH_LIMIT)
        .min(MAX_CATALOG_SEARCH_LIMIT);
    state.with_conn(|c| catalog::search(c, game, &query, effective_limit))
}

/// Kick off a background bulk-import. Pass `game` to import only one game,
/// or `None` to import all supported games. Returns immediately — progress
/// is pushed via `catalog:import:progress` events.
#[tauri::command]
pub async fn catalog_import_start(
    state: State<'_, AppState>,
    app: AppHandle,
    game: Option<Game>,
) -> Result<()> {
    if !state.import_controller.try_start() {
        return Err(Error::InvalidInput(
            "a catalog import is already running".into(),
        ));
    }

    let controller = Arc::clone(&state.import_controller);
    let secrets = Arc::clone(&state.secrets);

    let db = Database::at(state.db.path());

    tauri::async_runtime::spawn(async move {
        let source = match HttpBulkSource::new() {
            Ok(s) => s,
            Err(e) => {
                tracing::error!(error = %e, "failed to create HTTP client for import");
                controller.finish();
                return;
            }
        };
        let sink = TauriProgressSink::new(app, Arc::clone(&controller));
        let api_key = settings::get_ptcgapi_key(secrets.as_ref()).ok().flatten();
        let result =
            bulk::run_import(&db, &source, &sink, &controller, api_key.as_deref(), game).await;
        if let Err(e) = &result {
            tracing::warn!(error = %e, "catalog import finished with error");
        }
        controller.finish();
    });

    Ok(())
}

/// Request cancellation of a running import. No-op if nothing is running.
#[tauri::command]
pub fn catalog_import_cancel(state: State<'_, AppState>) -> Result<()> {
    state.import_controller.cancel();
    Ok(())
}

/// Poll the current import status (in-progress flag, latest progress
/// snapshot, last completed runs per game).
#[tauri::command]
pub fn catalog_import_status(state: State<'_, AppState>) -> Result<ImportStatus> {
    state.with_conn(|c| bulk::current_status(c, &state.import_controller))
}
