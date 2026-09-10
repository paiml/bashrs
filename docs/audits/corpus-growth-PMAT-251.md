# Corpus growth — PMAT-251

Six agy generation lanes produced 240 candidate entries (`entries-all.jsonl`). One was a
byte-identical cross-lane duplicate (`lane4#0` == `lane1#21`, both a `pmat comply --strict`
Makefile input); it was dropped per the summary's own instruction, leaving 239 candidates.
None of the 239 collided with an existing corpus input (checked against all 17,942 existing
`input` values).

All 239 were appended with fresh ids (B-16697.. / M-16697.. / D-16697.. continuing the shared
counter, in `entries-all.jsonl` file order) and measured with `bashrs corpus run --format json`
on the 7.0.3 debug binary built into the private target dir
(`/mnt/nvme-raid0/targets/bashrs-251c`). Two failed a hard gate (transpile/output-contains/lint/
determinism) and were moved to `docs/audits/corpus-pending-PMAT-251.jsonl`; the ids were then
renumbered contiguous from 16697 in file order so no id is skipped. **237 entries added.**

## Corpus run headline

| | passed | failed | total | score |
|---|---|---|---|---|
| Before (orchestrator measurement, 7.0.3 binary) | 17942 | 0 | 17942 | 99.36 (A+, V2 weighted grade) |
| After (this ticket, `bashrs-251c` debug binary) | 18179 | 0 | 18179 | 84.4/100 (B) — see note |

**Score note**: the `score`/`C Coverage` component in this worker's `bashrs corpus run --format
json` reads 0.0/15 pts (`coverage_ratio` is 0.0 for every entry, old and new alike) because the
debug binary here was built without coverage instrumentation — that dimension is an artifact of
*how this binary was built*, not of the new entries, and it affects the 17,942 pre-existing
entries identically. Comparing dimensions that ARE meaningful on this binary, new entries are
**not worse than the existing corpus**:

| dimension | new (237) | existing (17,942) |
|---|---|---|
| transpiled | 237/237 (100%) | 17942/17942 (100%) |
| output_contains (B1/B2) | 237/237 (100%) | 17919/17942 (99.87%) |
| lint_clean (D) | 237/237 (100%) | 17940/17942 (99.99%) |
| deterministic (E) | 237/237 (100%) | 17942/17942 (100%) |
| output_behavioral (B3) | 236/237 (99.6%) | 17702/17942 (98.66%) |
| cross_shell_agree (G) | 236/237 (99.6%) | 17886/17942 (99.69%) |
| metamorphic_consistent (F) | 237/237 (100%) | 17873/17942 (99.62%) |

One new entry, `B-16699` (bash), has `output_behavioral=false` / `cross_shell_agree=false` while
still transpiling, matching, linting and determinism-passing — the same class of pre-existing,
accepted softness as `B-143` (a known, unfixable shell-semantics B3 case per corpus history).
It is **kept**: `bashrs corpus run`'s pass/fail gate (and this ticket's acceptance bar) is driven
by the hard dimensions (transpile / contains / lint / deterministic), which `B-16699` passes.

## Added vs pending, by lane / format / tier

| lane | theme | generated | added | pending |
|---|---|---|---|---|
| 1 | bash-agentic-tooling | 40 | 40 | 0 |
| 2 | bash-polyglot | 40 | 39 | 1 |
| 3 | bash-idioms-adversarial | 40 | 39 | 1 |
| 4 | make-agentic-ops | 40 | 39 | 0 (dup, dropped pre-run) |
| 5 | make-polyglot-cron | 40 | 40 | 0 |
| 6 | dockerfile-agentic | 40 | 40 | 0 |
| **total** | | **240** | **237** | **2** (+1 cross-lane dup dropped before measurement) |

| format | added | pending |
|---|---|---|
| Bash | 83 | 1 |
| Makefile | 96 | 1 |
| Dockerfile | 58 | 0 |

| tier | added |
|---|---|
| Standard | 48 |
| Complex | 56 |
| Production | 66 |
| Adversarial | 67 |

## Predicted-hit vs pinned

All 237 kept entries transpiled and matched their **lane-predicted** `expected_output` line
verbatim on the first measured run — **237 predicted-hit, 0 pinned**. No `expected_output` was
rewritten; the lanes' predictions (function headers, `.PHONY:`/`NAME :=`/target lines,
`FROM image:tag`/`WORKDIR`/`EXPOSE` lines) were all correct as generated.

## Pending: 2 entries, both class-grouped with the measured defect

### class: invalid-dsl (1 entry) — the corpus growth found no transpiler defect here
- **M-16768** (`bash-polyglot-makefile-awk-vars`, lane 2): input contains `let sep = \":\";` — a
  stray backslash before the string delimiter that is not valid Rust syntax (a lane escaping bug:
  the intent was `let sep = ":";`). Measured: `Parse error: cannot parse string into token
  stream`. Not a transpiler defect — the parser correctly rejects malformed Rust.

### class: transpiler-defect (1 entry) — this IS a defect the corpus growth found
- **B-16777** (`bash-idioms-adversarial-quotes-metacharacters`, lane 3): input is syntactically
  valid Rust — `let s = "This string has \"double quotes\", 'single quotes', `backticks`, and
  $VAR";`. Transpile-time validation rejects it with `Validation error: Backtick command
  substitution detected in string literal (SC2006): 'This string has "double quotes", 'single
  quotes', '`. **The construct that triggers it: a plain Rust string literal whose *content*
  contains a backtick character.** The SC2006 backtick-substitution heuristic is applied to the
  Rust source string's raw characters before it is known that the string is inert literal data
  (which the emitter would single-quote in the output, making the backtick harmless) — it should
  only fire on backticks that survive into a shell-command context (`exec`/`capture` strings), not
  on an ordinary `let`/`println!` string literal. Filed for a follow-up ticket; not fixed here per
  scope (no Rust source may be edited by this ticket).

## pattern_tags coverage of what was added (237 entries)

| pattern | count | pattern | count |
|---|---|---|---|
| crontab | 13 | flock | 6 |
| python-in-bash | 20 | getopts | 0 |
| perl | 20 | uv | 13 |
| gh | 10 | worktree | 4 |
| pmat | 8 | nohup | 4 |
| cargo | 10 | timeout | 7 |
| trap | 1 | systemd | 8 |
| | | rsync | 7 |
| | | docker-compose | 7 |

`getopts` is the one pattern from the brief's checklist with zero coverage in this batch — none
of the six lanes produced a `getopts`-tagged entry. All other 15 listed patterns are represented.
Other real-world patterns the lanes covered beyond the checklist (by tag frequency): `docker` (48),
`make` (46), `awk` (17), `sed` (11), `jq` (10), `healthcheck` (6), `multi-stage` (7).
