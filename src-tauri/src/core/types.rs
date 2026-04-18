// SPDX-License-Identifier: AGPL-3.0-or-later
//! Game-agnostic domain types shared by the catalog, collection, scanning,
//! and pricing subsystems.

use serde::{Deserialize, Serialize};
use std::fmt;

/// The trading-card games Binderbase supports.
///
/// Add new games here; then add a matching module under `src/games/`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Game {
    Mtg,
    Pokemon,
}

impl Game {
    pub const fn all() -> &'static [Game] {
        &[Game::Mtg, Game::Pokemon]
    }

    pub const fn slug(self) -> &'static str {
        match self {
            Game::Mtg => "mtg",
            Game::Pokemon => "pokemon",
        }
    }

    pub const fn display_name(self) -> &'static str {
        match self {
            Game::Mtg => "Magic: The Gathering",
            Game::Pokemon => "Pokémon TCG",
        }
    }

    pub fn from_slug(s: &str) -> Option<Game> {
        match s {
            "mtg" => Some(Game::Mtg),
            "pokemon" => Some(Game::Pokemon),
            _ => None,
        }
    }
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.display_name())
    }
}

/// Binderbase's internal card identifier.
///
/// For `Mtg` this maps to a Scryfall oracle id. For `Pokemon` this maps to
/// the Pokémon TCG API card id (e.g., `swsh4-25`). Keep the original source
/// id rather than minting our own — it makes re-fetching reference data easy.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CardId(pub String);

impl fmt::Display for CardId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A specific printing of a card (set + collector number).
///
/// Distinct from `CardId` — a card can have many printings with different
/// art, set symbols, and prices. Collection rows reference printings, not
/// cards, because condition and value vary by printing.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PrintingId(pub String);

impl fmt::Display for PrintingId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Canonical card-condition grades.
///
/// The scale follows TCGplayer's grading vocabulary, which is the de facto
/// North American standard. We store the enum (not a free-form string) so
/// filtering and pricing joins are exact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CardCondition {
    NearMint,
    LightlyPlayed,
    ModeratelyPlayed,
    HeavilyPlayed,
    Damaged,
}

impl CardCondition {
    pub const fn code(self) -> &'static str {
        match self {
            CardCondition::NearMint => "NM",
            CardCondition::LightlyPlayed => "LP",
            CardCondition::ModeratelyPlayed => "MP",
            CardCondition::HeavilyPlayed => "HP",
            CardCondition::Damaged => "DMG",
        }
    }
}

/// Shared card shape across games.
///
/// Per-game specifics (mana cost, pokemon type, HP, etc.) live in game-specific
/// extension tables keyed by `(game, card_id)`. This keeps the core model
/// small and game-additions cheap.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub game: Game,
    pub id: CardId,
    pub name: String,
    pub set_code: String,
    pub set_name: String,
    pub collector_number: String,
    /// Small thumbnail image URL (CDN). Full-size images are fetched on demand.
    pub image_url: Option<String>,
    /// Optional rarity string — content is game-defined.
    pub rarity: Option<String>,
}
