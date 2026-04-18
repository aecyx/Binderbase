// SPDX-License-Identifier: AGPL-3.0-or-later
//! Per-game adapters.
//!
//! Each submodule implements the catalog (card lookup) and, where supported,
//! pricing for one TCG. The top-level dispatch uses the `Game` enum to select
//! the right adapter.
//!
//! When adding a new game:
//! 1. Add a variant to `core::Game`.
//! 2. Add a submodule here.
//! 3. Extend `fetch_card` (and pricing dispatch, when added) below.
//!
//! The long-term plan is an object-safe `CardCatalog` trait behind
//! `Arc<dyn CardCatalog>`, but that needs `async_trait` (or the stabilized
//! async-fn-in-trait + dyn support). Until then, the free functions in this
//! module are the adapter surface.

pub mod mtg;
pub mod pokemon;

use crate::core::{Card, CardId, Game, Result};

/// Machine-readable description of a game's data + pricing sources.
///
/// Surfaced to the UI (About screen, credits) and useful for log lines.
#[derive(Debug, Clone, serde::Serialize)]
pub struct GameDescriptor {
    pub game: Game,
    pub data_source: &'static str,
    pub pricing_source: Option<&'static str>,
}

pub fn describe(game: Game) -> GameDescriptor {
    match game {
        Game::Mtg => GameDescriptor {
            game,
            data_source: "Scryfall (api.scryfall.com)",
            pricing_source: Some("Scryfall bulk prices"),
        },
        Game::Pokemon => GameDescriptor {
            game,
            data_source: "Pokémon TCG API (api.pokemontcg.io)",
            pricing_source: Some("Pokémon TCG API prices (TCGplayer-derived)"),
        },
    }
}

/// Dispatch a card lookup to the correct game adapter.
pub async fn fetch_card(game: Game, id: &CardId) -> Result<Card> {
    match game {
        Game::Mtg => mtg::fetch_card(id).await,
        Game::Pokemon => pokemon::fetch_card(id).await,
    }
}
