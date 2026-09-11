# Decide six open items in bashrs (you are one of three blind lanes)

You have no shell and no file access. Everything you may use is in this message; do not claim to have run, read or measured anything else.

bashrs is a Rust shell linter, purifier and transpiler with one maintainer. Six items have been deferred for months because nobody would decide them. Your verdict IS the decision: there is no human tie-breaker, and a majority of the three lanes is binding. Decide each one.

For EVERY item id (T241 T242 T243 T246 R1 R2):
1. `questions`: one or two questions that actually decide it, `where` = the item id, `answered_by_the_text` true only when the facts below answer it.
2. `findings`: exactly one row per item. `file` = the item id. `claim` MUST begin with "DECIDE: <option>" where <option> is copied verbatim from that item's option list, then ". WHY: <at most 20 words>. COUNTER: <the strongest argument against your choice, at most 20 words>." `grounding` = "cited" when the facts support it, else "asserted". `fix` = "FLIPS IF: <the concrete fact that would reverse it>".
3. Choose exactly one option per item. Never invent an option, never straddle two.

`verdict`: "PASS" when every decision is supported by the facts; "FAIL" when you had to assert something material; "do-not-implement-as-written" only if an option as written would do harm. `summary`: at most 100 words on the two items you are least certain about.

## T241 — close or keep a five-month-old ticket
Roadmap entry PMAT-241, "REWRITE-5: Rewrite 5 remaining test files + add provable contracts (93 tests)". Opened 2026-04-07, status `blocked`, label `release:unscheduled`, no acceptance criteria, assigned to the maintainer. Its triage note says it is blocked on definition and that re-scoping or cancelling is the operator's call. Commit 4cf75e5fc9, dated the same day, is titled "fix: rewrite 5 remaining test files + add gates-quality-v1 contract (PMAT-241)" and records: all five named test files rewritten, one new provable contract with 4 falsification tests and 1 Kani harness, 14,300 tests passing, line coverage 90.27%. No later commit names PMAT-241.
Options: close-completed | keep-open-rescope | cancel

## T242 — close or keep a five-month-old ticket
Roadmap entry PMAT-242, "DEEP-CONTRACTS: Enrich 8 bashrs contracts to Grade A — spec depth + bindings + Kani". Opened 2026-04-07, status `blocked`, priority critical, label `release:unscheduled`, no acceptance criteria, same triage note as T241. Two commits dated 2026-04-07: 97f980a981 "feat: enrich 12 bashrs contracts to Grade A/B — deep provable coverage (PMAT-242)" and 16b23758d3 "feat: promote 3 more contracts to Grade A — full Kani harness coverage (PMAT-242)". A spec commit the same day records the resulting state: "12 bashrs contracts: 8×A + 4×B (mean 0.92)", up from "5×A + 7×B (mean 0.90)". Today `pmat comply` reports 103 of 103 work contracts carry falsifiable claims with evidence, and 0 of 103 participate in assume-guarantee chains.
Options: close-completed | keep-open-rescope | cancel

## T243 — an unmet coverage target with no named targets
Roadmap entry PMAT-243, "COV-95: Write tests for 8 uncovered production files — 90→95% coverage". Opened 2026-04-07, status `blocked`, priority critical, no acceptance criteria, and it names none of the eight files. The last recorded line coverage is 90.8%, from 2026-04-07. The repository's coverage cache is empty, so nothing has measured coverage since. The project's written quality standard is "Test coverage >95%", and its written rule is that all coverage work must use `pmat query --coverage-gaps`, never raw coverage output. Three releases have shipped since April, adding thousands of tests, none of them measured for coverage.
Options: redefine-measurement | cancel | keep-as-is

## T246 — 89 stale generated contracts
bashrs commits 89 `contracts/work/*.yaml` files that its contract tool generates from local work data. They were generated on 2026-04-08 by a version of the generator that predates a schema fix made eleven days later. Measured today: the contract verifier `pv` rejects all 89 of 89. Regenerating with the current generator produces 103 files, of which `pv` rejects 0, and the verifier's lint reports PASS. But the generator's own compliance check counts generated files as unclassified: the reading moves from "89 of 104 unclassified" before regeneration to "103 of 108 unclassified" after it. An upstream issue asking the check to recognise these files was filed today; it is unresolved. The files are committed, so any contract gate over the directory is red at baseline until they are regenerated.
Options: regenerate-and-commit-now | wait-for-upstream | stop-tracking-generated-stubs

## R1 — rewrite a lint rule on top of a parser
Rule SC2105 reports `break` or `continue` used outside a loop. It is implemented by scanning each physical line with three regexes, one for loop-opening keywords, one for `done`, one for `break`/`continue`, and replaying those events in byte order so same-line constructs are handled. A competitive review lane recommended replacing this with AST-based loop containment, so the rule would know exactly which loop a `break` sits in; the other two lanes let the current design stand. Measured since: a fix made the one-line loop, same-line function and trailing-comment forms agree with shellcheck. The one remaining divergence, `break` inside a subshell in a loop, is a different rule, SC2106, which bashrs does not implement and which is filed as an open issue. SC2105 is on the corpus scoring exclusion list, alongside ten other rules, because it false-positives on valid transpiler output. Five lint defects opened since the last release concern other rules, none of them loop containment.
Options: adopt-ast-now | defer-to-sc2106 | reject

## R2 — replace a release script with release tooling
Releases are published by a project script: it checks the tag out into a clean detached worktree, publishes the workspace's oracle crate first, waits for the crate to appear on the sparse index, then publishes the main crate, and it supports a dry run. That order exists because the previous release raced the index and failed. The workspace publishes two crates and has one maintainer. A review lane recommended cargo-release, which automates version bumping, tagging and publishing in dependency order. Another lane weighed release-plz, which additionally generates changelogs from commit messages and opens a release pull request; that lane let the current design stand. The project's release protocol requires the changelog to state measured numbers, including a corpus run of at least 17,942 entries and its score, and requires the release notes to state what was measured rather than what was assumed.
Options: keep-script | cargo-release-for-version-bump | adopt-release-plz
