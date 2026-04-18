// SPDX-License-Identifier: AGPL-3.0-or-later
//! Pokémon TCG adapter.
//!
//! Data source: Pokémon TCG API — https://docs.pokemontcg.io/
//!
//! An API key is optional for low-volume usage but recommended (higher rate
//! limits). We read it from the `POKEMONTCG_API_KEY` environment variable at
//! runtime; users can set it in the app-data config for a persistent value.
//! Missing key = unauthenticated requests (still works for dev).

use crate::core::{Card, CardId, Error, Game, Result};
use serde::Deserialize;

const POKEMONTCG_BASE: &str = "https://api.pokemontcg.io/v2";
const USER_AGENT: &str = concat!("Binderbase/", env!("CARGO_PKG_VERSION"));

pub async fn fetch_card(id: &CardId) -> Result<Card> {
    let url = format!("{POKEMONTCG_BASE}/cards/{}", urlencode(&id.0));
    let client = http_client()?;
    let mut req = client.get(url).header("Accept", "application/json");
    if let Ok(key) = std::env::var("POKEMONTCG_API_KEY") {
        if !key.is_empty() {
            req = req.header("X-Api-Key", key);
        }
    }
    let resp = req.send().await?;

    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(Error::CardNotFound(id.0.clone()));
    }
    let resp = resp.error_for_status()?;
    let wrapper: PtcgResponse = resp.json().await?;
    Ok(wrapper.data.into_card())
}

fn http_client() -> Result<reqwest::Client> {
    reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .map_err(Error::from)
}

fn urlencode(s: &str) -> String {
    s.replace('/', "%2F")
}

#[derive(Debug, Deserialize)]
struct PtcgResponse {
    data: PtcgCard,
}

#[derive(Debug, Deserialize)]
struct PtcgCard {
    id: String,
    name: String,
    number: String,
    rarity: Option<String>,
    set: PtcgSet,
    images: Option<PtcgImages>,
}

#[derive(Debug, Deserialize)]
struct PtcgSet {
    id: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct PtcgImages {
    small: Option<String>,
    large: Option<String>,
}

impl PtcgCard {
    fn into_card(self) -> Card {
        let image_url = self.images.and_then(|i| i.small.or(i.large));
        Card {
            game: Game::Pokemon,
            id: CardId(self.id),
            name: self.name,
            set_code: self.set.id,
            set_name: self.set.name,
            collector_number: self.number,
            image_url,
            rarity: self.rarity,
        }
    }
}
