// SPDX-License-Identifier: AGPL-3.0-or-later
//! Settings commands — API-key management.

use crate::core::Result;
use crate::settings;
use tauri::State;

use super::AppState;

/// Read the stored Pokémon TCG API key (or `None` if not set).
#[tauri::command]
pub fn settings_get_ptcgapi_key(state: State<'_, AppState>) -> Result<Option<String>> {
    settings::get_ptcgapi_key(state.secrets.as_ref())
}

/// Store (or clear) the Pokémon TCG API key. Empty/whitespace → delete.
#[tauri::command]
pub fn settings_set_ptcgapi_key(state: State<'_, AppState>, value: String) -> Result<()> {
    settings::set_ptcgapi_key(state.secrets.as_ref(), &value)
}
