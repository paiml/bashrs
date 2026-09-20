# PMAT-355 — receipt: SC2242 reports an English sentence as an error

**Ticket:** PMAT-355 (bashrs#355) — SC2242 counts `case` depth in RAW source. **Kind:** code.
**Branch:** `PMAT-355-sc2242-quote-sensitive`, which also carries the v7.4.2 cut (PMAT-354).
**Why the two share a PR:** `make release-gate` on the cut went RED on this defect, so v7.4.2 could
not be cut without the fix. That is stated in PMAT-354's receipt and in PMAT-354's acceptance
criterion, both widened from the diff after a three-engine quorum refused the PR for carrying work
the paperwork did not name.

## The defect

bashrs#332 made SC2242's `in_case`/`in_loop`/`in_function` depth COUNTERS rather than flags, which is
right for code. It made the literal case worse: the rule reads raw source, so

    echo "could not break the matcher — this case discriminates nothing"

opens a case depth on the word `case` and then finds `break` inside `break the matcher`. A sentence
is reported as a break outside a case. The rule's own comment already said a keyword inside a quoted
string "needs the parser"; `QUOTE_SENSITIVE_RULES` is the parser-free answer the module had all along
— a rule listed there receives the copy `mask_literals` has blanked.

## The second defect, which is the one worth reading

Adding `"SC2242"` to `QUOTE_SENSITIVE_RULES` turned bashrs's own guard red:

    test_GH226_quoting_allowlist_names_only_rules_that_exist
    SC2242 is allowlisted but names no rule module

`QUOTE_SENSITIVE_RULES` is a list of CODES; `allowlisted_checks()` is a list of `(code, check_fn)`
PAIRS, and the property test that runs every allowlisted rule over a pure literal iterates the
second. The rule was therefore declared quote-sensitive while **nothing ever ran it over a literal**.

That is bashrs#266's root cause exactly — two hand-maintained lists with nothing tying them together
— and this time the guard caught it, which is what the guard is for. The fix wires
`("SC2242", sc2242::check)` into the pair list, so the property test covers the rule, and #332's own
sentence is now one of its sources.

**A row asserting SC2242 is SILENT on that sentence is satisfied just as well by a rule that fires on
nothing**, so it is not evidence on its own. `test_PMAT355_sc2242_fires_on_the_unmasked_sentence` is
its negative control: the rule MUST report the raw sentence and MUST be silent once `mask_literals`
has run. Only with both does the silence mean the masking works rather than the rule being inert.

## Where each claim lives

| Claim | Where |
|---|---|
| SC2242 receives the masked copy | `rash/src/linter/quoting.rs`, `QUOTE_SENSITIVE_RULES` |
| the property test actually runs it | same file, `allowlisted_checks()` — the `(code, check_fn)` pair |
| #332's sentence is a fixture | same file, `test_GH226_quoting_allowlisted_rules_find_nothing_in_a_pure_literal` |
| the negative control | same file, `test_PMAT355_sc2242_fires_on_the_unmasked_sentence` |
| the bare `case`+`break` must STILL fire | `rash/tests/quoting_literal_payload_guard.rs` |

## Verification

verification:
  cmd=cargo test -p bashrs --lib -- linter::quoting::tests::test_GH226 linter::quoting::tests::test_PMAT355  claimed_exit=0  rerun_exit=0  log_path=docs/audits/impl-PMAT-355-receipt.md  sha256=0   # 34 passed; 0 failed
  cmd=cargo test -p bashrs --lib  claimed_exit=0  rerun_exit=0  log_path=docs/audits/impl-PMAT-355-receipt.md  sha256=0   # 15727 passed; 0 failed; 68 ignored
  cmd=cargo fmt --all -- --check  claimed_exit=0  rerun_exit=0  log_path=docs/audits/impl-PMAT-355-receipt.md  sha256=0
  cmd=cargo clippy --all-targets --all-features -- -D warnings  claimed_exit=0  rerun_exit=0  log_path=docs/audits/impl-PMAT-355-receipt.md  sha256=0

## What this receipt does NOT claim

- **It does not claim `release-lint` exits 0.** It exits 1, on four historical R6 tags that predate
  this repo's semver rule and that neither R3 nor R5 reads. PMAT-354's receipt carries the measured
  output; an earlier version of that line claimed `rerun_exit=0` and was refuted by a quorum lane
  re-running the command.
- **It does not claim the rule is now correct for every literal-vs-code case.** It claims two
  directions on one sentence and the allowlist property over four sources. `mask_literals` is a
  masking heuristic, not a shell parser, and `quoting_literal_payload_guard.rs` exists because the
  must-still-fire half has to be held separately.
- **It does not claim SC2242 is the last rule with this shape.** The guard that caught it is generic;
  what it cannot catch is a rule that belongs in `QUOTE_SENSITIVE_RULES` and is in neither list.
