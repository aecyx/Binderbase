# Binderbase

[![License](https://img.shields.io/badge/license-AGPL--3.0--or--later-blue.svg)](./LICENSE)
[![CI](https://github.com/aecyx/Binderbase/actions/workflows/ci.yml/badge.svg)](https://github.com/aecyx/Binderbase/actions/workflows/ci.yml)
[![OpenSSF Scorecard](https://api.securityscorecards.dev/projects/github.com/aecyx/Binderbase/badge)](https://securityscorecards.dev/viewer/?uri=github.com/aecyx/Binderbase)

A local-first trading-card-game scanner and collection manager.

<!-- TODO: add screenshot -->

## What it does

Binderbase identifies cards from scans or manual lookups, tracks your
collection with quantities and conditions, and fetches current market prices —
all stored in a local SQLite database on your machine.

## Who it's for

Collectors and players who want to catalog their cards without creating cloud
accounts or paying for a subscription. If you want your data under your control,
this is for you.

## Why local-first?

Your collection data, scans, and lookups stay on your machine. No cloud
account, no hosted database, no ongoing subscription. Card catalogs and pricing
refresh from public sources (Scryfall, Pokémon TCG API) and are cached locally.

## Supported games

- Magic: The Gathering
- Pokémon TCG

## Status

Pre-1.0. The scaffold is in place (Tauri shell, React frontend, Rust backend,
SQLite storage) but there is no downloadable release yet. Expect breaking
changes.

## Quick start

```bash
git clone https://github.com/aecyx/Binderbase.git
cd Binderbase
npm install
npm run tauri dev
```

Requires Node.js 22+, Rust (via rustup), and the
[Tauri prerequisites](https://tauri.app/start/prerequisites/) for your OS. See
[`CONTRIBUTING.md`](CONTRIBUTING.md) for the full development setup.

## Mobile

Mobile targets (iOS, Android) are planned but not part of the 1.0 scope. The
Tauri 2 shell supports them when we are ready.

## Links

- [Contributing](CONTRIBUTING.md) — development setup and PR expectations
- [Security](SECURITY.md) — vulnerability reporting
- [Code of Conduct](CODE_OF_CONDUCT.md) — community standards
- [Governance](GOVERNANCE.md) — project decision-making
- [Architecture](docs/ARCHITECTURE.md) — codebase structure
- [Operations](docs/OPERATIONS.md) — CI, release, and repo configuration

## License

[AGPL-3.0-or-later](LICENSE). If you modify and distribute Binderbase — or run
a modified version that users interact with over a network — you must make the
corresponding source available under the same terms.
