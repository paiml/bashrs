# Makefile Implementation - Action Plan

**Sprint**: Sprint 31 - Foundation Phase
**Target Version**: v1.4.0
**Status**: READY TO START
**Goal**: 10-20% GNU Make manual coverage
**Timeline**: 2-3 weeks

---

## 🎯 Immediate Next Steps (Week 1)

### Day 1-2: Project Setup

#### Task 1: Create Module Structure
```bash
# Create make_parser module
mkdir -p rash/src/make_parser
touch rash/src/make_parser/mod.rs
touch rash/src/make_parser/ast.rs
touch rash/src/make_parser/lexer.rs
touch rash/src/make_parser/parser.rs
touch rash/src/make_parser/semantic.rs
touch rash/src/make_parser/generators.rs
touch rash/src/make_parser/tests.rs

# Create make_transpiler module
mkdir -p rash/src/make_transpiler
touch rash/src/make_transpiler/mod.rs
touch rash/src/make_transpiler/codegen.rs
touch rash/src/make_transpiler/purification.rs
touch rash/src/make_transpiler/tests.rs

# Create make_linter module
mkdir -p rash/src/make_linter
touch rash/src/make_linter/mod.rs
touch rash/src/make_linter/rules.rs
touch rash/src/make_linter/tests.rs

# Update lib.rs
# Add: pub mod make_parser;
# Add: pub mod make_transpiler;
# Add: pub mod make_linter;
```

#### Task 2: Create Test Fixtures Directory
```bash
mkdir -p tests/fixtures/makefiles
mkdir -p tests/fixtures/makefiles/simple
mkdir -p tests/fixtures/makefiles/complex
mkdir -p tests/fixtures/makefiles/gnu_examples

# Create simple test Makefile
cat > tests/fixtures/makefiles/simple/hello.mk << 'EOF'
# Simple hello world Makefile
.PHONY: hello
hello:
	@echo "Hello, World!"
EOF
```

### Day 3-4: RULE-SYNTAX-001 (Basic Rule Syntax)

#### RED Phase
**File**: `rash/src/make_parser/tests.rs`

```rust
#[test]
fn test_basic_rule_syntax() {
    use crate::make_parser::ast::*;
    use crate::make_parser::parser::parse_makefile;

    // ARRANGE: Simple rule with target and recipe
    let makefile = r#"target: prerequisites
	recipe"#;

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok(), "Should parse basic rule syntax");

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have one item");

    // ASSERT: Item is a Target
    match &ast.items[0] {
        MakeItem::Target { name, prerequisites, recipe, .. } => {
            assert_eq!(name, "target");
            assert_eq!(prerequisites, &["prerequisites".to_string()]);
            assert_eq!(recipe.len(), 1);
            assert_eq!(recipe[0], "recipe");
        }
        _ => panic!("Expected Target item"),
    }
}
```

**Expected Result**: ❌ FAILS (types don't exist yet)

#### GREEN Phase

**File**: `rash/src/make_parser/ast.rs`

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct MakeAst {
    pub items: Vec<MakeItem>,
    pub metadata: MakeMetadata,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MakeItem {
    Target {
        name: String,
        prerequisites: Vec<String>,
        recipe: Vec<String>,
        phony: bool,
        span: Span,
    },
    // More variants to come
}

#[derive(Debug, Clone, PartialEq)]
pub struct MakeMetadata {
    pub source_file: Option<String>,
    pub line_count: usize,
    pub parse_time_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
}

impl Span {
    pub fn dummy() -> Self {
        Span { start: 0, end: 0, line: 0 }
    }
}
```

**File**: `rash/src/make_parser/parser.rs`

```rust
use super::ast::*;

pub fn parse_makefile(input: &str) -> Result<MakeAst, String> {
    let mut items = Vec::new();
    let lines: Vec<&str> = input.lines().collect();

    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            i += 1;
            continue;
        }

        // Parse target rule (simple version)
        if line.contains(':') {
            let parts: Vec<&str> = line.split(':').collect();
            let name = parts[0].trim().to_string();
            let prerequisites = parts[1]
                .split_whitespace()
                .map(|s| s.to_string())
                .collect();

            // Parse recipe (tab-indented lines)
            let mut recipe = Vec::new();
            i += 1;
            while i < lines.len() && lines[i].starts_with('\t') {
                recipe.push(lines[i].trim().to_string());
                i += 1;
            }

            items.push(MakeItem::Target {
                name,
                prerequisites,
                recipe,
                phony: false, // Will detect .PHONY later
                span: Span::dummy(),
            });
            continue;
        }

        i += 1;
    }

    Ok(MakeAst {
        items,
        metadata: MakeMetadata {
            source_file: None,
            line_count: lines.len(),
            parse_time_ms: 0,
        },
    })
}
```

**Expected Result**: ✅ PASSES

#### REFACTOR Phase
```bash
cargo clippy
cargo test test_basic_rule_syntax
```

**Expected Result**: ✅ ALL PASS, complexity <10

#### PROPERTY TESTING Phase

**File**: `rash/src/make_parser/tests.rs`

```rust
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_basic_rules_always_parse(
            target in "[a-z]{1,10}",
            prereq in "[a-z]{1,10}",
            recipe in "[a-z ]{1,20}"
        ) {
            let makefile = format!("{}:{}\n\t{}", target, prereq, recipe);
            let result = parse_makefile(&makefile);

            prop_assert!(result.is_ok());

            if let Ok(ast) = result {
                prop_assert_eq!(ast.items.len(), 1);
            }
        }
    }
}
```

**Expected Result**: ✅ 100 CASES PASS

#### DOCUMENTATION Phase

Update `docs/MAKE-INGESTION-ROADMAP.yaml`:

```yaml
- id: "RULE-SYNTAX-001"
  title: "Document basic rule syntax"
  status: "completed"  # ← Changed from "pending"
  version: "v1.4.0"    # ← Added
  priority: "CRITICAL"
  input: |
    target: prerequisites
    	recipe
  rust: |
    fn target() {
        check_prerequisites();
        run_recipe();
    }
  purified: |
    .PHONY: target
    target: prerequisites
    	recipe
  test_name: "test_basic_rule_syntax"
  tests_added:  # ← Added
    - "test_basic_rule_syntax (unit test)"
    - "prop_basic_rules_always_parse (property test, 100 cases)"
  notes: "Fundamental building block - implemented with EXTREME TDD"
  implementation:  # ← Added
    modules:
      - "rash/src/make_parser/ast.rs (MakeItem::Target)"
      - "rash/src/make_parser/parser.rs (parse_makefile)"
    lines_of_code: 85
```

### Day 5-7: VAR-BASIC-001 & VAR-BASIC-002 (Variables)

#### Follow same EXTREME TDD pattern:

1. **RED**: Write failing test for variable assignment
2. **GREEN**: Extend AST with `MakeItem::Variable`, implement parser
3. **REFACTOR**: Clean up, ensure complexity <10
4. **PROPERTY TESTING**: Add property tests for variables
5. **DOCUMENTATION**: Update roadmap

---

## 📅 Week 2-3: Core Features

### High Priority Tasks (Complete in Order)

| Task ID | Title | Priority | Status |
|---------|-------|----------|--------|
| RULE-SYNTAX-001 | Basic rule syntax | 1 | ✅ Week 1 |
| VAR-BASIC-001 | Variable assignment | 2 | 🔄 Week 2 |
| VAR-BASIC-002 | Variable reference | 3 | 🔄 Week 2 |
| VAR-FLAVOR-002 | Simple assignment (:=) | 4 | 🔄 Week 2 |
| PHONY-001 | .PHONY declarations | 5 | 🔄 Week 3 |
| RULE-001 | Target with recipe | 6 | 🔄 Week 3 |

### Milestone: Phase 1 Complete (v1.4.0)

**Deliverables**:
- ✅ Basic Makefile parser (rules, variables)
- ✅ Simple AST structure
- ✅ 6+ tasks completed (10-20% coverage)
- ✅ >85% test coverage
- ✅ >90% mutation kill rate
- ✅ All tests passing

---

## 🧪 Testing Requirements

### Per Task

1. **Unit Test**: ✅ 1 test per feature (EXTREME TDD)
2. **Property Test**: ✅ 1-3 property tests (proptest)
3. **Mutation Test**: ✅ Run `cargo mutants` on modified files
4. **Integration Test**: ✅ (Optional for Phase 1)

### Quality Gates

Before marking task as "completed":

- [ ] Unit test passes ✅
- [ ] Property tests pass (100+ cases) ✅
- [ ] All tests pass (`cargo test --lib`) ✅
- [ ] No clippy warnings (`cargo clippy`) ✅
- [ ] Complexity <10 (`pmat analyze complexity`) ✅
- [ ] Mutation kill rate >90% (`cargo mutants`) ✅
- [ ] Roadmap updated ✅

---

## 📊 Success Metrics for Phase 1

### Code Quality
- [x] Module structure created
- [ ] >85% test coverage
- [ ] >90% mutation kill rate
- [ ] 0 clippy warnings
- [ ] Complexity <10 per function

### Feature Completeness
- [ ] 6-10 tasks completed
- [ ] 10-20% GNU Make manual coverage
- [ ] Basic parsing works
- [ ] Variables supported
- [ ] Simple rules supported

### Documentation
- [x] Specification complete
- [x] Roadmap complete
- [ ] 6-10 tasks documented in roadmap
- [ ] Implementation notes added

---

## 🚦 STOP THE LINE Protocol

If you encounter any bugs during implementation:

```
🚨 STOP THE LINE - BUG DETECTED 🚨

1. STOP current work
2. Create bug ticket in roadmap
3. Write failing test (RED)
4. Fix bug (GREEN)
5. Refactor (REFACTOR)
6. Add property test (PROPERTY TESTING)
7. Run mutation test
8. Document (DOCUMENTATION)
9. Resume original task
```

---

## 📝 Commit Message Format

```
feat: Add basic Makefile rule parsing (RULE-SYNTAX-001)

- Implemented MakeItem::Target in ast.rs
- Added parse_makefile() in parser.rs
- Created 1 unit test + 1 property test (100 cases)
- All tests passing (935 total)
- Mutation kill rate: 91%

EXTREME TDD: RED→GREEN→REFACTOR→PROPERTY→DOCUMENTATION
Task: RULE-SYNTAX-001
```

---

## 🔄 Daily Workflow

### Morning (Start of Day)
```bash
# 1. Pull latest
git pull origin main

# 2. Check mutation test results
tail -50 /tmp/mutants-ast-improved-final.log

# 3. Run all tests
cargo test --lib

# 4. Check clippy
cargo clippy
```

### During Implementation (Per Task)
```bash
# RED: Write failing test
# Edit: rash/src/make_parser/tests.rs
cargo test test_<feature> # Should FAIL

# GREEN: Implement feature
# Edit: ast.rs, parser.rs, etc.
cargo test test_<feature> # Should PASS

# REFACTOR: Clean up
cargo clippy
cargo test

# PROPERTY TESTING: Add property tests
# Edit: tests.rs (add proptest)
cargo test prop_<feature>

# Mutation testing
cargo mutants --file src/make_parser/parser.rs -- --lib

# DOCUMENTATION: Update roadmap
# Edit: docs/MAKE-INGESTION-ROADMAP.yaml

# Commit
git add -A
git commit -m "feat: <message> (<TASK-ID>)"
```

### End of Day
```bash
# 1. Run full test suite
cargo test

# 2. Check coverage
cargo llvm-cov --all-features --workspace

# 3. Push changes
git push origin make-parser-foundation

# 4. Update progress in roadmap
# Edit: docs/MAKE-INGESTION-ROADMAP.yaml
# Update: status.completion_percent, completed_tasks
```

---

## 🎯 Phase 1 Completion Checklist

Before marking Phase 1 (v1.4.0) complete:

### Code
- [ ] `rash/src/make_parser/` module created
- [ ] `rash/src/make_parser/ast.rs` implemented
- [ ] `rash/src/make_parser/parser.rs` implemented
- [ ] `rash/src/make_parser/tests.rs` with 6+ tests
- [ ] All tests passing (>940 total)

### Quality
- [ ] >85% test coverage
- [ ] >90% mutation kill rate
- [ ] 0 clippy warnings
- [ ] Complexity <10 per function

### Documentation
- [ ] 6-10 tasks marked "completed" in roadmap
- [ ] Implementation details added
- [ ] Statistics updated (completion_percent, completed_tasks)
- [ ] CHANGELOG.md updated

### Testing
- [ ] 6+ unit tests (EXTREME TDD)
- [ ] 6+ property tests (proptest)
- [ ] Mutation testing completed
- [ ] Integration tests (optional)

### Release
- [ ] Tag v1.4.0
- [ ] Release notes created
- [ ] Sprint 31 summary written

---

## 📚 Reference Quick Links

- **Specification**: `docs/specification/lint-purify-test-write-Makefile-document-gnu-guide.md`
- **Roadmap**: `docs/MAKE-INGESTION-ROADMAP.yaml`
- **Summary**: `docs/MAKEFILE-PURIFICATION-SUMMARY.md`
- **Bash Implementation** (reference): `rash/src/bash_parser/`
- **GNU Make Manual**: https://www.gnu.org/software/make/manual/make.html

---

## 🎓 Key Principles

1. **EXTREME TDD**: Always RED→GREEN→REFACTOR→PROPERTY→DOCUMENTATION
2. **Test First**: Write failing test BEFORE implementation
3. **Quality Gates**: >90% mutation kill rate, >85% coverage
4. **STOP THE LINE**: Fix bugs immediately, don't continue
5. **Small Commits**: One task per commit with clear message
6. **Daily Progress**: Update roadmap daily

---

## 💡 Tips for Success

1. **Start Small**: RULE-SYNTAX-001 is intentionally simple
2. **Reference Bash**: Look at `bash_parser/` for patterns
3. **Use Property Tests**: Catch edge cases early
4. **Run Mutation Tests**: Verify test quality
5. **Keep Complexity Low**: <10 per function
6. **Document as You Go**: Update roadmap immediately
7. **Ask for Help**: Don't get stuck - reach out if needed

---

## 🎉 Expected Outcomes (End of Week 3)

### Code Metrics
- **Total Tests**: ~950 (up from 934)
- **Coverage**: >85%
- **Mutation Kill Rate**: >90%
- **Tasks Completed**: 6-10
- **Manual Coverage**: 10-20%

### Capabilities
- ✅ Parse basic Makefile rules
- ✅ Parse variable assignments
- ✅ Parse variable references
- ✅ Parse .PHONY declarations
- ✅ Generate purified Makefile output

### Documentation
- ✅ Roadmap updated with 6-10 completed tasks
- ✅ Implementation details documented
- ✅ Statistics accurate
- ✅ Sprint 31 summary written

---

**Action Plan Version**: 1.0.0
**Created**: 2025-10-15
**Target Start**: Sprint 31
**Target Completion**: 2-3 weeks
**Status**: READY TO EXECUTE

Let's build this! 🚀
