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

## v7.2.0 (released 2026-09-11)

| entry | what |
|---|---|
| PMAT-253 | forjar parity, the phases not shipped in v7.1.0: 1a–1c, 2, 3b, 4a, 4b, 6, 7a, 7b (`docs/specifications/pr-dogfood-parity-forjar.md`), with §4 settled by quorum |
| PMAT-243 | COV-MEASURE: measure line coverage, name the top ten gaps, record both |
| PMAT-246 | work contracts regenerated so pv validates all 103; the CB-1305 half is upstream (paiml-mcp-agent-toolkit#1306) |
| PMAT-256 | corpus runner sandbox: temporary cwd, HOME and PATH everywhere, bwrap where Linux has it (make corpus-score sandboxes the release run; the runner itself is v7.3.0) |
| PMAT-257 | the release itself: ten defects, pv gate 4 green, coverage 95.01 percent, the Pareto gates, the book |

## v7.3.0

Goal: the lowerings the v7.2.0 corpus removals exposed, the loop-containment rule that dogfooding keeps asking for, and the sandbox that stops a test writing into the repository. Coverage stays at or above 95 percent and every ticket carries a falsification test the contract verifier checks.

| entry | what |
|---|---|
| PMAT-258 | the release: #303, #316, #318, deps, comply, corpus growth, the book |
| PMAT-253 | forjar parity, the phases not shipped in v7.2.0: 1a–1c, 2, 3b, 4a, 4b, 6, 7a, 7b |
| PMAT-256 | corpus runner sandbox: temporary cwd, HOME and PATH everywhere, bwrap where Linux has it |
| PMAT-259 | corpus_converged reads its convergence log from the caller, so a test cannot depend on the checkout |

Issues on this release: #303 (SC2106 not implemented), #316 (53 corpus entries exercised std methods with no lowering), #318 (an untracked copy of the source tree written by a test).

## Unscheduled (blocked)

Nothing. The four entries that sat here blocked on definition were decided by quorum on 2026-09-11: PMAT-241 and PMAT-242 closed as completed in v6.66.0, PMAT-243 redefined as a measurement in v7.2.0, PMAT-246 closed by regenerating the work contracts. See `docs/audits/quorum-decisions-v7.2.0.md`.

## Backlog issues

#236, #234 and #233 sit on the GitHub milestone `Backlog`; they are issues, not roadmap entries, so no release names them.
