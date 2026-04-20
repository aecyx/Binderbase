<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->

# Cosign Verification — Binderbase Release Artifacts

This document records the end-to-end cosign verification procedure for
Binderbase release artifacts signed via Sigstore keyless (GitHub OIDC).

## Environment

- **Binderbase version:** 1.0.0
- **cosign version:** v2.4+ (any v2/v3 compatible release)

## Pre-requisites

Install [cosign](https://docs.sigstore.dev/cosign/system_config/installation/)
v2+ and ensure `sha256sum` (Linux) or `shasum` (macOS) is available.

## Step 1 — Download release assets

Download every asset from the GitHub release page:

```bash
# Example for v1.0.0 (replace tag as needed):
gh release download v1.0.0 --dir ./verify-v1.0.0
cd ./verify-v1.0.0
```

Expected files:

- Platform installers: `.msi`, `.dmg`, `.AppImage`, `.deb`, `.rpm`
- `SHA256SUMS.txt`
- One `.bundle` file per installer (cosign signature bundles)

## Step 2 — Verify SHA-256 checksums

```bash
sha256sum -c SHA256SUMS.txt
```

All lines must report `OK`. On macOS use `shasum -a 256 -c SHA256SUMS.txt`.

If any line fails, **stop** — the artifact may have been corrupted or tampered
with. Do not proceed to cosign verification.

## Step 3 — Verify cosign signatures

Run `cosign verify-blob` for each installer artifact:

```bash
# Windows .msi
cosign verify-blob \
  --bundle Binderbase_1.0.0_x64.msi.bundle \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com \
  --certificate-identity-regexp "^https://github\\.com/aecyx/Binderbase/\\.github/workflows/release\\.yml@refs/tags/.*$" \
  Binderbase_1.0.0_x64.msi

# macOS .dmg
cosign verify-blob \
  --bundle Binderbase_1.0.0_aarch64.dmg.bundle \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com \
  --certificate-identity-regexp "^https://github\\.com/aecyx/Binderbase/\\.github/workflows/release\\.yml@refs/tags/.*$" \
  Binderbase_1.0.0_aarch64.dmg

# Linux .AppImage
cosign verify-blob \
  --bundle Binderbase_1.0.0_amd64.AppImage.bundle \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com \
  --certificate-identity-regexp "^https://github\\.com/aecyx/Binderbase/\\.github/workflows/release\\.yml@refs/tags/.*$" \
  Binderbase_1.0.0_amd64.AppImage

# Linux .deb (if present)
cosign verify-blob \
  --bundle Binderbase_1.0.0_amd64.deb.bundle \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com \
  --certificate-identity-regexp "^https://github\\.com/aecyx/Binderbase/\\.github/workflows/release\\.yml@refs/tags/.*$" \
  Binderbase_1.0.0_amd64.deb
```

A successful verification prints `Verified OK`.

## Step 4 — Diagnosing failures

If any `verify-blob` invocation fails, inspect the bundle:

```bash
cat <file>.bundle | jq .
```

Extract the certificate subject alternative name (SAN):

```bash
cosign verify-blob \
  --bundle <file>.bundle \
  --insecure-ignore-tlog \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com \
  --certificate-identity-regexp ".*" \
  <file> 2>&1 | grep -i subject
```

The `--certificate-identity-regexp` in `.github/release-template.md` and
`README.md` must match the actual SAN. If it doesn't, update both files.

## Verification transcript

> Paste the full `cosign verify-blob` output here after running the
> verification against the v1.0.0 release artifacts.

## Notes for future releases

- The `--certificate-identity-regexp` pattern used across all documentation is:

  ```
  ^https://github\.com/aecyx/Binderbase/\.github/workflows/release\.yml@refs/tags/.*$
  ```

  This matches the OIDC subject assigned by GitHub Actions to the release
  workflow when triggered by a tag push. If the workflow file is renamed or
  moved, this regex must be updated in:
  - `.github/release-template.md`
  - `README.md` (Verifying with cosign section)
  - `docs/releases/v1.0.0-rc.1.md`
  - This file

- Sigstore keyless signing uses ephemeral certificates from Fulcio. The
  certificate is valid only for ~10 minutes but the signature is permanently
  recorded in the Rekor transparency log. Verification succeeds indefinitely
  as long as the Rekor entry exists.
