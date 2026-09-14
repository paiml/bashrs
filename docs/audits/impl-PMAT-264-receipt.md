# PMAT-264 receipt — the v7.4.0 close-out

## Identity

| field | value |
|---|---|
| ticket | PMAT-264, kind=code, `release:v7.4.0` |
| branch | PMAT-264-v7.4.0-closeout, from main at 4aa24ca271 |
| tag | v7.4.0 on 4aa24ca271, annotated |
| gate_cmd | `make release-gate` |
| required_check | `gate` |
| author model | claude-opus-5 |

orch_model: opus-5 [V]   orch_class: opus   orch_decision: admit   orch_basis: file
fable_binding: false   quota_age_h: absent   quota_mark: -   k_measured_at_set: refused (one ticket per session; PMAT-248 holds the goal file)

routes:
  ph1  class=measurement    route=self  w=100.00  basis=absent   re-measure #236 and #233 against the released binary
  ph2  class=orchestration  route=self  w=100.00  basis=absent   tag, publish, verify, statuses, the v7.5.0 plan

verification:
  cmd="make release-gate"  claimed_exit=0  rerun_exit=0  log_path=/home/noah/.cache/paiml-implement/logs/PMAT-264/release-gate.log  sha256=cf69d83eba36
  cmd="scripts/publish-from-tag.sh v7.4.0"  claimed_exit=0  rerun_exit=0  log_path=/home/noah/.cache/paiml-implement/logs/PMAT-264/publish.log  sha256=8379224d796f
  cmd="cargo install bashrs --version 7.4.0 --locked"  claimed_exit=0  rerun_exit=0  log_path=/home/noah/.cache/paiml-implement/logs/PMAT-264/install.log  sha256=458ae34ae65e

## The release, as measured rather than assumed

`make release-gate` was run twice. The first run is **discarded**: it started on `main` in the shared checkout, and while it ran that checkout moved to another branch carrying an unrelated linter change (1c533e042f, #331), so the tree it measured was not the tree being tagged. The second run is the record: a detached worktree at 4aa24ca271, nothing else in it.

| check | result |
|---|---|
| `make release-gate` (clean worktree at 4aa24ca271) | **green**, exit 0 |
| workspace tests | 15,693 passed, 0 failed, 68 ignored (62.95s) |
| coverage | **95.08 percent** lines (regions 94.67, functions 95.63), `--fail-under-lines 95` exit 0 |
| gate S | GATE S PASS, 89 files checked, 89 at ratchet, 0 improved, 94 errors |
| `pv lint contracts` | PASS, 0 errors; gate 4 at 86 refs, 86 found, 0 missing |
| corpus, under bwrap | **18,814 entries, 18,814 passed, 0 failed, 99.4/100 (A+)**; bash 99.4 (16,970), makefile 98.6 (1,049), dockerfile 100.0 (795) |
| book | builds, examples pass |
| required check on the merge commit | `gate` green on 4aa24ca271 (run 34686584716) |
| tag | `v7.4.0` annotated on 4aa24ca271, pushed |
| crates.io | bashrs-oracle 7.4.0 then bashrs 7.4.0 published by `scripts/publish-from-tag.sh` (dry run first, exit 0 both times) |
| release.yml | completed success (run 34688384966); GitHub release v7.4.0 published, not a draft |
| installed binary | `cargo install bashrs --version 7.4.0 --locked` → `bashrs 7.4.0`; `bashrs corpus summary` from a fresh directory reports **18,814 entries, 99.4/100 A+** |

## #233, closed with its own measurement

Both halves. The clippy-scoping half shipped in this release (PMAT-263); the "37 of 145 targets do not compile" half was fixed in v6.67.0 and re-measured at **0** compile errors. What the re-measurement exposed is that the required check runs `--lib`, so those targets ran nowhere in CI — the nightly full gate is the answer, and the issue comment says so. Closed, labelled `release:7.4.0`.

## #236, re-measured and re-scheduled

bashrs 7.4.0 against shellcheck 0.8.0 over the same repository (`docs/audits/sc-vs-shellcheck-v7.4.0.md`, logs `/home/noah/.cache/paiml-implement/logs/PMAT-264/i236-*.log`). SC2046 is **72 vs 5**, not the issue's 210:1; five of the thirteen codes it named are now **zero**. It stays open because SC2086 (316 vs 7), SC2154 (108 vs 0) and SC1004 (92 vs 0) still carry non-shellcheck meanings, each with a measured example in the audit. Retagged `release:7.5.0`.

## Statuses and the v7.5.0 plan

| entry | status | why |
|---|---|---|
| PMAT-263 | completed | the release shipped and is verified above |
| PMAT-265 | completed | merged in #330; six falsification tests, 19 corpus entries |
| PMAT-256 | completed | all three criteria shipped in v7.3.0 under PMAT-258; the third was measured at the v7.4.0 gate — the corpus scores identically inside and outside bwrap, byte for byte, and `.quality/last-corpus-run.json` is written to the caller's directory |
| PMAT-264 | completed | this diff |
| PMAT-253 | moved to `release:v7.5.0` | phases 1a–1c, 2, 3b, 4a, 4b, 6, 7a, 7b unchanged; decision D4 (gate S) shipped in v7.4.0 |

Every open issue names a release: #236 and #331 and #234 are all `release:7.5.0`; the backlog is empty. `release-lint` is green (R1–R5) with one open entry, PMAT-253, against a described v7.5.0.

## Gaps

- `release-lint`'s R1 treats a `cancelled` roadmap entry as open work; the local patch that fixes it is discarded on every skill upgrade and had to be re-applied twice in this cycle. Filed upstream as paiml-mcp-agent-toolkit#1326, still open.
- The shared checkout moved branches mid-gate. Release measurement now runs in its own detached worktree; the receipt says which run it kept and why.

verdict: PASS — v7.4.0 is tagged, published, installed and verified from the index; every ticket on it is closed with evidence; every open issue names a release; v7.5.0 is described in both the plan and the notes.

IMPL-PMAT-264-RECEIPT-END
