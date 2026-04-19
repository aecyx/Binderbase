# Threat model

This document describes the security-relevant architecture of Binderbase and
the threats considered during design.

## Assets

| Asset                         | Sensitivity                                             | Storage                                                                                                              |
| ----------------------------- | ------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------- |
| User's collection database    | Personal (card inventory, notes, prices)                | Local SQLite file (WAL mode)                                                                                         |
| PTCGAPI key                   | Secret (bearer token)                                   | OS keychain (macOS Keychain, Windows Credential Manager, Linux Secret Service); in-memory fallback with user warning |
| Card catalog                  | Public (derived from Scryfall / PTCGAPI)                | Local SQLite `cards` table                                                                                           |
| Scan index (thumbnail hashes) | Non-sensitive (perceptual hashes of public card images) | Local SQLite `card_hashes` table                                                                                     |

## Trust boundaries

```
┌─────────────────────────────────┐
│  User (trusted)                 │
│  └─ Tauri webview (React UI)   │
│       │  IPC (invoke_handler)   │
│       ▼                         │
│  Rust backend                   │
│  └─ SQLite (local file)        │
└────────┬────────────────────────┘
         │ HTTPS only
         ▼
┌─────────────────────────────────┐
│  Scryfall API (no auth)         │
│  PTCGAPI (bearer token)         │
│  (untrusted network)            │
└─────────────────────────────────┘
```

- **User ↔ App:** Trusted. The webview and Rust backend run in the same
  process on the user's machine.
- **App ↔ Scryfall / PTCGAPI:** HTTPS only. No auth for Scryfall; bearer
  token for PTCGAPI. Responses are validated before insertion into SQLite.
- **App ↔ Local filesystem:** Trusted. SQLite DB, scan images, and config
  live under user-writable directories.

## Entry points (Tauri command surface)

All commands are explicitly registered in `lib.rs::generate_handler!`:

| Command                                                          | Input                    | Notes                           |
| ---------------------------------------------------------------- | ------------------------ | ------------------------------- |
| `app_info`                                                       | none                     | Read-only app metadata          |
| `fetch_card`                                                     | game, card_id            | Catalog lookup                  |
| `catalog_get` / `catalog_search`                                 | game, query string       | Read-only DB query              |
| `catalog_import_start` / `_cancel` / `_status`                   | game                     | Network fetch → DB write        |
| `collection_list` / `_add` / `_remove`                           | game, entry data         | CRUD on collection              |
| `collection_export_csv`                                          | game filter              | Read → CSV string               |
| `collection_import_preview` / `_apply`                           | CSV text (user-supplied) | Parse → validate → DB write     |
| `scan_identify`                                                  | image bytes, game hint   | Decode → hash → DB lookup       |
| `scan_build_index` / `_cancel` / `_status`                       | game                     | Network fetch → hash → DB write |
| `pricing_get_cached` / `pricing_refresh` / `_refresh_collection` | game, card_ids           | DB read / network fetch         |
| `settings_get_ptcgapi_key` / `_set_ptcgapi_key`                  | key string               | OS keychain read/write          |

## Threats and mitigations (STRIDE-lite)

### Tampering with release binaries

**Threat:** An attacker substitutes a malicious binary on the download path.

**Mitigation:** Cosign keyless signing of every release artifact via GitHub
OIDC + Fulcio. SHA256 checksums attached to each release. Verification
instructions in README.

### Repudiation of updates

**Threat:** A compromised CI pipeline produces a signed but malicious build
without an audit trail.

**Mitigation:** Every cosign signature is recorded in the Rekor transparency
log, providing a tamper-evident, publicly auditable record of every signing
event.

### Information disclosure via keychain

**Threat:** PTCGAPI key leaks from the OS credential store.

**Mitigation:** Key is stored via platform-native APIs (macOS Keychain,
Windows DPAPI, Linux Secret Service). The in-memory fallback warns the user
that the key will not persist and is only held for the session lifetime.

### DoS via malicious image (scan)

**Threat:** `image::load_from_memory` decompresses arbitrary user input. A
crafted PNG/TIFF can act as a decompression bomb (tiny file, enormous decoded
buffer) that OOMs the app.

**Mitigation:** (Implemented in Phase 3)

1. Reject images larger than 10 MB on the wire.
2. After decode, reject images with `width × height > 50,000,000` pixels.
3. Surface as `Error::InputTooLarge` with a user-friendly message.

### CSV injection on export

**Threat:** Card names or notes beginning with `=`, `+`, `-`, `@`, `\t`, or
`\r` are treated as formulas when the exported CSV is opened in Excel or
LibreOffice — potential for credential exfiltration or command execution.

**Mitigation:** (Implemented in Phase 3) Prefix dangerous first characters
with a single-quote `'` (OWASP-recommended escape). Round-trip tested.

### Supply-chain tampering

**Threat:** A compromised dependency introduces malicious code.

**Mitigation:**

- `cargo-deny` in CI checks advisories, licenses, bans, and sources.
- All third-party GitHub Actions SHA-pinned to release commits.
- CodeQL static analysis on every PR.
- Dependabot enabled for automated dependency updates.

## Non-goals

- No network-based multi-user sync.
- No cloud storage or remote database.
- No telemetry, analytics, or crash reporting.
- No code-signing certificates (Windows EV, Apple Developer ID) — deferred;
  cosign + SHA256 verification is the interim trust mechanism.

## Accepted risks

These are the same items listed in `SECURITY.md` for auditor consistency:

- **Unsigned native binaries.** Users verify via cosign + SHA256.
- **Transitive unmaintained dependencies via Tauri 2's GTK stack on Linux.**
  Tracked in `src-tauri/deny.toml` with 6-month review cadence.
- **MTG bulk import peak RSS ~1 GB.** Documented in README system
  requirements.
