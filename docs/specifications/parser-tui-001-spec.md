---
title: Parser Probar TUI Testing
issue: PARSER-TUI-001
status: Complete
created: 2025-12-16T17:17:45.428462627+00:00
updated: 2025-12-16T17:20:00.000000000+00:00
---

# Parser Probar TUI Testing Specification

**Ticket ID**: PARSER-TUI-001
**Status**: Complete

## Summary

Comprehensive probar-based TUI testing infrastructure for the bashrs parser. Uses jugar-probar for Playwright-style frame assertions, UX coverage tracking, snapshot testing, and determinism verification. Covers 71 parser features across 9 categories with automated coverage reporting.

## Requirements

### Functional Requirements
- [x] Frame-based parser output testing with `expect_frame()` assertions
- [x] UX coverage tracking with `gui_coverage!` macro
- [x] Snapshot testing for AST output stability
- [x] Frame sequence testing for parser state transitions
- [x] Soft assertions for error collection
- [x] Determinism verification (same input → same output)
- [x] Parser playbook YAML definition

### Non-Functional Requirements
- [x] Performance: All tests run in <1 second
- [x] Test coverage: 71 parser features tested
- [x] Element coverage: ≥70% per category
- [x] Overall coverage: ≥80%

## Architecture

### Design Overview

The parser probar testing uses a layered architecture:

1. **ParserResult Wrapper** - Unified result type for parser output
2. **Frame Generation** - TUI frames from parser results
3. **Coverage Tracking** - UX coverage via probar
4. **Assertions** - Playwright-style frame assertions
5. **Snapshots** - Golden file comparison

### API Design

```rust
/// Result from parsing - contains AST and any errors
struct ParserResult {
    ast: Option<BashAst>,
    errors: Vec<String>,
}

impl ParserResult {
    fn from_parse(input: &str) -> Self;
    fn is_ok(&self) -> bool;
}

/// Parser frame from output - simulates TUI output
fn parser_frame(input: &str, result: &ParserResult) -> TuiFrame;

/// Parse and track coverage
fn parse_with_coverage(
    input: &str,
    feature: &str,
    tracker: &mut UxCoverageTracker,
) -> ParserResult;
```

## Implementation Plan

### Phase 1: Foundation ✅
- [x] Add jugar-probar as dev dependency
- [x] Create ParserResult wrapper
- [x] Create parser_frame() utility

### Phase 2: Core Implementation ✅
- [x] Variable parsing tests (10 features)
- [x] Command substitution tests (5 features)
- [x] Conditional parsing tests (10 features)
- [x] Loop parsing tests (9 features)
- [x] Function parsing tests (6 features)
- [x] Quoting tests (6 features)
- [x] Arithmetic tests (10 features)
- [x] Redirection tests (8 features)
- [x] Pipeline tests (7 features)

### Phase 3: Advanced Testing ✅
- [x] Frame sequence testing
- [x] Snapshot golden file testing
- [x] Soft assertions for errors
- [x] Determinism verification
- [x] Comprehensive coverage report

## Testing Strategy

### Unit Tests
- [x] test_parser_variables_coverage (10 features)
- [x] test_parser_command_substitution_coverage (5 features)
- [x] test_parser_conditionals_coverage (10 features)
- [x] test_parser_loops_coverage (9 features)
- [x] test_parser_functions_coverage (6 features)
- [x] test_parser_quoting_coverage (6 features)
- [x] test_parser_arithmetic_coverage (10 features)
- [x] test_parser_redirection_coverage (8 features)
- [x] test_parser_pipeline_coverage (7 features)

### Integration Tests
- [x] test_parser_state_sequence (5 frame sequence)
- [x] test_parser_snapshot_golden_files (determinism)
- [x] test_parser_comprehensive_coverage (71 features)
- [x] test_parser_error_soft_assertions (error handling)
- [x] test_parser_determinism (7 inputs × 3 runs)

## Files Created

| File | Lines | Purpose |
|------|-------|---------|
| `rash/tests/parser_probar_testing.rs` | 785 | Main test file |
| `rash/playbooks/parser.yaml` | 200 | State machine spec |

## Coverage Results

| Category | Features | Coverage |
|----------|----------|----------|
| Variables | 10 | 100% |
| Command Sub | 5 | 100% |
| Conditionals | 10 | 100% |
| Loops | 9 | 100% |
| Functions | 6 | 100% |
| Quoting | 6 | 100% |
| Arithmetic | 10 | 100% |
| Redirection | 8 | 100% |
| Pipeline | 7 | 100% |
| **Total** | **71** | **100%** |

## Success Criteria

- ✅ All 14 probar tests pass
- ✅ All 7361 bashrs tests pass
- ✅ Coverage ≥80% achieved
- ✅ Zero clippy errors
- ✅ Playbook YAML created
- ✅ Determinism verified

## References

- [jugar-probar documentation](https://github.com/paiml/probar)
- aprender probar example (external project, not in this repository)
- [Parser playbook](../../rash/playbooks/parser.yaml)
