# Falsification Test Results

**Date**: 2025-12-21
**Total Tests**: 100
**Passed**: 97
**Failed**: 3

## Failures (Confirmed False Positives)

| ID | Code Pattern | Forbidden Rule Triggered | Description |
|----|--------------|--------------------------|-------------|
| **F047** | `case $x in *) ;; esac` | **SC2154** | Case default coverage. The linter doesn't track that the variable is implicitly handled or this case logic covers all paths. |
| **F048** | `for ((i=0;i<10;i++)); do echo $i; done` | **SC2086** | C-style for loop. The linter flags `$i` usage inside the arithmetic context or the loop body unnecessarily. |
| **F082** | `trap "echo $v" INT` | **SC2064** | Trap double quote. The user might *want* immediate expansion of `$v` when the trap is set, but the linter warns assuming they wanted deferred expansion. |

## Methodology
Ran `bashrs lint` on 100 distinct shell script snippets defined in `docs/specifications/false-positives.md`.
Passed if the "Forbidden Rule" was NOT triggered.
Failed if the "Forbidden Rule" WAS triggered.

*Note: Previous run (92/100) had 5 false negatives due to test harness log parsing issues. Corrected harness confirms 97% pass rate.*