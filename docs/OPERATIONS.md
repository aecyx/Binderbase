# Operations

Runbook-style notes on how the Binderbase repo, CI, and release plumbing
are configured. This file describes the **intended** configuration — some
items must be applied through the GitHub web UI because they are not
expressible in repository files alone.

If you change any of the items below in the UI, please also update this
document so the repo's state is self-describing. When in doubt, prefer
the more restrictive of two options; we can loosen later.

> **Admin note.** These are recommendations for the maintainer (`@aecyx`
> at time of writing). Items marked _(GHAS only)_ require GitHub Advanced
> Security, which is included for public repos and needs a paid plan on
> private ones.

---

## About panel (repo home page, not Settings)

On `github.com/aecyx/Binderbase`, the right sidebar has an **About**
section. Click the small gear icon next to the word **About** to edit:

| Field       | Value                                                                                                                   |
| ----------- | ----------------------------------------------------------------------------------------------------------------------- |
| Description | `Local-first TCG scanner and collection manager (MTG + Pokémon).`                                                       |
| Website     | _(leave blank until we publish a landing page)_                                                                         |
| Topics      | `tauri`, `rust`, `react`, `typescript`, `tcg`, `mtg`, `pokemon-tcg`, `local-first`, `desktop-app`, `collection-manager` |

Also in the About panel, check:

- **Releases** — on (automatically becomes visible when we cut one).
- **Packages** — off (we don't publish to GitHub Packages).
- **Deployments** — off (no GitHub Environments yet).

---

## Settings → General

### Pull Requests

| Setting                                               | Value   | Notes                                                                                                                                    |
| ----------------------------------------------------- | ------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| Allow merge commits                                   | **Off** | Merge commits clutter history; squash + rebase are enough.                                                                               |
| Allow squash merging                                  | **On**  | Default style for PR merges. Keep the default commit message = PR title.                                                                 |
| Allow rebase merging                                  | **On**  | For linear-history fast-forwards on trivial PRs.                                                                                         |
| Always suggest updating pull request branches         | **On**  | Prompts PR authors to pull `main` in when it moves. Matches the "require branches up to date" branch rule.                               |
| Allow auto-merge                                      | **Off** | Revisit once required checks + branch protection are proven. Risk: CI flake auto-merges a bad PR.                                        |
| Automatically delete head branches                    | **On**  | Keeps the branch list clean after merge.                                                                                                 |
| Require contributors to sign off on web-based commits | **Off** | That's DCO sign-off — useful for large OSS projects absorbing anonymous contributions. AGPL + `CONTRIBUTING.md` already establish terms. |

### Archives

| Setting                             | Value   | Notes                              |
| ----------------------------------- | ------- | ---------------------------------- |
| Include Git LFS objects in archives | Default | Irrelevant — we don't use Git LFS. |

### Features

| Feature                  | Value   | Notes                                                                                                                                |
| ------------------------ | ------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| Wikis                    | **Off** | Documentation lives in `docs/` — versioned with the code, reviewed in PRs. Wikis fragment the docs surface.                          |
| Issues                   | **On**  | Public issue tracker (for private-repo collaborators).                                                                               |
| Sponsorships             | **Off** | Zero technical impact. Enable later if you want a "Sponsor" button — makes more sense after a 1.0 release.                           |
| Projects                 | **On**  | Useful even for solo work; free.                                                                                                     |
| Discussions              | **Off** | Turn on once there are users or external contributors. Moving issues → discussions later is one click; going the other way is messy. |
| Preserve this repository | **Off** | GitHub Archive Program. Intended for long-lived public OSS snapshots; pointless on a private repo.                                   |

### Interaction limits

Private repos ignore interaction limits (only collaborators can interact
anyway). Leave at default.

---

## Branch protection (`main`)

GitHub is migrating everyone to **Rulesets** (Settings → Rules →
Rulesets). You may see both the classic **Branches** page and the new
**Rules** page. Prefer Rulesets.

### If using Rulesets (preferred)

Create a ruleset named **"protect-main"** with:

- **Enforcement status:** **Active** (not Evaluate, not Disabled).
- **Bypass list:** empty. (Do not add yourself — bypassing defeats the point.)
- **Targets → Target branches → Add target:**
  - `Include by pattern` → `main`
  - _or_ `Include default branch`
  - **If Targets shows 0 branches, this is the missing step.** The rule
    doesn't apply to any branch until a target is added.
- **Rules → enable:**
  - Restrict creations: **Off** (we need to create branches).
  - Restrict updates: **On** (only via the PR flow).
  - Restrict deletions: **On** — you cannot delete `main`.
  - Require linear history: **On**.
  - Require a pull request before merging
    - Required approvals: **0** while solo, bump to **1** once we have a
      second maintainer. With 1 required and only one person able to
      approve, you can't merge your own PRs.
    - Dismiss stale pull request approvals when new commits are pushed: **On**.
    - Require review from Code Owners: **On** (uses `.github/CODEOWNERS`).
    - Require approval of the most recent reviewable push: **On**.
    - Allowed merge methods: Squash, Rebase (match the Settings → General choice above).
  - Require status checks to pass
    - Require branches to be up to date before merging: **On**.
    - Checks (add once each has run at least once on a PR):
      - `CI / frontend`
      - `CI / rust (ubuntu-latest)`
      - `CI / rust (windows-latest)`
      - `CI / rust (macos-latest)`
      - `CI / audit`
  - Block force pushes: **On**.
  - Require code scanning results: _(only appears with GHAS + CodeQL enabled; configure once you set up CodeQL below)_
  - Require signed commits: **Off for now**; turn on once you have a
    signing key set up in Git and `gh`. Low cost to add later.
  - Require deployments to succeed before merging: **Off**. We don't
    use GitHub Environments yet.

### If using classic Branch protection (fallback)

Settings → Branches → Add rule:

- Branch name pattern: `main` (just `main`, no slashes, no wildcards).
- Same rule set as above, one-to-one mapped.

### "Applied to 0 branches" diagnosis

If the UI shows the rule applies to 0 branches, the cause is one of:

1. **No target added** (most common with Rulesets). Fix: add `main`
   or "Include default branch" in Targets.
2. **Pattern typo.** `main` — no leading slash, no spaces, no wildcard
   unless intentional (`main*` matches `main`, `maintenance`, etc.).
3. **Ruleset status is Disabled or Evaluate.** Flip to **Active**.

---

## Code security (Settings → Code security, or Advanced Security on private repos)

| Setting                         | Value                                       | Notes                                                                                                                                                                                                                  |
| ------------------------------- | ------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Dependency graph                | **On**                                      | Default for repos with Dependabot configured.                                                                                                                                                                          |
| Automatic dependency submission | **On**                                      | Uploads the dep graph to GitHub so alerts work for ecosystems without a parseable lockfile. Redundant for us (npm/cargo have lockfiles) but defense-in-depth.                                                          |
| Dependabot alerts               | **On**                                      | Free, zero downside.                                                                                                                                                                                                   |
| Dependabot security updates     | **On**                                      | Auto-opens PRs for vulnerable deps. Review them — don't auto-merge.                                                                                                                                                    |
| Dependabot malware alerts       | **On**                                      | Alerts when a dep is flagged malicious (e.g., recent npm typo-squats). Free, zero downside.                                                                                                                            |
| Grouped security updates        | **On**                                      | Rolls multiple security PRs into one. Matches the grouping we do in `dependabot.yml` for eslint/prettier/types/tauri.                                                                                                  |
| Dependabot version updates      | **On**                                      | Configured via `.github/dependabot.yml` — nothing else to do in UI.                                                                                                                                                    |
| Secret scanning alerts          | **On**                                      | _(GHAS only on private repos.)_ Catches committed tokens. Worth the cost on its own.                                                                                                                                   |
| Push protection                 | **On**                                      | _(GHAS only on private repos.)_ Rejects pushes containing detected secrets at push time. Highest-value GHAS feature.                                                                                                   |
| Private vulnerability reporting | **On**                                      | This is what `SECURITY.md` points people to. Free on all repos.                                                                                                                                                        |
| CodeQL analysis                 | **On for JavaScript/TypeScript; skip Rust** | _(GHAS only on private repos.)_ TS/JS coverage is solid. Rust support is still in beta and adds noise; `cargo audit` in CI is doing more work today. Use the "Default" setup — GitHub commits a workflow file for you. |

If you don't have GHAS on this private repo, enable everything free
above and revisit CodeQL / secret scanning if the project flips public.

---

## Actions permissions (Settings → Actions → General)

### Actions permissions

Pick **"Allow `aecyx`, and select non-`aecyx`, actions and reusable workflows"** and enable:

- Allow actions created by GitHub: **On**.
- Allow actions by Marketplace verified creators: **On**.
- Allow specified actions and reusable workflows — add exactly:
  ```
  actions/checkout@*
  actions/setup-node@*
  actions/cache@*
  ```
  These are every third-party action our CI currently uses. If you add
  a new action to `.github/workflows/`, add it here in the same PR.

Alternative: **"Allow all actions and reusable workflows"** is
acceptable given every workflow change goes through human review. The
allowlist above is cheap insurance against a compromised-action
publish sneaking into a Dependabot bump.

### Fork pull request workflows from outside collaborators

**"Require approval for all outside collaborators"** (or at minimum
"Require approval for first-time contributors"). For a private repo
this rarely triggers, but it's a free safety net if the repo ever
flips public or you add outside collaborators.

### Fork pull request workflows in private repositories

Leave default (workflows do not run on forks of private repos anyway).

### Workflow permissions

- **Read repository contents and packages permissions** (minimum).
- **Allow GitHub Actions to create and approve pull requests:** **Off**
  (Dependabot manages its own PRs via its own bot identity; we don't
  want arbitrary workflow code opening PRs on our behalf).

---

## CI at a glance

`.github/workflows/ci.yml` runs on every push to `main` and every PR:

1. **frontend** — `npm ci`, `npm run format:check`, `npm run lint`,
   `npm run typecheck`, `npm run build`. Uses the Node version pinned in
   `.nvmrc`.
2. **rust** — matrix over `ubuntu-latest` / `windows-latest` /
   `macos-latest`:
   - `cargo fmt --all -- --check`
   - `cargo clippy --all-targets --all-features -- -D warnings`
   - `cargo test --all-features`
   - Cargo build + registry caches keyed on `Cargo.lock`.
3. **audit** — `npm audit --audit-level=high` and `cargo audit`. Both
   `continue-on-error: true` so a new advisory doesn't block merges, but
   the job stays visible as a signal to triage.

If a required check is flaky, prefer fixing the flake over removing the
check. If you must disable a check temporarily, open an issue first and
link it in the workflow comment.

## Dependabot

Configured in `.github/dependabot.yml`:

- **npm** — weekly, grouped updates for ESLint/Prettier, type packages,
  and Tauri packages. Non-grouped updates open as individual PRs.
- **cargo** (`/src-tauri`) — weekly.
- **github-actions** — monthly.

Auto-merge is **not** enabled. Review every bump by hand until we have a
green track record across patch/minor/major.

## Release & signing (TBD)

We haven't cut a release yet. When we do, the flow will be:

1. Tag `vX.Y.Z` on `main`.
2. A `release.yml` workflow builds signed installers per platform:
   - Windows: `.msi` signed with an EV or OV code-signing certificate.
   - macOS: `.dmg` signed + notarized with Apple Developer ID.
   - Linux: `.AppImage` + `.deb` (signing optional).
3. Installers uploaded as a GitHub Release; checksums posted in the
   release notes.

Signing certs and API keys will live in **Settings → Secrets and
variables → Actions**, never in the repo. Track setup in a future ADR
(ADR-0006: release signing).

## Secrets inventory

No secrets are committed to the repo. When any are added as GitHub
Actions secrets, record their **name and purpose** (not value) here:

| Secret name  | Purpose | Rotated |
| ------------ | ------- | ------- |
| _(none yet)_ |         |         |

## Local environment

- Node: see `.nvmrc` — use `nvm use` (or `fnm use`) to match CI.
- Rust: see `rust-toolchain.toml` — `rustup` will install stable + the
  pinned components (`rustfmt`, `clippy`) automatically.
- Editor: `.editorconfig` defines line endings, indent, and charset. Most
  editors pick this up with no extra config.
- Line endings: enforced via `.gitattributes` (`* text=auto eol=lf`).
  Windows users can keep `core.autocrlf=true`; the checkout-time
  normalization ensures git still sees LF.

## Known sandbox quirks (for AI-pair-programming sessions)

These don't affect humans but matter for the AI assistant working in
`C:\Code\Binderbase` from the Cowork sandbox:

- The WSL→Windows mount caches file bytes. Edits made through the file
  tools may not be immediately visible to bash-side tooling (`cargo`,
  `npm`, `git diff`). When this happens, `git checkout HEAD -- <path>`
  or rewriting the file via `cat > path <<EOF ... EOF` from bash forces
  the cache to refresh.
- The sandbox proxy allowlists `github.com:443` (for git push) but blocks
  `api.github.com`. CI status, repo settings, and releases have to be
  checked through the web UI or the `gh` CLI run from the user's
  Windows terminal.
- Git credentials live in `~/.netrc` inside the sandbox (chmod 600) so
  the assistant can push. The token is a fine-grained PAT scoped only
  to `aecyx/Binderbase`.
