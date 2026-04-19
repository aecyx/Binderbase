# Architecture decision register

This file indexes architecture decision records (ADRs) for Binderbase. Each
ADR captures one decision, the context that made it necessary, the options
considered, and the consequences. New decisions get a new ADR entry with a
monotonically increasing number.

> Format loosely based on Michael Nygard's ADR template. Keep each record
> short — link out to longer design docs if needed.

## Status legend

- **Proposed** — under discussion, not acted on yet.
- **Accepted** — currently in force.
- **Superseded** — replaced by a later ADR (linked).
- **Deprecated** — we no longer do this, but nothing replaces it.

---

## ADR-0001: Use Tauri 2 + React/TypeScript for the desktop app

- **Status:** Accepted
- **Date:** 2026-04-17
- **Deciders:** aecyx

### Context

Binderbase needs a cross-platform desktop UI that can do non-trivial local
work (SQLite, image processing, HTTP to card APIs) and, eventually, run on
mobile with access to the camera. The work must be doable by a small team
and keep the binary size reasonable.

### Decision

Build Binderbase as a Tauri 2 application. Rust handles native I/O and
business logic. The UI is React + TypeScript, bundled by Vite.

### Alternatives considered

- **Electron + Node.** Mature and well-documented, but ships a full Chromium
  runtime per app, has a much larger install size, and has no first-class
  path to mobile. Native work would be in JS or a separate Rust sidecar.
- **Native per-platform (SwiftUI + WinUI + GTK).** Best UX on each
  platform, but triples the maintenance burden and contradicts the
  "small team" constraint.
- **Flutter.** Strong mobile story, but less mature for desktop and adds a
  new language to a Rust-favoring stack.
- **Plain Rust + egui/iced.** Keeps the stack homogenous but loses access
  to the React ecosystem for accessibility and styling primitives we care
  about.

### Consequences

- We are committed to Rust for backend logic. New contributors need Rust
  familiarity.
- Tauri 2's mobile support is still maturing. We accept that risk and keep
  all Rust deps mobile-compatible (`rustls-tls`, `rusqlite` bundled, etc.).
- We get a small binary, tight CSP control, and a typed command bridge for
  free.

---

## ADR-0002: Local-first, no cloud services

- **Status:** Accepted
- **Date:** 2026-04-17
- **Deciders:** aecyx

### Context

Binderbase manages a user's collection, which is personal data some users
consider sensitive (or at least private). Running a backend would add
operational cost, expand the attack surface, and create a dependency that
outlives the maintainer's willingness to pay for hosting.

### Decision

Binderbase is local-first. All user data lives in SQLite in the OS's app-data
directory. External network is limited to public catalog APIs (Scryfall,
Pokemon TCG API) for reference metadata, prices, and images. No user
accounts, no sync, no telemetry.

### Alternatives considered

- **Optional cloud sync.** Rejected for 1.0: even "optional" services
  require us to run and secure them, and tend to become mandatory over time.
- **Third-party BaaS.** Same operational cost concerns, plus ties user data
  to a vendor.

### Consequences

- Users on multiple machines will need to copy their database manually (or
  via their own sync tool like iCloud/OneDrive/Syncthing).
- We avoid all of GDPR's data-controller complexity for our own servers,
  though we still link to APIs we don't operate.
- If we ever add sync, it must be end-to-end encrypted with keys the user
  holds. See placeholder ADR-0005 (not yet written).

---

## ADR-0003: License under AGPL-3.0-or-later

- **Status:** Accepted
- **Date:** 2026-04-17
- **Deciders:** aecyx

### Context

Binderbase is open source. We want modifications that are shipped to users
(including via a network interface, should one appear later) to be released
under the same terms.

### Decision

License the project under **GNU Affero General Public License v3.0 or later**.
Every source file references the license via the repo-root `LICENSE` file
and the license field in `Cargo.toml` / `package.json`.

### Alternatives considered

- **MIT / Apache-2.0.** Simpler and more widely adopted, but allow closed-
  source forks. Rejected because we want copyleft.
- **GPL-3.0.** Strong copyleft for distribution but doesn't cover the
  "provided over a network" case, which matters if we ever add a web
  companion.
- **MPL-2.0.** File-level copyleft, too permissive for our preference.

### Consequences

- Commercial users can still use Binderbase, but if they redistribute a
  modified version (including over a network) they must release their
  modifications under AGPL.
- Some enterprises forbid AGPL internally. That is acceptable — we are
  building for end users, not enterprises.
- Contributions are accepted under the same license (stated in
  `CONTRIBUTING.md`).

---

## ADR-0004: SQLite via `rusqlite` with the `bundled` feature

- **Status:** Accepted
- **Date:** 2026-04-17
- **Deciders:** aecyx

### Context

Every user install needs a reliable local database. System-installed SQLite
versions vary wildly across platforms (especially Linux distros) and
require a build-time dependency that complicates mobile ports.

### Decision

Use `rusqlite = { features = ["bundled", "chrono"] }`. Compile SQLite into
the binary. Every user gets the same SQLite version we tested against.

### Alternatives considered

- **System SQLite.** Smaller binary, but version drift and mobile
  portability issues.
- **`sqlx` with compile-time query checking.** Nice ergonomics, but async
  everywhere adds complexity we don't need for a desktop app, and it
  doesn't offer the same mobile story out of the box.
- **Embedded key-value (sled, redb).** Fine for simple use but loses SQL,
  ad-hoc queries, and the mature migration tooling.

### Consequences

- Binary size goes up by ~1–2 MB. Acceptable.
- We own the SQLite version pin; bumps are deliberate.
- WAL mode + `synchronous = NORMAL` gives us good interactive perf. Durability
  concerns are addressed by keeping writes small and the DB in the app-data
  dir (not on removable media).

---

## ADR-0005: Perceptual hashing (dHash) for card identification

**Status**: Accepted  
**Date**: 2025-01-01  
**Deciders**: AI-assisted design

### Context

The scanning feature needs to match a user-uploaded card photo against a
catalog of ~100k+ cards. The approach must work fully offline, run on
commodity hardware, and avoid large ML model downloads.

### Decision

Use **dHash (difference hash)** — a 256-bit perceptual hash (17×16 grayscale,
horizontal difference). Match by Hamming distance with confidence = 1 − (distance / 256).

- Hash storage in SQLite (`card_hashes` table, BLOB column).
- Index built on-demand by downloading card thumbnails and hashing them.
- Nearest-neighbor search is a brute-force linear scan (fast enough for <200k entries).

### Alternatives considered

| Option                              | Pros                       | Cons                                     |
| ----------------------------------- | -------------------------- | ---------------------------------------- |
| pHash (DCT-based)                   | More robust to scaling     | Heavier computation, external crate      |
| Learned embeddings (CLIP/MobileNet) | Highest accuracy           | Large model, ONNX runtime, complex       |
| OCR-first                           | Works for text-heavy cards | Fails on art-only cards, language issues |

### Consequences

- Simple, zero-dependency implementation (only `image` crate).
- Good-enough accuracy for near-exact matches; may need augmentation (rotation, crop) later.
- Linear scan is O(n) per query; if catalog grows past ~500k, consider VP-tree or LSH.

---

## How to add a new ADR

1. Pick the next free number.
2. Append a new section under `---` with `## ADR-NNNN: <title>`.
3. Fill in status, date, deciders, context, decision, alternatives, and
   consequences. Keep it under a page.
4. When a later ADR supersedes an earlier one, update both statuses and
   link between them.
5. Reference the ADR number from code comments when a non-obvious choice is
   downstream of one.
