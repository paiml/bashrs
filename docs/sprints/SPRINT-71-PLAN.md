# Sprint 71 Plan - Linter Phase 2: Security Rules (SEC)

## Overview

**Sprint**: 71
**Phase**: Linter Phase 2 - Security Rules
**Priority**: Priority 2 (per bashrs-lint-spec.md)
**Duration**: 2-3 weeks (10-15 days)
**Complexity**: High (security rules require careful analysis)

---

## Context

Sprint 70 successfully completed Phase 1 with 6 DET/IDEM rules in just 1 day by leveraging existing infrastructure. Sprint 71 continues Phase 2 by implementing critical Security (SEC) rules to detect injection vulnerabilities and unsafe patterns in shell scripts.

---

## Objectives

### Primary Goal
Implement **8 critical Security rules (SEC001-SEC008)** with comprehensive testing and auto-fix suggestions.

### Success Criteria
- ✅ 8 new SEC rules implemented and tested
- ✅ 100% test coverage on new rules
- ✅ All 1,444+ tests passing (zero regressions)
- ✅ Auto-fix suggestions for fixable rules
- ✅ Full CLI integration verified
- ✅ Documentation complete

---

## Security Rules to Implement

### SEC001: Command Injection via eval ⚠️ **CRITICAL**
**Severity**: Error
**Auto-fix**: Manual review required (not auto-fixable)

**Detects**:
```bash
# ❌ CRITICAL VULNERABILITY
eval "rm -rf $USER_INPUT"
eval "$CMD"
```

**Suggests**:
```bash
# ✅ SAFE ALTERNATIVE
# Use array and proper quoting instead of eval
# Or use explicit command construction
```

**Why this matters**: `eval` with user input is the #1 command injection vector in shell scripts.

---

### SEC002: Unquoted Variable in Command ⚠️ **HIGH**
**Severity**: Error
**Auto-fix**: Safe (add quotes)

**Detects**:
```bash
# ❌ UNSAFE
curl $URL
wget $FILE_PATH
ssh $HOST
```

**Suggests**:
```bash
# ✅ SAFE (auto-fixable)
curl "${URL}"
wget "${FILE_PATH}"
ssh "${HOST}"
```

**Why this matters**: Unquoted variables can lead to command injection if they contain spaces or special characters.

---

### SEC003: Unquoted find -exec {} ⚠️ **HIGH**
**Severity**: Error
**Auto-fix**: Safe (add quotes)

**Detects**:
```bash
# ❌ UNSAFE
find . -name "*.sh" -exec chmod +x {} \;
```

**Suggests**:
```bash
# ✅ SAFE (auto-fixable)
find . -name "*.sh" -exec chmod +x "{}" \;
```

**Why this matters**: Filenames with spaces/special chars can break or execute unintended commands.

---

### SEC004: wget/curl Without TLS Verification ⚠️ **MEDIUM**
**Severity**: Warning
**Auto-fix**: Potentially unsafe (requires user decision)

**Detects**:
```bash
# ❌ INSECURE
wget --no-check-certificate https://example.com/file
curl -k https://api.example.com/data
```

**Suggests**:
```bash
# ✅ SECURE
wget https://example.com/file  # Remove --no-check-certificate
curl https://api.example.com/data  # Remove -k flag
```

**Why this matters**: Disabling TLS verification opens man-in-the-middle attack vectors.

---

### SEC005: Hardcoded Secrets ⚠️ **HIGH**
**Severity**: Error
**Auto-fix**: Manual review required (not auto-fixable)

**Detects**:
```bash
# ❌ HARDCODED SECRET
API_KEY="sk-1234567890abcdef"
PASSWORD="MyP@ssw0rd"
TOKEN="ghp_xxxxxxxxxxxxxxxxxxxx"
AWS_SECRET_ACCESS_KEY="AKIAIOSFODNN7EXAMPLE"
```

**Suggests**:
```bash
# ✅ USE ENVIRONMENT VARIABLES
API_KEY="${API_KEY:-}"
PASSWORD="${PASSWORD:-}"
TOKEN="${GITHUB_TOKEN:-}"
AWS_SECRET_ACCESS_KEY="${AWS_SECRET_ACCESS_KEY:-}"
```

**Why this matters**: Hardcoded secrets in scripts lead to credential leaks when committed to version control.

---

### SEC006: Unsafe Temporary File Creation ⚠️ **MEDIUM**
**Severity**: Warning
**Auto-fix**: Safe (suggest mktemp)

**Detects**:
```bash
# ❌ PREDICTABLE TEMP FILES (race condition)
TMPFILE="/tmp/myapp.$$"
TMPFILE="/tmp/script_temp"
```

**Suggests**:
```bash
# ✅ SECURE TEMP FILES (auto-fixable)
TMPFILE="$(mktemp)"
TMPDIR="$(mktemp -d)"
```

**Why this matters**: Predictable temp file names enable symlink attacks and race conditions.

---

### SEC007: Running Commands as Root Without Validation ⚠️ **MEDIUM**
**Severity**: Warning
**Auto-fix**: Manual review required (context-dependent)

**Detects**:
```bash
# ❌ UNSAFE ROOT OPERATIONS
sudo rm -rf $DIR
sudo chmod 777 $FILE
```

**Suggests**:
```bash
# ✅ ADD VALIDATION
if [ -z "$DIR" ] || [ "$DIR" = "/" ]; then
    echo "Error: Invalid directory"
    exit 1
fi
sudo rm -rf "${DIR}"
```

**Why this matters**: Unvalidated root operations can destroy entire systems if variables are empty or contain `/`.

---

### SEC008: Using `curl | sh` Pattern ⚠️ **CRITICAL**
**Severity**: Error
**Auto-fix**: Manual review required (not auto-fixable)

**Detects**:
```bash
# ❌ EXTREMELY DANGEROUS
curl https://install.example.com/script.sh | sh
wget -qO- https://get.example.com | bash
```

**Suggests**:
```bash
# ✅ DOWNLOAD AND INSPECT FIRST
curl -o install.sh https://install.example.com/script.sh
# Review install.sh before running
chmod +x install.sh
./install.sh
```

**Why this matters**: Piping untrusted URLs directly to shell execution is a massive security risk.

---

## Implementation Plan

### Week 1: Core Security Rules (SEC001-SEC004)

**Days 1-2: SEC001 (eval injection) + SEC002 (unquoted vars)**
- Day 1: RED tests for SEC001 + SEC002
- Day 1: GREEN implementation for SEC001 + SEC002
- Day 2: REFACTOR + Property tests
- Day 2: CLI integration tests

**Days 3-4: SEC003 (find -exec) + SEC004 (TLS verification)**
- Day 3: RED tests for SEC003 + SEC004
- Day 3: GREEN implementation for SEC003 + SEC004
- Day 4: REFACTOR + Property tests
- Day 4: CLI integration tests

**Day 5: Integration and Testing**
- Run full test suite
- Fix any regressions
- Verify all 4 rules work together

### Week 2: Advanced Security Rules (SEC005-SEC008)

**Days 6-7: SEC005 (hardcoded secrets) + SEC006 (temp files)**
- Day 6: RED tests for SEC005 + SEC006
- Day 6: GREEN implementation for SEC005 + SEC006
- Day 7: REFACTOR + Property tests
- Day 7: CLI integration tests

**Days 8-9: SEC007 (root validation) + SEC008 (curl | sh)**
- Day 8: RED tests for SEC007 + SEC008
- Day 8: GREEN implementation for SEC007 + SEC008
- Day 9: REFACTOR + Property tests
- Day 9: CLI integration tests

**Day 10: Integration, Documentation, and Polish**
- Run full test suite (target: 1,500+ tests)
- Create comprehensive test fixture with all SEC violations
- Write SPRINT-71-COMPLETION.md
- Update ROADMAP.yaml

### Week 3 (Buffer): Hardening and Edge Cases

**Days 11-12: Edge Case Handling**
- Handle false positives for SEC rules
- Refine detection patterns
- Add more comprehensive tests

**Days 13-14: Property Testing + Mutation Testing**
- Add property tests for all 8 SEC rules
- Run mutation testing on SEC modules
- Target: ≥90% mutation kill rate

**Day 15: Release Preparation**
- Update CHANGELOG.md for v1.6.0
- Bump version numbers
- Prepare release notes

---

## Technical Approach

### Rule Pattern

Each SEC rule follows the established pattern from Sprint 70:

```rust
// rash/src/linter/rules/sec001.rs

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};

/// Check for command injection via eval
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        // Detect eval usage
        if line.contains("eval") {
            if let Some(col) = line.find("eval") {
                let span = Span::new(
                    line_num + 1,
                    col + 1,
                    line_num + 1,
                    col + 5,  // "eval" is 4 chars
                );

                let diag = Diagnostic::new(
                    "SEC001",
                    Severity::Error,
                    "Command injection risk via eval",
                    span,
                );
                // NO AUTO-FIX for SEC001 (manual review required)

                result.add(diag);
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_SEC001_detects_eval_usage() {
        let script = r#"eval "rm -rf $USER_INPUT""#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "SEC001");
        assert_eq!(diag.severity, Severity::Error);
    }

    #[test]
    fn test_SEC001_no_false_positive() {
        let script = "# This is eval-uation, not eval";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0);
    }
}
```

### Integration with mod.rs

Update `rash/src/linter/rules/mod.rs`:

```rust
// Security rules (bashrs-specific)
pub mod sec001;
pub mod sec002;
pub mod sec003;
pub mod sec004;
pub mod sec005;
pub mod sec006;
pub mod sec007;
pub mod sec008;

pub fn lint_shell(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // ... existing rules ...

    // Run security rules
    result.merge(sec001::check(source));
    result.merge(sec002::check(source));
    result.merge(sec003::check(source));
    result.merge(sec004::check(source));
    result.merge(sec005::check(source));
    result.merge(sec006::check(source));
    result.merge(sec007::check(source));
    result.merge(sec008::check(source));

    result
}
```

---

## Testing Strategy

### Unit Tests (Per Rule)

**Required tests**:
1. `test_SECXXX_detects_<issue>` - Basic detection
2. `test_SECXXX_no_false_positive` - Avoids false positives
3. `test_SECXXX_provides_fix` - Auto-fix test (if applicable)
4. `test_SECXXX_manual_review` - Manual review required (if applicable)
5. `test_SECXXX_multiple_violations` - Multiple issues in one script

**Target**: 5 tests per rule × 8 rules = 40 new unit tests

### Property Tests

**Generative testing** for each rule:

```rust
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_SEC002_unquoted_vars_always_detected(
            var_name in "[A-Z_]+",
            command in "curl|wget|ssh|git"
        ) {
            let script = format!("{} ${}", command, var_name);
            let result = check(&script);
            prop_assert_eq!(result.diagnostics.len(), 1);
        }
    }
}
```

**Target**: 100+ property test cases per rule

### CLI Integration Tests

**End-to-end verification**:

```rust
#[test]
fn test_SEC001_cli_integration() {
    use assert_cmd::Command;
    use std::fs;

    let temp = "/tmp/test_sec001.sh";
    fs::write(temp, r#"eval "rm -rf $USER_INPUT""#).unwrap();

    Command::cargo_bin("bashrs")
        .unwrap()
        .arg("lint")
        .arg(temp)
        .assert()
        .failure()
        .stdout(predicate::str::contains("SEC001"));

    fs::remove_file(temp).unwrap();
}
```

**Target**: 8 CLI integration tests (1 per rule)

### Comprehensive Test Fixture

Create `/tmp/test_all_sec_rules.sh` with all 8 violations:

```bash
#!/bin/bash
# Test fixture for all SEC rules

# SEC001: Command injection via eval
eval "rm -rf $USER_INPUT"

# SEC002: Unquoted variable in command
curl $URL
wget $FILE_PATH

# SEC003: Unquoted find -exec
find . -name "*.sh" -exec chmod +x {} \;

# SEC004: TLS verification disabled
wget --no-check-certificate https://example.com/file
curl -k https://api.example.com/data

# SEC005: Hardcoded secrets
API_KEY="sk-1234567890abcdef"
PASSWORD="MyP@ssw0rd"

# SEC006: Unsafe temporary file
TMPFILE="/tmp/myapp.$$"

# SEC007: Unsafe root operation
sudo rm -rf $DIR

# SEC008: curl | sh pattern
curl https://install.example.com/script.sh | sh
```

**Expected output**: 12 violations detected (some rules detect multiple instances)

---

## Quality Metrics

### Test Coverage
- **Target**: >85% coverage on new SEC modules
- **Method**: `cargo llvm-cov` after all rules implemented

### Mutation Testing
- **Target**: ≥90% mutation kill rate
- **Method**: `cargo mutants --file rash/src/linter/rules/sec*.rs`

### Performance
- **Target**: <10ms to lint test fixture with all rules
- **Method**: Benchmark with `cargo bench` (if needed)

### Regression Testing
- **Target**: 100% test pass rate (all 1,444+ existing tests + 48 new tests)
- **Method**: `cargo test --lib`

---

## Deliverables

### Code Files (8 new rule files)
1. `rash/src/linter/rules/sec001.rs` (eval injection)
2. `rash/src/linter/rules/sec002.rs` (unquoted vars)
3. `rash/src/linter/rules/sec003.rs` (find -exec)
4. `rash/src/linter/rules/sec004.rs` (TLS verification)
5. `rash/src/linter/rules/sec005.rs` (hardcoded secrets)
6. `rash/src/linter/rules/sec006.rs` (temp files)
7. `rash/src/linter/rules/sec007.rs` (root validation)
8. `rash/src/linter/rules/sec008.rs` (curl | sh)

### Modified Files
1. `rash/src/linter/rules/mod.rs` (integration)
2. `ROADMAP.yaml` (Sprint 71 completion)
3. `CHANGELOG.md` (v1.6.0 features)

### Documentation
1. `docs/sprints/SPRINT-71-PLAN.md` (this file)
2. `docs/sprints/SPRINT-71-COMPLETION.md` (completion report)
3. `docs/sprints/SPRINT-71-QRC.md` (quick reference card)

### Test Files
1. 40 unit tests (5 per rule × 8 rules)
2. 8 property tests (100+ cases each)
3. 8 CLI integration tests
4. 1 comprehensive test fixture

---

## Risk Mitigation

### Risk 1: False Positives in SEC Rules
**Probability**: Medium
**Impact**: High (security rules must be accurate)

**Mitigation**:
- Extensive property testing
- Review existing shellcheck patterns
- Community feedback (mark as "warning" initially if uncertain)

### Risk 2: Complex Pattern Detection
**Probability**: Medium
**Impact**: Medium (some patterns hard to detect with simple string matching)

**Mitigation**:
- Start with simple patterns (token-based)
- Plan AST-based approach for Phase 3
- Document known limitations

### Risk 3: Performance Degradation
**Probability**: Low
**Impact**: Medium (8 new rules = 8x more checks per script)

**Mitigation**:
- Benchmark after each rule added
- Optimize hot paths if needed
- Use early returns in detection logic

---

## Next Steps After Sprint 71

**Option A**: Continue with Portability Rules (P001-P080)
- Implement POSIX compliance checks
- Shell dialect compatibility
- 80+ rules (6-8 weeks)

**Option B**: Implement Core ShellCheck-equivalent Rules
- Q001-Q025 (Quoting issues)
- C001-C030 (Conditional logic)
- CMD001-CMD050 (Command usage)
- 105+ rules (8-10 weeks)

**Option C**: Focus on AST-Based Rule Engine
- Transition from token-based to AST-based detection
- Enables more sophisticated pattern matching
- Improves accuracy for existing rules

**Recommendation**: Option A (Portability Rules) to continue systematic category completion.

---

## Success Criteria

Sprint 71 will be considered successful when:

- [ ] ✅ All 8 SEC rules implemented (SEC001-SEC008)
- [ ] ✅ 48 new tests passing (40 unit + 8 CLI)
- [ ] ✅ Property tests passing (800+ cases)
- [ ] ✅ All 1,444+ existing tests passing (zero regressions)
- [ ] ✅ Mutation score ≥90% on SEC modules
- [ ] ✅ CLI integration verified
- [ ] ✅ Documentation complete
- [ ] ✅ ROADMAP.yaml updated
- [ ] ✅ Ready for v1.6.0 release

---

**Sprint Owner**: Claude Code (AI Assistant)
**Methodology**: EXTREME TDD (RED-GREEN-REFACTOR-INTEGRATION)
**Quality Standard**: Zero defects, 100% test pass rate
**Timeline**: 2-3 weeks (10-15 days)

**Status**: 📋 PLAN READY
**Next Step**: Begin RED phase for SEC001 + SEC002
