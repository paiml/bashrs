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

---

# Resume — session 2 (release drive)

## Identity

| field | value |
|---|---|
| session | `bf151141-60ff-4e66-9794-31fe5c18982d` (Fable 5.1, admitted by `model-gate`) |
| resumed from | branch HEAD `5e56b68419` (session 1's PR #285, merged with `origin/main`) |
| commits added | `a7799ff2ed` lockfile + roadmap · `fa81c8e918` corpus-registry contract · `4babc532bd` quorum acted on · this one (receipt) |
| discover.json | sha256 `5d85098f009109dd…` · `required_check=gate` (source: rulesets) · `gate_cmd=cargo test --workspace` (**`gate_cmd_fallback=true`**) · `contracts_dir=contracts` · `code_search=pmat query` · `quorum_tool=agy` |
| Phase 0 gates | `kind-gate`=0 (`kind=code files=15`) · `model-gate`=0 (`model=fable class=fable decision=admit basis=file`; roadmap label `orch:fable`, `notes: orch-basis:release`, the user's launch words quoted there) · `config-lint`=0 (`slots=3 gh_calls_per_min=30 bank=3`) · `target-guard`=PASS |
| status-line join `[U]→[V]` | `session_id`: **true** — `discover.sh --state-dir` printed `session=bf151141-…`, the lock path is `claude-subagent-bf151141-….lock` · `tasks[].id = agent_id`: **not measured** — one subagent ran (`a0ab62f976ac21d81`), its harness task row was not inspected · `transcript_path` on subagentStatusLine stdin: **not measured** · `k_measured=60` vs `global=60` at the phase-2 block (gap 0; measured with the `jq` line from the skill) |

## Plan and routing (resumed phases)

| phase | what | route (verbatim) | trigger | `A_i` | claimed | my rerun |
|---|---|---|---|---|---|---|
| 1 | regenerate `Cargo.lock` — the release commit `abceb98f73` bumped the workspace to 7.0.2 and left the lockfile at 7.0.1 for all five members, so `cargo publish -p bashrs --locked` would refuse the tag; file PMAT-245 in the roadmap | `route=self w=11.11 basis=quota.json@44h` | — | `cargo tree --locked -p bashrs --depth 0` | — | exit 0 |
| 2 | `pv` contract `corpus-registry-v1.yaml` + `corpus_registry_contract_tests.rs` (closes the NotRun gap above) | `route=agy-goal w=1.00 basis=quota.json@44h note=fable-binding effort=1[U]` → **taken `self`**: one file pair, `\|M\|=1`, an acceptance I must re-run anyway — named as the R-4 deviation it is | — | `cargo test -p bashrs --test corpus_registry_contract_tests` · `pv validate contracts/corpus-registry-v1.yaml` | — | 11 passed · 0 errors |
| 3 | review quorum on the resumed diff | `route=agy-quorum w=1.00 basis=quota.json@44h effort=1[U]` | Phase-4 pre-PR review | every lane finding re-executed | 3/3 PASS | acted on (F-009, F-010, negative control) |
| 4 | `gate` green on PR #285 → merge → `gate` on the merge commit → tag `v7.0.2` → GitHub Release (release.yml) → `cargo publish -p bashrs --locked` | `route=self` | — | crates.io lists 7.0.2 | — | pending |
| 5 | `cargo install bashrs --version 7.0.2`; DONE addendum | `route=self` | — | installed binary reports 7.0.2 | — | pending |

`K̂=5 basis=first-run[U]`. `K` re-declared 40→120 at `k=60` on the user's instruction *"pmat-implement docs/audit/quality-report09-2026.md-Issue #284 autonomously until new release"* — every tool call is a turn, and the default cap assumed a fresh ticket, not a release drive.

## Dispatch ledger

| # | mode | agent | lane | width | tools | conversations |
|---|---|---|---|---|---|---|
| 1 | delegate | `paiml-agy-delegate` (opus), id `a0ab62f976ac21d81` | quorum, run as `--mode plan` (the brief said `mode=review`, which `agy-lane.sh` rejects; the delegate named the mapping) | 3 | 18 | `acab8655`, `74f5275b`, `5e97dc00` |

`child_conversations=3`. **slots: `attempted=1 denied=0 running_peak=1 slots=3`.** `transcript-gate.sh` at the phase-2 block read `attempted=0 … vacuous but honest` — the delegate had not yet run; it is re-run in Phase 4 below. Lane debris: one lane wrote `tmp_diff.patch` (19,787 B, a copy of the diff) into `repo_root` under `--sandbox`; removed, tracked files unchanged. Reduced verdict: `docs/audits/quorum-PMAT-245-ph3.json` (`agreed=true`, `dissent=[]`, `partial=true` solely from lane 1's idle-notice on stderr).

## Verification — claimed vs my rerun

| claim | source | my rerun | verdict |
|---|---|---|---|
| `Cargo.lock` complete for `--locked` at 7.0.2 | lanes 1, 2, 3 | `cargo tree --locked` exit 0; lane 3 also ran `cargo publish -p bashrs --dry-run --locked` | **CONFIRMED** |
| F-001..F-008 fail an empty or truncated registry | lanes 1, 2, 3 | RED observed: release bar mutated to 20,000 → F-001 fails at `corpus_registry_contract_tests.rs:40`; reverted | **CONFIRMED** |
| F-001..F-008 pass vacuously on 17,942 synthetic entries + B-001, because F-007 compares the loader to itself | lanes 1, 2, 3 | true as written | **UPHELD — acted on** in `4babc532bd`: F-009 runs every 50th entry through `CorpusRunner::run_entry_with_trace` (100 % transpile, ≥ 98 % containment), a negative control proves it rejects three filler shapes, F-010 pins B-001/M-001/D-001 by hand and floors input mass at 2.5 MB |
| nothing in the diff blocks tagging today | lanes 1, 2, 3 | `check-book-updated.sh` PASS · `cargo fmt --check` PASS · `cargo deny check advisories` ok · `bashrs corpus run` on the 7.0.2 release binary: **17,942 entries, 17,942 passed, 84.4/100 (B)** — A 17942 · B1 17919 · B2 17919 · B3 17704 · C 0 · D 17940 · E 17942 · F 17873 · G 17889, identical to the CHANGELOG | **CONFIRMED** |
| the corpus data is genuine, not filler | lane 3 | 17,942 lines · 0 duplicate ids · 0 empty fields · Bash 16,431 / Makefile 804 / Dockerfile 707 · 2,908,886 B input | **CONFIRMED** |
| `pv validate` clean | me | 0 errors, 1 warning (SCHEMA-013 `qa_gate`, shared by every in-repo contract) | **CONFIRMED** |

## Jidoka

| defect | owner | whys | outcome |
|---|---|---|---|
| release commit `abceb98f73` left `Cargo.lock` at 7.0.1 | release process | 5 (`.pmat/jidoka.jsonl`) | fixed in `a7799ff2ed` |
| `pv-gate.sh` RED at baseline: 89 tracked `contracts/work/*.yaml` fail PV-VAL-001 while all 11 top-level contracts validate | pmat work artefacts | 5 | **filed PMAT-246**; non-blocking |
| my own F-007 was self-referential | me | the quorum caught it | F-009 / F-010 |

## Gaps (NotRun, and what closes each)

- **`gate_cmd` full local run — NotRun** (exceeds the tool cap). Closed by the `gate` check on PR #285.
- **Mutation testing on the new test file — NotRun.** RED observed by hand instead (F-001 bar mutation; F-009 negative control).
- **Dimension C — NotRun by construction**, unchanged from session 1.
- **`pv-gate.sh` — RED at baseline** for reasons outside this PR (PMAT-246).

## Verdict

**PARTIAL(andon), pending the gate.** Everything on the branch is measured. Promotion to DONE is recorded in a follow-up docs commit once PR #285's `gate` is green on the merge commit, `v7.0.2` is tagged and released, and crates.io lists 7.0.2 with `cargo install bashrs --version 7.0.2` verified.
