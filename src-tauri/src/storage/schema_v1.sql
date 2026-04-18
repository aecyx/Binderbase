-- Binderbase schema v1
--
-- Design notes:
--  * `cards` stores the canonical catalog (one row per printing). Keyed by
--    (game, card_id); the source id is the game-native identifier.
--  * `collection_entries` is the user's owned cards. One row per
--    (printing, condition, foil) bucket with a count; we do NOT store one row
--    per physical card because that scales poorly and no one wants to add 4
--    copies of the same card individually.
--  * `prices` caches lookups to avoid hammering upstream APIs. Kept in a
--    separate table because it's refreshed on a different cadence than the
--    catalog.

PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS games (
    slug TEXT PRIMARY KEY,
    display_name TEXT NOT NULL
);

INSERT OR IGNORE INTO games (slug, display_name) VALUES
    ('mtg', 'Magic: The Gathering'),
    ('pokemon', 'Pokémon TCG');

CREATE TABLE IF NOT EXISTS cards (
    game TEXT NOT NULL REFERENCES games(slug),
    card_id TEXT NOT NULL,               -- game-native id (e.g. Scryfall id)
    name TEXT NOT NULL,
    set_code TEXT NOT NULL,
    set_name TEXT NOT NULL,
    collector_number TEXT NOT NULL,
    rarity TEXT,
    image_url TEXT,
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (game, card_id)
);

CREATE INDEX IF NOT EXISTS cards_name_idx ON cards (name COLLATE NOCASE);
CREATE INDEX IF NOT EXISTS cards_set_idx  ON cards (game, set_code);

CREATE TABLE IF NOT EXISTS collection_entries (
    entry_id TEXT PRIMARY KEY,           -- UUIDv4
    game TEXT NOT NULL REFERENCES games(slug),
    card_id TEXT NOT NULL,
    condition TEXT NOT NULL,             -- 'NM','LP','MP','HP','DMG'
    foil INTEGER NOT NULL DEFAULT 0,     -- bool; per-game meaning (foil, holo, etc.)
    quantity INTEGER NOT NULL CHECK (quantity > 0),
    notes TEXT,
    acquired_at TEXT,                    -- ISO 8601 date, nullable
    acquired_price_cents INTEGER,        -- nullable; user-entered purchase price
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (game, card_id) REFERENCES cards (game, card_id) ON DELETE RESTRICT
);

CREATE INDEX IF NOT EXISTS collection_card_idx
    ON collection_entries (game, card_id);
CREATE INDEX IF NOT EXISTS collection_condition_idx
    ON collection_entries (game, card_id, condition, foil);

CREATE TABLE IF NOT EXISTS prices (
    game TEXT NOT NULL REFERENCES games(slug),
    card_id TEXT NOT NULL,
    currency TEXT NOT NULL,              -- ISO 4217 (USD, EUR, ...)
    source TEXT NOT NULL,                -- e.g. 'scryfall', 'ptcgapi'
    cents INTEGER NOT NULL,
    foil INTEGER NOT NULL DEFAULT 0,
    fetched_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (game, card_id, currency, source, foil),
    FOREIGN KEY (game, card_id) REFERENCES cards (game, card_id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS scan_events (
    scan_id TEXT PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    game TEXT REFERENCES games(slug),
    matched_card_id TEXT,
    confidence REAL,                     -- 0..1
    image_path TEXT,                     -- local path to the captured image (nullable)
    notes TEXT
);

CREATE INDEX IF NOT EXISTS scan_events_created_idx ON scan_events (created_at DESC);
