// SPDX-License-Identifier: AGPL-3.0-or-later
//! Collection commands — list, add, remove, CSV import/export.

use crate::collection::csv::{self, ImportPreview, ImportResult};
use crate::collection::{self, CollectionEntry, NewEntry};
use crate::core::{Game, Result};
use tauri::State;

use super::AppState;

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

/// Return the user's collection as a CSV string.
#[tauri::command]
pub fn collection_export_csv(state: State<'_, AppState>, game: Option<Game>) -> Result<String> {
    state.with_conn(|c| csv::export(c, game))
}

/// Dry-run a CSV import — parse, validate, and report what would happen.
#[tauri::command]
pub fn collection_import_preview(
    state: State<'_, AppState>,
    csv_text: String,
) -> Result<ImportPreview> {
    state.with_conn(|c| csv::import_preview(c, &csv_text))
}

/// Parse CSV text and insert valid rows into the collection.
#[tauri::command]
pub fn collection_import_apply(
    state: State<'_, AppState>,
    csv_text: String,
) -> Result<ImportResult> {
    state.with_conn(|c| csv::import_apply(c, &csv_text))
}
