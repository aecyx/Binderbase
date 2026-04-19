// SPDX-License-Identifier: AGPL-3.0-or-later
//! HTTP data sources for bulk catalog import.
//!
//! Scryfall (MTG) and PTCGAPI (Pokémon) wire types live here, along with the
//! [`BulkSource`] trait that lets tests swap in canned data without a network.

use crate::core::{Card, CardId, Error, Game, Result};
use async_trait::async_trait;
use serde::Deserialize;

use super::{PTCGAPI_BASE, PTCGAPI_PAGE_SIZE, SCRYFALL_BULK_INDEX_URL, USER_AGENT};

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
}

#[derive(Debug, Deserialize)]
struct ScryfallImageUris {
    small: Option<String>,
    normal: Option<String>,
}

impl ScryfallCard {
    fn into_card(self) -> Card {
        Card {
            game: Game::Mtg,
            id: CardId(self.id),
            name: self.name,
            set_code: self.set,
            set_name: self.set_name,
            collector_number: self.collector_number,
            image_url: self.image_uris.and_then(|u| u.small.or(u.normal)),
            rarity: self.rarity,
        }
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
        Card {
            game: Game::Pokemon,
            id: CardId(self.id),
            name: self.name,
            set_code: self.set.id,
            set_name: self.set.name,
            collector_number: self.number,
            image_url: self.images.and_then(|i| i.small.or(i.large)),
            rarity: self.rarity,
        }
    }
}

// ---------- trait ----------

/// HTTP layer behind a trait so tests can swap in a canned source without
/// spinning up a mock server.
#[async_trait]
pub trait BulkSource: Send + Sync {
    async fn fetch_mtg_cards(&self) -> Result<Vec<Card>>;
    async fn fetch_pokemon_page(
        &self,
        page: u32,
        api_key: Option<&str>,
    ) -> Result<(Vec<Card>, Option<u64>)>;
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
    async fn fetch_mtg_cards(&self) -> Result<Vec<Card>> {
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
        Ok(raw.into_iter().map(ScryfallCard::into_card).collect())
    }

    async fn fetch_pokemon_page(
        &self,
        page: u32,
        api_key: Option<&str>,
    ) -> Result<(Vec<Card>, Option<u64>)> {
        let url = format!("{PTCGAPI_BASE}/cards?page={page}&pageSize={PTCGAPI_PAGE_SIZE}");
        let mut req = self.client.get(url).header("Accept", "application/json");
        if let Some(k) = api_key {
            if !k.is_empty() {
                req = req.header("X-Api-Key", k);
            }
        }
        let body: PtcgPageResponse = req.send().await?.error_for_status()?.json().await?;
        let total = body.total_count;
        let cards = body.data.into_iter().map(PtcgCard::into_card).collect();
        Ok((cards, total))
    }
}
