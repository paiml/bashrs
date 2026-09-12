# Release notes

One section per release. `release-lint` rule R5 reads the headings: every tag that has been cut keeps its section, and every version open work names has one before the tag is cut.

Sections for releases before v7.1.0 carry the tag's own annotated message, which is the record that exists for them. From v7.1.0 the full notes live in `CHANGELOG.md` and the section here points at it.

## v7.4.0

In progress; the release ticket is PMAT-263, the close-out PMAT-264.

The required check lints the whole workspace (#233): `ci / lint` and `make release-gate` both ran a clippy scoped to the root stub package, and now name the workspace. The self-lint is gate S, with a per-file ratchet (`make dogfood-selflint`, PMAT-253 decision D4). A nightly workflow runs `cargo test --workspace`, the targets the required check skips. 203 corpus entries (18,592 to 18,795) and a corpus release bar that ratchets to the last shipped count. Two transpiler defects the new entries exposed are filed as PMAT-265.

Full notes: the `[7.4.0]` section of `CHANGELOG.md`.

## v7.3.0

Tagged 2026-09-12.

Three deferred decisions taken by blind quorum and implemented. **Breaking:** the rule bashrs called `SC2106` was shellcheck's `SC2009` and has moved there; `SC2106` now reports a `break` or `continue` inside a subshell. Change `# shellcheck disable=SC2106` to `SC2009` if you suppressed the ps-grep check.

Added: `len()` on a string lowers to `${#var}`, `to_string()` is the value, and `unwrap_or` lowers to the unset-only default `${var-d}` so a set-but-empty value survives. Methods with no honest POSIX spelling still fail the transpile naming the method.

Fixed: the corpus runner sandboxes itself, so a corpus run cannot write into the repository or the real home directory.

Full notes: the `[7.3.0]` section of `CHANGELOG.md`.

## v7.2.0

Tagged 2026-09-11.

v7.2.0 — ten dogfooding fixes, pv from theatre to gate, coverage at 95, a Pareto PR gate

Full notes: the `[7.2.0]` section of `CHANGELOG.md`.

## v7.1.0

Tagged 2026-09-11.

v7.1.0 - transpiler and directive fixes, DET005, dogfood gates, publish from a tag, +225 corpus entries

Full notes: the `[7.1.0]` section of `CHANGELOG.md`.

## v7.0.4

Tagged 2026-09-10.

v7.0.4 - false-positive fixes, dependency refresh, corpus growth

Full notes: the `[7.0.4]` section of `CHANGELOG.md`.

## v7.0.3

Tagged 2026-09-10.

v7.0.3 - six lexer-context false positives fixed; SC1012 takes shellcheck's meaning

Full notes: the `[7.0.3]` section of `CHANGELOG.md`.

## v7.0.2

Tagged 2026-09-09.

v7.0.2 - the corpus is back: 17,942 entries, measured 84.4/100 (B)

Full notes: the `[7.0.2]` section of `CHANGELOG.md`.

## v7.0.1

Tagged 2026-08-30.

bashrs 7.0.1 — zero false positives on the corpus, with every SEC finding intact

Full notes: the `[7.0.1]` section of `CHANGELOG.md`.

## v7.0.0

Tagged 2026-08-30.

bashrs 7.0.0 — SCxxxx is ShellCheck's namespace, and five lexer false positives are gone

Full notes: the `[7.0.0]` section of `CHANGELOG.md`.

## v6.68.0

Tagged 2026-08-21.

bashrs 6.68.0

## v6.67.0

Tagged 2026-08-18.

v6.67.0         v6.67.0 — the linter stops pattern-matching text

## v6.66.3

Tagged 2026-08-12.

bashrs 6.66.3

## v6.66.1

Tagged 2026-04-27.

v6.66.1 - CI fixes for binary-release multi-target builds

## v6.66.0

Tagged 2026-04-08.

v6.66.0 — Quality & Coverage Release

## v6.65.0

Tagged 2026-02-27.

v6.65.0 - Coverage milestone: 95.04% line coverage

## v6.64.0

Tagged 2026-02-15.

v6.64.0 — COMPLY system, gradual types, 30+ parser fixes, 99.2/100 A+

## v6.63.0

Tagged 2026-02-13.

v6.63.0 — corpus expansion, transpiler improvements, linter hardening

## v6.62.0

Tagged 2026-02-10.

v6.62.0 — Variable shadowing, 15,106 corpus entries

## v6.61.0

Tagged 2026-02-10.

v6.61.0 — 2 P0 transpiler fixes, 14,712 corpus entries

## v6.60.0

Tagged 2026-02-07.

v6.60.0 - Corpus Expansion & CLI Test Coverage

## v6.57.0

Tagged 2026-01-20.

docs(book): add DSL built-in functions reference

## v6.56.0

Tagged 2026-01-18.

v6.56.0 - Issue Triage Release

## v6.55.0

Tagged 2026-01-18.

Release v6.55.0 - Fix SC2128, SC2031, SC2154 false positives (Issue #132)

## v6.52.0

Tagged 2026-01-12.

release: v6.52.0 - Linter Enhancements & Dependency Updates

## v6.50.0

Tagged 2026-01-06.

v6.50.0 - Logic Extraction for EXTREME TDD

## v6.49.0

Tagged 2026-01-04.

v6.49.0 - 95% Test Coverage Achieved

## v6.48.0

Tagged 2025-12-30.

v6.48.0 - Test Coverage Improvements

## v6.47.0

Tagged 2025-12-29.

v6.47.0 - Lint fixes and test stabilization

## v6.46.0

Tagged 2025-12-21.

v6.46.0 - Probar Integration + Examples

## v6.45.0

Tagged 2025-12-21.

v6.45.0 - FP018 stderr redirect fix

## v6.44.0

Tagged 2025-12-16.

v6.44.0 - Complete Parser Bug Fixes

## v6.43.0

Tagged 2025-12-15.

v6.43.0 - TUI Module & WASM Cleanup

## v6.42.0

Tagged 2025-12-07.

v6.42.0 - ML-Powered Quality Gates

## v6.41.0

Tagged 2025-11-27.

v6.41.0 - bashrs-oracle Integration

## v6.40.0

Tagged 2025-11-27.

v6.40.0 - Fast Coverage Tests

## v6.39.0

Tagged 2025-11-25.

v6.39.0 - Verificar Integration & Parser Improvements

## v6.36.1

Tagged 2025-11-24.

v6.36.1 - Dependency Optimization

## v6.36.0

Tagged 2025-11-23.

v6.36.0 - Quality fixes and linter enhancements

## v6.35.0

Tagged 2025-11-15.

v6.35.0 - Complete bashrs Book with 100+ Examples

## v6.34.0

Tagged 2025-11-12.

v6.34.0 - Issue #1: Fix auto-fix invalid syntax bug

## v6.33.0

Tagged 2025-11-07.

v6.33.0 - Critical Auto-fix Bug Fix

## v6.32.1

Tagged 2025-11-07.

v6.32.1 - Fix SC2154 false positives for loop variables

## v6.32.0

Tagged 2025-11-07.

v6.32.0 - Dockerfile Linting & MAKE010 False Positive Fix

## v6.31.2

Tagged 2025-11-06.

v6.31.2 - Clippy Fixes and Documentation Streamlining

## v6.31.0

Tagged 2025-11-04.

v6.31.0 - Major Documentation Release + SEC Batch Testing

## v6.30.1

Tagged 2025-11-03.

v6.30.1 - Critical Parser Bug Fix

## v6.29.0

Tagged 2025-11-03.

v6.29.0 - Rule Registry 100% Complete

## v6.28.0-dev-snapshot

Tagged 2025-11-02.

v6.28.0-dev Development Snapshot - Shell-Specific Filtering Foundation

## v6.28.0

Tagged 2025-11-03.

v6.28.0 - Memory Measurement for Benchmarking

## v6.27.1

Tagged 2025-11-02.

v6.27.1 - Complete Linter Integration for Zsh

## v6.27.0

Tagged 2025-11-02.

v6.27.0 - Shell Type Detection for Zsh Compatibility

## v6.26.0

Tagged 2025-11-02.

v6.26.0 - Memory Measurement for Benchmarking

## v6.25.0

Tagged 2025-11-01.

v6.25.0 - Scientific Benchmarking Command

## v6.24.3

Tagged 2025-11-01.

v6.24.3 - Code Complexity Reduction via EXTREME TDD

## v6.24.2

Tagged 2025-10-31.

v6.24.2 - 7 SC Linter Ignored Test Fixes

## v6.24.1

Tagged 2025-10-31.

v6.24.1 - Linter Rule Bug Fixes

## v6.24.0

Tagged 2025-10-31.

v6.24.0 - ZERO Clippy Warnings (100% Clean)

## v6.23.0

Tagged 2025-10-31.

v6.23.0 - REPL DevEx Improvements & Quality Validation

## v6.22.0

Tagged 2025-10-30.

v6.22.0 - REPL Debugging Enhancements

## v6.21.0

Tagged 2025-10-30.

v6.21.0 - REPL Purification & Explanation

## v6.20.0

Tagged 2025-10-29.

v6.20.0 - REPL File Completion and Multi-line Input

## v6.19.0

Tagged 2025-10-29.

v6.19.0 - Interactive REPL Enhancements

## v6.18.1

Tagged 2025-10-29.

v6.18.1 - Code Quality Cleanup

## v6.18.0

Tagged 2025-10-29.

v6.18.0 - File Type-Aware Quality Scoring

## v6.17.1

Tagged 2025-10-29.

v6.17.1 - Critical Fix: Empty Function Builtin Shadowing

## v6.17.0

Tagged 2025-10-29.

v6.17.0 - Formatter Complete (15/15 tests passing)

## v6.16.2

Tagged 2025-10-29.

v6.16.2 - Function shorthand syntax + formatter improvements

## v6.16.1

Tagged 2025-10-29.

v6.16.1 - Complete test expression string equality support

## v6.16.0

Tagged 2025-10-29.

v6.16.0 - Parser improvements for test expressions

## v6.15.0

Tagged 2025-10-28.

v6.15.0 - Formatter Status Clarification

## v6.14.0

Tagged 2025-10-28.

v6.14.0 - Bash Script Formatting (INITIAL Release)

## v6.13.0

Tagged 2025-10-28.

v6.13.0 - Coverage Tracking (Bash Quality Tools)

## v6.12.0

Tagged 2025-10-28.

v6.12.0 - Comprehensive Quality Audit

## v6.11.0

Tagged 2025-10-28.

v6.11.0 - Bash Quality Scoring

## v6.10.0

Tagged 2025-10-28.

v6.10.0 - Bash Quality Tools MVP

## v6.9.0

Tagged 2025-10-28.

v6.9.0 - A+ Grade Quality Achievement

## v6.8.0

Tagged 2025-10-28.

v6.8.0 - A Grade Quality Achievement

## v6.7.0

Tagged 2025-10-28.

v6.7.0 - REPL Interactive Features

## v6.6.0

Tagged 2025-10-27.

v6.6.0 - REPL State + Sprint 32 Makefile Assessment

## v6.5.0

Tagged 2025-10-26.

v6.5.0 - Hybrid Workflow Documentation Complete

## v6.4.0

Tagged 2025-10-26.

v6.4.0 - REPL Foundation Release

## v6.3.0

Tagged 2025-10-26.

v6.3.0 - Production Quality Release with Critical Bug Fixes

## v6.2.0

Tagged 2025-10-22.

v6.2.0 - Major Documentation Release

## v6.1.0

Tagged 2025-10-22.

v6.1.0 - Shell Configuration Management

## v6.0.0

Tagged 2025-10-22.

v6.0.0 - 🏆 100% ShellCheck Coverage MILESTONE! 🏆

## v5.0.0

Tagged 2025-10-22.

v5.0.0 - 80% ShellCheck Coverage Milestone

## v4.8.0

Tagged 2025-10-21.

v4.8.0 - Sprint 114: 70% Milestone (Array Safety & Test Expressions)

## v4.3.0

Tagged 2025-10-21.

v4.3.0 - 50% ShellCheck Coverage MILESTONE! 🎉

## v4.2.0

Tagged 2025-10-21.

v4.2.0 - Sprints 106-107: 10 New Linter Rules

## v4.1.0

Tagged 2025-10-21.

v4.1.0 - Sprint 100 Milestone: Grep/Trap Safety

## v4.0.0

Tagged 2025-10-21.

v4.0.0 - Major Release: 84 Total Rules

## v3.1.0

Tagged 2025-10-20.

v3.1.0 - ShellCheck Phase 2: 15 New Linter Rules

## v3.0.0

Tagged 2025-10-20.

v3.0.0 - Phase 1 Complete: Makefile World-Class

## v2.1.1

Tagged 2025-10-19.

v2.1.1 - Sprint 80 FAST Validation + Bug Fixes

## v2.1.0

Tagged 2025-10-19.

v2.1.0 - Fix Safety Taxonomy

## v2.0.1

Tagged 2025-10-19.

v2.0.1 - Critical Auto-Fix Bug Fix (Issue #1)

## v2.0.0

Tagged 2025-10-19.

Release v2.0.0 - Makefile Linting + Book Accuracy + CLI Integration

## v1.4.0

Tagged 2025-10-18.

Release v1.4.0: CLI Integration for Makefile Purification (Sprint 69)

## v1.3.0

Tagged 2025-10-14.

v1.3.0: Mutation Testing Excellence - 100% kill rate on is_string_value

## v1.2.1

Tagged 2025-10-11.

fix: Priority-based conflict resolution for auto-fix (v1.2.1)

## v1.2.0

Tagged 2025-10-11.

feat: Add auto-fix capability with --fix flag (v1.2.0)

## v1.1.0

Tagged 2025-10-10.

examples: Add practical Bash→Rust purification workflow demonstration

## v1.0.0-rc3

Tagged 2025-10-10.

Release v1.0.0-rc3: Automatic Test Generator

## v1.0.0-rc2

Tagged 2025-10-09.

v1.0.0-rc2: Mutation Testing Excellence - Sprint 25 Day 2

## v1.0.0-rc1

Tagged 2025-10-04.

Release v1.0.0-rc1: First release candidate

## v1.0.0

Tagged 2025-10-10.

Release v1.0.0 - Rash Stable Release

## v0.9.3

Tagged 2025-10-03.

Release v0.9.3: Expanded Standard Library

## v0.9.2

Tagged 2025-10-03.

v0.9.2 - Property Test Enhancement (Sprint 23)

## v0.9.1

Tagged 2025-10-03.

v0.9.1 - Mutation Testing Analysis (Sprint 24)

## v0.9.0

Tagged 2025-10-03.

v0.9.0 - Standard Library Release

## v0.8.0

Tagged 2025-10-03.

v0.8.0 - While Loops Release

## v0.7.0

Tagged 2025-10-03.

docs: Add mutation testing specification

## v0.6.0

Tagged 2025-10-03.

v0.6.0: Match expressions with POSIX case statements

## v0.5.0

Tagged 2025-10-02.

Release v0.5.0 - For Loops (Sprints 16-18)

## v0.4.0

Tagged 2025-10-02.

Release v0.4.0 - Production Ready (Sprints 1-11)

## v0.3.3

Tagged 2025-07-07.

feat: Add comprehensive crates.io metadata

## v0.3.2

Tagged 2025-07-07.

chore: bump version to 0.3.2 for crates.io release

## v0.3.1

Tagged 2025-06-05.

feat: Include compile and playground features by default in v0.3.1

## v0.3.0

Tagged 2025-06-05.

Release v0.3.0 - Binary Compilation, Interactive Playground & Formal Verification

## v0.2.1

Tagged 2025-06-05.

feat: Simplify RASH CI/CD workflow by removing edge case jobs

## v0.2.0

Tagged 2025-06-04.

Release v0.2.0: Major Technical Debt Reduction & Code Quality Improvements

## v0.1.0

Tagged 2025-06-04.

Release v0.1.0 - Initial release with multi-platform support
