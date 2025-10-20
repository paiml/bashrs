# Sprint 85: ShellCheck Parity (15 High-Priority Rules)

**Status**: 🟢 READY TO EXECUTE
**Phase**: Phase 2 - Bash/Shell World-Class
**Duration**: 2 weeks (estimated 80 hours)
**Start Date**: 2025-10-20
**Goal**: Implement 15 high-priority ShellCheck rules with comprehensive testing

---

## Executive Summary

Sprint 85 is the first sprint of **Phase 2: Bash/Shell World-Class**, building on Phase 1's Makefile purification success. This sprint focuses on achieving parity with ShellCheck's most critical rules for bash/shell script safety.

### Success Criteria

- ✅ 15 new ShellCheck rules implemented (SC2001-SC2015 equivalents)
- ✅ 150+ tests (10 per rule minimum, EXTREME TDD)
- ✅ ≥95% coverage on new rules
- ✅ 100% auto-fix suggestions on all rules
- ✅ Zero regressions (all 1,752 existing tests pass)
- ✅ Performance: <5ms per file linting
- ✅ Complete documentation (plan + completion docs)

---

## Phase 2 Context

**Phase 2 Goal**: Achieve world-class bash/shell linting and purification capabilities.

**Sprints Overview**:
- **Sprint 85** (this sprint): ShellCheck Parity (15 rules)
- Sprint 86: Bash Purification (25 transformations)
- Sprint 87: Performance & Integration Testing
- Sprint 88: Documentation & v4.0.0 Release

**Duration**: 5-7 weeks total (Sprint 85: 2 weeks)

---

## High-Priority ShellCheck Rules (15 Rules)

### Category 1: Quoting and Word Splitting (5 rules)

**SC2086 - Unquoted Variable Expansion** (✅ ALREADY IMPLEMENTED)
- Status: Implemented in v1.1.0
- Location: `rash/src/linter/rules/sc2086.rs`
- Tests: 8 comprehensive tests
- Auto-fix: Yes

**SC2046 - Unquoted Command Substitution** (✅ ALREADY IMPLEMENTED)
- Status: Implemented in v1.1.0
- Location: `rash/src/linter/rules/sc2046.rs`
- Tests: 8 comprehensive tests
- Auto-fix: Yes

**SC2068 - Double Quote Array Expansions**
```bash
# Bad
for i in $@; do echo "$i"; done

# Good
for i in "$@"; do echo "$i"; done
```
- Priority: HIGH
- Impact: Array word splitting
- Auto-fix: Add quotes around `$@`, `$*`, `${array[@]}`

**SC2048 - Use "$@" (with quotes) to prevent word splitting**
```bash
# Bad
command $*

# Good
command "$@"
```
- Priority: HIGH
- Impact: Argument preservation
- Auto-fix: Replace `$*` with `"$@"`

**SC2066 - Quote variables in [[ ... ]] to prevent globbing**
```bash
# Bad
[[ $var == *.txt ]]

# Good
[[ "$var" == *.txt ]]
```
- Priority: MEDIUM
- Impact: Globbing in conditionals
- Auto-fix: Add quotes

---

### Category 2: Conditionals and Tests (4 rules)

**SC2076 - Don't quote right-hand side of =~ (regex matching)**
```bash
# Bad
[[ "$var" =~ "^[0-9]+$" ]]

# Good
[[ "$var" =~ ^[0-9]+$ ]]
```
- Priority: HIGH
- Impact: Regex matching broken
- Auto-fix: Remove quotes from regex

**SC2070 - Use -n/-z for string length tests**
```bash
# Bad
if [ "$var" ]; then

# Good (explicit)
if [ -n "$var" ]; then
```
- Priority: MEDIUM
- Impact: Clarity, edge cases
- Auto-fix: Replace with `-n` or `-z`

**SC2071 - Use arithmetic comparison, not string**
```bash
# Bad
if [ "$num" > 5 ]; then

# Good
if [ "$num" -gt 5 ]; then
```
- Priority: HIGH
- Impact: Incorrect comparisons
- Auto-fix: Replace `>` with `-gt`, `<` with `-lt`

**SC2072 - Decimal numbers not supported in arithmetic context**
```bash
# Bad
if (( num > 3.14 )); then

# Good
if (( $(echo "$num > 3.14" | bc -l) )); then
```
- Priority: MEDIUM
- Impact: Arithmetic errors
- Auto-fix: Suggest bc/awk for floating point

---

### Category 3: Command Substitution and Execution (3 rules)

**SC2006 - Use $(...) instead of legacy backticks**
```bash
# Bad
result=`date`

# Good
result=$(date)
```
- Priority: HIGH
- Impact: Readability, nesting
- Auto-fix: Replace backticks with `$(...)`

**SC2034 - Variable assigned but never used**
```bash
# Bad
unused_var="value"
echo "Hello"

# Good (remove or use)
echo "Hello"
```
- Priority: MEDIUM
- Impact: Dead code detection
- Auto-fix: Warning only (may be intentional)

**SC2154 - Variable referenced but not assigned**
```bash
# Bad
echo "$undefined_var"

# Good
undefined_var="value"
echo "$undefined_var"
```
- Priority: HIGH
- Impact: Undefined behavior
- Auto-fix: Warning only (check for typos)

---

### Category 4: Loops and Control Flow (3 rules)

**SC2045 - Don't use ls output for iteration**
```bash
# Bad
for f in $(ls); do

# Good
for f in *; do
```
- Priority: HIGH
- Impact: Filenames with spaces break
- Auto-fix: Replace `$(ls)` with glob pattern

**SC2044 - Don't use find output for iteration without -print0**
```bash
# Bad
for f in $(find . -name "*.txt"); do

# Good
while IFS= read -r -d '' f; do
  ...
done < <(find . -name "*.txt" -print0)
```
- Priority: HIGH
- Impact: Filenames with spaces/newlines
- Auto-fix: Replace with while read loop

**SC2043 - Use loop instead of for with single element**
```bash
# Bad
for i in "$var"; do
  echo "$i"
done

# Good
echo "$var"
```
- Priority: LOW
- Impact: Unnecessary complexity
- Auto-fix: Simplify to direct command

---

## Implementation Plan (2 Weeks)

### Week 1: Rules 1-8 (Category 1-2)

#### Day 1-2: Quoting and Word Splitting (3 new rules)
**Tasks**:
1. **SC2068 - Double Quote Array Expansions**
   - Create `rash/src/linter/rules/sc2068.rs`
   - Implement detection for `$@`, `$*`, `${array[@]}` without quotes
   - Write 10 tests (RED → GREEN → REFACTOR)
   - Add auto-fix: wrap in quotes

2. **SC2048 - Quote $@**
   - Create `rash/src/linter/rules/sc2048.rs`
   - Detect unquoted `$*` in command contexts
   - Write 10 tests
   - Auto-fix: replace with `"$@"`

3. **SC2066 - Quote in [[ ... ]]**
   - Create `rash/src/linter/rules/sc2066.rs`
   - Detect unquoted variables in `[[ ... ]]`
   - Write 10 tests
   - Auto-fix: add quotes

**Deliverables**:
- 3 rule files (~300 lines code)
- 30 tests (~600 lines)
- Integration with `lint_bash()` function
- Zero regressions

---

#### Day 3-4: Conditionals and Tests (4 rules)
**Tasks**:
1. **SC2076 - Don't quote regex in =~**
   - Create `rash/src/linter/rules/sc2076.rs`
   - Detect quoted regex in `[[ ... =~ "..." ]]`
   - Write 10 tests
   - Auto-fix: remove quotes

2. **SC2070 - Use -n/-z**
   - Create `rash/src/linter/rules/sc2070.rs`
   - Detect `[ "$var" ]` patterns
   - Write 10 tests
   - Auto-fix: replace with `-n "$var"`

3. **SC2071 - Arithmetic comparison**
   - Create `rash/src/linter/rules/sc2071.rs`
   - Detect string comparison `>` `<` with numbers
   - Write 10 tests
   - Auto-fix: replace with `-gt` `-lt`

4. **SC2072 - Decimal in arithmetic**
   - Create `rash/src/linter/rules/sc2072.rs`
   - Detect floating point in `(( ... ))`
   - Write 10 tests
   - Auto-fix: suggest bc

**Deliverables**:
- 4 rule files (~400 lines code)
- 40 tests (~800 lines)
- All tests passing
- Coverage ≥95%

---

#### Day 5: Integration and Testing
**Tasks**:
- Run full test suite (1,752 existing + 70 new = 1,822 tests)
- Verify zero regressions
- Run coverage analysis (target ≥95% on new rules)
- Performance testing (<5ms per file)
- Clippy and formatting

**Deliverables**:
- All 1,822 tests passing
- Coverage report
- Performance report

---

### Week 2: Rules 9-15 (Category 3-4)

#### Day 6-7: Command Substitution (3 rules)
**Tasks**:
1. **SC2006 - Use $(...)**
   - Create `rash/src/linter/rules/sc2006.rs`
   - Detect backticks
   - Write 10 tests
   - Auto-fix: convert to `$(...)`

2. **SC2034 - Unused variable**
   - Create `rash/src/linter/rules/sc2034.rs`
   - Track variable assignments and usage
   - Write 10 tests
   - Warning only (no auto-fix)

3. **SC2154 - Undefined variable**
   - Create `rash/src/linter/rules/sc2154.rs`
   - Track variable usage and assignments
   - Write 10 tests
   - Warning only (check typos)

**Deliverables**:
- 3 rule files (~300 lines code)
- 30 tests (~600 lines)
- All tests passing

---

#### Day 8-9: Loops and Control Flow (3 rules)
**Tasks**:
1. **SC2045 - Don't use ls**
   - Create `rash/src/linter/rules/sc2045.rs`
   - Detect `for f in $(ls)`
   - Write 10 tests
   - Auto-fix: replace with glob

2. **SC2044 - Don't use find without -print0**
   - Create `rash/src/linter/rules/sc2044.rs`
   - Detect `for f in $(find ...)`
   - Write 10 tests
   - Auto-fix: replace with while read

3. **SC2043 - Loop with single element**
   - Create `rash/src/linter/rules/sc2043.rs`
   - Detect `for i in "$var"`
   - Write 10 tests
   - Auto-fix: simplify

**Deliverables**:
- 3 rule files (~300 lines code)
- 30 tests (~600 lines)
- All tests passing
- Coverage ≥95%

---

#### Day 10: Final Integration and Documentation
**Tasks**:
1. **Final Testing**:
   - Run full test suite (1,752 + 150 = 1,902 tests)
   - Verify zero regressions
   - Coverage analysis (≥95% target)
   - Performance testing (<5ms per file)
   - Mutation testing on new rules

2. **Documentation**:
   - Create `SPRINT-85-COMPLETE.md`
   - Update `ROADMAP.yaml`
   - Update `README.md` with new rule counts
   - Create examples for each rule

**Deliverables**:
- 1,902 tests passing (100% pass rate)
- Coverage ≥95% on new rules
- Complete documentation (3 files)
- Performance report (<5ms per file)

---

## File Structure

```
rash/src/linter/rules/
├── sc2068.rs   # New: Double quote array expansions
├── sc2048.rs   # New: Quote $@
├── sc2066.rs   # New: Quote in [[ ... ]]
├── sc2076.rs   # New: Don't quote regex
├── sc2070.rs   # New: Use -n/-z
├── sc2071.rs   # New: Arithmetic comparison
├── sc2072.rs   # New: Decimal in arithmetic
├── sc2006.rs   # New: Use $(...) not backticks
├── sc2034.rs   # New: Unused variable
├── sc2154.rs   # New: Undefined variable
├── sc2045.rs   # New: Don't use ls
├── sc2044.rs   # New: Don't use find without -print0
├── sc2043.rs   # New: Loop with single element
└── mod.rs      # Update: Register 13 new rules

rash/tests/
├── sc2068_tests.rs   # New: 10 tests
├── sc2048_tests.rs   # New: 10 tests
├── sc2066_tests.rs   # New: 10 tests
├── sc2076_tests.rs   # New: 10 tests
├── sc2070_tests.rs   # New: 10 tests
├── sc2071_tests.rs   # New: 10 tests
├── sc2072_tests.rs   # New: 10 tests
├── sc2006_tests.rs   # New: 10 tests
├── sc2034_tests.rs   # New: 10 tests
├── sc2154_tests.rs   # New: 10 tests
├── sc2045_tests.rs   # New: 10 tests
├── sc2044_tests.rs   # New: 10 tests
└── sc2043_tests.rs   # New: 10 tests

docs/sprints/
├── SPRINT-85-PLAN.md      # This file
└── SPRINT-85-COMPLETE.md  # Created at end
```

---

## Testing Strategy (EXTREME TDD)

### Per-Rule Testing (10 tests each)
1. **Basic detection** (1 test)
2. **Auto-fix correctness** (1 test)
3. **Edge cases** (3 tests)
4. **False positive prevention** (2 tests)
5. **Integration test** (1 test)
6. **Property test** (1 test)
7. **Performance test** (1 test)

### Example: SC2068 Test Suite
```rust
// rash/tests/sc2068_tests.rs

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_sc2068_basic_detection() {
    // Detect unquoted $@
    let script = r#"
#!/bin/bash
for i in $@; do
  echo "$i"
done
"#;
    Command::cargo_bin("bashrs")
        .unwrap()
        .arg("lint")
        .write_stdin(script)
        .assert()
        .success()
        .stdout(predicate::str::contains("SC2068"));
}

#[test]
fn test_sc2068_autofix() {
    // Verify auto-fix adds quotes
    let script = r#"for i in $@; do echo "$i"; done"#;

    Command::cargo_bin("bashrs")
        .unwrap()
        .arg("lint")
        .arg("--fix")
        .write_stdin(script)
        .assert()
        .success()
        .stdout(predicate::str::contains(r#"for i in "$@""#));
}

#[test]
fn test_sc2068_array_expansion() {
    // Detect ${array[@]} without quotes
    let script = r#"
for i in ${array[@]}; do
  echo "$i"
done
"#;
    Command::cargo_bin("bashrs")
        .unwrap()
        .arg("lint")
        .write_stdin(script)
        .assert()
        .success()
        .stdout(predicate::str::contains("SC2068"));
}

#[test]
fn test_sc2068_false_positive_quoted() {
    // Should NOT flag "$@" (already quoted)
    let script = r#"
for i in "$@"; do
  echo "$i"
done
"#;
    Command::cargo_bin("bashrs")
        .unwrap()
        .arg("lint")
        .write_stdin(script)
        .assert()
        .success()
        .stdout(predicate::str::contains("SC2068").not());
}

// ... 6 more tests (edge cases, property, performance)
```

---

## Methodology

### EXTREME TDD (RED → GREEN → REFACTOR)

**For each rule**:
1. **RED**: Write 10 failing tests (1 hour)
2. **GREEN**: Implement rule to pass tests (2 hours)
3. **REFACTOR**: Clean up, ensure complexity <10 (1 hour)
4. **INTEGRATE**: Add to mod.rs, verify no regressions (30 min)

**Total per rule**: ~4.5 hours × 13 rules = ~58 hours
**Buffer**: 22 hours for integration, testing, docs
**Total**: 80 hours (2 weeks)

---

## Quality Gates

All gates must pass before marking sprint complete:

- [ ] ✅ **Tests**: 1,902 tests passing (1,752 existing + 150 new)
- [ ] ✅ **Pass Rate**: 100% (zero failures)
- [ ] ✅ **Coverage**: ≥95% on new rules
- [ ] ✅ **Performance**: <5ms per file linting
- [ ] ✅ **Regressions**: Zero (all existing tests pass)
- [ ] ✅ **Clippy**: Zero warnings
- [ ] ✅ **Complexity**: All functions <10
- [ ] ✅ **Auto-fix**: 100% of rules provide fixes (except SC2034, SC2154)
- [ ] ✅ **Documentation**: Plan + Completion docs

---

## Success Metrics

**Code Metrics**:
- Lines of code added: ~1,500 (13 rules + 130 tests)
- Test count: 1,752 → 1,902 (+150 tests)
- Rule count: 17 → 30 (+13 rules)
- Coverage: 88.71% → ≥95% on new rules

**Quality Metrics**:
- 100% test pass rate
- Zero regressions
- <5ms linting per file
- All auto-fix suggestions correct

**Documentation**:
- Sprint completion report
- ROADMAP.yaml updated
- README.md updated with new rules
- Example scripts for each rule

---

## Risks and Mitigations

### Risk 1: Bash Parsing Complexity
**Impact**: HIGH
**Likelihood**: MEDIUM
**Mitigation**: Use existing bash_parser infrastructure, leverage regex for simple patterns

### Risk 2: False Positives
**Impact**: HIGH
**Likelihood**: MEDIUM
**Mitigation**: Extensive edge case testing, false positive prevention tests

### Risk 3: Performance Degradation
**Impact**: MEDIUM
**Likelihood**: LOW
**Mitigation**: Performance testing per rule, optimize hotspots

### Risk 4: Auto-Fix Correctness
**Impact**: HIGH
**Likelihood**: LOW
**Mitigation**: Comprehensive auto-fix tests, verify with shellcheck

---

## Next Steps After Sprint 85

**Sprint 86: Bash Purification (25 Transformations)**
Duration: 2-3 weeks
Focus: Bash-to-Purified-Bash transformation pipeline

**Sprint 87: Performance & Integration Testing**
Duration: 1 week
Focus: Benchmarks, cross-shell testing, CI/CD

**Sprint 88: Documentation & v4.0.0 Release**
Duration: 1 week
Focus: Complete Phase 2 documentation, release v4.0.0

---

## References

- ShellCheck Wiki: https://www.shellcheck.net/wiki/
- Bash Manual: https://www.gnu.org/software/bash/manual/
- CLAUDE.md: Project development guidelines
- Sprint 83 (Makefile Purification): Reference for transformation patterns
- Sprint 84 (Performance): Reference for benchmarking methodology

---

## Approval and Kickoff

**Status**: 🟢 READY TO EXECUTE
**Prerequisites**: ✅ All met (Phase 1 complete, v3.0.0 released)
**Start Date**: 2025-10-20
**Expected Completion**: 2025-11-03 (2 weeks)

---

**Sprint Owner**: Development Team
**Reviewer**: Project Lead
**Methodology**: EXTREME TDD + Property Testing + Mutation Testing
**Quality Standard**: Zero defects, 100% test pass rate, ≥95% coverage
