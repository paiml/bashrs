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

#### Step 4: Property Testing
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

#### Step 5: Mutation Testing
```bash
# Run mutation testing on fixed module
cargo mutants --file src/<module>/<fixed_file>.rs

# TARGET: ≥90% kill rate
```

**VERIFY** mutation score ≥90% ✅

#### Step 6: Integration Testing
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

#### Step 7: Regression Prevention
- Add test to permanent test suite
- Update BASH-INGESTION-ROADMAP.yaml
- Mark task as "completed"
- Document in CHANGELOG.md

### Verification Checklist

Before resuming validation, verify ALL of these:

- [ ] ✅ **RED**: Failing test written and verified to fail
- [ ] ✅ **GREEN**: Implementation fixed, test passes
- [ ] ✅ **REFACTOR**: Code cleaned up, complexity <10
- [ ] ✅ **All tests pass**: 808+ tests, 100% pass rate
- [ ] ✅ **Property test**: Determinism/idempotency verified
- [ ] ✅ **Mutation test**: ≥90% kill rate on fixed module
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
6. Ran all 808 tests: PASSED ✅
7. Added property test: PASSED ✅
8. Mutation test: 92% kill rate ✅
9. Updated CHANGELOG, roadmap

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

