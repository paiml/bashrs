# PMAT-354 — receipt: bashrs v7.4.2

**Ticket:** PMAT-354 (bashrs#354) — release: v7.4.2. **Kind:** code.
**Branch:** `PMAT-354-release-v7.4.2` from `origin/main` at `94497c22b2`, in a detached worktree; `~/src/bashrs` (12 behind, 1 ahead, dirty) was not used for the cut.
**Authorized:** by the operator, 2026-09-20, in session. Relayed to this session once beforehand by another agent and REFUSED on that relay — a peer cannot carry a publish authorization, and the repo's own gate is independent of who asks.
**Second cut:** 2026-10-04, the same branch name re-created from `origin/main` at `8f870b2cde` (the gate tree `1241570740` plus #358, a paperwork-only closeout) in a fresh worktree. The authorization above was given for the first cut, and this commit claims none of its own. It does not tag or publish: those are steps 4 and 5 of the gate order below, and each is its own act.

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

## The second cut (2026-10-04)

The first cut merged as #356 on 2026-09-20 and was never tagged. By 2026-10-04 nine roadmap entries,
an issue with no entry (#375) and a nightly-build change with no entry (#397) had merged to `main`
after it. A tag cut from `main` carries them whether or not the paperwork names them, so the
paperwork moves to the tree: the CHANGELOG's `[Unreleased]` section becomes `[7.4.2] - 2026-10-04`
with the #375 entry it was missing, `docs/release-notes.md` and `docs/roadmaps/releases.md` describe
thirteen fixes and two changes to the nightly build instead of four fixes, the nine entries are
marked completed with `release:v7.4.2`, and the book carries the release, which
`scripts/check-book-updated.sh` requires of a release commit.

The rows are the eight squash commits in `git log 33c923bc01..1241570740`: everything that merged to
`main` after #356 (`33c923bc01`) up to the gate tree, read from the log rather than from the roadmap.
The third column leaves out each commit's CHANGELOG, roadmap and `.pmat/baseline.json` lines.

| Change | Commit | What it touched |
|---|---|---|
| SC2086 and SC2154 lint the masked copy, so `$name` inside `'...'` or after a trailing `#` is text (PMAT-362) | `cf672f7768` (#363) | `quoting.rs`, `mod_lint.rs`, `mod_lint_2.rs`, the payload guard |
| Eight rules join `QUOTE_SENSITIVE_RULES` (PMAT-364) | `a023ca02a8` (#365) | `quoting.rs`, the payload guard |
| SC2107 blanks `$( )`, `$(( ))` and backticks before it looks for `\|\|` and `&&` (PMAT-366) | `3b8232c0e8` (#367) | `sc2107.rs` |
| The nightly's Linux legs build on glibc 2.31 and are checked on 2.35; an aarch64-linux asset ships (PMAT-380, #385) | `ec19571390` (#394) | `nightly.yml` |
| The nightly builds whenever the `nightly` tag is not at HEAD | `19b2a1249b` (#397) | `nightly.yml` |
| SC1087 and SEC012 inside a single-quoted jq program (#375) | `ee4548b994` (#399) | `quoting.rs`, `sec012.rs`, the payload guard; it added no CHANGELOG entry, and this commit adds one |
| SC2210 and SC2225 find an assignment by word position, and SC2058 takes `test` only as a command name (PMAT-370, PMAT-371); SC1066 joins `QUOTE_SENSITIVE_RULES` (PMAT-388) | `119ab4b35a` (#372) | `shell_assignments.rs`, `sc2058.rs`, `sc2210.rs`, `sc2225.rs`, `linter/mod.rs`, `quoting.rs`, the payload guard |
| DET002 joins `\`-continued lines and exempts `date -d`, `-r` and `-f` (PMAT-376, PMAT-386) | `1241570740` (#392) | `det002.rs`, `timestamp_flow.rs`, `det002_tests_gh376.rs`, `det002_tests_gh386.rs` |

`1241570740`'s subject also names SC1066 and refers to #388, but the only code it changes is the four
files in its row. The SC1066 change is in `119ab4b35a`: `git log -S'SC1066' v7.4.1..1241570740`
lists no other commit, which is why the table puts PMAT-388 there.

**This commit is docs-only.** Nine paths change between the gate tree `1241570740` and this commit:
#358 added one line to `.pmat-work/ledger.jsonl` and edited `docs/roadmaps/roadmap.yaml` and this
receipt, and this commit edits those two again and changes `CHANGELOG.md`, three pages under
`book/src/linting/`, `docs/release-notes.md` and `docs/roadmaps/releases.md`. None of the nine is a
path a cargo build reads. None is a `.rs` file, a `Cargo.toml`, `Cargo.lock` or `build.rs`, or a file
a macro includes: resolved against the file that names it, each of the seventeen `include_str!` paths
is under `rash/` or `tests/`, 438 of the 439 `include!` paths are `.rs` files under `rash/`, `tests/`,
`bashrs-oracle/` or `rash-mcp/` and the last is `runtime.rs` in `OUT_DIR`, and there is no
`include_bytes!`. Neither build script reads a changed path: `rash/build.rs` reads a
provable-contracts binding and `.rs` sources, and `rash-runtime/build.rs` reads `src/lib.sh`. The two
published crates package files under `rash/` and `bashrs-oracle/` and the README each names, and none
of those changed. One test reads `CHANGELOG.md` at run time and asserts that it mentions complexity,
which it still does. So the gates measured on `1241570740` measured the code this tag ships.

The PR's CI checks this commit again only in part. `.github/workflows/ci.yml` runs clippy with
`--workspace --all-targets` and the workspace's library tests (`test_workspace: true`, which is
`--workspace --lib`). It does not run the integration tests (the keyring test in the table below is
one), the corpus score or the clean room, so those results stand on `1241570740` alone.

The release gate's steps, run one at a time on `1241570740` on 2026-10-04:

| Step | Result on `1241570740` |
|---|---|
| `cargo fmt --all -- --check` | exit 0 |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | exit 0 |
| `cargo test --workspace` | exit 101: `test_INSTALLER_CLI_007_keyring_list` (`rash/tests/cli_installer_tests.rs:334`) failed with "Failed to parse keyring: EOF…". The same command with `--no-fail-fast`: 17590 passed, 0 failed, 180 ignored. 7.4.1's notes record keyring tests racing on the process environment (#339, #341); this is the same test family, and whether it is the same cause was not measured. Recorded, not waived |
| `pv lint contracts` | PASS |
| `make coverage-gate` | exit 0: regions 94.74%, functions 95.69%, lines 95.11% |
| `make dogfood-selflint` | GATE S PASS: 89 files at the ratchet, 0 improved, 94 errors in total |
| `make corpus-score` | 99.4/100 (A+), 18814/18814 passed; B3 behavioral 18570/18814, G cross-shell 18756/18814 |
| `./scripts/check-book-updated.sh` | the book builds and its tests pass; exit 1 only on "Book not updated since last release (v7.4.1)", which this commit answers |
| `clean-room-bashrs` (paiml/infra) | complete, exit 0 |

The corpus row is a second measurement. The first ran with `TMPDIR` set to a directory that the
recipe's outer `bwrap --ro-bind / /` mounts read-only, so the runner's inner sandbox could not start:
B3 and G measured 1844/18814, and the score still read 88.7 (B), under a message that named the wrong
cause (`bwrap not found on PATH`, with `bwrap` on the PATH). The re-run used the same tree, the same
recipe line and the same prebuilt binary, with `TMPDIR` unset. A corpus score that passes while nine
in ten behavioral checks did not run is a gate reporting what it did not measure, and it is recorded
here for that reason.

On the consumer side, also on 2026-10-04: two SC1066 reproducers, cut from scripts a forjar apply
refused, report one error each on 7.4.1 and none on 7.4.2. forjar built against this bashrs reports
no more script errors than forjar built against 7.4.1 on any of eleven machine configs (paiml/infra's
ten, and one on a branch): 110 against 151 in total, and the one SC1066 among them is gone. Of
paiml/infra's 969 shell scripts, 22 report fewer errors and 2 report one more. Each of the two is a
DET002 on a `\`-continued `printf … >> "$INBOX"`, whose one-line form 7.4.1 already reports: the
continuation now reaches the same verdict as the one-line form, which is what #376 asked for.

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
  cmd=release-lint.sh docs/roadmaps/roadmap.yaml --repo . --plan docs/roadmaps/releases.md --notes docs/release-notes.md  claimed_exit=1  rerun_exit=1  log_path=docs/audits/impl-PMAT-354-receipt.md  sha256=0   # R1-R5 clean; exits 1 only on the 4 historical R6 tags. Re-measured 2026-10-04 on the second cut, after the nine entries that merged after the first cut were marked completed with release:v7.4.2. The INVARIANT, not a count: v7.4.2 has no open entry but PMAT-354 itself, which cannot close before the cut. A snapshot count belongs in no receipt on a branch that closes entries — this line quoted v7.4.2(3), then (2), then (1) as its own PRs landed, and a quorum lane refuted it by re-running the command each time
  cmd=make release-gate  claimed_exit=0  rerun_exit=0  log_path=docs/audits/impl-PMAT-354-receipt.md  sha256=0   # the first cut: fmt, clippy, workspace tests, pv lint contracts, coverage-gate, dogfood-selflint, corpus-score, book. The second cut ran these steps one at a time on 1241570740, and their results, including the two that did not exit 0, are the table under "The second cut"
  cmd=./scripts/check-book-updated.sh  claimed_exit=0  rerun_exit=0  log_path=docs/audits/impl-PMAT-354-receipt.md  sha256=0   # the second cut, on this commit: the book builds, its tests pass, and book/ has changed since v7.4.1. On 1241570740 it exited 1 only because book/ had not
  cmd=make -f machines/clean-room/Makefile clean-room-bashrs  claimed_exit=0  rerun_exit=0  log_path=docs/audits/impl-PMAT-354-receipt.md  sha256=0   # in paiml/infra: the published crate builds from crates.io deps alone

## Gate order

1. `make release-gate` on this cut.
2. `clean-room-bashrs` in paiml/infra — the published crate is `rash/`, not the workspace root (`bashrs-specs`), which is why the clean-room target is scoped `-p bashrs`.
3. Merge quorum, 3/3.
4. Tag `v7.4.2` on `main`, verified to be an ancestor-carrier of `94497c22b2`, `109e3fd33c` and `1241570740`, the tree the second cut's gates measured.
5. `make publish-from-tag TAG=v7.4.2`.
6. The forjar pin PR to `bashrs = "7.4.2"`, **done-when intel's `ci-disk-watch-timer-enable` converges** — the measurement that proves the whole chain rather than the version string.
