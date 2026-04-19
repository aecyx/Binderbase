// SPDX-License-Identifier: AGPL-3.0-or-later
//! Schema migrations for the local SQLite database.
//!
//! Each `apply_vN` function brings the database from version N−1 to N. The
//! runner executes them in order, skipping any that have already been applied.
//! Never edit a shipped migration — add a new one instead.

use super::SCHEMA_VERSION;
use crate::core::{Error, Result};
use rusqlite::Connection;

/// Ensure the database is at the latest schema version.
///
/// Creates the `schema_version` bookkeeping table if it doesn't exist, reads
/// the current version, applies any outstanding migrations, and returns an
/// error if the on-disk schema is *newer* than what this build of the code
/// understands (i.e. the user downgraded the binary).
pub fn run(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL
        );",
    )?;

    let current: u32 = conn
        .query_row("SELECT MAX(version) FROM schema_version", [], |r| {
            r.get::<_, Option<u32>>(0)
        })
        .unwrap_or(None)
        .unwrap_or(0);

    if current > SCHEMA_VERSION {
        return Err(Error::Storage(format!(
            "database schema version ({current}) is newer than this build supports ({SCHEMA_VERSION}); \
             upgrade Binderbase or use a matching database"
        )));
    }

    if current < 1 {
        apply_v1(conn)?;
    }
    if current < 2 {
        apply_v2(conn)?;
    }
    if current < 3 {
        apply_v3(conn)?;
    }

    Ok(())
}

fn apply_v1(conn: &Connection) -> Result<()> {
    conn.execute_batch(include_str!("schema_v1.sql"))?;
    conn.execute(
        "INSERT INTO schema_version (version, applied_at) VALUES (1, datetime('now'))",
        [],
    )?;
    Ok(())
}

fn apply_v2(conn: &Connection) -> Result<()> {
    conn.execute_batch(include_str!("schema_v2.sql"))?;
    conn.execute(
        "INSERT INTO schema_version (version, applied_at) VALUES (2, datetime('now'))",
        [],
    )?;
    Ok(())
}

fn apply_v3(conn: &Connection) -> Result<()> {
    conn.execute_batch(include_str!("schema_v3.sql"))?;
    conn.execute(
        "INSERT INTO schema_version (version, applied_at) VALUES (3, datetime('now'))",
        [],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn memory_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "foreign_keys", "ON").unwrap();
        conn
    }

    #[test]
    fn fresh_database_migrates_to_latest() {
        let conn = memory_conn();
        run(&conn).unwrap();

        let version: u32 = conn
            .query_row("SELECT MAX(version) FROM schema_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(version, SCHEMA_VERSION);
    }

    #[test]
    fn idempotent_migration() {
        let conn = memory_conn();
        run(&conn).unwrap();
        // Running again should be a no-op, not an error.
        run(&conn).unwrap();

        let version: u32 = conn
            .query_row("SELECT MAX(version) FROM schema_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(version, SCHEMA_VERSION);
    }

    #[test]
    fn v1_to_v2_upgrade_adds_new_tables_without_rewriting_v1() {
        // A DB that stopped at v1 should upgrade cleanly when the code moves
        // to v2 — without re-applying v1's DDL.
        let conn = memory_conn();
        // `run()` normally creates schema_version; since we're calling
        // apply_v1 directly we need the table to exist first.
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_version (
                version INTEGER PRIMARY KEY,
                applied_at TEXT NOT NULL
            );",
        )
        .unwrap();
        apply_v1(&conn).unwrap();

        // Simulate an unexpected user row that would be clobbered by a naive
        // re-run of v1's `INSERT OR IGNORE` (it's OR IGNORE so it's safe, but
        // still a good canary against accidental re-apply).
        conn.execute(
            "INSERT INTO cards (game, card_id, name, set_code, set_name, collector_number)
             VALUES ('mtg', 'probe', 'Probe', 'X', 'X', '1')",
            [],
        )
        .unwrap();

        run(&conn).unwrap();

        let version: u32 = conn
            .query_row("SELECT MAX(version) FROM schema_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(version, SCHEMA_VERSION);

        // v1 data is intact.
        let name: String = conn
            .query_row(
                "SELECT name FROM cards WHERE game = 'mtg' AND card_id = 'probe'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(name, "Probe");

        // v2 tables exist and are empty.
        let settings_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM settings", [], |r| r.get(0))
            .unwrap();
        assert_eq!(settings_count, 0);
        let imports_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM catalog_imports", [], |r| r.get(0))
            .unwrap();
        assert_eq!(imports_count, 0);
    }

    #[test]
    fn rejects_schema_newer_than_code() {
        let conn = memory_conn();
        run(&conn).unwrap();

        // Simulate a future migration by inserting a version beyond SCHEMA_VERSION.
        conn.execute(
            "INSERT INTO schema_version (version, applied_at) VALUES (?1, datetime('now'))",
            [SCHEMA_VERSION + 1],
        )
        .unwrap();

        let err = run(&conn).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("newer than this build"),
            "expected version-mismatch error, got: {msg}"
        );
    }

    #[test]
    fn latest_schema_creates_expected_tables() {
        let conn = memory_conn();
        run(&conn).unwrap();

        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<std::result::Result<_, _>>()
            .unwrap();

        for expected in &[
            "card_hashes",
            "cards",
            "catalog_imports",
            "collection_entries",
            "games",
            "prices",
            "scan_events",
            "schema_version",
            "settings",
        ] {
            assert!(
                tables.iter().any(|t| t == expected),
                "missing table: {expected} (found: {tables:?})"
            );
        }
    }

    #[test]
    fn v2_to_v3_upgrade_adds_card_hashes() {
        let conn = memory_conn();
        // Manually run only v1 and v2.
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_version (
                version INTEGER PRIMARY KEY,
                applied_at TEXT NOT NULL
            );",
        )
        .unwrap();
        apply_v1(&conn).unwrap();
        apply_v2(&conn).unwrap();

        let version: u32 = conn
            .query_row("SELECT MAX(version) FROM schema_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(version, 2);

        // Full migration should upgrade to v3.
        run(&conn).unwrap();

        let version: u32 = conn
            .query_row("SELECT MAX(version) FROM schema_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(version, 3);

        // card_hashes table exists.
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM card_hashes", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }
}
