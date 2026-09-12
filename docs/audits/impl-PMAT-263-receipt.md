# PMAT-263 receipt — v7.4.0

## Identity

| field | value |
|---|---|
| ticket | PMAT-263, kind=code, `release:v7.4.0`; title "v7.4.0: the required check runs what it claims (#233), the per-file self-lint ratchet, deps, corpus growth" |
| branch | PMAT-263-v7.4.0, from main at 0744266226's parent |
| discover.json sha256 | ad322bbd7b520cb8… |
| gate_cmd | `cargo test --workspace` (release gate); `make pr-gate` on the branch |
| required_check | `gate` |
| author model | claude-opus-5 |

## Admission and routing (AUTO-IMPL-SKILL-003 §7)

orch_model: opus-5 [V]   orch_class: opus   orch_decision: admit   orch_basis: file (model-gate.sh PMAT-263)
fable_binding: false   quota_age_h: absent   quota_mark: -   k_measured_at_set: refused (goal.sh set: one ticket per session, PMAT-248 was set here)

routes:
  ph1  class=impl          route=sonnet-worker  w=1  basis=absent   gate S (worker A, maxTurns 40; committed by the orchestrator after re-running the gate)
  ph2  class=research      route=agy-quorum     w=6  basis=absent   corpus generation, six blind lanes
  ph3  class=impl          route=sonnet-worker  w=1  basis=absent   #236 measurement (worker B, no output at maxTurns; carried)
  ph4  class=orchestration route=self           w=100.00  basis=absent   #233, deps, docs, book, receipt

verification:
  cmd="make pr-gate"  claimed_exit=0  rerun_exit=0  log_path=/home/noah/.cache/paiml-implement/logs/PMAT-263/pr-gate.log  sha256=0e27876b1dc2
  cmd="make coverage-gate"  claimed_exit=0  rerun_exit=0  log_path=/home/noah/.cache/paiml-implement/logs/PMAT-263/coverage-gate.log  sha256=fff22d8c35cf
  cmd="make dogfood-selflint"  claimed_exit=0  rerun_exit=0  log_path=/home/noah/.cache/paiml-implement/logs/PMAT-263/dogfood-selflint.log  sha256=d8a7d348089e
  cmd="bashrs corpus run (bwrap)"  claimed_exit=0  rerun_exit=0  log_path=/home/noah/.cache/paiml-implement/logs/PMAT-263/corpus-sandboxed.log  sha256=47ed788f2d7d
  cmd="bashrs corpus run (plain, temp cwd)"  claimed_exit=0  rerun_exit=0  log_path=/home/noah/.cache/paiml-implement/logs/PMAT-263/corpus-plain.log  sha256=47ed788f2d7d

## What this diff does, against the ticket's four names

| name in the title | what shipped | evidence |
|---|---|---|
| the required check runs what it claims (#233) | `clippy_args: --workspace --all-targets` in ci.yml; `make release-gate` clippy gains `--workspace`; the tree made clean under the full command; `nightly-full-gate.yml` runs `cargo test --workspace` daily | `cargo clippy --workspace --all-targets --all-features -- -D warnings` exit 0 (measured after the fixes; before them: 4 errors without `-A unused-variables`, 93 with `--all-features`) |
| the per-file self-lint ratchet | gate S: `scripts/dogfood/selflint.sh`, `selflint-ratchet.tsv` (32 rows), `make dogfood-selflint` replaces the advisory target of the same name, wired into `release-gate` | `GATE S PASS 89 file(s) checked, 89 at ratchet, 0 improved, 94 error(s) total`; 5 tests in `rash/tests/dogfood_selflint_gate.rs`, contract F-DOG-001..005 |
| deps | bytemuck_derive 1.12.1, libredox 0.1.24; regex 1.13 / rayon 1.12 requirements | `cargo metadata --locked` OK; sysinfo 0.39 (Rust 1.95), renacer 0.10/0.11 (unicode-width conflict) and aprender 0.66 measured and not taken |
| corpus growth | 203 entries, ids 17501..17703; release bar 17,942 → 18,592 | 240 candidates → 216 transpile → 143 Bash run under bwrap, 0 timeouts → 13 duplicate names dropped; `corpus_registry_contract_tests` 11 passed |

## Phases and routing

| phase | route line | executor | result |
|---|---|---|---|
| 1 gate S | `route=sonnet-worker w=1 basis=quota.json@0h` | paiml-impl-worker A (stopped at maxTurns 40; files left untracked, committed by the orchestrator after re-running the gate and its tests) | delivered |
| 2 corpus | `route=agy-quorum w=6 basis=quota.json@0h` | paiml-agy-delegate, six blind generation lanes | 240 candidates, screened and appended here |
| 3 #236 measurement | `route=sonnet-worker w=1 basis=quota.json@0h` | paiml-impl-worker B | **no output at maxTurns**; not in this diff, carried (see Gaps) |
| 4 #233, deps, docs, book | `route=self` | orchestrator | delivered |

Slots: 3 of 3 used in the first wave; denials 0.

## Verification: claimed against my own rerun

| check | result |
|---|---|
| `make pr-gate` | green: fmt, clippy lib, **15,687 lib tests passed** (nextest, 49s), `pv lint contracts` PASS (0 errors); 61s wall |
| `pv lint contracts` gate 4 | 80 refs, 80 found, 0 missing (the new contract's `--test` references are outside gate 4's scan; each of its five tests was run by name) |
| `cargo test -p bashrs --test dogfood_selflint_gate` | 5 passed |
| `make dogfood-selflint` | GATE S PASS, 89 files, 94 errors, 0 over |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | exit 0 |
| coverage | **95.08 percent** lines (`--fail-under-lines 95`, exit 0); regions 94.68, functions 95.61 |
| corpus, the branch binary, inside bwrap and plain in a temp cwd | identical: 18,795 entries, 18,795 passed, 0 failed, **V2 99.4/100 (A+)**; A 100.0, B1 99.9, B2 99.9, B3 98.7, D 100.0, F 99.6, G 99.7; bash 99.4, makefile 98.6, dockerfile 100.0 |
| corpus contract tests | 11 passed (`F_CORPUS_001` against the raised bar) |
| book | `mdbook build` OK; `check-book-updated.sh` in the release gate |
| release-lint | `open=4 releases=v7.4.0(4) highest_tag=v7.3.0`, R1–R5 green |
| commit trailers | 8 of 8 carry `Pmat-Ticket: PMAT-263` |

`make pr-gate` was red once on this branch: `test_REG_COV_053_bash_shellcheck_consistency`, because the sixty new Makefile entries carried `shellcheck:true` (the lanes copied the prompt's example). Fixed as a sixty-token change (b176057616); the second run was green.

## Findings the corpus work produced (filed, not fixed here)

- PMAT-265: `std::env::var("X")` lowers to the nonsense command `$(std::env::var X)`; `std::env::var("X").unwrap_or(d)` and `.unwrap_or_else(|_| d)` fail the transpile although a bare variable receiver is lowered since 7.3.0 (20 candidates); `$((…))` inside a `capture()` string is refused as command substitution by `validate_string_literal_in_exec` (3 candidates). One diff, one ticket, after this one merges.

## Jidoka

| defect | owner | whys |
|---|---|---|
| release-gate's clippy line lints only the stub package | Makefile | bare `cargo clippy` at a root that is both workspace and package → root package only → the same GH-214 class fixed for tests but not for clippy → nothing measured what the line linted → widened and measured |
| worker B produced nothing in 40 turns | orchestration | the #236 phase (measure two rules against shellcheck on a third repository, fix the largest class, write an audit) is larger than one worker budget; it is re-sized as its own ticket rather than resumed |

## Gaps

- #236 (SC2046/SC2086 over-reporting versus shellcheck): not measured in this cycle; stays `release:backlog` and is the first candidate for v7.5.0 as its own ticket.
- PMAT-253's remaining phases (1a–1c, 2, 3b, 4a, 4b, 6, 7a, 7b) are unchanged; decision D4 is the part that shipped.
- PMAT-256: its three criteria shipped in v7.3.0 under PMAT-258; the third ("scores identically inside and outside the sandbox; `.quality/last-corpus-run.json` written to the caller's directory") is measured above: the two full runs of this branch's binary agree on every dimension, and each wrote `.quality/last-corpus-run.json` under the directory it was started in. The close-out records it as completed.

verdict: PASS — the four names in the ticket's title are each delivered and re-measured here; the phase that produced nothing is carried as a gap, not claimed.

IMPL-PMAT-263-RECEIPT-END
