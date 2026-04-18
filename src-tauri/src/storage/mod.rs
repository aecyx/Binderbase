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
pub const SCHEMA_VERSION: u32 = 1;

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
}

mod migrations {
    use super::{Result, SCHEMA_VERSION};
    use rusqlite::Connection;

    pub fn run(conn: &Connection) -> Result<()> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_version (
                version INTEGER PRIMARY KEY,
                applied_at TEXT NOT NULL
            );",
        )?;
        let current: Option<u32> = conn
            .query_row("SELECT MAX(version) FROM schema_version", [], |r| {
                r.get::<_, Option<u32>>(0)
            })
            .unwrap_or(None);

        let current = current.unwrap_or(0);
        if current < 1 {
            apply_v1(conn)?;
        }
        // Future: if current < 2 { apply_v2(conn)?; }

        assert!(current <= SCHEMA_VERSION, "schema newer than code");
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
}
