// SPDX-License-Identifier: AGPL-3.0-or-later
//! User settings — non-secret preferences and secret credentials.
//!
//! Two stores, same module:
//!
//! * **`settings` table** (SQLite). Non-secret key/value pairs: last-import
//!   timestamps, UI preferences, catalog filters. See `get`/`set`/`delete`.
//!
//! * **[`SecretStore`]** (OS keychain). Secrets — right now only the Pokémon
//!   TCG API key. The production impl ([`KeyringSecrets`]) delegates to the
//!   `keyring` crate, which talks to:
//!     - Windows: DPAPI-backed Credential Manager
//!     - macOS:   Keychain Services
//!     - Linux:   Secret Service (GNOME Keyring / KWallet)
//!
//!   Tests use [`InMemorySecrets`] so they don't pollute the developer's real
//!   credential store. Production code takes `&dyn SecretStore`, so the two
//!   are swap-in compatible.
//!
//! Keys (both stores) follow a dotted namespace: `catalog.last_imported_at`,
//! `ptcgapi.key`, etc. Keep them short and stable — they're effectively a
//! schema.

use crate::core::{Error, Result};
use rusqlite::{params, Connection};

// ---------- public key constants ----------

/// Secret-store key for the Pokémon TCG API key.
pub const PTCGAPI_KEY: &str = "ptcgapi.key";

/// Settings-table key for the timestamp of the last successful catalog import
/// (ISO 8601). One entry per game: `catalog.last_imported_at.{game.slug()}`.
pub fn last_imported_at_key(game_slug: &str) -> String {
    format!("catalog.last_imported_at.{game_slug}")
}

// ---------- SQLite-backed settings ----------

/// Read a setting. Returns `None` if the key has never been set.
pub fn get(conn: &Connection, key: &str) -> Result<Option<String>> {
    let mut stmt = conn.prepare_cached("SELECT value FROM settings WHERE key = ?1")?;
    let mut rows = stmt.query_map(params![key], |r| r.get::<_, String>(0))?;
    match rows.next() {
        Some(r) => Ok(Some(r?)),
        None => Ok(None),
    }
}

/// Upsert a setting. `updated_at` is refreshed on every write.
pub fn set(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO settings (key, value, updated_at)
         VALUES (?1, ?2, datetime('now'))
         ON CONFLICT (key) DO UPDATE SET
            value = excluded.value,
            updated_at = datetime('now')",
        params![key, value],
    )?;
    Ok(())
}

/// Delete a setting. No-op if the key isn't present.
pub fn delete(conn: &Connection, key: &str) -> Result<()> {
    conn.execute("DELETE FROM settings WHERE key = ?1", params![key])?;
    Ok(())
}

// ---------- secret store abstraction ----------

/// A place to stash a secret, keyed by string. Production uses the OS
/// credential store; tests use an in-memory map.
///
/// `Send + Sync` because we stuff this behind `Arc<dyn SecretStore>` in
/// `AppState` and hand it to async import tasks.
pub trait SecretStore: Send + Sync {
    /// `Ok(None)` means "no such entry" — distinct from an actual backend
    /// error, which becomes `Err(Error::Storage(..))`.
    fn get(&self, key: &str) -> Result<Option<String>>;
    fn set(&self, key: &str, value: &str) -> Result<()>;
    fn delete(&self, key: &str) -> Result<()>;
}

/// OS-keychain-backed secret store. Uses a fixed service string so every
/// secret Binderbase stores is namespaced under the same vault entry.
pub struct KeyringSecrets {
    service: String,
}

impl KeyringSecrets {
    pub fn new() -> Self {
        Self {
            service: "binderbase".into(),
        }
    }

    /// Test-only constructor with a custom service name so two parallel test
    /// processes don't fight over the same keychain entry.
    #[cfg(test)]
    pub fn with_service(service: impl Into<String>) -> Self {
        Self {
            service: service.into(),
        }
    }

    fn entry(&self, key: &str) -> Result<keyring::Entry> {
        keyring::Entry::new(&self.service, key).map_err(|e| Error::Storage(format!("keyring: {e}")))
    }
}

impl Default for KeyringSecrets {
    fn default() -> Self {
        Self::new()
    }
}

impl SecretStore for KeyringSecrets {
    fn get(&self, key: &str) -> Result<Option<String>> {
        match self.entry(key)?.get_password() {
            Ok(v) => Ok(Some(v)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(Error::Storage(format!("keyring get {key}: {e}"))),
        }
    }

    fn set(&self, key: &str, value: &str) -> Result<()> {
        self.entry(key)?
            .set_password(value)
            .map_err(|e| Error::Storage(format!("keyring set {key}: {e}")))
    }

    fn delete(&self, key: &str) -> Result<()> {
        match self.entry(key)?.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()), // idempotent
            Err(e) => Err(Error::Storage(format!("keyring delete {key}: {e}"))),
        }
    }
}

/// Test-only secret store. Also useful as a fallback if the OS keychain is
/// unavailable (headless Linux CI, sandboxed installs); right now we only
/// wire it up in tests, but exposing it publicly keeps that door open.
pub struct InMemorySecrets {
    inner: std::sync::Mutex<std::collections::HashMap<String, String>>,
}

impl InMemorySecrets {
    pub fn new() -> Self {
        Self {
            inner: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }
}

impl Default for InMemorySecrets {
    fn default() -> Self {
        Self::new()
    }
}

impl SecretStore for InMemorySecrets {
    fn get(&self, key: &str) -> Result<Option<String>> {
        let guard = self
            .inner
            .lock()
            .map_err(|_| Error::Internal("in-memory secret store poisoned".into()))?;
        Ok(guard.get(key).cloned())
    }

    fn set(&self, key: &str, value: &str) -> Result<()> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| Error::Internal("in-memory secret store poisoned".into()))?;
        guard.insert(key.to_string(), value.to_string());
        Ok(())
    }

    fn delete(&self, key: &str) -> Result<()> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| Error::Internal("in-memory secret store poisoned".into()))?;
        guard.remove(key);
        Ok(())
    }
}

// ---------- typed PTCGAPI-key helpers ----------

/// Retrieve the Pokémon TCG API key, if the user has set one.
pub fn get_ptcgapi_key(store: &dyn SecretStore) -> Result<Option<String>> {
    store.get(PTCGAPI_KEY)
}

/// Store the Pokémon TCG API key. An empty string is treated as a delete —
/// saves the UI a separate "clear" button.
pub fn set_ptcgapi_key(store: &dyn SecretStore, value: &str) -> Result<()> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        store.delete(PTCGAPI_KEY)
    } else {
        store.set(PTCGAPI_KEY, trimmed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::test_support::memory_conn;

    // ---- SQLite settings table ----

    #[test]
    fn get_returns_none_for_missing_key() {
        let conn = memory_conn();
        assert!(get(&conn, "does.not.exist").unwrap().is_none());
    }

    #[test]
    fn set_then_get_round_trips() {
        let conn = memory_conn();
        set(&conn, "ui.default_game", "mtg").unwrap();
        assert_eq!(
            get(&conn, "ui.default_game").unwrap().as_deref(),
            Some("mtg")
        );
    }

    #[test]
    fn set_overwrites_existing_value() {
        let conn = memory_conn();
        set(&conn, "k", "v1").unwrap();
        set(&conn, "k", "v2").unwrap();
        assert_eq!(get(&conn, "k").unwrap().as_deref(), Some("v2"));

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM settings", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1, "upsert on same key should not duplicate rows");
    }

    #[test]
    fn delete_removes_key_and_is_idempotent() {
        let conn = memory_conn();
        set(&conn, "k", "v").unwrap();
        delete(&conn, "k").unwrap();
        assert!(get(&conn, "k").unwrap().is_none());
        // Second delete is fine.
        delete(&conn, "k").unwrap();
    }

    #[test]
    fn last_imported_at_key_is_per_game() {
        assert_eq!(last_imported_at_key("mtg"), "catalog.last_imported_at.mtg");
        assert_eq!(
            last_imported_at_key("pokemon"),
            "catalog.last_imported_at.pokemon"
        );
    }

    // ---- InMemorySecrets (SecretStore impl) ----

    #[test]
    fn in_memory_secret_store_round_trip() {
        let store = InMemorySecrets::new();
        assert!(store.get(PTCGAPI_KEY).unwrap().is_none());

        store.set(PTCGAPI_KEY, "abcd-1234").unwrap();
        assert_eq!(
            store.get(PTCGAPI_KEY).unwrap().as_deref(),
            Some("abcd-1234")
        );

        store.delete(PTCGAPI_KEY).unwrap();
        assert!(store.get(PTCGAPI_KEY).unwrap().is_none());
        // Double delete is fine.
        store.delete(PTCGAPI_KEY).unwrap();
    }

    #[test]
    fn set_ptcgapi_key_treats_empty_as_delete() {
        let store = InMemorySecrets::new();
        store.set(PTCGAPI_KEY, "real-key").unwrap();
        assert_eq!(
            get_ptcgapi_key(&store).unwrap().as_deref(),
            Some("real-key")
        );

        // Empty / whitespace → delete, not an overwrite with "".
        set_ptcgapi_key(&store, "").unwrap();
        assert!(get_ptcgapi_key(&store).unwrap().is_none());

        set_ptcgapi_key(&store, "another").unwrap();
        set_ptcgapi_key(&store, "   ").unwrap();
        assert!(get_ptcgapi_key(&store).unwrap().is_none());
    }

    #[test]
    fn set_ptcgapi_key_trims_whitespace() {
        let store = InMemorySecrets::new();
        set_ptcgapi_key(&store, "   with-spaces   ").unwrap();
        assert_eq!(
            get_ptcgapi_key(&store).unwrap().as_deref(),
            Some("with-spaces")
        );
    }
}
