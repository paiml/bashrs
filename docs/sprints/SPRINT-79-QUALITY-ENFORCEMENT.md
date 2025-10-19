# Sprint 79: Quality Enforcement + Dogfooding + Book TDD

**Status**: PLANNING
**Goal**: Enforce quality with pmat integration, dogfood bashrs on itself, update book religiously with TDD
**Methodology**: EXTREME TDD + Property/Mutation/Fuzz Testing
**Outcome**: v2.0.1 release with production-grade quality enforcement

---

## 🎯 Objectives

### A. Use pmat for Quality Enforcement (pmat Integration)
Integrate paiml-mcp-agent-toolkit patterns from `../paiml-mcp-agent-toolkit`:
- `lint-scripts`: Lint all shell scripts with bashrs
- `lint-makefile`: Lint Makefile with bashrs
- `validate-docs`: Validate book accuracy
- Quality gates in Makefile

### B. Dogfood bashrs on Own Project
Apply bashrs linters to bashrs itself:
- Fix all Makefile linting issues
- Fix all shell script linting issues
- Enforce quality in CI/CD

### C. Update Book Religiously with TDD
Follow ruchy pattern for book accuracy:
- Every code example must compile/run
- Test all examples before committing
- Automated validation in pre-commit hooks

---

## 📋 Tasks Breakdown

### Phase 1: pmat Integration (Quality Infrastructure)

#### TASK 1.1: Add lint-scripts Target to Makefile
**Input**: Current Makefile (no script linting)
**Output**: Makefile with `lint-scripts` target using bashrs

**Test (RED)**:
```rust
#[test]
fn test_SPRINT79_001_lint_scripts_target_exists() {
    let makefile = std::fs::read_to_string("Makefile").unwrap();
    assert!(makefile.contains("lint-scripts:"));
}
```

**Implementation (GREEN)**:
```makefile
.PHONY: lint-scripts
lint-scripts:
	@echo "🔍 Linting shell scripts..."
	@find scripts -name "*.sh" -type f -exec bashrs lint {} \;
	@echo "✅ All scripts linted!"
```

---

#### TASK 1.2: Add lint-makefile Target
**Input**: Current Makefile
**Output**: Self-linting Makefile target

**Test (RED)**:
```rust
#[test]
fn test_SPRINT79_002_lint_makefile_target() {
    let makefile = std::fs::read_to_string("Makefile").unwrap();
    assert!(makefile.contains("lint-makefile:"));
}
```

**Implementation (GREEN)**:
```makefile
.PHONY: lint-makefile
lint-makefile:
	@echo "🔍 Linting Makefile..."
	@bashrs make lint Makefile --format human
	@echo "✅ Makefile linted!"
```

---

#### TASK 1.3: Add validate-book Target
**Input**: Current book validation (Sprint 78)
**Output**: Integrated validate-book in Makefile

**Test (RED)**:
```rust
#[test]
fn test_SPRINT79_003_validate_book_in_makefile() {
    let makefile = std::fs::read_to_string("Makefile").unwrap();
    assert!(makefile.contains("validate-book:"));
}
```

**Implementation (GREEN)**:
```makefile
.PHONY: validate-book
validate-book:
	@echo "📖 Validating book accuracy..."
	@./scripts/validate-book.sh
	@echo "✅ Book validated!"
```

---

#### TASK 1.4: Add Quality Gate Target (validate-all)
**Input**: Individual quality targets
**Output**: Comprehensive `validate` target

**Test (RED)**:
```rust
#[test]
fn test_SPRINT79_004_validate_all_target() {
    let makefile = std::fs::read_to_string("Makefile").unwrap();
    assert!(makefile.contains("validate:"));
    assert!(makefile.contains("lint-scripts"));
    assert!(makefile.contains("lint-makefile"));
    assert!(makefile.contains("validate-book"));
}
```

**Implementation (GREEN)**:
```makefile
.PHONY: validate
validate: lint-scripts lint-makefile validate-book test
	@echo "✅ All quality gates passed!"
	@echo "  ✓ Shell scripts linted"
	@echo "  ✓ Makefile linted"
	@echo "  ✓ Book accuracy validated"
	@echo "  ✓ Tests passing"
```

---

### Phase 2: Dogfooding (Fix Our Own Issues)

#### TASK 2.1: Fix quality-gates.sh Issues
**Current State**: 63 warnings, 1 info (from dogfooding)
**Target**: <10 warnings

**Issues to Fix**:
- SC2086: Unquoted variables (`${NC}`, `$prop_count`, etc.)
- IDEM002: Non-idempotent `rm` → `rm -f`
- SC2046: Unquoted command substitutions

**Test (RED)**:
```rust
#[test]
fn test_SPRINT79_005_quality_gates_clean() {
    let output = Command::new("bashrs")
        .arg("lint")
        .arg("scripts/quality-gates.sh")
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    let warnings = stderr.matches("warning").count();

    assert!(warnings < 10, "Too many warnings: {}", warnings);
}
```

**Implementation (GREEN)**: Fix all quoting issues in quality-gates.sh

---

#### TASK 2.2: Fix validate-book.sh False Positives
**Current State**: 5 warnings (backticks in strings - false positives)
**Target**: 0 warnings (or document as acceptable)

**Test (RED)**:
```rust
#[test]
fn test_SPRINT79_006_validate_book_sh_clean() {
    let output = Command::new("bashrs")
        .arg("lint")
        .arg("scripts/validate-book.sh")
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    // Should have minimal warnings (backticks are intentional)
    assert!(stderr.contains("✓"));
}
```

**Options**:
1. Add shellcheck disable comments for false positives
2. Escape backticks properly
3. Use different markdown syntax

---

#### TASK 2.3: Fix Issue #1 (Makefile Auto-fix Bug)
**Issue**: Auto-fix appends instead of replaces
**Test (RED)**: Already exists in issue report

```rust
#[test]
fn test_ISSUE_001_makefile_autofix_replaces() {
    let original = "VERSION = $(shell git describe)\n";
    let result = lint_makefile(original);
    let fixed = apply_fixes(original, &result, &FixOptions::default()).unwrap();

    // Should replace, not append
    assert!(fixed.contains("VERSION :="));
    assert!(!fixed.contains("VERSION VERSION"));
}
```

**Implementation (GREEN)**:
1. Fix `apply_fixes()` in `rash/src/linter/fix.rs`
2. Ensure proper text replacement (not append)
3. Add comprehensive tests

---

### Phase 3: Book TDD (Religious Documentation)

#### TASK 3.1: Add Chapter 22 - Quality Enforcement
**Content**:
- How to use bashrs for quality gates
- pmat integration patterns
- CI/CD best practices

**Test (RED)**:
```rust
#[test]
fn test_SPRINT79_007_chapter22_exists() {
    let chapter = Path::new("rash-book/src/ch22-quality-enforcement.md");
    assert!(chapter.exists());
}

#[test]
fn test_SPRINT79_008_chapter22_examples_compile() {
    // Extract all rust code blocks from chapter 22
    // Compile each one
    // Assert all compile successfully
}
```

---

#### TASK 3.2: Update Chapter 21 with Auto-fix Caveat
**Current**: Claims auto-fix works 100%
**Reality**: Bug #1 exists

**Test (RED)**:
```rust
#[test]
fn test_SPRINT79_009_chapter21_documents_autofix_bug() {
    let chapter = std::fs::read_to_string("rash-book/src/ch21-makefile-linting-tdd.md").unwrap();
    assert!(chapter.contains("known limitation") || chapter.contains("Issue #1"));
}
```

**Implementation (GREEN)**: Add section documenting Issue #1

---

### Phase 4: CI/CD Integration

#### TASK 4.1: Add Quality Gates to GitHub Actions
**File**: `.github/workflows/quality.yml`

**Test (RED)**:
```rust
#[test]
fn test_SPRINT79_010_quality_workflow_exists() {
    let workflow = Path::new(".github/workflows/quality.yml");
    assert!(workflow.exists());
}
```

**Implementation (GREEN)**:
```yaml
name: Quality Gates

on:
  push:
    branches: [main]
  pull_request:

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable

      - name: Install bashrs
        run: cargo install --path rash

      - name: Lint scripts
        run: make lint-scripts

      - name: Lint Makefile
        run: make lint-makefile

      - name: Validate book
        run: make validate-book

      - name: Run tests
        run: cargo test --all
```

---

#### TASK 4.2: Add Pre-commit Hook Script
**File**: `scripts/pre-commit-hook.sh`

**Test (RED)**:
```rust
#[test]
fn test_SPRINT79_011_precommit_hook_exists() {
    let hook = Path::new("scripts/pre-commit-hook.sh");
    assert!(hook.exists());
    assert!(hook.metadata().unwrap().permissions().mode() & 0o111 != 0);
}
```

**Implementation (GREEN)**:
```bash
#!/bin/bash
# Pre-commit Quality Gate
set -e

echo "🔍 Running quality gates..."

# Lint scripts
make lint-scripts

# Lint Makefile
make lint-makefile

# Validate book
make validate-book

# Run fast tests
cargo test --lib

echo "✅ Quality gates passed!"
```

---

### Phase 5: Property/Mutation/Fuzz Testing

#### TASK 5.1: Property Tests for Auto-fix
**Test**:
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_SPRINT79_012_autofix_idempotent(
        makefile in "[a-zA-Z0-9 =:$()\n]{10,100}"
    ) {
        let result1 = lint_and_fix(&makefile);
        let result2 = lint_and_fix(&result1);

        // Applying fix twice should give same result
        prop_assert_eq!(result1, result2);
    }
}
```

---

#### TASK 5.2: Mutation Testing on Fix Logic
**Target**: >90% mutation score on `rash/src/linter/fix.rs`

**Command**:
```bash
cargo mutants --file rash/src/linter/fix.rs -- --lib
```

**Test**: Verify mutation score meets threshold

---

#### TASK 5.3: Fuzz Testing Makefile Parser
**Test**:
```rust
#[test]
fn test_SPRINT79_013_fuzz_makefile_parser() {
    // Use cargo-fuzz to fuzz Makefile parser
    // Ensure no panics on malformed input
}
```

---

## 🎯 Success Criteria

### Quality Metrics
- [ ] All shell scripts: <10 warnings each
- [ ] Makefile: 0 errors, <5 warnings
- [ ] Book validation: 100% passing
- [ ] Issue #1 fixed: Auto-fix works correctly
- [ ] Tests: 1,552+ passing (0 regressions)
- [ ] Property tests: 100+ cases passing
- [ ] Mutation score: >90% on fix logic

### Documentation
- [ ] Chapter 22 created (Quality Enforcement)
- [ ] Chapter 21 updated (Auto-fix caveat)
- [ ] All code examples compile
- [ ] Book accuracy: 100%

### CI/CD
- [ ] Quality workflow added to GitHub Actions
- [ ] Pre-commit hook script created
- [ ] Makefile has comprehensive `validate` target

---

## 📦 Release Checklist (v2.0.1)

- [ ] Fix Issue #1 (auto-fix bug)
- [ ] Fix all dogfooding issues (<10 warnings per script)
- [ ] Update CHANGELOG.md with v2.0.1 entry
- [ ] Bump version: 2.0.0 → 2.0.1
- [ ] Run full test suite (cargo test --all)
- [ ] Create git tag v2.0.1
- [ ] Build release binary
- [ ] Publish to crates.io
- [ ] Create GitHub release
- [ ] Dogfood v2.0.1 (verify fixes work)

---

## 🚀 EXTREME TDD Workflow

For each task:

1. **RED**: Write failing test first
2. **GREEN**: Implement minimal solution
3. **REFACTOR**: Clean up code (complexity <10)
4. **PROPERTY**: Add property tests (100+ cases)
5. **MUTATION**: Run mutation testing (>90% kill rate)
6. **FUZZ**: Fuzz test critical paths
7. **DOCUMENT**: Update book with example
8. **VALIDATE**: Run full `make validate`

---

## 📊 Estimated Timeline

- **Phase 1** (pmat Integration): 2 hours
- **Phase 2** (Dogfooding): 3 hours
- **Phase 3** (Book TDD): 2 hours
- **Phase 4** (CI/CD): 1 hour
- **Phase 5** (Property/Mutation/Fuzz): 2 hours
- **Release** (v2.0.1): 1 hour

**Total**: ~11 hours (1-2 days)

---

## 🎯 Next Steps

1. Start with Phase 1, Task 1.1 (lint-scripts target)
2. Follow EXTREME TDD for each task
3. Mark todos as completed after each phase
4. Create v2.0.1 release when all tasks complete
5. Dogfood v2.0.1 to verify all fixes work

---

**Sprint 79 Start Date**: 2025-10-19
**Target Completion**: 2025-10-20
**Release Target**: v2.0.1 (patch release)
