# TICKET: REPL-012-002

## Title
Idempotency Report Formatting (Format Non-Idempotent Operation Suggestions)

## Priority
**P1 - High** (Second task in REPL-012 Idempotency Analyzer sprint)

## Status
🟢 **READY TO START** - Dependencies met (REPL-012-001 completed)

## Context
Building on REPL-012-001 (idempotency scanning), this task adds **human-friendly report formatting** for non-idempotent operation suggestions.

**Concept**: When scripts contain non-idempotent operations, format the suggestions in a clear, readable way.

**Why Report Formatting?**
- REPL-012-001 detects issues but returns raw `IdempotencyIssue` vectors
- Users need friendly formatting to understand what's wrong and how to fix it
- Good error messages help developers fix idempotency issues quickly

**Purpose**: Format idempotency scan results for human consumption.

## Dependencies
- ✅ REPL-012-001 (Idempotency scanner) completed
- ✅ `IdempotencyIssue` struct exists with line, code, explanation, suggestion fields
- ✅ `IdempotencyChecker` available with scan results

## Acceptance Criteria

### 1. Add `format_idempotency_report()` function

```rust
/// Format idempotency scan results for display
///
/// # Examples
///
/// ```
/// use bashrs::repl::determinism::{IdempotencyIssue, NonIdempotentOperation};
/// use bashrs::repl::format_idempotency_report;
///
/// let issues = vec![
///     IdempotencyIssue {
///         line: 5,
///         operation_type: NonIdempotentOperation::MkdirWithoutP,
///         code: "mkdir /tmp/foo".to_string(),
///         explanation: "mkdir without -p fails if directory already exists".to_string(),
///         suggestion: "Add -p flag: mkdir -p".to_string(),
///     },
/// ];
///
/// let formatted = format_idempotency_report(&issues);
/// assert!(formatted.contains("Line 5:"));
/// assert!(formatted.contains("mkdir /tmp/foo"));
/// assert!(formatted.contains("Add -p flag"));
/// ```
pub fn format_idempotency_report(issues: &[IdempotencyIssue]) -> String {
    if issues.is_empty() {
        return String::from("✓ Script is idempotent - safe to re-run");
    }

    let mut output = String::new();
    output.push_str("⚠️  Non-idempotent operations detected!\n\n");
    output.push_str(&format!("Found {} issue(s):\n\n", issues.len()));

    for issue in issues {
        output.push_str(&format!("Line {}:\n", issue.line));
        output.push_str(&format!("  Code:        {}\n", issue.code));
        output.push_str(&format!("  Problem:     {}\n", issue.explanation));
        output.push_str(&format!("  💡 Fix:      {}\n", issue.suggestion));
        output.push('\n');
    }

    output
}
```

### 2. Add display helper for `IdempotencyChecker`

```rust
impl IdempotencyChecker {
    /// Format scan results for display
    pub fn format_report(&self) -> String {
        let mut output = String::new();

        // Show idempotency status
        if self.is_idempotent() {
            output.push_str("✓ Script is idempotent\n");
            output.push_str("  Safe to run multiple times\n");
        } else {
            output.push_str("⚠️  Script is non-idempotent\n");
            output.push_str(&format!("  {} issue(s) found\n", self.detections.len()));
        }

        // Show issue breakdown by type
        let mkdir_count = self.count_by_operation(NonIdempotentOperation::MkdirWithoutP);
        let rm_count = self.count_by_operation(NonIdempotentOperation::RmWithoutF);
        let ln_count = self.count_by_operation(NonIdempotentOperation::LnWithoutF);

        if mkdir_count > 0 || rm_count > 0 || ln_count > 0 {
            output.push_str("\nOperation breakdown:\n");
            if mkdir_count > 0 {
                output.push_str(&format!("  mkdir (missing -p): {}\n", mkdir_count));
            }
            if rm_count > 0 {
                output.push_str(&format!("  rm (missing -f):    {}\n", rm_count));
            }
            if ln_count > 0 {
                output.push_str(&format!("  ln (missing -f):    {}\n", ln_count));
            }
        }

        // Show detailed issues
        if !self.detections.is_empty() {
            output.push('\n');
            output.push_str(&format_idempotency_report(&self.detections));
        }

        output
    }
}
```

### 3. Unit Tests (RED → GREEN → REFACTOR)

```rust
#[cfg(test)]]
mod tests {
    use super::*;

    // ===== REPL-012-002: REPORT FORMATTING TESTS =====

    #[test]
    fn test_REPL_012_002_format_single_mkdir_issue() {
        // ARRANGE: One mkdir issue
        let issues = vec![IdempotencyIssue {
            line: 10,
            operation_type: NonIdempotentOperation::MkdirWithoutP,
            code: "mkdir /tmp/test".to_string(),
            explanation: "mkdir without -p fails if directory already exists".to_string(),
            suggestion: "Add -p flag: mkdir -p".to_string(),
        }];

        // ACT: Format report
        let formatted = format_idempotency_report(&issues);

        // ASSERT: Should show line, code, problem, and fix
        assert!(formatted.contains("Line 10:"));
        assert!(formatted.contains("mkdir /tmp/test"));
        assert!(formatted.contains("mkdir without -p"));
        assert!(formatted.contains("Add -p flag"));
        assert!(formatted.contains("⚠️"));
    }

    #[test]
    fn test_REPL_012_002_format_single_rm_issue() {
        // ARRANGE: One rm issue
        let issues = vec![IdempotencyIssue {
            line: 15,
            operation_type: NonIdempotentOperation::RmWithoutF,
            code: "rm /tmp/file.txt".to_string(),
            explanation: "rm without -f fails if file doesn't exist".to_string(),
            suggestion: "Add -f flag: rm -f".to_string(),
        }];

        // ACT: Format report
        let formatted = format_idempotency_report(&issues);

        // ASSERT: Should show rm-specific details
        assert!(formatted.contains("Line 15:"));
        assert!(formatted.contains("rm /tmp/file.txt"));
        assert!(formatted.contains("rm without -f"));
        assert!(formatted.contains("Add -f flag"));
    }

    #[test]
    fn test_REPL_012_002_format_multiple_issues() {
        // ARRANGE: Multiple issues
        let issues = vec![
            IdempotencyIssue {
                line: 5,
                operation_type: NonIdempotentOperation::MkdirWithoutP,
                code: "mkdir foo".to_string(),
                explanation: "mkdir without -p fails if directory already exists".to_string(),
                suggestion: "Add -p flag: mkdir -p".to_string(),
            },
            IdempotencyIssue {
                line: 10,
                operation_type: NonIdempotentOperation::RmWithoutF,
                code: "rm bar".to_string(),
                explanation: "rm without -f fails if file doesn't exist".to_string(),
                suggestion: "Add -f flag: rm -f".to_string(),
            },
        ];

        // ACT: Format report
        let formatted = format_idempotency_report(&issues);

        // ASSERT: Should show all issues
        assert!(formatted.contains("Line 5:"));
        assert!(formatted.contains("Line 10:"));
        assert!(formatted.contains("Found 2 issue(s)"));
    }

    #[test]
    fn test_REPL_012_002_format_no_issues() {
        // ARRANGE: Empty issues
        let issues = vec![];

        // ACT: Format report
        let formatted = format_idempotency_report(&issues);

        // ASSERT: Should show success message
        assert!(formatted.contains("✓"));
        assert!(formatted.contains("idempotent"));
        assert!(formatted.contains("safe to re-run"));
    }

    #[test]
    fn test_REPL_012_002_checker_format_report() {
        // ARRANGE: Checker with detected issues
        let mut checker = IdempotencyChecker::new();
        let script = r#"
mkdir /tmp/foo
rm /tmp/bar
ln -s /tmp/baz /tmp/link
"#;
        checker.scan(script);

        // ACT: Format report
        let formatted = checker.format_report();

        // ASSERT: Should show status and breakdown
        assert!(formatted.contains("⚠️"));
        assert!(formatted.contains("Operation breakdown"));
        assert!(formatted.contains("mkdir (missing -p)"));
        assert!(formatted.contains("rm (missing -f)"));
        assert!(formatted.contains("ln (missing -f)"));
    }

    #[test]
    fn test_REPL_012_002_checker_format_idempotent() {
        // ARRANGE: Checker with idempotent script
        let mut checker = IdempotencyChecker::new();
        let script = r#"
mkdir -p /tmp/foo
rm -f /tmp/bar
ln -sf /tmp/baz /tmp/link
"#;
        checker.scan(script);

        // ACT: Format report
        let formatted = checker.format_report();

        // ASSERT: Should show success status
        assert!(formatted.contains("✓"));
        assert!(formatted.contains("idempotent"));
        assert!(!formatted.contains("⚠️"));
    }

    #[test]
    fn test_REPL_012_002_format_preserves_line_numbers() {
        // ARRANGE: Issues with specific line numbers
        let issues = vec![
            IdempotencyIssue {
                line: 42,
                operation_type: NonIdempotentOperation::MkdirWithoutP,
                code: "mkdir test".to_string(),
                explanation: "explanation".to_string(),
                suggestion: "suggestion".to_string(),
            },
        ];

        // ACT: Format report
        let formatted = format_idempotency_report(&issues);

        // ASSERT: Line number should be preserved
        assert!(formatted.contains("Line 42:"));
    }
}
```

### 4. Property Tests

```rust
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_REPL_012_002_format_never_panics(
            num_issues in 0usize..10,
            line in 1usize..100,
        ) {
            // Generate arbitrary issues
            let issues: Vec<IdempotencyIssue> = (0..num_issues)
                .map(|i| IdempotencyIssue {
                    line: line + i,
                    operation_type: NonIdempotentOperation::MkdirWithoutP,
                    code: format!("mkdir /tmp/test{}", i),
                    explanation: "explanation".to_string(),
                    suggestion: "suggestion".to_string(),
                })
                .collect();

            // Format should never panic
            let _ = format_idempotency_report(&issues);
        }

        #[test]
        fn prop_REPL_012_002_empty_always_idempotent(
            _any in 0..100,
        ) {
            let formatted = format_idempotency_report(&[]);

            // Empty issues should always show idempotent
            prop_assert!(
                formatted.contains("✓") && formatted.contains("idempotent"),
                "Empty report should show idempotent: {}",
                formatted
            );
        }

        #[test]
        fn prop_REPL_012_002_count_matches_issues(
            num_issues in 1usize..20,
        ) {
            let issues: Vec<IdempotencyIssue> = (0..num_issues)
                .map(|i| IdempotencyIssue {
                    line: i + 1,
                    operation_type: NonIdempotentOperation::RmWithoutF,
                    code: format!("rm file{}", i),
                    explanation: "explanation".to_string(),
                    suggestion: "suggestion".to_string(),
                })
                .collect();

            let formatted = format_idempotency_report(&issues);

            // Count should match number of issues
            prop_assert!(
                formatted.contains(&format!("Found {} issue(s)", num_issues)),
                "Report should show correct count: {}",
                formatted
            );
        }
    }
}
```

### 5. Quality Gates

- [ ] ✅ All unit tests pass (≥7 tests)
- [ ] ✅ All property tests pass (≥3 tests)
- [ ] ✅ Coverage >85%
- [ ] ✅ Clippy warnings: 0
- [ ] ✅ Complexity <10 per function
- [ ] ✅ Mutation score ≥90% (deferred to separate pass)

## EXTREME TDD Methodology

### RED Phase
1. Write failing test: `test_REPL_012_002_format_single_mkdir_issue`
2. Write failing test: `test_REPL_012_002_format_single_rm_issue`
3. Write failing test: `test_REPL_012_002_format_multiple_issues`
4. Write failing test: `test_REPL_012_002_format_no_issues`
5. Write failing test: `test_REPL_012_002_checker_format_report`
6. Write failing test: `test_REPL_012_002_checker_format_idempotent`
7. Write failing test: `test_REPL_012_002_format_preserves_line_numbers`
8. Run tests → **FAIL** ✅ (expected)

### GREEN Phase
1. Implement `format_idempotency_report()` function
2. Implement `IdempotencyChecker::format_report()` method
3. Run tests → **PASS** ✅

### REFACTOR Phase
1. Extract formatting helpers if needed
2. Ensure string building is efficient
3. Keep complexity <10
4. Run tests → **PASS** ✅

### PROPERTY Phase
1. Add property test: `prop_REPL_012_002_format_never_panics`
2. Add property test: `prop_REPL_012_002_empty_always_idempotent`
3. Add property test: `prop_REPL_012_002_count_matches_issues`
4. Run property tests (100+ cases) → **PASS** ✅

### MUTATION Phase (Deferred)
1. Run `cargo mutants --file rash/src/repl/determinism.rs`
2. Target: ≥90% kill rate
3. Address any MISSED mutants

## Implementation Plan

### Files to Modify
- `rash/src/repl/determinism.rs` - Add `format_idempotency_report()` and `IdempotencyChecker::format_report()`
- `rash/src/repl/mod.rs` - Export `format_idempotency_report`

### Files to Create
- None (extends existing determinism module)

### Test Files
- `rash/src/repl/determinism.rs` - Unit tests in module
- `rash/src/repl/determinism.rs` - Property tests in module

## Task Breakdown

- [ ] **Task 1**: Write RED tests for report formatting
- [ ] **Task 2**: Implement `format_idempotency_report()` (GREEN phase)
- [ ] **Task 3**: Implement `IdempotencyChecker::format_report()` (GREEN phase)
- [ ] **Task 4**: Refactor if needed (REFACTOR phase)
- [ ] **Task 5**: Add property tests (PROPERTY phase)
- [ ] **Task 6**: Verify all quality gates
- [ ] **Task 7**: Update roadmap (mark REPL-012-002 complete)
- [ ] **Task 8**: Commit with proper message

## Example Usage

```bash
$ bashrs repl
bashrs> :debug examples/deploy.sh
bashrs> :verify-idempotency

Scanning for non-idempotent operations...

⚠️  Script is non-idempotent
  3 issue(s) found

Operation breakdown:
  mkdir (missing -p): 1
  rm (missing -f):    1
  ln (missing -f):    1

⚠️  Non-idempotent operations detected!

Found 3 issue(s):

Line 5:
  Code:        mkdir /app/releases/$RELEASE
  Problem:     mkdir without -p fails if directory already exists
  💡 Fix:      Add -p flag: mkdir -p

Line 10:
  Code:        rm /app/current
  Problem:     rm without -f fails if file doesn't exist
  💡 Fix:      Add -f flag: rm -f

Line 15:
  Code:        ln -s /app/releases/$RELEASE /app/current
  Problem:     ln -s without -f fails if symlink already exists
  💡 Fix:      Add -f flag: ln -sf
```

## Toyota Way Principles

### 自働化 (Jidoka) - Build Quality In
- EXTREME TDD ensures report formatting is correct from first line
- Property tests catch edge cases in formatting logic

### 反省 (Hansei) - Reflect and Improve
- Learn from existing `format_replay_diff()` (replay differences)
- Apply same clarity principles to idempotency reports

### 改善 (Kaizen) - Continuous Improvement
- Clear error messages help developers fix issues faster
- User-friendly output improves developer experience

### 現地現物 (Genchi Genbutsu) - Go and See
- Show actual problems, not just "non-idempotent"
- Help developers understand what's wrong and how to fix it

## Related Files
- `rash/src/repl/determinism.rs` - IdempotencyChecker (REPL-012-001)
- `rash/src/repl/determinism.rs` - format_replay_diff() (reference for formatting pattern)
- `docs/REPL-DEBUGGER-ROADMAP.yaml` - Roadmap reference

## Success Criteria Summary
```
BEFORE: Raw IdempotencyIssue vectors (hard to read)
AFTER:  ✅ Formatted idempotency reports with line numbers
        ✅ Clear code, problem, and fix sections
        ✅ Human-friendly status messages (✓/⚠️)
        ✅ Operation breakdown (mkdir/rm/ln counts)
        ✅ IdempotencyChecker::format_report() for complete output
        ✅ All quality gates passed
        ✅ Property tests validate formatting
```

---

**Created**: 2025-10-30
**Sprint**: REPL-012 (Idempotency Analyzer)
**Estimated Time**: 1-2 hours
**Methodology**: EXTREME TDD (RED → GREEN → REFACTOR → PROPERTY → MUTATION)

---
