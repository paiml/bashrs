# PMAT-258 receipt — v7.3.0

## Identity

| field | value |
|---|---|
| ticket | PMAT-258, kind=code (kind-gate exit 0), model-gate `model=opus class=opus decision=admit basis=file` |
| branch | PMAT-258-v7.3.0, HEAD dc4a960750 |
| discover.json sha256 | 7f83096d273cbde1… |
| gate_cmd | `cargo test --workspace`, **gate_cmd_fallback=true** |
| required_check | `gate` |

## Scope, and why the corpus is in it

The ticket's title names the corpus, and the operator's standing instruction for every release is to grow it with agy "especially patterns we use in work like crontabs, makefiles, perl/bash/python mixed with bash". Quorum round 1 lane 2 read the 139 new entries as out of scope because this receipt did not exist to say so; it does now. The entries are not decoration: they exercise the lowerings this release adds, and each was transpiled against a binary built from this branch before being kept.

## Decisions taken by quorum, not by the orchestrator

Three lanes, blind, unanimous on each (artifacts under `/run/user/1000/paiml-implement/agy/PMAT-258/<session>/decide/`):

| id | decision | vote |
|---|---|---|
| D1 | rename the ps-grep rule to SC2009 and implement the real SC2106 | 3-0 |
| D2 | lower `unwrap_or` as the unset-only `${var-d}` | 3-0 |
| D3 | where bwrap is absent, proceed with the scoped run and warn | 3-0 |

Two lanes returned an overall FAIL while choosing the same options, both saying D2's prompt omitted whether Rust's `unwrap_or` treats `Some("")` as present. That is a fair criticism of the prompt; the premise is true and does not change the choice.

## Verification: claimed against my own rerun

| check | result |
|---|---|
| `pv lint contracts` gate 4 | **80 refs, 80 found, 0 missing**; pv-gate GREEN |
| coverage | **95.01 percent** lines, `--fail-under-lines 95` exit 0 |
| `cargo test -p bashrs --lib` | 15,671 passed, 0 failed |
| `cargo nextest run -p bashrs --lib` | 15,672 passed in 64s |
| `make pr-gate` | green, 149s |
| corpus | 18,453 → 18,592 entries; contract tests 11 passed |
| book | builds, examples pass |
| commit trailers | 9 of 9 carry `Pmat-Ticket: PMAT-258` |

## Quorum review, round 1: what it caught

Three lanes, three different models, none in the author's family (`gemini-3.1-pro-high`, `gpt-oss-120b-medium`, `gemini-3.8-flash-high`). Verdicts 1 PASS, 2 FAIL. Every cited location was re-read here.

- **Lane 3, confirmed and fixed.** `Sandbox::new()` took the presence of bwrap for usability. A host can ship bwrap and still refuse the namespaces it needs, and bwrap then exits 1 before running anything; every caller reads "exit code is not 124" as success, so an unusable bwrap would have passed every corpus entry WITHOUT RUNNING — the vacuous-green class of #284. The sandbox now probes bwrap with `true` through the real argument set and falls back when that fails. The lane's environment was itself such a host, which is why it saw the sandbox tests fail where they pass here.
- **Lane 3, not reproduced.** The three sandbox tests pass here (3 passed), `make pr-gate` is green, and CB-2113 traceability holds: 9 of 9 commits carry the trailer. Those findings are artefacts of the lane's own sandbox.
- **Lane 2, acted on.** The corpus growth had no receipt to justify it. This document is that justification.

## Jidoka

1. The renamed SC2009 was missing from the linter's apply list, so it fired under no code at all. Caught by a test that runs the full lint pipeline rather than the rule function.
2. The new `len()` lowering emitted `${#var}` for any variable. bashrs flattens arrays to scalars, so an array receiver would have reported the joined text's length as an element count — the wrong-value class again. Narrowed to receivers the converter does not know to be arrays; the array count path owns the rest.
3. Four tests pinned the older erroring contract, one of them from v7.2.0 whose premise the runtime expansion supersedes. Re-specified rather than deleted.

## Gaps

| gap | closing artifact |
|---|---|
| PMAT-253 remaining forjar-parity phases | still open on v7.3.0 |
| `push_str`, `push`, `insert`, `rev` | still fail the transpile naming the method; no POSIX spelling that is both honest and deterministic |
| #236, #234, #233 | backlog |
| release-lint treats cancelled entries as open | filed upstream, paiml/paiml-mcp-agent-toolkit#1326 |

## Quorum review, rounds 2 and 3

- **Round 2** (same three models): lane 3's objection was gone once the bwrap probe landed, lane 1 passed, and lane 2 returned NO-VERDICT, which is not a refusal. Its raw output says verdict PASS, but the model emitted two JSON objects inside a markdown fence, so agy marked the envelope status ERROR. A malformed envelope is not a vote, and the artifact was not edited to make it one.
- **Round 3** (gemini-3.1-pro-high, gemini-3.7-flash-high, gemini-3.8-flash-high): 2 PASS, 1 FAIL, and the FAIL was right. Contract F-LCX-020 named the module `tests_pmat258` in sc2106.rs, but that test sits in the file's existing `tests` module. pv gate 4 still passed, because it matches only the text after the last colon pair against bare test names, so the gate was green while the command written in the contract found nothing. Corrected, and every other test path in contracts/ was re-run individually to confirm it resolves to at least one test.

That is the second time this release that a green gate hid a wrong claim, and both were caught by a reviewer rather than by the gate.
