// SPDX-License-Identifier: AGPL-3.0-or-later
//! Binderbase — local-first TCG scanner and collection manager.
//!
//! Crate layout (see `docs/ARCHITECTURE.md`):
//!
//! - [`core`]: game-agnostic domain types and the app-wide `Error`.
//! - [`games`]: per-game catalog adapters (MTG via Scryfall, Pokémon via PTCGAPI).
//! - [`storage`]: SQLite connection and migrations.
//! - [`collection`]: CRUD over the user's owned cards.
//! - [`pricing`]: local price cache + lookup.
//! - [`scanning`]: image-to-card pipeline.
//! - [`commands`]: Tauri command surface exposed to the frontend.

pub mod collection;
pub mod commands;
pub mod core;
pub mod games;
pub mod pricing;
pub mod scanning;
pub mod storage;

use commands::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Logs go to stderr. Users can set RUST_LOG=binderbase=debug for more.
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("binderbase=info,warn")),
        )
        .with_writer(std::io::stderr)
        .try_init();

    let state = AppState::init().expect("failed to initialize Binderbase state");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::app_info,
            commands::fetch_card,
            commands::collection_list,
            commands::collection_add,
            commands::collection_remove,
            commands::pricing_get_cached,
            commands::scan_identify,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
