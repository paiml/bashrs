# PMAT-439 — receipt: SC2075 pairs a backslash string with the next quoted string

**Ticket:** PMAT-439 (bashrs#439): SC2075 reports an [error] on a valid `'\\'` or `'\'` when another
single-quoted string follows on the same line. **Kind:** code.
**Branch:** `fix/439-sc2075-backslash`, from `63abc48ec4`.

## The defect

SC2075 was the regex `'[^']*\\'[^']*'`. That regex reads `\'` as an escaped quote. Inside '...' a
backslash is literal and the next quote closes the string, so in

    echo 'a\b' | tr -d '\\' | grep -x 'ab'

it matched `'\\' | grep -x '`, from the opening quote of `'\\'` to the opening quote of `'ab'`, and
reported 1:20-36 as an Error. bash runs the line and prints `ab`. Any caller that refuses a bashrs
error therefore refuses a valid script.

## The fix

A per-line scanner replaces the regex. It tracks the quoting context the line is in: code, `$( )`,
backticks or `"..."`. It skips comments, escapes in code and `$'...'` bodies, and steps over `$$`, so
the quote after `$$` opens a plain '...'. bash confirms this: `echo $$'a\nb'` prints the pid, then
`a\nb` literally. Inside '...':

- An attempt is a `\'` followed by a letter, digit or underscore, as in `'it\'s'`, `'can\'t'` or
  `'a\'1'`. It is reported only when a later quote on the line closes the string its author meant.
- Every other backslash is literal, and the next quote closes the string: `'\\'`, `'\'`,
  `'a\'"b"`, `'a\'$x`, `awk -F'\' '{print $2}'`.
- With no later quote on the line, `echo 'a\'b` is the valid word a\b (bash prints `a\b`) and is not
  reported.

`linter::quoting` is not used. It reads the line the way the shell does, so it closes the string at
`\'` and is out of step with the author from there on. The existing `'don\'t' 'won\'t'` test needs
two reports. The rule header says why.

The four existing true-positive fixtures keep their spans: `echo 'can\'t'` 1:6-14,
`msg='it\'s broken'` 1:5-19, `echo 'don\'t' 'won\'t'` 1:6-14 and 1:15-23, and
`result=$(echo 'can\'t')` 1:15-23. Max cyclomatic complexity in the file is 7.

## Where each claim lives

| Claim | Where |
|---|---|
| the reproducer, the lone `'\'` form and the `"$( ... )"` form report no SC2075 | `rash/src/linter/rules/sc2075.rs`, `test_PMAT439_sc2075_backslashes_then_another_quoted_string`, `..._lone_backslash_then_another_quoted_string`, `..._same_pipeline_inside_quoted_substitution` |
| the whole linter reports no Error from any rule on those three; `echo 'it\'s'` still reports SC2075 | `rash/src/linter/lexer_context_tests.rs`, `test_PMAT439_gh439_backslash_before_a_closing_single_quote_is_literal` (contract F-LCX-021) |
| a backslash string followed by a separate word is clean, including `awk -F'\'` and `'a\'b` with no later quote | `test_PMAT439_sc2075_backslash_string_then_a_separate_word` |
| `\'` inside `"..."`, `$'...'` or unquoted code is not an attempt | `test_PMAT439_sc2075_not_single_quoted_context` |
| the true positive still fires as an Error: alone, after a valid backslash string, with a digit, and after `$$` | `test_PMAT439_sc2075_true_positive_still_fires` |
| an attempt followed by a space, `'can\' t'`, is not clean: SC1078 reports the quote it leaves open | lexer_context_tests, same test, the `shell_fires(..., "SC1078")` line |
| contract rows | `contracts/linter-lexer-context-v1.yaml`: the GH-439 reference, the SC2075 invariant, F-LCX-021 |

## Commits

| Commit | What |
|---|---|
| `121a2b041e` | RED: tests, contract rows, roadmap entry |
| `0bdc20364d` | GREEN: the scanner |
| `3a1a4f72dc` | pins `awk -F'\'` and the SC1078 control |
| `b797e6b2eb` | an attempt is reported only when the string it meant closes; `$$` |

The commit that adds this receipt adds only files under `docs/audits/`.

## Evidence that the tests discriminate

- At `121a2b041e`, against the old regex: `cargo test -p bashrs --lib -- PMAT439 sc2075` gave 11
  passed and 7 failed. At `0bdc20364d` it gave 18 passed.
- Reverting the rule body to the regex with the tests kept gave 11 passed and 7 failed. Restoring
  it gave 18 passed.
- `b797e6b2eb`'s new rows against `3a1a4f72dc`'s rule: 16 passed and 2 failed. `'a\'b` was
  reported, and `$$'a\'b'` was silent. With the fix: 18 passed.
- Control 1 changes `is_word_byte` to accept digits and underscore only. It fails 6 tests:
  `true_positive_still_fires`, `escaped_quote`, `its`, `multiple`, `in_command_sub` and the
  lexer_context GH-439 test.
- Control 2 records an attempt only when the close is followed by whitespace or the end of the
  line. It fails 1 test: `in_command_sub`.

## Verification

verification:
  cmd=cargo test -p bashrs --lib -- PMAT439 sc2075  claimed_exit=0  rerun_exit=0  log_path=docs/audits/impl-PMAT-439-receipt.md  sha256=0   # 18 passed; 0 failed
  cmd=cargo nextest run --workspace  claimed_exit=0  rerun_exit=0  log_path=docs/audits/impl-PMAT-439-receipt.md  sha256=0   # at b797e6b2eb: 17347 passed; 0 failed; 163 skipped
  cmd=cargo clippy --workspace --all-targets -- -D warnings  claimed_exit=0  rerun_exit=0  log_path=docs/audits/impl-PMAT-439-receipt.md  sha256=0   # at b797e6b2eb
  cmd=cargo fmt --all -- --check  claimed_exit=0  rerun_exit=0  log_path=docs/audits/impl-PMAT-439-receipt.md  sha256=0
  cmd=pv lint contracts  claimed_exit=0  rerun_exit=0  log_path=docs/audits/impl-PMAT-439-receipt.md  sha256=0   # Result: PASS
  cmd=bashrs corpus run  claimed_exit=0  rerun_exit=0  log_path=docs/audits/impl-PMAT-439-receipt.md  sha256=0   # 18814 entries; 18814 passed; 0 failed; V2 99.4/100 (A+): A 100.0, B1 99.9, B2 99.9, B3 98.7, D 100.0, F 99.6, G 99.7
  cmd=bashrs lint <the issue's script>  claimed_exit=0  rerun_exit=0  log_path=docs/audits/impl-PMAT-439-receipt.md  sha256=0   # and exit 2 (SC2075) on echo 'it\'s'

The corpus ran the way `make corpus-score` runs it: under bwrap, from an empty temporary directory,
using this branch's binary from `CARGO_TARGET_DIR`. It did not use `$(PWD)/target`, which is a
different build. The first run scored 88.7 (B), with B3 and G at 9.8 percent and no entry failed.
`TMPDIR` pointed at a directory that `--ro-bind / /` made read-only, so the behavioural runs had
nowhere to write. Bound writable, the same binary gives the line above, which matches the 7.4.x
release figures. Lint clean misses 3 entries, B-17300, M-397 and M-443. Linting their output
reports SC2135, SC2168 and SC2148, and no SC2075.

## Review

Two blind rounds, each with three lanes: sonnet, sonnet adversarial and haiku. These are
**degraded: same-family**: the author is Opus 5.5, and no lane used the author's model. Each lane got
a snapshot of the base files and two candidate diffs, A and B, with `index` lines removed. One was
the real diff and one a control. Lane r2 saw them in the opposite order. Verdicts and findings are in
`docs/audits/quorum-PMAT-439-review.json`.

- **Round 1, on `3a1a4f72dc`'s diff.** Real diff PASS 3/3; control 1 caught 3/3. The lanes found two
  regressions against the old regex, `echo 'a\'b` reported and `$$'a\'b'` silent. Both were checked
  in bash and fixed in `b797e6b2eb`.
- **Round 2, on `b797e6b2eb`'s diff, which is the judged diff.** `git diff 63abc48ec4 b797e6b2eb` has
  sha256 `671890d15c791bd704ce630d3e01113f042769b55316b0fa4b86c507694ced3c`. Real diff PASS 3/3;
  control 2 caught 3/3. Every lane traced control 2 to `test_sc2075_in_command_sub`, which is the one
  test cargo fails for it.

Two hooks refused commands during the work.

- A pre-edit hook refused the first edit because no active ticket was recorded. The ticket id was
  recorded as the hook instructs, and then the edit was made.
- A deletion guard refused two shell commands, one with a variable path and one that only quoted the
  guard. Neither command ran, and both were reissued without any deletion.

The post-commit hook restages `.pmat/baseline.json` after each commit. That churn was restored each
time and is not part of the branch, matching #427, #430 and #433.

## What this receipt does NOT claim

- **It does not claim the rule parses shell.** It scans one line at a time, as the regex did. A
  `'...'` or `"..."` that spans lines, or a heredoc body, is scanned line by line.
- **It does not claim every escape attempt is SC2075.** A `\'` followed by a space or punctuation, as
  in `'can\' t'`, `'say \'.'` or `'it\'-s'`, is no longer reported by SC2075. SC1078 reports the
  quote it leaves open, and that is pinned for the space form only.
- **It does not clear two false positives the regex shares.** `echo 'a\'b 'c'` and `echo 'a\'b'c'`
  are valid shell and both still report SC2075. In each, a later opening quote is read as the
  closer. Checking that the closer ends a word would clear them. That is not done here, and it is no
  regression.
- **It does not claim the `$( )` context is exact.** A `)` inside `$( )`, such as a case pattern,
  ends the context early. On the tested lines that changes no quote decision.
- **It does not read the corpus score as evidence for the fix.** The corpus lints transpiler output,
  which seldom contains these shapes. The score shows only that nothing regressed.
