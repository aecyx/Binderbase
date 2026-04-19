# Security policy

## Supported versions

Only the `1.0.x` line (latest patch) is supported once shipped. Pre-release
lines are unsupported after the corresponding stable release ships.

| Version        | Supported |
| -------------- | --------- |
| `1.0.x`        | Yes       |
| `main`         | Yes       |
| older releases | No        |

## Reporting a vulnerability

**Please do not open a public GitHub issue for security problems.**

Report vulnerabilities privately via GitHub's private advisory flow:

> <https://github.com/aecyx/Binderbase/security/advisories/new>

Include:

- A description of the issue and the impact you believe it has.
- Steps to reproduce, ideally with a minimal sample.
- The version (or commit SHA) you tested against.
- Your preferred contact method, if anything is time-sensitive.

We aim to acknowledge reports within **3 business days** and to provide a
triage decision (accept / need-more-info / not-applicable) within **10
business days**. For accepted reports we will coordinate a fix and a
disclosure timeline with you. We credit reporters in release notes unless you
prefer to remain anonymous.

## Scope

In scope:

- The Binderbase desktop application (Rust backend + React frontend).
- The Tauri command surface and how it validates input.
- The local SQLite store and how migrations, backups, or exports handle data.
- Build/CI configuration that could compromise released binaries.

Out of scope:

- The public APIs we call (Scryfall, Pokemon TCG API). Report issues there to
  the respective maintainers.
- Vulnerabilities in dependencies that do not affect Binderbase as shipped
  (please still let us know so we can upgrade).
- Social engineering and physical attacks.

## Handling of user data

Binderbase is local-first by design. It does not upload your collection or
personal data anywhere. External network calls are limited to the public
catalog APIs listed above, and only as needed to fetch card metadata,
pricing, and images. If a vulnerability could cause Binderbase to exfiltrate
user data, treat it as high severity and report it through the private flow.

## Safe-harbor

We will not pursue legal action against researchers who:

- Act in good faith.
- Avoid privacy violations, destruction of data, and interruption of service.
- Only interact with their own installations and accounts.
- Give us reasonable time to remediate before disclosure.

## Branch protection

The `main` branch is protected by a GitHub ruleset. The intended
policy is documented in
[`.github/branch-protection.yml`](branch-protection.yml). The repo
admin applies the ruleset via the GitHub UI.

### Solo-maintainer tradeoffs

Binderbase is currently single-maintainer, which forces two
Branch-Protection concessions:

1. **Required approving review count is `1`, not `2`.**
   Setting the count to `2` would make merges impossible without a
   co-maintainer or a third-party bot reviewer.
2. **Admin enforcement is off** (`enforce_admins: false`).
   The repo admin must be able to bypass the approval requirement to
   merge their own PRs. With admin enforcement on, no one can approve.

Both cost ~1 point each on the OpenSSF Scorecard Branch-Protection
check. The tradeoffs are accepted; if a second maintainer is added in
the future, both will be tightened in the same PR that adds them to
CODEOWNERS.

Other Branch-Protection settings:

- Force pushes blocked.
- Branch deletion blocked.
- Linear history required.
- Code-owner review required.
- Stale reviews dismissed on new pushes.
- Approval of the most recent reviewable push required.
- All status checks listed in `branch-protection.yml` must pass.

## Accepted risks

The following are known, documented trade-offs accepted for the 1.0 release:

- **Unsigned native binaries.** Windows and macOS binaries are not code-signed
  (no EV/Developer ID certificate). Users verify downloads via cosign keyless
  signatures and SHA256 checksums attached to each release.
- **Transitive unmaintained/unsound dependencies (19 advisories).** All 19
  are upstream-blocked in the Tauri 2.x dependency tree. None are active
  exploitable vulnerabilities (0 CVEs, 0 CVSS scores) — all are `unmaintained`
  or `unsound` INFO-level advisories in code paths we don't exercise directly.
  Specific groups:
  - **GTK3 stack (12 crates):** gtk-rs 0.18 bindings + glib unsound + proc-macro-error.
    Blocked on [tauri-apps/tauri#14684](https://github.com/tauri-apps/tauri/pull/14684)
    (GTK4 migration, targeted at Tauri 3.0) and
    [tauri-apps/tauri#12563](https://github.com/tauri-apps/tauri/issues/12563).
  - **kuchikiki chain (2 crates):** fxhash + rand 0.7. Pulled via
    tauri-utils → kuchikiki (Brave fork) → selectors. Blocked on tauri-utils
    replacing its HTML parser.
  - **unic-\* (5 crates):** Pulled via tauri-utils → urlpattern 0.3.0.
    urlpattern 0.6.0 exists upstream but tauri-utils 2.x pins 0.3.0.

  Full per-advisory triage in [`docs/security/advisory-triage.md`](../../docs/security/advisory-triage.md).
  Suppressed in `osv-scanner.toml` with a 1-year ignoreUntil (2027-04-19).
  Also tracked in `src-tauri/deny.toml` with per-advisory upstream issue links.
  OSV-Scanner CI runs weekly + on push/PR to detect new advisories.

- **MTG bulk import peak RSS ~1 GB.** The Scryfall bulk JSON (~200 MB) is
  loaded into memory. Documented in README system requirements.

## Threat model

See [`docs/THREATMODEL.md`](../docs/THREATMODEL.md) for the full threat model.
