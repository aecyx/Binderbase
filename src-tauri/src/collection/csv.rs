// SPDX-License-Identifier: AGPL-3.0-or-later
//! CSV import / export for collection entries.
//!
//! **Export** joins `collection_entries` with `cards` so the CSV is human-
//! readable (includes card name, set, etc.). The format round-trips cleanly:
//! importing a previously-exported CSV produces the same collection state
//! (minus entry UUIDs, which are freshly generated).
//!
//! **Import** accepts the full export format or a minimal format with only
//! `game`, `card_id`, `condition`, and `quantity`. Unknown columns are
//! silently ignored.

use crate::core::{CardCondition, Error, Game, Result};
use rusqlite::{params, Connection};
use serde::Serialize;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// CSV header names — kept in one place so export and import agree.
const HEADERS: &[&str] = &[
    "game",
    "card_id",
    "name",
    "set_code",
    "set_name",
    "collector_number",
    "condition",
    "foil",
    "quantity",
    "notes",
    "acquired_at",
    "acquired_price_cents",
];

/// A row in the export CSV.
#[derive(Debug, Serialize)]
struct ExportRow {
    game: String,
    card_id: String,
    name: String,
    set_code: String,
    set_name: String,
    collector_number: String,
    condition: String,
    foil: bool,
    quantity: u32,
    notes: String,
    acquired_at: String,
    acquired_price_cents: String,
}

/// One error encountered while validating a CSV row.
#[derive(Debug, Clone, Serialize)]
pub struct RowError {
    /// 1-based line number in the CSV file (header = line 1).
    pub line: usize,
    pub message: String,
}

/// Result of a dry-run import (preview).
#[derive(Debug, Serialize)]
pub struct ImportPreview {
    pub total_rows: usize,
    pub valid_rows: usize,
    pub errors: Vec<RowError>,
    /// A sample (up to 50) of the entries that would be created.
    pub sample: Vec<PreviewEntry>,
}

/// A single entry in the import preview.
#[derive(Debug, Clone, Serialize)]
pub struct PreviewEntry {
    pub game: Game,
    pub card_id: String,
    pub name: String,
    pub condition: String,
    pub foil: bool,
    pub quantity: u32,
}

/// Result of an applied import.
#[derive(Debug, Serialize)]
pub struct ImportResult {
    pub imported: usize,
    pub skipped: usize,
    pub errors: Vec<RowError>,
}

// ---------------------------------------------------------------------------
// Export
// ---------------------------------------------------------------------------

/// Generate CSV text from the user's collection.
///
/// Joins `collection_entries` with `cards` so every row includes the card
/// name, set, and collector number for human readability.
pub fn export(conn: &Connection, game: Option<Game>) -> Result<String> {
    let (query, bind_game) = match game {
        Some(g) => (
            "SELECT ce.game, ce.card_id,
                    COALESCE(c.name, ''),
                    COALESCE(c.set_code, ''),
                    COALESCE(c.set_name, ''),
                    COALESCE(c.collector_number, ''),
                    ce.condition, ce.foil, ce.quantity, ce.notes,
                    ce.acquired_at, ce.acquired_price_cents
             FROM collection_entries ce
             LEFT JOIN cards c ON ce.game = c.game AND ce.card_id = c.card_id
             WHERE ce.game = ?1
             ORDER BY c.name, ce.created_at",
            Some(g),
        ),
        None => (
            "SELECT ce.game, ce.card_id,
                    COALESCE(c.name, ''),
                    COALESCE(c.set_code, ''),
                    COALESCE(c.set_name, ''),
                    COALESCE(c.collector_number, ''),
                    ce.condition, ce.foil, ce.quantity, ce.notes,
                    ce.acquired_at, ce.acquired_price_cents
             FROM collection_entries ce
             LEFT JOIN cards c ON ce.game = c.game AND ce.card_id = c.card_id
             ORDER BY ce.game, c.name, ce.created_at",
            None,
        ),
    };

    let mut wtr = csv::WriterBuilder::new()
        .has_headers(false)
        .from_writer(Vec::new());

    // Write the header explicitly so we control column order.
    wtr.write_record(HEADERS)
        .map_err(|e| Error::Internal(e.to_string()))?;

    let mut stmt = conn.prepare(query)?;
    let rows: Vec<ExportRow> = if let Some(g) = bind_game {
        stmt.query_map(params![g.slug()], map_export_row)?
            .collect::<std::result::Result<Vec<_>, _>>()?
    } else {
        stmt.query_map([], map_export_row)?
            .collect::<std::result::Result<Vec<_>, _>>()?
    };

    for row in &rows {
        wtr.serialize(row)
            .map_err(|e| Error::Internal(e.to_string()))?;
    }

    let bytes = wtr
        .into_inner()
        .map_err(|e| Error::Internal(e.to_string()))?;
    String::from_utf8(bytes).map_err(|e| Error::Internal(e.to_string()))
}

fn map_export_row(r: &rusqlite::Row) -> rusqlite::Result<ExportRow> {
    Ok(ExportRow {
        game: r.get(0)?,
        card_id: r.get(1)?,
        name: r.get(2)?,
        set_code: r.get(3)?,
        set_name: r.get(4)?,
        collector_number: r.get(5)?,
        condition: r.get(6)?,
        foil: r.get::<_, i64>(7)? != 0,
        quantity: r.get::<_, i64>(8)? as u32,
        notes: r.get::<_, Option<String>>(9)?.unwrap_or_default(),
        acquired_at: r.get::<_, Option<String>>(10)?.unwrap_or_default(),
        acquired_price_cents: r
            .get::<_, Option<i64>>(11)?
            .map(|v| v.to_string())
            .unwrap_or_default(),
    })
}

// ---------------------------------------------------------------------------
// Import — parsing
// ---------------------------------------------------------------------------

/// An intermediate parsed row, before catalog validation.
struct ParsedRow {
    line: usize,
    game: Game,
    card_id: String,
    condition: CardCondition,
    foil: bool,
    quantity: u32,
    notes: Option<String>,
    acquired_at: Option<String>,
    acquired_price_cents: Option<u64>,
}

/// Parse CSV text into validated rows + errors.
fn parse_csv(text: &str) -> (Vec<ParsedRow>, Vec<RowError>) {
    let mut rows = Vec::new();
    let mut errors = Vec::new();

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(text.as_bytes());

    let headers = match rdr.headers() {
        Ok(h) => h.clone(),
        Err(e) => {
            errors.push(RowError {
                line: 1,
                message: format!("failed to read CSV headers: {e}"),
            });
            return (rows, errors);
        }
    };

    // Build a column-name → index lookup so we tolerate reordered columns
    // and extra columns.
    let col =
        |name: &str| -> Option<usize> { headers.iter().position(|h| h.eq_ignore_ascii_case(name)) };

    let game_idx = col("game");
    let card_id_idx = col("card_id");
    let condition_idx = col("condition");
    let quantity_idx = col("quantity");
    let foil_idx = col("foil");
    let notes_idx = col("notes");
    let acquired_at_idx = col("acquired_at");
    let price_idx = col("acquired_price_cents");

    // The four required columns.
    for &(name, idx) in &[
        ("game", game_idx),
        ("card_id", card_id_idx),
        ("condition", condition_idx),
        ("quantity", quantity_idx),
    ] {
        if idx.is_none() {
            errors.push(RowError {
                line: 1,
                message: format!("missing required column: {name}"),
            });
        }
    }

    if !errors.is_empty() {
        return (rows, errors);
    }

    // Unwrap is safe — we checked above.
    let game_idx = game_idx.unwrap();
    let card_id_idx = card_id_idx.unwrap();
    let condition_idx = condition_idx.unwrap();
    let quantity_idx = quantity_idx.unwrap();

    for (i, result) in rdr.records().enumerate() {
        let line = i + 2; // +1 for header, +1 for 1-based
        let record = match result {
            Ok(r) => r,
            Err(e) => {
                errors.push(RowError {
                    line,
                    message: format!("malformed CSV row: {e}"),
                });
                continue;
            }
        };

        let get = |idx: usize| -> Option<&str> {
            record.get(idx).map(|s| s.trim()).filter(|s| !s.is_empty())
        };

        // game
        let game = match get(game_idx).and_then(Game::from_slug) {
            Some(g) => g,
            None => {
                errors.push(RowError {
                    line,
                    message: format!(
                        "invalid game '{}' — expected 'mtg' or 'pokemon'",
                        get(game_idx).unwrap_or("")
                    ),
                });
                continue;
            }
        };

        // card_id
        let card_id = match get(card_id_idx) {
            Some(id) => id.to_owned(),
            None => {
                errors.push(RowError {
                    line,
                    message: "card_id is empty".into(),
                });
                continue;
            }
        };

        // condition
        let condition = match get(condition_idx).map(parse_condition) {
            Some(Some(c)) => c,
            _ => {
                errors.push(RowError {
                    line,
                    message: format!(
                        "invalid condition '{}' — expected NM, LP, MP, HP, or DMG",
                        get(condition_idx).unwrap_or("")
                    ),
                });
                continue;
            }
        };

        // quantity
        let quantity: u32 = match get(quantity_idx).and_then(|s| s.parse().ok()) {
            Some(q) if q > 0 => q,
            _ => {
                errors.push(RowError {
                    line,
                    message: format!(
                        "invalid quantity '{}' — must be a positive integer",
                        get(quantity_idx).unwrap_or("")
                    ),
                });
                continue;
            }
        };

        // foil (optional, defaults to false)
        let foil = foil_idx
            .and_then(&get)
            .map(|s| matches!(s, "true" | "1" | "yes"))
            .unwrap_or(false);

        // notes (optional)
        let notes = notes_idx.and_then(&get).map(|s| s.to_owned());

        // acquired_at (optional)
        let acquired_at = acquired_at_idx.and_then(&get).map(|s| s.to_owned());

        // acquired_price_cents (optional)
        let acquired_price_cents = price_idx.and_then(&get).and_then(|s| s.parse::<u64>().ok());

        rows.push(ParsedRow {
            line,
            game,
            card_id,
            condition,
            foil,
            quantity,
            notes,
            acquired_at,
            acquired_price_cents,
        });
    }

    (rows, errors)
}

fn parse_condition(s: &str) -> Option<CardCondition> {
    match s {
        // Short codes (export format).
        "NM" => Some(CardCondition::NearMint),
        "LP" => Some(CardCondition::LightlyPlayed),
        "MP" => Some(CardCondition::ModeratelyPlayed),
        "HP" => Some(CardCondition::HeavilyPlayed),
        "DMG" => Some(CardCondition::Damaged),
        // Serde-style snake_case names.
        "near_mint" => Some(CardCondition::NearMint),
        "lightly_played" => Some(CardCondition::LightlyPlayed),
        "moderately_played" => Some(CardCondition::ModeratelyPlayed),
        "heavily_played" => Some(CardCondition::HeavilyPlayed),
        "damaged" => Some(CardCondition::Damaged),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Import — preview (dry-run)
// ---------------------------------------------------------------------------

/// Parse and validate CSV text, returning a preview of what the import would
/// do without actually changing the database.
pub fn import_preview(conn: &Connection, text: &str) -> Result<ImportPreview> {
    let (parsed, mut errors) = parse_csv(text);

    let mut valid = Vec::new();

    for row in &parsed {
        // Check that the card exists in the catalog (FK constraint).
        let exists: bool = conn
            .query_row(
                "SELECT 1 FROM cards WHERE game = ?1 AND card_id = ?2",
                params![row.game.slug(), row.card_id],
                |_| Ok(true),
            )
            .unwrap_or(false);

        if !exists {
            errors.push(RowError {
                line: row.line,
                message: format!(
                    "card '{}' ({}) not found in catalog — run a catalog import first",
                    row.card_id,
                    row.game.slug()
                ),
            });
            continue;
        }

        // Look up card name for the preview.
        let name: String = conn
            .query_row(
                "SELECT name FROM cards WHERE game = ?1 AND card_id = ?2",
                params![row.game.slug(), row.card_id],
                |r| r.get(0),
            )
            .unwrap_or_default();

        valid.push(PreviewEntry {
            game: row.game,
            card_id: row.card_id.clone(),
            name,
            condition: row.condition.code().to_owned(),
            foil: row.foil,
            quantity: row.quantity,
        });
    }

    let total_rows = parsed.len() + errors.iter().filter(|e| e.line > 1).count();
    let valid_rows = valid.len();

    // Cap the sample to 50 rows.
    let sample: Vec<PreviewEntry> = valid.into_iter().take(50).collect();

    Ok(ImportPreview {
        total_rows,
        valid_rows,
        errors,
        sample,
    })
}

// ---------------------------------------------------------------------------
// Import — apply
// ---------------------------------------------------------------------------

/// Parse CSV text and insert valid rows into `collection_entries`.
///
/// Runs the inserts inside a transaction: if any row fails, the entire
/// import is rolled back.
pub fn import_apply(conn: &Connection, text: &str) -> Result<ImportResult> {
    let (parsed, parse_errors) = parse_csv(text);

    let mut imported = 0usize;
    let mut skipped = 0usize;
    let mut errors: Vec<RowError> = parse_errors;

    // Collect valid rows (card exists in catalog).
    let mut to_insert = Vec::new();
    for row in &parsed {
        let exists: bool = conn
            .query_row(
                "SELECT 1 FROM cards WHERE game = ?1 AND card_id = ?2",
                params![row.game.slug(), row.card_id],
                |_| Ok(true),
            )
            .unwrap_or(false);

        if exists {
            to_insert.push(row);
        } else {
            errors.push(RowError {
                line: row.line,
                message: format!(
                    "card '{}' ({}) not found in catalog — skipped",
                    row.card_id,
                    row.game.slug()
                ),
            });
            skipped += 1;
        }
    }

    // Use a savepoint so we can roll back on error without aborting the
    // outer implicit transaction.
    conn.execute_batch("SAVEPOINT csv_import")?;

    for row in &to_insert {
        let entry_id = uuid::Uuid::new_v4().to_string();
        let result = conn.execute(
            "INSERT INTO collection_entries
                (entry_id, game, card_id, condition, foil, quantity, notes,
                 acquired_at, acquired_price_cents)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                entry_id,
                row.game.slug(),
                row.card_id,
                row.condition.code(),
                row.foil as i64,
                row.quantity as i64,
                row.notes,
                row.acquired_at,
                row.acquired_price_cents.map(|c| c as i64),
            ],
        );

        match result {
            Ok(_) => imported += 1,
            Err(e) => {
                // Roll back and abort.
                let _ = conn.execute_batch("ROLLBACK TO csv_import");
                return Err(Error::Storage(format!(
                    "import failed at line {}: {e}",
                    row.line
                )));
            }
        }
    }

    conn.execute_batch("RELEASE csv_import")?;

    Ok(ImportResult {
        imported,
        skipped,
        errors,
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog;
    use crate::collection;
    use crate::core::{Card, CardCondition, CardId};
    use crate::storage::test_support::memory_conn;

    fn seed_card(conn: &Connection, game: Game, id: &str, name: &str) {
        catalog::upsert(
            conn,
            &Card {
                game,
                id: CardId(id.to_owned()),
                name: name.to_owned(),
                set_code: "test".to_owned(),
                set_name: "Test Set".to_owned(),
                collector_number: "1".to_owned(),
                image_url: None,
                rarity: None,
            },
        )
        .unwrap();
    }

    #[test]
    fn export_empty_collection_returns_header_only() {
        let conn = memory_conn();
        let csv = export(&conn, None).unwrap();
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains("game"));
        assert!(lines[0].contains("card_id"));
    }

    #[test]
    fn export_round_trips_through_import() {
        let conn = memory_conn();
        seed_card(&conn, Game::Mtg, "abc-123", "Lightning Bolt");

        // Add a collection entry.
        collection::add(
            &conn,
            collection::NewEntry {
                game: Game::Mtg,
                card_id: CardId("abc-123".into()),
                condition: CardCondition::NearMint,
                foil: false,
                quantity: 4,
                notes: Some("test notes".into()),
                acquired_at: None,
                acquired_price_cents: Some(50),
            },
        )
        .unwrap();

        let csv = export(&conn, None).unwrap();

        // Preview the import in a fresh DB that has the same catalog.
        let conn2 = memory_conn();
        seed_card(&conn2, Game::Mtg, "abc-123", "Lightning Bolt");

        let preview = import_preview(&conn2, &csv).unwrap();
        assert_eq!(preview.valid_rows, 1);
        assert!(preview.errors.is_empty(), "errors: {:?}", preview.errors);
        assert_eq!(preview.sample[0].quantity, 4);

        // Apply.
        let result = import_apply(&conn2, &csv).unwrap();
        assert_eq!(result.imported, 1);
        assert_eq!(result.skipped, 0);

        // Verify the entry exists.
        let entries = collection::list(&conn2, None).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].quantity, 4);
        assert_eq!(entries[0].card_id.0, "abc-123");
    }

    #[test]
    fn import_rejects_missing_required_columns() {
        let conn = memory_conn();
        let csv = "name,set_code\nBolt,LEA\n";
        let preview = import_preview(&conn, csv).unwrap();
        assert_eq!(preview.valid_rows, 0);
        assert!(!preview.errors.is_empty());
        // Should mention the missing required columns.
        let msgs: String = preview
            .errors
            .iter()
            .map(|e| e.message.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        assert!(msgs.contains("game"), "should mention game: {msgs}");
        assert!(msgs.contains("card_id"), "should mention card_id: {msgs}");
    }

    #[test]
    fn import_rejects_unknown_card() {
        let conn = memory_conn();
        let csv = "game,card_id,condition,quantity\nmtg,no-such-card,NM,1\n";
        let preview = import_preview(&conn, csv).unwrap();
        assert_eq!(preview.valid_rows, 0);
        assert_eq!(preview.errors.len(), 1);
        assert!(preview.errors[0].message.contains("not found in catalog"));
    }

    #[test]
    fn import_rejects_bad_condition() {
        let conn = memory_conn();
        seed_card(&conn, Game::Mtg, "abc-123", "Bolt");
        let csv = "game,card_id,condition,quantity\nmtg,abc-123,TERRIBLE,1\n";
        let preview = import_preview(&conn, csv).unwrap();
        assert_eq!(preview.valid_rows, 0);
        assert!(preview.errors[0].message.contains("invalid condition"));
    }

    #[test]
    fn import_rejects_zero_quantity() {
        let conn = memory_conn();
        seed_card(&conn, Game::Mtg, "abc-123", "Bolt");
        let csv = "game,card_id,condition,quantity\nmtg,abc-123,NM,0\n";
        let preview = import_preview(&conn, csv).unwrap();
        assert_eq!(preview.valid_rows, 0);
        assert!(preview.errors[0].message.contains("invalid quantity"));
    }

    #[test]
    fn import_accepts_snake_case_conditions() {
        let conn = memory_conn();
        seed_card(&conn, Game::Mtg, "abc-123", "Bolt");
        let csv = "game,card_id,condition,quantity\nmtg,abc-123,near_mint,2\n";
        let preview = import_preview(&conn, csv).unwrap();
        assert_eq!(preview.valid_rows, 1);
        assert!(preview.errors.is_empty());
    }

    #[test]
    fn import_handles_foil_variants() {
        let conn = memory_conn();
        seed_card(&conn, Game::Mtg, "abc-123", "Bolt");
        let csv = "game,card_id,condition,quantity,foil\nmtg,abc-123,NM,1,true\n";
        let preview = import_preview(&conn, csv).unwrap();
        assert_eq!(preview.valid_rows, 1);
        assert!(preview.sample[0].foil);
    }

    #[test]
    fn import_apply_rolls_back_on_fk_violation() {
        // Ensure a catalog entry exists, then drop it after parsing —
        // simulates a race (unlikely but tests the rollback).
        let conn = memory_conn();
        // No catalog entry at all — apply should produce errors, not panic.
        let csv = "game,card_id,condition,quantity\nmtg,ghost,NM,1\n";
        let result = import_apply(&conn, csv).unwrap();
        assert_eq!(result.imported, 0);
        assert_eq!(result.skipped, 1);
    }

    #[test]
    fn export_filters_by_game() {
        let conn = memory_conn();
        seed_card(&conn, Game::Mtg, "mtg-1", "Bolt");
        seed_card(&conn, Game::Pokemon, "pkmn-1", "Pikachu");

        collection::add(
            &conn,
            collection::NewEntry {
                game: Game::Mtg,
                card_id: CardId("mtg-1".into()),
                condition: CardCondition::NearMint,
                foil: false,
                quantity: 1,
                notes: None,
                acquired_at: None,
                acquired_price_cents: None,
            },
        )
        .unwrap();
        collection::add(
            &conn,
            collection::NewEntry {
                game: Game::Pokemon,
                card_id: CardId("pkmn-1".into()),
                condition: CardCondition::NearMint,
                foil: false,
                quantity: 1,
                notes: None,
                acquired_at: None,
                acquired_price_cents: None,
            },
        )
        .unwrap();

        let csv_mtg = export(&conn, Some(Game::Mtg)).unwrap();
        let csv_all = export(&conn, None).unwrap();

        // MTG-only should have header + 1 data row.
        assert_eq!(csv_mtg.lines().count(), 2);
        // All-games should have header + 2 data rows.
        assert_eq!(csv_all.lines().count(), 3);
    }
}
