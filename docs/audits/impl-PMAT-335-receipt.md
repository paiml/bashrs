# PMAT-335 receipt

## Identity

| field | value |
|---|---|
| ticket | PMAT-335, kind=code, closes #335 |
| branch | `fix/335-fix-rewrites-quotes-across-line`, from main at 90b2be2511 |
| gate_cmd | `make pr-gate` |
| required_check | `gate` |
| author model | claude-opus-5 |

routes (`route.sh` was not run for these phases, so no weights are recorded):
  ph1  class=impl           route=self  quote-aware single-quoted segments; SC2081 and SC2016 on them; SC2081's fix made safe-with-assumptions
  ph2  class=impl           route=self  `apply_fixes` refuses a fix that changes the program; the SC2046 test span the net exposed
  ph3  class=orchestration  route=self  CLI regression test, CHANGELOG, discrimination, this receipt

## How it was found

#335. Published bashrs 7.4.0, plain `bashrs fix`, on a working test script:

- `assert_row 'append one entry' PASS "$TD/append.yaml" 'added=1'` became `assert_row 'append one entry" PASS "$TD/append.yaml" "added=1'`. Five arguments became two.
- `trap 'rm -rf -- "${TD:?}"' EXIT` became `trap "rm -rf -- "${TD:?}"" EXIT`, so `$TD` expands when the trap is set, not when it fires.

`bash -n` accepts both, so nothing after the fix noticed.

## The fix

| layer | file | what it does now |
|---|---|---|
| segments | `rash/src/linter/quoted_segments.rs` | pairs single quotes inside the words the shell-word lexer delimits, never across a line, and returns a segment only when the line's bytes at its columns are `'content'` |
| SC2081, SC2016 | `rash/src/linter/rules/sc2081.rs`, `rash/src/linter/rules/sc2016.rs` | both use those segments; SC2081's fix is `Fix::new_with_assumptions`, applied only with `--assumptions`, and escapes `"`, `\` and backticks |
| net | `rash/src/linter/autofix_apply.rs` | `apply_fixes` skips a fix whose result merges two words of a top-level command, adds or removes a command, or adds an unquoted expansion; a command may still gain a word (`mkdir d` to `mkdir -p d`) |
| test | `rash/src/linter/autofix_tests_apply_single.rs` | `test_fix_priority_sc2046_coverage` used a hand-written span (end 22) where SC2046 emits 19, and asserted a substring of a corrupted result; it now uses the emitted span and asserts the whole output |

## Verification: my own reruns

| check | result |
|---|---|
| `make pr-gate` | rc=0 in 88s: fmt, clippy lib, **15,713 library tests passed, 68 skipped** (nextest), `pv lint contracts` gate 4 **88 refs, 88 found, 0 missing** |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | clean |
| `cargo test -p bashrs --test cli_fix_335` | passes: runs the #335 script under bash, runs `bashrs fix` on it, runs it again, and requires identical output |
| CI on 732c163d7b | `gate`, `ci / gate`, `ci / test`, `ci / lint`, `ci / coverage`, `ci / security` and `ci / provenance` pass; `ci / bench` skipped |

## Discrimination

One mutation at a time on the committed tree, `cargo test -p bashrs --lib -- _335_ sc2081 sc2016 QS_335` (42 tests):

| mutation | tests that failed |
|---|---|
| M1, the net never refuses (`if false && rewrite_changes_program(…)`) | 3: `test_335_net_refuses_a_safe_fix_that_merges_words`, `test_335_net_refuses_a_safe_fix_that_unquotes_an_expansion`, `test_335_net_refuses_a_span_that_swallows_the_next_separator` |
| M2, SC2081's fix marked safe again (`Fix::new`) | 2: `test_sc2081_335_fix_changes_meaning_so_it_is_not_safe`, `test_335_default_fix_leaves_the_issue_file_the_same_program` |
| M3, SC2081's rule body from main, new tests kept | 4: `test_sc2081_335_fix_escapes_embedded_double_quotes`, `test_sc2081_335_unquoted_expansion_between_strings_is_not_single_quoted`, `test_sc2081_335_fix_changes_meaning_so_it_is_not_safe`, `test_sc2081_335_no_match_across_string_boundaries` |
| M4, SC2016's rule body from main, new tests kept | 1: `test_sc2016_335_no_match_across_string_boundaries` |
| control, no mutation | 0 of 42 |

## Jidoka

| defect | owner | whys |
|---|---|---|
| `bashrs fix` rewrote a working script into a different program | `rash/src/linter/` | a regex over the raw line paired quotes across words; the fix it fed was marked safe; the replacement did not escape; nothing checked that the result was the same program; the tests asserted diagnostics and substrings, never behaviour |

verdict: PASS. The defect is fixed at all three layers, each layer's tests go red when that layer is reverted, and the #335 script runs identically before and after `bashrs fix`.

IMPL-PMAT-335-RECEIPT-END
