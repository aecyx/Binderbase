<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->

## What's Changed

<!-- Brief summary of the release — fill in before tagging. -->

### Security

- Cosign keyless signing of release artifacts (Sigstore OIDC)
- CodeQL analysis on push/PR (JavaScript/TypeScript + Actions)
- `cargo-deny` supply chain checks (advisories, bans, licenses, sources)
- CSV export formula-injection protection (OWASP)
- Scan image DoS guard (10 MB raw / 50 MP decoded limits)
- `InputTooLarge` error variant with user-friendly frontend propagation
- Threat model documented (`docs/THREATMODEL.md`)
- SECURITY.md with supported versions, reporting instructions, accepted risks
- Weekly cargo-fuzz CI for CSV import parser

### Checksums

See the `SHA256SUMS` asset attached to this release.

### Verification

Every artifact is signed with [Sigstore](https://sigstore.dev/) keyless signing.
To verify (requires [cosign](https://docs.sigstore.dev/cosign/system_config/installation/) v3+):

```bash
cosign verify-blob \
  --bundle Binderbase_1.0.0_amd64.msi.bundle \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com \
  --certificate-identity-regexp "^https://github\\.com/aecyx/Binderbase/\\.github/workflows/release\\.yml@refs/tags/.*$" \
  Binderbase_1.0.0_amd64.msi
```

---

**Full Changelog**: https://github.com/aecyx/Binderbase/compare/v1.0.0-rc.1...v1.0.0
