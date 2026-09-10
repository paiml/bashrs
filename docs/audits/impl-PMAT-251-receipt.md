# impl receipt — PMAT-251 (kind: code) — release v7.0.4

## Identity

| field | value |
|---|---|
| ticket | PMAT-251 — NEXT-RELEASE after 7.0.3: remaining open defects, pmat comply, dependencies, corpus growth, roadmap items |
| kind | code (`kind:code`, `orch:fable`, `orch-basis:release` quoting the operator) |
| branches | `PMAT-251-release` (deps, comply, #254, docs, version) ← merged `PMAT-251-fix` (rule fixes) and `PMAT-251-corpus` (corpus growth); each in its own worktree under `/mnt/nvme-raid0/wt/` with its own target dir |
| release | PR #297 → merge `5ce6bfe4f7` (2026-09-10 14:22Z, merge commit; `gate` re-run on the merge commit by CI run 34488622059) → tag `v7.0.4` → GitHub release https://github.com/paiml/bashrs/releases/tag/v7.0.4 (published 2026-09-10 15:26:45Z by release.yml run 34495173909 attempt 2 — attempt 1 failed at `cargo package -p bashrs --locked` because it ran 16 s before `bashrs-oracle 7.0.4` was on the index; re-run, not edited) → crates.io `bashrs-oracle 7.0.4` and `bashrs 7.0.4` published 15:22:43Z and 15:23:26Z (`publish-oracle.log`, `publish-bashrs.log`, both exit 0; `cratesio-check.log`: max_version 7.0.4, yanked=false); `cargo install bashrs --version 7.0.4 --locked` into a scratch prefix → `bashrs 7.0.4` (`install-704.log`) |
| orchestrator | Fable 5.1, session `bf151141-60ff-4e66-9794-31fe5c18982d`; slots=3, bank=3 |
| discover.json | main checkout sha256 `3ebc73541c51bea0…`; per-worktree discover.json regenerated (repo keys 4101953014, 3739087144, 3646537572); `gate_cmd_fallback=true` (`cargo test --workspace`; CI's gate is `cargo test --workspace --lib` under nextest) |
| operator basis | roadmap `notes:` of PMAT-251 and PMAT-252 quote the operator verbatim, including: "update crates.io dependencies to latest working version", "use \"agy\" when needed to increase the corpus with examples and manage process via dedicated worker: opus max --- watching agy. [ensure corpus variety, and especially patterns we use in work like \"crontabs\", makefiles, [perl, bash, python] mixed with bash. all common patterns for agentic work and across sovereign repo …]" |

## Admission summary (receipt-lint fields)

orch_model: fable [V]   orch_class: code   orch_decision: admit (kind:code + orch:fable + orch-basis:release; `goal.sh set` refused under R-5 because PMAT-248 was set in this session — refusal recorded, no state edited)   orch_basis: release
fable_binding: true   quota_age_h: absent   quota_mark: ?   k_measured_at_set: 310

routes:
  ph1  class=cross  route=agy-goal  w=1.00  basis=absent  fallback=opus-worker (agy lanes cannot run cargo — measured PMAT-248; §11 cross-crate → opus)
  ph2  class=research  route=agy-grillme  w=1.00  basis=absent  executed-as=agy-quorum width 6 with a generation schema (the operator's "dedicated worker … watching agy" is the opus delegate)
  ph3  class=impl  route=agy-goal  w=1.00  basis=absent  fallback=sonnet-worker (same lane limitation)
  ph4  class=impl  route=agy-goal  w=1.00  basis=absent  fallback=sonnet-worker
  ph5  class=impl  route=agy-goal  w=1.00  basis=absent  fallback=sonnet-worker
  ph6  class=impl  route=agy-goal  w=1.00  basis=absent  fallback=sonnet-worker
  ph7  class=orchestration  route=self  w=0  basis=absent
  ph8  class=orchestration  route=self  w=0  basis=absent

verification:
  cmd=bash a-deps.sh (ph1 acceptance: lockfile current, every outside-requirement crate upgraded or excused, workspace builds)  claimed_exit=none  rerun_exit=0  log_path=docs/audits/logs/PMAT-251/deps-accept4.log  sha256=6a6bfee9a0eeba40515694b757b614e3e290a4242b20e3010ead4f4f410ed3fc
  cmd=cargo test --workspace --lib (ph1, deps branch; 15,269 passed 0 failed)  claimed_exit=none  rerun_exit=0  log_path=docs/audits/logs/PMAT-251/deps-gate-condensed.log  sha256=2cf87bf235931a8031f4d8d284784d2c4b5b688deab60c9ff64af05158fc3e02
  cmd=cargo test --workspace --lib (ph3, corpus branch)  claimed_exit=none  rerun_exit=0  log_path=docs/audits/logs/PMAT-251/corpus-gate-condensed.log  sha256=1d5c0ef0fdbd688ee4dbc3a7e3b16d81964767d1761ca29344dfa53e1c7d323b
  cmd=bash a-corpus.sh + bashrs corpus run --format json with the 7.0.4 release binary (18,178 passed 0 failed, 84.4/100 B)  claimed_exit=none  rerun_exit=0  log_path=docs/audits/logs/PMAT-251/corpus-run-704.log  sha256=59512517044758d0c866d557f4236ca086f06d00a1c9dbb9402b5225bcf62b24
  cmd=cargo test --workspace --lib (release branch before the sysinfo/renacer pins; 15,288 passed 0 failed)  claimed_exit=none  rerun_exit=0  log_path=docs/audits/logs/PMAT-251/final-gate-condensed.log  sha256=ee21362b941a4cb9a39931b35a529cf54d0d791c01decff05a60fb1405be9ade
  cmd=cargo test --workspace --lib (release branch at 67a2946ce2, after the pins; 15,288 passed 0 failed)  claimed_exit=none  rerun_exit=0  log_path=docs/audits/logs/PMAT-251/final-gate2-condensed.log  sha256=6423ec57c13ea0941323967e931cd896c325a5c41f6725ca3388f625ac2010a5
  cmd=cargo test -p bashrs --lib -- bench sysinfo (140 passed after sysinfo 0.38)  claimed_exit=none  rerun_exit=0  log_path=docs/audits/logs/PMAT-251/bench-tests-condensed.log  sha256=9d76fb8fcbe6b4167bcec7c978d2bc29e8d2b7c90dd38314c99497799afeb54c
  cmd=bashrs lint on the seven issues' reproducers with the release binary (ph4-ph6 claimed green in partial receipts)  claimed_exit=0  rerun_exit=0  log_path=docs/audits/logs/PMAT-251/repro-transcripts.log  sha256=e9cbd5065ba34a3f651f9ed2391fcab5b6608ff43c4baef394df1ac4c4b09205
  cmd=cargo publish --dry-run -p bashrs-oracle --locked  claimed_exit=none  rerun_exit=0  log_path=docs/audits/logs/PMAT-251/publish-dry-oracle.log  sha256=eec9e79516290a616c41f4d4ca130fab05a0b6152c51081f038a2cb85492ca73
  cmd=cargo publish --dry-run -p bashrs --locked (101 expected until bashrs-oracle 7.0.4 is on crates.io; re-run after publish below)  claimed_exit=none  rerun_exit=101  log_path=docs/audits/logs/PMAT-251/publish-dry-bashrs.log  sha256=6d890d0d4116471133a1309eefa6d6259e80b30a33f30910669e1915625896dc

  cmd=git push origin v7.0.4; cargo publish -p bashrs-oracle --locked (tag checkout, release worktree)  claimed_exit=none  rerun_exit=0  log_path=docs/audits/logs/PMAT-251/publish-oracle.log  sha256=0c8565a153041f89ab7d9ccb8d0f800b130c3acecda3fdfa960aa06e06a30912
  cmd=cargo publish -p bashrs --locked (tag checkout, after the oracle was on the index)  claimed_exit=none  rerun_exit=0  log_path=docs/audits/logs/PMAT-251/publish-bashrs.log  sha256=3069fd302f411a3557316cdf75289bd6f1fc5053adc01ad77a4e67996b173161
  cmd=curl crates.io/api/v1/crates/{bashrs,bashrs-oracle} — max_version 7.0.4, yanked=false  claimed_exit=none  rerun_exit=0  log_path=docs/audits/logs/PMAT-251/cratesio-check.log  sha256=8a712b84170518ce8745ce22855fea956499f905a385b6003ba84e4962695438
  cmd=cargo install bashrs --version 7.0.4 --locked --root <scratch>; bashrs --version; bashrs lint smoke.sh  claimed_exit=none  rerun_exit=0  log_path=docs/audits/logs/PMAT-251/install-704.log  sha256=8a3513ff8f55d0f55f644ad0e2bc0a2bd486de37e348f6ae0eb300a405492391
  cmd=release.yml run 34495173909 attempt 1 (gh run view --log-failed): cargo package raced the oracle publish; attempt 2 success, release visible  claimed_exit=none  rerun_exit=101→0  log_path=docs/audits/logs/PMAT-251/release-run-fail.log  sha256=7390feda2404b185ed8c7158fbea2aa4cec80bc418e6218837b2cb236ed45021

## Plan (8 phases, each with an acceptance command)

| phase | what | A_i | mode |
|---|---|---|---|
| 1 | dependencies to the latest versions that build | `scratchpad/a-deps.sh` (lockfile current; every crate whose latest release is outside the requirement upgraded or excused with the exact error in `docs/audits/deps-exceptions-PMAT-251.txt`; workspace builds — bashrs-wasm test targets built alone, #295) | opus worker (maxTurns twice) + direct finish |
| 2 | corpus generation through agy | six lanes return schema-valid JSON, ≥ 30 entries each | opus delegate, 6 lanes |
| 3 | corpus integration, measured entry by entry | `scratchpad/a-corpus.sh` (every generated entry appended and passing every V2 dimension, or pending with a measured reason; passed == 17,942 + added; failed 0; contract tests) | sonnet worker (maxTurns twice) + direct finish |
| 4 | MAKE010 #256 #257 | `cargo test -p bashrs --lib -- make010` + `pv validate contracts/linter-docker-make-v1.yaml` | sonnet worker (maxTurns twice) + direct finish |
| 5 | SEC011 #264, DET002 #263 | `cargo test -p bashrs --lib -- sec011 det002` + pv validate ×2 | sonnet worker (maxTurns twice) + direct finish |
| 6 | SC2188 #249, SC1009 #238 | `cargo test -p bashrs --lib -- sc2188 sc1009 lexer_context` + pv validate ×2 | sonnet worker (maxTurns twice) + direct finish |
| 7 | #254 deletions, comply files, docs, version, merges | `cargo check --workspace --all-targets`; `./scripts/check-book-updated.sh`; `cargo fmt --all -- --check`; corpus run with the release binary | direct |
| 8 | PR, gate, merge, tag, publish, install check | CI `gate` on the merge commit; `cargo install bashrs --version 7.0.4` | direct |

Estimate: `estimate.sh bashrs 3` printed `K_HAT=3 BASIS=first-run[U]` for the triage sibling; for this ticket the pooled basis is `docs/audits/impl-estimates.jsonl:L3` (PMAT-248: est 7 / actual 36 turns). K̂ 12, K 48. Actual: 64 orchestrator turns (k 310→374 across PMAT-251 and PMAT-252, which ran interleaved).

## Dispatch ledger

| phase | description | agent | model | tool uses | maxTurns | resumed | notes |
|---|---|---|---|---|---|---|---|
| 1 | `PMAT-251/ph1 worker A: dependencies to the latest versions that build` | `a323470750eb3722b` | opus | 41 + resume | hit ×2 | once | 18 commits (cargo update 194; toml, rand, schemars, notify, zstd, base64, ratatui/crossterm, rustyline, phf/hashbrown/itertools, criterion/rstest/assert_cmd/verificar, provable-contracts, safetensors, oracle in-tree, entrenar, pins, syn 3); the orchestrator finished: lockfile, sysinfo 0.38, renacer 0.9, exceptions, acceptance, gate |
| 2 | `PMAT-251/ph2.delegate quorum width 6 on corpus generation` | `a2c4a5cce29c0dd5e` (delegate) | opus | 26 | no | no | lane=quorum mode=plan width=6, schema `corpus-schema.json`; conversations `408b8895-90ef-4fb2-965f-99a7625c8449`, `a2a2bf81-599e-46c2-9d1a-c51dca03d3f8`, `4aa9d06a-2fab-4542-a602-72451a3f4984`, `e8eb7f5f-b199-4418-a8cb-9f5666db4e61`, `22c6e776-b4b2-4e97-bbb6-75e8896551a6`, `5579cfd1-486a-4c8f-887e-f5a2d1be428a`; child_conversations 6; `children=6 method=lane-files label=consensus` (width, not agreement — generation lanes carry no verdict); 240 entries, 0 parse failures, 0 repo side effects (lanes ran from an empty scratch cwd) |
| 3 | `PMAT-251/ph3 worker B: corpus integration` | `a5149d47ad66f48d5` | sonnet | 42 + resume | hit ×2 | once | 239 appended, 237 kept after measurement, 2 pending; the orchestrator moved B-16699 (runs cargo under B3) to pending → 236 added |
| 4 | `PMAT-251/ph4 worker B: MAKE010 #256 #257` | `aa995295d4f73afa8` | sonnet | 45 + resume | hit ×2 | once | RED `c46b100912`, GREEN `ab8ead2bc4`; cognitive 33 → split to ≤ 13 |
| 5 | `PMAT-251/ph5 worker C: SEC011 #264, DET002 #263` | `a2b2fdde28643f831` | sonnet | 45 + resume | hit ×2 | once | RED `5d9c70f691`, GREEN `b35fcc36c9`; `rash/src/linter/timestamp_flow.rs` touched (sink classification), named |
| 6 | `PMAT-251/ph6 worker B: SC2188 #249, SC1009 #238` | `a090641d91f1d4b68` | sonnet | 47 + resume | hit ×2 | once | RED `7cf63301cc`, GREEN `6ebf2ec788` |

Slots: peak 3 of 3 (phases 1+PMAT-252 batches; phases 3+4+5). Denials: 0. I-3 at receipt time: `transcript-gate.sh` PASS attempted=31 denied=0 running_peak=3 slots=3 (session-wide: PMAT-248, PMAT-251, PMAT-252; agent_calls=22 resumes=9). Every worker exhausted `maxTurns` after its one resume; per the skill the orchestrator escalated to itself for the remaining orchestration-class steps (acceptance reruns, gates, transcripts, commits of already-green work) and did not resume a second time.

## Verification (claimed vs re-run by the orchestrator)

| check | worker claim | orchestrator rerun |
|---|---|---|
| ph1 `a-deps.sh` | not reached | PASS (`deps-accept4.log`) after fixing the script's own count (it had matched `Updating crates.io index`) |
| ph1 lib gate | not reached | `cargo test --workspace --lib`: 15,269 passed, 0 failed (`deps-gate.log`); `cargo deny check advisories`: ok |
| ph3 `a-corpus.sh` | not reached | 18,179/0 then, after B-16699 → pending, 18,178 passed 0 failed; `corpus_registry_contract_tests` 11/11 |
| ph3 lib gate | not reached | GATE_EXIT=0 on the corpus branch (`corpus-gate.log`) |
| ph4 acceptance | claimed green (partial receipt) | 117 tests passed (make010 sec011 det002 filter); 3 contracts valid; complexity max 6/13, 8/22, 3/3, 9/19 |
| ph6 acceptance | claimed green (partial receipt) | 48 tests passed; 2 contracts valid; complexity 8/16, 5/12 |
| reproducers (7 issues) | workers' transcripts not returned | `repro-transcripts.log`: #256 both cases clean, #257 only `cp` reported, #264 guard clean / unguarded reported, #263 bare `date >` reported ×3 / terminal clean, #249 lone `2>&1` reported / attached clean, #238 comment-led clean / comment-only reported |
| release branch lib gate | — | 15,288 passed, 0 failed (`final-gate.log`); after sysinfo/renacer: 15,288 passed, 0 failed (`final-gate2-condensed.log`, GATE_EXIT=0); bench tests 140 passed (`bench-tests-condensed.log`) |
| corpus, release binary 7.0.4 | — | 18,178 passed, 0 failed, 84.4/100 (B), D 18,176/18,178 (`corpus-run-704.log/json`) |
| `pmat comply check` on the branch | — | 166 checks, 7 fail (7.0.3: 10): CB-400 1,253/5,584 unchanged; CB-200 771 (was 777); CB-1305, CB-1308, CB-1700, CB-1701, CB-2100 |
| book + format | — | `check-book-updated.sh` OK; `cargo fmt --all -- --check` clean |
| publish dry-run | — | `bashrs-oracle` OK; `bashrs` fails until `bashrs-oracle 7.0.4` is on crates.io (path+version dependency) — publish order oracle → bashrs |

## Corpus growth (operator's ask: variety, crontabs, Makefiles, perl/python in bash, agentic patterns)

Pattern frequencies measured across 4,218 sovereign-repo scripts and 223 Makefiles seeded the lane brief (`/run/user/1000/paiml-implement/agy/PMAT-251/<sid>/corpus/brief.md`, inlined into every lane prompt). Six themes: bash-agentic-tooling, bash-polyglot, bash-idioms-adversarial, make-agentic-ops, make-polyglot-cron, dockerfile-agentic. Result: 236 added (Bash 82, Makefile 96, Dockerfile 58; Production 66, Adversarial 67, Complex 56, Standard 48 before the one removal), 237 predicted-hit / 0 pinned on the first run, 3 pending with measured reasons (`docs/audits/corpus-pending-PMAT-251.jsonl`): one malformed input (lane escaping), one unsafe exec string (`timeout 10 cargo test` under B3), one transpiler rejection filed as #294. Full report: `docs/audits/corpus-growth-PMAT-251.md`.

## Jidoka

| defect | owner | action |
|---|---|---|
| `cargo build --workspace --all-targets` fails on bashrs-wasm's probar test target (panic strategy), pre-existing on main | build profile | filed #295, acceptance builds that target alone |
| `array_join` over a local array literal emits unset `$items` | transpiler | filed #293 |
| string literal containing a backtick rejected before emission | transpiler validator | filed #294 |
| workers exhaust maxTurns=40 in every phase | skill / phase sizing | one resume each, then direct finish; noted for the bundle owner |
| pmat pre-commit refused a GREEN commit at cognitive 33 | make010.rs | split into helpers (13) |

## Roadmap items

- PMAT-246 (89 `contracts/work/*.yaml` fail `pv validate`): `pmat comply refresh-bindings` regenerated the binding index (0 bindings) and left the work contracts unchanged; a sample of 8 still fails `pv validate` — stays planned, now also visible as CB-1305 (89/104 unclassified).
- PMAT-250 (lift command substitutions into `shell_words`): not touched; scheduled with the 7.1.0 items (#265, #232, #293, #294, #295).
- PMAT-252 (triage) shipped separately: PR #296.

## Gaps

- `pv_lane`: contracts extended (five `linter-*-v1.yaml` files, `pv validate` valid); no new contract for the dependency phase (its acceptance is a script, `scratchpad/a-deps.sh`, copied to `docs/audits/logs/PMAT-251/a-deps.sh`).
- dogfood: not attached.
- Status blocks were emitted retrospectively at receipt time (`impl-PMAT-251-status.log`), so `k` is the receipt-time measurement in every block.
- The dependabot lane: the ten crate-requirement PRs (#138–#147) are superseded by this release and closed with a comment; the five GitHub-Actions bumps (#150 #154 #206 #207 #208) touch `.github/workflows/*` and are left for a web-UI decision.

## Final status block (from `impl-PMAT-251-status.log`)

```
[status] ticket=PMAT-251 phase=8/8 global=374/48(K=48) k_measured=374 sub=0/3 basis=docs/audits/impl-estimates.jsonl:L3 route=self q=?/? gate=PASS denied=0 filed=#293,#294,#295 blocker=none note="gate green on merge 5ce6bfe4f7 (run 34488622059); tag v7.0.4 pushed; bashrs-oracle 7.0.4 and bashrs 7.0.4 published; release.yml attempt 1 raced the oracle publish, attempt 2 success; cargo install 7.0.4 OK; dependabot #138-#147 closed as superseded"
```

## Verdict

**DONE.** v7.0.4 is on GitHub (merge `5ce6bfe4f7`, `gate` green on the merge commit, tag, release) and on crates.io (`bashrs-oracle 7.0.4`, `bashrs 7.0.4`, installable). Seven of the thirteen open issues are fixed and measured against their own reproducers (#256 #257 #264 #263 #249 #238 #254); #287 closed on a 3/3 quorum (PMAT-252); the dependency refresh is measured by `a-deps.sh` with two pins excused by their exact errors; the corpus grew by 236 measured entries with the variety the operator asked for; `pmat comply check` went from 10 to 7 failures.

Not done, and said so: CB-400/CB-200/CB-1305/CB-1308/CB-1700/CB-1701/CB-2100 still fail comply (the last three are the ruleset-vs-classic-API read, disabled in `.pmat.yaml` with the reason); PMAT-246 and PMAT-250 stay planned; #265 #232 #236 #234 #233 and the three defects filed here (#293 #294 #295) are milestoned 7.1.0 / Backlog; the five GitHub-Actions dependabot bumps await a web-UI decision. Refusals recorded verbatim above: `goal.sh set` (R-5), `model-gate.sh` on the triage sibling, kind-gate on pmat's own ledger row. Every worker hit `maxTurns`; the orchestrator finished the orchestration-class steps itself and did not resume twice. Turns: 64 for this ticket and its triage sibling together (k 310→374), against K̂ 12 — the estimate row is appended for the next run.
