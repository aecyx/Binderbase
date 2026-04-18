# Operations

Runbook-style notes on how the Binderbase repo, CI, and release plumbing
are configured. This file describes the **intended** configuration — some
items must be applied through the GitHub web UI because they are not
expressible in repository files alone.

If you change any of the items below in the UI, please also update this
document so the repo's state is self-describing.

---

## Repository settings (apply via GitHub UI)

GitHub → **Settings → General**:

| Setting                            | Value                                                                                                                   |
| ---------------------------------- | ----------------------------------------------------------------------------------------------------------------------- |
| Description                        | `Local-first TCG scanner and collection manager (MTG + Pokémon).`                                                       |
| Website                            | _(leave blank until we publish a landing page)_                                                                         |
| Topics                             | `tauri`, `rust`, `react`, `typescript`, `tcg`, `mtg`, `pokemon-tcg`, `local-first`, `desktop-app`, `collection-manager` |
| Default branch                     | `main`                                                                                                                  |
| Allow merge commits                | **Off**                                                                                                                 |
| Allow squash merging               | **On** — default PR merge style                                                                                         |
| Allow rebase merging               | **On** — for linear-history fast-forwards                                                                               |
| Automatically delete head branches | **On**                                                                                                                  |
| Features → Wikis                   | **Off** (docs live in `docs/`)                                                                                          |
| Features → Issues                  | **On**                                                                                                                  |
| Features → Discussions             | **On** once we have users                                                                                               |
| Features → Projects                | **On**                                                                                                                  |

## Branch protection (apply via GitHub UI)

GitHub → **Settings → Branches → Branch protection rules → Add rule**.

Protect `main` with:

- **Require a pull request before merging**
  - Require approvals: **1** (owner review counts once we grow past one
    contributor; until then, set to **0** and rely on CI + self-review).
  - Dismiss stale pull request approvals when new commits are pushed: **On**.
  - Require review from Code Owners: **On** (uses `.github/CODEOWNERS`).
- **Require status checks to pass before merging**
  - Require branches to be up to date before merging: **On**.
  - Required checks (add once they've run at least once on a PR):
    - `CI / frontend`
    - `CI / rust (ubuntu-latest)`
    - `CI / rust (windows-latest)`
    - `CI / rust (macos-latest)`
    - `CI / audit`
- **Require conversation resolution before merging**: **On**.
- **Require linear history**: **On**.
- **Require signed commits**: **On** once you set up a signing key. Not
  required for the solo-dev phase, but turning it on later is cheap.
- **Do not allow bypassing the above settings**: **On** — including for
  admins. Forces even the owner to push changes through a PR.
- **Restrict who can push to matching branches**: leave unchecked (the
  above rules already gate writes).
- **Allow force pushes**: **Off**.
- **Allow deletions**: **Off**.

## Security settings (apply via GitHub UI)

GitHub → **Settings → Code security**:

- **Dependency graph**: **On** (default for private repos with
  Dependabot configured).
- **Dependabot alerts**: **On**.
- **Dependabot security updates**: **On**.
- **Dependabot version updates**: already configured in
  `.github/dependabot.yml`.
- **Secret scanning**: **On** (push protection + alerts). Available on
  private repos only with GitHub Advanced Security.
- **Private vulnerability reporting**: **On**. Referenced from
  `SECURITY.md` as the primary disclosure channel.

## Permissions for the default `GITHUB_TOKEN`

GitHub → **Settings → Actions → General → Workflow permissions**:

- **Read repository contents and packages permissions** (default, minimal).
- **Allow GitHub Actions to create and approve pull requests**: **Off**
  (Dependabot manages its own PRs; we don't want arbitrary workflow code
  opening PRs on our behalf).

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
- The sandbox proxy allowlists `github.com` (for git push) but blocks
  `api.github.com`. CI status, repo settings, and releases have to be
  checked through the web UI or the `gh` CLI run from the user's
  Windows terminal.
- Git credentials live in `~/.netrc` inside the sandbox (chmod 600) so
  the assistant can push. The token is a fine-grained PAT scoped only
  to `aecyx/Binderbase`.
