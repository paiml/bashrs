# PMAT-261 receipt — coverage back above the gate

## Why

The v7.3.0 release gate measured **94.98 percent** line coverage, under the repository's 95 percent gate. PMAT-259 and PMAT-260 had added lines that no test reached: the `corpus_converged_with_log` split, and the diff and report twins beside it.

## What

Fifteen tests across two files, all through the `_with` twins so none of them reads the repository's own `.quality/convergence.log` or runs the full corpus:

- the convergence check from both sides — missing log, too few iterations, all checks passing, rate under the bar, delta over it, a regression between iterations — plus the two pure helpers beside it;
- the diff handler in human and json form, with an explicit iteration pair and the unknown-iteration error;
- the report handler written to a path and to stdout, and the date helper.

Every fixture is a log written into a `TempDir` and a one-entry registry.

## Verification

| check | before | after |
|---|---|---|
| line coverage | 94.98% | **95.06%** |
| `cargo llvm-cov --lib -p bashrs --fail-under-lines 95` | fails | exit 0 |
| function coverage | 95.58% | 95.61% |
| new tests | — | 15, running in 0.59s |
