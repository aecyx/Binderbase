// SPDX-License-Identifier: AGPL-3.0-or-later
//! Tauri commands — the thin bridge between the frontend and the Rust core.
//!
//! Keep this file a thin dispatcher: parse inputs, call into domain modules,
//! map the result into the Tauri `Result` shape. Business logic belongs in
//! the domain modules (core, games, collection, pricing, scanning).

use crate::collection::{self, CollectionEntry, NewEntry};
use crate::core::{Card, CardId, Error, Game, Result};
use crate::games;
use crate::pricing::{self, Price};
use crate::scanning::{self, ScanResult};
use crate::storage::Database;
use serde::Serialize;
use std::sync::Mutex;
use tauri::State;

/// App-wide state. We currently only carry the database handle; add more
/// fields as the app grows (e.g., HTTP client pool, background job handles).
pub struct AppState {
    pub db: Database,
    // Single serialized connection for write operations to keep logic simple
    // in 0.1; swap for a pool when concurrency actually matters.
    pub conn: Mutex<rusqlite::Connection>,
}

impl AppState {
    pub fn init() -> Result<Self> {
        let db = Database::in_user_data_dir()?;
        let conn = db.connect()?;
        Ok(Self {
            db,
            conn: Mutex::new(conn),
        })
    }

    fn with_conn<T, F>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&rusqlite::Connection) -> Result<T>,
    {
        let guard = self.conn.lock().map_err(|_| {
            Error::Internal("database lock poisoned — a previous query panicked".into())
        })?;
        f(&guard)
    }
}

// ---------- app info ----------

#[derive(Debug, Serialize)]
pub struct AppInfo {
    pub name: &'static str,
    pub version: &'static str,
    pub db_path: String,
    pub supported_games: Vec<games::GameDescriptor>,
}

#[tauri::command]
pub fn app_info(state: State<'_, AppState>) -> AppInfo {
    AppInfo {
        name: "Binderbase",
        version: env!("CARGO_PKG_VERSION"),
        db_path: state.db.path().to_string_lossy().into_owned(),
        supported_games: Game::all().iter().copied().map(games::describe).collect(),
    }
}

// ---------- catalog ----------

#[tauri::command]
pub async fn fetch_card(game: Game, id: String) -> Result<Card> {
    games::fetch_card(game, &CardId(id)).await
}

// ---------- collection ----------

#[tauri::command]
pub fn collection_list(
    state: State<'_, AppState>,
    game: Option<Game>,
) -> Result<Vec<CollectionEntry>> {
    state.with_conn(|c| collection::list(c, game))
}

#[tauri::command]
pub fn collection_add(state: State<'_, AppState>, entry: NewEntry) -> Result<CollectionEntry> {
    state.with_conn(|c| collection::add(c, entry))
}

#[tauri::command]
pub fn collection_remove(state: State<'_, AppState>, entry_id: String) -> Result<()> {
    state.with_conn(|c| collection::remove(c, &entry_id))
}

// ---------- pricing ----------

#[tauri::command]
pub fn pricing_get_cached(
    state: State<'_, AppState>,
    game: Game,
    card_id: String,
) -> Result<Vec<Price>> {
    state.with_conn(|c| pricing::get_cached(c, game, &CardId(card_id)))
}

// ---------- scanning ----------

#[tauri::command]
pub fn scan_identify(image: Vec<u8>, game_hint: Option<Game>) -> Result<ScanResult> {
    scanning::identify(&image, game_hint)
}
