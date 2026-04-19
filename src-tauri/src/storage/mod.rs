// SPDX-License-Identifier: AGPL-3.0-or-later
//! SQLite storage layer.
//!
//! We use `rusqlite` with the `bundled` feature — the SQLite source is
//! compiled into the binary, so there is no system library dependency on any
//! desktop or mobile target.
//!
//! Connections are not shared across threads. Callers get a fresh connection
//! via `Database::connect` (cheap) or take a short-lived lock on the shared
//! application connection. Rusqlite's `Connection` is not `Sync`, so higher
//! layers wrap it in `Mutex` or use a per-call open.

use crate::core::{Error, Result};
use directories::ProjectDirs;
use rusqlite::Connection;
use std::path::{Path, PathBuf};

/// Current schema version. Bump this and add a migration when the schema
/// changes. See `migrations` module below.
pub const SCHEMA_VERSION: u32 = 3;

/// Qualifiers used to locate the app's per-user data directory.
/// These become e.g.:
///   Windows: C:\Users\<user>\AppData\Roaming\Binderbase\Binderbase\data
///   macOS:   ~/Library/Application Support/com.Binderbase.Binderbase
///   Linux:   ~/.local/share/binderbase
const APP_QUALIFIER: &str = "com";
const APP_ORG: &str = "Binderbase";
const APP_NAME: &str = "Binderbase";

pub struct Database {
    path: PathBuf,
}

impl Database {
    /// Locate (and create if missing) the standard per-user data dir, then
    /// point `Database` at `binderbase.sqlite3` inside it.
    pub fn in_user_data_dir() -> Result<Self> {
        let dirs = ProjectDirs::from(APP_QUALIFIER, APP_ORG, APP_NAME)
            .ok_or_else(|| Error::Storage("could not resolve user data directory".into()))?;
        let data_dir = dirs.data_dir();
        std::fs::create_dir_all(data_dir)?;
        Ok(Self::at(data_dir.join("binderbase.sqlite3")))
    }

    /// Use an explicit path. Handy for tests (`:memory:` or a tempdir).
    pub fn at(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Open a new connection and run any outstanding migrations.
    pub fn connect(&self) -> Result<Connection> {
        let conn = Connection::open(&self.path)?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        migrations::run(&conn)?;
        Ok(conn)
    }

    /// Open a temporary connection, run `f`, and close it. Convenient for
    /// background tasks that need a short-lived connection.
    pub fn with_connection<T, F>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&Connection) -> Result<T>,
    {
        let conn = self.connect()?;
        f(&conn)
    }
}

mod migrations;

/// Test-only helpers shared across modules.
///
/// This is `pub(crate)` and gated on `#[cfg(test)]` so it ships nothing to
/// release builds but any sibling module's `#[cfg(test)] mod tests` can reach
/// it via `crate::storage::test_support::memory_conn()`. Keeps every suite on
/// one migration path — if schema_v1.sql changes, every test picks it up for
/// free.
#[cfg(test)]
pub(crate) mod test_support {
    use super::migrations;
    use rusqlite::Connection;

    /// Open an in-memory SQLite connection with the production pragmas applied
    /// and all migrations run. Panics on failure — only for use in tests.
    pub fn memory_conn() -> Connection {
        let conn = Connection::open_in_memory().expect("open in-memory sqlite");
        conn.pragma_update(None, "foreign_keys", "ON")
            .expect("enable foreign_keys");
        // journal_mode=WAL is meaningless on :memory: — skip it to avoid noise.
        conn.pragma_update(None, "synchronous", "NORMAL")
            .expect("set synchronous");
        migrations::run(&conn).expect("run migrations");
        conn
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn database_connect_creates_file_and_migrates() {
        let dir = TempDir::new().unwrap();
        let db = Database::at(dir.path().join("test.sqlite3"));
        let conn = db.connect().unwrap();

        let version: u32 = conn
            .query_row("SELECT MAX(version) FROM schema_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(version, SCHEMA_VERSION);
    }

    #[test]
    fn connect_is_idempotent() {
        let dir = TempDir::new().unwrap();
        let db = Database::at(dir.path().join("test.sqlite3"));
        db.connect().unwrap();
        // Second connect on the same file should not fail.
        let conn = db.connect().unwrap();

        let version: u32 = conn
            .query_row("SELECT MAX(version) FROM schema_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(version, SCHEMA_VERSION);
    }

    #[test]
    fn foreign_keys_enabled() {
        let dir = TempDir::new().unwrap();
        let db = Database::at(dir.path().join("test.sqlite3"));
        let conn = db.connect().unwrap();

        let fk: i64 = conn
            .pragma_query_value(None, "foreign_keys", |r| r.get(0))
            .unwrap();
        assert_eq!(fk, 1);
    }

    #[test]
    fn wal_mode_enabled() {
        let dir = TempDir::new().unwrap();
        let db = Database::at(dir.path().join("test.sqlite3"));
        let conn = db.connect().unwrap();

        let mode: String = conn
            .pragma_query_value(None, "journal_mode", |r| r.get(0))
            .unwrap();
        assert_eq!(mode, "wal");
    }
}
