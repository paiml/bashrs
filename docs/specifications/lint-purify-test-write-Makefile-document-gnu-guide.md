# GNU Make Purification & Testing Specification
## EXTREME TDD Implementation Guide for Makefile Transformation

**Version**: 1.0.0
**Status**: SPECIFICATION
**Methodology**: EXTREME TDD (RED → GREEN → REFACTOR → PROPERTY TESTING → DOCUMENTATION)
**Reference**: [GNU Make Manual](https://www.gnu.org/software/make/manual/make.html)
**Target**: Bidirectional Make ↔ Rust Transformation with Purification

---

## 📋 Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [GNU Make Manual Structure](#gnu-make-manual-structure)
4. [Transformation Workflows](#transformation-workflows)
5. [AST Design](#ast-design)
6. [Purification Rules](#purification-rules)
7. [EXTREME TDD Implementation](#extreme-tdd-implementation)
8. [Testing Strategy](#testing-strategy)
9. [Quality Gates](#quality-gates)
10. [Roadmap](#roadmap)

---

## 🎯 Overview

### Project Goal

Extend the Rash (bashrs) project to support **GNU Make** using the same proven methodology:
- Parse Makefiles into a strongly-typed AST
- Transform to safe, deterministic, idempotent Makefiles (purified)
- Bidirectional transformation: Make ↔ Rust
- Comprehensive linting, mutation testing, and property testing

### Why Makefiles Need Purification

**Common Makefile Problems**:
```makefile
# ❌ Non-deterministic: uses timestamp
RELEASE := release-$(shell date +%s)

# ❌ Non-idempotent: no .PHONY
test:
	cargo test

# ❌ Portability issues: GNU Make specific
SOURCES := $(wildcard src/*.c)

# ❌ Non-deterministic: order depends on filesystem
FILES := $(shell find . -name "*.c")

# ❌ Race conditions: parallel make issues
deploy: build
	rm -rf dist
	mkdir dist
	cp build/* dist/
```

**Purified Output**:
```makefile
# ✅ Deterministic: explicit version
RELEASE := release-1.0.0

# ✅ Idempotent: .PHONY declared
.PHONY: test
test:
	cargo test

# ✅ Portable: explicit file list
SOURCES := src/main.c src/util.c src/config.c

# ✅ Deterministic: explicit ordered list
FILES := src/main.c src/module.c

# ✅ Race-safe: idempotent directory creation
.PHONY: deploy
deploy: build
	mkdir -p dist
	cp build/* dist/
```

---

## 🏗️ Architecture

### System Components

```
┌─────────────────────────────────────────────────────────────────┐
│                    Makefile Purification System                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│  ┌────────────┐    ┌──────────┐    ┌────────────┐               │
│  │  Makefile  │───▶│  Parser  │───▶│   AST      │               │
│  │   Input    │    │          │    │ (MakeAst)  │               │
│  └────────────┘    └──────────┘    └────────────┘               │
│                                            │                      │
│                                            ▼                      │
│                    ┌──────────────────────────────┐              │
│                    │   Semantic Analyzer          │              │
│                    │   - Variable tracking        │              │
│                    │   - Target dependencies      │              │
│                    │   - Effect analysis          │              │
│                    └──────────────────────────────┘              │
│                              │           │                        │
│                    ┌─────────┴──┐    ┌──┴─────────┐             │
│                    ▼            ▼    ▼            ▼             │
│            ┌──────────┐  ┌────────────┐  ┌──────────┐          │
│            │ Rust Gen │  │ Purifier   │  │ Linter   │          │
│            │          │  │            │  │          │          │
│            └──────────┘  └────────────┘  └──────────┘          │
│                 │             │               │                  │
│                 ▼             ▼               ▼                  │
│         ┌──────────┐  ┌────────────┐  ┌──────────┐             │
│         │  Rust    │  │  Purified  │  │  Lint    │             │
│         │  Code    │  │  Makefile  │  │  Report  │             │
│         └──────────┘  └────────────┘  └──────────┘             │
│                                                                   │
│  ┌──────────────────────────────────────────────────────┐       │
│  │               Testing & Verification                  │       │
│  │  • Unit Tests (EXTREME TDD)                          │       │
│  │  • Property Tests (proptest)                         │       │
│  │  • Mutation Tests (cargo-mutants)                    │       │
│  │  • Integration Tests (real Makefiles)                │       │
│  └──────────────────────────────────────────────────────┘       │
└─────────────────────────────────────────────────────────────────┘
```

### Module Structure

```
rash/
├── src/
│   ├── make_parser/          # Makefile parser
│   │   ├── mod.rs
│   │   ├── ast.rs            # AST types (MakeAst, MakeTarget, etc.)
│   │   ├── parser.rs         # Parsing logic
│   │   ├── lexer.rs          # Tokenization
│   │   ├── semantic.rs       # Semantic analysis
│   │   ├── generators.rs     # Purified Makefile generation
│   │   └── tests.rs          # Unit + property tests
│   │
│   ├── make_transpiler/      # Make → Rust
│   │   ├── mod.rs
│   │   ├── codegen.rs        # Rust code generation
│   │   ├── purification.rs   # Purification rules
│   │   └── tests.rs
│   │
│   └── make_linter/          # Linting rules
│       ├── mod.rs
│       ├── rules.rs          # Lint rule definitions
│       └── tests.rs
│
├── tests/
│   ├── make_integration_tests.rs
│   └── fixtures/
│       └── makefiles/
│           ├── simple.mk
│           ├── complex.mk
│           └── gnu_examples/
│
└── docs/
    └── MAKE-INGESTION-ROADMAP.yaml
```

---

## 📚 GNU Make Manual Structure

### Manual Organization (for Systematic Validation)

The GNU Make Manual will be our validation source, similar to how we use the GNU Bash Manual:

**Key Chapters** (from GNU Make 4.4 Manual):

1. **Overview of make**
2. **An Introduction to Makefiles**
   - 2.1 What a Rule Looks Like
   - 2.2 A Simple Makefile
   - 2.3 How make Processes a Makefile
   - 2.4 Variables Make Makefiles Simpler
   - 2.5 Letting make Deduce the Recipes
   - 2.6 Another Style of Makefile
3. **Writing Makefiles**
   - 3.1 What Makefiles Contain
   - 3.2 What Name to Give Your Makefile
   - 3.3 Including Other Makefiles
   - 3.4 The Variable MAKEFILES
   - 3.5 How Makefiles Are Remade
   - 3.6 Overriding Part of Another Makefile
4. **Writing Rules**
   - 4.1 Rule Syntax
   - 4.2 Types of Prerequisites
   - 4.3 Using Wildcard Characters in File Names
   - 4.4 Searching Directories for Prerequisites
   - 4.5 Phony Targets
   - 4.6 Rules without Recipes or Prerequisites
   - 4.7 Empty Target Files to Record Events
   - 4.8 Special Built-in Target Names
   - 4.9 Multiple Targets in a Rule
   - 4.10 Multiple Rules for One Target
   - 4.11 Static Pattern Rules
   - 4.12 Double-Colon Rules
5. **Writing Recipes in Rules**
   - 5.1 Recipe Syntax
   - 5.2 Recipe Echoing
   - 5.3 Recipe Execution
   - 5.4 Parallel Execution
   - 5.5 Errors in Recipes
   - 5.6 Interrupting or Killing make
   - 5.7 Recursive Use of make
6. **How to Use Variables**
   - 6.1 Basics of Variable References
   - 6.2 The Two Flavors of Variables
   - 6.3 Advanced Features for Reference to Variables
   - 6.4 How Variables Get Their Values
   - 6.5 Setting Variables
   - 6.6 Appending More Text to Variables
   - 6.7 The override Directive
   - 6.8 Defining Multi-Line Variables
   - 6.9 Undefining Variables
   - 6.10 Variables from the Environment
   - 6.11 Target-specific Variable Values
   - 6.12 Pattern-specific Variable Values
7. **Conditional Parts of Makefiles**
8. **Functions for Transforming Text**
   - 8.1 Function Call Syntax
   - 8.2 Functions for String Substitution and Analysis
   - 8.3 Functions for File Names
   - 8.4 Functions for Conditionals
   - 8.5 The foreach Function
   - 8.6 The file Function
   - 8.7 The call Function
   - 8.8 The value Function
   - 8.9 The eval Function
   - 8.10 The origin Function
   - 8.11 The flavor Function
   - 8.12 Functions That Control Make
   - 8.13 The shell Function
   - 8.14 The guile Function
9. **How to Run make**
10. **Using Implicit Rules**
11. **Using make to Update Archive Files**
12. **Extending GNU make**
13. **Integrating GNU make**

### Validation Strategy

**Follow the same approach as Bash ingestion**:

1. **Create MAKE-INGESTION-ROADMAP.yaml**
   - Task for each section of the GNU Make Manual
   - Input: Make construct
   - Rust: Rust equivalent
   - Purified: Safe, deterministic Make
   - Test name for each

2. **STOP THE LINE Protocol**
   - Halt when a bug is discovered
   - Fix with EXTREME TDD before continuing
   - Document in roadmap

3. **Systematic Coverage**
   - Process manual chapter by chapter
   - Track coverage percentage
   - Aim for 100% manual coverage

---

## 🔄 Transformation Workflows

### Workflow 1: Makefile → Rust → Purified Makefile (PRIMARY)

**Use Case**: Clean up existing Makefiles, remove non-determinism, ensure idempotency

```makefile
# INPUT: Legacy Makefile
TIMESTAMP := $(shell date +%s)
RELEASE := release-$(TIMESTAMP)

test:
	cargo test

deploy: build
	rm -rf dist
	mkdir dist
	scp -r dist/* server:/opt/
```

↓ **Parse to Rust**

```rust
// Rust representation with purification
use std::fs;

fn test() -> Result<(), String> {
    // Purified: uses explicit version, not timestamp
    let release = "release-1.0.0";

    // Tests are deterministic
    run_tests()?;
    Ok(())
}

fn deploy() -> Result<(), String> {
    build()?;

    // Purified: idempotent directory creation
    fs::create_dir_all("dist")?;

    // Purified: explicit, deterministic operations
    copy_artifacts()?;
    Ok(())
}
```

↓ **Generate Purified Makefile**

```makefile
# OUTPUT: Purified Makefile
RELEASE := release-1.0.0

.PHONY: test
test:
	cargo test

.PHONY: deploy
deploy: build
	mkdir -p dist
	cp -r dist/* server:/opt/
```

### Workflow 2: Rust → Makefile (SECONDARY)

**Use Case**: Generate safe Makefiles from Rust build scripts

```rust
// INPUT: Rust build definition
fn main() {
    let targets = vec!["build", "test", "clean"];

    build_project()?;
    run_tests()?;
    clean_artifacts()?;
}
```

↓ **Generate Makefile**

```makefile
# OUTPUT: Generated Makefile
.PHONY: all build test clean

all: build test

build:
	cargo build --release

test:
	cargo test

clean:
	cargo clean
```

---

## 🌳 AST Design

### Core AST Types

```rust
// rash/src/make_parser/ast.rs

/// Top-level Makefile AST
#[derive(Debug, Clone, PartialEq)]
pub struct MakeAst {
    pub items: Vec<MakeItem>,
    pub metadata: MakeMetadata,
}

/// Makefile constructs
#[derive(Debug, Clone, PartialEq)]
pub enum MakeItem {
    /// Variable assignment: VAR = value
    Variable {
        name: String,
        value: MakeExpr,
        flavor: VarFlavor,  // = vs := vs ?= vs +=
        span: Span,
    },

    /// Target rule: target: prerequisites \n\t recipe
    Target {
        name: String,
        prerequisites: Vec<String>,
        recipe: Vec<String>,
        phony: bool,  // Marked with .PHONY
        span: Span,
    },

    /// Pattern rule: %.o: %.c
    PatternRule {
        target_pattern: String,
        prereq_patterns: Vec<String>,
        recipe: Vec<String>,
        span: Span,
    },

    /// Conditional: ifeq/ifneq/ifdef/ifndef
    Conditional {
        condition: MakeCondition,
        then_items: Vec<MakeItem>,
        else_items: Vec<MakeItem>,
        span: Span,
    },

    /// Include directive: include other.mk
    Include {
        path: String,
        optional: bool,  // -include vs include
        span: Span,
    },

    /// Function call: $(call func, args)
    FunctionCall {
        name: String,
        args: Vec<MakeExpr>,
        span: Span,
    },

    /// Comment
    Comment {
        text: String,
        span: Span,
    },
}

/// Variable flavors (different assignment operators)
#[derive(Debug, Clone, PartialEq)]
pub enum VarFlavor {
    Recursive,        // =  (recursive expansion)
    Simple,           // := (simple expansion)
    Conditional,      // ?= (conditional assignment)
    Append,           // += (append)
    Shell,            // != (shell assignment)
}

/// Make expressions (right-hand side of assignments, function args)
#[derive(Debug, Clone, PartialEq)]
pub enum MakeExpr {
    /// Literal string
    Literal(String),

    /// Variable reference: $(VAR) or ${VAR}
    VarRef(String),

    /// Function call: $(func arg1, arg2)
    Function {
        name: String,
        args: Vec<MakeExpr>,
    },

    /// Shell command: $(shell command)
    Shell(String),

    /// Wildcard: $(wildcard *.c)
    Wildcard(String),

    /// Concatenation of expressions
    Concat(Vec<MakeExpr>),
}

/// Conditional types
#[derive(Debug, Clone, PartialEq)]
pub enum MakeCondition {
    IfEq(MakeExpr, MakeExpr),
    IfNeq(MakeExpr, MakeExpr),
    IfDef(String),
    IfNDef(String),
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
```

---

## 🧼 Purification Rules

### Determinism Rules

| Rule | Problem | Purified Solution |
|------|---------|------------------|
| **NO_TIMESTAMPS** | `$(shell date +%s)` | Use explicit version: `1.0.0` |
| **NO_RANDOM** | `$(shell echo $$RANDOM)` | Use seed-based: `generate_id(42)` |
| **NO_PID** | `$(shell echo $$$$)` | Use fixed IDs: `build-id-12345` |
| **NO_UNORDERED_FIND** | `$(shell find . -name "*.c")` | Explicit sorted list: `a.c b.c c.c` |
| **NO_WILDCARD_IN_RECIPES** | `rm *.o` in recipe | `rm $(OBJECTS)` with explicit var |

### Idempotency Rules

| Rule | Problem | Purified Solution |
|------|---------|------------------|
| **REQUIRE_PHONY** | Target without .PHONY | Add `.PHONY: target` |
| **MKDIR_P** | `mkdir dir` | `mkdir -p dir` |
| **RM_F** | `rm file` | `rm -f file` |
| **CP_NO_PROMPT** | `cp source dest` | `cp -f source dest` |
| **IDEMPOTENT_DEPLOY** | Destructive operations | Safe, repeatable operations |

### Portability Rules

| Rule | Problem | Purified Solution |
|------|---------|------------------|
| **NO_GNU_EXTENSIONS** | `$(wildcard)`, `$(shell)` | POSIX alternatives or explicit lists |
| **PORTABLE_SHELL** | Bash-isms in recipes | POSIX sh syntax |
| **PATH_SEPARATORS** | Hardcoded `/` or `\` | Variable-based: `$(PATH_SEP)` |
| **NO_GNU_FUNCTIONS** | `$(filter-out)`, `$(eval)` | Alternatives or compatibility notes |

### Safety Rules

| Rule | Problem | Purified Solution |
|------|---------|------------------|
| **NO_EVAL_INJECTION** | `$(eval $(shell ...))` | Static definitions only |
| **NO_UNQUOTED_VARS** | `rm $(FILES)` | Explicit, verified lists |
| **NO_RECURSIVE_MAKE** | `$(MAKE) -C dir` | Document, but allow with warnings |
| **EXPLICIT_DEPENDENCIES** | Missing prerequisites | Track and verify all deps |

---

## 🧪 EXTREME TDD Implementation

### Phase 1: RED (Write Failing Test)

**Example: .PHONY Target Rule**

```rust
// rash/src/make_parser/tests.rs

#[test]
fn test_phony_target_declaration() {
    // ARRANGE: Makefile with .PHONY target
    let makefile = r#"
.PHONY: test
test:
	cargo test
"#;

    // ACT: Parse to AST
    let ast = parse_makefile(makefile).unwrap();

    // ASSERT: Target marked as phony
    let test_target = ast.items.iter()
        .filter_map(|item| match item {
            MakeItem::Target { name, phony, .. } if name == "test" => Some(phony),
            _ => None,
        })
        .next()
        .expect("test target should exist");

    assert!(*test_target, "test target should be marked as .PHONY");

    // ASSERT: Purified output preserves .PHONY
    let purified = generate_purified_makefile(&ast);
    assert!(purified.contains(".PHONY: test"));
}
```

**Run test** → ❌ FAILS (AST types don't exist yet)

### Phase 2: GREEN (Implement)

```rust
// rash/src/make_parser/ast.rs
#[derive(Debug, Clone, PartialEq)]
pub enum MakeItem {
    Target {
        name: String,
        prerequisites: Vec<String>,
        recipe: Vec<String>,
        phony: bool,  // ✅ Added
        span: Span,
    },
    // ... other variants
}

// rash/src/make_parser/parser.rs
impl Parser {
    fn parse_target(&mut self) -> Result<MakeItem, ParseError> {
        let name = self.parse_target_name()?;
        let prerequisites = self.parse_prerequisites()?;
        let recipe = self.parse_recipe()?;

        // ✅ Check if target was declared as .PHONY
        let phony = self.phony_targets.contains(&name);

        Ok(MakeItem::Target {
            name,
            prerequisites,
            recipe,
            phony,
            span: self.current_span(),
        })
    }
}

// rash/src/make_parser/generators.rs
pub fn generate_purified_makefile(ast: &MakeAst) -> String {
    let mut output = String::new();

    // ✅ Generate .PHONY declarations
    let phony_targets: Vec<_> = ast.items.iter()
        .filter_map(|item| match item {
            MakeItem::Target { name, phony: true, .. } => Some(name.as_str()),
            _ => None,
        })
        .collect();

    if !phony_targets.is_empty() {
        output.push_str(&format!(".PHONY: {}\n\n", phony_targets.join(" ")));
    }

    // Generate targets
    for item in &ast.items {
        output.push_str(&generate_item(item));
    }

    output
}
```

**Run test** → ✅ PASSES

### Phase 3: REFACTOR (Clean Up)

- Extract helper functions
- Reduce complexity
- Document edge cases
- Run full test suite → ✅ ALL PASS

### Phase 4: PROPERTY TESTING

```rust
// rash/src/make_parser/tests.rs

use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_phony_targets_always_declared(
        target_name in "[a-z]{1,10}",
        recipe in "cargo (test|build|clean)"
    ) {
        let makefile = format!(".PHONY: {}\n{}:\n\t{}\n", target_name, target_name, recipe);
        let ast = parse_makefile(&makefile).unwrap();
        let purified = generate_purified_makefile(&ast);

        // PROPERTY: .PHONY declaration always in output
        prop_assert!(purified.contains(&format!(".PHONY: {}", target_name)));
    }

    #[test]
    fn prop_purification_is_deterministic(
        makefile in make_generator::arbitrary_makefile()
    ) {
        let ast = parse_makefile(&makefile).unwrap();
        let purified1 = generate_purified_makefile(&ast);
        let purified2 = generate_purified_makefile(&ast);

        // PROPERTY: Determinism - byte-identical output
        prop_assert_eq!(purified1, purified2);
    }
}
```

### Phase 5: DOCUMENTATION

Update `MAKE-INGESTION-ROADMAP.yaml`:

```yaml
- id: "PHONY-001"
  title: "Document .PHONY target declarations"
  status: "completed"
  version: "v1.4.0"
  input: ".PHONY: test\ntest:\n\tcargo test"
  rust: "fn test() { run_cargo_test(); }"
  purified: ".PHONY: test\ntest:\n\tcargo test"
  test_name: "test_phony_target_declaration"
  tests_added:
    - "test_phony_target_declaration (unit test)"
    - "prop_phony_targets_always_declared (property test, 100 cases)"
  notes: ".PHONY declarations preserved and enforced. Verified with EXTREME TDD."
  implementation:
    modules:
      - "rash/src/make_parser/ast.rs (added phony field to Target)"
      - "rash/src/make_parser/parser.rs (parse .PHONY declarations)"
      - "rash/src/make_parser/generators.rs (generate .PHONY output)"
    lines_of_code: 45
```

---

## 🧪 Testing Strategy

### Test Pyramid

```
              ┌─────────────┐
              │ Integration │  ← 10% (Real Makefiles from projects)
              │    Tests    │
              └─────────────┘
                    ▲
            ┌───────────────┐
            │   Property    │  ← 30% (Generative, 100+ cases each)
            │    Tests      │
            └───────────────┘
                    ▲
         ┌──────────────────────┐
         │   Mutation Tests     │  ← 20% (cargo-mutants, >90% kill rate)
         └──────────────────────┘
                    ▲
    ┌────────────────────────────────┐
    │        Unit Tests              │  ← 40% (EXTREME TDD, one per feature)
    └────────────────────────────────┘
```

### Unit Tests (EXTREME TDD)

**Test every GNU Make feature**:

```rust
#[test]
fn test_simple_assignment() {
    let makefile = "VAR = value\n";
    let ast = parse_makefile(makefile).unwrap();
    assert_eq!(ast.items.len(), 1);
}

#[test]
fn test_recursive_vs_simple_expansion() {
    let makefile = r#"
VAR1 = $(VAR2)   # Recursive
VAR2 := value    # Simple
"#;
    let ast = parse_makefile(makefile).unwrap();
    // Verify correct flavor assignment
}

#[test]
fn test_target_with_prerequisites() {
    let makefile = r#"
all: build test
"#;
    let ast = parse_makefile(makefile).unwrap();
    // Verify prerequisites parsed correctly
}

#[test]
fn test_pattern_rule() {
    let makefile = r#"
%.o: %.c
	$(CC) -c $< -o $@
"#;
    let ast = parse_makefile(makefile).unwrap();
    // Verify pattern rule structure
}
```

### Property Tests

```rust
proptest! {
    // Property: All Makefiles parse or produce a clear error
    #[test]
    fn prop_parse_or_error(makefile in make_generator::any()) {
        let result = parse_makefile(&makefile);
        prop_assert!(result.is_ok() || result.unwrap_err().is_clear());
    }

    // Property: Purified Makefiles are idempotent
    #[test]
    fn prop_purified_is_idempotent(makefile in make_generator::any()) {
        if let Ok(ast) = parse_makefile(&makefile) {
            let purified1 = generate_purified_makefile(&ast);
            let ast2 = parse_makefile(&purified1).unwrap();
            let purified2 = generate_purified_makefile(&ast2);
            prop_assert_eq!(purified1, purified2);
        }
    }

    // Property: No non-deterministic constructs in purified output
    #[test]
    fn prop_no_nondeterminism_in_purified(makefile in make_generator::any()) {
        if let Ok(ast) = parse_makefile(&makefile) {
            let purified = generate_purified_makefile(&ast);
            prop_assert!(!purified.contains("$(shell date"));
            prop_assert!(!purified.contains("$RANDOM"));
            prop_assert!(!purified.contains("$$$$"));
        }
    }
}
```

### Mutation Tests

**Target: ≥90% mutation kill rate**

```bash
# Run mutation testing on Make parser
cargo mutants --file src/make_parser/parser.rs -- --lib

# Run mutation testing on purification logic
cargo mutants --file src/make_transpiler/purification.rs -- --lib
```

**Example Mutants**:
```rust
// Original
if target.phony {
    output.push_str(".PHONY: ");
}

// Mutant 1: Changed condition
if !target.phony {  // ❌ Should be caught by tests
    output.push_str(".PHONY: ");
}

// Mutant 2: Removed condition
{  // ❌ Should be caught by tests
    output.push_str(".PHONY: ");
}
```

### Integration Tests

**Test against real-world Makefiles**:

```rust
#[test]
fn test_real_makefile_linux_kernel() {
    // Simplified Linux kernel Makefile
    let makefile = include_str!("../fixtures/makefiles/linux-kernel-simplified.mk");
    let ast = parse_makefile(makefile).unwrap();
    let purified = generate_purified_makefile(&ast);

    // Verify key targets preserved
    assert!(purified.contains(".PHONY: clean"));
    assert!(purified.contains(".PHONY: all"));

    // Verify purification applied
    assert!(!purified.contains("$(shell date"));
}

#[test]
fn test_real_makefile_rust_project() {
    let makefile = include_str!("../fixtures/makefiles/rust-cargo-wrapper.mk");
    let ast = parse_makefile(makefile).unwrap();
    let purified = generate_purified_makefile(&ast);

    // Verify standard Rust targets
    assert!(purified.contains(".PHONY: build test clean"));
}
```

---

## 🚦 Quality Gates

### Test Coverage

```toml
# .cargo/config.toml
[alias]
coverage = "llvm-cov --all-features --workspace --lcov --output-path lcov.info"
```

**Target**: >85% line coverage

### Mutation Testing

```bash
# Quality gate: >90% mutation kill rate
cargo mutants -- --lib

# Expected output:
# Mutations killed: 145/160 (90.6%)
```

### Complexity

**Target**: Cyclomatic complexity <10 per function

```bash
pmat analyze complexity --max 10 --path src/make_parser/
```

### Linting

```bash
cargo clippy -- -D warnings

# Makefile-specific lints
pmat analyze makefile Makefile --rules all
```

---

## 🗺️ Roadmap

### MAKE-INGESTION-ROADMAP.yaml Structure

```yaml
roadmap:
  title: "GNU Make Ingestion Roadmap - EXTREME TDD"
  goal: "Transform Makefiles to safe, deterministic, idempotent builds"
  methodology: "EXTREME TDD (RED-GREEN-REFACTOR-PROPERTY-DOCUMENTATION)"
  reference: "GNU Make Manual 4.4"
  status: "IN_PROGRESS"

  statistics:
    total_tasks: 150
    completed: 0
    in_progress: 0
    coverage_percent: 0

chapters:
  - id: 1
    name: "Overview of make"
    tasks:
      - id: "OVERVIEW-001"
        title: "Document basic make invocation"
        status: "pending"
        input: "make all"
        rust: "fn all() { build(); test(); }"
        purified: ".PHONY: all\nall: build test"
        test_name: "test_make_invocation"

  - id: 2
    name: "An Introduction to Makefiles"
    sections:
      - id: "2.1"
        name: "What a Rule Looks Like"
        tasks:
          - id: "RULE-SYNTAX-001"
            title: "Document basic rule syntax"
            status: "pending"
            priority: "HIGH"
            input: "target: prerequisites\n\trecipe"
            rust: "fn target() { recipe(); }"
            purified: ".PHONY: target\ntarget: prerequisites\n\trecipe"
            test_name: "test_basic_rule_syntax"

  - id: 3
    name: "Writing Makefiles"
    sections:
      - id: "3.3"
        name: "Including Other Makefiles"
        tasks:
          - id: "INCLUDE-001"
            title: "Document include directive"
            status: "pending"
            input: "include common.mk"
            rust: "mod common;"
            purified: "include common.mk"
            test_name: "test_include_directive"

  - id: 4
    name: "Writing Rules"
    sections:
      - id: "4.5"
        name: "Phony Targets"
        tasks:
          - id: "PHONY-001"
            title: "Document .PHONY declarations"
            status: "pending"
            priority: "HIGH"
            input: ".PHONY: clean\nclean:\n\trm -f *.o"
            rust: "fn clean() { remove_files(\"*.o\"); }"
            purified: ".PHONY: clean\nclean:\n\trm -f *.o"
            test_name: "test_phony_declarations"
            notes: "Critical for purification - ensures non-file targets work correctly"

      - id: "4.11"
        name: "Static Pattern Rules"
        tasks:
          - id: "PATTERN-001"
            title: "Document pattern rules"
            status: "pending"
            input: "%.o: %.c\n\t$(CC) -c $< -o $@"
            rust: "fn compile_c_file(src: &str) -> String { ... }"
            purified: "%.o: %.c\n\t$(CC) -c $< -o $@"
            test_name: "test_pattern_rules"

  - id: 6
    name: "How to Use Variables"
    sections:
      - id: "6.2"
        name: "The Two Flavors of Variables"
        tasks:
          - id: "VAR-FLAVOR-001"
            title: "Document recursive assignment (=)"
            status: "pending"
            input: "VAR = $(OTHER)"
            rust: "lazy_static! { static ref VAR: String = OTHER.clone(); }"
            purified: "VAR = $(OTHER)"
            test_name: "test_recursive_assignment"

          - id: "VAR-FLAVOR-002"
            title: "Document simple assignment (:=)"
            status: "pending"
            input: "VAR := value"
            rust: "const VAR: &str = \"value\";"
            purified: "VAR := value"
            test_name: "test_simple_assignment"

  - id: 8
    name: "Functions for Transforming Text"
    sections:
      - id: "8.13"
        name: "The shell Function"
        tasks:
          - id: "FUNC-SHELL-001"
            title: "Purify $(shell date) constructs"
            status: "pending"
            priority: "CRITICAL"
            input: "RELEASE := $(shell date +%s)"
            rust: "const RELEASE: &str = \"1.0.0\";"
            purified: "RELEASE := 1.0.0"
            test_name: "test_purify_shell_date"
            notes: "Non-deterministic - must be purified to explicit value"
            purification_rule: "NO_TIMESTAMPS"

completed_features:
  - title: "Basic Makefile parsing"
    version: "v1.4.0"
    tasks: []
    tests: 0
    methodology: "EXTREME TDD + Property Testing"

high_priority_tasks:
  - id: "PHONY-001"
    title: ".PHONY declarations"
    priority: 1
  - id: "RULE-SYNTAX-001"
    title: "Basic rule syntax"
    priority: 2
  - id: "VAR-FLAVOR-001"
    title: "Variable assignment"
    priority: 3
  - id: "FUNC-SHELL-001"
    title: "Purify shell functions"
    priority: 4

status:
  overall: "IN_PROGRESS"
  completion_percent: 0
  completed_tasks: 0
  total_tasks: 150
  methodology: "EXTREME TDD (RED-GREEN-REFACTOR-PROPERTY) + Mutation Testing"
  goal: "100% GNU Make manual coverage"
  target_version: "v2.0.0"
```

### Implementation Phases

**Phase 1: Foundation (v1.4.0)**
- Basic Makefile parsing (rules, variables, targets)
- Simple AST structure
- Purified Makefile generation
- Core property tests
- 10-20% manual coverage

**Phase 2: Core Features (v1.5.0)**
- .PHONY support
- Pattern rules
- Variable flavors (=, :=, ?=, +=)
- Conditionals (ifeq, ifdef)
- Function calls
- 40-50% manual coverage

**Phase 3: Advanced Features (v1.6.0)**
- Include directives
- Automatic variables ($@, $<, $^)
- Built-in functions (wildcard, shell, foreach)
- Purification engine
- 70-80% manual coverage

**Phase 4: Purification & Safety (v1.7.0)**
- Complete purification rules
- Determinism enforcement
- Idempotency checks
- Portability analysis
- Linting integration
- 90-95% manual coverage

**Phase 5: Production Ready (v2.0.0)**
- 100% GNU Make manual coverage
- Comprehensive test suite (>90% mutation kill rate)
- Real-world Makefile validation
- Documentation complete
- Integration with paiml-mcp-agent-toolkit

---

## 📊 Success Metrics

### Code Quality
- ✅ >85% test coverage (llvm-cov)
- ✅ >90% mutation kill rate (cargo-mutants)
- ✅ Complexity <10 (pmat complexity)
- ✅ 0 clippy warnings
- ✅ 100% proptest property preservation

### Feature Completeness
- ✅ 100% GNU Make manual coverage
- ✅ All purification rules implemented
- ✅ All linting rules implemented
- ✅ Bidirectional transformation (Make ↔ Rust)

### Performance
- ✅ Parse 10,000-line Makefile in <100ms
- ✅ Memory usage <50MB for large Makefiles
- ✅ Incremental parsing support

### Real-World Validation
- ✅ Parse Linux kernel Makefile
- ✅ Parse GNU Make's own Makefile
- ✅ Parse 100+ open-source project Makefiles
- ✅ Zero regressions in purified output

---

## 🎓 Developer Guide

### Getting Started

```bash
# 1. Create feature branch
git checkout -b make-parser-foundation

# 2. Create AST types (RED phase)
# Write failing test first in rash/src/make_parser/tests.rs

# 3. Implement parser (GREEN phase)
# Add parsing logic in rash/src/make_parser/parser.rs

# 4. Generate purified output (GREEN phase)
# Add generation in rash/src/make_parser/generators.rs

# 5. Run tests
cargo test --lib make_parser

# 6. Add property tests (PROPERTY TESTING phase)
# Add proptest tests in tests.rs

# 7. Run mutation tests
cargo mutants --file src/make_parser/parser.rs -- --lib

# 8. Update roadmap (DOCUMENTATION phase)
# Mark task as completed in MAKE-INGESTION-ROADMAP.yaml

# 9. Commit with conventional commit message
git commit -m "feat: Add basic Makefile rule parsing (RULE-SYNTAX-001)"
```

### STOP THE LINE Protocol

**If you discover a bug during GNU Make manual validation**:

```
🚨 STOP THE LINE - P0 BUG DETECTED 🚨

1. HALT all validation work
2. Create P0 ticket in MAKE-INGESTION-ROADMAP.yaml
3. Write failing test (RED)
4. Implement fix (GREEN)
5. Refactor (REFACTOR)
6. Add property tests (PROPERTY TESTING)
7. Run mutation tests
8. Update documentation (DOCUMENTATION)
9. ONLY THEN resume validation
```

### Example Workflow

**Task: Implement .PHONY support**

```bash
# RED: Write failing test
cat >> rash/src/make_parser/tests.rs << 'EOF'
#[test]
fn test_phony_declarations() {
    let makefile = ".PHONY: clean\nclean:\n\trm -f *.o";
    let ast = parse_makefile(makefile).unwrap();
    assert!(ast.has_phony_target("clean"));
}
EOF

# Run test - should FAIL
cargo test test_phony_declarations
# ❌ FAILED: parse_makefile not implemented

# GREEN: Implement parsing
# Edit rash/src/make_parser/parser.rs
# Edit rash/src/make_parser/ast.rs

# Run test - should PASS
cargo test test_phony_declarations
# ✅ PASSED

# REFACTOR: Clean up code
cargo clippy
cargo test

# PROPERTY TESTING: Add property tests
cat >> rash/src/make_parser/tests.rs << 'EOF'
proptest! {
    #[test]
    fn prop_phony_preserved(target in "[a-z]+") {
        let makefile = format!(".PHONY: {}\n{}:\n\techo test", target, target);
        let ast = parse_makefile(&makefile).unwrap();
        let purified = generate_purified_makefile(&ast);
        prop_assert!(purified.contains(&format!(".PHONY: {}", target)));
    }
}
EOF

cargo test prop_phony_preserved

# Mutation testing
cargo mutants --file src/make_parser/parser.rs -- --lib
# Target: >90% kill rate

# DOCUMENTATION: Update roadmap
# Edit docs/MAKE-INGESTION-ROADMAP.yaml
# Mark PHONY-001 as "completed"

git add -A
git commit -m "feat: Implement .PHONY target support (PHONY-001)

- Added .PHONY parsing in parser.rs
- Updated AST with phony field
- Preserved .PHONY in purified output
- Added 1 unit test + 1 property test
- Mutation kill rate: 92%

EXTREME TDD: RED→GREEN→REFACTOR→PROPERTY→DOCUMENTATION
Task: PHONY-001"
```

---

## 🔗 Integration with Existing Tools

### Integration with paiml-mcp-agent-toolkit

```rust
// Expose Make analysis via MCP tools
pub struct MakeAnalyzer {
    parser: MakeParser,
    linter: MakeLinter,
    purifier: MakePurifier,
}

impl MakeAnalyzer {
    pub fn analyze_makefile(&self, path: &str) -> AnalysisReport {
        let makefile = std::fs::read_to_string(path)?;
        let ast = self.parser.parse(&makefile)?;

        let lint_results = self.linter.lint(&ast);
        let purified = self.purifier.purify(&ast);

        AnalysisReport {
            quality_score: lint_results.score(),
            violations: lint_results.violations,
            purified_makefile: purified,
            rust_code: self.transpile_to_rust(&ast),
        }
    }
}
```

### CLI Interface

```bash
# Analyze Makefile
rash make analyze Makefile

# Purify Makefile
rash make purify Makefile --output Makefile.pure

# Convert to Rust
rash make to-rust Makefile --output build.rs

# Lint Makefile
rash make lint Makefile --fix
```

---

## 📚 References

1. **GNU Make Manual**: https://www.gnu.org/software/make/manual/make.html
2. **POSIX make**: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/make.html
3. **Rash Bash Implementation**: `/home/noahgift/src/bashrs/rash/src/bash_parser/`
4. **PMAT Makefile Linting**: `/home/noahgift/src/paiml-mcp-agent-toolkit/docs/makefile-linting-guide.md`

---

## 🎯 Next Steps

1. **Review this specification** with the team
2. **Create MAKE-INGESTION-ROADMAP.yaml** following the structure above
3. **Start with Phase 1**: Basic parsing, EXTREME TDD
4. **Validate against GNU Make Manual** chapter by chapter
5. **Maintain >90% mutation kill rate** throughout
6. **Target v2.0.0** for production-ready Make purification

---

**Specification Version**: 1.0.0
**Last Updated**: 2025-10-15
**Status**: READY FOR IMPLEMENTATION
**Methodology**: EXTREME TDD + Property Testing + Mutation Testing + GNU Manual Validation
