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
    // Future: if current < 2 { apply_v2(conn)?; }

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
    fn fresh_database_migrates_to_v1() {
        let conn = memory_conn();
        run(&conn).unwrap();

        let version: u32 = conn
            .query_row("SELECT MAX(version) FROM schema_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(version, 1);
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
        assert_eq!(version, 1);
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
    fn v1_creates_expected_tables() {
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
            "cards",
            "collection_entries",
            "games",
            "prices",
            "scan_events",
            "schema_version",
        ] {
            assert!(
                tables.iter().any(|t| t == expected),
                "missing table: {expected} (found: {tables:?})"
            );
        }
    }
}
