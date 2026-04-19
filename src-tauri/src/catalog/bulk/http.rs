// SPDX-License-Identifier: AGPL-3.0-or-later
//! HTTP data sources for bulk catalog import.
//!
//! Scryfall (MTG) and PTCGAPI (Pokémon) wire types live here, along with the
//! [`BulkSource`] trait that lets tests swap in canned data without a network.

use crate::core::{Card, CardId, Error, Game, Result};
use crate::pricing::Price;
use async_trait::async_trait;
use serde::Deserialize;

use super::{PTCGAPI_BASE, PTCGAPI_PAGE_SIZE, SCRYFALL_BULK_INDEX_URL, USER_AGENT};

// ---------- helpers ----------

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

/// Convert a floating-point dollar amount to integer cents.
fn f64_to_cents(f: f64) -> Option<u64> {
    let cents = (f * 100.0).round() as u64;
    if cents > 0 {
        Some(cents)
    } else {
        None
    }
}

// ---------- Scryfall wire types ----------

#[derive(Debug, Deserialize)]
struct ScryfallBulkIndex {
    data: Vec<ScryfallBulkItem>,
}

#[derive(Debug, Deserialize)]
struct ScryfallBulkItem {
    #[serde(rename = "type")]
    kind: String,
    download_uri: String,
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

/// Daily price snapshot from Scryfall. All values are dollar-formatted strings
/// (e.g. `"1.23"`) or `null`.
#[derive(Debug, Deserialize)]
struct ScryfallPrices {
    usd: Option<String>,
    usd_foil: Option<String>,
    eur: Option<String>,
    eur_foil: Option<String>,
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
                        fetched_at: String::new(), // set by DB on upsert
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

// ---------- PTCGAPI wire types ----------

#[derive(Debug, Deserialize)]
struct PtcgPageResponse {
    data: Vec<PtcgCard>,
    #[serde(rename = "totalCount", default)]
    total_count: Option<u64>,
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

/// TCGplayer price block from the Pokémon TCG API.
#[derive(Debug, Deserialize)]
struct PtcgTcgplayer {
    #[serde(default)]
    prices: Option<PtcgTcgplayerPrices>,
}

#[derive(Debug, Deserialize)]
struct PtcgTcgplayerPrices {
    normal: Option<PtcgPricePoint>,
    holofoil: Option<PtcgPricePoint>,
}

#[derive(Debug, Deserialize)]
struct PtcgPricePoint {
    market: Option<f64>,
}

impl PtcgCard {
    fn into_card_and_prices(self) -> (Card, Vec<Price>) {
        let card_id = CardId(self.id);
        let mut prices = Vec::new();

        if let Some(ref tcg) = self.tcgplayer {
            if let Some(ref tp) = tcg.prices {
                if let Some(cents) = tp
                    .normal
                    .as_ref()
                    .and_then(|p| p.market)
                    .and_then(f64_to_cents)
                {
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
                if let Some(cents) = tp
                    .holofoil
                    .as_ref()
                    .and_then(|p| p.market)
                    .and_then(f64_to_cents)
                {
                    prices.push(Price {
                        game: Game::Pokemon,
                        card_id: card_id.clone(),
                        currency: "usd".into(),
                        source: "tcgplayer".into(),
                        cents,
                        foil: true,
                        fetched_at: String::new(),
                    });
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

// ---------- trait ----------

/// HTTP layer behind a trait so tests can swap in a canned source without
/// spinning up a mock server.
#[async_trait]
pub trait BulkSource: Send + Sync {
    async fn fetch_mtg_cards(&self) -> Result<(Vec<Card>, Vec<Price>)>;
    async fn fetch_pokemon_page(
        &self,
        page: u32,
        api_key: Option<&str>,
    ) -> Result<(Vec<Card>, Vec<Price>, Option<u64>)>;
}

// ---------- production source ----------

/// Production HTTP source — real network.
pub struct HttpBulkSource {
    client: reqwest::Client,
}

impl HttpBulkSource {
    pub fn new() -> Result<Self> {
        let client = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .map_err(Error::from)?;
        Ok(Self { client })
    }
}

#[async_trait]
impl BulkSource for HttpBulkSource {
    async fn fetch_mtg_cards(&self) -> Result<(Vec<Card>, Vec<Price>)> {
        let index: ScryfallBulkIndex = self
            .client
            .get(SCRYFALL_BULK_INDEX_URL)
            .header("Accept", "application/json")
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let default_cards = index
            .data
            .iter()
            .find(|d| d.kind == "default_cards")
            .ok_or_else(|| {
                Error::Internal("no default_cards entry in Scryfall bulk index".into())
            })?;

        let bytes = self
            .client
            .get(&default_cards.download_uri)
            .header("Accept", "application/json")
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?;

        let raw: Vec<ScryfallCard> = serde_json::from_slice(&bytes)?;
        let mut cards = Vec::with_capacity(raw.len());
        let mut prices = Vec::new();
        for sc in raw {
            let (card, card_prices) = sc.into_card_and_prices();
            cards.push(card);
            prices.extend(card_prices);
        }
        Ok((cards, prices))
    }

    async fn fetch_pokemon_page(
        &self,
        page: u32,
        api_key: Option<&str>,
    ) -> Result<(Vec<Card>, Vec<Price>, Option<u64>)> {
        let url = format!("{PTCGAPI_BASE}/cards?page={page}&pageSize={PTCGAPI_PAGE_SIZE}");
        let mut req = self.client.get(url).header("Accept", "application/json");
        if let Some(k) = api_key {
            if !k.is_empty() {
                req = req.header("X-Api-Key", k);
            }
        }
        let body: PtcgPageResponse = req.send().await?.error_for_status()?.json().await?;
        let total = body.total_count;
        let mut cards = Vec::with_capacity(body.data.len());
        let mut prices = Vec::new();
        for pc in body.data {
            let (card, card_prices) = pc.into_card_and_prices();
            cards.push(card);
            prices.extend(card_prices);
        }
        Ok((cards, prices, total))
    }
}
