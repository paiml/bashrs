# PMAT-243: coverage measured, and the gaps named

The ticket asked for 95 percent line coverage and named none of the files it meant. It had rested on an April figure nothing had re-measured since. This is the measurement.

## What was measured

`cargo llvm-cov --lib -p bashrs --summary-only`, on the v7.2.0 branch:

| metric | covered | total | percent |
|---|---|---|---|
| lines | 177,015 | 192,995 | **91.72%** |
| functions | 19,280 | 20,535 | **93.89%** |
| regions | 268,353 | 293,648 | **91.39%** |

The standing figure was 90.8 percent, measured 2026-04-07. Three releases and several thousand tests later, line coverage has moved less than a point.

## The gaps, from `pmat query --coverage-gaps --limit 10 --exclude-tests`

All ten are in one crate:

| function | file |
|---|---|
| `lint_shell_wasm`, `lint_makefile_wasm`, `lint_dockerfile_wasm` | `bashrs-wasm/src/lib.rs` |
| `classify_shell_wasm`, `explain_shell_wasm`, `bashrs_version`, `Finding` | `bashrs-wasm/src/lib.rs` |
| `load_codebert_model`, `load_codebert_probe`, `classify_codebert_wasm` | `bashrs-wasm/src/lib.rs` |

Each reports 0.0 percent. They are `wasm_bindgen` entry points: the library test suite never calls them, and the measurement above does not compile that crate at all. The tool also excludes 71 functions across 11 files through the Makefile's `COVERAGE_EXCLUDE`, and 4 dead-code functions.

## What this means for the 95 percent standard

The standard in CLAUDE.md says coverage above 95 percent, without naming a crate. The measured command covers `bashrs` only, and the largest named gaps are in `bashrs-wasm`, which it does not measure. So the standard and the measurement disagree about their subject before they disagree about their number.

Two things follow, and both are decisions rather than work:

1. The gap between 91.72 and 95 percent is 3.28 points of `bashrs` lines, about 6,300 lines. That is a real body of work, not a rounding error, and it should be scheduled as such or the standard re-dated.
2. The WASM entry points need a harness that calls them, or an explicit exclusion. Counting them as uncovered while never running them makes the number worse without telling anyone anything.

## Reproducing

```
cargo llvm-cov --lib -p bashrs --summary-only
pmat query --coverage-gaps --limit 10 --exclude-tests
```

Never `cargo tarpaulin`, and never a raw coverage dump in place of `pmat query --coverage-gaps`: both are repository rules.
