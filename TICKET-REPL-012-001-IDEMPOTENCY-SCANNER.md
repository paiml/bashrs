# TICKET: REPL-012-001

## Title
Idempotency Scanner (Detect Non-Idempotent Operations)

## Priority
**P1 - High** (First task in REPL-012 Idempotency Analyzer sprint)

## Status
🟢 **READY TO START** - Dependencies met (REPL-011 sprint completed)

## Context
Building on REPL-011 (Determinism Checker), this task adds **idempotency scanning** - static analysis to detect operations that are not safe to re-run.

**Concept**: An idempotent operation produces the same result when run multiple times. Scripts should be safe to re-run without errors or unintended effects.

**Why Idempotency Scanning?**
- `mkdir` without `-p` fails if directory exists
- `rm` without `-f` fails if file doesn't exist
- `ln -s` without `-f` fails if symlink exists
- Static analysis provides fast feedback (no execution needed)

**Purpose**: Detect non-idempotent operations through pattern matching.

## Dependencies
- ✅ REPL-011-001 (Pattern detection) provides architectural template
- ✅ `DeterminismChecker` exists as reference implementation
- ✅ `PurificationReport.idempotency_fixes` exists (purifier.rs:46)

## Acceptance Criteria

### 1. Add `IdempotencyChecker` struct (similar to `DeterminismChecker`)

```rust
//! Idempotency checking for bash scripts
//!
//! Detects non-idempotent operations that may fail on re-run:
//! - mkdir without -p: Fails if directory exists
//! - rm without -f: Fails if file doesn't exist
//! - ln -s without -f: Fails if symlink exists

/// Detects non-idempotent operations in bash scripts
#[derive(Debug, Clone, PartialEq)]
pub struct IdempotencyChecker {
    /// Issues detected in the script
    detections: Vec<IdempotencyIssue>,
}

/// A single non-idempotent operation detection
#[derive(Debug, Clone, PartialEq)]
pub struct IdempotencyIssue {
    /// Line number where operation was found (1-indexed)
    pub line: usize,
    /// Type of non-idempotent operation
    pub operation_type: NonIdempotentOperation,
    /// The actual code that triggered the detection
    pub code: String,
    /// Human-readable explanation
    pub explanation: String,
    /// Suggested fix
    pub suggestion: String,
}

/// Types of non-idempotent operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NonIdempotentOperation {
    /// mkdir without -p flag
    MkdirWithoutP,
    /// rm without -f flag
    RmWithoutF,
    /// ln -s without -f flag
    LnWithoutF,
}

impl IdempotencyChecker {
    /// Create a new idempotency checker
    pub fn new() -> Self {
        Self {
            detections: Vec::new(),
        }
    }

    /// Scan bash script for non-idempotent operations
    ///
    /// Returns: List of detected issues
    pub fn scan(&mut self, script: &str) -> Vec<IdempotencyIssue> {
        // Clear previous results
        self.detections.clear();

        for (line_num, line) in script.lines().enumerate() {
            let line_num = line_num + 1; // 1-indexed

            // Detect mkdir without -p
            if line.contains("mkdir ") && !line.contains("mkdir -p") {
                self.detections.push(IdempotencyIssue {
                    line: line_num,
                    operation_type: NonIdempotentOperation::MkdirWithoutP,
                    code: line.to_string(),
                    explanation: "mkdir without -p fails if directory already exists".to_string(),
                    suggestion: "Add -p flag: mkdir -p".to_string(),
                });
            }

            // Detect rm without -f (but not rm -rf)
            if line.contains("rm ") && !line.contains(" -f") && !line.contains(" -rf") {
                self.detections.push(IdempotencyIssue {
                    line: line_num,
                    operation_type: NonIdempotentOperation::RmWithoutF,
                    code: line.to_string(),
                    explanation: "rm without -f fails if file doesn't exist".to_string(),
                    suggestion: "Add -f flag: rm -f".to_string(),
                });
            }

            // Detect ln -s without -f
            if line.contains("ln -s") && !line.contains("ln -sf") {
                self.detections.push(IdempotencyIssue {
                    line: line_num,
                    operation_type: NonIdempotentOperation::LnWithoutF,
                    code: line.to_string(),
                    explanation: "ln -s without -f fails if symlink already exists".to_string(),
                    suggestion: "Add -f flag: ln -sf".to_string(),
                });
            }
        }

        self.detections.clone()
    }

    /// Check if script is idempotent (no issues found)
    pub fn is_idempotent(&self) -> bool {
        self.detections.is_empty()
    }

    /// Get count of issues by operation type
    pub fn count_by_operation(&self, operation: NonIdempotentOperation) -> usize {
        self.detections
            .iter()
            .filter(|issue| issue.operation_type == operation)
            .count()
    }
}

impl Default for IdempotencyChecker {
    fn default() -> Self {
        Self::new()
    }
}
```

### 2. Unit Tests (RED → GREEN → REFACTOR)

```rust
#[cfg(test)]]
mod tests {
    use super::*;

    // ===== REPL-012-001: IDEMPOTENCY SCANNER TESTS =====

    #[test]
    fn test_REPL_012_001_detect_mkdir_without_p() {
        // ARRANGE: Script with mkdir (no -p)
        let script = "mkdir /tmp/testdir";
        let mut checker = IdempotencyChecker::new();

        // ACT: Scan for issues
        let issues = checker.scan(script);

        // ASSERT: Should detect mkdir without -p
        assert_eq!(issues.len(), 1, "Should detect 1 issue");
        assert_eq!(issues[0].operation_type, NonIdempotentOperation::MkdirWithoutP);
        assert_eq!(issues[0].line, 1);
        assert!(issues[0].explanation.contains("mkdir"));
        assert!(issues[0].suggestion.contains("-p"));
    }

    #[test]
    fn test_REPL_012_001_mkdir_with_p_is_idempotent() {
        // ARRANGE: Script with mkdir -p
        let script = "mkdir -p /tmp/testdir";
        let mut checker = IdempotencyChecker::new();

        // ACT: Scan for issues
        let issues = checker.scan(script);

        // ASSERT: Should not detect any issues
        assert_eq!(issues.len(), 0, "mkdir -p should be idempotent");
        assert!(checker.is_idempotent());
    }

    #[test]
    fn test_REPL_012_001_detect_rm_without_f() {
        // ARRANGE: Script with rm (no -f)
        let script = "rm /tmp/testfile";
        let mut checker = IdempotencyChecker::new();

        // ACT: Scan for issues
        let issues = checker.scan(script);

        // ASSERT: Should detect rm without -f
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].operation_type, NonIdempotentOperation::RmWithoutF);
        assert!(issues[0].explanation.contains("rm"));
        assert!(issues[0].suggestion.contains("-f"));
    }

    #[test]
    fn test_REPL_012_001_rm_with_f_is_idempotent() {
        // ARRANGE: Script with rm -f
        let script = "rm -f /tmp/testfile";
        let mut checker = IdempotencyChecker::new();

        // ACT: Scan for issues
        let issues = checker.scan(script);

        // ASSERT: Should not detect any issues
        assert_eq!(issues.len(), 0, "rm -f should be idempotent");
    }

    #[test]
    fn test_REPL_012_001_detect_ln_without_f() {
        // ARRANGE: Script with ln -s (no -f)
        let script = "ln -s /source /target";
        let mut checker = IdempotencyChecker::new();

        // ACT: Scan for issues
        let issues = checker.scan(script);

        // ASSERT: Should detect ln -s without -f
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].operation_type, NonIdempotentOperation::LnWithoutF);
        assert!(issues[0].explanation.contains("ln"));
        assert!(issues[0].suggestion.contains("-f"));
    }

    #[test]
    fn test_REPL_012_001_ln_with_sf_is_idempotent() {
        // ARRANGE: Script with ln -sf
        let script = "ln -sf /source /target";
        let mut checker = IdempotencyChecker::new();

        // ACT: Scan for issues
        let issues = checker.scan(script);

        // ASSERT: Should not detect any issues
        assert_eq!(issues.len(), 0, "ln -sf should be idempotent");
    }

    #[test]
    fn test_REPL_012_001_multiple_issues() {
        // ARRANGE: Script with multiple non-idempotent operations
        let script = r#"
mkdir /tmp/dir1
rm /tmp/file1
ln -s /source /target
mkdir -p /tmp/dir2
        "#;
        let mut checker = IdempotencyChecker::new();

        // ACT: Scan for issues
        let issues = checker.scan(script);

        // ASSERT: Should detect 3 issues (line 3 is idempotent)
        assert_eq!(issues.len(), 3);
        assert_eq!(checker.count_by_operation(NonIdempotentOperation::MkdirWithoutP), 1);
        assert_eq!(checker.count_by_operation(NonIdempotentOperation::RmWithoutF), 1);
        assert_eq!(checker.count_by_operation(NonIdempotentOperation::LnWithoutF), 1);
    }

    #[test]
    fn test_REPL_012_001_fully_idempotent_script() {
        // ARRANGE: Fully idempotent script
        let script = r#"
#!/bin/sh
mkdir -p /app/releases
rm -f /app/current
ln -sf /app/releases/v1.0.0 /app/current
        "#;
        let mut checker = IdempotencyChecker::new();

        // ACT: Scan for issues
        let issues = checker.scan(script);

        // ASSERT: Should detect no issues
        assert_eq!(issues.len(), 0);
        assert!(checker.is_idempotent());
    }

    #[test]
    fn test_REPL_012_001_rescan_clears_previous() {
        // ARRANGE: Checker with previous detections
        let mut checker = IdempotencyChecker::new();
        checker.scan("mkdir /tmp/test1");
        assert_eq!(checker.scan("mkdir /tmp/test1").len(), 1);

        // ACT: Scan new script
        let issues = checker.scan("mkdir -p /tmp/test2");

        // ASSERT: Should clear previous detections
        assert_eq!(issues.len(), 0, "Rescan should clear previous detections");
        assert!(checker.is_idempotent());
    }

    #[test]
    fn test_REPL_012_001_line_numbers_correct() {
        // ARRANGE: Multi-line script
        let script = r#"
# Line 1: comment
mkdir /tmp/dir  # Line 2: issue
# Line 3: comment
rm /tmp/file    # Line 4: issue
        "#;
        let mut checker = IdempotencyChecker::new();

        // ACT: Scan for issues
        let issues = checker.scan(script);

        // ASSERT: Line numbers should be correct
        assert_eq!(issues.len(), 2);
        assert_eq!(issues[0].line, 3); // Line 3 in 1-indexed
        assert_eq!(issues[1].line, 5); // Line 5 in 1-indexed
    }
}
```

### 3. Property Tests

```rust
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_REPL_012_001_mkdir_without_p_always_detected(
            path in "[a-z0-9/]{1,50}"
        ) {
            let script = format!("mkdir {}", path);
            let mut checker = IdempotencyChecker::new();
            let issues = checker.scan(&script);

            // mkdir without -p should always be detected
            prop_assert_eq!(issues.len(), 1);
            prop_assert_eq!(issues[0].operation_type, NonIdempotentOperation::MkdirWithoutP);
        }

        #[test]
        fn prop_REPL_012_001_mkdir_with_p_never_detected(
            path in "[a-z0-9/]{1,50}"
        ) {
            let script = format!("mkdir -p {}", path);
            let mut checker = IdempotencyChecker::new();
            let issues = checker.scan(&script);

            // mkdir -p should never be detected as non-idempotent
            prop_assert_eq!(issues.len(), 0);
            prop_assert!(checker.is_idempotent());
        }

        #[test]
        fn prop_REPL_012_001_scan_never_panics(
            script in ".*{0,1000}"
        ) {
            let mut checker = IdempotencyChecker::new();
            // Should never panic on any input
            let _ = checker.scan(&script);
        }

        #[test]
        fn prop_REPL_012_001_rescan_always_clears(
            script1 in "mkdir [a-z]{1,20}",
            script2 in "mkdir -p [a-z]{1,20}"
        ) {
            let mut checker = IdempotencyChecker::new();

            // First scan should find issue
            let issues1 = checker.scan(&script1);
            prop_assert_eq!(issues1.len(), 1);

            // Second scan should clear and find no issues
            let issues2 = checker.scan(&script2);
            prop_assert_eq!(issues2.len(), 0);
            prop_assert!(checker.is_idempotent());
        }
    }
}
```

### 4. Quality Gates

- [ ] ✅ All unit tests pass (≥10 tests)
- [ ] ✅ All property tests pass (≥4 tests)
- [ ] ✅ Coverage >85%
- [ ] ✅ Clippy warnings: 0
- [ ] ✅ Complexity <10 per function
- [ ] ✅ Mutation score ≥90% (deferred to separate pass)

## EXTREME TDD Methodology

### RED Phase
1. Write failing test: `test_REPL_012_001_detect_mkdir_without_p`
2. Write failing test: `test_REPL_012_001_mkdir_with_p_is_idempotent`
3. Write failing test: `test_REPL_012_001_detect_rm_without_f`
4. Write failing test: `test_REPL_012_001_rm_with_f_is_idempotent`
5. Write failing test: `test_REPL_012_001_detect_ln_without_f`
6. Write failing test: `test_REPL_012_001_ln_with_sf_is_idempotent`
7. Write failing test: `test_REPL_012_001_multiple_issues`
8. Write failing test: `test_REPL_012_001_fully_idempotent_script`
9. Write failing test: `test_REPL_012_001_rescan_clears_previous`
10. Write failing test: `test_REPL_012_001_line_numbers_correct`
11. Run tests → **FAIL** ✅ (expected)

### GREEN Phase
1. Create `IdempotencyChecker` struct in `rash/src/repl/determinism.rs` (or new file `idempotency.rs`)
2. Create `IdempotencyIssue` struct
3. Create `NonIdempotentOperation` enum
4. Implement `scan()` method with pattern detection
5. Implement `is_idempotent()` and `count_by_operation()` helpers
6. Run tests → **PASS** ✅

### REFACTOR Phase
1. Extract detection logic into helper methods if needed
2. Ensure complexity <10
3. Keep scan() method simple and readable
4. Run tests → **PASS** ✅

### PROPERTY Phase
1. Add property test: `prop_REPL_012_001_mkdir_without_p_always_detected`
2. Add property test: `prop_REPL_012_001_mkdir_with_p_never_detected`
3. Add property test: `prop_REPL_012_001_scan_never_panics`
4. Add property test: `prop_REPL_012_001_rescan_always_clears`
5. Run property tests (100+ cases) → **PASS** ✅

### MUTATION Phase (Deferred)
1. Run `cargo mutants --file rash/src/repl/determinism.rs` (or idempotency.rs)
2. Target: ≥90% kill rate
3. Address any MISSED mutants

## Implementation Plan

### Files to Modify
- `rash/src/repl/determinism.rs` - Add IdempotencyChecker (after ReplayVerifier) **OR**
- `rash/src/repl/idempotency.rs` - New file for idempotency checking (preferred if module grows)
- `rash/src/repl/mod.rs` - Export IdempotencyChecker types

### Files to Create
- Option 1: Extend `determinism.rs` with idempotency checking
- Option 2: Create new `rash/src/repl/idempotency.rs` module (cleaner separation)

### Test Files
- Same file as implementation (inline tests)

## Task Breakdown

- [ ] **Task 1**: Decide module location (extend determinism.rs vs new idempotency.rs)
- [ ] **Task 2**: Write RED tests for IdempotencyChecker (10 unit tests)
- [ ] **Task 3**: Implement IdempotencyChecker struct and enums (GREEN phase)
- [ ] **Task 4**: Implement scan() method with pattern detection
- [ ] **Task 5**: Refactor if needed (REFACTOR phase)
- [ ] **Task 6**: Add property tests (PROPERTY phase)
- [ ] **Task 7**: Verify all quality gates
- [ ] **Task 8**: Update roadmap (mark REPL-012-001 complete)
- [ ] **Task 9**: Commit with proper message

## Example Usage

```bash
$ bashrs repl
bashrs> :debug examples/deploy.sh
bashrs> :check-idempotency

Scanning for non-idempotent operations...

❌ Found 3 non-idempotent operations:

Line 5: mkdir /app/releases/v1.0.0
  Issue: mkdir without -p fails if directory already exists
  Fix: Add -p flag: mkdir -p

Line 10: rm /app/current
  Issue: rm without -f fails if file doesn't exist
  Fix: Add -f flag: rm -f

Line 15: ln -s /app/releases/v1.0.0 /app/current
  Issue: ln -s without -f fails if symlink already exists
  Fix: Add -f flag: ln -sf

Suggestion: Run :purify to automatically fix these issues
```

## Toyota Way Principles

### 自働化 (Jidoka) - Build Quality In
- EXTREME TDD ensures idempotency detection is correct from first line
- Property tests catch edge cases in pattern matching

### 反省 (Hansei) - Reflect and Improve
- Learn from REPL-011-001 (determinism pattern detection)
- Apply same architecture pattern for consistency

### 改善 (Kaizen) - Continuous Improvement
- Static analysis provides fast feedback (no execution needed)
- Helps developers write safer, more reliable scripts

### 現地現物 (Genchi Genbutsu) - Go and See
- Detect real-world idempotency issues developers encounter
- Provide actionable suggestions for fixes

## Related Files
- `rash/src/repl/determinism.rs` - DeterminismChecker (REPL-011-001) - architectural template
- `rash/src/repl/purifier.rs` - PurificationReport.idempotency_fixes (lines 46-50)
- `docs/REPL-DEBUGGER-ROADMAP.yaml` - Roadmap reference

## Success Criteria Summary
```
BEFORE: No idempotency checking in REPL
AFTER:  ✅ IdempotencyChecker detects mkdir without -p
        ✅ Detects rm without -f
        ✅ Detects ln -s without -f
        ✅ Provides actionable suggestions
        ✅ All quality gates passed
        ✅ Property tests validate detection
```

---

**Created**: 2025-10-30
**Sprint**: REPL-012 (Idempotency Analyzer)
**Estimated Time**: 2-3 hours
**Methodology**: EXTREME TDD (RED → GREEN → REFACTOR → PROPERTY → MUTATION)

---
