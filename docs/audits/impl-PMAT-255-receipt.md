# impl receipt — PMAT-255 (kind: code) — release v7.1.0

## Identity

| field | value |
|---|---|
| ticket | PMAT-255 — v7.1.0: #293 #294 #295 #265 #232 (+ #301 found while measuring), PMAT-244, PMAT-250, PMAT-246, dependencies, pmat comply, corpus growth through agy, PMAT-253 phases 3a and 5 |
| kind | code (`kind:code`, `release:v7.1.0`) |
| branches | `PMAT-255-release` ← merged `PMAT-255-transpiler`, `PMAT-255-linter`, `PMAT-255-corpus`, `PMAT-255-dogfood`, `PMAT-255-publish` and main (PMAT-254); each sub-branch in its own worktree with its own target dir |
| release | PR #307 → merge `02b6bed455` → tag `v7.1.0` → GitHub release https://github.com/paiml/bashrs/releases/tag/v7.1.0 (release.yml run 34554658403, first attempt, 2026-09-11 02:28:36Z) → crates.io `bashrs-oracle 7.1.0` (02:26:17Z), then `bashrs 7.1.0` (02:26:48Z), by `scripts/publish-from-tag.sh v7.1.0` after a clean `DRY_RUN=1` run; both on the sparse index, neither yanked |
| install | `cargo install bashrs --version 7.1.0 --locked` into a scratch prefix gives `bashrs 7.1.0`; `lint --format json` stdout parses as one JSON document; the DET005 example reports DET005 |
| orchestrator | Claude Opus 5 (the session had switched from Fable 5.1 before this ticket); session `bf151141-60ff-4e66-9794-31fe5c18982d`; slots=3, bank=3 |
| operator basis | the operator wrote, verbatim: "using pmat-implement continue autonomously to next release (all open tickets (triage and tag to releases/assign), update crates.io dependencies to latest working version,defects, pmat comply, pmat goal, roadmap items etc).  Also use \"agy\" when needed to increase the corpus with examples and manage process via dedicated worker:  opus max --- watching agy. [ensure corpus variety, and especially patterns we use in work like \"crontabs\", makefiles, [perl, bash, python] mixed with bash.  all common patterns for agentic work and across sovereign repo. you can query via pmat query RAG if needed]" |

## Admission summary (receipt-lint fields)

orch_model: opus [V]   orch_class: code   orch_decision: admit (model-gate: model=opus class=opus decision=admit basis=file; goal.sh set refused, quoted below)   orch_basis: operator
fable_binding: false   quota_age_h: absent   quota_mark: ?   k_measured_at_set: 425

`goal.sh set` → `goal: one ticket per session: PMAT-248 was set here — start a new claude session` (R-5). "pmat goal" is not a pmat subcommand (`pmat g` is `pmat generate`); the release goal is `docs/roadmaps/releases.md`, which release-lint reads.

routes:
  ph1  class=orchestration  route=self  w=100.00  basis=absent  (dependencies: re-check of the three majors, lockfile)
  ph2  class=research  route=agy-grillme  w=1.00  basis=absent  executed-as=agy-quorum width 6 with the generation schema
  ph3  class=impl  route=agy-goal  w=1.00  basis=absent  fallback=sonnet-worker (agy lanes have no dependable shell for cargo)
  ph4  class=impl  route=agy-goal  w=1.00  basis=absent  fallback=sonnet-worker, then direct
  ph5  class=impl  route=agy-goal  w=1.00  basis=absent  fallback=sonnet-worker
  ph6  class=impl  route=agy-goal  w=1.00  basis=absent  fallback=sonnet-worker
  ph7  class=impl  route=agy-goal  w=1.00  basis=absent  fallback=sonnet-worker, then direct
  ph8  class=impl  route=agy-goal  w=1.00  basis=absent  executed-as=direct (#295, #301)
  ph9  class=impl  route=agy-goal  w=1.00  basis=absent  fallback=sonnet-worker, then direct
  ph10  class=impl  route=agy-goal  w=1.00  basis=absent  fallback=sonnet-worker, then direct
  ph11  class=orchestration  route=self  w=100.00  basis=absent  (version, CHANGELOG, book)
  ph12  class=review  route=agy-quorum  w=1.00  basis=absent  (CRUX and diff review, width 6)
  ph13  class=orchestration  route=self  w=100.00  basis=absent  (verification, PR, merge, tag, publish)

verification:
  cmd=bash a-transpiler.sh (#293 #294: tests, reproducers under dash and bash, exec true positives refused)  claimed_exit=none  rerun_exit=0  log_path=docs/audits/logs/PMAT-255/accept-transpiler.log  sha256=13323e0d1fb324e4599f4cce4a9e09ee59300b1a37d430f97ae217eb4c3775b7
  cmd=bash a-suppress.sh (#265: shellcheck-namespace bashrs codes reported and not honoured; silent disable-line reported)  claimed_exit=none  rerun_exit=0  log_path=docs/audits/logs/PMAT-255/accept-suppress.log  sha256=3f19e0853468a93b48485e1162f325899b7d50e46091cbdfb33767bab3f26bfc
  cmd=bash a-det005.sh (#232: DET005 at warning on the four examples, no DET002 on them, neither on durations)  claimed_exit=none  rerun_exit=0  log_path=docs/audits/logs/PMAT-255/accept-det005.log  sha256=5800d87d0d7d86e01ab947f19328f62058d19db98be03b65d58a189fe37b6315
  cmd=bash a-sc2105.sh (PMAT-244: one-line loops clean, top-level break still reported)  claimed_exit=none  rerun_exit=0  log_path=docs/audits/logs/PMAT-255/accept-sc2105.log  sha256=d28832ecfa8747fb9f0f3e4fa76f811b69e37972180e5fd8ee54e32c2d1df5ab
  cmd=bash a-shellwords.sh (PMAT-250: private scanner gone, SC2046 verdicts unchanged)  claimed_exit=none  rerun_exit=0  log_path=docs/audits/logs/PMAT-255/accept-shellwords.log  sha256=d16115db7d6154779b3d9ccfca48a85f24c0db493eec12c74680c6a520388b14
  cmd=bash a-corpus-255.sh (240 generated: 225 added, 15 pending; sandboxed run clean; no new debris)  claimed_exit=1 (the script's own jq -e defect)  rerun_exit=0  log_path=docs/audits/logs/PMAT-255/accept-corpus-255.log  sha256=5ca8535cbbaebbd4de261c1ae75654c2dacc9dcc97a225a1c2b4978b12b79c49
  cmd=bash a-dogfood.sh (gates D G K pass; each declared mutation turns its gate red)  claimed_exit=0 (only with CARGO_TARGET_DIR set)  rerun_exit=0  log_path=docs/audits/logs/PMAT-255/accept-dogfood.log  sha256=0f69aca46db17bbed290f15cad7e81fe3412d4ff244a043d36a28e416555d9f5
  cmd=bash a-publish.sh (dry run order, three refusals, make target, release.yml waits on the sparse index)  claimed_exit=0  rerun_exit=0  log_path=docs/audits/logs/PMAT-255/accept-publish.log  sha256=e7f32a3d852033193b1ecfc1e98035906b912a4126cb4ed34ba6cdf218e2eb1c
  cmd=cargo test --workspace --lib on the release tree (CI's gate command)  claimed_exit=none  rerun_exit=0  log_path=docs/audits/logs/PMAT-255/lib-gate-710.log  sha256=8f002434f665f94cb55d95839a40189ce68f7aa6bcf21d6dc65a333993d92cc7
  cmd=bashrs corpus run --format json inside bwrap, release binary  claimed_exit=none  rerun_exit=0  log_path=docs/audits/logs/PMAT-255/corpus-run-710-summary.json  sha256=f17bfaadba3f126e0c60687cbff0e2586c0c5f422d61cb2644eaf3328bce88b6
  cmd=make dogfood  claimed_exit=none  rerun_exit=0  log_path=docs/audits/logs/PMAT-255/dogfood-710.log  sha256=6f293d162519a8e1f473be292bf024669bf0df434f017549abc64642a41846b3
  cmd=bash a-deps-255.sh (lockfile current, majors excused with errors, every target builds)  claimed_exit=none  rerun_exit=0  log_path=docs/audits/logs/PMAT-255/deps-accept-255.log  sha256=6a6bfee9a0eeba40515694b757b614e3e290a4242b20e3010ead4f4f410ed3fc
  cmd=cargo build --workspace --all-targets (#295)  claimed_exit=none  rerun_exit=0  log_path=docs/audits/logs/PMAT-255/alltargets-255.log  sha256=44bf09c6c5b21e6f92967d96e88c8420393f91a1d9c3bdf670fc56aa8f53c024
  cmd=./scripts/check-book-updated.sh  claimed_exit=none  rerun_exit=0  log_path=docs/audits/logs/PMAT-255/book-check-710.log  sha256=d6fe9d3f7e8d52f66910d6a1f42f1625c34c64fc2ce14ae52353b1a3e702796e
  cmd=cargo fmt --all -- --check  claimed_exit=none  rerun_exit=0  log_path=docs/audits/logs/PMAT-255/fmt-710.log  sha256=1a670175a9a78732a00f898faab898a42e64616999d9999c0d52f80649d64f8a
  cmd=pmat comply check (four checks fail, listed)  claimed_exit=none  rerun_exit=1  log_path=docs/audits/logs/PMAT-255/comply-710-summary.log  sha256=96b8f0bab37dee626e91be1fd53bee64b554d90c7049d56a76e146b141cc6d2a
  cmd=DRY_RUN=1 scripts/publish-from-tag.sh v7.1.0 (bashrs-oracle dry run; bashrs skipped with the reason)  claimed_exit=none  rerun_exit=0  log_path=docs/audits/logs/PMAT-255/publish-dry-710.log  sha256=5a8bceab0b467a410e9f9a639eeb7ec1b18865a46485cdf8150ef7cc52b39d02
  cmd=scripts/publish-from-tag.sh v7.1.0 (bashrs-oracle, sparse-index wait, bashrs; clean detached worktree of the tag)  claimed_exit=none  rerun_exit=0  log_path=docs/audits/logs/PMAT-255/publish-710.log  sha256=544d21b3b7a12392b8d2e4710f14c7155a9e70880a628248da96d9b68e03b289
  cmd=crates.io API and sparse index for bashrs and bashrs-oracle 7.1.0  claimed_exit=none  rerun_exit=0  log_path=docs/audits/logs/PMAT-255/cratesio-710.log  sha256=6b9cdeac0f98a1516238df02247202aae301001b4fde980f0b376a163e67506d
  cmd=cargo install bashrs --version 7.1.0 --locked; bashrs --version; lint --format json smoke  claimed_exit=none  rerun_exit=0  log_path=docs/audits/logs/PMAT-255/install-710.log  sha256=9ff3ca09bab99e57003eae7b1a99ae8d261a9a31d7316e6350a3ccf8ebf245e9

## Dispatch ledger

| phase | description | agent | model | tool uses | maxTurns hit | resumed | notes |
|---|---|---|---|---|---|---|---|
| 2 | `PMAT-255/ph2.delegate quorum width 6 on corpus generation with measured variety` | `a964504143296bda0` | opus → agy | 20 | no | no | six lanes, 240 entries, 0 schema failures; themes chosen by gap against 164 sovereign repositories (2,890 units) |
| 3 | `PMAT-255/ph3.corpus worker C: integrate 233 safe generated entries under a sandboxed corpus run` | `a6e93041c6a1b0897` | sonnet | 74 | yes | once | 225 added, 15 pending; found the jq -e defect in the acceptance script and the cargo shell-function trap |
| 4 | `PMAT-255/ph4.transpiler worker B: #293 array lowering and #294 inert backticks in string literals` | `a65ba370ee454337f` | sonnet | 89 | yes | once | RED 194fdef178, fix left uncommitted; the orchestrator finished it, added the exec guard and later the exact array lowering |
| 5 | `PMAT-255/ph5.suppress worker C: #265 shellcheck-namespace directives and silent disable-line` | `a54c9d3c8943656e6` | sonnet | 88 | yes | once | RED 604a69d05b, GREEN 8eef6bbedd, book a2b7b81805; reused BASHRS001 from #240 |
| 6 | `PMAT-255/ph6.det005 worker B: DET005 time-dependent control flow, and DET002 silent on durations (#232)` | `a76d9b06bb4539233` | sonnet | 82 | yes | once | RED 332ce2a7ed, GREEN 05566399fb, book 6d327c7106 |
| 7 | `PMAT-255/ph7.sc2105-250 worker B: SC2105 one-line loops (PMAT-244), then command substitutions in shell_words (PMAT-250)` | `a63a5aabd7fed0d74` | sonnet | 83 | yes | once | PMAT-244 dcdbaa81f5/011edc8f13, PMAT-250 11a897b9cd/660d195f05; the orchestrator later restored substitutions inside ${…} |
| 9 | `PMAT-255/ph9.dogfood worker C: mechanical dogfood gates D, G, K with GATE lines and applied mutations` | `a7640dea94cdeec01` | sonnet | 87 | yes | once | 2541482281; the orchestrator made the gates resolve the target dir through cargo metadata |
| 10 | `PMAT-255/ph10.publish worker D: publish-from-tag script and the release workflow's wait for the oracle on the index` | `a20a40ec7d69f31a8` | sonnet | 64 | yes | once | ae5ba89f16, 812b6eb969; the orchestrator switched both polls to the sparse index and added the worktree and dry-run refusals |
| 12 | `PMAT-255/ph12.review quorum width 6: CRUX over the 7.1.0 bullets and a blind review of the release diff` | `ab0777f70c6fe5da0` | opus → agy | 22 | no | no | CRUX 3/3 PASS agreed; diff 3/3 FAIL on ten points, each re-measured (docs/audits/quorum-PMAT-255-review.json) |

Slots: peak 3 of 3. Denials: 0 (hook event log). I-3: `transcript-gate.sh` PASS attempted=51 denied=0 running_peak=3 slots=3 (session-wide, PMAT-253/254/255). Every Claude worker stopped at maxTurns 40 and again after its one resume except dogfood and publish; the orchestrator re-ran every acceptance itself against the final release tree (the verification block).

## What shipped

- #293 exact array lowering; #294 inert literals with the exec data-flow guard; #295 dev-profile panic; #265 directive namespaces; #232 DET005 and DET002 durations; #301 logs on stderr; PMAT-244 SC2105 loop structure; PMAT-250 SC2046 on shell_words (verdicts unchanged, including `${var:-$(…)}`); PMAT-253 phase 3a gates D G K and phase 5 publish-from-tag with the sparse-index wait.
- Corpus +225 (`docs/audits/corpus-growth-PMAT-255.md`); seven generated entries excluded before any run because the behavioural check executes Bash entries with the caller's cwd, HOME and PATH; every corpus measurement ran inside bwrap.
- Dependencies: toml 1.1.6, toml_edit 0.25.14; sysinfo 0.39, renacer 0.11, aprender 0.66 re-measured as still refused (`docs/audits/deps-exceptions-PMAT-255.txt`).

## Review (forjar style)

`docs/audits/crux-7.1.0.md`: twelve behaviour bullets × at least three systems, three blind lanes, agreed. Two dissent rows propose adopting AST-based loop containment for SC2105 and cargo-release for publishing — the operator's decisions. The diff review failed the release on ten points; re-measured: P2, P6, P8, P9 fixed before merge; P3 (#302), P4 (#304), P5's subshell case (#303) filed; P1 refuted as an injection (method-call exec emits no eval; the silent drop is #305); P7 and P10 had no defect; `.len()` → 'unknown' found while re-measuring is #306.

## Jidoka

| defect | owner | action |
|---|---|---|
| #294's relaxation let a literal reach eval through a variable or parameter (the lib gate's injection tests failed) | transpiler validation | `validation::exec_flow`: every literal is held to the substitution rule when any exec/capture argument is dynamic |
| the orchestrator merged the transpiler branch while its guard commit had been refused by the complexity hook | orchestrator | re-merged after splitting `walk_expr`; commit exits are now checked before merging |
| `bashrs lint --format json` printed a tracing line on stdout | CLI | #301, fixed |
| the corpus runner executes generated entries with the caller's environment | corpus runner / process | screen + bwrap for every measurement; worth a runner-side sandbox (not filed yet) |
| the Bash tool's `cargo` is a shell function that replaces CARGO_TARGET_DIR | environment | `command cargo` in the tool shell; recorded in memory |
| acceptance-script defects of the orchestrator's own (jq -e over JSONL, a regex tied to the web-API URL, dogfood's dependence on CARGO_TARGET_DIR) | orchestrator | fixed and re-run |
| release-lint counts cancelled entries; the triage rail forbids the plan file release-lint reads; headless /teamwork never fans out; fanout.sh counts lane-reduce.json | the skill / agy | reported, not edited |

## Gaps

- pmat comply still fails CB-400, CB-200, CB-1305 (PMAT-246 upstream) and CB-2100; CB-1351 passes after `refresh-bindings`.
- Dogfood gate B (comply) waits on decision 4 of `docs/specifications/pr-dogfood-parity-forjar.md`; the quorum-receipt gate (phase 1) does not exist yet, so this PR carries no `.quorum/` receipt.
- PMAT-241/242/243 stay blocked on definition; PMAT-246 blocked upstream; PMAT-253's remaining phases are v7.2.0.

## Estimates

K̂ 48 (basis docs/audits/impl-estimates.jsonl:L4, PMAT-248's 36), K 96. PMAT-254 and PMAT-255 ran interleaved from k=425 to k=476; no per-ticket split is claimed.

## Final status block

```
[status] ticket=PMAT-255 phase=13/13 global=476/48(K=96) k_measured=476 sub=0/0 basis=docs/audits/impl-estimates.jsonl:L4 route=self w=100.00 q=? gate=PASS slots=0/3 denied=0 red=- filed=#301,#302,#303,#304,#305,#306 blocker=PMAT-246(upstream) next=none
```

## Verdict

verdict: DONE — v7.1.0 merged green, tagged, released and on crates.io; every fix measured against its issue's reproducer on the final tree; the review's real findings fixed before merge and the rest filed for v7.2.0.
