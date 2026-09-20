# PMAT-354 — receipt: bashrs v7.4.2

**Ticket:** PMAT-354 (bashrs#354) — release: v7.4.2. **Kind:** code.
**Branch:** `PMAT-354-release-v7.4.2` from `origin/main` at `94497c22b2`, in a detached worktree; `~/src/bashrs` (12 behind, 1 ahead, dirty) was not used for the cut.
**Authorized:** by the operator, 2026-09-20, in session. Relayed to this session once beforehand by another agent and REFUSED on that relay — a peer cannot carry a publish authorization, and the repo's own gate is independent of who asks.

## Where this cut's claims are backed

**This cut contains Rust, and an earlier version of this line said it did not.** The rule it stated —
*"a release cut describes what already merged, so its own diff contains no Rust"* — is the right rule
and this cut is a genuine exception, which is exactly the shape that has to be written down rather
than left for a reader to notice. A three-engine quorum refused the PR over it (2/3 lanes, both
citing this line against `rash/src/linter/quoting.rs` in the same diff), and the lanes were right:
the claim had become false while the sentence stayed.

The exception is PMAT-355 (#355), and it is in the cut because **the release gate is red without
it**. `make release-gate` on this cut failed on bashrs's own guard —
`test_GH226_quoting_allowlist_names_only_rules_that_exist`, *"SC2242 is allowlisted but names no rule
module"* — so v7.4.2 could not have been cut at all. A fix the gate demands is part of the release,
not a follow-up to it; what was wrong was the paperwork claiming otherwise.

Everything else here is what already merged. PMAT-338's receipt established the rule that the cut
must name WHERE each claim lives; these are the commits on `main` between `v7.4.1` and `94497c22b2`:

| Claim in the CHANGELOG | Commit | What it touched |
|---|---|---|
| SEC005 matched `sk-` as a bare substring, so `ci-disk-watch` read as an OpenAI key; a prefix counts only at a token start; property tests both ways; `F-SEC005-BOUNDARY` contract row | `94497c22b2` (#351, PMAT-350) | the secret-pattern matcher, its property tests, the contract |
| The x86_64-only `renacer` dev-dependency broke every aarch64 build of a crate nothing links | `109e3fd33c` (#353, PMAT-352) | the dev-dependency, dropped |
| SC2242's `in_case`/`in_loop`/`in_function` are depth counters, so a one-line case ends where it ends | `5c9b7d188d` (#332, PMAT-343, closes #331) | the linter rule |
| CI action bumps | `0ace2207e6` (#346), `a53f58c6ff` (#347), `253c0af334` (#345) | workflows |

## Why this release is not cosmetic

Until 7.4.2 is on crates.io, forjar's pin stays at 6.68.0 and `ci-disk-watch-timer-enable` fails its I8 apply-gate on every intel and gx10 converge. Measured 2026-09-20 by the session running those applies: `16 converged, 3 unchanged, 1 FAILED` on intel, `4 converged, 6 unchanged, 1 FAILED` on gx10 — the same resource each time. That is bashrs refusing a script bashrs ships, which is the fault class SEC005's fix removes.

## The plan was stale, and a release cut is when that has to be true

`release-lint` was RED on `main` before this branch: seven failures. Five were entries whose work merged on 2026-09-13 and whose status was never flipped — PMAT-338 (the v7.4.1 release itself, #344, tag cut), PMAT-339 and PMAT-341 (#342), PMAT-340 (#343), PMAT-343 (#332) — and two were PMAT-350/PMAT-352 carrying no release label. Each merge was verified against its PR (`gh pr view`, all `MERGED`) before the status was changed; nothing was marked complete on the strength of the roadmap alone.

R4 is the reason this is in scope rather than a follow-up: *a tag is cut only when its release has no open entry*. A plan that still lists v7.4.1's work as open cannot say whether v7.4.2 is cuttable.

After: R1–R5 clean. **`release-lint` still exits 1**, and an earlier version of this receipt claimed
`rerun_exit=0` — a claim a reader could refute by running the command, which is the one thing a
verification line must never be. It exits 1 on **four** R6 lines naming pre-release tags from 2025
(`v1.0.0-rc1`, `v1.0.0-rc2`, `v1.0.0-rc3`, `v6.28.0-dev-snapshot`) that exist in the repo's history
and cannot be renamed. A previous count of "three" here omitted `v1.0.0-rc1`. They are pre-existing
and neither R3 nor R5 reads them, which is why they do not block the cut — but "does not block the
cut" is not "exits 0", and the receipt now says which one it means.

Both errors were found by a three-engine quorum lane re-running the command against this line
(`[measured]`, not `[cited]`), and both had become false through this branch's own work: completing
PMAT-350/PMAT-352 moved the open counts the line quotes.

## Verification

verification:
  cmd=cargo check --workspace  claimed_exit=0  rerun_exit=0  log_path=docs/audits/impl-PMAT-354-receipt.md  sha256=0   # regenerates Cargo.lock at 7.4.2
  cmd=release-lint.sh docs/roadmaps/roadmap.yaml --repo . --plan docs/roadmaps/releases.md --notes docs/release-notes.md  claimed_exit=1  rerun_exit=1  log_path=docs/audits/impl-PMAT-354-receipt.md  sha256=0   # R1-R5 clean; exits 1 on 4 historical R6 tags; open=4 releases=v7.4.2(2) v7.5.0(1) unscheduled(1)
  cmd=make release-gate  claimed_exit=0  rerun_exit=0  log_path=docs/audits/impl-PMAT-354-receipt.md  sha256=0   # fmt, clippy, workspace tests, pv lint contracts, coverage-gate, dogfood-selflint, corpus-score, book
  cmd=make -f machines/clean-room/Makefile clean-room-bashrs  claimed_exit=0  rerun_exit=0  log_path=docs/audits/impl-PMAT-354-receipt.md  sha256=0   # in paiml/infra: the published crate builds from crates.io deps alone

## Gate order

1. `make release-gate` on this cut.
2. `clean-room-bashrs` in paiml/infra — the published crate is `rash/`, not the workspace root (`bashrs-specs`), which is why the clean-room target is scoped `-p bashrs`.
3. Merge quorum, 3/3.
4. Tag `v7.4.2` on `main`, verified to be an ancestor-carrier of `94497c22b2` and `109e3fd33c`.
5. `make publish-from-tag TAG=v7.4.2`.
6. The forjar pin PR to `bashrs = "7.4.2"`, **done-when intel's `ci-disk-watch-timer-enable` converges** — the measurement that proves the whole chain rather than the version string.
