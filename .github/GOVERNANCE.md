# Governance

This document describes how Binderbase is maintained and how decisions are made.
It will evolve as the project grows.

## Current state

Binderbase is a pre-1.0, single-maintainer project. The sole maintainer is
[@aecyx](https://github.com/aecyx). Day-to-day decisions — feature scope, API
design, dependency choices, release timing — are made informally by the
maintainer.

## Decision-making

For routine work (bug fixes, dependency updates, small features), the
maintainer decides and merges. No formal process is needed at this stage.

For larger or contentious changes, the process is:

1. Open a GitHub issue describing the proposal.
2. Allow at least 7 days for discussion.
3. If no substantive objections are raised, the proposal is accepted (lazy
   consensus).
4. If consensus is not reached, the maintainer makes the final call and
   documents the reasoning in the issue or in
   [`docs/DECISIONS.md`](docs/DECISIONS.md).

The maintainer has final say on all decisions pre-1.0. This is a practical
choice, not a power grab — the project needs velocity more than process right
now.

## Becoming a maintainer

There is currently one maintainer. Additional maintainers will be invited based
on:

- Sustained, high-quality contributions over time (code, reviews, docs, triage).
- Demonstrated understanding of the project's goals and architecture.
- Trust built through collaboration.

The current maintainer extends the invitation. There is no application process.
Once a second maintainer joins, this document will be updated to describe shared
responsibilities, merge rights, and any voting or approval rules.

## When this model changes

This governance model is intentionally minimal. It should be revisited when any
of the following conditions are met:

- **3 or more active maintainers.** Informal decision-making breaks down past
  two people. Adopt a lightweight RFC or proposal process.
- **1.0 release.** Stability commitments require clearer rules for breaking
  changes and release cadence.
- **Extended maintainer unavailability.** If the sole maintainer is unreachable
  for 90+ days, a trusted contributor may fork or adopt the project. This
  section will be formalized if/when a second maintainer exists.

## Related documents

- [Contributing guide](CONTRIBUTING.md) — development setup and PR expectations.
- [Security policy](SECURITY.md) — vulnerability reporting.
- [Code of Conduct](CODE_OF_CONDUCT.md) — community standards and enforcement.
