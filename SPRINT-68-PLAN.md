# Sprint 68: Code Generation - Implementation Plan

**Date**: October 18, 2025 (continued session)
**Status**: 🚀 IN PROGRESS
**Phase**: Phase 4 - Code Generation (Makefile Emitter)
**Estimated Duration**: 4-6 hours

---

## Mission

Implement the code generator to emit purified Makefile text from a purified AST, enabling the complete end-to-end purification workflow:

```
Input Makefile → Parse → AST → Analyze → Purify → Generate → Purified Makefile
```

---

## Success Criteria

- [ ] Generate variable assignments with correct flavor (`:=`, `=`, `?=`, `+=`, `!=`)
- [ ] Generate targets with prerequisites and recipes
- [ ] Generate pattern rules (`%.o: %.c`)
- [ ] Generate conditional blocks (`ifeq`, `ifdef`, etc.)
- [ ] Preserve comments in output
- [ ] Handle `.PHONY` declarations
- [ ] Proper indentation (tabs for recipes)
- [ ] Property tests verify round-trip: `parse(generate(ast)) ≈ ast`
- [ ] Mutation testing ≥90% kill rate
- [ ] All 1,408+ existing tests still pass
- [ ] Zero regressions

---

## Architecture

### Module: `rash/src/make_parser/generators.rs`

```rust
pub fn generate_purified_makefile(ast: &MakeAst) -> String {
    // Main entry point - emit entire Makefile
}

fn generate_item(item: &MakeItem) -> String {
    // Emit single MakeItem
}

fn generate_variable(name: &str, value: &str, flavor: &VarFlavor) -> String {
    // Emit: VAR := value
}

fn generate_target(name: &str, prereqs: &[String], recipe: &[String], phony: bool) -> String {
    // Emit: target: prereq1 prereq2
    //         \trecipe line 1
    //         \trecipe line 2
}

fn generate_pattern_rule(target: &str, prereqs: &[String], recipe: &[String]) -> String {
    // Emit: %.o: %.c
    //         \t$(CC) -c $< -o $@
}

fn generate_conditional(condition: &MakeCondition, then_items: &[MakeItem], else_items: Option<&[MakeItem]>) -> String {
    // Emit: ifeq ($(VAR),value)
    //       ...
    //       else
    //       ...
    //       endif
}

fn generate_comment(text: &str) -> String {
    // Emit: # comment text
}

fn generate_include(path: &str, optional: bool) -> String {
    // Emit: include file.mk
    //   or: -include file.mk
}
```

---

## EXTREME TDD Workflow

### Phase 1: Variables (30 min)
**RED**: Write test `test_GENERATE_001_simple_variable`
```rust
#[test]
fn test_GENERATE_001_simple_variable() {
    let ast = MakeAst {
        items: vec![
            MakeItem::Variable {
                name: "CC".to_string(),
                value: "gcc".to_string(),
                flavor: VarFlavor::Simple,
                span: Span::dummy(),
            }
        ],
        metadata: MakeMetadata::new(),
    };

    let output = generate_purified_makefile(&ast);
    assert_eq!(output.trim(), "CC := gcc");
}
```
**Expected**: ❌ FAIL (generator returns empty string)

**GREEN**: Implement `generate_variable()` and `generate_purified_makefile()`
**Expected**: ✅ PASS

**REFACTOR**: Extract helper functions, ensure complexity <10

### Phase 2: Targets (45 min)
**RED**: Write test `test_GENERATE_002_target_with_recipe`
```rust
#[test]
fn test_GENERATE_002_target_with_recipe() {
    let ast = MakeAst {
        items: vec![
            MakeItem::Target {
                name: "build".to_string(),
                prerequisites: vec!["main.c".to_string()],
                recipe: vec!["gcc -o build main.c".to_string()],
                phony: false,
                span: Span::dummy(),
            }
        ],
        metadata: MakeMetadata::new(),
    };

    let output = generate_purified_makefile(&ast);
    let expected = "build: main.c\n\tgcc -o build main.c";
    assert_eq!(output.trim(), expected);
}
```
**Expected**: ❌ FAIL

**GREEN**: Implement `generate_target()`
**Expected**: ✅ PASS

**REFACTOR**: Clean up implementation

### Phase 3: Comments (15 min)
**RED**: Write test `test_GENERATE_003_comment_preservation`
**GREEN**: Implement `generate_comment()`
**REFACTOR**: Clean up

### Phase 4: Pattern Rules (30 min)
**RED**: Write test `test_GENERATE_004_pattern_rule`
**GREEN**: Implement `generate_pattern_rule()`
**REFACTOR**: Clean up

### Phase 5: Conditionals (45 min)
**RED**: Write test `test_GENERATE_005_conditional_ifeq`
**GREEN**: Implement `generate_conditional()`
**REFACTOR**: Clean up

### Phase 6: .PHONY (20 min)
**RED**: Write test `test_GENERATE_006_phony_targets`
**GREEN**: Enhance `generate_target()` for .PHONY
**REFACTOR**: Clean up

### Phase 7: Integration (30 min)
**RED**: Write test `test_GENERATE_007_complex_makefile`
**GREEN**: Ensure all pieces work together
**REFACTOR**: Final cleanup

### Phase 8: Property Testing (45 min)
Add property tests:
- `prop_GENERATE_001_roundtrip_variables`: parse(generate(var)) preserves var
- `prop_GENERATE_002_roundtrip_targets`: parse(generate(target)) preserves target
- `prop_GENERATE_003_idempotent_generation`: generate(generate(x)) == generate(x)

### Phase 9: Mutation Testing (30 min)
Run mutation testing:
```bash
cargo mutants --file rash/src/make_parser/generators.rs -- --lib
```
Target: ≥90% kill rate

---

## Test Cases

### Basic Tests (Unit)
1. **test_GENERATE_001_simple_variable**: `CC := gcc`
2. **test_GENERATE_002_target_with_recipe**: Basic target
3. **test_GENERATE_003_comment_preservation**: `# Comment`
4. **test_GENERATE_004_pattern_rule**: `%.o: %.c`
5. **test_GENERATE_005_conditional_ifeq**: `ifeq ($(VAR),value)`
6. **test_GENERATE_006_phony_targets**: `.PHONY` declaration
7. **test_GENERATE_007_complex_makefile**: Multiple items

### Edge Cases
8. **test_GENERATE_008_empty_recipe**: Target with no recipe
9. **test_GENERATE_009_multiple_prereqs**: Target with many prerequisites
10. **test_GENERATE_010_multiline_recipe**: Target with multiple recipe lines
11. **test_GENERATE_011_nested_conditionals**: Conditional within conditional
12. **test_GENERATE_012_all_var_flavors**: Test all 5 variable flavors

### Property Tests (Generative)
13. **prop_GENERATE_001_roundtrip_variables**: 100+ variable cases
14. **prop_GENERATE_002_roundtrip_targets**: 100+ target cases
15. **prop_GENERATE_003_idempotent_generation**: 100+ generation cases

---

## Quality Gates

Before marking Sprint 68 complete:

- [ ] ✅ All unit tests pass (≥15 tests)
- [ ] ✅ All property tests pass (≥3 tests, 300+ cases)
- [ ] ✅ Mutation kill rate ≥90%
- [ ] ✅ All 1,408+ existing tests still pass
- [ ] ✅ Zero regressions
- [ ] ✅ Complexity <10 for all functions
- [ ] ✅ Code coverage >85%
- [ ] ✅ End-to-end test: Parse → Purify → Generate → Verify

---

## Expected Output Format

### Variables
```makefile
VAR := value
RECURSIVE = value
CONDITIONAL ?= value
APPEND += value
SHELL != command
```

### Targets
```makefile
target: prereq1 prereq2
	recipe line 1
	recipe line 2
```

### Pattern Rules
```makefile
%.o: %.c
	$(CC) -c $< -o $@
```

### Conditionals
```makefile
ifeq ($(DEBUG),1)
CFLAGS := -g
else
CFLAGS := -O2
endif
```

### .PHONY
```makefile
.PHONY: clean test

clean:
	rm -f *.o
```

### Comments
```makefile
# This is a comment
VAR := value  # Inline comment support (if parser supports it)
```

---

## Risks and Mitigations

### Risk 1: Tab vs Space in Recipes
**Problem**: Makefiles REQUIRE tabs for recipes, spaces will fail
**Mitigation**: Use `\t` explicitly, add test to verify tab characters

### Risk 2: Whitespace Preservation
**Problem**: Extra whitespace might break Makefiles
**Mitigation**: Trim unnecessary whitespace, but preserve intentional spacing

### Risk 3: Round-Trip Fidelity
**Problem**: `parse(generate(ast))` might not equal `ast` due to formatting
**Mitigation**: Use semantic equivalence, not byte-for-byte equality

### Risk 4: Edge Cases in Conditionals
**Problem**: Nested conditionals, complex conditions
**Mitigation**: Comprehensive edge case tests

---

## Deliverables

### Code
- ✅ `rash/src/make_parser/generators.rs` (fully implemented, ~200-300 lines)
- ✅ Tests in `rash/src/make_parser/tests.rs` (≥15 new tests, ~150-200 lines)

### Documentation
- ✅ `SPRINT-68-HANDOFF.md` - Comprehensive handoff
- ✅ `SPRINT-68-QRC.md` - Quick reference card
- ✅ Updated module docstrings

### Commits
- ✅ `feat: Sprint 68 - Code generation implementation`
- ✅ `test: Sprint 68 - Property tests for code generation`
- ✅ `docs: Sprint 68 completion handoff`

---

## Next Steps After Sprint 68

### Sprint 69: CLI Integration (4-6 hours)
Implement `rash purify Makefile` command:
```bash
rash purify Makefile              # Analyze and report
rash purify --fix Makefile        # Auto-fix safe issues
rash purify --fix -o out.mk in.mk # Output to new file
rash purify --report Makefile     # Show transformation report
```

---

## Timeline

| Phase | Task | Duration | Status |
|-------|------|----------|--------|
| 1 | Variables | 30 min | 🚀 NEXT |
| 2 | Targets | 45 min | ⏳ Pending |
| 3 | Comments | 15 min | ⏳ Pending |
| 4 | Pattern Rules | 30 min | ⏳ Pending |
| 5 | Conditionals | 45 min | ⏳ Pending |
| 6 | .PHONY | 20 min | ⏳ Pending |
| 7 | Integration | 30 min | ⏳ Pending |
| 8 | Property Tests | 45 min | ⏳ Pending |
| 9 | Mutation Testing | 30 min | ⏳ Pending |
| 10 | Documentation | 30 min | ⏳ Pending |

**Total Estimated**: 4 hours 40 minutes

---

**Sprint 68 Status**: 🚀 IN PROGRESS
**Current Phase**: Planning Complete
**Next Action**: Phase 1 - RED test for variable generation

---

**Achievement Target**: Complete end-to-end purification workflow! 🎯
