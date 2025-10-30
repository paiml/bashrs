# TICKET: REPL-011-001

## Title
Scan for Non-Deterministic Patterns ($RANDOM, $$, timestamps)

## Priority
**P1 - High** (First task in REPL-011 Determinism Checker sprint)

## Status
🟢 **READY TO START** - Dependencies met (REPL-010 completed)

## Context
bashrs's core mission is to produce **deterministic, idempotent bash scripts**. The debugger needs to detect and warn about non-deterministic patterns that violate this principle.

**Non-deterministic patterns to detect**:
1. **$RANDOM** - Bash random number variable
2. **$$** - Current process ID
3. **Timestamps** - `$(date)`, `date +%s`, etc.
4. **$BASHPID** - Bash-specific process ID
5. **$SRANDOM** - Bash 5.1+ cryptographic random

**Purpose**: Build the foundation for determinism verification - warn developers when their scripts contain non-deterministic operations.

## Dependencies
- ✅ REPL-010 (Purification-Aware Debugging) completed
- ✅ DebugSession struct exists
- ✅ REPL parser can handle bash scripts

## Acceptance Criteria

### 1. Add `DeterminismChecker` struct

```rust
/// Detects non-deterministic patterns in bash scripts
#[derive(Debug, Clone, PartialEq)]
pub struct DeterminismChecker {
    /// Patterns detected in the script
    detections: Vec<DeterminismIssue>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeterminismIssue {
    /// Line number where pattern was found
    pub line: usize,
    /// Type of non-deterministic pattern
    pub pattern_type: NonDeterministicPattern,
    /// The actual code that triggered the detection
    pub code: String,
    /// Human-readable explanation
    pub explanation: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NonDeterministicPattern {
    Random,        // $RANDOM
    ProcessId,     // $$
    Timestamp,     // $(date), date +%s, etc.
    BashPid,       // $BASHPID
    SecureRandom,  // $SRANDOM
}

impl DeterminismChecker {
    pub fn new() -> Self {
        Self {
            detections: Vec::new(),
        }
    }

    /// Scan bash script for non-deterministic patterns
    pub fn scan(&mut self, script: &str) -> Vec<DeterminismIssue> {
        self.detections.clear();

        for (line_num, line) in script.lines().enumerate() {
            let line_num = line_num + 1; // 1-indexed

            // Detect $RANDOM
            if line.contains("$RANDOM") || line.contains("${RANDOM}") {
                self.detections.push(DeterminismIssue {
                    line: line_num,
                    pattern_type: NonDeterministicPattern::Random,
                    code: line.to_string(),
                    explanation: "Uses $RANDOM which produces different values on each run".to_string(),
                });
            }

            // Detect $$
            if line.contains("$$") {
                self.detections.push(DeterminismIssue {
                    line: line_num,
                    pattern_type: NonDeterministicPattern::ProcessId,
                    code: line.to_string(),
                    explanation: "Uses $$ (process ID) which changes on each execution".to_string(),
                });
            }

            // Detect timestamps: date, $(date), `date`, etc.
            if line.contains("date") &&
               (line.contains("$(date") || line.contains("`date") ||
                line.contains("date +") || line.contains("date -")) {
                self.detections.push(DeterminismIssue {
                    line: line_num,
                    pattern_type: NonDeterministicPattern::Timestamp,
                    code: line.to_string(),
                    explanation: "Uses date command which produces different values over time".to_string(),
                });
            }

            // Detect $BASHPID
            if line.contains("$BASHPID") || line.contains("${BASHPID}") {
                self.detections.push(DeterminismIssue {
                    line: line_num,
                    pattern_type: NonDeterministicPattern::BashPid,
                    code: line.to_string(),
                    explanation: "Uses $BASHPID which changes on each execution".to_string(),
                });
            }

            // Detect $SRANDOM (Bash 5.1+)
            if line.contains("$SRANDOM") || line.contains("${SRANDOM}") {
                self.detections.push(DeterminismIssue {
                    line: line_num,
                    pattern_type: NonDeterministicPattern::SecureRandom,
                    code: line.to_string(),
                    explanation: "Uses $SRANDOM (cryptographic random) which produces different values on each run".to_string(),
                });
            }
        }

        self.detections.clone()
    }

    /// Check if script is deterministic (no issues found)
    pub fn is_deterministic(&self) -> bool {
        self.detections.is_empty()
    }

    /// Get count of issues by pattern type
    pub fn count_by_pattern(&self, pattern: NonDeterministicPattern) -> usize {
        self.detections.iter()
            .filter(|issue| issue.pattern_type == pattern)
            .count()
    }
}
```

### 2. Unit Tests (RED → GREEN → REFACTOR)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // ===== REPL-011-001: DETERMINISM CHECKER TESTS =====

    #[test]
    fn test_REPL_011_001_detect_random() {
        // ARRANGE: Script with $RANDOM
        let script = "SESSION_ID=$RANDOM\necho $SESSION_ID";
        let mut checker = DeterminismChecker::new();

        // ACT: Scan for patterns
        let issues = checker.scan(script);

        // ASSERT: Should detect $RANDOM
        assert_eq!(issues.len(), 1, "Should find 1 issue");
        assert_eq!(issues[0].line, 1);
        assert_eq!(issues[0].pattern_type, NonDeterministicPattern::Random);
        assert!(issues[0].explanation.contains("RANDOM"));
        assert!(!checker.is_deterministic());
    }

    #[test]
    fn test_REPL_011_001_detect_pid() {
        // ARRANGE: Script with $$
        let script = "TMPFILE=/tmp/script_$$\necho $TMPFILE";
        let mut checker = DeterminismChecker::new();

        // ACT: Scan for patterns
        let issues = checker.scan(script);

        // ASSERT: Should detect $$
        assert_eq!(issues.len(), 1, "Should find 1 issue");
        assert_eq!(issues[0].line, 1);
        assert_eq!(issues[0].pattern_type, NonDeterministicPattern::ProcessId);
        assert!(issues[0].explanation.contains("process ID"));
        assert!(!checker.is_deterministic());
    }

    #[test]
    fn test_REPL_011_001_detect_timestamp() {
        // ARRANGE: Script with date command
        let script = "RELEASE=$(date +%s)\necho $RELEASE";
        let mut checker = DeterminismChecker::new();

        // ACT: Scan for patterns
        let issues = checker.scan(script);

        // ASSERT: Should detect date command
        assert_eq!(issues.len(), 1, "Should find 1 issue");
        assert_eq!(issues[0].line, 1);
        assert_eq!(issues[0].pattern_type, NonDeterministicPattern::Timestamp);
        assert!(issues[0].explanation.contains("date"));
        assert!(!checker.is_deterministic());
    }

    #[test]
    fn test_REPL_011_001_detect_bashpid() {
        // ARRANGE: Script with $BASHPID
        let script = "echo \"Running as $BASHPID\"";
        let mut checker = DeterminismChecker::new();

        // ACT: Scan for patterns
        let issues = checker.scan(script);

        // ASSERT: Should detect $BASHPID
        assert_eq!(issues.len(), 1, "Should find 1 issue");
        assert_eq!(issues[0].line, 1);
        assert_eq!(issues[0].pattern_type, NonDeterministicPattern::BashPid);
        assert!(issues[0].explanation.contains("BASHPID"));
    }

    #[test]
    fn test_REPL_011_001_detect_srandom() {
        // ARRANGE: Script with $SRANDOM (Bash 5.1+)
        let script = "TOKEN=$SRANDOM\necho $TOKEN";
        let mut checker = DeterminismChecker::new();

        // ACT: Scan for patterns
        let issues = checker.scan(script);

        // ASSERT: Should detect $SRANDOM
        assert_eq!(issues.len(), 1, "Should find 1 issue");
        assert_eq!(issues[0].line, 1);
        assert_eq!(issues[0].pattern_type, NonDeterministicPattern::SecureRandom);
        assert!(issues[0].explanation.contains("SRANDOM"));
    }

    #[test]
    fn test_REPL_011_001_deterministic_script() {
        // ARRANGE: Script without non-deterministic patterns
        let script = "VERSION=1.0.0\necho \"Release $VERSION\"";
        let mut checker = DeterminismChecker::new();

        // ACT: Scan for patterns
        let issues = checker.scan(script);

        // ASSERT: Should find no issues
        assert_eq!(issues.len(), 0, "Should find 0 issues");
        assert!(checker.is_deterministic());
    }

    #[test]
    fn test_REPL_011_001_multiple_patterns() {
        // ARRANGE: Script with multiple non-deterministic patterns
        let script = r#"
SESSION_ID=$RANDOM
TMPFILE=/tmp/script_$$
TIMESTAMP=$(date +%s)
        "#.trim();
        let mut checker = DeterminismChecker::new();

        // ACT: Scan for patterns
        let issues = checker.scan(script);

        // ASSERT: Should detect all 3 patterns
        assert_eq!(issues.len(), 3, "Should find 3 issues");
        assert_eq!(checker.count_by_pattern(NonDeterministicPattern::Random), 1);
        assert_eq!(checker.count_by_pattern(NonDeterministicPattern::ProcessId), 1);
        assert_eq!(checker.count_by_pattern(NonDeterministicPattern::Timestamp), 1);
    }

    #[test]
    fn test_REPL_011_001_rescan_clears_previous() {
        // ARRANGE: Scanner with previous results
        let mut checker = DeterminismChecker::new();
        checker.scan("SESSION_ID=$RANDOM");
        assert_eq!(checker.detections.len(), 1);

        // ACT: Scan new script
        let issues = checker.scan("echo 'hello'");

        // ASSERT: Previous results should be cleared
        assert_eq!(issues.len(), 0);
        assert!(checker.is_deterministic());
    }

    #[test]
    fn test_REPL_011_001_line_numbers_correct() {
        // ARRANGE: Multi-line script with issues on different lines
        let script = "echo 'start'\nID=$RANDOM\necho 'middle'\nTMP=$$\necho 'end'";
        let mut checker = DeterminismChecker::new();

        // ACT: Scan for patterns
        let issues = checker.scan(script);

        // ASSERT: Line numbers should be correct
        assert_eq!(issues.len(), 2);
        assert_eq!(issues[0].line, 2); // $RANDOM on line 2
        assert_eq!(issues[1].line, 4); // $$ on line 4
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
        fn prop_REPL_011_001_pattern_detection_never_false_negative(
            prefix in "[a-z_]{0,10}",
            suffix in "[a-z_]{0,10}"
        ) {
            // Scripts with $RANDOM should always be detected
            let script = format!("{}$RANDOM{}", prefix, suffix);
            let mut checker = DeterminismChecker::new();
            let issues = checker.scan(&script);

            prop_assert!(
                issues.iter().any(|i| i.pattern_type == NonDeterministicPattern::Random),
                "Should always detect $RANDOM: '{}'", script
            );
        }

        #[test]
        fn prop_REPL_011_001_deterministic_scripts_pass(
            var_name in "[A-Z_]{1,10}",
            value in "[a-z0-9]{1,20}"
        ) {
            // Simple variable assignments should be deterministic
            let script = format!("{}={}", var_name, value);
            let mut checker = DeterminismChecker::new();
            let issues = checker.scan(&script);

            prop_assert_eq!(issues.len(), 0, "Simple assignments should be deterministic");
            prop_assert!(checker.is_deterministic());
        }

        #[test]
        fn prop_REPL_011_001_scan_never_panics(script in ".*{0,1000}") {
            // Scanner should never panic on any input
            let mut checker = DeterminismChecker::new();
            let _ = checker.scan(&script);
        }

        #[test]
        fn prop_REPL_011_001_rescan_always_clears(
            script1 in ".*{0,100}",
            script2 in ".*{0,100}"
        ) {
            // Rescanning should always clear previous results
            let mut checker = DeterminismChecker::new();
            checker.scan(&script1);
            let issues2 = checker.scan(&script2);

            // Second scan results should match fresh scan
            let mut fresh_checker = DeterminismChecker::new();
            let fresh_issues = fresh_checker.scan(&script2);

            prop_assert_eq!(issues2.len(), fresh_issues.len());
        }
    }
}
```

### 4. Quality Gates

- [ ] ✅ All unit tests pass (≥9 tests)
- [ ] ✅ All property tests pass (≥4 tests)
- [ ] ✅ Coverage >85%
- [ ] ✅ Clippy warnings: 0
- [ ] ✅ Complexity <10 per function
- [ ] ✅ Integration test with DebugSession
- [ ] ✅ Mutation score ≥90% (deferred to separate pass)

## EXTREME TDD Methodology

### RED Phase
1. Write failing test: `test_REPL_011_001_detect_random`
2. Write failing test: `test_REPL_011_001_detect_pid`
3. Write failing test: `test_REPL_011_001_detect_timestamp`
4. Write failing test: `test_REPL_011_001_detect_bashpid`
5. Write failing test: `test_REPL_011_001_detect_srandom`
6. Write failing test: `test_REPL_011_001_deterministic_script`
7. Write failing test: `test_REPL_011_001_multiple_patterns`
8. Write failing test: `test_REPL_011_001_rescan_clears_previous`
9. Write failing test: `test_REPL_011_001_line_numbers_correct`
10. Run tests → **FAIL** ✅ (expected)

### GREEN Phase
1. Implement `DeterminismChecker` struct
2. Implement `NonDeterministicPattern` enum
3. Implement `DeterminismIssue` struct
4. Implement `scan()` method with pattern detection
5. Implement `is_deterministic()` helper
6. Implement `count_by_pattern()` helper
7. Run tests → **PASS** ✅

### REFACTOR Phase
1. Extract pattern detection into separate methods if needed
2. Consider regex-based detection for more robust pattern matching
3. Keep complexity <10
4. Run tests → **PASS** ✅

### PROPERTY Phase
1. Add property test: `prop_REPL_011_001_pattern_detection_never_false_negative`
2. Add property test: `prop_REPL_011_001_deterministic_scripts_pass`
3. Add property test: `prop_REPL_011_001_scan_never_panics`
4. Add property test: `prop_REPL_011_001_rescan_always_clears`
5. Run property tests (100+ cases) → **PASS** ✅

### MUTATION Phase (Deferred)
1. Run `cargo mutants --file rash/src/repl/determinism.rs`
2. Target: ≥90% kill rate
3. Address any MISSED mutants

## Implementation Plan

### Files to Create
- `rash/src/repl/determinism.rs` - New module for determinism checking

### Files to Modify
- `rash/src/repl/mod.rs` - Add `pub mod determinism;`
- `rash/src/repl/debugger.rs` - Optionally integrate with DebugSession (future)

### Test Files
- `rash/src/repl/determinism.rs` - Unit tests in module
- `rash/src/repl/determinism.rs` - Property tests in module

## Task Breakdown

- [ ] **Task 1**: Write RED tests for DeterminismChecker
- [ ] **Task 2**: Implement DeterminismChecker struct and enums (GREEN phase)
- [ ] **Task 3**: Implement scan() method with pattern detection
- [ ] **Task 4**: Refactor if needed (REFACTOR phase)
- [ ] **Task 5**: Add property tests (PROPERTY phase)
- [ ] **Task 6**: Verify all quality gates
- [ ] **Task 7**: Update roadmap (mark REPL-011-001 complete)
- [ ] **Task 8**: Commit with proper message

## Example Usage

```bash
$ bashrs repl
bashrs> :debug examples/deploy.sh
bashrs> :check-determinism

Non-deterministic patterns found:
⚠️  Line 3: SESSION_ID=$RANDOM
    → Uses $RANDOM which produces different values on each run

⚠️  Line 7: TMPFILE=/tmp/deploy_$$
    → Uses $$ (process ID) which changes on each execution

⚠️  Line 12: RELEASE=$(date +%s)
    → Uses date command which produces different values over time

3 issues found. Script is NOT deterministic.

Suggestions:
- Replace $RANDOM with fixed seed or version-based identifier
- Replace $$ with fixed temporary file path
- Replace $(date) with fixed version or release tag
```

## Toyota Way Principles

### 自働化 (Jidoka) - Build Quality In
- EXTREME TDD ensures pattern detection is correct from first line
- Property tests catch edge cases in pattern matching

### 反省 (Hansei) - Reflect and Improve
- Learn from bash best practices (determinism is critical)
- Reflect on what patterns violate determinism

### 改善 (Kaizen) - Continuous Improvement
- Start with basic pattern detection
- Future: AST-based detection for more precision

### 現地現物 (Genchi Genbutsu) - Go and See
- Test with real bash scripts from production
- Validate detection against actual non-deterministic behavior

## Related Files
- `rash/src/repl/debugger.rs` - DebugSession (integration point)
- `rash/src/repl/purifier.rs` - Purifier (reference for transformations)
- `docs/REPL-DEBUGGER-ROADMAP.yaml` - Roadmap reference

## Detection Patterns Summary

| Pattern | Example | Impact |
|---------|---------|--------|
| $RANDOM | `ID=$RANDOM` | Different value each run |
| $$ | `TMPFILE=/tmp/script_$$` | Different PID each run |
| date | `RELEASE=$(date +%s)` | Different timestamp each run |
| $BASHPID | `echo $BASHPID` | Different PID each run |
| $SRANDOM | `TOKEN=$SRANDOM` | Cryptographic random |

## Success Criteria Summary
```
BEFORE: No way to detect non-deterministic patterns
AFTER:  ✅ DeterminismChecker scans scripts
        ✅ Detects $RANDOM, $$, timestamps, $BASHPID, $SRANDOM
        ✅ Reports line numbers and explanations
        ✅ All quality gates passed
        ✅ Property tests validate detection
```

---

**Created**: 2025-10-30
**Sprint**: REPL-011 (Determinism Checker)
**Estimated Time**: 2-3 hours
**Methodology**: EXTREME TDD (RED → GREEN → REFACTOR → PROPERTY → MUTATION)

---
