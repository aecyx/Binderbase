// SPDX-License-Identifier: AGPL-3.0-or-later
//! Pricing commands — on-demand refresh of card prices.

use crate::collection;
use crate::core::{CardId, Game, Result};
use crate::games;
use crate::pricing::{self, Price};
use crate::settings;
use crate::storage::Database;
use serde::Serialize;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};

use super::AppState;

/// Refresh prices for a single card. Hits the live game API, upserts the
/// results into the local cache, and returns the fresh prices.
#[tauri::command]
pub async fn pricing_refresh(
    state: State<'_, AppState>,
    game: Game,
    card_id: String,
) -> Result<Vec<Price>> {
    let cid = CardId(card_id);
    let api_key = settings::get_ptcgapi_key(state.secrets.as_ref())
        .ok()
        .flatten();
    let (_card, prices) = games::fetch_card_with_prices(game, &cid, api_key.as_deref()).await?;

    state.with_conn(|conn| {
        for p in &prices {
            pricing::upsert(conn, p)?;
        }
        Ok(())
    })?;

    // Return the freshly persisted rows so `fetched_at` is populated.
    state.with_conn(|conn| pricing::get_cached(conn, game, &cid))
}

/// Progress payload emitted once per card during `pricing_refresh_collection`.
#[derive(Debug, Clone, Serialize)]
pub struct RefreshProgress {
    pub done: u32,
    pub total: u32,
    pub card_id: String,
    pub game: Game,
    pub ok: bool,
    pub error: Option<String>,
}

/// Refresh prices for every card in the user's collection (optionally
/// filtered by game). Emits `pricing:refresh:progress` events as it goes.
///
/// Rate-limits: ≤10 requests/second for Scryfall, best-effort for PTCG.
#[tauri::command]
pub async fn pricing_refresh_collection(
    state: State<'_, AppState>,
    app: AppHandle,
    game: Option<Game>,
) -> Result<()> {
    let api_key = settings::get_ptcgapi_key(state.secrets.as_ref())
        .ok()
        .flatten();

    // Collect distinct (game, card_id) pairs from the collection.
    let pairs: Vec<(Game, CardId)> = state.with_conn(|conn| {
        let entries = collection::list(conn, game)?;
        let mut seen = std::collections::HashSet::new();
        let mut out = Vec::new();
        for e in entries {
            if seen.insert((e.game, e.card_id.clone())) {
                out.push((e.game, e.card_id));
            }
        }
        Ok(out)
    })?;

    if pairs.is_empty() {
        return Ok(());
    }

    let total = pairs.len() as u32;
    let db = Database::at(state.db.path());
    let api_key = api_key.map(Arc::from);

    tauri::async_runtime::spawn(async move {
        for (done, (g, cid)) in (0_u32..).zip(pairs.iter()) {
            let done = done + 1;
            let result = games::fetch_card_with_prices(*g, cid, api_key.as_deref()).await;

            let (ok, error) = match result {
                Ok((_card, prices)) => {
                    let upsert_result = db.with_connection(|conn| {
                        for p in &prices {
                            pricing::upsert(conn, p)?;
                        }
                        Ok(())
                    });
                    match upsert_result {
                        Ok(()) => (true, None),
                        Err(e) => (false, Some(e.to_string())),
                    }
                }
                Err(e) => (false, Some(e.to_string())),
            };

            let progress = RefreshProgress {
                done,
                total,
                card_id: cid.0.clone(),
                game: *g,
                ok,
                error,
            };
            let _ = app.emit("pricing:refresh:progress", &progress);

            // Rate-limit: ~100 ms between requests (≤10 rps).
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    });

    Ok(())
}
