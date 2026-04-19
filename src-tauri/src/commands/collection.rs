// SPDX-License-Identifier: AGPL-3.0-or-later
//! Collection commands — list, add, remove.

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
