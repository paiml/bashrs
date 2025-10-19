# Sprint 74: Linter Enhancement - Makefile Support + v2.0.0 Release

**Date**: 2025-10-19
**Goal**: Add Makefile-specific linting rules and prepare v2.0.0 release
**Status**: 🚀 **READY TO START**
**Estimated Duration**: 4-6 hours

---

## Executive Summary

Enhance the bashrs linter with Makefile-specific rules to complement the existing shell script linting capabilities. This sprint completes the linter feature set needed for v2.0.0 release.

**Current Linter Status**:
- ✅ 127 linter tests passing (100%)
- ✅ 3 ShellCheck rules (SC2046, SC2086, SC2116)
- ✅ 3 Determinism rules (DET001-003)
- ✅ 3 Idempotency rules (IDEM001-003)
- ✅ 8 Security rules (SEC001-008)
- ✅ Auto-fix capability (100% success rate)

**Gap**: No Makefile-specific lint rules yet

---

## Sprint Goal

Add **Makefile-specific linter rules** that integrate with the existing Makefile parser and purification infrastructure from Sprints 67-69.

### Success Criteria

- [ ] 5+ new Makefile lint rules (MAKE001-MAKE005)
- [ ] 100% test coverage on new rules
- [ ] Integration with `bashrs make lint` command
- [ ] Auto-fix suggestions for fixable issues
- [ ] All 1,444+ tests still passing (zero regressions)
- [ ] Documentation complete
- [ ] Ready for v2.0.0 release

---

## Makefile Lint Rules to Implement

### MAKE001: Non-Deterministic Wildcard Usage

**Description**: Detect unordered `$(wildcard)` usage

**Bad**:
```makefile
SOURCES = $(wildcard *.c)
```

**Good**:
```makefile
SOURCES = $(sort $(wildcard *.c))
```

**Auto-fix**: Wrap with `$(sort ...)`

**Priority**: HIGH (determinism critical)

---

### MAKE002: Non-Idempotent Directory Creation

**Description**: Detect `mkdir` without `-p` flag in recipes

**Bad**:
```makefile
build:
\tmkdir build
```

**Good**:
```makefile
build:
\tmkdir -p build
```

**Auto-fix**: Add `-p` flag

**Priority**: HIGH (idempotency critical)

---

### MAKE003: Unsafe Variable Expansion in Recipes

**Description**: Detect unquoted variables in shell commands

**Bad**:
```makefile
install:
\trm -rf $BUILD_DIR
```

**Good**:
```makefile
install:
\trm -rf "$BUILD_DIR"
```

**Auto-fix**: Add quotes around variable

**Priority**: HIGH (security critical)

---

### MAKE004: Missing .PHONY Declaration

**Description**: Detect targets that should be .PHONY

**Bad**:
```makefile
clean:
\trm -f *.o

test:
\tpytest tests/
```

**Good**:
```makefile
.PHONY: clean test

clean:
\trm -f *.o

test:
\tpytest tests/
```

**Auto-fix**: Add `.PHONY: clean test` at beginning

**Priority**: MEDIUM (best practice)

---

### MAKE005: Recursive Variable Assignment

**Description**: Detect `:=` vs `=` issues

**Bad**:
```makefile
# Infinite recursion if VERSION not set externally
VERSION = $(shell git describe)-$(VERSION)
```

**Good**:
```makefile
# Immediate expansion prevents recursion
VERSION := $(shell git describe)
```

**Auto-fix**: Suggest `:=` for variables using `$(shell ...)`

**Priority**: MEDIUM (robustness)

---

## Implementation Plan

### Phase 1: Rule Implementation (2-3 hours)

**Step 1: Create Rule Modules**

Create 5 new rule files in `rash/src/linter/rules/`:
- `make001.rs` - Wildcard detection
- `make002.rs` - mkdir detection
- `make003.rs` - Variable quoting
- `make004.rs` - .PHONY detection
- `make005.rs` - Recursive assignment

**Step 2: Integrate with Makefile Parser**

Update `rash/src/linter/rules/mod.rs`:
```rust
// Makefile-specific rules
pub mod make001;
pub mod make002;
pub mod make003;
pub mod make004;
pub mod make005;

/// Lint a Makefile and return all diagnostics
pub fn lint_makefile(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Parse Makefile to AST
    match crate::make_parser::parser::parse_makefile(source) {
        Ok(items) => {
            // Run Makefile-specific rules
            result.merge(make001::check(&items));
            result.merge(make002::check(&items));
            result.merge(make003::check(&items));
            result.merge(make004::check(&items));
            result.merge(make005::check(&items));
        }
        Err(e) => {
            // Add parse error diagnostic
            result.add_error(format!("Failed to parse Makefile: {}", e));
        }
    }

    result
}
```

**Step 3: Write Tests (EXTREME TDD)**

For each rule, create comprehensive tests:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    // RED: Write failing test
    #[test]
    fn test_MAKE001_detects_unordered_wildcard() {
        let makefile = "SOURCES = $(wildcard *.c)";
        let result = check(makefile);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "MAKE001");
    }

    // GREEN: Implement rule to pass test

    // Test auto-fix
    #[test]
    fn test_MAKE001_provides_fix() {
        let makefile = "SOURCES = $(wildcard *.c)";
        let result = check(makefile);
        assert!(result.diagnostics[0].fix.is_some());
        assert_eq!(
            result.diagnostics[0].fix.as_ref().unwrap().suggestion,
            "SOURCES = $(sort $(wildcard *.c))"
        );
    }

    // Test no false positives
    #[test]
    fn test_MAKE001_no_warning_already_sorted() {
        let makefile = "SOURCES = $(sort $(wildcard *.c))";
        let result = check(makefile);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
```

**Target**: 25+ tests (5 rules × 5 tests each)

---

### Phase 2: CLI Integration (1 hour)

**Step 1: Add `bashrs make lint` Command**

Update `rash/src/cli/args.rs`:
```rust
pub enum MakeCommand {
    Parse { ... },
    Purify { ... },
    /// Lint a Makefile for issues
    Lint {
        /// Makefile to lint
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Output format
        #[arg(long, value_enum, default_value = "pretty")]
        format: OutputFormat,

        /// Apply auto-fixes
        #[arg(long)]
        fix: bool,
    },
}
```

**Step 2: Implement CLI Handler**

Update `rash/src/cli/commands.rs`:
```rust
pub fn handle_make_lint(file: &Path, format: OutputFormat, fix: bool) -> Result<()> {
    // Read Makefile
    let source = fs::read_to_string(file)?;

    // Lint Makefile
    let result = lint_makefile(&source);

    // Output diagnostics
    match format {
        OutputFormat::Pretty => print_pretty(&result),
        OutputFormat::Json => print_json(&result),
        OutputFormat::Sarif => print_sarif(&result),
    }

    // Apply fixes if requested
    if fix && result.has_fixable_issues() {
        apply_fixes(file, &result)?;
    }

    // Exit with error code if issues found
    if result.has_errors() {
        std::process::exit(1);
    }

    Ok(())
}
```

**Step 3: Add CLI Tests**

Create `rash/tests/cli_make_lint_tests.rs`:
```rust
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

fn make_cmd() -> Command {
    Command::cargo_bin("bashrs").unwrap()
}

#[test]
fn test_make_lint_detects_wildcard_issue() {
    let makefile = "tests/fixtures/makefiles/unordered_wildcard.mk";
    fs::write(makefile, "SOURCES = $(wildcard *.c)").unwrap();

    make_cmd()
        .arg("make")
        .arg("lint")
        .arg(makefile)
        .assert()
        .failure()
        .stdout(predicate::str::contains("MAKE001"));

    let _ = fs::remove_file(makefile);
}

#[test]
fn test_make_lint_auto_fix() {
    let makefile = "tests/fixtures/makefiles/fixable.mk";
    fs::write(makefile, "SOURCES = $(wildcard *.c)").unwrap();

    make_cmd()
        .arg("make")
        .arg("lint")
        .arg("--fix")
        .arg(makefile)
        .assert()
        .success();

    let fixed = fs::read_to_string(makefile).unwrap();
    assert!(fixed.contains("$(sort $(wildcard"));

    let _ = fs::remove_file(makefile);
}
```

**Target**: 10+ CLI integration tests

---

### Phase 3: Documentation (30 minutes)

**Step 1: Rule Documentation**

Create `docs/linter/makefile-rules.md`:
```markdown
# Makefile Linter Rules

## MAKE001: Non-Deterministic Wildcard

**Description**: ...
**Examples**: ...
**Auto-fix**: ...

## MAKE002: Non-Idempotent mkdir
...
```

**Step 2: Update README**

Add Makefile linting example:
```bash
# Lint Makefile
bashrs make lint Makefile

# Auto-fix issues
bashrs make lint --fix Makefile

# JSON output for CI/CD
bashrs make lint --format json Makefile
```

**Step 3: Update CHANGELOG**

Add Sprint 74 achievements:
```markdown
## [2.0.0] - 2025-10-19

### Added
- Makefile-specific linter rules (MAKE001-MAKE005)
- `bashrs make lint` command with auto-fix support
- 35+ new linter tests
- Enhanced error messages from Sprint 73
```

---

### Phase 4: Testing & Validation (1 hour)

**Step 1: Run Full Test Suite**
```bash
cargo test --lib
# Target: 1,479+ tests passing (1,444 + 35 new)
```

**Step 2: Linter-Specific Tests**
```bash
cargo test --lib linter
# Target: 162+ tests passing (127 + 35 new)
```

**Step 3: CLI Integration Tests**
```bash
cargo test cli_make_lint
# Target: 10+ tests passing
```

**Step 4: Manual Validation**
```bash
# Create test Makefile with issues
cat > test.mk << 'EOF'
SOURCES = $(wildcard *.c)

build:
\tmkdir build
\tgcc $(SOURCES) -o app

clean:
\trm -rf build
EOF

# Lint it
bashrs make lint test.mk

# Expected output:
# MAKE001: Non-deterministic wildcard at line 1
# MAKE002: Non-idempotent mkdir at line 4
# MAKE004: Missing .PHONY for 'clean' target

# Auto-fix
bashrs make lint --fix test.mk

# Verify fixed
cat test.mk
```

---

## Quality Metrics

### Test Coverage Target

- **New code coverage**: >85%
- **Overall coverage**: >88% (maintain current level)
- **Linter module coverage**: >90%

### Complexity Target

- All functions <10 cognitive complexity
- Rule functions should be 2-5 complexity

### Mutation Testing

- Run mutation tests on new rules: `cargo mutants --file rash/src/linter/rules/make*.rs`
- Target: ≥90% kill rate

---

## Integration with Existing Infrastructure

### Leverage Sprint 67-69 Work

**Sprint 67-69 delivered**:
- ✅ Makefile parser (`rash/src/make_parser/parser.rs`)
- ✅ Makefile AST (`rash/src/make_parser/ast.rs`)
- ✅ Semantic analysis (`rash/src/make_parser/semantic.rs`)
- ✅ Purification logic (`rash/src/make_parser/purify.rs`)

**Sprint 74 builds on this**:
- Reuse AST parsing for linting
- Reuse semantic analysis functions:
  - `detect_wildcard()` → MAKE001
  - Already in place for purification
- Create feedback loop:
  - **Lint** → Detect issues
  - **Purify** → Auto-fix issues
  - **Lint again** → Verify fixes

### Synergy with Sprint 73

**Sprint 73 delivered**:
- ✅ Enhanced error handling for Make parser
- ✅ Quality error messages with code snippets
- ✅ Source location tracking

**Sprint 74 benefits**:
- Use enhanced errors for linter diagnostics
- Show code snippets in lint warnings
- Precise source location for fixes

---

## v2.0.0 Release Checklist

### Pre-Release

- [ ] Sprint 73 Phase 6 complete (mutation testing)
- [ ] Sprint 74 complete (Makefile linter)
- [ ] All tests passing (1,479+)
- [ ] Coverage >88%
- [ ] Complexity <10
- [ ] No clippy warnings

### Release Steps

1. **Update Version**:
   ```bash
   # Update Cargo.toml
   sed -i 's/version = "1.5.0"/version = "2.0.0"/' rash/Cargo.toml
   ```

2. **Update CHANGELOG.md**:
   - Sprint 73 achievements (error handling)
   - Sprint 74 achievements (Makefile linter)
   - Breaking changes (if any)

3. **Tag Release**:
   ```bash
   git add -A
   git commit -m "Release v2.0.0: Makefile linter + error handling"
   git tag -a v2.0.0 -m "Release v2.0.0"
   git push origin main --tags
   ```

4. **Publish to crates.io**:
   ```bash
   cd rash
   cargo publish
   ```

5. **Create GitHub Release**:
   - Go to https://github.com/noahgift/bashrs/releases
   - Create release from v2.0.0 tag
   - Copy CHANGELOG section
   - Attach binaries (optional)

---

## Timeline

**Total Duration**: 4-6 hours

- **Phase 1** (Rule Implementation): 2-3 hours
  - MAKE001-005 implementation
  - 25+ tests written
  - All tests passing

- **Phase 2** (CLI Integration): 1 hour
  - `bashrs make lint` command
  - 10+ CLI tests
  - Integration complete

- **Phase 3** (Documentation): 30 minutes
  - Rule docs
  - README updates
  - CHANGELOG

- **Phase 4** (Testing & Validation): 1 hour
  - Full test suite
  - Manual validation
  - Quality checks

- **Phase 5** (Release): 30 minutes
  - Version bump
  - Tag and publish
  - GitHub release

---

## Success Metrics

### Code Metrics

- **Tests added**: 35+ (25 unit + 10 integration)
- **Total tests**: 1,479+ (from 1,444)
- **Test pass rate**: 100%
- **Code added**: ~600 lines rules + ~400 lines tests
- **Documentation**: ~200 lines

### Quality Metrics

- **Code coverage**: >88% (maintain)
- **Linter coverage**: >90% (new target)
- **Complexity**: <10 all functions
- **Mutation kill rate**: ≥90% (new rules)
- **Zero regressions**: All existing tests pass

### Feature Completeness

- [ ] 5 new Makefile rules
- [ ] Auto-fix for all fixable issues
- [ ] CLI integration complete
- [ ] Documentation complete
- [ ] v2.0.0 released

---

## Risk Assessment

### Risks: **VERY LOW**

**Rationale**:
1. ✅ Makefile parser already exists and tested (Sprint 67-69)
2. ✅ Linter infrastructure exists (Sprint 1, 70)
3. ✅ Clear patterns from existing rules
4. ✅ Comprehensive test suite to catch regressions

### Mitigation

- Use EXTREME TDD (RED-GREEN-REFACTOR)
- Run full test suite after each phase
- Manual validation before release

---

## Next Steps (Immediate)

1. **Start Phase 1**: Implement MAKE001 rule
   - Write failing test
   - Implement rule
   - Add auto-fix
   - Verify test passes

2. **Continue to MAKE002-005**: Repeat for each rule

3. **Phase 2**: CLI integration

4. **Phase 3**: Documentation

5. **Phase 4**: Validation

6. **Phase 5**: v2.0.0 Release

---

**Prepared by**: Claude (AI Assistant)
**Date**: 2025-10-19
**Methodology**: EXTREME TDD + 反省 (Hansei) + 改善 (Kaizen)
**Sprint**: 74 - Makefile Linter Enhancement
**Target Release**: v2.0.0
**Status**: 🚀 READY TO START
**Confidence**: **VERY HIGH** - Building on proven infrastructure
