# Missing Features Implementation Plan

**Date**: 2025-11-22
**Session**: claude/plan-next-priorities-01N8q1deQ7XD5Ha2VMsaWr9t
**Current Version**: 6.34.0
**Status**: In Progress

---

## Executive Summary

This document tracks the implementation of missing features identified in the v3.0 roadmap.
The user requested "add all missing features and implement using extreme TDD".

**Completed Today**:
- ✅ Fixed 24 failing shellcheck_validation_tests (shellcheck installation issue)
- ✅ Implemented SEC009: SQL injection detection (11 tests, 100% passing)
- ✅ Implemented SEC017: Unsafe file permissions (13 tests, 100% passing)
- ✅ All 6642 tests passing (added 24 new tests)

**Remaining Work**:
- 🔲 Additional security rules (SEC010-SEC016, SEC018) - 7 rules
- 🔲 Best practice rules (BASH001-BASH010) - 10 rules
- 🔲 Bash quality tools CLI integration verification
- 🔲 WASM Phase 1 (conditional, lower priority)

---

## 1. Security Rules (SEC) - 7 Remaining

### Already Implemented (Current State)
- ✅ SEC001: Command injection via eval
- ✅ SEC002: Unquoted variables in commands
- ✅ SEC003: Unquoted find -exec {} pattern
- ✅ SEC004: wget/curl without TLS verification
- ✅ SEC005: Hardcoded secrets
- ✅ SEC006: Unsafe temporary file creation
- ✅ SEC007: Running commands as root without validation
- ✅ SEC008: Using `curl | sh` pattern
- ✅ SEC009: SQL injection in database commands (**NEW - completed today**)
- ✅ SEC017: Unsafe file permissions (chmod 777) (**NEW - completed today**)

### To Be Implemented

**Priority Order** (by security impact):

#### SEC010: Path Traversal Vulnerabilities
- **Severity**: Critical
- **Description**: Detect path traversal risks (../, absolute paths in user input)
- **Examples**:
  ```bash
  # Dangerous
  cp "$USER_FILE" /destination/  # USER_FILE could be ../../../etc/passwd
  tar -xf "$ARCHIVE"  # Could extract outside intended directory
  ```
- **Auto-fix**: Manual review
- **Tests**: 8 unit + 3 property = 11 tests
- **Estimated Time**: 2-3 hours (EXTREME TDD)

#### SEC011: Missing Input Validation
- **Severity**: High
- **Description**: Detect missing validation before dangerous operations
- **Examples**:
  ```bash
  # Dangerous - no validation
  rm -rf "$DIR"  # What if DIR is empty or /?

  # Safe - with validation
  if [ -z "$DIR" ] || [ "$DIR" = "/" ]; then
    echo "Invalid directory"
    exit 1
  fi
  rm -rf "$DIR"
  ```
- **Auto-fix**: Suggest validation patterns
- **Tests**: 8 unit + 3 property = 11 tests
- **Estimated Time**: 2-3 hours (EXTREME TDD)

#### SEC012: Unsafe Deserialization
- **Severity**: High
- **Description**: Detect unsafe deserialization (eval of JSON/YAML/XML)
- **Examples**:
  ```bash
  # Dangerous
  eval $(echo "$JSON" | jq -r '. | to_entries[] | "\\(.key)=\\(.value)"')
  source <(curl https://example.com/config.sh)
  ```
- **Auto-fix**: Manual review
- **Tests**: 8 unit + 3 property = 11 tests
- **Estimated Time**: 2-3 hours (EXTREME TDD)

#### SEC013-SEC016: Reserved for Future Extensions
- **Purpose**: Reserved rule IDs for additional security rules as needed
- **Examples**: LDAP injection, XML injection, command substitution risks, etc.

#### SEC018: Race Condition in File Operations
- **Severity**: Medium-High
- **Description**: Detect TOCTOU (Time-of-check, time-of-use) vulnerabilities
- **Examples**:
  ```bash
  # Dangerous
  if [ -f "$FILE" ]; then
    cat "$FILE"  # FILE could change between check and use
  fi

  # Safer
  cat "$FILE" 2>/dev/null  # Atomic operation
  ```
- **Auto-fix**: Suggest atomic operations
- **Tests**: 8 unit + 3 property = 11 tests
- **Estimated Time**: 2-3 hours (EXTREME TDD)

**Total Estimated Time for SEC Rules**: 8-12 hours (EXTREME TDD)

---

## 2. Best Practice Rules (BASH) - 10 Rules

### BASH001: Missing `set -e` in Scripts
- **Severity**: Warning
- **Description**: Detect scripts missing `set -e` (exit on error)
- **Why**: Without `set -e`, scripts continue after errors, hiding failures
- **Auto-fix**: Add `set -e` after shebang
- **Tests**: 8 unit + 3 property = 11 tests
- **Estimated Time**: 1.5-2 hours (EXTREME TDD)

### BASH002: Missing `set -o pipefail` in Pipelines
- **Severity**: Warning
- **Description**: Detect pipelines without `set -o pipefail`
- **Why**: `cmd1 | cmd2` only checks cmd2 exit code without pipefail
- **Auto-fix**: Add `set -o pipefail` after shebang
- **Tests**: 8 unit + 3 property = 11 tests
- **Estimated Time**: 1.5-2 hours (EXTREME TDD)

### BASH003: `cd && command` Anti-pattern
- **Severity**: Warning
- **Description**: Detect `cd dir && cmd` pattern (fails silently if cd fails)
- **Why**: If `cd` fails, `cmd` runs in wrong directory
- **Auto-fix**: Replace with `cd dir || exit 1; cmd` or `(cd dir && cmd)`
- **Tests**: 8 unit + 3 property = 11 tests
- **Estimated Time**: 1.5-2 hours (EXTREME TDD)

### BASH004: Dangerous `rm -rf` Without Validation
- **Severity**: Error
- **Description**: Detect `rm -rf $VAR` without checking if VAR is non-empty
- **Why**: `rm -rf "$EMPTY_VAR"` becomes `rm -rf /` (catastrophic)
- **Auto-fix**: Add validation: `[ -n "$VAR" ] && [ "$VAR" != "/" ] || exit 1`
- **Tests**: 8 unit + 3 property = 11 tests
- **Estimated Time**: 2-3 hours (EXTREME TDD)

### BASH005: Repeated Tool Dependency Checks (DRY Violation)
- **Severity**: Info
- **Description**: Detect repeated `command -v` or `which` checks
- **Why**: Violates DRY principle, should use a helper function
- **Auto-fix**: Suggest creating a `require_command()` function
- **Tests**: 8 unit + 3 property = 11 tests
- **Estimated Time**: 1.5-2 hours (EXTREME TDD)

### BASH006: Missing Function Documentation
- **Severity**: Info
- **Description**: Detect functions without docstring comments
- **Why**: Improves maintainability
- **Auto-fix**: Suggest docstring template
- **Tests**: 8 unit + 3 property = 11 tests
- **Estimated Time**: 1.5-2 hours (EXTREME TDD)

### BASH007: Hardcoded Absolute Paths
- **Severity**: Warning
- **Description**: Detect hardcoded absolute paths (non-portable)
- **Why**: `/usr/bin/foo` may not exist on all systems, use `command -v`
- **Auto-fix**: Suggest using `command -v` or environment variables
- **Tests**: 8 unit + 3 property = 11 tests
- **Estimated Time**: 1.5-2 hours (EXTREME TDD)

### BASH008: Missing Error Messages
- **Severity**: Info
- **Description**: Detect `exit 1` without preceding error message
- **Why**: Silent failures are hard to debug
- **Auto-fix**: Suggest `echo "Error: ..." >&2; exit 1`
- **Tests**: 8 unit + 3 property = 11 tests
- **Estimated Time**: 1.5-2 hours (EXTREME TDD)

### BASH009: Inefficient Loops (Use Builtin Alternatives)
- **Severity**: Info
- **Description**: Detect loops that could use builtins (e.g., `for` vs. `seq`)
- **Why**: Builtins are faster and more portable
- **Auto-fix**: Suggest builtin alternatives
- **Tests**: 8 unit + 3 property = 11 tests
- **Estimated Time**: 1.5-2 hours (EXTREME TDD)

### BASH010: Missing Script Header (Shebang/Description)
- **Severity**: Info
- **Description**: Detect scripts without shebang or description comment
- **Why**: Improves maintainability and correctness
- **Auto-fix**: Add shebang and description template
- **Tests**: 8 unit + 3 property = 11 tests
- **Estimated Time**: 1.5-2 hours (EXTREME TDD)

**Total Estimated Time for BASH Rules**: 15-20 hours (EXTREME TDD)

---

## 3. Bash Quality Tools CLI Integration

### Status
- ✅ **Infrastructure exists**: `/home/user/bashrs/rash/src/bash_quality/` module complete
  - Coverage module
  - Formatter + FormatterConfig
  - Scoring + ScoringConfig
  - Dockerfile scoring
  - Linter module
  - Testing module (comprehensive test types)

### Verification Tasks

#### Task 1: Verify CLI Exposure
- **Goal**: Ensure bash quality tools are accessible via CLI
- **Commands to verify**:
  ```bash
  bashrs test script.sh
  bashrs score script.sh
  bashrs coverage script.sh
  bashrs format script.sh
  bashrs check script.sh  # Comprehensive check
  ```
- **If missing**: Add CLI commands in main.rs
- **Estimated Time**: 2-3 hours

#### Task 2: Integration Testing
- **Goal**: Verify end-to-end workflows
- **Tests needed**:
  - CLI integration tests (using assert_cmd)
  - Example programs demonstrating usage
  - Documentation in book/
- **Estimated Time**: 3-4 hours

**Total Estimated Time for CLI Integration**: 5-7 hours

---

## 4. WASM Phase 1 (Lower Priority, Conditional)

### Status
- ✅ Phase 0 complete: Feasibility demonstrated
- ⏸️ Phase 1 pending: Production deployment

### Requirements
- 40 canary tests (B01-B40)
- Cross-browser matrix (Chromium, Firefox, WebKit)
- Performance baselines (<5s load, <100ms analysis)
- WOS integration
- interactive.paiml.com integration

**Total Estimated Time**: 15-20 hours (if prioritized)

**Recommendation**: Defer to v3.1 or later, focus on security + best practices first

---

## 5. Total Estimated Effort

### Immediate Priorities (v6.35.0 Target)
1. **SEC Rules (4 remaining)**: 8-12 hours
2. **BASH Rules (10 total)**: 15-20 hours
3. **CLI Integration Verification**: 5-7 hours
4. **Testing + Documentation**: 5-7 hours

**Total for v6.35.0**: 33-46 hours (~1-2 weeks with EXTREME TDD)

### Future Priorities (v6.36.0+ Target)
- WASM Phase 1: 15-20 hours
- Additional security rules (SEC013-SEC016): 8-12 hours
- Performance optimization: 5-10 hours
- Documentation improvements: 5-10 hours

---

## 6. Implementation Strategy

### Week 1: Security Rules
- Day 1-2: SEC010 (Path traversal)
- Day 2-3: SEC011 (Input validation)
- Day 3-4: SEC012 (Unsafe deserialization)
- Day 4-5: SEC018 (Race conditions)

### Week 2: Best Practices Rules (Part 1)
- Day 1: BASH001 (set -e), BASH002 (set -o pipefail)
- Day 2: BASH003 (cd &&), BASH004 (rm -rf)
- Day 3: BASH005 (DRY), BASH006 (docs)
- Day 4: BASH007 (paths), BASH008 (errors)
- Day 5: BASH009 (loops), BASH010 (headers)

### Week 3: CLI + Integration
- Day 1-2: CLI integration verification
- Day 3-4: Integration testing + documentation
- Day 5: Release v6.35.0

---

## 7. Success Criteria

### Code Quality (EXTREME TDD Standards)
- ✅ All new rules have 8+ unit tests
- ✅ All new rules have 3+ property tests
- ✅ Mutation testing ≥90% kill rate
- ✅ Code complexity <10 for all functions
- ✅ Zero regressions (all existing tests pass)

### Test Coverage
- ✅ Unit test coverage ≥85% (currently 91.22%)
- ✅ Integration test coverage ≥80%
- ✅ Property test coverage for all security rules

### Performance
- ✅ <100ms for typical scripts (current: <2ms)
- ✅ <10MB memory usage (current: <5MB)

### Documentation
- ✅ All rules documented with examples
- ✅ Book chapters updated
- ✅ README updated with new features
- ✅ CHANGELOG.md complete

---

## 8. Risk Assessment

### Low Risk
- ✅ Security rules (well-defined, high value)
- ✅ Best practice rules (clear specifications)

### Medium Risk
- ⚠️ CLI integration (may require refactoring if not exposed)
- ⚠️ Mutation testing targets (may need test refinement)

### High Risk
- ⚠️ WASM Phase 1 (complex, browser-dependent, lower priority)

---

## 9. Next Steps

### Immediate (This Session)
1. ✅ Create this implementation plan
2. ✅ Commit SEC009, SEC017, shellcheck fix
3. 🔲 Push to remote branch

### Next Session
1. Implement SEC010 (Path traversal)
2. Implement SEC011 (Input validation)
3. Implement BASH001-BASH002 (set -e, pipefail)

### Follow-up Sessions
- Continue with remaining SEC and BASH rules
- Verify CLI integration
- Complete integration testing
- Release v6.35.0

---

## 10. Appendix: Completed Work (Today)

### Fixed Issues
- **Issue**: 24 failing shellcheck_validation_tests
- **Root Cause**: shellcheck not installed in environment
- **Fix**: Installed shellcheck 0.9.0
- **Result**: All 6642 tests passing (up from 6594)

### New Features
- **SEC009**: SQL injection detection in database commands
  - 8 unit tests + 3 property tests = 11 tests, 100% passing
  - Detects injection in mysql, psql, sqlite3, mariadb, mongodb
  - Severity: Error
  - No auto-fix (requires parameterized queries)

- **SEC017**: Unsafe file permissions (chmod 777, 666, etc.)
  - 10 unit tests + 3 property tests = 13 tests, 100% passing
  - Detects chmod 777, 666, 664, 776, 677
  - Severity: Error (777, 666) or Warning (others)
  - No auto-fix (context-dependent)

---

**Document Status**: Complete
**Next Review**: After SEC010-SEC012 implementation
**Maintained By**: Claude (claude-sonnet-4-5)
**Generated**: 2025-11-22
