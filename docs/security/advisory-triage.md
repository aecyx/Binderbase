<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->

# RUSTSEC Advisory Triage

Last updated: 2026-04-19

All 19 advisories are transitive dependencies pulled in by Tauri 2.x. None
are type `vulnerability` with a CVE or CVSS score. All are either
`unmaintained` (INFO) or `unsound` (INFO) advisories.

## Triage table

| ID                | Type         | Crate              | Version | CVE | CVSS | Upstream blocker                                                    | Why we cannot clear it                                                                                     |
| ----------------- | ------------ | ------------------ | ------- | --- | ---- | ------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------- |
| RUSTSEC-2024-0411 | unmaintained | gdkwayland-sys     | 0.18.2  | —   | —    | [tauri#14684](https://github.com/tauri-apps/tauri/pull/14684)       | Tauri 2.x requires gtk-rs 0.18; GTK4 migration is Tauri 3.0                                                |
| RUSTSEC-2024-0412 | unmaintained | gdk                | 0.18.2  | —   | —    | [tauri#14684](https://github.com/tauri-apps/tauri/pull/14684)       | Tauri 2.x requires gtk-rs 0.18; GTK4 migration is Tauri 3.0                                                |
| RUSTSEC-2024-0413 | unmaintained | atk                | 0.18.2  | —   | —    | [tauri#14684](https://github.com/tauri-apps/tauri/pull/14684)       | Tauri 2.x requires gtk-rs 0.18; GTK4 migration is Tauri 3.0                                                |
| RUSTSEC-2024-0414 | unmaintained | gdkx11-sys         | 0.18.2  | —   | —    | [tauri#14684](https://github.com/tauri-apps/tauri/pull/14684)       | Tauri 2.x requires gtk-rs 0.18; GTK4 migration is Tauri 3.0                                                |
| RUSTSEC-2024-0415 | unmaintained | gtk                | 0.18.2  | —   | —    | [tauri#14684](https://github.com/tauri-apps/tauri/pull/14684)       | Tauri 2.x requires gtk-rs 0.18; GTK4 migration is Tauri 3.0                                                |
| RUSTSEC-2024-0416 | unmaintained | atk-sys            | 0.18.2  | —   | —    | [tauri#14684](https://github.com/tauri-apps/tauri/pull/14684)       | Tauri 2.x requires gtk-rs 0.18; GTK4 migration is Tauri 3.0                                                |
| RUSTSEC-2024-0417 | unmaintained | gdkx11             | 0.18.2  | —   | —    | [tauri#14684](https://github.com/tauri-apps/tauri/pull/14684)       | Tauri 2.x requires gtk-rs 0.18; GTK4 migration is Tauri 3.0                                                |
| RUSTSEC-2024-0418 | unmaintained | gdk-sys            | 0.18.2  | —   | —    | [tauri#14684](https://github.com/tauri-apps/tauri/pull/14684)       | Tauri 2.x requires gtk-rs 0.18; GTK4 migration is Tauri 3.0                                                |
| RUSTSEC-2024-0419 | unmaintained | gtk3-macros        | 0.18.2  | —   | —    | [tauri#14684](https://github.com/tauri-apps/tauri/pull/14684)       | Tauri 2.x requires gtk-rs 0.18; GTK4 migration is Tauri 3.0                                                |
| RUSTSEC-2024-0420 | unmaintained | gtk-sys            | 0.18.2  | —   | —    | [tauri#14684](https://github.com/tauri-apps/tauri/pull/14684)       | Tauri 2.x requires gtk-rs 0.18; GTK4 migration is Tauri 3.0                                                |
| RUSTSEC-2024-0429 | unsound      | glib               | 0.18.5  | —   | —    | [gtk-rs-core#1343](https://github.com/gtk-rs/gtk-rs-core/pull/1343) | Fixed in glib >=0.20.0; Tauri 2.x pins glib 0.18 via gtk-rs 0.18                                           |
| RUSTSEC-2024-0370 | unmaintained | proc-macro-error   | 1.0.4   | —   | —    | [tauri#14684](https://github.com/tauri-apps/tauri/pull/14684)       | Used by glib-macros/gtk3-macros; drops with GTK4 migration                                                 |
| RUSTSEC-2025-0057 | unmaintained | fxhash             | 0.2.1   | —   | —    | tauri-utils kuchikiki dep                                           | Pulled via selectors→kuchikiki→tauri-utils; no newer version exists                                        |
| RUSTSEC-2026-0097 | unsound      | rand               | 0.7.3   | —   | —    | tauri-utils kuchikiki dep                                           | Build-dep via phf_codegen→selectors→kuchikiki→tauri-utils; patched in >=0.8.6 but phf_codegen 0.8 pins 0.7 |
| RUSTSEC-2025-0075 | unmaintained | unic-char-range    | 0.9.0   | —   | —    | tauri-utils urlpattern 0.3                                          | tauri-utils pins urlpattern 0.3.0 which depends on unic-\*; 0.6 drops it but is semver-incompatible        |
| RUSTSEC-2025-0080 | unmaintained | unic-common        | 0.9.0   | —   | —    | tauri-utils urlpattern 0.3                                          | tauri-utils pins urlpattern 0.3.0 which depends on unic-\*; 0.6 drops it but is semver-incompatible        |
| RUSTSEC-2025-0081 | unmaintained | unic-char-property | 0.9.0   | —   | —    | tauri-utils urlpattern 0.3                                          | tauri-utils pins urlpattern 0.3.0 which depends on unic-\*; 0.6 drops it but is semver-incompatible        |
| RUSTSEC-2025-0098 | unmaintained | unic-ucd-version   | 0.9.0   | —   | —    | tauri-utils urlpattern 0.3                                          | tauri-utils pins urlpattern 0.3.0 which depends on unic-\*; 0.6 drops it but is semver-incompatible        |
| RUSTSEC-2025-0100 | unmaintained | unic-ucd-ident     | 0.9.0   | —   | —    | tauri-utils urlpattern 0.3                                          | tauri-utils pins urlpattern 0.3.0 which depends on unic-\*; 0.6 drops it but is semver-incompatible        |

## Summary

- **0** type=vulnerability advisories
- **15** type=unmaintained advisories (INFO)
- **2** type=unsound advisories (INFO): glib VariantStrIter (GHSA-wrw7-89jp-8q8g) and rand thread_rng (GHSA-cq8v-f236-94qc)
- **0** CVEs, **0** CVSS scores

All are suppressed in `osv-scanner.toml` with a 1-year expiry (2027-04-19).
