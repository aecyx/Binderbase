# CII Best Practices — Self-Assessment Draft

> **Purpose:** Pre-fill for the [OpenSSF Best Practices](https://www.bestpractices.dev/)
> questionnaire. A maintainer can transfer answers directly into the web form.
>
> **Sourced from:** [bestpractices.dev passing criteria](https://www.bestpractices.dev/en/criteria/0)
> (67 criteria, scraped 2025-07-17).
>
> **Status key:** Met | Unmet | N/A

## Summary

| Category       |    Met | Unmet |    N/A |  Total |
| -------------- | -----: | ----: | -----: | -----: |
| Basics         |     13 |     0 |      0 |     13 |
| Change Control |      8 |     0 |      1 |      9 |
| Reporting      |      7 |     0 |      1 |      8 |
| Quality        |     11 |     2 |      0 |     13 |
| Security       |      7 |     0 |      9 |     16 |
| Analysis       |      6 |     0 |      2 |      8 |
| **Total**      | **52** | **2** | **13** | **67** |

All MUST and SHOULD criteria are Met or N/A.
The 2 Unmet items are both SUGGESTED-level and do not block a passing badge.

---

## Basics

### Basic project website content

#### description_good — Project description

> The project website MUST succinctly describe what the software does (what
> problem does it solve?). This MUST be in language that potential users can
> understand (e.g., it uses minimal jargon).

**Status:** Met
**Evidence:** [README.md](../../README.md) — "A local-first trading-card-game
scanner and collection manager" followed by "What it does", "Who it's for", and
"Why local-first?" sections in plain language.

---

#### interact — How to obtain, give feedback, contribute

> The project website MUST provide information on how to: obtain, provide
> feedback (as bug reports or enhancements), and contribute to the software.

**Status:** Met
**Evidence:** [README.md](../../README.md) links to the GitHub repository (clone
instructions). [CONTRIBUTING.md](../../.github/CONTRIBUTING.md) explains how to
contribute. GitHub Issues are open for bug reports and enhancement requests.

---

#### contribution — Contribution process

> The information on how to contribute MUST explain the contribution process
> (e.g., are pull requests used?).

**Status:** Met
**Evidence:** [CONTRIBUTING.md](../../.github/CONTRIBUTING.md) documents the
pull-request workflow, coding standards, commit conventions, and pre-PR
checklist.

---

#### contribution_requirements — Contribution standards (SHOULD)

> The information on how to contribute SHOULD include the requirements for
> acceptable contributions (e.g., a reference to any required coding standard).

**Status:** Met
**Evidence:** [CONTRIBUTING.md](../../.github/CONTRIBUTING.md) specifies Prettier
formatting, ESLint zero-warning policy, clippy `-D warnings`, conventional
commit format, and a full pre-PR checklist.

---

### FLOSS license

#### floss_license — Released as FLOSS

> The software produced by the project MUST be released as FLOSS.

**Status:** Met
**Evidence:** Licensed under AGPL-3.0-or-later. [LICENSE](../../LICENSE) at
repository root.

---

#### floss_license_osi — OSI-approved license (SUGGESTED)

> It is SUGGESTED that any required license(s) for the software produced by the
> project be approved by the OSI.

**Status:** Met
**Evidence:** AGPL-3.0-or-later is
[OSI-approved](https://opensource.org/licenses/AGPL-3.0).

---

#### license_location — License in standard location

> The project MUST post the license(s) of its results in a standard location in
> their source repository.

**Status:** Met
**Evidence:** [LICENSE](../../LICENSE) at repository root.

---

### Documentation

#### documentation_basics — Basic documentation

> The project MUST provide basic documentation for the software produced by the
> project.

**Status:** Met
**Evidence:** [README.md](../../README.md) covers what the software does,
prerequisites, getting started (clone + `npm install` + `npm run tauri dev`),
and supported games. [CONTRIBUTING.md](../../.github/CONTRIBUTING.md) provides
detailed build instructions.

---

#### documentation_interface — External interface reference

> The project MUST provide reference documentation that describes the external
> interface (both input and output) of the software produced by the project.

**Status:** Met
**Evidence:** Binderbase is a desktop GUI application — the external interface is
the graphical user interface itself. [README.md](../../README.md) describes the
feature set (scanning, collection management, pricing). No programmatic API is
exposed to end users.
If the questionnaire expects formal user-guide documentation beyond the README,
change status to Unmet and note that a user guide is planned.

---

### Other

#### sites_https — HTTPS on project sites

> The project sites (website, repository, and download URLs) MUST support HTTPS
> using TLS.

**Status:** Met
**Evidence:** Repository at `https://github.com/aecyx/Binderbase`. Project
website at `https://binderbase.app`. GitHub Releases served over HTTPS.

---

#### discussion — Searchable discussion mechanism

> The project MUST have one or more mechanisms for discussion (including proposed
> changes and issues) that are searchable, allow messages and topics to be
> addressed by URL, enable new people to participate in some of the discussions,
> and do not require client-side installation of proprietary software.

**Status:** Met
**Evidence:** GitHub Issues and Pull Requests. Both are searchable,
URL-addressable, open to the public, and require no proprietary client software.

---

#### english — Documentation in English (SHOULD)

> The project SHOULD provide documentation in English and be able to accept bug
> reports and comments about code in English.

**Status:** Met
**Evidence:** All documentation, issues, and code comments are in English.

---

#### maintained — Project is maintained

> The project MUST be maintained.

**Status:** Met
**Evidence:** Active commits on `main`, CI passing, dependencies actively
managed, pre-release (1.0.0-rc.1) in progress.

---

## Change Control

### Public version-controlled source repository

#### repo_public — Public VCS repository

> The project MUST have a version-controlled source repository that is publicly
> readable and has a URL.

**Status:** Met
**Evidence:** `https://github.com/aecyx/Binderbase` — public GitHub repository.

---

#### repo_track — Track changes, authors, dates

> The project's source repository MUST track what changes were made, who made
> the changes, and when the changes were made.

**Status:** Met
**Evidence:** Git history tracks all changes, authors, and timestamps.

---

#### repo_interim — Interim versions between releases

> To enable collaborative review, the project's source repository MUST include
> interim versions for review between releases; it MUST NOT include only final
> releases.

**Status:** Met
**Evidence:** The repository contains regular commits between tagged releases.
Development is done via feature branches merged through pull requests.

---

#### repo_distributed — Distributed VCS (SUGGESTED)

> It is SUGGESTED that common distributed version control software be used
> (e.g., git) for the project's source repository.

**Status:** Met
**Evidence:** Git (hosted on GitHub).

---

### Unique version numbering

#### version_unique — Unique version identifier per release

> The project results MUST have a unique version identifier for each release
> intended to be used by users.

**Status:** Met
**Evidence:** Semantic versioning — current release is `1.0.0-rc.1`, consistent
across `package.json`, `Cargo.toml`, and `tauri.conf.json`.

---

#### version_semver — Semantic versioning (SUGGESTED)

> It is SUGGESTED that the Semantic Versioning format be used for releases.

**Status:** Met
**Evidence:** `1.0.0-rc.1` follows SemVer (including pre-release identifier).

---

#### version_tags — Git tags for releases (SUGGESTED)

> It is SUGGESTED that projects identify each release within their version
> control system. For example, it is SUGGESTED that those using git identify
> each release using git tags.

**Status:** Met
**Evidence:** Release workflow triggers on `v*` tags. The `release.yml` workflow
builds and publishes artifacts when a tag is pushed.

---

### Release notes

#### release_notes — Human-readable release notes

> The project MUST provide, in each release, release notes that are a
> human-readable summary of major changes in that release.

**Status:** Met
**Evidence:** [CHANGELOG.md](../../CHANGELOG.md) follows
[Keep a Changelog](https://keepachangelog.com/) format with categorized entries
(Added, Security, Technical).

---

#### release_notes_vulns — CVEs noted in release notes

> The release notes MUST identify every publicly known run-time vulnerability
> fixed in this release that already had a CVE assignment or similar when the
> release was created.

**Status:** N/A
**Rationale:** No CVEs have been assigned against Binderbase. The 19 transitive
dependency advisories are documented in
[advisory-triage.md](advisory-triage.md) but none are project-level CVEs.

---

## Reporting

### Bug-reporting process

#### report_process — Bug report process

> The project MUST provide a process for users to submit bug reports (e.g.,
> using an issue tracker or a mailing list).

**Status:** Met
**Evidence:** GitHub Issues with issue templates under `.github/ISSUE_TEMPLATE/`.

---

#### report_tracker — Issue tracker (SHOULD)

> The project SHOULD use an issue tracker for tracking individual issues.

**Status:** Met
**Evidence:** GitHub Issues.

---

#### report_responses — Respond to bug reports

> The project MUST acknowledge a majority of bug reports submitted in the last
> 2-12 months (inclusive); the response need not include a fix.

**Status:** Met
**Evidence:** Single-maintainer project (@aecyx) actively responding to issues.

---

#### enhancement_responses — Respond to enhancements (SHOULD)

> The project SHOULD respond to a majority (>50%) of enhancement requests in
> the last 2-12 months (inclusive).

**Status:** Met
**Evidence:** Maintainer responds to enhancement requests via GitHub Issues.

---

#### report_archive — Publicly archived reports

> The project MUST have a publicly available archive for reports and responses
> for later searching.

**Status:** Met
**Evidence:** GitHub Issues are publicly searchable and permanently archived.

---

### Vulnerability report process

#### vulnerability_report_process — Published vuln reporting process

> The project MUST publish the process for reporting vulnerabilities on the
> project site.

**Status:** Met
**Evidence:** [SECURITY.md](../../.github/SECURITY.md) documents the
vulnerability reporting process, supported versions, and accepted risks.

---

#### vulnerability_report_private — Private vuln reporting

> If private vulnerability reports are supported, the project MUST include how
> to send the information in a way that is kept private.

**Status:** Met
**Evidence:** [SECURITY.md](../../.github/SECURITY.md) directs reporters to use
GitHub's private security advisory feature ("Report a vulnerability" button).

---

#### vulnerability_report_response — 14-day initial response

> The project's initial response time for any vulnerability report received in
> the last 6 months MUST be less than or equal to 14 days.

**Status:** N/A
**Rationale:** No vulnerability reports have been received in the last 6 months.

---

## Quality

### Working build system

#### build — Working build system

> If the software produced by the project requires building for use, the project
> MUST provide a working build system that can automatically rebuild the
> software from source code.

**Status:** Met
**Evidence:** `npm run tauri build` produces platform-specific installers.
`cargo build` for the Rust backend, `npm run build` for the frontend. CI runs
the full build on every push/PR.

---

#### build_common_tools — Common build tools (SUGGESTED)

> It is SUGGESTED that common tools be used for building the software.

**Status:** Met
**Evidence:** Cargo (Rust), npm (Node.js), Vite (frontend bundler), Tauri CLI.

---

#### build_floss_tools — Buildable with FLOSS tools (SHOULD)

> The project SHOULD be buildable using only FLOSS tools.

**Status:** Met
**Evidence:** All build tools are FLOSS: Rust/Cargo (MIT/Apache-2.0),
Node.js/npm (MIT), Vite (MIT), Tauri (MIT/Apache-2.0).

---

### Automated test suite

#### test — Automated test suite

> The project MUST use at least one automated test suite that is publicly
> released as FLOSS. The project MUST clearly show or document how to run the
> test suite(s).

**Status:** Met
**Evidence:** `cargo test` runs 58+ Rust unit/integration tests.
`npx playwright test` runs end-to-end tests. CI workflow (`ci.yml`) runs both
suites on every push/PR. [CONTRIBUTING.md](../../.github/CONTRIBUTING.md)
documents the pre-PR test checklist.

---

#### test_invocation — Standard test invocation (SHOULD)

> A test suite SHOULD be invocable in a standard way for that language.

**Status:** Met
**Evidence:** `cargo test` (standard Rust), `npx playwright test` (standard
Node.js/Playwright).

---

#### test_most — High test coverage (SUGGESTED)

> It is SUGGESTED that the test suite cover most (or ideally all) the code
> branches, input fields, and functionality.

**Status:** Unmet
**Rationale:** No formal code coverage measurement is configured. Rust tests
cover core logic (storage, CSV import, collection CRUD) and fuzz testing covers
the CSV parser, but branch coverage percentage is not tracked.
**To clear:** Add `cargo-tarpaulin` or `llvm-cov` to CI and report coverage.

---

#### test_continuous_integration — CI (SUGGESTED)

> It is SUGGESTED that the project implement continuous integration.

**Status:** Met
**Evidence:** `.github/workflows/ci.yml` runs frontend lint/typecheck/build and
Rust clippy/fmt/test on every push and pull request to `main`.

---

### New functionality testing

#### test_policy — Test policy for new functionality

> The project MUST have a general policy (formal or not) that as major new
> functionality is added to the software produced by the project, tests of that
> functionality should be added to an automated test suite.

**Status:** Met
**Evidence:** [CONTRIBUTING.md](../../.github/CONTRIBUTING.md) pre-PR checklist
requires `cargo test` to pass. Maintainer policy is to add tests for new
functionality (evidenced by 58+ tests across modules).

---

#### tests_are_added — Tests added in practice

> The project MUST have evidence that the test policy has been adhered to in the
> most recent major changes to the software produced by the project.

**Status:** Met
**Evidence:** Recent commits include tests for collection CSV import, catalog
bulk operations, storage migrations, and scanning index. The fuzz target
`csv_import_preview` was added alongside the CSV import feature.

---

#### tests_documented_added — Test policy in change proposals (SUGGESTED)

> It is SUGGESTED that this policy on adding tests be documented in the
> instructions for change proposals.

**Status:** Unmet
**Rationale:** [CONTRIBUTING.md](../../.github/CONTRIBUTING.md) requires running
`cargo test` in the pre-PR checklist but does not explicitly state "new features
must include new tests" as a documented requirement.
**To clear:** Add a sentence to CONTRIBUTING.md's pre-PR checklist or coding
standards stating that new functionality should be accompanied by tests.

---

### Warning flags

#### warnings — Compiler warnings or linter enabled

> The project MUST enable one or more compiler warning flags, a "safe" language
> mode, or use a separate "linter" tool to look for code quality errors or
> common simple mistakes.

**Status:** Met
**Evidence:** `cargo clippy --all-targets --all-features -- -D warnings` (treats
all warnings as errors). ESLint for TypeScript with `--max-warnings 0`. Both
enforced in CI.

---

#### warnings_fixed — Warnings addressed

> The project MUST address warnings.

**Status:** Met
**Evidence:** CI fails on any warning. Zero clippy warnings, zero ESLint
warnings in the current codebase.

---

#### warnings_strict — Maximally strict warnings (SUGGESTED)

> It is SUGGESTED that projects be maximally strict with warnings in the
> software produced by the project, where practical.

**Status:** Met
**Evidence:** Clippy with `-D warnings` (all warnings fatal). ESLint with
`--max-warnings 0`. Prettier format check enforced.

---

## Security

### Secure development knowledge

#### know_secure_design — Secure design knowledge

> The project MUST have at least one primary developer who knows how to design
> secure software.

**Status:** Met
**Evidence:** [THREATMODEL.md](../THREATMODEL.md) demonstrates understanding of
Saltzer-Schroeder principles — least privilege (local-only data, minimal
permissions), economy of mechanism (SQLite WAL, no server), fail-safe defaults
(no network by default), complete mediation (Tauri command allowlist).
[SECURITY.md](../../.github/SECURITY.md) documents accepted risks and
vulnerability reporting.

---

#### know_common_errors — Knowledge of common vulnerability types

> At least one of the project's primary developers MUST know of common kinds of
> errors that lead to vulnerabilities in this kind of software, as well as at
> least one method to counter or mitigate each of them.

**Status:** Met
**Evidence:** SQL injection → prevented by rusqlite parameterized queries.
XSS → prevented by React's default escaping + CSP. CSV injection → sanitized
in the CSV export path. Path traversal → Tauri's fs scope limits file access.
Image-based DoS → size limits on scan input. Dependency vulnerabilities →
OSV-Scanner + cargo-deny in CI.

---

### Use basic good cryptographic practices

#### crypto_published — Publicly reviewed crypto algorithms

> The software produced by the project MUST use, by default, only cryptographic
> protocols and algorithms that are publicly published and reviewed by experts
> (if cryptographic protocols and algorithms are used).

**Status:** N/A
**Rationale:** Binderbase does not implement or directly configure cryptographic
protocols. TLS for external API calls is handled by the OS/rustls stack. SQLite
stores data locally without encryption.

---

#### crypto_call — Delegate crypto to purpose-built libraries (SHOULD)

> If the software produced by the project is an application or library, and its
> primary purpose is not to implement cryptography, then it SHOULD only call on
> software specifically designed to implement cryptographic functions; it SHOULD
> NOT re-implement its own.

**Status:** Met
**Evidence:** All cryptographic operations are delegated: TLS via rustls/OS
stack, cosign for release signing. No custom cryptographic code exists in the
codebase.

---

#### crypto_floss — Crypto implementable with FLOSS

> All functionality in the software produced by the project that depends on
> cryptography MUST be implementable using FLOSS.

**Status:** N/A
**Rationale:** No direct cryptographic functionality. TLS is provided by FLOSS
libraries (rustls).

---

#### crypto_keylength — NIST minimum key lengths

> The security mechanisms within the software produced by the project MUST use
> default keylengths that at least meet the NIST minimum requirements through
> the year 2030.

**Status:** N/A
**Rationale:** Binderbase does not manage cryptographic keys directly.

---

#### crypto_working — No broken crypto algorithms

> The default security mechanisms within the software produced by the project
> MUST NOT depend on broken cryptographic algorithms.

**Status:** N/A
**Rationale:** No direct use of cryptographic algorithms. TLS cipher suite
selection is handled by the OS/rustls defaults.

---

#### crypto_weaknesses — No seriously weak crypto (SHOULD)

> The default security mechanisms within the software produced by the project
> SHOULD NOT depend on cryptographic algorithms or modes with known serious
> weaknesses.

**Status:** N/A
**Rationale:** No direct cryptographic algorithm usage.

---

#### crypto_pfs — Perfect forward secrecy (SHOULD)

> The security mechanisms within the software produced by the project SHOULD
> implement perfect forward secrecy for key agreement protocols.

**Status:** N/A
**Rationale:** Binderbase does not implement key agreement protocols. TLS
forward secrecy is handled by the underlying rustls/OS stack.

---

#### crypto_password_storage — Iterated hashed passwords

> If the software produced by the project causes the storing of passwords for
> authentication of external users, the passwords MUST be stored as iterated
> hashes with a per-user salt.

**Status:** N/A
**Rationale:** Binderbase does not store passwords for user authentication. API
keys for external services are stored in the OS keychain via the `keyring`
crate.

---

#### crypto_random — Cryptographically secure RNG

> The security mechanisms within the software produced by the project MUST
> generate all cryptographic keys and nonces using a cryptographically secure
> random number generator.

**Status:** N/A
**Rationale:** Binderbase does not generate cryptographic keys or nonces.

---

### Secured delivery against MITM attacks

#### delivery_mitm — MITM-resistant delivery

> The project MUST use a delivery mechanism that counters MITM attacks. Using
> https or ssh+scp is acceptable.

**Status:** Met
**Evidence:** Source code delivered via GitHub (HTTPS/SSH). Release artifacts
served from GitHub Releases over HTTPS. Release workflow uses cosign for
artifact signing.

---

#### delivery_unsigned — No unsigned hash retrieval over HTTP

> A cryptographic hash (e.g., a sha1sum) MUST NOT be retrieved over http and
> used without checking for a cryptographic signature.

**Status:** Met
**Evidence:** All release artifacts are signed with cosign (keyless, Sigstore
transparency log). No plain HTTP hash distribution.

---

### Publicly known vulnerabilities fixed

#### vulnerabilities_fixed_60_days — Patch public vulns within 60 days

> There MUST be no unpatched vulnerabilities of medium or higher severity that
> have been publicly known for more than 60 days.

**Status:** Met
**Evidence:** No CVEs have been assigned against Binderbase. The 19 transitive
dependency advisories are all informational (no CVSS score, no CVE) and are
documented in [advisory-triage.md](advisory-triage.md) with upstream blocker
analysis.

---

#### vulnerabilities_critical_fixed — Fix critical vulns rapidly (SHOULD)

> Projects SHOULD fix all critical vulnerabilities rapidly after they are
> reported.

**Status:** Met
**Evidence:** No critical vulnerabilities have been reported. Monitoring is in
place via OSV-Scanner (weekly + on every push/PR) and cargo-deny.

---

### Other security issues

#### no_leaked_credentials — No leaked credentials

> The public repositories MUST NOT leak a valid private credential (e.g., a
> working password or private key) that is intended to limit public access.

**Status:** Met
**Evidence:** No credentials in the repository. API keys are stored in the OS
keychain at runtime via the `keyring` crate. `.gitignore` excludes environment
files. CONTRIBUTING.md's non-negotiables section explicitly prohibits secrets in
the repo.

---

## Analysis

### Static code analysis

#### static_analysis — Static analysis tool applied

> At least one static code analysis tool (beyond compiler warnings and "safe"
> language modes) MUST be applied to any proposed major production release of
> the software before its release.

**Status:** Met
**Evidence:** CodeQL (GitHub's semantic analysis) runs on every push/PR via
`.github/workflows/codeql.yml`, covering `javascript-typescript` and `actions`
languages. Cargo clippy (beyond basic compiler warnings) also runs on every
push/PR.

---

#### static_analysis_common_vulnerabilities — Vuln-focused static analysis (SUGGESTED)

> It is SUGGESTED that at least one of the static analysis tools used include
> rules or approaches to look for common vulnerabilities in the analyzed
> language or environment.

**Status:** Met
**Evidence:** CodeQL uses `security-and-quality` query suite, which includes
rules for XSS, injection, path traversal, and other common vulnerability
classes.

---

#### static_analysis_fixed — Fix medium+ static analysis findings

> All medium and higher severity exploitable vulnerabilities discovered with
> static code analysis MUST be fixed in a timely way after they are confirmed.

**Status:** N/A
**Rationale:** No medium or higher severity findings have been reported by
CodeQL or clippy.

---

#### static_analysis_often — Frequent static analysis (SUGGESTED)

> It is SUGGESTED that static source code analysis occur on every commit or at
> least daily.

**Status:** Met
**Evidence:** CodeQL and clippy both run on every push and pull request via CI.

---

### Dynamic code analysis

#### dynamic_analysis — Dynamic analysis tool (SUGGESTED)

> It is SUGGESTED that at least one dynamic analysis tool be applied to any
> proposed major production release of the software before its release.

**Status:** Met
**Evidence:** cargo-fuzz runs weekly via `.github/workflows/fuzz.yml` and can be
triggered manually. Fuzz target: `csv_import_preview` tests the CSV import
parser with randomized inputs.

---

#### dynamic_analysis_unsafe — Memory safety dynamic analysis (SUGGESTED)

> It is SUGGESTED that if the software includes software written using a
> memory-unsafe language (e.g., C or C++), then at least one dynamic tool be
> routinely used in combination with a mechanism to detect memory safety
> problems. If the project does not produce software written in a memory-unsafe
> language, choose "not applicable" (N/A).

**Status:** N/A
**Rationale:** Binderbase is written in Rust (memory-safe) and
TypeScript/JavaScript (memory-managed). No C or C++ source code is produced by
the project. Transitive C dependencies (SQLite, GTK) are upstream-maintained.

---

#### dynamic_analysis_enable_assertions — Assertions during dynamic analysis (SUGGESTED)

> It is SUGGESTED that the project use a configuration for at least some dynamic
> analysis (such as testing or fuzzing) which enables many assertions.

**Status:** Met
**Evidence:** Rust's `debug_assertions` are enabled during `cargo test` and
`cargo fuzz` runs. Rust panics serve as assertions — any out-of-bounds access,
unwrap failure, or explicit `assert!` call will halt the fuzzer and report the
failing input.

---

#### dynamic_analysis_fixed — Fix medium+ dynamic analysis findings

> All medium and higher severity exploitable vulnerabilities discovered with
> dynamic code analysis MUST be fixed in a timely way after they are confirmed.

**Status:** N/A
**Rationale:** No medium or higher severity findings have been discovered via
fuzzing.
