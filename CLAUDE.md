# CLAUDE.md - Rash Development Guidelines

## Project Context
**Rash (bashrs)** is a shell safety and purification tool:

### 🚨 IMPLEMENTATION STATUS (Updated: 2024-10-18)

**What's IMPLEMENTED (v1.4.0)**:
- ✅ **Bash → Purified Bash**: Parse bash scripts and output deterministic, idempotent POSIX sh (70% production-ready)
- ✅ **Makefile Purification**: Parse and purify Makefiles (v1.4.0)
- ✅ **Security Linter**: 8 critical security rules (SEC001-SEC008)
- ✅ **Determinism/Idempotency Rules**: 6 DET/IDEM linter rules

**What's PLANNED (v3.0+)**:
- ⏸️ **Rust → Safe Shell**: Write Rust code, transpile to shell (infrastructure partial, not production-ready)
- ⏸️ **Full Linter**: 800+ rules (current: 14 rules, 1.75% complete)

---

### PRIMARY WORKFLOW: Bash → Purified Bash (v1.4.0 - WORKING)

Ingest messy bash scripts and output purified, safe, deterministic POSIX shell.

**Purification pipeline**:
1. Parse legacy bash (with $RANDOM, timestamps, non-idempotent code)
2. Apply semantic transformations (determinism + idempotency enforcement)
3. Generate purified POSIX sh (safe to re-run, deterministic)

**Why this is valuable**:
- Clean up existing bash scripts automatically
- Remove non-deterministic patterns ($RANDOM, timestamps, process IDs)
- Make operations idempotent (mkdir -p, rm -f, ln -sf)
- Ensure POSIX compliance (passes shellcheck)
- Quote all variables for injection safety

**Current Status**: 70% production-ready (needs docs, examples, performance tuning)

---

### FUTURE WORKFLOW: Rust → Safe Shell (v3.0+ - PLANNED)

**Vision**: Write REAL Rust code, test with standard Rust tooling, then transpile to provably safe, deterministic POSIX shell scripts.

**Current State**:
- ⚠️ Partial stdlib infrastructure (function name mappings exist)
- ❌ Rust parser/analyzer not implemented
- ❌ Rust std → shell implementation incomplete
- ❌ No production examples or tests

**Estimated Work**: 12-16 weeks from current state

This workflow is **deferred to v3.0+** to focus on completing the working Bash purifier.

---

## Workflow 1: Bash → Purified Bash (PRIMARY - WORKING)

### Input: Messy Bash
```bash
#!/bin/bash
# deploy.sh - PROBLEMATIC

# Non-deterministic
SESSION_ID=$RANDOM
RELEASE="release-$(date +%s)"

# Non-idempotent
mkdir /app/releases/$RELEASE
rm /app/current
ln -s /app/releases/$RELEASE /app/current
```

### Step 1: Parse to Rust
```rust
// deploy.rs - Generated from deploy.sh
use std::fs;

fn deploy_app(version: &str) -> Result<(), String> {
    // Purified: deterministic (not $RANDOM)
    let session_id = format!("session-{}", version);

    // Purified: deterministic (not timestamp)
    let release = format!("release-{}", version);

    // Purified: idempotent (mkdir -p)
    let release_dir = format!("/app/releases/{}", release);
    fs::create_dir_all(&release_dir)
        .map_err(|e| e.to_string())?;

    // Purified: idempotent (remove if exists)
    let _ = fs::remove_file("/app/current");

    // Purified: idempotent (symlink)
    std::os::unix::fs::symlink(&release_dir, "/app/current")
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deploy_deterministic() {
        // Same version = same behavior
        assert!(deploy_app("1.0.0").is_ok());
        assert!(deploy_app("1.0.0").is_ok());
    }
}
```

### Step 2: Purified Bash Output
```bash
#!/bin/sh
# Purified by Rash v1.0.0

deploy_app() {
    _version="$1"

    # Deterministic (was $RANDOM)
    session_id="session-${_version}"

    # Deterministic (was timestamp)
    release="release-${_version}"

    # Idempotent (was mkdir)
    release_dir="/app/releases/${release}"
    mkdir -p "${release_dir}" || return 1

    # Idempotent (was rm)
    rm -f "/app/current"

    # Idempotent (was ln -s)
    ln -s "${release_dir}" "/app/current" || return 1
}

deploy_app "$1"
```

**Purification Report**:
```
Issues Fixed: 5
- $RANDOM → version-based ID
- $(date +%s) → fixed release tag
- mkdir → mkdir -p (idempotent)
- rm → rm -f (idempotent)
- ln -s → remove + ln -s (idempotent)

Quality: ✅ Deterministic, ✅ Idempotent, ✅ POSIX
```

---

## Key Point: Bash Purification (Current Focus)

**What Rash DOES (v1.4.0)**:
- ✅ Parses bash scripts to AST
- ✅ Detects non-deterministic patterns ($RANDOM, timestamps, $$)
- ✅ Detects non-idempotent operations (mkdir, rm, ln)
- ✅ Generates purified POSIX sh output
- ✅ Enforces variable quoting for safety
- ✅ Passes shellcheck validation
- ✅ Lints bash scripts (14 rules: 6 DET/IDEM + 8 SEC)

**What Rash WILL DO (v3.0+ - Future)**:
- ⏸️ Parse REAL Rust code
- ⏸️ Map Rust std library to shell
- ⏸️ Transpile Rust → Safe Shell
- ⏸️ Full linter (800+ rules)

**Current Value Proposition**:
Take messy, non-deterministic bash scripts and output safe, deterministic, idempotent POSIX shell scripts.

---

## Development Principles

### EXTREME TDD Definition

**EXTREME TDD** is traditional Test-Driven Development enhanced with comprehensive quality gates:

**Formula**: EXTREME TDD = TDD + Property Testing + Mutation Testing + Fuzz Testing + PMAT + Examples

**Components**:
1. **TDD (RED → GREEN → REFACTOR)**: Write failing test → Implement → Clean up
2. **Property Testing**: Generative tests with 100+ cases (proptest)
3. **Mutation Testing**: Verify test quality with ≥90% kill rate (cargo-mutants)
4. **Fuzz Testing**: Automated input generation to find edge cases (when applicable)
5. **PMAT Quality Gates**: Code complexity (<10), quality score (≥9.0), TDG verification
6. **Example Verification**: `cargo run --example` must pass for all relevant examples

**Evidence of Effectiveness**:
- v6.27.1: Property test caught sh shebang detection bug
- v6.26.0: 4 unit tests + integration tests + property tests
- v6.25.0: 17 comprehensive tests + property tests + mutation tests

**Why EXTREME?**
- Catches bugs traditional TDD misses (property tests found 1 bug today)
- Proves test effectiveness mathematically (mutation testing)
- Validates real-world usage (example verification)
- Ensures code quality objectively (PMAT gates)

### 自働化 (Jidoka) - Build Quality In
- **Bash Purification**: Validate purified output passes shellcheck
- **Test Coverage**: >85% on all modules (currently 1,489 tests passing)
- **Never ship incomplete code**: All purifier outputs must be fully safe

### 現地現物 (Genchi Genbutsu) - Direct Observation
- **Test against real shells**: dash, ash, busybox sh, bash
- **Profile actual scenarios**: Bootstrap installers on Alpine containers
- **Verify purification**: Ensure purified bash behaves identically to original

### 反省 (Hansei) - Fix Before Adding
- **Current priorities** (v2.0 focus):
    1. Bash purification production readiness (70% → 100%)
    2. Real-world examples and documentation
    3. Performance optimization (<100ms for typical scripts)
    4. Security linter expansion (SEC009-SEC045 - deferred to v2.x)

### 改善 (Kaizen) - Continuous Improvement
- **Quality baselines**: All generated shell must pass quality gates
- **Performance**: <100ms transpilation, <10MB memory
- **Test coverage**: >85% on all modules

---

## 🚨 STOP THE LINE Protocol (Andon Cord)

**CRITICAL**: When validating GNU Bash Manual transformations (Workflow 2), **STOP THE LINE** immediately when a bug is discovered.

### When to Pull the Andon Cord

**STOP IMMEDIATELY** if you discover:
1. ❌ **Missing implementation** - Bash construct not parsed correctly
2. ❌ **Incorrect transformation** - Bash→Rust or Rust→Purified output is wrong
3. ❌ **Non-deterministic output** - Purified bash contains $RANDOM, $$, timestamps, etc.
4. ❌ **Non-idempotent output** - Purified bash not safe to re-run (missing -p, -f flags)
5. ❌ **Test failure** - EXTREME TDD test fails (RED without GREEN)
6. ❌ **POSIX violation** - Generated shell fails `shellcheck -s sh`

### STOP THE LINE Procedure

When you find a bug during Bash manual validation:

```
🚨 STOP THE LINE - P0 BUG DETECTED 🚨

1. HALT all validation work
2. Document the bug clearly
3. Create P0 ticket
4. Fix with EXTREME TDD
5. Verify fix with comprehensive testing
6. ONLY THEN resume validation
```

### P0 Ticket Creation

Create ticket following this template:

```markdown
## P0: [Short Description]

**Severity**: P0 - STOP THE LINE
**Category**: [Parser|Transformer|Emitter|Verifier]
**Found During**: GNU Bash Manual validation (Task ID: <TASK-ID>)

### Bug Description
Clear description of what's broken.

### Expected Behavior
- Input: `<bash code>`
- Rust: `<expected rust>`
- Purified: `<expected purified bash>`

### Actual Behavior
- Input: `<bash code>`
- Actual Rust: `<actual rust output>`
- Actual Purified: `<actual purified output>`
- Error: `<error message if any>`

### Reproduction
1. Step-by-step reproduction
2. Minimal test case

### Impact
How this affects Bash manual coverage and workflows.
```

### Fix with EXTREME TDD + PmaT

**MANDATORY FIX PROCESS**:

#### Step 1: RED Phase (Write Failing Test)
```rust
#[test]
fn test_<bug_description>() {
    // ARRANGE: Set up test case from bug report
    let bash_input = "<failing bash code>";

    // ACT: Run transformation
    let result = transform(bash_input);

    // ASSERT: Verify expected behavior
    assert_eq!(result.rust_code, "<expected rust>");
    assert_eq!(result.purified_bash, "<expected purified>");
    assert!(result.is_deterministic());
    assert!(result.is_idempotent());
}
```

Run test: `cargo test test_<bug_description>`
**VERIFY IT FAILS** ❌ (This is RED phase)

#### Step 2: GREEN Phase (Implement Fix)
- Fix the parser/transformer/emitter code
- Run test: `cargo test test_<bug_description>`
- **VERIFY IT PASSES** ✅ (This is GREEN phase)

#### Step 3: REFACTOR Phase
- Clean up implementation
- Extract helper functions if needed
- Ensure complexity <10
- Run full test suite: `cargo test`
- **ALL 808+ TESTS MUST PASS** ✅

#### Step 4: REPL VERIFICATION (Interactive Validation)
**NEW**: Validate feature interactively in REPL to catch integration issues

```bash
# Start bashrs REPL
$ bashrs repl

# Test the bash construct manually
bashrs> x=5
bashrs> echo $x
5

# Test edge cases interactively
bashrs> echo ${x:-default}
5
bashrs> echo ${unset:-fallback}
fallback

# Test complex scenarios
bashrs> for i in 1 2 3; do echo $i; done
1
2
3
```

**Why REPL Verification?**
- Catches integration issues unit tests miss
- Validates real-world interactive usage
- Improves developer confidence in transformations
- Documents REPL-specific edge cases

**Document findings**: Note any REPL-specific issues discovered for future reference

**VERIFY REPL BEHAVIOR MATCHES SPECIFICATION** ✅

#### Step 5: Property Testing
```rust
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_<bug_description>_always_deterministic(
            input in "<regex pattern for valid inputs>"
        ) {
            let result = transform(&input);
            // Verify determinism property
            prop_assert!(result.is_deterministic());
        }
    }
}
```

Run: `cargo test prop_<bug_description>`
**VERIFY PROPERTY HOLDS** ✅

#### Step 6: Mutation Testing
```bash
# Run mutation testing on fixed module
cargo mutants --file src/<module>/<fixed_file>.rs

# TARGET: ≥90% kill rate
```

**VERIFY** mutation score ≥90% ✅

#### Step 7: pmat Verification
**NEW**: Verify code quality with paiml-mcp-agent-toolkit

```bash
# Verify code complexity
$ pmat analyze complexity --max 10
✅ All functions below complexity 10

# Verify overall quality score
$ pmat quality-score --min 9.0
✅ Quality score: 9.5/10

# Verify against specification (if applicable)
$ pmat verify --spec rash.spec --impl target/debug/bashrs
✅ Implementation matches specification
```

**Why pmat Verification?**
- Ensures code complexity stays manageable (<10)
- Verifies overall code quality metrics
- Catches quality issues missed by standard tooling
- Provides objective quality scoring

**Address any issues detected** before proceeding

**VERIFY PMAT QUALITY GATES PASS** ✅

#### Step 8: Example Verification
**NEW**: Verify real-world usage with example programs

```bash
# Find relevant examples
ls examples/*.rs | grep <feature>

# Run example to verify it works
cargo run --example quality_tools_demo
cargo run --example <relevant_example>

# Check example output
# - No panics
# - Expected behavior
# - Good user experience
```

**Why Example Verification?**
- Validates real-world usage patterns
- Catches API usability issues
- Ensures examples stay synchronized with code
- Provides confidence for users

**Examples to verify**:
- `quality_tools_demo` - General quality tools
- `<feature>_example` - Feature-specific examples

**VERIFY EXAMPLES RUN SUCCESSFULLY** ✅

#### Step 9: Integration Testing
```rust
#[test]
fn test_integration_<bug_description>() {
    // End-to-end test: bash → rust → purified → shellcheck
    let bash = "<original bash>";
    let rust = bash_to_rust(bash);
    let purified = rust_to_purified(&rust);

    // Verify shellcheck passes
    assert!(shellcheck_passes(&purified));

    // Verify determinism
    let purified2 = rust_to_purified(&rust);
    assert_eq!(purified, purified2);
}
```

Run: `cargo test test_integration_<bug_description>`
**VERIFY INTEGRATION WORKS** ✅

#### Step 10: Regression Prevention
- Add test to permanent test suite
- Update BASH-INGESTION-ROADMAP.yaml
- Mark task as "completed"
- Document in CHANGELOG.md

### Verification Checklist (EXTREME TDD)

Before resuming validation, verify ALL of these:

- [ ] ✅ **RED**: Failing test written and verified to fail
- [ ] ✅ **GREEN**: Implementation fixed, test passes
- [ ] ✅ **REFACTOR**: Code cleaned up, complexity <10
- [ ] ✅ **REPL VERIFICATION**: Interactive validation completed, behavior matches specification
- [ ] ✅ **All tests pass**: 6021+ tests, 100% pass rate
- [ ] ✅ **Property test**: Property tests pass (100+ generated cases)
- [ ] ✅ **Mutation test**: ≥90% kill rate on fixed module
- [ ] ✅ **pmat VERIFICATION**: Quality gates pass (complexity <10, quality score ≥9.0)
- [ ] ✅ **Example verification**: `cargo run --example` passes for relevant examples
- [ ] ✅ **Integration test**: End-to-end workflow verified
- [ ] ✅ **Shellcheck**: Purified output passes POSIX compliance
- [ ] ✅ **Documentation**: CHANGELOG, roadmap updated
- [ ] ✅ **Ticket closed**: P0 marked as RESOLVED

### Resume Validation

**ONLY AFTER** all checklist items are ✅, you may resume GNU Bash manual validation.

### Example: Real STOP THE LINE Event

```
🚨 STOP THE LINE 🚨

Task: PARAM-POS-001 (Positional parameters $1, $2)
Bug: Parser treats "$1" in double quotes as Literal("$1")
     instead of Variable("1")

Action Taken:
1. HALTED validation work
2. Created test: test_positional_parameters_in_quotes
3. Test FAILED ❌ (RED confirmed)
4. Fixed parser to expand variables in double quotes
5. Test PASSED ✅ (GREEN confirmed)
6. Ran all 6021+ tests: PASSED ✅
7. Added property test (100+ cases): PASSED ✅
8. Mutation test: 92% kill rate ✅
9. Example verification: quality_tools_demo runs ✅
10. Updated CHANGELOG, roadmap

Status: ✅ RESOLVED - Resuming validation
```

---

## Critical Invariants

### Workflow 1 (Bash → Purified Bash) - PRIMARY
1. **Behavioral equivalence**: Purified bash must behave same as original
2. **Determinism**: Remove all $RANDOM, timestamps, process IDs
3. **Idempotency**: Add -p, -f flags for safe re-run (mkdir -p, rm -f, ln -sf)
4. **POSIX compliance**: Every purified script must pass `shellcheck -s sh`
5. **Safety**: All variables quoted to prevent injection
6. **Test coverage**: >85% on all parser/transformer/generator modules

### Workflow 2 (Rust → Shell) - FUTURE (v3.0+)
1. **POSIX compliance**: Every generated script must pass `shellcheck -s sh`
2. **Determinism**: Same Rust input must produce byte-identical shell output
3. **Safety**: No injection vectors in generated scripts
4. **Idempotency**: Operations safe to re-run
5. **Rust std mapping**: Comprehensive std::fs, std::process, std::env coverage

## Verification with paiml-mcp-agent-toolkit
```bash
# Verify bash purification (Workflow 1 - PRIMARY)
pmat verify --spec rash.spec --impl target/debug/bashrs

# Test purified scripts
pmat test --shell-matrix "sh,dash,ash" --input examples/*.sh

# Analyze purified output quality
pmat analyze complexity --max 10
pmat quality-score --min 9.0

# Future: Verify Rust → Shell transpilation (Workflow 2 - v3.0+)
# pmat test --shell-matrix "sh,dash,ash" --input examples/*.rs
```

## Documentation Standards

### Roadmap Format
**CRITICAL**: All project roadmaps MUST be in YAML format (.yaml extension).

**Required Format**:
- ✅ **ONLY YAML**: All roadmaps must use `.yaml` extension
- ❌ **NO MARKDOWN**: Do NOT create `.md` roadmap files
- ✅ **Structured Data**: Use YAML for machine-readable task tracking
- ✅ **Consistency**: All roadmaps follow same schema

**Roadmap Locations**:
- Project roadmap: `ROADMAP.yaml`
- Feature roadmaps: `docs/<feature>-ROADMAP.yaml`
- Sprint roadmaps: Embedded in `ROADMAP.yaml` under `sprints` key

**Required YAML Schema**:
```yaml
roadmap:
  title: "Roadmap Name"
  goal: "Clear objective"
  methodology: "EXTREME TDD"
  status: "IN_PROGRESS|COMPLETE|READY"

  statistics:
    total_tasks: <number>
    completed: <number>
    in_progress: <number>
    coverage_percent: <number>

chapters:  # or sections/tasks depending on structure
  - id: <unique-id>
    name: "Chapter/Section Name"
    tasks:
      - id: "<TASK-ID>"
        title: "Task description"
        status: "pending|in_progress|completed"
        priority: "HIGH|MEDIUM|LOW"
        input: "Example input"
        rust: "Rust transformation"
        purified: "Purified bash output"
        test_name: "test_function_name"
        notes: "Additional context"
```

**Why YAML for Roadmaps**:
1. Machine-readable for automation
2. Easy to query with tools (yq, jq)
3. Structured schema enforcement
4. Integration with CI/CD pipelines
5. Better version control diffs

**Existing Roadmaps**:
- `ROADMAP.yaml` - Main project roadmap
- `docs/BASH-INGESTION-ROADMAP.yaml` - Bash transformation roadmap

---

---

## 🧪 CLI Testing Protocol (MANDATORY)

**CRITICAL**: All CLI testing MUST use `assert_cmd` crate. Using `std::process::Command` directly for CLI testing is a **quality defect**.

### assert_cmd Pattern (Mandatory)

```rust
// MANDATORY: Add to dev-dependencies in Cargo.toml
// [dev-dependencies]
// assert_cmd = "2.0"
// predicates = "3.0"

use assert_cmd::Command;
use predicates::prelude::*;

/// Helper function to create rash command
fn rash_cmd() -> Command {
    Command::cargo_bin("rash").expect("Failed to find rash binary")
}

#[test]
fn test_rash_parse_basic() {
    rash_cmd()
        .arg("parse")
        .arg("examples/hello.sh")
        .assert()
        .success()
        .stdout(predicate::str::contains("AST"));
}

#[test]
fn test_rash_purify_deterministic() {
    rash_cmd()
        .arg("purify")
        .arg("examples/messy.sh")
        .arg("--output")
        .arg("output.sh")
        .assert()
        .success()
        .stdout(predicate::str::contains("Purified"));
}

#[test]
fn test_rash_transpile_rust_to_shell() {
    rash_cmd()
        .arg("transpile")
        .arg("examples/install.rs")
        .assert()
        .success()
        .stdout(predicate::str::contains("#!/bin/sh"));
}
```

**Never Use**: `std::process::Command` for CLI testing. This bypasses cargo's test infrastructure and is considered a quality defect.

### Test Naming Convention (Mandatory)

**Format**: `test_<TASK_ID>_<feature>_<scenario>`

**Examples**:
```rust
// GOOD: Traceability to BASH-INGESTION-ROADMAP.yaml
#[test]
fn test_PARAM_POS_001_positional_params_basic() { }

#[test]
fn test_PARAM_POS_001_positional_params_in_quotes() { }

#[test]
fn test_EXP_PARAM_009_remove_longest_suffix_basic() { }

#[test]
fn test_EXP_PARAM_009_remove_longest_suffix_property() { }

// BAD: No task ID traceability
#[test]
fn test_params() { }  // ❌ Not traceable

#[test]
fn test_expansion() { }  // ❌ Not traceable
```

### Rash CLI Tool Validation Protocol

For every new Rash CLI feature, test with ALL relevant tools:

**Core Tools** (validate EVERY feature):
1. `rash parse <file>` - Parse bash/Makefile to AST
2. `rash purify <file>` - Purify bash/Makefile
3. `rash transpile <file>` - Transpile Rust to shell
4. `rash lint <file>` - Lint bash/Makefile
5. `rash check <file>` - Type-check and validate

**Quality Tools** (validate for production features):
6. `rash ast <file>` - Output AST in JSON format
7. `rash analyze <file>` - Analyze complexity and safety

**Testing Tools** (validate test infrastructure):
8. Property tests (proptest) - 100+ cases per feature
9. Mutation tests (cargo-mutants) - ≥90% kill rate
10. Integration tests - End-to-end workflows

### Complete CLI Test Example (EXTREME TDD)

```rust
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

fn rash_cmd() -> Command {
    Command::cargo_bin("rash").expect("Failed to find rash binary")
}

// RED: Write failing test first
#[test]
fn test_PHONY_001_parse_phony_declarations() {
    // ARRANGE: Create test Makefile with .PHONY
    let makefile = "tests/fixtures/makefiles/phony_test.mk";
    fs::write(makefile, ".PHONY: clean\nclean:\n\trm -f *.o").unwrap();

    // ACT & ASSERT: Parse should succeed
    rash_cmd()
        .arg("parse")
        .arg(makefile)
        .assert()
        .success()
        .stdout(predicate::str::contains("Target"))
        .stdout(predicate::str::contains("phony: true"));

    // Cleanup
    let _ = fs::remove_file(makefile);
}

// GREEN: Implement feature to make test pass
// (Implement MakeItem::Target with phony field)

// REFACTOR: Clean up implementation
// (Extract helper functions, ensure complexity <10)

// PROPERTY TESTING: Add generative tests
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_PHONY_001_prop_phony_preserved(
            target in "[a-z]{1,10}"
        ) {
            let makefile_content = format!(".PHONY: {}\n{}:\n\techo test", target, target);
            let temp = format!("/tmp/test_{}.mk", target);
            fs::write(&temp, &makefile_content).unwrap();

            // Verify parse succeeds
            rash_cmd()
                .arg("parse")
                .arg(&temp)
                .assert()
                .success();

            // Verify purify preserves .PHONY
            rash_cmd()
                .arg("purify")
                .arg(&temp)
                .assert()
                .success()
                .stdout(predicate::str::contains(format!(".PHONY: {}", target)));

            let _ = fs::remove_file(&temp);
        }
    }
}

// DOCUMENTATION: Update MAKE-INGESTION-ROADMAP.yaml
// Mark PHONY-001 as completed
```

### CLI Testing Quality Gates

Before marking any CLI feature as "completed":

- [ ] ✅ **assert_cmd**: All CLI tests use `assert_cmd::Command`
- [ ] ✅ **Test naming**: All tests follow `test_<TASK_ID>_<feature>_<scenario>` convention
- [ ] ✅ **Tool validation**: Feature tested with all relevant Rash CLI tools
- [ ] ✅ **Success cases**: Happy path tests pass
- [ ] ✅ **Error cases**: Error handling tests pass
- [ ] ✅ **Edge cases**: Boundary conditions tested
- [ ] ✅ **Property tests**: Generative tests pass (100+ cases)
- [ ] ✅ **Mutation tests**: ≥90% kill rate on CLI-related code
- [ ] ✅ **Integration tests**: End-to-end CLI workflows verified
- [ ] ✅ **Documentation**: CLI usage documented in README/docs

### CLI Error Handling Pattern

```rust
#[test]
fn test_PARSE_001_invalid_file_error() {
    rash_cmd()
        .arg("parse")
        .arg("nonexistent.sh")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error"))
        .stderr(predicate::str::contains("nonexistent.sh"));
}

#[test]
fn test_PARSE_001_malformed_input_error() {
    let malformed = "tests/fixtures/malformed.sh";
    fs::write(malformed, "if then fi").unwrap();

    rash_cmd()
        .arg("parse")
        .arg(malformed)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Parse error"));

    let _ = fs::remove_file(malformed);
}
```

### CLI Integration Test Pattern

```rust
#[test]
fn test_integration_bash_to_purified_workflow() {
    // ARRANGE: Create messy bash
    let messy_bash = "tests/fixtures/integration/messy_deploy.sh";
    fs::write(messy_bash, r#"
#!/bin/bash
SESSION_ID=$RANDOM
RELEASE="release-$(date +%s)"
mkdir /tmp/releases/$RELEASE
"#).unwrap();

    // ACT: Full workflow - parse → purify → shellcheck

    // Step 1: Parse should succeed
    rash_cmd()
        .arg("parse")
        .arg(messy_bash)
        .assert()
        .success();

    // Step 2: Purify should produce deterministic output
    let purified = "tests/fixtures/integration/purified_deploy.sh";
    rash_cmd()
        .arg("purify")
        .arg(messy_bash)
        .arg("--output")
        .arg(purified)
        .assert()
        .success();

    // Step 3: Verify purified content
    let purified_content = fs::read_to_string(purified).unwrap();
    assert!(!purified_content.contains("$RANDOM"));
    assert!(!purified_content.contains("date +%s"));
    assert!(purified_content.contains("mkdir -p"));

    // Step 4: Shellcheck validation
    Command::new("shellcheck")
        .arg("-s")
        .arg("sh")
        .arg(purified)
        .assert()
        .success();

    // Cleanup
    let _ = fs::remove_file(messy_bash);
    let _ = fs::remove_file(purified);
}
```

---

## Tools

- `cargo test` - Test actual Rust code
- `cargo build` - Build Rust code (before transpilation)
- `cargo clippy` - Lint Rust code
- `cargo llvm-cov` - Measure coverage (we use llvm, not tarpaulin)
- `cargo mutants` - Mutation testing
- `shellcheck` - Validate generated shell output
- `pmat` - Quality analysis with paiml-mcp-agent-toolkit
- `assert_cmd` - **MANDATORY** for all CLI testing

## Quality Standards

All outputs must meet:
- ✅ 100% shellcheck compliance (POSIX)
- ✅ 100% determinism tests pass
- ✅ 100% idempotency tests pass
- ✅ >85% code coverage
- ✅ Complexity <10
- ✅ Mutation score >90% (updated target)
- ✅ Zero defects policy
- ✅ **NEW**: All CLI tests use `assert_cmd`
- ✅ **NEW**: All tests follow `test_<TASK_ID>_<feature>_<scenario>` naming

---

## 📦 Release Protocol (MANDATORY)

**CRITICAL**: Every release MUST be published to both GitHub AND crates.io. Following Toyota Way principles, releasing is NOT complete until both distribution channels are updated.

### Release Checklist

**MANDATORY steps for ALL releases** (major, minor, patch):

#### Phase 1: Quality Verification (STOP THE LINE if any fail)
- [ ] ✅ **All tests pass**: `cargo test --lib` (100% pass rate required)
- [ ] ✅ **Integration tests pass**: All CLI and end-to-end tests
- [ ] ✅ **Clippy clean**: `cargo clippy --all-targets -- -D warnings`
- [ ] ✅ **Format check**: `cargo fmt -- --check`
- [ ] ✅ **No regressions**: All existing features still work
- [ ] ✅ **Shellcheck**: All generated scripts pass `shellcheck -s sh`
- [ ] ✅ **Book updated**: `./scripts/check-book-updated.sh` (enforces book examples pass and book is updated)

#### Phase 2: Documentation (Required before release)
- [ ] ✅ **CHANGELOG.md updated**: Complete release notes with:
  - Version number and date
  - All bug fixes with issue numbers
  - All new features
  - Breaking changes (if any)
  - Migration guide (if needed)
  - Quality metrics (tests passing, coverage, etc.)
- [ ] ✅ **README.md updated**: If new features added
- [ ] ✅ **Version bumped**: Update `Cargo.toml` workspace version
- [ ] ✅ **Book updated**: New features documented in `book/` with tested examples
  - Run `mdbook test book` to verify all examples compile and pass
  - Update relevant chapters (getting-started, concepts, linting, config, etc.)
  - Add new examples for significant features
  - **CRITICAL**: Cannot release without book update (enforced by `./scripts/check-book-updated.sh`)

#### Phase 3: Git Release
- [ ] ✅ **Commit created**: `git add` all changes, create commit with:
  ```bash
  git commit -m "release: v<version> - <brief description>

  <detailed release notes>

  🤖 Generated with Claude Code
  Co-Authored-By: Claude <noreply@anthropic.com>"
  ```
- [ ] ✅ **Git tag created**: Annotated tag with release notes
  ```bash
  git tag -a v<version> -m "v<version> - <description>

  <release notes summary>"
  ```
- [ ] ✅ **Pushed to GitHub**: Both commit and tags
  ```bash
  git push && git push --tags
  ```

#### Phase 4: crates.io Release (MANDATORY - DO NOT SKIP)
- [ ] ✅ **Dry run verification**: Test the publish process
  ```bash
  cargo publish --dry-run
  ```
- [ ] ✅ **Review package contents**: Verify what will be published
  ```bash
  cargo package --list
  ```
- [ ] ✅ **Publish to crates.io**: Actually publish the release
  ```bash
  cargo publish
  ```
- [ ] ✅ **Verify publication**: Check https://crates.io/crates/bashrs
- [ ] ✅ **Test installation**: Verify users can install
  ```bash
  cargo install bashrs --version <version>
  ```

#### Phase 5: Verification (Post-Release)
- [ ] ✅ **GitHub release visible**: Check https://github.com/paiml/bashrs/releases
- [ ] ✅ **crates.io listing updated**: Verify version on crates.io
- [ ] ✅ **Installation works**: Test `cargo install bashrs`
- [ ] ✅ **Documentation builds**: Check docs.rs/bashrs

### Release Types and Versioning

Following [Semantic Versioning](https://semver.org/):

**MAJOR version** (x.0.0) - Breaking changes:
- Incompatible API changes
- Removal of deprecated features
- Major workflow changes
- Example: v1.0.0 → v2.0.0

**MINOR version** (0.x.0) - New features (backward compatible):
- New CLI commands
- New linter rules
- New features without breaking existing code
- Example: v2.0.0 → v2.1.0

**PATCH version** (0.0.x) - Bug fixes only:
- Critical bug fixes (like Issue #1 auto-fix bug)
- Security fixes
- Documentation fixes
- No new features
- Example: v2.0.0 → v2.0.1

### Example: Complete Release Process (v2.0.1)

Following the v2.0.1 release (Issue #1 fix) as reference:

```bash
# Phase 1: Quality Verification
cargo test --lib                    # All 1,545 tests pass ✅
cargo clippy --all-targets         # No warnings ✅
cargo fmt -- --check                # Formatted ✅

# Phase 2: Documentation
# - Updated CHANGELOG.md with Issue #1 fix details ✅
# - Bumped Cargo.toml version 2.0.0 → 2.0.1 ✅

# Phase 3: Git Release
git add CHANGELOG.md Cargo.toml rash/src/linter/rules/*.rs rash/tests/test_issue_001_autofix.rs docs/
git commit -m "fix: v2.0.1 - Critical auto-fix bug (Issue #1)..."
git tag -a v2.0.1 -m "v2.0.1 - Critical Auto-Fix Bug Fix..."
git push && git push --tags         # Pushed to GitHub ✅

# Phase 4: crates.io Release (MANDATORY)
cargo publish --dry-run             # Verify package ✅
cargo package --list                # Review contents ✅
cargo publish                       # Publish to crates.io ✅

# Phase 5: Verification
# - Check GitHub: https://github.com/paiml/bashrs/releases/tag/v2.0.1 ✅
# - Check crates.io: https://crates.io/crates/bashrs ✅
# - Test install: cargo install bashrs --version 2.0.1 ✅
```

### Common Release Mistakes to Avoid

❌ **DO NOT**:
1. Skip crates.io publishing (users won't get the update)
2. Release without updating CHANGELOG.md
3. Release with failing tests
4. Release without testing the package
5. Create release without git tag
6. Push tag before verifying local tests

✅ **ALWAYS**:
1. Publish to BOTH GitHub and crates.io
2. Follow all 5 phases in order
3. Test the package before publishing
4. Update all documentation
5. Verify the release after publishing

### crates.io Publishing Requirements

Before you can publish to crates.io, ensure:

1. **Cargo.toml metadata complete**:
   - `description` field filled
   - `license` specified (MIT)
   - `repository` URL correct
   - `homepage` URL set
   - `keywords` relevant (max 5)
   - `categories` appropriate

2. **crates.io API token configured**:
   ```bash
   cargo login <your-token>
   ```

3. **No local uncommitted changes**:
   ```bash
   git status  # Should be clean
   ```

4. **Version not already published**:
   - Check https://crates.io/crates/bashrs/versions
   - Cannot republish same version

### Release Frequency

**Patch releases** (bug fixes): As needed, within 24-48 hours of critical bugs
**Minor releases** (new features): Monthly or when feature is complete
**Major releases** (breaking changes): Quarterly or when necessary

### Toyota Way Applied to Releases

- **🚨 Jidoka (自働化)**: Build quality into the release process - all tests must pass
- **🔍 Hansei (反省)**: Reflect on what could be improved in release process
- **📈 Kaizen (改善)**: Continuously improve release automation
- **🎯 Genchi Genbutsu (現地現物)**: Verify the release works for real users (test install)

**Remember**: A release is NOT complete until it's available on crates.io. GitHub releases alone are insufficient for Rust projects.

---


## 🌐 WebAssembly (WASM) Development

**Status**: Phase 0 Complete - Feasibility Demonstrated
**Target**: NASA-level quality for WOS and interactive.paiml.com deployment

### WASM Philosophy: Mission-Critical Quality

bashrs WASM provides shell script analysis in browsers for production systems:
- **WOS (Web Operating System)**: Real-time shell script linting
- **interactive.paiml.com**: Educational bash tutorials
- **Production websites**: Config file validation

**Zero tolerance for failure** - users depend on these tools.

### Tools (NEVER use Python!)

**Primary: Ruchy (WASM-optimized HTTP server)**
```bash
# Ruchy is verified by bashrs and optimized for WASM
cd rash/examples/wasm
ruchy serve --port 8000 --watch-wasm

# Why Ruchy?
# ✅ Correct MIME types for .wasm (application/wasm)
# ✅ CORS headers for local development
# ✅ Watch mode for auto-rebuild
# ✅ Verified by bashrs (not Python!)
# ✅ Zero configuration needed
```

**Alternative: Pure Bash**
```bash
# If ruchy unavailable, bash works too
cd rash/examples/wasm
bash -c 'while true; do printf "HTTP/1.1 200 OK\nContent-Type: text/html\n\n$(cat index.html)" | nc -l 8000; done'
```

**❌ NEVER use Python**:
```bash
# ❌ WRONG - Do not use python3 -m http.server
# ❌ WRONG - bashrs doesn't depend on Python
# ✅ RIGHT - Use ruchy or bash
```

### WASM Testing: SQLite + WOS + interactive.paiml.com Standards

**Inspiration**:
- SQLite: 608:1 test-to-code ratio, 100% MC/DC coverage
- WOS Canary Tests: 60 tests, 8-second runtime
- interactive.paiml.com: Real WASM execution testing

**Test Harnesses** (4 required):

1. **Browser Canary Tests** (40 tests)
   - B01-B10: Config Analysis Workflows
   - B11-B20: Streaming I/O Performance
   - B21-B30: Error Handling & Anomalies
   - B31-B40: Cross-Browser Compatibility

2. **Unit Tests** (Rust + wasm-bindgen-test)
   ```rust
   #[wasm_bindgen_test]
   fn test_analyze_config_basic() {
       // Test core logic in WASM
   }
   ```

3. **Property-Based Tests** (Fuzzing)
   ```rust
   proptest! {
       #[test]
       fn prop_analyze_never_panics(config in ".*{0,10000}") {
           // Should never panic on any input
       }
   }
   ```

4. **Mutation Testing** (>90% kill rate)
   ```bash
   cargo mutants --file rash/src/wasm/api.rs
   ```

### Performance Baselines (MANDATORY)

All operations must meet these targets:

| Operation | Target | Test |
|-----------|--------|------|
| WASM load | <5s | B01 |
| Config analysis (1KB) | <100ms | B02-B05 |
| Stream 10MB | <1s, >10 MB/s | B11-B12 |
| Callback latency | <1ms avg | B13 |
| Memory per analysis | <10MB | B14 |

Tests automatically **FAIL** if performance degrades.

### Cross-Browser Matrix (REQUIRED)

| Browser | Version | Tests | Status |
|---------|---------|-------|--------|
| Chromium | Latest | All 40 | Required |
| Firefox | Latest | All 40 | Required |
| WebKit/Safari | Latest | All 40 | Required |
| Mobile Chrome | Latest | B31-B35 | Required |
| Mobile Safari | Latest | B31-B35 | Required |

### Quality Gates

**Before Commit**:
```bash
# Fast canary tests (<2 min)
make wasm-canary

# Verify WASM builds
make wasm-build

# Unit tests
cargo test --lib --features wasm
```

**Before Release**:
```bash
# Full browser matrix (~15 min)
make wasm-canary-all

# Property-based tests
cargo test --lib --features wasm --release -- --include-ignored

# Mutation testing (>90% kill rate required)
make wasm-mutation

# Performance benchmarks
make wasm-bench
```

### Deployment Targets

**1. WOS (Web Operating System)**
- URL: https://wos.paiml.com
- Integration: bashrs as system linter
- Requirements: <5s load, works offline, <1MB binary

**2. interactive.paiml.com**
- URL: https://interactive.paiml.com
- Integration: Real-time shell tutorials
- Requirements: <100ms feedback, educational errors

### WASM Project Structure

```
rash/
├── src/wasm/
│   ├── mod.rs        # Module architecture
│   ├── api.rs        # JavaScript API (analyze, purify, version)
│   ├── streaming.rs  # Streaming I/O benchmarks
│   ├── config.rs     # Config re-exports
│   └── filesystem.rs # Virtual filesystem (Phase 1)
├── examples/wasm/
│   ├── index.html       # Browser demo
│   ├── README.md        # Building instructions
│   ├── TESTING-SPEC.md  # NASA-level testing spec
│   ├── PHASE0-RESULTS.md # Feasibility results
│   └── pkg/             # Compiled WASM (generated)
└── .cargo/
    └── config.toml   # WASM build configuration
```

### Building WASM

```bash
# Install wasm-pack (first time only)
cargo install wasm-pack

# Build for web
cd rash
wasm-pack build --target web --features wasm

# Output: pkg/bashrs_bg.wasm (960KB)

# Serve with ruchy
cd examples/wasm
ruchy serve --port 8000 --watch-wasm
```

### WASM Testing Commands

```bash
# Development (fast)
make wasm-canary              # Chromium only (~2 min)
make wasm-canary-fast         # Config tests only (~1 min)

# Pre-release (comprehensive)
make wasm-canary-all          # All browsers (~15 min)
make wasm-canary-chromium     # Chromium only
make wasm-canary-firefox      # Firefox only
make wasm-canary-webkit       # WebKit only

# Debugging
make wasm-canary-headed       # Visible browser
make wasm-canary-ui           # Playwright UI mode
make wasm-canary-debug        # Debugger attached

# Reports
make wasm-canary-report       # HTML test report
```

### Anomaly Testing (CRITICAL)

WASM must handle **all** failure modes gracefully:

1. **Memory Anomalies**: OOM during analysis
2. **Storage Anomalies**: localStorage full/corrupted
3. **Network Anomalies**: WASM load failure
4. **Browser Anomalies**: Tab suspension, page reload
5. **Input Anomalies**: Malformed configs, huge files

**Every anomaly must have a test** - no exceptions.

### Documentation Requirements

Every WASM feature MUST have:

1. ✅ **API Documentation** (rustdoc)
2. ✅ **Browser Demo** (examples/wasm/*)
3. ✅ **E2E Tests** (40+ canary tests)
4. ✅ **Performance Benchmarks**
5. ✅ **Integration Guide** (WOS + interactive.paiml.com)
6. ✅ **Troubleshooting Guide**

### WASM Phases

**Phase 0** (COMPLETE): Feasibility Study
- ✅ WASM builds successfully
- ✅ Config analysis works (CONFIG-001 to CONFIG-004)
- ✅ Basic browser demo
- ⏳ Streaming benchmarks (pending browser testing)
- ⏳ Go/No-Go decision (pending performance validation)

**Phase 1** (FUTURE): Production Ready
- [ ] All 40 canary tests pass
- [ ] Cross-browser compatibility validated
- [ ] Performance baselines met
- [ ] Integrated with WOS
- [ ] Integrated with interactive.paiml.com
- [ ] Zero defects in production

**Phase 2** (FUTURE): Advanced Features
- [ ] Offline support (Service Worker)
- [ ] Incremental analysis
- [ ] Syntax highlighting integration
- [ ] LSP server in WASM

### Resources

- **WASM Spec**: `rash/examples/wasm/TESTING-SPEC.md`
- **Phase 0 Results**: `rash/examples/wasm/PHASE0-RESULTS.md`
- **WOS Canary Tests**: `/home/noahgift/src/wos/e2e/tests/canary/README.md`
- **interactive.paiml.com**: `/home/noahgift/src/interactive.paiml.com/tests/wasm/`
- **SQLite Testing**: https://sqlite.org/testing.html

### Critical Reminders

1. **❌ NEVER use Python** - Use ruchy or bash
2. **✅ ALWAYS run canary tests** before commit
3. **✅ ALWAYS test cross-browser** before release
4. **✅ ALWAYS verify performance** baselines
5. **✅ ALWAYS handle anomalies** gracefully
6. **✅ ALWAYS document** new features

**WASM is mission-critical** - users depend on it. NASA-level quality is non-negotiable.

---
