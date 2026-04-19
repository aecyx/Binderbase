# Handoff — Phase 1: Catalog persistence

**For:** Opus 4.6 agent in VS Code
**Repo:** `C:\Code\Binderbase`
**Branch:** whatever the working branch is (the changes are already in the tree; you are verifying + committing, not authoring)

## What changed

Claude wrote these changes directly to the working tree. Do **not** re-author them — treat them as fixed inputs and verify.

New file:

- `src-tauri/src/catalog/mod.rs` — `upsert`, `get`, `search`, `row_to_card`, plus 9 unit tests. SPDX-headed AGPL-3.0-or-later.

Modified files:

- `src-tauri/src/storage/mod.rs` — added `#[cfg(test)] pub(crate) mod test_support` exposing `memory_conn()` so sibling modules' test suites share one migration path.
- `src-tauri/src/commands.rs` — imported `catalog`; refactored `fetch_card` into a local-first flow (cache hit → return; miss → live fetch → upsert → return, upsert errors logged via `tracing::warn!` not propagated); added `catalog_get` and `catalog_search` commands with server-side limit clamp (default 25, max 200).
- `src-tauri/src/lib.rs` — registered `pub mod catalog;` and wired `commands::catalog_get` + `commands::catalog_search` into `tauri::generate_handler!`.
- `src/lib/tauri.ts` — added `api.catalog.get` and `api.catalog.search` typed wrappers.

## Your job

Verify the changes compile, lint, typecheck, and test cleanly on the user's real Windows environment, fix any issues surfaced, then commit.

### 1. Rust backend (`cd src-tauri`)

Run, in order, and do not advance past a failure:

```powershell
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Expected outcomes:

- `cargo fmt --check` — clean. If not, run `cargo fmt` and commit the diff alongside.
- `cargo clippy -- -D warnings` — clean. If clippy flags anything in the new/modified files, fix idiomatically; do **not** silence with `#[allow(...)]` unless the clippy rule is genuinely wrong for the context (and say so in the commit body).
- `cargo test` — all pre-existing tests still pass, **plus** these new catalog tests pass:
  - `catalog::tests::upsert_then_get_round_trips`
  - `catalog::tests::get_returns_none_for_unknown_card`
  - `catalog::tests::upsert_overwrites_changed_fields_and_keeps_one_row`
  - `catalog::tests::search_is_case_insensitive_substring`
  - `catalog::tests::search_filters_by_game`
  - `catalog::tests::search_empty_query_returns_empty`
  - `catalog::tests::search_escapes_like_metacharacters`
  - `catalog::tests::search_respects_limit`
  - `catalog::tests::search_zero_limit_is_invalid_input`

If `cargo test` fails on `storage::test_support` not being findable, the likely cause is `pub(crate)` visibility not being enough across the `#[cfg(test)]` boundary — if so, escalate; don't paper over it.

### 2. Frontend (repo root)

```powershell
npm run lint
npm run typecheck
npm run format:check
```

All three must be clean. If `format:check` fails only because of whitespace, run `npm run format` and fold the diff into the same commit.

### 3. Diff review

Before committing, `git diff --stat` and eyeball:

- No file outside the list above is touched. If anything else changed, figure out why before committing — it was not intentional on Claude's side.
- No SPDX headers were lost on the modified files.

### 4. Commit

Single conventional commit:

```
feat(catalog): add local-first catalog persistence

- New `catalog` module with upsert/get/search over the `cards` table.
- `fetch_card` is now local-first: hits the catalog before the network
  and upserts successful live fetches so repeat lookups are free.
- New `catalog_get` and `catalog_search` commands power autocomplete
  and fast direct lookups from the UI.
- `storage::test_support::memory_conn` shared test helper.
- Typed frontend wrappers `api.catalog.{get,search}` stay in sync.

Phase 1 of the 1.0 roadmap — unblocks `collection::add` (the FK into
`cards` now has rows to point at).
```

Do **not** push. Matt pushes.

### 5. If anything breaks

- Comment the failure inline at the bottom of this file (just a dated line is fine) so Claude has the context on the next turn.
- Leave the failure unfixed unless the fix is obvious and contained — escalate anything that requires a design call.
