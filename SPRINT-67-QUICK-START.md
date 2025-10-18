# Sprint 67 Quick Start - Purification Engine 🚀

## TL;DR - Start Here

```bash
# 1. Verify current state
cargo test --lib  # Should show: test result: ok. 1380 passed

# 2. Read context
cat SPRINT-66-HANDOFF.md  # Previous sprint (Phase 2 complete!)
cat PROJECT-STATE-2025-10-18-SPRINT-66.md  # Current project state

# 3. Start EXTREME TDD - RED phase
# Create purification module: rash/src/make_parser/purify.rs
# Write failing tests for purification transformations

# 4. Run tests (expect FAILURES - RED phase)
cargo test --lib test_PURIFY

# 5. Implement purification engine (GREEN phase)
# Add transformation rules to purify.rs
```

## What You're Building

**Goal**: Implement purification engine that auto-fixes non-deterministic patterns detected by semantic analysis.

**Why Critical**: Phase 2 detects issues - Phase 3 fixes them automatically!

**Estimated Time**: 10-12 hours

## Purification Transformations

### Transformation 1: Wildcard Sorting

**Input** (non-deterministic):
```makefile
FILES := $(wildcard *.c)
```

**Output** (purified):
```makefile
FILES := $(sort $(wildcard *.c))
```

**Rule**: Wrap `$(wildcard ...)` with `$(sort ...)`

### Transformation 2: Shell Find Sorting

**Input** (non-deterministic):
```makefile
FILES := $(shell find src -name '*.c')
```

**Output** (purified):
```makefile
FILES := $(sort $(shell find src -name '*.c'))
```

**Rule**: Wrap `$(shell find ...)` with `$(sort ...)`

### Transformation 3: Nested Wildcard in Filter

**Input** (non-deterministic):
```makefile
OBJS := $(filter %.o, $(wildcard *.c))
```

**Output** (purified):
```makefile
OBJS := $(filter %.o, $(sort $(wildcard *.c)))
```

**Rule**: Find nested `$(wildcard)` and wrap with `$(sort)`

### Transformation 4: Nested Wildcard in FOREACH

**Input** (non-deterministic):
```makefile
OBJS := $(foreach file, $(wildcard *.c), $(file:.c=.o))
```

**Output** (purified):
```makefile
OBJS := $(foreach file, $(sort $(wildcard *.c)), $(file:.c=.o))
```

**Rule**: Find wildcard in foreach list, wrap with sort

### Transformation 5: Nested Wildcard in CALL

**Input** (non-deterministic):
```makefile
FILES := $(call process, $(wildcard *.c))
```

**Output** (purified):
```makefile
FILES := $(call process, $(sort $(wildcard *.c)))
```

**Rule**: Find wildcard in call args, wrap with sort

### Transformation 6: Shell Date (Manual Required)

**Input** (non-deterministic):
```makefile
RELEASE := release-$(shell date +%s)
```

**Output** (comment + suggestion):
```makefile
# PURIFY: Replace $(shell date) with fixed version
# RELEASE := release-1.0.0
RELEASE := release-$(shell date +%s)
```

**Rule**: Cannot auto-fix - add comment with suggestion

### Transformation 7: $RANDOM (Manual Required)

**Input** (non-deterministic):
```makefile
SESSION_ID := session-$RANDOM
```

**Output** (comment + suggestion):
```makefile
# PURIFY: Replace $RANDOM with deterministic value
# SESSION_ID := session-12345
SESSION_ID := session-$RANDOM
```

**Rule**: Cannot auto-fix - add comment with suggestion

## Implementation Strategy

### Module Structure

```
rash/src/make_parser/
├── ast.rs          (existing - AST definitions)
├── parser.rs       (existing - parser)
├── semantic.rs     (existing - issue detection)
├── purify.rs       (NEW - purification engine)
└── tests.rs        (existing - add purification tests)
```

### Core Functions to Implement

```rust
// rash/src/make_parser/purify.rs

use crate::make_parser::ast::MakeAst;
use crate::make_parser::semantic::{analyze_makefile, SemanticIssue};

/// Purify a Makefile AST by fixing non-deterministic patterns
pub fn purify_makefile(ast: &MakeAst) -> PurificationResult {
    // 1. Run semantic analysis to find issues
    let issues = analyze_makefile(ast);

    // 2. Apply transformations for each issue
    let transformations = plan_transformations(&issues);

    // 3. Apply safe transformations
    let purified_ast = apply_transformations(ast, &transformations);

    // 4. Return result with report
    PurificationResult {
        ast: purified_ast,
        transformations_applied: transformations.len(),
        issues_fixed: count_fixed(&transformations),
        manual_fixes_needed: count_manual(&transformations),
    }
}

/// Plan which transformations to apply
fn plan_transformations(issues: &[SemanticIssue]) -> Vec<Transformation> {
    let mut transformations = Vec::new();

    for issue in issues {
        match issue.rule.as_str() {
            "NO_WILDCARD" => {
                transformations.push(Transformation::WrapWithSort {
                    pattern: "$(wildcard",
                    safe: true,
                });
            }
            "NO_UNORDERED_FIND" => {
                transformations.push(Transformation::WrapWithSort {
                    pattern: "$(shell find",
                    safe: true,
                });
            }
            "NO_TIMESTAMPS" | "NO_RANDOM" => {
                transformations.push(Transformation::AddComment {
                    rule: issue.rule.clone(),
                    suggestion: issue.suggestion.clone(),
                    safe: false,  // Manual fix required
                });
            }
            _ => {}
        }
    }

    transformations
}

/// Apply transformations to AST
fn apply_transformations(
    ast: &MakeAst,
    transformations: &[Transformation]
) -> MakeAst {
    let mut purified = ast.clone();

    for transformation in transformations {
        match transformation {
            Transformation::WrapWithSort { pattern, .. } => {
                wrap_with_sort(&mut purified, pattern);
            }
            Transformation::AddComment { rule, suggestion, .. } => {
                add_purify_comment(&mut purified, rule, suggestion);
            }
        }
    }

    purified
}

/// Wrap pattern with $(sort ...)
fn wrap_with_sort(ast: &mut MakeAst, pattern: &str) {
    for item in &mut ast.items {
        if let MakeItem::Variable { value, .. } = item {
            if value.contains(pattern) {
                *value = wrap_pattern_with_sort(value, pattern);
            }
        }
    }
}

/// Helper: Wrap specific pattern with sort
fn wrap_pattern_with_sort(value: &str, pattern: &str) -> String {
    // Find pattern (e.g., "$(wildcard *.c)")
    // Wrap with "$(sort ...)"

    // Simple approach: Find opening and matching closing paren
    if let Some(start) = value.find(pattern) {
        // TODO: Implement proper parenthesis matching
        // For now, simple replacement
        value.replace(pattern, &format!("$(sort {}", pattern))
            .replace("))", "))")  // Adjust closing parens
    } else {
        value.to_string()
    }
}
```

### Data Structures

```rust
#[derive(Debug, Clone)]
pub struct PurificationResult {
    pub ast: MakeAst,
    pub transformations_applied: usize,
    pub issues_fixed: usize,
    pub manual_fixes_needed: usize,
}

#[derive(Debug, Clone)]
pub enum Transformation {
    WrapWithSort {
        pattern: String,
        safe: bool,
    },
    AddComment {
        rule: String,
        suggestion: String,
        safe: bool,
    },
}
```

## EXTREME TDD Workflow

### Phase 1: RED - Write Failing Tests

```rust
// In rash/src/make_parser/tests.rs

#[test]
fn test_PURIFY_001_wrap_wildcard_with_sort() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::purify::purify_makefile;

    // ARRANGE: Non-deterministic wildcard
    let makefile = "FILES := $(wildcard *.c)";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Wildcard wrapped with sort
    assert_eq!(result.transformations_applied, 1);
    assert_eq!(result.issues_fixed, 1);

    // Check purified output
    let purified_var = &result.ast.items[0];
    if let MakeItem::Variable { value, .. } = purified_var {
        assert!(value.contains("$(sort $(wildcard"));
    } else {
        panic!("Expected Variable");
    }
}

#[test]
fn test_PURIFY_002_nested_wildcard_in_filter() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::purify::purify_makefile;

    // ARRANGE: Nested wildcard in filter
    let makefile = "OBJS := $(filter %.o, $(wildcard *.c))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Inner wildcard wrapped with sort
    let purified_var = &result.ast.items[0];
    if let MakeItem::Variable { value, .. } = purified_var {
        assert!(value.contains("$(filter %.o, $(sort $(wildcard"));
    } else {
        panic!("Expected Variable");
    }
}

#[test]
fn test_PURIFY_003_shell_date_adds_comment() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::purify::purify_makefile;

    // ARRANGE: Shell date (cannot auto-fix)
    let makefile = "RELEASE := release-$(shell date +%s)";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Comment added, manual fix needed
    assert_eq!(result.manual_fixes_needed, 1);

    // Check comment was added
    // (Implementation detail: how to represent comments in AST?)
}
```

### Phase 2: GREEN - Implement to Pass Tests

1. Create `rash/src/make_parser/purify.rs`
2. Implement core functions
3. Run tests: `cargo test --lib test_PURIFY`
4. Iterate until all tests pass

### Phase 3: REFACTOR

- Extract helper functions
- Ensure cyclomatic complexity <10
- Add documentation
- Clean up code structure

### Phase 4: PROPERTY Testing

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_purify_wildcard_always_adds_sort(
        pattern in "[a-z*.]+",
    ) {
        let makefile = format!("X := $(wildcard {})", pattern);
        let ast = parse_makefile(&makefile).unwrap();
        let result = purify_makefile(&ast);

        let purified_var = &result.ast.items[0];
        if let MakeItem::Variable { value, .. } = purified_var {
            prop_assert!(value.contains("$(sort $(wildcard"));
        }
    }
}
```

### Phase 5: MUTATION Testing

```bash
cargo mutants --file rash/src/make_parser/purify.rs -- --lib

# TARGET: ≥90% kill rate
```

## Success Criteria

- [ ] ✅ purify.rs module created
- [ ] ✅ Core purification functions implemented
- [ ] ✅ 15-20 RED tests written and passing
- [ ] ✅ Wildcard → sort(wildcard) transformation works
- [ ] ✅ Shell find → sort(shell find) transformation works
- [ ] ✅ Nested patterns purified correctly
- [ ] ✅ Manual fix patterns add comments
- [ ] ✅ Property tests passing (100+ generated cases)
- [ ] ✅ Mutation score ≥90%
- [ ] ✅ Zero regressions (1,380+ tests still pass)
- [ ] ✅ Sprint 67 handoff created

## Expected Timeline

### RED Phase (3-4 hours)
- Hour 1: Create purify.rs module skeleton
- Hour 2-3: Write 15-20 failing tests
- Hour 4: Verify all tests fail (RED confirmed)

### GREEN Phase (4-5 hours)
- Hour 5-6: Implement wrap_with_sort transformation
- Hour 7: Implement nested pattern detection
- Hour 8: Implement comment addition for manual fixes
- Hour 9: All tests passing (GREEN confirmed)

### REFACTOR Phase (1-2 hours)
- Hour 10: Extract helpers, reduce complexity
- Hour 11: Add documentation, clean up

### PROPERTY + MUTATION (1-2 hours)
- Hour 12: Add property tests, run mutation testing

## Challenges & Solutions

### Challenge 1: Parenthesis Matching

**Problem**: Finding matching closing paren for `$(wildcard *.c)`

**Solution**: Implement proper paren matching algorithm
```rust
fn find_matching_paren(s: &str, start: usize) -> Option<usize> {
    let mut depth = 0;
    for (i, ch) in s[start..].chars().enumerate() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(start + i);
                }
            }
            _ => {}
        }
    }
    None
}
```

### Challenge 2: Nested Function Detection

**Problem**: Finding `$(wildcard)` inside `$(filter %.o, $(wildcard *.c))`

**Solution**: Recursive pattern matching or regex-based extraction

### Challenge 3: AST Modification

**Problem**: AST is immutable, how to create purified version?

**Solution**: Clone AST, modify clone, return new AST

### Challenge 4: Comment Representation

**Problem**: AST doesn't have Comment node type

**Solution**:
- Option 1: Add Comment variant to MakeItem enum
- Option 2: Store comments in separate field
- Option 3: Emit comments during code generation (not in AST)

## Files to Create/Modify

**New Files**:
- `rash/src/make_parser/purify.rs` (main implementation)

**Modified Files**:
- `rash/src/make_parser/mod.rs` (add pub mod purify)
- `rash/src/make_parser/tests.rs` (add 15-20 purification tests)
- `rash/src/make_parser/ast.rs` (potentially add Comment variant)

## Quick Commands Reference

```bash
# Create new module
touch rash/src/make_parser/purify.rs

# Run purification tests
cargo test --lib test_PURIFY

# Run all tests
cargo test --lib

# Check test count
cargo test --lib 2>&1 | grep "test result"

# Mutation testing
cargo mutants --file rash/src/make_parser/purify.rs -- --lib
```

## After Sprint 67

Once purification engine is complete:

**Next Priorities**:
1. CLI Integration (Sprint 68) - `rash purify` command
2. Code generation (Sprint 69) - Emit purified Makefile as text
3. Phase 3 planning - Define next 50-100 tasks

---

**Ready to Start?** Create `rash/src/make_parser/purify.rs` and begin writing RED tests! 🚀

**Remember**: EXTREME TDD - write tests FIRST, verify they FAIL, then implement to make them PASS.
