// SPDX-License-Identifier: AGPL-3.0-or-later
//! Tauri commands — the thin bridge between the frontend and the Rust core.
//!
//! Keep each submodule a thin dispatcher: parse inputs, call into domain
//! modules, map the result into the Tauri `Result` shape. Business logic
//! belongs in the domain modules (core, games, collection, pricing, scanning).

pub mod catalog;
pub mod collection;
pub mod settings;

// Re-export every `#[tauri::command]` so `lib.rs` can reference them as
// `commands::<name>` without reaching into submodules.
pub use catalog::{
    catalog_get, catalog_import_cancel, catalog_import_start, catalog_import_status,
    catalog_search, fetch_card,
};
pub use collection::{collection_add, collection_list, collection_remove};
pub use settings::{settings_get_ptcgapi_key, settings_set_ptcgapi_key};

use crate::catalog::bulk::ImportController;
use crate::core::{Error, Game, Result};
use crate::games;
use crate::pricing::{self, Price};
use crate::scanning::{self, ScanResult};
use crate::settings::{KeyringSecrets, SecretStore};
use crate::storage::Database;
use serde::Serialize;
use std::sync::{Arc, Mutex};
use tauri::State;

use crate::core::CardId;

/// App-wide state. We currently only carry the database handle; add more
/// fields as the app grows (e.g., HTTP client pool, background job handles).
pub struct AppState {
    pub db: Database,
    pub conn: Mutex<rusqlite::Connection>,
    pub import_controller: Arc<ImportController>,
    pub secrets: Arc<dyn SecretStore>,
}

impl AppState {
    pub fn init() -> Result<Self> {
        let db = Database::in_user_data_dir()?;
        let conn = db.connect()?;
        Ok(Self {
            db,
            conn: Mutex::new(conn),
            import_controller: Arc::new(ImportController::new()),
            secrets: Arc::new(KeyringSecrets::new()),
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
