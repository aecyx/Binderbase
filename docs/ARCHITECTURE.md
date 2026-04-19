# Binderbase — architecture overview

> Status: living document. Last reviewed 2026-04-18. When you change any of the
> module boundaries or data sources below, update this file in the same PR.

Binderbase is a local-first desktop application for scanning and managing
trading card game (TCG) collections. The first release ships Magic: The
Gathering and Pokemon TCG support on Windows, macOS, and Linux. A mobile
build (with phone-camera scanning) is planned for a later release; the
architecture below is designed to accommodate that.

## High-level shape

```
+--------------------------------------------------------------+
|                        Desktop shell                         |
|  Tauri 2 (Rust)  <->  WebView  <->  React + TypeScript UI    |
+--------------------------------------------------------------+
                |                           |
                v                           v
+----------------------------+   +----------------------------+
|   Local SQLite (bundled)   |   |   Public HTTPS (rustls)    |
|  app-data dir, WAL mode    |   |  Scryfall, Pokemon TCG API |
+----------------------------+   +----------------------------+
```

- **Tauri 2** hosts a Rust process that owns all native I/O (filesystem,
  network, SQLite) and exposes a narrow, typed command surface to the
  webview.
- **React + TypeScript** renders the UI. The UI never reaches the network
  directly and never touches SQLite directly. Every backend interaction goes
  through the typed `api.*` helpers in `src/lib/tauri.ts`.
- **SQLite (bundled via `rusqlite`)** stores the entire collection plus a
  local cache of card metadata and prices. No server component.
- **External HTTP** is limited to two public catalog APIs. Their URLs are
  allow-listed in the Tauri CSP (`src-tauri/tauri.conf.json`).

No telemetry, no crash reporting that phones home, no auth service, no cloud
storage. This is a deliberate product constraint; see `docs/DECISIONS.md`
ADR-0002.

## Module layout

### Backend (`src-tauri/src/`)

| Module        | Responsibility                                                                          |
| ------------- | --------------------------------------------------------------------------------------- |
| `core/`       | Shared types (`Game`, `Card`, `CardCondition`, IDs) and the `Error` enum.               |
| `games/`      | Per-game adapters. `mtg.rs` talks to Scryfall, `pokemon.rs` to PTCGAPI.                 |
| `storage/`    | SQLite connection, schema, and migrations (`schema_vN.sql`).                            |
| `catalog/`    | Local reads/writes for the `cards` table + bulk import (`bulk/`).                       |
| `settings/`   | Non-secret prefs (SQLite) and secret credentials (OS keychain).                         |
| `collection/` | CRUD over `collection_entries`.                                                         |
| `pricing/`    | Cached price reads and upserts.                                                         |
| `scanning/`   | Image decode + (future) card identification pipeline.                                   |
| `commands/`   | Tauri command surface. Split by domain: `catalog`, `collection`, `pricing`, `settings`. |
| `lib.rs`      | Wires modules, initializes state, registers commands.                                   |

### Frontend (`src/`)

| Path                      | Responsibility                                                   |
| ------------------------- | ---------------------------------------------------------------- |
| `components/`             | Shared widgets (e.g. `TopNav`).                                  |
| `features/scan/`          | Image upload + identify flow.                                    |
| `features/collection/`    | Collection table, add/remove.                                    |
| `features/pricing/`       | Card lookup + price history.                                     |
| `features/import_export/` | CSV and deck-list import/export (stubbed in 0.1).                |
| `lib/tauri.ts`            | Typed `invoke` wrappers. The only file allowed to call `invoke`. |
| `styles/global.css`       | Design tokens and base element styles.                           |
| `types/index.ts`          | TS mirrors of Rust types.                                        |

## Data model (overview)

See `src-tauri/src/storage/schema_v1.sql` for the authoritative DDL. Summary:

- `games(slug)` — `mtg`, `pokemon`.
- `cards(game, card_id)` — composite primary key. Cached catalog metadata.
  `card_id` is opaque (Scryfall UUID for MTG, PTCGAPI id for Pokemon).
- `collection_entries(entry_id, game, card_id, condition, foil, quantity)` —
  the user's collection. `entry_id` is a UUID; `quantity` has a `CHECK > 0`.
- `prices(game, card_id, currency, source, foil)` — composite natural key.
  Prices stored as integer `cents` (avoiding float issues).
- `scan_events(scan_id, game, matched_card_id, confidence, image_path)` —
  audit log for scans, lets us improve matching over time.
- `schema_version(version)` — single row tracking the applied migration level.
- `settings(key, value)` — generic key-value store for non-secret preferences.
- `catalog_imports(import_id, game, status, ...)` — audit log for bulk imports.

Schema versioning is tracked by `PRAGMA user_version`; migrations are
accumulating `schema_vN.sql` files applied in order by
`storage::migrations::apply_up_to`.

## Command surface (Tauri <-> UI)

Currently:

| Command                              | Purpose                                                 |
| ------------------------------------ | ------------------------------------------------------- |
| `app_info`                           | Version, build metadata, DB path, supported games.      |
| `fetch_card(game, card_id)`          | Local-first lookup; falls through to live API on miss.  |
| `catalog_get(game, card_id)`         | Local catalog read — no network.                        |
| `catalog_search(game?, query, lim?)` | Substring search for autocomplete.                      |
| `catalog_import_start`               | Kick off background bulk import of all games.           |
| `catalog_import_cancel`              | Request cancellation of a running import.               |
| `catalog_import_status`              | Poll progress, in-progress flag, last runs per game.    |
| `collection_list(game?)`             | List collection, optionally filtered.                   |
| `collection_add(entry)`              | Insert a collection entry.                              |
| `collection_remove(entry_id)`        | Delete by entry id.                                     |
| `collection_export_csv(game?)`       | Export collection as CSV (joined with card metadata).   |
| `collection_import_preview(csv)`     | Dry-run CSV import — parse, validate, preview.          |
| `collection_import_apply(csv)`       | Apply a validated CSV import.                           |
| `pricing_get_cached(game, id)`       | Read cached prices.                                     |
| `pricing_refresh(game, id)`          | Refresh prices for a single card from the live API.     |
| `pricing_refresh_collection(game?)`  | Batch refresh all collection cards; emits progress.     |
| `scan_identify(bytes, hint?)`        | Decode an image and return candidate matches (stubbed). |
| `settings_get_ptcgapi_key`           | Read the stored Pokémon TCG API key.                    |
| `settings_set_ptcgapi_key(value)`    | Store or clear the Pokémon TCG API key.                 |

All errors serialize as `{ kind, message }` — matched by the TS
`BinderbaseError` discriminated union. UI should branch on `kind`, not parse
the message.

## External integrations

- **Scryfall** (<https://api.scryfall.com>) — MTG card metadata and prices
  in `usd` / `usd_foil`. Public, permissive usage, asks for a `User-Agent`
  identifying the client.
- **Pokemon TCG API** (<https://api.pokemontcg.io>) — Pokemon card metadata
  and TCGplayer price snapshots. An API key is optional but strongly
  recommended; we read it from `POKEMONTCG_API_KEY` in the environment.

Both services are HTTPS only, talked to via `reqwest` with `rustls-tls` (no
native-tls, to keep the mobile port tractable). Responses are cached in
SQLite; we never re-hit the network just to render.

## Scanning pipeline (planned)

0.1 only decodes the uploaded image and returns dimensions. The intended
pipeline for 1.0:

1. **Capture** — desktop uploads a file; mobile (future) uses the camera.
2. **Preprocess** — rotate, crop to card rectangle, normalize lighting.
3. **Feature extract** — perceptual hash of the card art region. Probably
   `img_hash` or a hand-rolled DCT-based hash.
4. **Candidate search** — nearest-neighbour against pre-computed hashes for
   the selected game. Hashes ship with the app (or are built on first run).
5. **Rerank** — optional OCR on the name and set-code regions to break ties.
6. **Result** — top candidates with confidence scores; user confirms.

None of these steps require a server. The hash index is a local SQLite
table. See `docs/DECISIONS.md` ADR-0003 (planned) for the pipeline choice.

## Mobile strategy

Tauri 2's mobile support is usable today. To keep the door open:

- All Rust dependencies are chosen to work on Android and iOS: `rustls-tls`
  instead of `native-tls`, `rusqlite` with `bundled`, `image` with codec
  features only.
- The command surface does not assume desktop-only primitives (no spawning
  subprocesses, no shell, no arbitrary filesystem access).
- The UI uses flexible layouts and avoids hover-only affordances so the
  same components will hold up on touch.
- Scanning is the one place we will add platform-specific code — mobile
  gets a camera feed, desktop gets file upload.

## Performance and offline behavior

- App boot does not require network; the UI loads even if both APIs are
  down. Catalog lookups fail loudly in that case.
- SQLite is opened with WAL and `synchronous=NORMAL` for good interactive
  perf without risking durability on crash for the local collection use
  case.
- Prices are cached and shown stale with a "fetched at" timestamp rather
  than blocking the UI. Refresh is explicit.

## Security posture

- CSP limits content sources to the two catalog APIs and their image CDNs.
- All network requests go through Rust; the webview cannot make arbitrary
  HTTP calls.
- Input validation happens in Rust before it touches SQLite (prepared
  statements everywhere; no string interpolation).
- No plaintext secrets in the repo. API keys come from env.
- Release builds strip symbols, enable LTO, and `panic = "abort"`.

## Out of scope (for now)

- Multi-user accounts or sync between devices.
- Any server component, including "optional" ones.
- Commercial marketplaces beyond showing TCGplayer prices.
- Non-TCG card types (sports cards, etc.). The `Game` enum can grow later.
