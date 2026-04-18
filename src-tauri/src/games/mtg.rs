//! Magic: The Gathering adapter.
//!
//! Data source: Scryfall — https://scryfall.com/docs/api
//!
//! Rate limits: Scryfall asks callers to stay under ~10 rps and to send a
//! `User-Agent` identifying the app. We respect this via the shared HTTP
//! client.
//!
//! Strategy for later: prefer Scryfall bulk data (daily JSON dumps) for bulk
//! catalog + price refreshes; use the live API only for on-demand single-card
//! lookups.

use crate::core::{Card, CardId, Error, Game, Result};
use serde::Deserialize;

const SCRYFALL_BASE: &str = "https://api.scryfall.com";
const USER_AGENT: &str = concat!("Binderbase/", env!("CARGO_PKG_VERSION"));

/// Look up a single card by Scryfall id.
pub async fn fetch_card(id: &CardId) -> Result<Card> {
    let url = format!("{SCRYFALL_BASE}/cards/{}", urlencode(&id.0));
    let client = http_client()?;
    let resp = client
        .get(url)
        .header("Accept", "application/json")
        .send()
        .await?;

    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(Error::CardNotFound(id.0.clone()));
    }
    let resp = resp.error_for_status()?;
    let raw: ScryfallCard = resp.json().await?;
    Ok(raw.into_card())
}

fn http_client() -> Result<reqwest::Client> {
    reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .map_err(Error::from)
}

/// Minimal URL-path encoder for the one character that matters to us in this
/// path (forward slash). Full encoding is handled by `reqwest` for query
/// params; oracle/Scryfall ids are URL-safe UUIDs so this is mostly paranoia.
fn urlencode(s: &str) -> String {
    s.replace('/', "%2F")
}

#[derive(Debug, Deserialize)]
struct ScryfallCard {
    id: String,
    name: String,
    set: String,
    set_name: String,
    collector_number: String,
    rarity: Option<String>,
    image_uris: Option<ScryfallImageUris>,
}

#[derive(Debug, Deserialize)]
struct ScryfallImageUris {
    small: Option<String>,
    normal: Option<String>,
}

impl ScryfallCard {
    fn into_card(self) -> Card {
        let image_url = self.image_uris.and_then(|u| u.small.or(u.normal));
        Card {
            game: Game::Mtg,
            id: CardId(self.id),
            name: self.name,
            set_code: self.set,
            set_name: self.set_name,
            collector_number: self.collector_number,
            image_url,
            rarity: self.rarity,
        }
    }
}
