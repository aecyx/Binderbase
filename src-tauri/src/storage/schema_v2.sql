-- Binderbase schema v2
--
-- Adds:
--  * `settings` — tiny key/value store for non-secret user preferences
--    (last-import timestamps, preferred game filter, etc.). Keyed by a short
--    namespaced string ("catalog.last_imported_at", "ui.default_game", ...).
--    Secrets (e.g. the Pokémon TCG API key) do NOT live here — they go to
--    the OS keychain via the `settings::SecretStore` abstraction.
--
--  * `catalog_imports` — one row per bulk-import run. Used for
--    "last updated on <date>" UI and to diagnose partial / failed imports.
--    Status is one of: 'running', 'completed', 'cancelled', 'failed'.

CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS catalog_imports (
    import_id       TEXT PRIMARY KEY,         -- UUIDv4
    game            TEXT NOT NULL REFERENCES games(slug),
    started_at      TEXT NOT NULL DEFAULT (datetime('now')),
    finished_at     TEXT,
    status          TEXT NOT NULL,            -- 'running' | 'completed' | 'cancelled' | 'failed'
    cards_imported  INTEGER NOT NULL DEFAULT 0,
    error_message   TEXT
);

CREATE INDEX IF NOT EXISTS catalog_imports_game_idx
    ON catalog_imports (game, started_at DESC);
