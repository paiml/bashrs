# PMAT-259 receipt — the convergence-log path

## What and why

`test_cov_corpus_converged_no_log` asserts that `corpus_converged` errors when there is no convergence log. The function read `.quality/convergence.log` from the process working directory, so the assertion held only in a checkout that happened not to carry that file. This one carries it, dated 2026-02-09, and the test failed in the v7.3.0 release gate.

`corpus_converged_with_log` takes the path; `corpus_converged` passes the default. No caller changes and no behaviour changes. The test points at a `tempfile::TempDir`.

## Why it is its own ticket

It was found while gating PMAT-258 and could have ridden along, but a quorum reviews a diff against its ticket's stated intent: three lanes refused the follow-up precisely because its one commit does not implement the release ticket they were shown. One ticket per diff is the rule that makes the review meaningful.

## Verification

| check | result |
|---|---|
| `cargo test -p bashrs --lib` | 15,672 passed, 0 failed |
| the test under change | passes, and now fails for the right reason if the split is reverted |
| pre-commit gates | format, complexity, clippy stage, SATD all green |

## Related

The corpus gate's regression check had the same shape earlier in this release and was fixed the same way. Both were found by the full release gate rather than the PR gate, which is the division of labour the two gates exist for.
