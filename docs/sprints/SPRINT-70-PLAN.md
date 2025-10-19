# Sprint 70: Linter Phase 1 - Core Infrastructure + DET/IDEM Rules

**Status**: 🟢 READY TO EXECUTE
**Duration**: 4-6 weeks (estimated 25-35 hours)
**Version Target**: v1.5.0
**Date**: 2025-10-19
**Methodology**: EXTREME TDD (RED-GREEN-REFACTOR-INTEGRATION)

---

## 🎯 Sprint Goal

Deliver a **minimal viable linter** with bashrs's unique value proposition: determinism and idempotency checking. This establishes the foundation for future rule expansion while providing immediate, differentiated value.

**Why Phase 1 First?**
- ✅ Unique features (DET/IDEM) no other linter has
- ✅ Leverages existing purification infrastructure
- ✅ Provides immediate user value
- ✅ Validates architecture before investing in 800+ rules
- ✅ Early feedback on UX and performance

---

## 📦 Deliverables

### 1. Core Linter Infrastructure

**Files to Create**:
```
rash/src/lint/
├── mod.rs                    # Main linter module
├── rule.rs                   # Rule trait and registry
├── visitor.rs                # AST visitor for rule evaluation
├── diagnostic.rs             # Diagnostic/violation types
├── fix.rs                    # Auto-fix engine
├── config.rs                 # Configuration system (minimal)
└── rules/
    ├── mod.rs
    ├── determinism.rs        # DET001-DET020
    └── idempotency.rs        # IDEM001-IDEM015
```

### 2. CLI Integration

**Command**:
```bash
bashrs lint [OPTIONS] <FILE>...

OPTIONS:
    --fix              Apply safe auto-fixes
    --dry-run          Preview fixes without applying
    --format <FORMAT>  Output format: pretty, json [default: pretty]
    --list-rules       List all available rules
    --explain <RULE>   Explain specific rule
```

### 3. Rule Implementations

#### Determinism Rules (DET001-DET020)

**Priority Rules** (implement in Sprint 70):

| Rule | Description | Auto-Fix | Priority |
|------|-------------|----------|----------|
| **DET001** | $RANDOM usage | Yes | P0 |
| **DET002** | Timestamp usage (date +%s) | Yes | P0 |
| **DET003** | Unordered wildcard | Yes | P0 |
| **DET004** | Process ID usage ($$) | Yes | P1 |
| **DET005** | Hostname dependency | Yes | P1 |
| **DET006** | Unsorted glob results | Yes | P1 |
| **DET007** | /dev/random usage | Manual | P2 |
| **DET008** | Network timing dependency | Manual | P2 |

**Deferred** (DET009-DET020): Implement in later sprints based on user feedback

#### Idempotency Rules (IDEM001-IDEM015)

**Priority Rules** (implement in Sprint 70):

| Rule | Description | Auto-Fix | Priority |
|------|-------------|----------|----------|
| **IDEM001** | Non-idempotent mkdir | Yes | P0 |
| **IDEM002** | Non-idempotent rm | Yes | P0 |
| **IDEM003** | Non-idempotent ln | Yes | P0 |
| **IDEM004** | Append without check (>>) | Yes | P1 |
| **IDEM005** | cp without -n | Yes | P1 |
| **IDEM006** | mv without -n | Yes | P1 |
| **IDEM007** | touch without -a | Manual | P2 |
| **IDEM008** | Non-idempotent chmod | Manual | P2 |

**Deferred** (IDEM009-IDEM015): Implement in later sprints

### 4. Tests

**Test Coverage Requirements**:
- ✅ Unit tests: >85% coverage on lint modules
- ✅ Property tests: 100+ generated cases per rule
- ✅ Integration tests: End-to-end CLI workflows
- ✅ Mutation tests: ≥90% kill rate

**Test Count Target**: +150 tests
- ~80 unit tests (rule logic)
- ~40 property tests (generative)
- ~20 integration tests (CLI)
- ~10 auto-fix tests

### 5. Documentation

- Sprint 70 plan (this document)
- Sprint 70 handoff document
- Sprint 70 quick reference card
- Rule reference documentation (DET/IDEM rules)
- CLI usage examples

---

## 🏗️ Architecture

### Leveraging Existing Infrastructure

**Reuse from Makefile Parser**:
```rust
// Already implemented:
rash/src/make_parser/parser.rs      // Parse shell scripts to AST
rash/src/make_parser/ast.rs         // AST representation
rash/src/make_parser/semantic.rs    // detect_random(), detect_wildcard()
rash/src/make_parser/purify.rs      // Transformation rules
```

**New Components**:

#### 1. Rule Trait
```rust
// rash/src/lint/rule.rs
pub trait Rule: Send + Sync {
    fn code(&self) -> &'static str;           // e.g., "DET001"
    fn severity(&self) -> Severity;           // Error/Warning/Info
    fn message(&self) -> &'static str;        // Short description
    fn explanation(&self) -> &'static str;    // Detailed explanation

    fn check(&self, ast: &ShellAst) -> Vec<Diagnostic>;
    fn fix(&self, diagnostic: &Diagnostic) -> Option<Fix>;
}

pub enum Severity {
    Error,
    Warning,
    Info,
}
```

#### 2. Diagnostic Type
```rust
// rash/src/lint/diagnostic.rs
pub struct Diagnostic {
    pub rule: String,           // "DET001"
    pub severity: Severity,
    pub message: String,
    pub span: Span,             // Line/column info
    pub fix: Option<Fix>,
}

pub struct Fix {
    pub description: String,
    pub replacement: String,
    pub span: Span,
    pub safety: FixSafety,
}

pub enum FixSafety {
    Safe,           // Apply with --fix
    PotentiallyUnsafe,  // Require --fix-unsafe
    Manual,         // Not auto-fixable
}
```

#### 3. Linter Engine
```rust
// rash/src/lint/mod.rs
pub struct Linter {
    rules: Vec<Box<dyn Rule>>,
    config: LintConfig,
}

impl Linter {
    pub fn new() -> Self { /* ... */ }

    pub fn lint_file(&self, path: &Path) -> LintResult {
        // 1. Parse shell script to AST
        // 2. Visit AST with all rules
        // 3. Collect diagnostics
        // 4. Return results
    }

    pub fn apply_fixes(&self, diagnostics: Vec<Diagnostic>) -> Result<()> {
        // Apply auto-fixes to file
    }
}
```

### Integration Points

**CLI Integration** (`rash/src/cli/commands.rs`):
```rust
pub fn lint_command(args: LintArgs) -> Result<()> {
    let linter = Linter::new();

    for file in args.files {
        let result = linter.lint_file(&file)?;

        if args.fix {
            linter.apply_fixes(result.diagnostics)?;
        }

        print_diagnostics(&result, args.format);
    }

    Ok(())
}
```

---

## 📋 Task Breakdown

### Week 1-2: Core Infrastructure (10-15 hours)

**Task 1.1: Rule Framework** (3-4 hours)
- [ ] Create `lint/` module structure
- [ ] Implement `Rule` trait
- [ ] Implement `Diagnostic` and `Fix` types
- [ ] Create rule registry
- [ ] Write 10 unit tests

**Task 1.2: AST Visitor** (3-4 hours)
- [ ] Implement AST visitor pattern
- [ ] Rule evaluation engine
- [ ] Diagnostic collection
- [ ] Write 8 unit tests

**Task 1.3: CLI Integration** (4-5 hours)
- [ ] Add `bashrs lint` subcommand
- [ ] Argument parsing (--fix, --dry-run, --format)
- [ ] Basic output formatting (pretty)
- [ ] Write 5 CLI integration tests

**Milestone 1**: Can run `bashrs lint script.sh` (no rules yet)

### Week 3-4: Determinism Rules (8-12 hours)

**Task 2.1: DET001 - $RANDOM Usage** (2-3 hours)
- [ ] RED: Write failing test
- [ ] GREEN: Implement detection logic
- [ ] REFACTOR: Clean up implementation
- [ ] Add auto-fix (suggest version-based ID)
- [ ] Property tests (10+ cases)
- [ ] Mutation testing (≥90% kill rate)

**Task 2.2: DET002 - Timestamp Usage** (2-3 hours)
- [ ] RED: Write failing test
- [ ] GREEN: Detect `date +%s`, `$(date)` patterns
- [ ] REFACTOR: Extract helper functions
- [ ] Add auto-fix (suggest VERSION variable)
- [ ] Property tests
- [ ] Mutation testing

**Task 2.3: DET003 - Unordered Wildcard** (2-3 hours)
- [ ] RED: Write failing test
- [ ] GREEN: Detect unwrapped $(wildcard)
- [ ] REFACTOR: Reuse semantic.rs logic
- [ ] Add auto-fix (wrap with sort)
- [ ] Property tests
- [ ] Mutation testing

**Task 2.4: DET004-DET008** (2-3 hours)
- [ ] Implement remaining P0/P1 rules
- [ ] Tests for each rule
- [ ] Auto-fixes where applicable

**Milestone 2**: DET rules functional with auto-fix

### Week 5: Idempotency Rules (6-8 hours)

**Task 3.1: IDEM001 - Non-idempotent mkdir** (2 hours)
- [ ] RED: Write failing test
- [ ] GREEN: Detect mkdir without -p
- [ ] Add auto-fix (add -p flag)
- [ ] Property + mutation tests

**Task 3.2: IDEM002 - Non-idempotent rm** (2 hours)
- [ ] RED: Write failing test
- [ ] GREEN: Detect rm without -f
- [ ] Add auto-fix (add -f flag)
- [ ] Property + mutation tests

**Task 3.3: IDEM003 - Non-idempotent ln** (2 hours)
- [ ] RED: Write failing test
- [ ] GREEN: Detect ln -s without rm -f first
- [ ] Add auto-fix (prepend rm -f)
- [ ] Property + mutation tests

**Task 3.4: IDEM004-IDEM008** (2 hours)
- [ ] Implement remaining P0/P1 rules
- [ ] Tests for each rule

**Milestone 3**: IDEM rules functional with auto-fix

### Week 6: Auto-Fix Engine + Polish (5-7 hours)

**Task 4.1: Auto-Fix Engine** (3-4 hours)
- [ ] Implement fix application
- [ ] Handle overlapping fixes
- [ ] Dry-run mode
- [ ] Backup creation (.bak files)
- [ ] Write 8 auto-fix tests

**Task 4.2: Output Formats** (2-3 hours)
- [ ] Pretty output (colored, with context)
- [ ] JSON output (structured)
- [ ] --list-rules command
- [ ] --explain <rule> command

**Milestone 4**: Sprint 70 COMPLETE

---

## 🎯 Success Criteria

### Must Have (P0)
- [ ] ✅ `bashrs lint` command functional
- [ ] ✅ DET001-DET003 rules implemented + auto-fix
- [ ] ✅ IDEM001-IDEM003 rules implemented + auto-fix
- [ ] ✅ 150+ tests passing (100%)
- [ ] ✅ >85% code coverage on lint modules
- [ ] ✅ ≥90% mutation kill rate
- [ ] ✅ Pretty output format working
- [ ] ✅ Documentation complete

### Should Have (P1)
- [ ] ✅ DET004-DET006 rules
- [ ] ✅ IDEM004-IDEM006 rules
- [ ] ✅ JSON output format
- [ ] ✅ --list-rules command
- [ ] ✅ --explain command

### Nice to Have (P2)
- [ ] DET007-DET008 rules
- [ ] IDEM007-IDEM008 rules
- [ ] Rule quality metrics tracking
- [ ] Performance benchmarks

---

## 📊 Example Usage

### Basic Linting
```bash
# Lint a script
$ bashrs lint deploy.sh

deploy.sh
  DET001 [Error] Non-deterministic $RANDOM usage
    5 │ SESSION_ID=$RANDOM
      │            ^^^^^^^
      │ Replace with deterministic identifier
      │
      │ Fix: SESSION_ID="session-${VERSION}"

  IDEM001 [Warning] Non-idempotent mkdir
    8 │ mkdir /app/releases
      │ ^^^^^
      │ Add -p flag for idempotent operation
      │
      │ Fix: mkdir -p /app/releases

Found 1 error, 1 warning in deploy.sh
Run with --fix to apply automatic fixes
```

### Auto-Fix
```bash
# Preview fixes
$ bashrs lint --fix --dry-run deploy.sh
Would apply 2 fixes to deploy.sh:
  DET001 line 5: SESSION_ID=$RANDOM → SESSION_ID="session-${VERSION}"
  IDEM001 line 8: mkdir /app/releases → mkdir -p /app/releases

# Apply fixes
$ bashrs lint --fix deploy.sh
Applied 2 fixes to deploy.sh
Backup created: deploy.sh.bak

# Verify
$ bashrs lint deploy.sh
No issues found! ✅
```

### Integration with Purify
```bash
# Full workflow: lint → purify → verify
bashrs lint deploy.sh              # Find DET/IDEM issues
bashrs purify deploy.sh            # Auto-purify script
bashrs lint deploy-purified.sh     # Verify no DET/IDEM issues remain
```

---

## 🧪 Testing Strategy

### Unit Tests (80 tests)
```rust
#[test]
fn test_DET001_random_detected() {
    let script = "SESSION_ID=$RANDOM";
    let diagnostics = lint(script);

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].rule, "DET001");
    assert_eq!(diagnostics[0].severity, Severity::Error);
}

#[test]
fn test_DET001_autofix_suggested() {
    let script = "SESSION_ID=$RANDOM";
    let diagnostics = lint(script);

    assert!(diagnostics[0].fix.is_some());
    assert_eq!(diagnostics[0].fix.unwrap().safety, FixSafety::Safe);
}
```

### Property Tests (40 tests)
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_DET001_always_detects_random(var_name in "[A-Z_]+") {
        let script = format!("{}=$RANDOM", var_name);
        let diagnostics = lint(&script);
        prop_assert_eq!(diagnostics.len(), 1);
        prop_assert_eq!(diagnostics[0].rule, "DET001");
    }
}
```

### Integration Tests (20 tests)
```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_lint_cli_detects_random() {
    let temp = "/tmp/test_random.sh";
    fs::write(temp, "SESSION_ID=$RANDOM").unwrap();

    Command::cargo_bin("bashrs")
        .unwrap()
        .arg("lint")
        .arg(temp)
        .assert()
        .failure()
        .stdout(predicate::str::contains("DET001"))
        .stdout(predicate::str::contains("$RANDOM"));

    fs::remove_file(temp).unwrap();
}
```

---

## 📈 Quality Gates

Before marking Sprint 70 as complete:

- [ ] ✅ All tests pass (150+/150+)
- [ ] ✅ Code coverage >85%
- [ ] ✅ Mutation kill rate ≥90%
- [ ] ✅ Zero clippy warnings
- [ ] ✅ All DET/IDEM P0 rules implemented
- [ ] ✅ Auto-fix working for all P0 rules
- [ ] ✅ CLI integration complete
- [ ] ✅ Documentation complete
- [ ] ✅ Examples working
- [ ] ✅ No regressions in existing tests

---

## 🚀 Next Steps After Sprint 70

### Option 1: Continue Linter (Phase 2)
- Implement security rules (SEC001-SEC045)
- Add portability rules (P001-P080)
- Expand to 100+ rules

### Option 2: Return to Bash Validation ⭐ RECOMMENDED
- Resume GNU Bash manual validation
- Target: 35% → 50% completion
- Validate linter rules against real-world use cases

### Option 3: Linter Features
- Add configuration file support (bashrs.toml)
- Implement caching for performance
- Add GitHub Actions output format

---

## 🎯 Value Proposition

**Why This Sprint Delivers Immediate Value**:

1. **Unique Features**: DET/IDEM rules no other linter has
2. **Complements Purify**: Catches issues before purification
3. **Early Validation**: Proves linter architecture works
4. **User Feedback**: Real-world usage informs future rules
5. **Marketing**: "World's first determinism/idempotency linter"

**What Users Get**:
```bash
# Before Sprint 70
bashrs purify deploy.sh    # Purify script (no pre-check)

# After Sprint 70
bashrs lint deploy.sh      # Check for DET/IDEM issues first
bashrs purify deploy.sh    # Purify with confidence
bashrs lint --purified deploy-purified.sh  # Verify quality
```

---

## 📅 Timeline

| Week | Focus | Hours | Deliverable |
|------|-------|-------|-------------|
| 1-2 | Infrastructure | 10-15 | Rule framework, CLI |
| 3-4 | DET rules | 8-12 | DET001-DET008 |
| 5 | IDEM rules | 6-8 | IDEM001-IDEM008 |
| 6 | Auto-fix + Polish | 5-7 | Complete v1.5.0 |

**Total**: 29-42 hours (4-6 weeks)

---

**Status**: 🟢 READY TO EXECUTE
**Methodology**: EXTREME TDD
**Next Action**: Begin Task 1.1 (Rule Framework)
**Expected Completion**: v1.5.0 release with DET/IDEM linter

---

## 🏆 Sprint Success Definition

Sprint 70 is **COMPLETE** when:

1. ✅ A developer can run `bashrs lint deploy.sh`
2. ✅ DET/IDEM issues are detected and reported
3. ✅ `--fix` applies corrections automatically
4. ✅ 150+ tests pass (100%)
5. ✅ Documentation shows clear examples
6. ✅ v1.5.0 released to crates.io

**Unique Value**: "bashrs is now the world's first linter with built-in determinism and idempotency checking."
