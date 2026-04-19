// SPDX-License-Identifier: AGPL-3.0-or-later
//! Local catalog — reads and writes for the `cards` table.
//!
//! Every card the app has ever heard of ends up here. The catalog is the
//! authoritative local source for card metadata (name, set, collector number,
//! image URL, rarity). It is populated two ways:
//!
//! 1. **Fetch-on-demand.** Any successful `games::fetch_card` result is
//!    upserted here via [`commands::fetch_card`], so repeat lookups skip the
//!    network.
//! 2. **Bulk import.** A user-initiated "Update catalog" action (Phase 2 of
//!    the 1.0 roadmap) will ingest Scryfall bulk data and paginate PTCGAPI.
//!    Not yet implemented.
//!
//! Callers pass a `&Connection` so this module stays I/O-free and
//! transaction-friendly. The command layer owns the locking.

use crate::core::{Card, CardId, Error, Game, Result};
use rusqlite::{params, Connection};

/// Insert or replace a catalog row. Idempotent on `(game, card_id)`.
///
/// `updated_at` is explicitly refreshed on conflict so repeat upserts
/// reflect the latest fetch time, not the original insert time.
pub fn upsert(conn: &Connection, card: &Card) -> Result<()> {
    conn.execute(
        "INSERT INTO cards
            (game, card_id, name, set_code, set_name, collector_number,
             rarity, image_url, updated_at)
         VALUES
            (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, datetime('now'))
         ON CONFLICT (game, card_id) DO UPDATE SET
            name             = excluded.name,
            set_code         = excluded.set_code,
            set_name         = excluded.set_name,
            collector_number = excluded.collector_number,
            rarity           = excluded.rarity,
            image_url        = excluded.image_url,
            updated_at       = datetime('now')",
        params![
            card.game.slug(),
            card.id.0,
            card.name,
            card.set_code,
            card.set_name,
            card.collector_number,
            card.rarity,
            card.image_url,
        ],
    )?;
    Ok(())
}

/// Fetch a single card by `(game, card_id)`.
///
/// `Ok(None)` means the catalog hasn't heard of this card yet — the caller
/// decides whether to fall through to a live lookup (`commands::fetch_card`
/// does; bare catalog reads don't).
pub fn get(conn: &Connection, game: Game, card_id: &CardId) -> Result<Option<Card>> {
    let mut stmt = conn.prepare_cached(
        "SELECT game, card_id, name, set_code, set_name, collector_number,
                image_url, rarity
         FROM cards
         WHERE game = ?1 AND card_id = ?2",
    )?;

    let mut rows = stmt.query_map(params![game.slug(), &card_id.0], row_to_card)?;
    match rows.next() {
        Some(r) => Ok(Some(r?)),
        None => Ok(None),
    }
}

/// Substring-search cards by name. Case-insensitive; ordered alphabetically;
/// capped at `limit`.
///
/// Empty/whitespace-only queries return an empty Vec — autocomplete UIs
/// call this on every keystroke and should not dump the whole catalog on
/// the first tap of the backspace key.
pub fn search(conn: &Connection, game: Option<Game>, query: &str, limit: u32) -> Result<Vec<Card>> {
    let q = query.trim();
    if q.is_empty() {
        return Ok(Vec::new());
    }
    if limit == 0 {
        return Err(Error::InvalidInput("limit must be > 0".into()));
    }

    // Escape LIKE metacharacters so user input matches literally. `\` is
    // declared as the escape character in the SQL below.
    let escaped = q
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");
    let pattern = format!("%{escaped}%");
    let limit_i = limit as i64;

    // Two SQL variants rather than one-with-NULL-filter, so the query
    // planner can use the (game, ...) indexes when a game filter is set.
    match game {
        Some(g) => {
            let mut stmt = conn.prepare(
                "SELECT game, card_id, name, set_code, set_name, collector_number,
                        image_url, rarity
                 FROM cards
                 WHERE game = ?1
                   AND name LIKE ?2 ESCAPE '\\' COLLATE NOCASE
                 ORDER BY name
                 LIMIT ?3",
            )?;
            let rows = stmt
                .query_map(params![g.slug(), pattern, limit_i], row_to_card)?
                .collect::<std::result::Result<Vec<_>, _>>()?;
            Ok(rows)
        }
        None => {
            let mut stmt = conn.prepare(
                "SELECT game, card_id, name, set_code, set_name, collector_number,
                        image_url, rarity
                 FROM cards
                 WHERE name LIKE ?1 ESCAPE '\\' COLLATE NOCASE
                 ORDER BY name
                 LIMIT ?2",
            )?;
            let rows = stmt
                .query_map(params![pattern, limit_i], row_to_card)?
                .collect::<std::result::Result<Vec<_>, _>>()?;
            Ok(rows)
        }
    }
}

fn row_to_card(row: &rusqlite::Row) -> rusqlite::Result<Card> {
    let game_slug: String = row.get(0)?;
    // Unknown game slugs fall back to MTG rather than panicking. Schema
    // CHECK/FK should prevent this, but defensive default avoids crashing
    // the UI on a malformed row.
    Ok(Card {
        game: Game::from_slug(&game_slug).unwrap_or(Game::Mtg),
        id: CardId(row.get(1)?),
        name: row.get(2)?,
        set_code: row.get(3)?,
        set_name: row.get(4)?,
        collector_number: row.get(5)?,
        image_url: row.get(6)?,
        rarity: row.get(7)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::test_support::memory_conn;

    fn sample_card() -> Card {
        Card {
            game: Game::Mtg,
            id: CardId("abc-123".into()),
            name: "Black Lotus".into(),
            set_code: "LEA".into(),
            set_name: "Limited Edition Alpha".into(),
            collector_number: "232".into(),
            image_url: Some("https://example.invalid/lotus.jpg".into()),
            rarity: Some("rare".into()),
        }
    }

    fn bare_card(game: Game, id: &str, name: &str) -> Card {
        Card {
            game,
            id: CardId(id.into()),
            name: name.into(),
            set_code: "X".into(),
            set_name: "X".into(),
            collector_number: "1".into(),
            image_url: None,
            rarity: None,
        }
    }

    #[test]
    fn upsert_then_get_round_trips() {
        let conn = memory_conn();
        let card = sample_card();
        upsert(&conn, &card).unwrap();

        let got = get(&conn, Game::Mtg, &card.id).unwrap().expect("present");
        assert_eq!(got.name, card.name);
        assert_eq!(got.set_code, card.set_code);
        assert_eq!(got.rarity, card.rarity);
        assert_eq!(got.image_url, card.image_url);
    }

    #[test]
    fn get_returns_none_for_unknown_card() {
        let conn = memory_conn();
        let missing = get(&conn, Game::Mtg, &CardId("does-not-exist".into())).unwrap();
        assert!(missing.is_none());
    }

    #[test]
    fn upsert_overwrites_changed_fields_and_keeps_one_row() {
        let conn = memory_conn();
        let mut card = sample_card();
        upsert(&conn, &card).unwrap();

        card.rarity = Some("mythic".into());
        card.image_url = None;
        upsert(&conn, &card).unwrap();

        let got = get(&conn, Game::Mtg, &card.id).unwrap().unwrap();
        assert_eq!(got.rarity.as_deref(), Some("mythic"));
        assert!(got.image_url.is_none());

        let count: u32 = conn
            .query_row("SELECT COUNT(*) FROM cards", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1, "upsert on existing PK should not duplicate rows");
    }

    #[test]
    fn search_is_case_insensitive_substring() {
        let conn = memory_conn();
        upsert(&conn, &bare_card(Game::Mtg, "1", "Black Lotus")).unwrap();
        upsert(&conn, &bare_card(Game::Mtg, "2", "Lotus Petal")).unwrap();
        upsert(&conn, &bare_card(Game::Mtg, "3", "Mox Pearl")).unwrap();

        let hits = search(&conn, None, "lotus", 10).unwrap();
        let names: Vec<_> = hits.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, vec!["Black Lotus", "Lotus Petal"]);
    }

    #[test]
    fn search_filters_by_game() {
        let conn = memory_conn();
        upsert(&conn, &bare_card(Game::Mtg, "1", "Charizard")).unwrap();
        upsert(&conn, &bare_card(Game::Pokemon, "2", "Charizard")).unwrap();

        let only_mtg = search(&conn, Some(Game::Mtg), "charizard", 10).unwrap();
        assert_eq!(only_mtg.len(), 1);
        assert_eq!(only_mtg[0].game, Game::Mtg);

        let both = search(&conn, None, "charizard", 10).unwrap();
        assert_eq!(both.len(), 2);
    }

    #[test]
    fn search_empty_query_returns_empty() {
        let conn = memory_conn();
        upsert(&conn, &sample_card()).unwrap();
        assert!(search(&conn, None, "", 10).unwrap().is_empty());
        assert!(search(&conn, None, "   ", 10).unwrap().is_empty());
    }

    #[test]
    fn search_escapes_like_metacharacters() {
        // A naive LIKE would match everything on a query of '%'. With the
        // escape clause, it only matches names containing a literal percent.
        let conn = memory_conn();
        upsert(&conn, &bare_card(Game::Mtg, "1", "50% Off")).unwrap();
        upsert(&conn, &bare_card(Game::Mtg, "2", "Anything")).unwrap();

        let hits = search(&conn, None, "50%", 10).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].name, "50% Off");
    }

    #[test]
    fn search_respects_limit() {
        let conn = memory_conn();
        for i in 0..5 {
            upsert(
                &conn,
                &bare_card(Game::Mtg, &format!("c{i}"), &format!("Island {i}")),
            )
            .unwrap();
        }
        let hits = search(&conn, None, "island", 3).unwrap();
        assert_eq!(hits.len(), 3);
    }

    #[test]
    fn search_zero_limit_is_invalid_input() {
        let conn = memory_conn();
        let err = search(&conn, None, "x", 0).unwrap_err();
        assert!(matches!(err, Error::InvalidInput(_)));
    }
}
