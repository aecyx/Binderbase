# Security policy

## Supported versions

Binderbase is pre-1.0. Only the latest `main` branch and the most recent
released version receive security updates.

| Version        | Supported |
| -------------- | --------- |
| `main`         | Yes       |
| latest release | Yes       |
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
