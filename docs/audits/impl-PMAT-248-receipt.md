# impl receipt — PMAT-248

## Identity

| field | value |
|---|---|
| ticket | PMAT-248 (`kind:code`, `orch:fable`, `orch-basis:release`) |
| spec | `docs/audit/quality-report09-2026.md` §8 action 3, §4 (the lexer reads non-shell text as shell) |
| branch | `PMAT-248-lexer-context-fp` from `main` @ `8133676e24` |
| orchestrator | Fable 5.1 (measured by `model-gate.sh`: `model=fable class=fable decision=admit basis=file`) |
| session | `bf151141-60ff-4e66-9794-31fe5c18982d`; second ticket of this session — PMAT-245 shipped 7.0.2 here. The R-5 one-ticket file `goals-<sid>.jsonl` did not survive the host reboot between the two, so `goal.sh set` accepted PMAT-248; the user's instruction to continue in this session is quoted verbatim in the roadmap `notes`. |
| discover.json | `default_branch=main required_check=gate gate_cmd="cargo test --workspace" gate_cmd_fallback=true quorum_tool=agy contracts_dir=contracts code_search="pmat query"` — `gate_cmd_fallback=true`: the repo declares no gate command, so `cargo test --workspace` is the fallback and the CI `gate` job is the authority |
| slots | `slots=3 gh_calls_per_min=30 bank=3` (config-lint PASS) |

## Kind and model gates (Phase 0)

| gate | result |
|---|---|
| `kind-gate.sh PMAT-248 --base main` | `kind=code files=0` exit 0 |
| `model-gate.sh PMAT-248` | `at-or-above-tier: measured tier=1 [A] meets required tier=1 [A]`; exit 0 |
| `config-lint.sh` | PASS |
| `target-guard.sh docs/audit/quality-report09-2026.md` | PASS (under `repo_root`) |

## What was measured before planning

`bashrs 7.0.2` built from `main @ 8133676e24`, each issue's own first reproducer, `bashrs lint` / `bashrs lint <Makefile>`:

| issue | reproducer | 7.0.2 result | verdict |
|---|---|---|---|
| #235 | `echo "Make sure you've done this"` | no issues | already fixed — pin |
| #237 | `if [ $((n % i)) -eq 0 ]` | `SC2046 … $((n % i)` (unbalanced span) | red |
| #241 | `[ "$(echo "$coverage >= 80" \| bc -l)" -eq 1 ]` | SC2122 gone; **SC2047** on `$coverage` | red (new code, same defect) |
| #242 | `cat <<MARKER` … `<li>x &lt; 10</li>` | **SC1109 error** on the body line; SC2276 info on `cat <<MARKER` | red |
| #252 | `--body "… \`[text](url)\` …"` | SC1028/SC1078 gone; **SC2006/SC2046/SC2099** on the escaped backticks | red (new codes, same defect) |
| #255 | `dev-setup: ## Set up local dev environment` | **SC2168 error** on the comment | red |
| #258 | `[ -f /etc/passwd ] # check the file` | no issues | already fixed — pin |
| #261 | `printf 'hello %s\n' "$1"` | SC1012 info | red |

Owning modules (from `pmat query`): `rash/src/linter/quoting.rs` (GH-226/272 literal masking, `QUOTE_SENSITIVE_RULES`), `rash/src/linter/shell_words.rs` (GH-228 quotedness), `rash/src/linter/rules/{sc2046,sc2047,sc1012,sc2276,sc1109,sc2168}.rs`, `rash/src/linter/make_preprocess.rs`, dispatch in `rash/src/linter/rules/mod_lint_2.rs::lint_shell_filtered`. |M| = 6 ⇒ Q1.

## Plan

Contract first: `contracts/linter-lexer-context-v1.yaml` (L2; F-LCX-001..010). End-to-end tests: `rash/src/linter/lexer_context_tests.rs`, one per issue, each asserting the false-positive code is absent on the reproducer **and** still fires on a true-positive twin. Tests for red issues carry `#[ignore = "PMAT-248 phase N …"]` until their phase; the orchestrator removes the attribute after re-running the acceptance command, so worker scopes stay disjoint from the shared test module.

| phase | what | scope_paths | A_i | route (`route.sh`, verbatim) | trigger |
|---|---|---|---|---|---|
| 1 | contract + test module + plan grill | `contracts/linter-lexer-context-v1.yaml`, `contracts/README.md`, `rash/src/linter/lexer_context_tests.rs`, `rash/src/linter/mod.rs` | `pv validate contracts/linter-lexer-context-v1.yaml && cargo test -p bashrs --lib linter::lexer_context_tests` | direct (orchestration: `route=self w=100.00 basis=absent`); grill: `route=agy-plan w=1.00 basis=absent effort=1[U]` | Q2 (plan artifact) |
| 2 | #237 SC2046: `$((` is arithmetic expansion | `rash/src/linter/rules/sc2046.rs` | `cargo test -p bashrs --lib linter::lexer_context_tests::test_PMAT248_gh237 -- --include-ignored` | `route=agy-goal w=1.00 basis=absent note=fable-binding effort=1[U]` | — |
| 3 | #241 SC2047: quotedness via `shell_words` | `rash/src/linter/rules/sc2047.rs` | `… ::test_PMAT248_gh241 -- --include-ignored` | impl (agy-goal first; sonnet-worker fallback under the one-writer rule) | — |
| 4 | #242 + #252 in `quoting.rs`: escapes inside `"..."`; SC1109/backtick rules onto the masked copy; SC2276 only when piped | `rash/src/linter/quoting.rs`, `rash/src/linter/rules/sc2276.rs`, `rash/tests/quoting_literal_payload_guard.rs` | `… ::test_PMAT248_gh242 … ::test_PMAT248_gh252 -- --include-ignored` | impl | — |
| 5 | #255 Makefile: non-recipe lines never reach shell rules | `rash/src/linter/make_preprocess.rs` | `… ::test_PMAT248_gh255 -- --include-ignored` | impl | — |
| 6 | #261 SC1012: printf interprets its format | `rash/src/linter/rules/sc1012.rs` | `… ::test_PMAT248_gh261 -- --include-ignored` | impl | — |
| 7 | release 7.0.3: version, CHANGELOG with the measured corpus score, book check, quorum review of the diff, PR, gate on merge commit, tag, publish, install check | `Cargo.toml`, `Cargo.lock`, `CHANGELOG.md`, this receipt | `bashrs corpus run` ≥ 17,942 entries, ≤ 333 failures; `gh run` gate green on merge commit; `cargo install bashrs --version 7.0.3` | `route=self`; review: `route=agy-quorum w=1.00 basis=absent effort=1[U]` width 3 | Q1 (|M|≥3), pre-PR review |

Estimates: `estimate.sh bashrs 7` → `K_HAT=7 BASIS=first-run[U] ROWS=1` (one pooled row is below the script's pooling floor). K declared 120 with `basis=docs/audits/impl-estimates.jsonl:L1-L2` (PMAT-245: 194 turns for a five-phase release drive including three CI-timeout loops; 98 for the release half alone). `goal.sh set` recorded `k_measured_at_set=247`; `global=k` below is `k_measured − 247`.

## Jidoka

| when | defect | owner | whys | action |
|---|---|---|---|---|
| Phase 1, first commit | `pmat hooks install --strict --force` (required by the skill) makes the pre-commit SATD gate refuse: 17 SATD markers repo-wide against `PMAT_MAX_SATD_COMMENTS=5` | repo, pre-existing | (1) strict refuses over-threshold; (2) the count is repo-wide (`pmat analyze satd` without a path); (3) 17 markers predate the branch; (4) the previous non-strict hook only warned, so nothing forced them down; (5) no ticket owned them | see the SATD section below |

## Status log

(appended at each phase boundary)

## Phase 1 — measured

| check | result |
|---|---|
| `pv validate contracts/linter-lexer-context-v1.yaml` | `0 error(s), 1 warning(s). Contract is valid.` (the shared SCHEMA-013 qa_gate warning) |
| `cargo test -p bashrs --lib linter::lexer_context_tests -- --include-ignored` | **2 passed, 7 failed** — the two pins (#235, #258) green; all seven red tests fail exactly where the contract predicts (`assert_absent`, `lexer_context_tests.rs:27`). RED observed before any fix. |
| `cargo test -p bashrs --lib linter::lexer_context_tests` (as CI runs it) | 2 passed, 7 ignored — the gate stays green between phases |
| `cargo fmt --all -- --check` | clean |
| `pmat comply check` | 166 checks · 69 pass · 15 warn · 10 fail · 72 skip (baseline before this ticket; the failing ids are listed under Universe) |

## Phase 1 — plan grill (delegate, `teamwork`, width 1) and what I re-checked

Lane `6d48c711-9c91-40ad-ba37-91e16048e7b9` (agy 1.2.0), verdict **do-not-implement-as-written**, four claims, all `grounding=asserted`: every `run_command` in the lane failed (`fork/exec /usr/bin/bash: no such file or directory` in agy's remote exec), so it ran no test and read three of the named files not at all. Each claim re-checked here:

| lane claim | my check | outcome |
|---|---|---|
| Phase 2: SC2046's regex is brittle; build it on `shell_words` | `shell_words.rs:368` already treats `$(( … ))` as no expansion (`test_SW_020`); SC2046 today fires on `echo $(date)` and on `$((n % i))` | **accepted** — Phase 2 rewrites SC2046 on `shell_words::simple_commands`, which fixes #237 in the owning layer |
| Phase 4: allowlisting SC2046/SC2006/SC2099 breaks quotedness because masking replaces the `"` delimiters | `shell_words.rs:351` already honours `\$ \` \" \\` inside `"…"`; SC2046 will no longer read the masked copy at all (Phase 2) | **accepted in effect** — backtick rules move to `shell_words` or the allowlist, the worker decides against the twin; SC1109 (text match on heredoc bodies) goes on the allowlist |
| Phase 5: blanking non-recipe lines blinds MAKE rules | `mod_std.rs::lint_makefile` feeds MAKE001–020 the original `source`; only SC2133/SC2168/SC2299 see the preprocessed text | **claim does not hold** — Phase 5 as written, plus a test that a MAKE rule on a non-recipe line still fires |
| Phase 6: SC1012's safe set must include awk, sed, perl, jq, ruby | measured: `echo 'a\nb'` already draws SC2028 and SC2271; shellcheck's SC1012 never fires inside single quotes (#261); bashrs's single-quote meaning is the #236 defect class | **revised further** — SC1012 takes shellcheck's meaning: an unquoted `\t \n \r` the shell drops (`echo a\tb`, today SC2025/SC2042 only); nothing inside `'…'` |
| `#[ignore]` scaffolding "unacceptable" | — | **accepted** — each phase adds its test in its RED commit; the module now holds only the two pins; the six stashed tests live in the scratchpad until their phase |

Twins measured against the 7.0.2 binary (all fire today): SC2046 `echo $(date)`; SC2047 `[ $x -eq 1 ]`; SC1109 `echo a &lt; b`; SC2276 `cat <<EOF | grep x` (and, the defect, plain `cat <<EOF`); SC2006 `` echo `date` ``; SC2168 `<TAB>local x=1` and also `x := 1 # local note`; SC1012 `echo 'a\nb'` (to become SC2028/SC2271 only).

## Phase 1 — SATD lane (delegate, `goal`, width 1, `writes=true`) — PMAT-249

Lane `a78fc54a-e681-4961-9d2b-69cf3476c1b0`: **exit 3, LANE ISOLATION VIOLATED** — it edited the shared checkout named in the prompt instead of its worktree (26 comment sites in 20 files). Kept: the ten listed items (rules a/b/c as briefed, three pairs byte-identical) and nine restatements the lane found on its own (`ast/restricted_expr.rs`, `ast/visitor.rs`, `rules/mod.rs` + `rules/mod_helpers.rs` pair, `rules/mod_lint_2.rs` ×2, `rules/sc2164.rs`). Reverted: `ir/mod.rs` (two `FIXME(PMAT-238)` on a live ticket), `cli/corpus_weight_commands.rs` (module doc made less accurate), `rules/sc1009.rs` (doc examples), `rules/sc1127.rs` (its own `//`-comment examples emptied — restored, then the example text reworded so it stays an example of a `//` line). Root cause named by the delegate: a `--writes` brief must not name the absolute repo path. Lesson applied to every later brief. Both delegate runs returned `partial=true`; under R-4 the implementation phases fall back to `paiml-impl-worker` (sonnet), named here as the reason.
