# Releases

This is the plan `release-lint.sh` reads (PMAT-049, rules R1–R4). Every open roadmap entry names exactly one release. `unscheduled` is reserved for blocked entries. A tag is cut only when its release has no open entry.

## v7.1.0

Goal: the five v7.1.0 issues fixed and measured against their own reproducers, the publish race from v7.0.4 closed, the first mechanical dogfood gates, a corpus grown where the sovereign repos are and the corpus is not, and dependencies current.

| entry | what |
|---|---|
| PMAT-254 | triage: release labels on every open entry, this plan |
| PMAT-255 | the release: #293 #294 #295 #265 #232, PMAT-244, PMAT-250, dependencies, comply, corpus growth, PMAT-253 phases 3a and 5 |
| PMAT-244 | SC2105 reported for a `break` inside a one-line loop |
| PMAT-250 | `shell_words` exposes command substitutions, so SC2046 drops its private scanner |

Issues on the GitHub milestone `v7.1.0`: #293, #294, #295, #265, #232.

## v7.2.0

| entry | what |
|---|---|
| PMAT-253 | forjar parity, the phases not shipped in v7.1.0: 1a–1c, 2, 3b, 4a, 4b, 6, 7a, 7b (`docs/specifications/pr-dogfood-parity-forjar.md`) |

## Unscheduled (blocked)

| entry | blocked on |
|---|---|
| PMAT-241 | definition: no acceptance criteria since 2026-04-07 |
| PMAT-242 | definition: no acceptance criteria since 2026-04-07 |
| PMAT-243 | definition: no acceptance criteria since 2026-04-07 |
| PMAT-246 | upstream: pmat writes work-contract stubs that pv 0.65 validates as kernel contracts (missing `metadata`, `verification_summary.total_obligations`) |

## Backlog issues

#236, #234 and #233 sit on the GitHub milestone `Backlog`; they are issues, not roadmap entries, so no release names them.
