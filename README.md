# Binderbase

A local-first trading-card-game scanner and collection manager. Desktop for 1.0; mobile (with phone-camera scanning) planned.

**Supported games (1.0):** Magic: The Gathering, Pokémon TCG.

## Why local-first?

Your collection data, scans, and lookups stay on your machine. No cloud account, no hosted database, no ongoing subscription. Card catalogs and pricing refresh from public sources (Scryfall, Pokémon TCG API) and are cached locally.

## Tech

- **Shell:** [Tauri 2](https://tauri.app/) — desktop today, iOS/Android support ready when we are.
- **Frontend:** React 19 + TypeScript + Vite.
- **Backend:** Rust (SQLite via rusqlite, image processing via image, HTTP via reqwest).
- **Storage:** Local SQLite database in the platform app-data directory.

## Prerequisites

- **Node.js 22+** and **npm** (ships with Node).
- **Rust** (install via [rustup](https://www.rust-lang.org/learn/get-started#installing-rust)).
- **OS prerequisites for Tauri:** see <https://tauri.app/start/prerequisites/> (WebView2 on Windows, GTK/WebKit on Linux).

## Quick start

```bash
npm install
npm run tauri dev     # desktop dev build with HMR
```

To produce a release binary for your platform:

```bash
npm run tauri build
```

## Mobile (planned, not 1.0)

Mobile targets are initialized per platform:

```bash
npm run tauri android init     # requires Android Studio / NDK
npm run tauri ios init         # macOS only; requires Xcode
```

See [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) for how the codebase is structured to share logic between desktop and mobile.

See [`docs/OPERATIONS.md`](docs/OPERATIONS.md) for repo, CI, and release configuration — including branch protection and required status checks.

## Repository layout

```
binderbase/
├── src/                     # React / TypeScript frontend
├── src-tauri/               # Rust backend
├── docs/                    # Architecture notes, decision records
└── .github/workflows/       # CI
```

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md) for setup, coding conventions, and the pre-PR checklist.

## Security

To report a vulnerability, see [`SECURITY.md`](SECURITY.md). Please do **not** open a public issue for security problems.

## License

Binderbase is licensed under the [GNU AGPL v3.0-or-later](LICENSE). If you modify and distribute Binderbase — or run a modified version that users interact with over a network — you must make the corresponding source available under the same terms.
