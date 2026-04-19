<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->

# Changelog

All notable changes to Binderbase are documented in this file.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Security

- CSV export sanitises formula-trigger characters (`=`, `+`, `-`, `@`, tab, CR)
  to prevent spreadsheet injection (OWASP).
- Scan-identify rejects images > 10 MB (pre-decode) and > 50 megapixels
  (post-decode) to prevent decompression-bomb DoS.
- New `InputTooLarge` error variant surfaces size-limit rejections to the UI.
- Cosign keyless signing of release artifacts via Sigstore OIDC.
- CodeQL static analysis on push/PR (JavaScript/TypeScript + GitHub Actions).
- `cargo-deny` CI check for advisories, banned crates, licenses, and sources.
- Weekly `cargo-fuzz` CI target for the CSV import parser.
- Threat model documented in `docs/THREATMODEL.md`.
- `SECURITY.md` updated with supported versions, accepted risks, and reporting
  instructions.
- Bumped Tauri to 2.10.3, tauri-build to 2.5, tauri-plugin-opener to 2.5.
  Chased all 19 RUSTSEC advisories individually — all are upstream-blocked in
  Tauri 2.x (GTK3 stack, kuchikiki/selectors chain, urlpattern/unic chain).
  Updated `deny.toml` with per-advisory upstream issue links and specific
  comments. Two new advisories added to ignore list: RUSTSEC-2024-0429
  (glib unsound) and RUSTSEC-2026-0097 (rand 0.7 unsound).
- Per-advisory triage table in `docs/security/advisory-triage.md` documenting
  all 19 RUSTSEC advisories with type, CVE, CVSS, upstream blocker, and reason.
- `osv-scanner.toml` with time-bounded ignores (expiry 2027-04-19) for all 19
  non-vulnerability INFO advisories.
- OSV-Scanner CI workflow (weekly + push/PR) uploading SARIF to GitHub Code
  Scanning, SHA-pinned to google/osv-scanner-action v2.3.5.

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
