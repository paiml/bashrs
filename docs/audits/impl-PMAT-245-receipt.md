# impl receipt — PMAT-245

## Identity

| field | value |
|---|---|
| ticket | PMAT-245 (`kind:code`, critical) |
| issue | #284 · PR #285 · report PR #283 |
| branch | `PMAT-245-restore-corpus` off `main` |
| base HEAD | `3c50327fff` (v7.0.1) |
| branch HEAD | `bdb23bfd31` |
| discover.json | `default_branch=main` `required_check=gate` `gate_cmd=cargo test --workspace` (**`gate_cmd_fallback=true`**) `code_search=pmat query` `quorum_tool=agy` |
| Phase 0 gates | `kind-gate`=0 (`kind=code files=1`) · `model-gate`=0 (`model=opus class=opus decision=admit basis=file`) · `config-lint`=0 (`slots=3`) · `target-guard`=PASS |

## Plan and routing

| phase | what | route (verbatim) | trigger | acceptance `A_i` |
|---|---|---|---|---|
| 1 | RED: assert `load_full().len() >= 17_000` | `route=self` | — | `cargo test -p bashrs --lib test_PMAT245_corpus_registry_not_empty` fails |
| 2 | Restore `corpus_data.rs` from blob; drop orphaned stubs | `route=agy-goal w=1.00 basis=quota.json@39h note=fable-binding effort=1[U] bucket_collision=true` → **taken `self`**: the restore is one `git cat-file` redirect and a build, i.e. mechanical with `\|M\|=1`; delegating it would have cost a lane to run a command I must re-run anyway | — | binary reports ≥ 17,942 entries; `A_1` passes |
| 3 | Review quorum + act on findings | `route=agy-quorum w=1.00 basis=quota.json@39h effort=1[U]` | Q2 (spec artifact), Phase-4 pre-PR review | every lane finding re-executed |
| 4 | DoD, receipt, PR | `route=self` | — | CI `gate` green |

`K̂=4 BASIS=first-run[U] ROWS=0`.

## Dispatch ledger

| # | mode | agent | lane | width | turns | conversations |
|---|---|---|---|---|---|---|
| 1 | delegate | `paiml-agy-delegate` (opus) | quorum (audit) | 3 | 13 tools | `a1cc9b96`, `5eaa973d`, `e828564b` |
| 2 | subagent | `general-purpose` (sonnet) | GitHub ledger | 1 | 29 tools | — |
| 3 | subagent | `general-purpose` (sonnet) | in-repo measurement | 1 | 63 tools | — |
| 4 | delegate | `paiml-agy-delegate` (opus) | quorum (diff review) | 3 | 23 tools | `c67d2e5f`, `7db9ad63`, `3d82ab76` |

`child_conversations=3+3=6` agy lanes. **slots: `attempted=4 denied=0 running_peak=3 slots=3`** (`transcript-gate.sh` PASS, `segments=126 files=4 resumes=0 workflow_started=0`).

## Verification table — claimed vs my rerun

| claim | source | my rerun | verdict |
|---|---|---|---|
| `corpus run` = 0 entries | subagent 3 | ran on 2 independently built binaries | **CONFIRMED** |
| loss caused by `a98f6a89d0`/`ee8c6a2d04` | **me, in #284** | `git log` hid a path/lineage break; main's copy was created as *stubs* on 2026-03-25 | **FALSIFIED — corrected in #284 and #283** |
| blob defines 78 loaders → 282 undefined | **me** | grep was `pub fn`, not `fn`: blob defines **9,406**; 0 undefined | **FALSIFIED — restore is a drop-in** |
| `#233`: 37 of 145 test targets don't compile | issue #233 | `cargo test --workspace --no-run` → 0 errors, 113 executables | **FALSIFIED at HEAD** |
| `#234`: 120 functions below grade A | issue #234 | CB-200 reports **774**; 107 F-grade files matches exactly (pre-commit hook agrees) | **understated** |
| lane 1: severe latent duplicate-definition hazard | agy lane | 157 duplicate loader names confirmed, but files unreferenced → never compile in; deletion is a 0.12 s no-op | **overstated, acted on anyway** |
| lane 2: test is inside the required check | agy lane | `cargo nextest list -p bashrs --lib` lists it; `ci.yml` → `test_workspace: true` → `gate` | **CONFIRMED** |
| lane 3: "84.4 + 15 = 99.4 ≈ 99.1" invalid | agy lane | assumes C would be 15/15; nothing measured it | **UPHELD AGAINST ME — retracted** |
| `A_1` RED → GREEN | me | `0 entries` → `ok` | **CONFIRMED** |

## Jidoka

| defect | owner | whys | outcome |
|---|---|---|---|
| corpus loads 0 entries | `corpus/registry` | 5 (see #284) | fixed on this branch |
| my own root cause was wrong | me | `git log -- <path>` hides renames | corrected publicly before any code moved |
| `84.4+15=99.4` unsound | me | assumed an unmeasured dimension | retracted in `bdb23bfd31` |

## Gaps (NotRun, and what closes each)

- **`gate_cmd` full local run — NotRun.** `cargo test --workspace` exceeds the 10-minute tool cap; launched detached, still compiling at receipt time. **Closed by:** the `gate` check on PR #285, which is the authoritative required check. Targeted acceptance (`-p bashrs --lib`) was re-run and passes.
- **Mutation testing — NotRun.** `cargo mutants` on the restored module. **Closed by:** a follow-up run; the corpus itself is data, not logic.
- **`pv` contract — NotRun.** `contracts_dir=contracts` exists but no contract covers the corpus-registry invariant. **Closed by:** a `corpus-registry-v1.yaml` FALSIFY contract asserting `load_full().len() >= 17_000`.
- **Dimension C — NotRun by construction.** `.pmat/coverage-cache.json` is empty for this HEAD, so 15 of 100 corpus points cannot be scored. **Closed by:** populating the coverage cache, tracked with PMAT-243.

## Verdict

**PARTIAL(andon)** — the fix is complete, RED→GREEN is measured, every quorum finding was re-executed and either acted on or refuted with evidence, but the `gate_cmd` was not re-run to completion locally. Promote to `DONE` when PR #285's `gate` check is green.
