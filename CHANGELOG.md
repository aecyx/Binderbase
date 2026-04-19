<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->

# Changelog

All notable changes to Binderbase are documented in this file.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.0-rc.1] — 2026-04-19

### Added

- **Catalog persistence** — Download full card catalogs (MTG via Scryfall, Pokémon
  via Pokémon TCG API) into a local SQLite database. Bulk import with progress
  events and cancellation support.
- **Card scanning** — Perceptual-hash pipeline identifies cards from photos.
  Build a scan index per game; scan results show matched cards with confidence
  scores. Inline "Add to collection" directly from scan results.
- **Collection management** — Add, view, filter, and delete collection entries.
  Track condition, foil status, and quantity per card. Price column shows best
  cached price.
- **Price lookup** — Search cards by name, view cached prices, and refresh live
  prices on demand (single card or batch refresh across entire collection).
  Rate-limited to respect upstream APIs.
- **CSV import / export** — Export your collection as CSV (optionally filtered by
  game). Import entries from a CSV file with validation and error reporting.
- **Card-name search** — Type-ahead search against the local catalog for fast
  offline card lookup.
- **Settings** — Configure Pokémon TCG API key and data directory via a
  settings page.
- **E2E test suite** — Playwright tests running against the Tauri webview via
  Chrome DevTools Protocol (CDP).

### Technical

- Tauri 2 desktop app (Rust backend, React 19 + TypeScript 6 frontend).
- SQLite with WAL mode, schema versioned (v1 → v3), auto-migration on startup.
- CSP-locked security policy; no cloud dependency; fully local-first.
- CI pipeline: frontend lint/format/typecheck, Rust clippy/fmt/test on
  Linux/macOS/Windows, dependency audit.
- AGPL-3.0-or-later license with SPDX headers on all source files.

[Unreleased]: https://github.com/aecyx/Binderbase/compare/v1.0.0-rc.1...HEAD
[1.0.0-rc.1]: https://github.com/aecyx/Binderbase/releases/tag/v1.0.0-rc.1
