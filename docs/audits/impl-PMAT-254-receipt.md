# impl receipt — PMAT-254 (kind: triage)

## Identity

| field | value |
|---|---|
| ticket | PMAT-254 — triage for v7.1.0: release labels on every open roadmap entry and issue, the release goal, the seven stale April entries resolved with evidence, assignment |
| kind | triage (`kind:triage`, `release:v7.1.0`) |
| branch | `PMAT-254-release-triage`, based on main `7fedbd212e` |
| discover.json | sha256 prefix `8b04a877a12c741d`, `repo_root` the triage worktree, `gate_cmd_fallback=true` |
| snapshot source | `gh issue list --repo paiml/bashrs --state open --limit 500 --json number,title,body` |
| snapshot | count 8, sha256 `9617b2709ade8a1b2c0a12ff5fa209bf05e542876df7917cd036b23f8581359c`, stable at receipt time |
| operator basis | the operator wrote, verbatim: "using pmat-implement continue autonomously to next release (all open tickets (triage and tag to releases/assign), update crates.io dependencies to latest working version,defects, pmat comply, pmat goal, roadmap items etc)" |

## Admission summary (receipt-lint fields)

orch_model: opus [V]   orch_class: triage   orch_decision: admit (model-gate: model=opus class=opus decision=admit basis=file; goal.sh set refused, quoted below)   orch_basis: operator
fable_binding: false   quota_age_h: absent   quota_mark: ?   k_measured_at_set: 425

routes:
  ph0  class=orchestration  route=self  w=100.00  basis=absent  (discovery, snapshot)
  ph1  class=mechanical  route=agy-goal  w=1.00  basis=absent  executed-as=direct (agy lanes cannot run gh; the eight issues already carried their labels, milestones and assignees from PMAT-252, so the batch was eight read-backs and one re-measurement)
  ph2  class=orchestration  route=self  w=100.00  basis=absent  (roadmap triage)
  ph3  class=review  route=agy-quorum  w=1.00  basis=absent  (ledger quorum, width 3)
  ph4  class=orchestration  route=self  w=100.00  basis=absent  (receipt, PR)

verification:
  cmd=bash batch.sh verify ledger-1.json --batch batch-001.json --repo paiml/bashrs --ticket PMAT-254 --agent orchestrator (8 rows, 8 mutations read back from GitHub, coverage recorded by pmat)  claimed_exit=none  rerun_exit=0  log_path=docs/audits/logs/PMAT-254/verify-1.log  sha256=fcd23d7d21883d32328738878977999ef8745a514a283b0c7ecbc5b538f74365
  cmd=bash snapshot.sh check snapshot.json (count 8, stable at receipt time)  claimed_exit=none  rerun_exit=0  log_path=docs/audits/logs/PMAT-254/snapshot-check.log  sha256=701b5c91d45e558a52df5557468ee3e2f4499646643e8c122523452178c54208
  cmd=bash kind-gate.sh PMAT-254 docs/roadmaps/roadmap.yaml --base main (the diff touches only the triage rail)  claimed_exit=none  rerun_exit=0  log_path=docs/audits/logs/PMAT-254/kind-gate.log  sha256=2edc5c763f7d48975811a166837721534276a5a8a6d6048b21cc2ce55e11b82e
  cmd=bash release-lint.sh docs/roadmaps/roadmap.yaml --plan releases.md from PMAT-255-release (R1 fails only on the three cancelled entries)  claimed_exit=none  rerun_exit=1  log_path=docs/audits/logs/PMAT-254/release-lint.log  sha256=b39364c44cd5b29ce7313720ebcef188cd24c559082a35710fad36fc0430999f
  cmd=bash receipt-lint.sh <ledger quorum receipt> --kind delegate  claimed_exit=none  rerun_exit=0  log_path=docs/audits/logs/PMAT-254/delegate-receipt-lint.log  sha256=9ec34bc4400cb9d3879bbf34e66f08f7abf44bab1ebc682dc0456dba2843abaf
  cmd=bash receipt-lint.sh triage-receipt.json --kind triage  claimed_exit=none  rerun_exit=0  log_path=docs/audits/logs/PMAT-254/triage-receipt.json  sha256=938f2c0886c49c68676f13301aa194e9cecf81543401b8d97b05c3b01383634a

## Admission gates (Phase 0), verbatim

- `kind-gate.sh PMAT-254` → `kind=triage ticket=PMAT-254 files=1`
- `model-gate.sh PMAT-254` → `model=opus class=opus decision=admit basis=file`
- `config-lint.sh` → `slots=3 gh_calls_per_min=30 bank=3`
- `goal.sh set --ticket PMAT-254 …` → `goal: one ticket per session: PMAT-248 was set here — start a new claude session` (R-5; recorded, no state edited; the operator's instruction covers all open tickets in this session)

## Dispatch ledger

| phase | description | agent | model | lanes | notes |
|---|---|---|---|---|---|
| 3 | `PMAT-254/ph3.quorum quorum width 3 on the v7.1.0 triage ledger and roadmap decisions` | `aab453559d2faa09f` | opus → agy | 3 | conversations `00fab1ba-2f8f-4560-b9c6-86bc633d9115`, `3048be40-aed6-4e76-b12e-91ef509c18c6`, `e5202fdc-88ba-4e60-ae8c-ee5064ed54f7`; child_conversations 3; lane 3 ended on a transport error with an uncounted FAIL |

Slots: the quorum ran beside two PMAT-255 workers, 3 of 3. Denials: 0.

## Triage result — issues (8; verdicts A fix in v7.1.0 · C backlog)

| issue | verdict | release | evidence (measured on bashrs 7.0.4 unless stated) |
|---|---|---|---|
| #295 | A | 7.1.0 | `cargo build --workspace --all-targets` fails on bashrs-wasm's probar test target; cause the dev profile's panic=abort |
| #294 | A | 7.1.0 | literals holding a backtick or `$( )` refused at validation although emitted single-quoted |
| #293 | A | 7.1.0 | array_join/array_len over a local array literal abort `items: parameter not set` |
| #265 | A | 7.1.0 | `# shellcheck disable=DET002` suppresses DET002 silently; a preceding-line `# bashrs disable-line=` is ignored |
| #232 | A | 7.1.0 | the four DET005 examples raise nothing; the duration example raises DET002 ×8 |
| #236 | C | backlog | paiml-mcp-agent-toolkit df867892b, 97 files: bashrs 5,019 vs shellcheck 130 diagnostics (SC2046 66 vs 5, SC2086 316 vs 7); bashrs misses 4 of shellcheck's 12 SC2046/SC2086 lines; evidence posted on the issue |
| #234 | C | backlog | comply CB-200: 771 functions below grade A |
| #233 | C | backlog | not re-measured this round |

Every label and milestone was already in place (PMAT-252); `mutate.sh label` wrote nothing (check-then-write) and `batch.sh verify` read all eight back. The one write is the #236 evidence comment (`mutations.jsonl`, 1 line). All eight carry an assignee.

## Triage result — roadmap (every open entry tagged; release-lint R1–R4)

| entry | decision | evidence |
|---|---|---|
| PMAT-238 | completed | both files it names compile inside the lib: codegen_tests_cstyle_elif.rs through include!() at codegen_tests_ext_generate_generate_generate.rs:320, lexer_tests_tokenize_sim.rs through #[path] at lexer_read_operators.rs:452; the v7.0.4 lib lists 15,294 tests |
| PMAT-239 | completed | purification_property and quickcheck modules compile inside the lib (24 and 33 tests listed) |
| GH-189 | completed | #189 closed 2026-04-07; the file it named (benchmark_publish_tests_write_test.rs) is neither present nor referenced, and the lib builds |
| PMAT-240 | cancelled | empty entry: title DONE, no acceptance criteria, no spec |
| PMAT-241, PMAT-242, PMAT-243 | blocked, release:unscheduled | no acceptance criteria since 2026-04-07; re-scope or cancel is the operator's call |
| PMAT-244, PMAT-250 | release:v7.1.0 | delivered under PMAT-255 |
| PMAT-246 | blocked, release:unscheduled | re-measured: the 89 files are pmat's work-contract stubs; pv 0.65.2 needs `metadata` and `verification_summary.total_obligations`; `pmat comply refresh-bindings` 3.40.0 leaves them unchanged — upstream |
| PMAT-253 | release:v7.2.0, orch:fable removed | model-gate refused it for every model (its basis token was not in the vocabulary) |
| PMAT-254, PMAT-255 | release:v7.1.0 | this triage and the release |

The release goal is `docs/roadmaps/releases.md`, on the release branch because the triage rail admits only `docs/audits/**`, `docs/roadmaps/roadmap.yaml` and `.quorum/**`. release-lint with it: `open=12`, R1 fails only on the three cancelled entries.

## Quorum on the ledger (`docs/audits/quorum-PMAT-254-ledger.json`, agreed=false)

Lanes 1 and 2 returned FAIL on different rows; lane 3 ended on a transport error. No change had two counted lanes. Both counted lanes agreed with 15 rows and called release-lint's treatment of cancelled entries a lint defect. Each proposed change, re-measured:

| proposal | lane | orchestrator |
|---|---|---|
| #265 → E, close now | 1 | refuted — honouring the directive is the bug #265 reports; shellcheck abandons the file |
| #236 → A | 1 | not adopted — the cited "suggested order" is the orchestrator's own PMAT-251 text, not the operator's; the gap is re-measured and posted on the issue |
| GH-189 → blocked | 1 | refuted — the file #189 named is neither present nor referenced, and the lib builds |
| close #287 now | 1 | refuted — #287 closed 2026-09-10T11:13:11Z |
| eight "missing" issues (#264 #263 #256 #257 #238 #249 #254 #287) | 1, 2 | refuted — all eight are closed; the lanes read PMAT-251's roadmap text, not GitHub |
| #234 → A | 2 | not adopted — PMAT-255's comply scope is CB-1351, CB-2100 and CB-400, not CB-200 |

## Close gate (T-6)

No issue closed; no close was proposed that survived re-measurement.

## Jidoka and skill findings

| finding | owner | action |
|---|---|---|
| release-lint counts `cancelled` as open work (R1) | the skill's release-lint.sh | reported; the cancelled entries stay honest |
| the triage rail forbids `docs/roadmaps/releases.md`, the plan release-lint reads | the skill (kind-gate vs release-lint) | the plan lives on the release branch |
| bashrs `lint --format json` prints a tracing INFO line on stdout before the JSON | bashrs CLI (`cli/commands.rs:220`) | to be filed after this receipt, fixed in v7.1.0 |

## Estimates

K̂ 12 (basis first-run[U]), K 40. PMAT-254 and PMAT-255 run interleaved in one session, so no per-ticket actual is claimed; k_measured at this receipt is 450.

## Status

```
[status] ticket=PMAT-254 phase=4/4 global=450/12(K=40) k_measured=450 sub=0/0 basis=first-run[U] route=self w=100.00 q=? gate=PASS slots=0/3 denied=0 red=- filed=- blocker=- next=merge the triage PR, then file the stdout log-line defect
```

## Verdict

verdict: DONE — every open issue and roadmap entry names a release, the stale entries are resolved on measured evidence, nothing was closed, and the quorum's dissent is re-measured row by row.
