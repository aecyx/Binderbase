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
use crate::pricing::Price;
use serde::Deserialize;

const POKEMONTCG_BASE: &str = "https://api.pokemontcg.io/v2";
const USER_AGENT: &str = concat!("Binderbase/", env!("CARGO_PKG_VERSION"));

pub async fn fetch_card(id: &CardId) -> Result<Card> {
    let (card, _prices) = fetch_card_with_prices(id, None).await?;
    Ok(card)
}

/// Look up a single card, returning both the card and any TCGplayer-derived
/// price data included in the response.
pub async fn fetch_card_with_prices(
    id: &CardId,
    api_key: Option<&str>,
) -> Result<(Card, Vec<Price>)> {
    let url = format!("{POKEMONTCG_BASE}/cards/{}", urlencode(&id.0));
    let client = http_client()?;
    let mut req = client.get(url).header("Accept", "application/json");
    if let Some(key) = api_key {
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
    Ok(wrapper.data.into_card_and_prices())
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
    #[serde(default)]
    tcgplayer: Option<PtcgTcgplayer>,
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

#[derive(Debug, Deserialize)]
struct PtcgTcgplayer {
    prices: Option<PtcgPriceVariants>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PtcgPriceVariants {
    normal: Option<PtcgPriceFields>,
    holo_foil: Option<PtcgPriceFields>,
    reverse_holo_foil: Option<PtcgPriceFields>,
    #[serde(rename = "1stEditionHolofoil")]
    first_edition_holofoil: Option<PtcgPriceFields>,
}

#[derive(Debug, Deserialize)]
struct PtcgPriceFields {
    market: Option<f64>,
}

/// Convert dollars (f64) to integer cents.
fn f64_to_cents(f: f64) -> Option<u64> {
    let cents = (f * 100.0).round() as u64;
    if cents > 0 {
        Some(cents)
    } else {
        None
    }
}

impl PtcgCard {
    fn into_card_and_prices(self) -> (Card, Vec<Price>) {
        let card_id = CardId(self.id);
        let mut prices = Vec::new();

        if let Some(tp) = &self.tcgplayer {
            if let Some(ref variants) = tp.prices {
                // Non-foil entries
                if let Some(ref normal) = variants.normal {
                    if let Some(cents) = normal.market.and_then(f64_to_cents) {
                        prices.push(Price {
                            game: Game::Pokemon,
                            card_id: card_id.clone(),
                            currency: "usd".into(),
                            source: "tcgplayer".into(),
                            cents,
                            foil: false,
                            fetched_at: String::new(),
                        });
                    }
                }
                // Foil entries
                let foil_variants = [
                    &variants.holo_foil,
                    &variants.reverse_holo_foil,
                    &variants.first_edition_holofoil,
                ];
                // Take the first available foil market price
                for pf in foil_variants.iter().filter_map(|v| v.as_ref()) {
                    if let Some(cents) = pf.market.and_then(f64_to_cents) {
                        prices.push(Price {
                            game: Game::Pokemon,
                            card_id: card_id.clone(),
                            currency: "usd".into(),
                            source: "tcgplayer".into(),
                            cents,
                            foil: true,
                            fetched_at: String::new(),
                        });
                        break;
                    }
                }
            }
        }

        let card = Card {
            game: Game::Pokemon,
            id: card_id,
            name: self.name,
            set_code: self.set.id,
            set_name: self.set.name,
            collector_number: self.number,
            image_url: self.images.and_then(|i| i.small.or(i.large)),
            rarity: self.rarity,
        };
        (card, prices)
    }
}
