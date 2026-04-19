// SPDX-License-Identifier: AGPL-3.0-or-later
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
use crate::pricing::Price;
use serde::Deserialize;

const SCRYFALL_BASE: &str = "https://api.scryfall.com";
const USER_AGENT: &str = concat!("Binderbase/", env!("CARGO_PKG_VERSION"));

/// Look up a single card by Scryfall id.
pub async fn fetch_card(id: &CardId) -> Result<Card> {
    let (card, _prices) = fetch_card_with_prices(id).await?;
    Ok(card)
}

/// Look up a single card by Scryfall id, returning both the card and any
/// price data included in the Scryfall response.
pub async fn fetch_card_with_prices(id: &CardId) -> Result<(Card, Vec<Price>)> {
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
    Ok(raw.into_card_and_prices())
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
    #[serde(default)]
    prices: Option<ScryfallPrices>,
}

#[derive(Debug, Deserialize)]
struct ScryfallImageUris {
    small: Option<String>,
    normal: Option<String>,
}

/// Daily price snapshot from Scryfall. Dollar-formatted strings or `null`.
#[derive(Debug, Deserialize)]
struct ScryfallPrices {
    usd: Option<String>,
    usd_foil: Option<String>,
    eur: Option<String>,
    eur_foil: Option<String>,
}

/// Parse a dollar-string like `"1.23"` into integer cents.
fn parse_price(s: &str) -> Option<u64> {
    let f: f64 = s.parse().ok()?;
    let cents = (f * 100.0).round() as u64;
    if cents > 0 {
        Some(cents)
    } else {
        None
    }
}

impl ScryfallCard {
    fn into_card_and_prices(self) -> (Card, Vec<Price>) {
        let card_id = CardId(self.id);
        let mut prices = Vec::new();

        if let Some(ref p) = self.prices {
            let entries: &[(&Option<String>, &str, bool)] = &[
                (&p.usd, "usd", false),
                (&p.usd_foil, "usd", true),
                (&p.eur, "eur", false),
                (&p.eur_foil, "eur", true),
            ];
            for (raw, currency, foil) in entries {
                if let Some(cents) = raw.as_deref().and_then(parse_price) {
                    prices.push(Price {
                        game: Game::Mtg,
                        card_id: card_id.clone(),
                        currency: (*currency).into(),
                        source: "scryfall".into(),
                        cents,
                        foil: *foil,
                        fetched_at: String::new(),
                    });
                }
            }
        }

        let card = Card {
            game: Game::Mtg,
            id: card_id,
            name: self.name,
            set_code: self.set,
            set_name: self.set_name,
            collector_number: self.collector_number,
            image_url: self.image_uris.and_then(|u| u.small.or(u.normal)),
            rarity: self.rarity,
        };
        (card, prices)
    }
}
