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

Transforms non-deterministic bash ($RANDOM, timestamps) into safe, idempotent POSIX sh with -p/-f flags.

---


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


### Quality Targets
- Test coverage >85%, complexity <10
- Purified scripts pass shellcheck
- Performance: <100ms transpilation, <10MB memory

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
**CRITICAL**: All roadmaps MUST be YAML (.yaml), not markdown. See `ROADMAP.yaml` and `docs/BASH-INGESTION-ROADMAP.yaml`.

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

**Never Use**: `std::process::Command` for CLI testing.

### Test Naming Convention
**Format**: `test_<TASK_ID>_<feature>_<scenario>` (e.g. `test_PARAM_POS_001_positional_params_basic`)

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

## PMAT Integration (v2.200.0+)

**Status**: Fully integrated with paiml-mcp-agent-toolkit v2.200.0

### Quality Scoring

```bash
# Rust project quality score (0-134 scale)
pmat rust-project-score --path . --verbose
# Current: 127.0/134 (94.8%, Grade A+)

# Repository health score (0-100 scale)
pmat repo-score --path .
# Current: 84.5/100 (Grade B+)
```

### Pre-commit Hooks

**MANDATORY**: Pre-commit hooks are installed for automatic quality enforcement.

```bash
# Install pre-commit hooks (PMAT-managed)
pmat hooks install

# Check hook status
pmat hooks status

# View hook configuration
cat .git/hooks/pre-commit
```

**Pre-commit Quality Gates**:
- Complexity analysis (max cyclomatic: 30, cognitive: 25)
- SATD (Self-Admitted Technical Debt) detection
- Auto-enforced on every commit

### Workflow Prompts

PMAT provides pre-configured workflow prompts for EXTREME TDD development:

```bash
# List available workflow prompts
pmat prompt show --help

# Code coverage enforcement (>85% target)
pmat prompt show code-coverage --format text

# Debug with Five Whys root cause analysis
pmat prompt show debug --format text

# Quality gate enforcement (12 gates)
pmat prompt show quality-enforcement --format text

# Security audit and fixes
pmat prompt show security-audit --format text
```

### Code Quality Analysis

```bash
# Complexity analysis
pmat analyze complexity --path rash/src/

# Technical debt grading (TDG)
pmat analyze tdg --path rash/src/

# Self-admitted technical debt (SATD)
pmat analyze satd --path .

# Dead code detection
pmat analyze dead-code --path .
```

### Quality Gates Configuration

Workspace lints are configured in `Cargo.toml`:

```toml
[workspace.lints.rust]
unsafe_op_in_unsafe_fn = "deny"
unreachable_pub = "warn"
missing_docs = "warn"
rust_2018_idioms = "warn"

[workspace.lints.clippy]
# Cloudflare-class defect prevention (2025-11-18 outage)
unwrap_used = { level = "deny", priority = 1 }
expect_used = { level = "warn", priority = 1 }
checked_conversions = "warn"
dbg_macro = "warn"
todo = "warn"
unimplemented = "warn"
```

### Known Issues (Tracked)

**CRITICAL** (Cloudflare-class defect):
- 289 unwrap() calls in production code (detected by rust-project-score)
- Workspace lints now enforce `unwrap_used = "deny"`
- Long-term remediation tracked in separate issue
- See Cloudflare outage 2025-11-18: unwrap() panic caused 3+ hour outage

**Dependency Health**:
- 2 unmaintained dependencies (fxhash, instant) via pforge-runtime
- Non-critical warnings, no security vulnerabilities

### Test Coverage Metrics (2025-11-21)

**Current Coverage**: **91.22%** (exceeds 85% target) ✅

```bash
# Run coverage analysis
make coverage

# Results (7632 tests, 7631 passed):
# - Total Lines: 95,758
# - Covered Lines: 87,351
# - Coverage: 91.22%
# - Functions: 95.85% coverage
# - Regions: 90.66% coverage
```

**Coverage by Category**:
- Core linter rules: 95-100% (excellent)
- REPL modules: 90-99% (excellent)
- Parser modules: 80-90% (good)
- Test infrastructure: 95-100% (excellent)
- Low coverage areas (needs improvement):
  - `bash_parser/semantic.rs`: 47.06% (complex semantic analysis)
  - `test_generator/core.rs`: 54.84% (test generation logic)
  - `wasm/api.rs`: 55.46% (WASM bindings, Phase 0)

### Recent Improvements (2025-11-21)

- ✅ **Workspace lints configured**: Rust and Clippy lints enabled
- ✅ **Pre-commit hooks installed**: PMAT-managed automatic quality checks
- ✅ **Quality score improved**: 118.0/134 → 127.0/134 (+9 points, +6.7%)
- ✅ **Grade improved**: A+ (88.1%) → A+ (94.8%)
- ✅ **Tooling category**: 52.5/130 → 61.5/130 (+9 points, +6.9%)
- ✅ **Coverage measured**: 91.22% line coverage (exceeds 85% target)
- ✅ **Coverage tooling**: cargo-llvm-cov v0.6.21 operational

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

### 📅 Release Schedule (Friday-Only Policy)

**MANDATORY**: crates.io releases MUST happen on **Fridays ONLY**.

**Why Friday releases?**
1. **Weekend buffer**: Issues can be addressed over the weekend if needed
2. **User flexibility**: Users have time to upgrade without weekday pressure
3. **Team availability**: Full team available early in week to handle feedback
4. **Predictable cadence**: Users know when to expect updates
5. **Quality assurance**: Allows full week of testing before release

**Release Preparation Schedule**:
- **Monday-Thursday**: Development, testing, documentation updates
- **Thursday EOD**: All quality gates must pass, documentation complete
- **Friday morning**: Final verification, create tags, publish to crates.io
- **Friday afternoon**: Post-release verification, monitor for issues

**Exceptions** (Require explicit approval):
- **CRITICAL security fixes**: May be released any day with documented justification
- **Zero-day vulnerabilities**: Immediate release with STOP THE LINE protocol
- **User-blocking bugs**: P0 issues affecting production deployments

**If today is NOT Friday**:
```bash
# Prepare but DO NOT publish to crates.io
git tag -a v<version> -m "Release notes..."
git push && git push --tags  # ✅ OK: Push to GitHub
cargo publish --dry-run      # ✅ OK: Verify package
# ❌ DO NOT RUN: cargo publish (wait until Friday!)
```

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

**⚠️ FRIDAY-ONLY**: crates.io releases MUST happen on **Fridays ONLY** (see Release Schedule above).

- [ ] ✅ **Verify it's Friday**: Check current day of week before proceeding
- [ ] ✅ **Dry run verification**: Test the publish process
  ```bash
  cargo publish --dry-run
  ```
- [ ] ✅ **Review package contents**: Verify what will be published
  ```bash
  cargo package --list
  ```
- [ ] ✅ **Publish to crates.io**: Actually publish the release (**Friday morning only**)
  ```bash
  # ⚠️ ONLY RUN ON FRIDAY ⚠️
  cargo publish
  cargo publish -p bashrs-runtime  # If multi-crate workspace
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

### Release Types (SemVer)
- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes only

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

### WASM Testing Standards

**Test Harnesses** (4 required):
1. Browser Canary Tests (40 tests): B01-B40 covering analysis, I/O, errors, cross-browser
2. Unit Tests: `#[wasm_bindgen_test]`
3. Property-Based Tests: Fuzzing with proptest
4. Mutation Testing: >90% kill rate

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
Test all failure modes: OOM, storage full, network failure, tab suspension, malformed input. Every anomaly must have a test.

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

### Critical WASM Reminders
- ❌ NEVER use Python (use ruchy or bash)
- ✅ Run canary tests before commit
- ✅ Test cross-browser before release
- ✅ Verify performance baselines
- ✅ Handle all anomalies gracefully

---
