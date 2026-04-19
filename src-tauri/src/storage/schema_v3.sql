-- Binderbase schema v3
--
-- Adds:
--  * `card_hashes` — perceptual hashes of card art images, computed from
--    CDN thumbnails during `scan_build_index`. Queried by `scan_identify`
--    to find nearest-neighbour matches via Hamming distance.

CREATE TABLE IF NOT EXISTS card_hashes (
    game       TEXT NOT NULL,
    card_id    TEXT NOT NULL,
    hash       BLOB NOT NULL,            -- 256-bit dHash (32 bytes)
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (game, card_id),
    FOREIGN KEY (game, card_id) REFERENCES cards (game, card_id) ON DELETE CASCADE
);
