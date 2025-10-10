# Sprint 1: Core Lint Infrastructure - EXTREME TDD

**Sprint Goal**: Implement foundational linting infrastructure with 3 critical ShellCheck rules using EXTREME TDD methodology.

**Quality Gates**:
- ✅ Test coverage >85%
- ✅ Mutation score >80%
- ✅ All tests passing (756 baseline + new lint tests)
- ✅ Zero clippy warnings
- ✅ Complexity <10 per function

---

## Ticket LINT-001: Diagnostic Infrastructure
**Priority**: P0 (Blocker)
**Estimate**: 3 hours
**Owner**: TBD

### Description
Create core diagnostic types and infrastructure for linting both ingested shell and generated shell scripts.

### Acceptance Criteria
1. ✅ `Diagnostic` struct with code, severity, message, span, fix
2. ✅ `Severity` enum: Error, Warning, Info, Note
3. ✅ `Fix` struct with replacement text
4. ✅ `LintResult` collection type
5. ✅ 100% test coverage on diagnostic types
6. ✅ Mutation score >80%

### TDD Steps
```rust
// Step 1: Write failing test
#[test]
fn test_diagnostic_creation() {
    let diag = Diagnostic {
        code: "SC2086".to_string(),
        severity: Severity::Warning,
        message: "Double quote to prevent globbing".to_string(),
        span: Span::new(1, 5, 1, 10),
        fix: Some(Fix {
            replacement: "\"$var\"".to_string(),
        }),
    };

    assert_eq!(diag.code, "SC2086");
    assert_eq!(diag.severity, Severity::Warning);
}

// Step 2: Implement minimal code to pass
// Step 3: Run mutation tests
// Step 4: Refactor
```

### Mutation Testing Strategy
```bash
# Verify tests catch bugs
cargo mutants --file rash/src/linter/diagnostic.rs

# Expected mutations to catch:
# - Change severity Warning → Error (test should fail)
# - Remove optional fix (test should fail)
# - Change span coordinates (test should fail)
```

### Files to Create
- `rash/src/linter/mod.rs`
- `rash/src/linter/diagnostic.rs`
- `rash/src/linter/diagnostic_tests.rs`

---

## Ticket LINT-002: SC2086 - Unquoted Variable Expansion
**Priority**: P0 (Blocker)
**Estimate**: 4 hours
**Dependencies**: LINT-001

### Description
Implement ShellCheck SC2086 rule: Detect unquoted variable expansions in shell scripts that could cause word splitting or globbing.

### Acceptance Criteria
1. ✅ Detect `$var` without quotes in command positions
2. ✅ Detect `$var` without quotes in argument positions
3. ✅ Provide auto-fix: `$var` → `"$var"`
4. ✅ Handle false positives (arithmetic context, etc.)
5. ✅ 100% test coverage
6. ✅ Mutation score >80%

### TDD Test Cases
```rust
// Test 1: Basic unquoted variable detection
#[test]
fn test_sc2086_basic_detection() {
    let bash_code = r#"
#!/bin/bash
FILES=$1
ls $FILES  # Should trigger SC2086
"#;

    let diagnostics = lint_shell(bash_code);
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, "SC2086");
    assert!(diagnostics[0].message.contains("Double quote"));
}

// Test 2: Auto-fix suggestion
#[test]
fn test_sc2086_autofix() {
    let bash_code = "ls $FILES";
    let diagnostics = lint_shell(bash_code);

    assert!(diagnostics[0].fix.is_some());
    assert_eq!(diagnostics[0].fix.unwrap().replacement, "\"$FILES\"");
}

// Test 3: False positive - arithmetic context
#[test]
fn test_sc2086_no_false_positive_arithmetic() {
    let bash_code = "result=$(( $x + $y ))";
    let diagnostics = lint_shell(bash_code);

    // Should NOT trigger SC2086 in arithmetic context
    assert_eq!(diagnostics.len(), 0);
}

// Test 4: Multiple violations
#[test]
fn test_sc2086_multiple_violations() {
    let bash_code = r#"
rm -rf $DIR
cat $FILE1 $FILE2
"#;

    let diagnostics = lint_shell(bash_code);
    assert_eq!(diagnostics.len(), 3); // $DIR, $FILE1, $FILE2
}
```

### Mutation Testing Strategy
```bash
# Mutations to verify:
# 1. Change detection logic (should fail tests)
# 2. Skip arithmetic context check (should fail false-positive test)
# 3. Remove auto-fix generation (should fail auto-fix test)

cargo mutants --file rash/src/linter/rules/sc2086.rs
```

### Implementation Steps
1. ✅ Write all test cases first (RED)
2. ✅ Implement minimal SC2086 detection (GREEN)
3. ✅ Run mutation tests (VERIFY)
4. ✅ Refactor for clarity (REFACTOR)
5. ✅ Verify complexity <10

### Files to Create
- `rash/src/linter/rules/mod.rs`
- `rash/src/linter/rules/sc2086.rs`
- `rash/src/linter/rules/sc2086_tests.rs`

---

## Ticket LINT-003: SC2046 - Unquoted Command Substitution
**Priority**: P0 (Blocker)
**Estimate**: 3 hours
**Dependencies**: LINT-001, LINT-002

### Description
Implement ShellCheck SC2046 rule: Detect unquoted command substitutions that could cause word splitting.

### Acceptance Criteria
1. ✅ Detect `$(command)` without quotes
2. ✅ Detect backtick command substitution (deprecated)
3. ✅ Provide auto-fix: `$(cmd)` → `"$(cmd)"`
4. ✅ Handle nested command substitutions
5. ✅ 100% test coverage
6. ✅ Mutation score >80%

### TDD Test Cases
```rust
#[test]
fn test_sc2046_basic_detection() {
    let bash_code = "files=$(find . -name '*.txt')";
    let diagnostics = lint_shell(bash_code);
    assert_eq!(diagnostics[0].code, "SC2046");
}

#[test]
fn test_sc2046_backtick_detection() {
    let bash_code = "files=`ls *.txt`";
    let diagnostics = lint_shell(bash_code);

    // Should trigger both SC2046 (unquoted) and recommend $() syntax
    assert!(diagnostics.iter().any(|d| d.code == "SC2046"));
}

#[test]
fn test_sc2046_nested_substitution() {
    let bash_code = r#"
result=$(echo $(cat file.txt))
"#;
    let diagnostics = lint_shell(bash_code);

    // Should detect both unquoted substitutions
    assert!(diagnostics.len() >= 2);
}
```

### Mutation Testing Strategy
```bash
cargo mutants --file rash/src/linter/rules/sc2046.rs

# Expected mutations:
# - Skip nested substitution detection (test should fail)
# - Change backtick detection (test should fail)
```

---

## Ticket LINT-004: SC2116 - Useless Echo in Command Substitution
**Priority**: P1 (High)
**Estimate**: 2 hours
**Dependencies**: LINT-001

### Description
Implement ShellCheck SC2116 rule: Detect redundant echo wrapping in command substitutions.

### Acceptance Criteria
1. ✅ Detect `$(echo $var)` pattern
2. ✅ Provide auto-fix: `$(echo $var)` → `$var`
3. ✅ Handle edge cases (echo with flags)
4. ✅ 100% test coverage
5. ✅ Mutation score >80%

### TDD Test Cases
```rust
#[test]
fn test_sc2116_basic_detection() {
    let bash_code = "result=$(echo $var)";
    let diagnostics = lint_shell(bash_code);

    assert_eq!(diagnostics[0].code, "SC2116");
    assert!(diagnostics[0].message.contains("Useless echo"));
}

#[test]
fn test_sc2116_autofix() {
    let bash_code = "result=$(echo $var)";
    let diagnostics = lint_shell(bash_code);

    assert_eq!(diagnostics[0].fix.unwrap().replacement, "$var");
}

#[test]
fn test_sc2116_false_positive_with_flags() {
    let bash_code = "result=$(echo -n $var)";
    let diagnostics = lint_shell(bash_code);

    // Should NOT trigger - echo has flags
    assert_eq!(diagnostics.len(), 0);
}
```

---

## Ticket LINT-005: CLI Integration - `bashrs lint` Subcommand
**Priority**: P0 (Blocker)
**Estimate**: 3 hours
**Dependencies**: LINT-001, LINT-002, LINT-003, LINT-004

### Description
Integrate linter into bashrs CLI as `bashrs lint` subcommand with multiple output formats.

### Acceptance Criteria
1. ✅ `bashrs lint <file.sh>` analyzes shell files
2. ✅ `bashrs lint <file.rs>` analyzes Rash source
3. ✅ `--format=human` (default) output
4. ✅ `--format=json` output
5. ✅ `--format=sarif` output
6. ✅ Exit code 0 (no issues), 1 (warnings), 2 (errors)
7. ✅ 100% test coverage on CLI integration
8. ✅ Mutation score >80%

### TDD Test Cases
```rust
#[test]
fn test_lint_shell_file() {
    Command::cargo_bin("bashrs")
        .unwrap()
        .arg("lint")
        .arg("tests/fixtures/unsafe.sh")
        .assert()
        .failure() // Has violations
        .stdout(predicate::str::contains("SC2086"));
}

#[test]
fn test_lint_json_format() {
    let output = Command::cargo_bin("bashrs")
        .unwrap()
        .arg("lint")
        .arg("--format=json")
        .arg("tests/fixtures/unsafe.sh")
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["diagnostics"].is_array());
}

#[test]
fn test_lint_clean_file_exit_code() {
    Command::cargo_bin("bashrs")
        .unwrap()
        .arg("lint")
        .arg("tests/fixtures/safe.sh")
        .assert()
        .success(); // Exit code 0
}
```

### Files to Modify
- `rash/src/main.rs` (add lint subcommand)
- `rash/src/cli.rs` (add lint args)

### Files to Create
- `rash/src/linter/output.rs` (formatters)
- `tests/cli_lint_tests.rs`
- `tests/fixtures/unsafe.sh`
- `tests/fixtures/safe.sh`

---

## Ticket LINT-006: Mutation Testing Suite
**Priority**: P0 (Blocker)
**Estimate**: 2 hours
**Dependencies**: LINT-002, LINT-003, LINT-004, LINT-005

### Description
Comprehensive mutation testing to verify test quality for all lint rules.

### Acceptance Criteria
1. ✅ Mutation score >80% on all linter modules
2. ✅ Document surviving mutants (acceptable mutations)
3. ✅ Add mutation testing to CI pipeline
4. ✅ Create mutation testing report

### Commands
```bash
# Run mutation tests on entire linter module
cargo mutants --package rash --file 'rash/src/linter/**/*.rs' --jobs 8

# Generate mutation report
cargo mutants --json --output mutation-report.json

# Verify mutation score
pmat quality-score --mutation-score --min 80
```

### Deliverables
- `docs/mutation-testing-report-sprint1.md`
- `.github/workflows/mutation-tests.yml` (CI integration)

---

## Sprint 1 Metrics (Quality Gates)

### Code Coverage
```bash
cargo llvm-cov --package rash --html --output-dir coverage-report
cargo llvm-cov --package rash --json --output-path coverage.json

# Verify >85% coverage
pmat coverage-check --min 85 --file coverage.json
```

### Complexity Analysis
```bash
# Verify complexity <10
pmat analyze complexity --max 10 --path rash/src/linter/

# Should output:
# ✅ All functions complexity <10
```

### Test Execution
```bash
# All tests must pass
cargo test --package rash

# Expected: 756 baseline + ~30 lint tests = ~786 tests passing
```

### Performance Benchmarks
```bash
# Lint performance target: <50ms for 1000-line shell script
cargo bench --bench lint_performance

# Should output:
# lint_1000_lines    time: [45.2 ms 46.8 ms 48.3 ms]
```

---

## Sprint 1 Timeline (EXTREME TDD)

### Day 1 (Setup + Infrastructure)
- Hour 0-1: Setup pmat tickets and quality gates
- Hour 1-3: **LINT-001** Diagnostic infrastructure (TDD)
- Hour 3-4: Mutation testing LINT-001

### Day 2 (Core Rules)
- Hour 0-4: **LINT-002** SC2086 implementation (TDD)
- Hour 4-6: Mutation testing LINT-002
- Hour 6-8: **LINT-003** SC2046 implementation (TDD)

### Day 3 (Integration)
- Hour 0-2: **LINT-004** SC2116 implementation (TDD)
- Hour 2-5: **LINT-005** CLI integration (TDD)
- Hour 5-6: Mutation testing LINT-004, LINT-005

### Day 4 (Quality Verification)
- Hour 0-2: **LINT-006** Comprehensive mutation testing
- Hour 2-4: Coverage verification and gap filling
- Hour 4-6: Performance benchmarking
- Hour 6-8: Documentation and retrospective

---

## Definition of Done (Sprint 1)

- [ ] All 6 tickets completed
- [ ] Test coverage >85% (verified with cargo llvm-cov)
- [ ] Mutation score >80% (verified with cargo mutants)
- [ ] All 786+ tests passing
- [ ] Zero clippy warnings
- [ ] Complexity <10 for all functions
- [ ] `bashrs lint` working with 3 output formats
- [ ] Documentation updated
- [ ] Committed to main branch

---

## pmat Integration Commands

```bash
# Create formal tickets in pmat
pmat ticket create --file docs/tickets/SPRINT-1-LINT-CORE.md

# Track progress
pmat ticket list --sprint 1

# Verify quality gates
pmat verify --coverage 85 --mutation 80 --complexity 10

# Generate sprint report
pmat report --sprint 1 --output docs/sprint-reports/sprint-1.md
```

---

**Sprint Start Date**: 2025-10-10
**Sprint End Date**: 2025-10-14
**Review Date**: 2025-10-14 4:00 PM
