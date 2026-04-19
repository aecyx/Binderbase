# Binderbase

[![License](https://img.shields.io/badge/license-AGPL--3.0--or--later-blue.svg)](./LICENSE)
[![CI](https://github.com/aecyx/Binderbase/actions/workflows/ci.yml/badge.svg)](https://github.com/aecyx/Binderbase/actions/workflows/ci.yml)
[![OpenSSF Scorecard](https://api.securityscorecards.dev/projects/github.com/aecyx/Binderbase/badge)](https://securityscorecards.dev/viewer/?uri=github.com/aecyx/Binderbase)

<!-- [![CII Best Practices](https://www.bestpractices.dev/projects/XXXX/badge)](https://www.bestpractices.dev/projects/XXXX) -->

A local-first trading-card-game scanner and collection manager.

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

Release candidate. Download the latest build from
[Releases](https://github.com/aecyx/Binderbase/releases).

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

## System requirements

- **OS:** Windows 10+, macOS 12+, or a Linux desktop with WebKitGTK 4.1
- **RAM:** 4 GB minimum. The MTG catalog import temporarily holds ~200 MB of
  card data in memory; peak RSS may reach ~1 GB during import.
- **Disk:** ~2 GB free if you build the full scan index (thumbnail downloads
  for ~42 k cards across both games).
- **Network:** Catalog import and scan-index builds make many HTTP requests to
  Scryfall / Pokémon TCG API. A stable connection is recommended; there is no
  resume — if the connection drops, you'll need to restart the operation.

## Installing unsigned binaries

Binderbase is not currently code-signed. On first launch your OS may show a
warning:

- **Windows SmartScreen:** Click _More info → Run anyway_.
- **macOS Gatekeeper:** Right-click the app → _Open_ → confirm.
- **Linux:** No extra step needed for AppImage/deb packages.

You can verify the download using the `SHA256SUMS.txt` file attached to each
[release](https://github.com/aecyx/Binderbase/releases).

### Verifying with cosign

Release artifacts are signed using **keyless signing via GitHub OIDC +
Sigstore Fulcio** — no private key management on our side. Every signature is
publicly auditable in the [Rekor](https://rekor.sigstore.dev/) transparency
log. Each release asset has a corresponding `.bundle` file containing the
signature and certificate.

To verify a download (requires [cosign](https://docs.sigstore.dev/cosign/system_config/installation/) v3+):

```bash
cosign verify-blob \
  --bundle Binderbase_1.0.0-rc.1_x64.msi.bundle \
  --certificate-identity-regexp 'https://github.com/aecyx/Binderbase/\.github/workflows/release\.yml@refs/tags/v.+' \
  --certificate-oidc-issuer 'https://token.actions.githubusercontent.com' \
  Binderbase_1.0.0-rc.1_x64.msi
```

Replace the file names with the actual artifact you downloaded.

## Scanning limitations

The card scanner currently works best with **cleanly cropped card images**
(single card filling most of the frame). Full phone-photo support with
automatic card detection, perspective correction, and deskew is planned for a
future release.

## Data sources

- **Magic: The Gathering** card data from [Scryfall](https://scryfall.com).
  Binderbase is not produced by or endorsed by Scryfall.
- **Pokémon TCG** card data from the
  [Pokémon TCG API](https://pokemontcg.io). Binderbase is not produced by or
  endorsed by Pokémon TCG API.

## Privacy

Binderbase does **not** collect or transmit any user data. There is no
telemetry, analytics, or crash reporting. The only network calls are to the
Scryfall and Pokémon TCG API servers to fetch card data and prices. All
collection data stays on your machine.

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
