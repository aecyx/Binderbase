# Contributing to Binderbase

Thanks for your interest in Binderbase! This document describes how to set up a
development environment and the expectations for contributions.

Binderbase is licensed under **AGPL-3.0-or-later**. By submitting a pull
request, you agree that your contribution will be distributed under the same
license.

## Prerequisites

- **Node.js** 22 or later (see `.nvmrc`). `nvm use` will pick the right version.
- **Rust** stable, installed via [rustup](https://rustup.rs/). The toolchain is
  pinned in `rust-toolchain.toml`, so rustup will install the correct channel
  and components (`rustfmt`, `clippy`) automatically the first time you build.
- **Platform build deps for Tauri 2** — follow the official guide:
  <https://tauri.app/start/prerequisites/>. On Linux you need WebKit2GTK,
  GTK3, and a few others (`libwebkit2gtk-4.1-dev`, `libgtk-3-dev`,
  `libayatana-appindicator3-dev`, `librsvg2-dev`, `libssl-dev`,
  `build-essential`).

## Getting started

```bash
git clone https://github.com/aecyx/Binderbase.git
cd Binderbase
npm install
npm run tauri dev
```

The first Rust build takes a while — subsequent builds are incremental.

## Project layout

```
Binderbase/
├── src/                  # React + TypeScript frontend
│   ├── components/       # Shared UI
│   ├── features/         # scan, collection, pricing, import_export
│   ├── lib/              # Frontend helpers (Tauri invoke wrapper, etc.)
│   ├── styles/           # Design tokens and global CSS
│   └── types/            # TypeScript mirrors of Rust types
├── src-tauri/            # Rust backend
│   └── src/
│       ├── core/         # Shared types and errors
│       ├── games/        # Per-game adapters (MTG, Pokemon)
│       ├── storage/      # SQLite + migrations
│       ├── catalog/      # Local card metadata cache + bulk import
│       ├── collection/   # Local collection CRUD
│       ├── pricing/      # Cached prices
│       ├── scanning/     # Image identification
│       ├── settings/     # Preferences (SQLite) + secrets (OS keychain)
│       └── commands/     # Tauri command surface (one file per domain)
├── docs/                 # Architecture notes and ADRs
└── .github/              # CI, issue/PR templates, community health docs
```

## Coding standards

### File organization

Every file should have a single, clear responsibility. When a module grows
beyond ~300 lines of production code (excluding tests), split it into a
directory module (`mod.rs` + submodules). The `mod.rs` re-exports the public
surface so callers never need to know about the internal split.

### TypeScript / React

- Format with Prettier (`npm run format`) before committing.
- Lint clean: `npm run lint` must pass with zero warnings.
- `npm run typecheck` must pass.
- Keep components accessible: label every form control, set `role`/`aria-*`
  attributes where needed, ensure keyboard navigation works, maintain 4.5:1
  color contrast for text.
- Wrap every backend call in the typed `api.*` functions from `src/lib/tauri.ts`.
  Avoid calling `invoke` directly from components.

### Rust

- Format with `cargo fmt --all` before committing.
- `cargo clippy --all-targets --all-features -- -D warnings` must pass.
- Return `core::Error` from fallible Tauri commands — do not leak
  `anyhow::Error` or string errors across the boundary.
- Prefer `thiserror` variants for new error cases; match them explicitly in
  callers where recovery matters.
- Do not introduce unsafe code without an accompanying SAFETY comment and a
  design note in `docs/DECISIONS.md`.

### SQL / storage

- Schema changes go in a new `schema_vN.sql` file, never by editing an existing
  one. Update `SCHEMA_VERSION` and add a migration step.
- All new tables should include `created_at` and `updated_at` columns where it
  aids auditing.

## Commit messages

We follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat(collection): support bulk add from CSV
fix(pricing): handle missing USD field from Scryfall
docs: add ADR-0003 for the scan pipeline
chore(deps): bump reqwest to 0.12.9
```

Scopes should match module directories where it makes sense (`collection`,
`pricing`, `scanning`, `games`, `storage`, `ui`, `docs`, `ci`, `deps`).

## Pre-PR checklist

Before opening a pull request:

- [ ] `npm run format:check`
- [ ] `npm run lint`
- [ ] `npm run typecheck`
- [ ] `npm run build`
- [ ] `cd src-tauri && cargo fmt --all -- --check`
- [ ] `cd src-tauri && cargo clippy --all-targets --all-features -- -D warnings`
- [ ] `cd src-tauri && cargo test`
- [ ] Manual smoke test of any UI or command surface you touched.

CI runs the same checks on every PR. If CI fails, fix it before asking for a
review.

## Non-negotiables

- **No cloud services.** Binderbase is desktop-first and local-first. External
  HTTP calls are limited to the public card catalog APIs (Scryfall, Pokemon
  TCG API). Do not add analytics, crash reporting that phones home, remote
  feature flags, or anything that requires us to run infrastructure.
- **No telemetry without opt-in.** Any future telemetry must be off by
  default, clearly disclosed in the UI, and document exactly what is sent.
- **No secrets in the repo.** Use `.env` files locally; add to `.env.example`
  with blank values if a new knob is introduced.
- **No copyrighted card images bundled.** We link to publicly hosted images
  and cache them locally at runtime only.

## Reporting bugs and asking for features

Use the issue templates under `.github/ISSUE_TEMPLATE/`. For security issues
see `SECURITY.md` — do **not** open a public issue for vulnerabilities.

## Code of conduct

Be kind. Assume good faith. If a discussion gets heated, step away. Personal
attacks, harassment, or discrimination will result in removal from the project.
