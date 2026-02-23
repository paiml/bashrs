#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(unused_imports)]

use super::super::*;
use crate::make_parser::ast::{MakeCondition, Span, VarFlavor};


// ============================================================================
// FUNC-SUBST-001: $(subst from,to,text) Function
// ============================================================================

#[test]
fn test_FUNC_SUBST_001_basic_subst() {
    // ARRANGE: Variable with $(subst) function
    let makefile = "OBJS = $(subst .c,.o,main.c util.c)";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(
        result.is_ok(),
        "Should parse $(subst) function, got error: {:?}",
        result.err()
    );

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 variable");

    // ASSERT: Check variable with $(subst) function
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "OBJS", "Variable name should be OBJS");
            assert_eq!(
                value, "$(subst .c,.o,main.c util.c)",
                "Variable value should preserve $(subst) function syntax"
            );
        }
        other => panic!("Expected Variable item, got {:?}", other),
    }
}

#[test]
fn test_FUNC_SUBST_001_subst_in_prerequisites() {
    // ARRANGE: Target with $(subst) in prerequisites
    // NOTE: Parser splits prerequisites on whitespace, so $(subst .c,.o,$(SRCS))
    // with spaces becomes 2 prerequisites: "$(subst" and ".c,.o,$(SRCS))"
    // This is expected behavior - the parser is not function-aware
    let makefile = "build: $(subst .c,.o,$(SRCS))\n\tgcc -o $@ $^";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok(), "Should parse $(subst) in prerequisites");

    let ast = result.unwrap();

    // ASSERT: Check target prerequisites
    match &ast.items[0] {
        MakeItem::Target {
            name,
            prerequisites,
            recipe,
            ..
        } => {
            assert_eq!(name, "build");
            // Parser splits on whitespace, so we get 2 prerequisites
            assert_eq!(prerequisites.len(), 2);
            // The function syntax is preserved, just split
            assert_eq!(prerequisites[0], "$(subst");
            assert_eq!(prerequisites[1], ".c,.o,$(SRCS))");
            // Recipe should be parsed correctly
            assert_eq!(recipe.len(), 1, "Should have 1 recipe line");
            assert_eq!(recipe[0], "gcc -o $@ $^");
        }
        other => panic!("Expected Target, got {:?}", other),
    }
}

#[test]
fn test_FUNC_SUBST_001_multiple_subst() {
    // ARRANGE: Multiple variables with $(subst) functions
    let makefile = "OBJS = $(subst .c,.o,$(SRCS))\nLIBS = $(subst .a,.so,$(DEPS))";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok(), "Should parse multiple $(subst) functions");

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 2, "Should have 2 variables");

    // ASSERT: Check first $(subst)
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "OBJS");
            assert_eq!(value, "$(subst .c,.o,$(SRCS))");
        }
        other => panic!("Expected Variable, got {:?}", other),
    }

    // ASSERT: Check second $(subst)
    match &ast.items[1] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "LIBS");
            assert_eq!(value, "$(subst .a,.so,$(DEPS))");
        }
        other => panic!("Expected Variable, got {:?}", other),
    }
}

#[test]
fn test_FUNC_SUBST_001_subst_with_spaces() {
    // ARRANGE: $(subst) with spaces in arguments
    let makefile = "FILES = $(subst src/,build/,src/main.c src/util.c)";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok(), "Should parse $(subst) with spaces");

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1);

    // ASSERT: Check $(subst) preserved with spaces
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "FILES");
            assert_eq!(
                value, "$(subst src/,build/,src/main.c src/util.c)",
                "Should preserve spaces in $(subst) text argument"
            );
        }
        other => panic!("Expected Variable, got {:?}", other),
    }
}

#[test]
fn test_FUNC_SUBST_001_nested_subst() {
    // ARRANGE: Nested $(subst) functions
    let makefile = "RESULT = $(subst .c,.o,$(subst src/,build/,$(SRCS)))";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok(), "Should parse nested $(subst) functions");

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1);

    // ASSERT: Check nested $(subst) preserved
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "RESULT");
            assert_eq!(
                value, "$(subst .c,.o,$(subst src/,build/,$(SRCS)))",
                "Should preserve nested $(subst) functions"
            );
        }
        other => panic!("Expected Variable, got {:?}", other),
    }
}

#[test]
fn test_FUNC_SUBST_001_subst_with_other_functions() {
    // ARRANGE: $(subst) combined with $(wildcard)
    let makefile = "OBJS = $(subst .c,.o,$(wildcard src/*.c))";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(
        result.is_ok(),
        "Should parse $(subst) with $(wildcard) function"
    );

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1);

    // ASSERT: Check combined functions preserved
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "OBJS");
            assert_eq!(
                value, "$(subst .c,.o,$(wildcard src/*.c))",
                "Should preserve $(subst) combined with other functions"
            );
        }
        other => panic!("Expected Variable, got {:?}", other),
    }
}

// ============================================================================
// FUNC-SUBST-001: Property Tests
// ============================================================================

#[cfg(test)]
mod property_tests_func_subst {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_FUNC_SUBST_001_basic_subst_always_preserved(
            var in "[A-Z]{1,8}",
            from in "\\.[a-z]{1,3}",
            to in "\\.[a-z]{1,3}",
            text in "[a-z]{1,10}"
        ) {
            // ARRANGE: Variable with $(subst from,to,text)
            let makefile = format!("{} = $(subst {},{},{})", var, from, to, text);

            // ACT: Parse makefile
            let result = parse_makefile(&makefile);

            // ASSERT: Successfully parsed
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            prop_assert_eq!(ast.items.len(), 1);

            // ASSERT: $(subst) function preserved
            match &ast.items[0] {
                MakeItem::Variable { name, value, .. } => {
                    prop_assert_eq!(name, &var);
                    let expected = format!("$(subst {},{},{})", from, to, text);
                    prop_assert_eq!(value, &expected);
                }
                other => return Err(TestCaseError::fail(format!("Expected Variable, got {:?}", other))),
            }
        }

        #[test]
        fn prop_FUNC_SUBST_001_parsing_is_deterministic(
            var in "[A-Z]{1,8}",
            from in "\\.[a-z]{1,3}",
            to in "\\.[a-z]{1,3}",
            varref in "[A-Z]{1,8}"
        ) {
            // ARRANGE: Variable with $(subst from,to,$(VAR))
            let makefile = format!("{} = $(subst {},{},$({})) ", var, from, to, varref);

            // ACT: Parse twice
            let result1 = parse_makefile(&makefile);
            let result2 = parse_makefile(&makefile);

            // ASSERT: Same results
            prop_assert!(result1.is_ok());
            prop_assert!(result2.is_ok());

            let ast1 = result1.unwrap();
            let ast2 = result2.unwrap();

            // Same number of items
            prop_assert_eq!(ast1.items.len(), ast2.items.len());

            // Same variable value
            match (&ast1.items[0], &ast2.items[0]) {
                (MakeItem::Variable { value: v1, .. }, MakeItem::Variable { value: v2, .. }) => {
                    prop_assert_eq!(v1, v2);
                }
                _ => return Err(TestCaseError::fail("Expected Variables")),
            }
        }

        #[test]
        fn prop_FUNC_SUBST_001_nested_functions_preserved(
            var in "[A-Z]{1,8}",
            from1 in "\\.[a-z]{1,2}",
            to1 in "\\.[a-z]{1,2}",
            from2 in "[a-z]{1,5}/",
            to2 in "[a-z]{1,5}/",
            varref in "[A-Z]{1,8}"
        ) {
            // ARRANGE: Nested $(subst)
            let makefile = format!(
                "{} = $(subst {},{},$(subst {},{},$({})))",
                var, from1, to1, from2, to2, varref
            );

            // ACT: Parse makefile
            let result = parse_makefile(&makefile);

            // ASSERT: Successfully parsed
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            prop_assert_eq!(ast.items.len(), 1);

            // ASSERT: Nested functions preserved
            match &ast.items[0] {
                MakeItem::Variable { value, .. } => {
                    // Should contain both subst calls
                    prop_assert!(value.contains("$(subst"));
                    prop_assert!(value.contains(&from1));
                    prop_assert!(value.contains(&to1));
                    prop_assert!(value.contains(&from2));
                    prop_assert!(value.contains(&to2));
                }
                other => return Err(TestCaseError::fail(format!("Expected Variable, got {:?}", other))),
            }
        }

        #[test]
        fn prop_FUNC_SUBST_001_multiple_functions_preserved(
            var1 in "[A-Z]{1,8}",
            var2 in "[A-Z]{1,8}",
            from1 in "\\.[a-z]{1,3}",
            to1 in "\\.[a-z]{1,3}",
            from2 in "\\.[a-z]{1,3}",
            to2 in "\\.[a-z]{1,3}",
            ref1 in "[A-Z]{1,8}",
            ref2 in "[A-Z]{1,8}"
        ) {
            prop_assume!(var1 != var2);

            // ARRANGE: Two variables with $(subst) functions
            let makefile = format!(
                "{} = $(subst {},{},$({})) \n{} = $(subst {},{},$({})) ",
                var1, from1, to1, ref1, var2, from2, to2, ref2
            );

            // ACT: Parse makefile
            let result = parse_makefile(&makefile);

            // ASSERT: Successfully parsed
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            prop_assert_eq!(ast.items.len(), 2);

            // ASSERT: Both functions preserved
            match &ast.items[0] {
                MakeItem::Variable { name, value, .. } => {
                    prop_assert_eq!(name, &var1);
                    prop_assert!(value.contains("$(subst"));
                    prop_assert!(value.contains(&from1));
                }
                other => return Err(TestCaseError::fail(format!("Expected Variable, got {:?}", other))),
            }

            match &ast.items[1] {
                MakeItem::Variable { name, value, .. } => {
                    prop_assert_eq!(name, &var2);
                    prop_assert!(value.contains("$(subst"));
                    prop_assert!(value.contains(&from2));
                }
                other => return Err(TestCaseError::fail(format!("Expected Variable, got {:?}", other))),
            }
        }

        #[test]
        fn prop_FUNC_SUBST_001_combined_with_wildcard(
            var in "[A-Z]{1,8}",
            from in "\\.[a-z]{1,3}",
            to in "\\.[a-z]{1,3}",
            pattern in "[a-z]{1,8}",
            ext in "[a-z]{1,3}"
        ) {
            // ARRANGE: $(subst) with $(wildcard)
            let makefile = format!(
                "{} = $(subst {},{},$(wildcard {}/*.{}))",
                var, from, to, pattern, ext
            );

            // ACT: Parse makefile
            let result = parse_makefile(&makefile);

            // ASSERT: Successfully parsed
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            prop_assert_eq!(ast.items.len(), 1);

            // ASSERT: Combined functions preserved
            match &ast.items[0] {
                MakeItem::Variable { value, .. } => {
                    prop_assert!(value.contains("$(subst"));
                    prop_assert!(value.contains("$(wildcard"));
                }
                other => return Err(TestCaseError::fail(format!("Expected Variable, got {:?}", other))),
            }
        }

        #[test]
        fn prop_FUNC_SUBST_001_no_spaces_in_function(
            var in "[A-Z]{1,8}",
            from in "[a-z]{1,5}",
            to in "[a-z]{1,5}",
            text in "[a-z]{1,10}"
        ) {
            // ARRANGE: $(subst) without spaces (single token)
            let makefile = format!("{} = $(subst {},{},{})", var, from, to, text);

            // ACT: Parse makefile
            let result = parse_makefile(&makefile);

            // ASSERT: Successfully parsed
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            prop_assert_eq!(ast.items.len(), 1);

            // ASSERT: Function preserved as one value
            match &ast.items[0] {
                MakeItem::Variable { value, .. } => {
                    let expected = format!("$(subst {},{},{})", from, to, text);
                    prop_assert_eq!(value, &expected);
                }
                other => return Err(TestCaseError::fail(format!("Expected Variable, got {:?}", other))),
            }
        }
    }
}

// ============================================================================
// SPRINT 64: Function Call Parser Tests
// ============================================================================
// Goal: Parse GNU Make function calls like $(filter %.o, foo.o bar.c)
// Context: Enables recursive purification for 13 deterministic functions
// Reference: SPRINT-63-HANDOFF.md, SPRINT-61-HANDOFF.md, SPRINT-62-HANDOFF.md

#[test]
fn test_PARSER_FUNC_001_basic_filter() {
    // ARRANGE: Makefile with filter function
    let makefile = "OBJS := $(filter %.o, foo.o bar.c baz.o)";

    // ACT: Parse makefile
    let ast = parse_makefile(makefile).unwrap();

    // ASSERT: Should parse variable with function call
    assert_eq!(ast.items.len(), 1);

    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "OBJS");
            // For now, just verify it contains the function call
            // Later: will verify FunctionCall AST node structure
            assert!(value.contains("$(filter"));
        }
        _ => panic!("Expected Variable, got {:?}", ast.items[0]),
    }
}

#[test]
fn test_PARSER_FUNC_002_basic_sort() {
    // ARRANGE: Makefile with sort function
    let makefile = "SORTED := $(sort foo bar baz foo)";

    // ACT: Parse makefile
    let ast = parse_makefile(makefile).unwrap();

    // ASSERT: Should parse variable with sort function call
    assert_eq!(ast.items.len(), 1);

    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "SORTED");
            assert!(value.contains("$(sort"));
        }
        _ => panic!("Expected Variable, got {:?}", ast.items[0]),
    }
}

#[test]
fn test_PARSER_FUNC_003_filter_multiple_patterns() {
    // ARRANGE: filter with multiple pattern arguments
    let makefile = "OBJS := $(filter %.o %.a, foo.o bar.c baz.a)";

    // ACT: Parse makefile
    let ast = parse_makefile(makefile).unwrap();

    // ASSERT: Should parse both patterns
    assert_eq!(ast.items.len(), 1);

    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "OBJS");
            assert!(value.contains("$(filter"));
            assert!(value.contains("%.o"));
            assert!(value.contains("%.a"));
        }
        _ => panic!("Expected Variable, got {:?}", ast.items[0]),
    }
}

#[test]
fn test_PARSER_FUNC_004_nested_wildcard() {
    // ARRANGE: CRITICAL - nested function calls
    // This is the key pattern for recursive purification
    let makefile = "OBJS := $(filter %.o, $(wildcard *.c))";

    // ACT: Parse makefile
    let ast = parse_makefile(makefile).unwrap();

    // ASSERT: Should parse nested $(wildcard) inside $(filter)
    assert_eq!(ast.items.len(), 1);

    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "OBJS");
            assert!(value.contains("$(filter"));
            assert!(value.contains("$(wildcard"));
        }
        _ => panic!("Expected Variable, got {:?}", ast.items[0]),
    }
}

#[test]
fn test_PARSER_FUNC_005_word() {
    // ARRANGE: word function (extracts Nth word)
    let makefile = "SECOND := $(word 2, foo bar baz)";

    // ACT: Parse makefile
    let ast = parse_makefile(makefile).unwrap();

    // ASSERT: Should parse word function
    assert_eq!(ast.items.len(), 1);

    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "SECOND");
            assert!(value.contains("$(word"));
        }
        _ => panic!("Expected Variable, got {:?}", ast.items[0]),
    }
}

#[test]
fn test_PARSER_FUNC_006_notdir() {
    // ARRANGE: notdir function (remove directory part)
    let makefile = "FILES := $(notdir src/main.c include/util.h)";

    // ACT: Parse makefile
    let ast = parse_makefile(makefile).unwrap();

    // ASSERT: Should parse notdir function
    assert_eq!(ast.items.len(), 1);

    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "FILES");
            assert!(value.contains("$(notdir"));
        }
        _ => panic!("Expected Variable, got {:?}", ast.items[0]),
    }
}

#[test]
fn test_PARSER_FUNC_007_addsuffix() {
    // ARRANGE: addsuffix function
    let makefile = "OBJS := $(addsuffix .o, foo bar baz)";

    // ACT: Parse makefile
    let ast = parse_makefile(makefile).unwrap();

    // ASSERT: Should parse addsuffix function
    assert_eq!(ast.items.len(), 1);

    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "OBJS");
            assert!(value.contains("$(addsuffix"));
        }
        _ => panic!("Expected Variable, got {:?}", ast.items[0]),
    }
}

#[test]
fn test_PARSER_FUNC_008_addprefix() {
    // ARRANGE: addprefix function
    let makefile = "OBJS := $(addprefix obj/, foo.o bar.o)";

    // ACT: Parse makefile
    let ast = parse_makefile(makefile).unwrap();

    // ASSERT: Should parse addprefix function
    assert_eq!(ast.items.len(), 1);

    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "OBJS");
            assert!(value.contains("$(addprefix"));
        }
        _ => panic!("Expected Variable, got {:?}", ast.items[0]),
    }
}

#[test]
fn test_PARSER_FUNC_009_filter_out() {
    // ARRANGE: filter-out function (inverse of filter)
    let makefile = "SOURCES := $(filter-out test_%, main.c test_foo.c util.c)";

    // ACT: Parse makefile
    let ast = parse_makefile(makefile).unwrap();

    // ASSERT: Should parse filter-out function
    assert_eq!(ast.items.len(), 1);

    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "SOURCES");
            assert!(value.contains("$(filter-out"));
        }
        _ => panic!("Expected Variable, got {:?}", ast.items[0]),
    }
}

#[test]
fn test_PARSER_FUNC_010_wordlist() {
    // ARRANGE: wordlist function (extract range of words)
    let makefile = "MIDDLE := $(wordlist 2, 4, foo bar baz qux quux)";

    // ACT: Parse makefile
    let ast = parse_makefile(makefile).unwrap();

    // ASSERT: Should parse wordlist function
    assert_eq!(ast.items.len(), 1);

    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "MIDDLE");
            assert!(value.contains("$(wordlist"));
        }
        _ => panic!("Expected Variable, got {:?}", ast.items[0]),
    }
}

#[test]
fn test_PARSER_FUNC_011_words() {
    // ARRANGE: words function (count words)
    let makefile = "COUNT := $(words foo bar baz)";

    // ACT: Parse makefile
    let ast = parse_makefile(makefile).unwrap();

    // ASSERT: Should parse words function
    assert_eq!(ast.items.len(), 1);

    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "COUNT");
            assert!(value.contains("$(words"));
        }
        _ => panic!("Expected Variable, got {:?}", ast.items[0]),
    }
}

#[test]
fn test_PARSER_FUNC_012_firstword() {
    // ARRANGE: firstword function
    let makefile = "FIRST := $(firstword foo bar baz)";

    // ACT: Parse makefile
    let ast = parse_makefile(makefile).unwrap();

    // ASSERT: Should parse firstword function
    assert_eq!(ast.items.len(), 1);

    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "FIRST");
            assert!(value.contains("$(firstword"));
        }
        _ => panic!("Expected Variable, got {:?}", ast.items[0]),
    }
}

#[test]
fn test_PARSER_FUNC_013_lastword() {
    // ARRANGE: lastword function
    let makefile = "LAST := $(lastword foo bar baz)";

    // ACT: Parse makefile
    let ast = parse_makefile(makefile).unwrap();

    // ASSERT: Should parse lastword function
    assert_eq!(ast.items.len(), 1);

    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "LAST");
            assert!(value.contains("$(lastword"));
        }
        _ => panic!("Expected Variable, got {:?}", ast.items[0]),
    }
}

#[test]
fn test_PARSER_FUNC_014_suffix() {
    // ARRANGE: suffix function (extract file suffixes)
    let makefile = "SUFFIXES := $(suffix foo.c bar.o baz.txt)";

    // ACT: Parse makefile
    let ast = parse_makefile(makefile).unwrap();

    // ASSERT: Should parse suffix function
    assert_eq!(ast.items.len(), 1);

    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "SUFFIXES");
            assert!(value.contains("$(suffix"));
        }
        _ => panic!("Expected Variable, got {:?}", ast.items[0]),
    }
}

#[test]
fn test_PARSER_FUNC_015_basename() {
    // ARRANGE: basename function (remove suffix)
    let makefile = "BASES := $(basename foo.c bar.o baz.txt)";

    // ACT: Parse makefile
    let ast = parse_makefile(makefile).unwrap();

    // ASSERT: Should parse basename function
    assert_eq!(ast.items.len(), 1);

    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "BASES");
            assert!(value.contains("$(basename"));
        }
        _ => panic!("Expected Variable, got {:?}", ast.items[0]),
    }
}

// ============================================================================
// Sprint 65: Recursive Semantic Analysis Tests
// ============================================================================
// These tests verify that semantic analysis detects non-deterministic patterns
// even when they're nested inside function arguments.
//
// Example: $(filter %.c, $(wildcard src/*.c))
//                         ^^^^^^^^^^^^^^^^^
//                         Nested wildcard needs detection

#[test]
fn test_SEMANTIC_RECURSIVE_001_detect_wildcard_in_filter_args() {
    // ARRANGE: wildcard nested in filter arguments
    let makefile = "FILES := $(filter %.c, $(wildcard src/*.c))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Check if value contains nested wildcard
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "FILES");
            // This test verifies we can DETECT the pattern
            // (actual semantic analysis implementation will come in GREEN phase)
            assert!(value.contains("$(wildcard"));
            assert!(value.contains("$(filter"));
        }
        _ => panic!("Expected Variable"),
    }
}

#[test]
fn test_SEMANTIC_RECURSIVE_002_detect_wildcard_in_sort_args() {
    // ARRANGE: wildcard nested in sort arguments
    let makefile = "SORTED := $(sort $(wildcard *.c))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT & ASSERT: Verify nested pattern detected
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "SORTED");
            assert!(value.contains("$(wildcard"));
            assert!(value.contains("$(sort"));
        }
        _ => panic!("Expected Variable"),
    }
}

#[test]
fn test_SEMANTIC_RECURSIVE_003_detect_shell_date_in_addsuffix_args() {
    // ARRANGE: shell date nested in addsuffix arguments
    let makefile = "TIMESTAMPED := $(addsuffix -$(shell date +%s), foo bar)";
    let ast = parse_makefile(makefile).unwrap();

    // ACT & ASSERT: Verify nested shell date detected
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "TIMESTAMPED");
            assert!(value.contains("$(shell date"));
            assert!(value.contains("$(addsuffix"));
        }
        _ => panic!("Expected Variable"),
    }
}

#[test]
fn test_SEMANTIC_RECURSIVE_004_detect_random_in_word_args() {
    // ARRANGE: $RANDOM nested in word arguments
    let makefile = "PICK := $(word $RANDOM, foo bar baz)";
    let ast = parse_makefile(makefile).unwrap();

    // ACT & ASSERT: Verify nested $RANDOM detected
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "PICK");
            assert!(value.contains("$RANDOM") || value.contains("RANDOM"));
            assert!(value.contains("$(word"));
        }
        _ => panic!("Expected Variable"),
    }
}

#[test]
fn test_SEMANTIC_RECURSIVE_005_detect_shell_find_in_filter_args() {
    // ARRANGE: shell find nested in filter arguments
    let makefile = "FOUND := $(filter %.c, $(shell find src -name '*.c'))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT & ASSERT: Verify nested shell find detected
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "FOUND");
            assert!(value.contains("$(shell find"));
            assert!(value.contains("$(filter"));
        }
        _ => panic!("Expected Variable"),
    }
}

#[test]
fn test_SEMANTIC_RECURSIVE_006_detect_deeply_nested_wildcard() {
    // ARRANGE: deeply nested - wildcard in filter in sort
    let makefile = "DEEP := $(sort $(filter %.c, $(wildcard src/*.c)))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT & ASSERT: Verify all nesting levels detected
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "DEEP");
            assert!(value.contains("$(wildcard"));
            assert!(value.contains("$(filter"));
            assert!(value.contains("$(sort"));
        }
        _ => panic!("Expected Variable"),
    }
}

#[test]
fn test_SEMANTIC_RECURSIVE_007_detect_multiple_nested_wildcards() {
    // ARRANGE: multiple wildcard calls in single function
    let makefile = "MULTI := $(filter %.c %.h, $(wildcard src/*.c) $(wildcard inc/*.h))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT & ASSERT: Verify multiple nested patterns detected
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "MULTI");
            // Should contain two wildcard calls
            let wildcard_count = value.matches("$(wildcard").count();
            assert!(
                wildcard_count >= 2,
                "Expected at least 2 wildcard calls, found {}",
                wildcard_count
            );
        }
        _ => panic!("Expected Variable"),
    }
}

#[test]
fn test_SEMANTIC_RECURSIVE_008_safe_filter_no_wildcard() {
    // ARRANGE: filter without nested non-deterministic code (SAFE)
    let makefile = "SAFE := $(filter %.c, foo.c bar.c baz.c)";
    let ast = parse_makefile(makefile).unwrap();

    // ACT & ASSERT: Verify no non-deterministic patterns
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "SAFE");
            // Should NOT contain wildcard/shell/random
            assert!(!value.contains("$(wildcard"));
            assert!(!value.contains("$(shell"));
            assert!(!value.contains("$RANDOM"));
        }
        _ => panic!("Expected Variable"),
    }
}

#[test]
fn test_SEMANTIC_RECURSIVE_009_purified_sort_wrapped_wildcard() {
    // ARRANGE: PURIFIED - wildcard wrapped with sort (SAFE)
    let makefile = "PURIFIED := $(filter %.c, $(sort $(wildcard src/*.c)))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT & ASSERT: Verify purified pattern (sort wraps wildcard)
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "PURIFIED");
            // Should contain both wildcard (non-deterministic) and sort (purifier)
            assert!(value.contains("$(wildcard"));
            assert!(value.contains("$(sort"));
            // Verify sort comes BEFORE wildcard (wrapping)
            let sort_pos = value.find("$(sort").unwrap();
            let wildcard_pos = value.find("$(wildcard").unwrap();
            assert!(sort_pos < wildcard_pos, "sort should wrap wildcard");
        }
        _ => panic!("Expected Variable"),
    }
}

#[test]
fn test_SEMANTIC_RECURSIVE_010_detect_wildcard_in_firstword() {
    // ARRANGE: wildcard in firstword (HIGH RISK - returns different results)
    let makefile = "FIRST := $(firstword $(wildcard *.c))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT & ASSERT: Critical case - firstword depends on order
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "FIRST");
            assert!(value.contains("$(wildcard"));
            assert!(value.contains("$(firstword"));
        }
        _ => panic!("Expected Variable"),
    }
}

#[test]
fn test_SEMANTIC_RECURSIVE_011_detect_wildcard_in_lastword() {
    // ARRANGE: wildcard in lastword (HIGH RISK - returns different results)
    let makefile = "LAST := $(lastword $(wildcard *.c))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT & ASSERT: Critical case - lastword depends on order
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "LAST");
            assert!(value.contains("$(wildcard"));
            assert!(value.contains("$(lastword"));
        }
        _ => panic!("Expected Variable"),
    }
}

#[test]
fn test_SEMANTIC_RECURSIVE_012_detect_wildcard_in_wordlist() {
    // ARRANGE: wildcard in wordlist (HIGH RISK - indices depend on order)
    let makefile = "RANGE := $(wordlist 2, 4, $(wildcard *.c))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT & ASSERT: Critical case - wordlist indices depend on order
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "RANGE");
            assert!(value.contains("$(wildcard"));
            assert!(value.contains("$(wordlist"));
        }
        _ => panic!("Expected Variable"),
    }
}

#[test]
fn test_SEMANTIC_RECURSIVE_013_detect_shell_date_in_filter() {
    // ARRANGE: shell date in filter arguments (timestamp in filtering)
    let makefile = "DATED := $(filter release-%, $(shell date +%Y%m%d))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT & ASSERT: Verify shell date nested in filter
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "DATED");
            assert!(value.contains("$(shell date"));
            assert!(value.contains("$(filter"));
        }
        _ => panic!("Expected Variable"),
    }
}

#[test]
fn test_SEMANTIC_RECURSIVE_014_detect_multiple_nested_issues() {
    // ARRANGE: multiple different non-deterministic patterns nested
    let makefile = r#"
COMPLEX := $(filter %.c, $(wildcard *.c)) $(word $RANDOM, $(shell find src))
"#;
    let ast = parse_makefile(makefile).unwrap();

    // ACT & ASSERT: Verify all three types of issues detected
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "COMPLEX");
            // Should detect wildcard, $RANDOM, and shell find
            assert!(value.contains("$(wildcard"));
            assert!(value.contains("RANDOM"));
            assert!(value.contains("$(shell find"));
        }
        _ => panic!("Expected Variable"),
    }
}

#[test]
fn test_SEMANTIC_RECURSIVE_015_pattern_rule_with_nested_wildcard() {
    // ARRANGE: pattern rule with wildcard in prerequisites
    let makefile = r#"
%.o: $(filter %.c, $(wildcard src/*.c))
	gcc -c $< -o $@
"#;
    let ast = parse_makefile(makefile).unwrap();

    // ACT & ASSERT: Verify pattern rule contains nested wildcard
    match &ast.items[0] {
        MakeItem::PatternRule {
            target_pattern,
            prereq_patterns,
            ..
        } => {
            assert_eq!(target_pattern, "%.o");
            // Prerequisites should contain nested wildcard
            let prereqs = prereq_patterns.join(" ");
            assert!(prereqs.contains("$(wildcard"));
            assert!(prereqs.contains("$(filter"));
        }
        _ => panic!("Expected PatternRule, got {:?}", ast.items[0]),
    }
}

// ============================================================================
// Sprint 65: Integration Tests for analyze_makefile() with Nested Patterns
// ============================================================================
// These tests verify that analyze_makefile() detects non-deterministic patterns
// even when nested inside function arguments

#[test]
fn test_SEMANTIC_ANALYZE_001_detect_nested_wildcard_in_filter() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: wildcard nested in filter arguments
    let makefile = "FILES := $(filter %.c, $(wildcard src/*.c))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should detect nested wildcard
    assert!(
        !issues.is_empty(),
        "Expected to detect nested wildcard, but got no issues"
    );
    assert_eq!(
        issues.len(),
        1,
        "Expected exactly 1 issue for nested wildcard"
    );
    assert_eq!(issues[0].rule, "NO_WILDCARD");
    assert!(issues[0].message.contains("FILES"));
    assert!(issues[0].message.contains("wildcard"));
}

#[test]
fn test_SEMANTIC_ANALYZE_002_detect_nested_shell_date_in_addsuffix() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: shell date nested in addsuffix arguments
    let makefile = "TIMESTAMPED := $(addsuffix -$(shell date +%s), foo bar)";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should detect nested shell date
    assert!(!issues.is_empty(), "Expected to detect nested shell date");
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].rule, "NO_TIMESTAMPS");
    assert!(issues[0].message.contains("TIMESTAMPED"));
}

#[test]
fn test_SEMANTIC_ANALYZE_003_detect_nested_random_in_word() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: $RANDOM nested in word arguments
    let makefile = "PICK := $(word $RANDOM, foo bar baz)";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should detect nested $RANDOM
    assert!(!issues.is_empty(), "Expected to detect nested $RANDOM");
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].rule, "NO_RANDOM");
    assert!(issues[0].message.contains("PICK"));
}

#[test]
fn test_SEMANTIC_ANALYZE_004_no_issue_for_safe_filter() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: filter without nested non-deterministic code (SAFE)
    let makefile = "SAFE := $(filter %.c, foo.c bar.c baz.c)";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should NOT detect any issues (no wildcard, no shell, no random)
    assert_eq!(
        issues.len(),
        0,
        "Expected no issues for safe filter, but got: {:?}",
        issues
    );
}

#[test]
fn test_SEMANTIC_ANALYZE_005_purified_wildcard_not_detected() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: PURIFIED wildcard wrapped with sort
    // Enhancement IMPLEMENTED: detect $(sort $(wildcard)) as "already purified"
    let makefile = "PURIFIED := $(filter %.c, $(sort $(wildcard src/*.c)))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Purified wildcards should NOT be detected
    assert_eq!(
        issues.len(),
        0,
        "Purified wildcard should not be detected: {:?}",
        issues
    );
}

#[test]
fn test_SEMANTIC_ANALYZE_006_deeply_nested_unpurified_wildcard() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: deeply nested - wildcard in filter in sort (NOT PURIFIED PROPERLY)
    // This is NOT purified because the wildcard itself is not wrapped with sort
    // The outer sort only sorts the filter results, not the wildcard results
    let makefile = "DEEP := $(sort $(filter %.c, $(wildcard src/*.c)))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should detect wildcard because it's not directly wrapped with sort
    assert!(
        !issues.is_empty(),
        "Wildcard should be detected when not directly wrapped with sort"
    );
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].rule, "NO_WILDCARD");
}

#[test]
fn test_SEMANTIC_ANALYZE_007_multiple_nested_wildcards() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: multiple wildcard calls in single function
    let makefile = "MULTI := $(filter %.c %.h, $(wildcard src/*.c) $(wildcard inc/*.h))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should detect wildcard (may report as 1 issue for the variable)
    assert!(!issues.is_empty());
    assert_eq!(issues[0].rule, "NO_WILDCARD");
    assert!(issues[0].message.contains("MULTI"));
}

#[test]
fn test_SEMANTIC_ANALYZE_008_nested_shell_find_in_filter() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: shell find nested in filter arguments
    let makefile = "FOUND := $(filter %.c, $(shell find src -name '*.c'))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should detect nested shell find
    assert!(!issues.is_empty());
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].rule, "NO_UNORDERED_FIND");
    assert!(issues[0].message.contains("FOUND"));
}

#[test]
fn test_SEMANTIC_ANALYZE_009_multiple_different_nested_issues() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: multiple different non-deterministic patterns nested
    let makefile = r#"
COMPLEX := $(filter %.c, $(wildcard *.c)) $(word $RANDOM, $(shell find src))
"#;
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should detect all three types of issues
    // Current implementation detects all patterns in the value string
    assert!(
        issues.len() >= 3,
        "Expected at least 3 issues (wildcard, random, shell find), got {}",
        issues.len()
    );

    // Verify all three rule types are detected
    let rules: Vec<&str> = issues.iter().map(|i| i.rule.as_str()).collect();
    assert!(rules.contains(&"NO_WILDCARD"), "Should detect wildcard");
    assert!(rules.contains(&"NO_RANDOM"), "Should detect $RANDOM");
    assert!(
        rules.contains(&"NO_UNORDERED_FIND"),
        "Should detect shell find"
    );
}

#[test]
fn test_SEMANTIC_ANALYZE_010_nested_wildcard_in_firstword() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: wildcard in firstword (HIGH RISK - different results based on order)
    let makefile = "FIRST := $(firstword $(wildcard *.c))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should detect wildcard (critical case for firstword)
    assert!(!issues.is_empty());
    assert_eq!(issues[0].rule, "NO_WILDCARD");
    assert!(issues[0].message.contains("FIRST"));
}

// ============================================================================
// Sprint 66: High-Risk Functions - FOREACH Detection Tests
// ============================================================================
//
// Goal: Verify semantic analysis detects non-deterministic patterns in
//       $(foreach) loops where iteration order matters.
//
// Hypothesis (based on Sprint 64-65): Existing .contains() approach
// already detects these patterns at any nesting level.
//
// Test Strategy: Write verification tests first (EXTREME TDD RED phase)

#[test]
fn test_SEMANTIC_FOREACH_001_detect_wildcard_in_foreach_list() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: $(foreach) iterating over $(wildcard) - ORDER MATTERS!
    // This is CRITICAL because foreach processes items in iteration order
    let makefile = "OBJS := $(foreach file, $(wildcard *.c), $(file:.c=.o))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should detect wildcard in foreach list (non-deterministic order)
    // Based on Sprint 65 discovery: .contains("$(wildcard") should catch this!
    assert!(
        !issues.is_empty(),
        "Expected to detect wildcard in foreach list"
    );

    // Verify it's detected as NO_WILDCARD
    let wildcard_issues: Vec<_> = issues.iter().filter(|i| i.rule == "NO_WILDCARD").collect();
    assert!(!wildcard_issues.is_empty(), "Should detect as NO_WILDCARD");
}

#[test]
fn test_SEMANTIC_FOREACH_002_safe_foreach_with_explicit_list() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: $(foreach) with explicit list (SAFE - deterministic order)
    let makefile = "OBJS := $(foreach file, foo.c bar.c baz.c, $(file:.c=.o))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should NOT detect issues (explicit list is deterministic)
    assert_eq!(
        issues.len(),
        0,
        "Expected no issues for explicit list: {:?}",
        issues
    );
}

#[test]
fn test_SEMANTIC_FOREACH_003_nested_shell_date_in_foreach() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: $(shell date) nested in foreach body
    let makefile = "TIMESTAMPED := $(foreach f, foo bar, $(f)-$(shell date +%s))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should detect shell date
    assert!(!issues.is_empty(), "Expected to detect shell date");
    assert!(issues.iter().any(|i| i.rule == "NO_TIMESTAMPS"));
}

#[test]
fn test_SEMANTIC_FOREACH_004_random_in_foreach_body() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: $RANDOM in foreach body
    let makefile = "IDS := $(foreach item, a b c, id-$RANDOM)";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should detect random
    assert!(!issues.is_empty(), "Expected to detect $RANDOM");
    assert!(issues.iter().any(|i| i.rule == "NO_RANDOM"));
}

#[test]
fn test_SEMANTIC_FOREACH_005_shell_find_in_foreach_list() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: $(shell find) as foreach list source
    let makefile = "PROCESSED := $(foreach f, $(shell find src -name '*.c'), process-$(f))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should detect shell find
    assert!(!issues.is_empty(), "Expected to detect shell find");
    assert!(issues.iter().any(|i| i.rule == "NO_UNORDERED_FIND"));
}

// ============================================================================
// Sprint 66: High-Risk Functions - CALL Detection Tests
// ============================================================================

#[test]
fn test_SEMANTIC_CALL_001_detect_wildcard_in_call_args() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: $(call) with $(wildcard) in arguments
    // Define a function and call it with wildcard
    let makefile = r#"
reverse = $(2) $(1)
FILES := $(call reverse, $(wildcard *.c), foo.c)
"#;
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should detect wildcard in call arguments
    assert!(
        !issues.is_empty(),
        "Expected to detect wildcard in call args"
    );

    // Check that FILES variable has wildcard issue
    let files_issues: Vec<_> = issues
        .iter()
        .filter(|i| i.message.contains("FILES"))
        .collect();
    assert!(
        !files_issues.is_empty(),
        "Should detect wildcard in FILES variable"
    );
}

#[test]
fn test_SEMANTIC_CALL_002_safe_call_with_explicit_args() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: $(call) with explicit arguments (SAFE)
    let makefile = r#"
reverse = $(2) $(1)
RESULT := $(call reverse, foo.c, bar.c)
"#;
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should NOT detect issues (explicit args are deterministic)
    assert_eq!(
        issues.len(),
        0,
        "Expected no issues for explicit args: {:?}",
        issues
    );
}

#[test]
fn test_SEMANTIC_CALL_003_shell_date_in_call_args() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: $(call) with $(shell date) in arguments
    let makefile = r#"
timestamp = build-$(1)-$(2)
RELEASE := $(call timestamp, v1.0, $(shell date +%s))
"#;
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should detect shell date
    assert!(!issues.is_empty(), "Expected to detect shell date");
    assert!(issues.iter().any(|i| i.rule == "NO_TIMESTAMPS"));
}

#[test]
fn test_SEMANTIC_CALL_004_random_in_call_args() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: $RANDOM in call arguments
    let makefile = r#"
generate_id = id-$(1)-$(2)
SESSION := $(call generate_id, sess, $RANDOM)
"#;
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should detect random
    assert!(!issues.is_empty(), "Expected to detect $RANDOM");
    assert!(issues.iter().any(|i| i.rule == "NO_RANDOM"));
}

#[test]
fn test_SEMANTIC_CALL_005_shell_find_in_call_args() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::semantic::analyze_makefile;

    // ARRANGE: $(shell find) in call arguments
    let makefile = r#"
process_files = Processing: $(1)
OUTPUT := $(call process_files, $(shell find src -name '*.c'))
"#;
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Run semantic analysis
    let issues = analyze_makefile(&ast);

    // ASSERT: Should detect shell find
    assert!(!issues.is_empty(), "Expected to detect shell find");
    assert!(issues.iter().any(|i| i.rule == "NO_UNORDERED_FIND"));
}

// ============================================================================
// Sprint 67: Purification Engine Tests
// ============================================================================
//
// Goal: Implement purification engine that auto-fixes non-deterministic
//       patterns detected by semantic analysis.
//
// Approach: EXTREME TDD - Write RED tests first, then implement

#[test]
fn test_PURIFY_001_wrap_simple_wildcard_with_sort() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::purify::purify_makefile;

    // ARRANGE: Simple wildcard (non-deterministic)
    let makefile = "FILES := $(wildcard *.c)";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Wildcard wrapped with sort
    // Sprint 83 Day 5: Performance optimization may detect additional issues (e.g., missing .SUFFIXES)
    assert!(
        result.transformations_applied >= 1,
        "Should apply at least 1 transformation"
    );
    assert!(result.issues_fixed >= 1, "Should fix at least 1 issue");
    assert_eq!(result.manual_fixes_needed, 0, "No manual fixes needed");

    // Check purified output
    let purified_var = &result.ast.items[0];
    if let crate::make_parser::ast::MakeItem::Variable { value, .. } = purified_var {
        assert!(
            value.contains("$(sort $(wildcard"),
            "Should contain $(sort $(wildcard"
        );
        assert_eq!(value, "$(sort $(wildcard *.c))", "Should be fully wrapped");
    } else {
        panic!("Expected Variable");
    }
}

#[test]
fn test_PURIFY_002_wrap_nested_wildcard_in_filter() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::purify::purify_makefile;

    // ARRANGE: Wildcard nested in filter
    let makefile = "OBJS := $(filter %.o, $(wildcard *.c))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Inner wildcard wrapped with sort
    // Sprint 83 Day 5: Performance optimization may detect additional issues
    assert!(
        result.transformations_applied >= 1,
        "Should apply at least 1 transformation"
    );
    assert!(result.issues_fixed >= 1, "Should fix at least 1 issue");

    let purified_var = &result.ast.items[0];
    if let crate::make_parser::ast::MakeItem::Variable { value, .. } = purified_var {
        assert!(
            value.contains("$(sort $(wildcard"),
            "Should wrap inner wildcard"
        );
        assert_eq!(
            value, "$(filter %.o, $(sort $(wildcard *.c)))",
            "Should preserve filter"
        );
    } else {
        panic!("Expected Variable");
    }
}

#[test]
fn test_PURIFY_003_wrap_shell_find_with_sort() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::purify::purify_makefile;

    // ARRANGE: Shell find (non-deterministic)
    let makefile = "FILES := $(shell find src -name '*.c')";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Shell find wrapped with sort
    assert!(
        result.transformations_applied >= 1,
        "Should apply at least 1 transformation"
    );
    assert!(result.issues_fixed >= 1, "Should fix at least 1 issue");

    let purified_var = &result.ast.items[0];
    if let crate::make_parser::ast::MakeItem::Variable { value, .. } = purified_var {
        assert!(
            value.contains("$(sort $(shell find"),
            "Should wrap shell find"
        );
    } else {
        panic!("Expected Variable");
    }
}

#[test]
fn test_PURIFY_004_nested_wildcard_in_foreach() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::purify::purify_makefile;

    // ARRANGE: Wildcard in foreach list
    let makefile = "OBJS := $(foreach file, $(wildcard *.c), $(file:.c=.o))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Wildcard in foreach list wrapped with sort
    assert!(
        result.transformations_applied >= 1,
        "Should apply at least 1 transformation"
    );
    assert!(result.issues_fixed >= 1, "Should fix at least 1 issue");

    let purified_var = &result.ast.items[0];
    if let crate::make_parser::ast::MakeItem::Variable { value, .. } = purified_var {
        assert!(value.contains("$(sort $(wildcard"), "Should wrap wildcard");
        assert!(
            value.contains("$(foreach file, $(sort $(wildcard"),
            "Should preserve foreach"
        );
    } else {
        panic!("Expected Variable");
    }
}

#[test]
fn test_PURIFY_005_nested_wildcard_in_call() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::purify::purify_makefile;

    // ARRANGE: Wildcard in call arguments
    let makefile = r#"
process = Processing $(1)
FILES := $(call process, $(wildcard *.c))
"#;
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Wildcard in call args wrapped with sort
    // We should have 2 items: function definition + variable
    assert!(
        result.transformations_applied >= 1,
        "Should apply at least 1 transformation"
    );

    // Find the FILES variable
    let files_var = result
        .ast
        .items
        .iter()
        .find(|item| {
            if let crate::make_parser::ast::MakeItem::Variable { name, .. } = item {
                name == "FILES"
            } else {
                false
            }
        })
        .expect("FILES variable should exist");

    if let crate::make_parser::ast::MakeItem::Variable { value, .. } = files_var {
        assert!(value.contains("$(sort $(wildcard"), "Should wrap wildcard");
        assert!(
            value.contains("$(call process, $(sort $(wildcard"),
            "Should preserve call"
        );
    } else {
        panic!("Expected Variable");
    }
}

#[test]
fn test_PURIFY_006_shell_date_manual_fix() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::purify::purify_makefile;

    // ARRANGE: Shell date (cannot auto-fix)
    let makefile = "RELEASE := release-$(shell date +%s)";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Manual fix needed
    // Sprint 83 enhancement: Now detects multiple issues (semantic + Sprint 83 reproducible builds)
    // - Semantic analysis: NO_TIMESTAMPS (1)
    // - Sprint 83: DetectTimestamp (1)
    // - Sprint 83: SuggestSourceDateEpoch (1)
    // Total: 3 manual fixes
    assert!(
        result.manual_fixes_needed >= 1,
        "Should need at least 1 manual fix"
    );
    assert!(
        result.transformations_applied >= 1,
        "Should plan transformation"
    );

    // Check report mentions manual fix or timestamp
    assert!(!result.report.is_empty(), "Should have report");
    assert!(
        result.report.iter().any(|r| r.contains("Manual fix")
            || r.contains("timestamp")
            || r.contains("SOURCE_DATE_EPOCH")),
        "Report should mention manual fix or timestamp issue"
    );
}

#[test]
fn test_PURIFY_007_random_manual_fix() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::purify::purify_makefile;

    // ARRANGE: $RANDOM (cannot auto-fix)
    let makefile = "SESSION := session-$RANDOM";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Manual fix needed
    // Sprint 83 enhancement: Now detects multiple issues (semantic + Sprint 83 reproducible builds)
    // - Semantic analysis: NO_RANDOM (1)
    // - Sprint 83: DetectRandom (1)
    // Total: 2 manual fixes
    assert!(
        result.manual_fixes_needed >= 1,
        "Should need at least 1 manual fix"
    );
    assert!(
        result.transformations_applied >= 1,
        "Should plan transformation"
    );
}

#[test]
fn test_PURIFY_008_safe_patterns_unchanged() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::purify::purify_makefile;

    // ARRANGE: Safe deterministic patterns
    let makefile = "FILES := foo.c bar.c baz.c";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: No transformations needed
    assert_eq!(
        result.transformations_applied, 0,
        "Should apply 0 transformations"
    );
    assert_eq!(result.issues_fixed, 0, "Should fix 0 issues");
    assert_eq!(result.manual_fixes_needed, 0, "Should need 0 manual fixes");

    // AST should be unchanged
    assert_eq!(
        result.ast.items.len(),
        ast.items.len(),
        "AST should be unchanged"
    );
}

#[test]
fn test_PURIFY_009_report_generation() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::purify::purify_makefile;

    // ARRANGE: Mix of auto-fix and manual fix
    let makefile = r#"
FILES := $(wildcard *.c)
RELEASE := release-$(shell date +%s)
"#;
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Report generated
    assert!(!result.report.is_empty(), "Should generate report");
    assert!(
        result.report.len() >= 2,
        "Should have at least 2 report entries"
    );

    // Check report contains expected information
    let report_text = result.report.join("\n");
    assert!(
        report_text.contains("Wrapped") || report_text.contains("Manual fix"),
        "Report should describe transformations"
    );
}

// ============================================================================
// Property-Based Tests for Purification
// ============================================================================

#[cfg(test)]
mod purify_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Property: Purifying wildcard patterns always wraps with $(sort)
        #[test]
        fn prop_PURIFY_010_wildcard_always_wraps_with_sort(
            pattern in "[a-zA-Z0-9*._/-]{1,20}",
        ) {
            let makefile = format!("FILES := $(wildcard {})", pattern);
            let ast = parse_makefile(&makefile).unwrap();
            let result = purify_makefile(&ast);

            // Should apply transformation
            prop_assert!(result.transformations_applied >= 1,
                        "Should apply at least 1 transformation");

            // Should wrap with sort
            let purified_var = &result.ast.items[0];
            if let MakeItem::Variable { value, .. } = purified_var {
                prop_assert!(value.contains("$(sort $(wildcard"),
                           "Should wrap wildcard with sort");
            } else {
                prop_assert!(false, "Expected Variable item");
            }
        }

        /// Property: Purifying shell find patterns always wraps with $(sort)
        #[test]
        fn prop_PURIFY_011_shell_find_always_wraps_with_sort(
            dir in "[a-zA-Z0-9/_-]{1,15}",
            ext in "[a-z]{1,5}",
        ) {
            let makefile = format!("FILES := $(shell find {} -name '*.{}')", dir, ext);
            let ast = parse_makefile(&makefile).unwrap();
            let result = purify_makefile(&ast);

            // Should apply transformation
            prop_assert!(result.transformations_applied >= 1,
                        "Should apply at least 1 transformation");

            // Should wrap with sort
            let purified_var = &result.ast.items[0];
            if let MakeItem::Variable { value, .. } = purified_var {
                prop_assert!(value.contains("$(sort $(shell find"),
                           "Should wrap shell find with sort");
            } else {
                prop_assert!(false, "Expected Variable item");
            }
        }

        /// Property: Purification is idempotent - purifying twice gives same result
        #[test]
        fn prop_PURIFY_012_idempotent(
            pattern in "[a-zA-Z0-9*._/-]{1,15}",
        ) {
            let makefile = format!("FILES := $(wildcard {})", pattern);
            let ast = parse_makefile(&makefile).unwrap();

            // Purify once
            let result1 = purify_makefile(&ast);

            // Purify again
            let result2 = purify_makefile(&result1.ast);

            // Second purification should do nothing (already purified)
            prop_assert_eq!(result2.transformations_applied, 0,
                           "Second purification should apply 0 transformations");
            prop_assert_eq!(result2.issues_fixed, 0,
                           "Second purification should fix 0 issues");
        }

        /// Property: Purification preserves variable count
        #[test]
        fn prop_PURIFY_013_preserves_variable_count(
            var_name in "[A-Z][A-Z0-9_]{0,10}",
            pattern in "[a-zA-Z0-9*._/-]{1,15}",
        ) {
            let makefile = format!("{} := $(wildcard {})", var_name, pattern);
            let ast = parse_makefile(&makefile).unwrap();
            let original_count = ast.items.len();

            let result = purify_makefile(&ast);

            prop_assert_eq!(result.ast.items.len(), original_count,
                           "Purification should preserve variable count");
        }

        /// Property: Safe patterns require zero transformations
        #[test]
        fn prop_PURIFY_014_safe_patterns_unchanged(
            var_name in "[A-Z][A-Z0-9_]{0,10}",
            value in "[a-zA-Z0-9. _-]{1,30}",
        ) {
            // Only test values that don't contain special characters
            prop_assume!(!value.contains('$'));
            prop_assume!(!value.contains('('));

            let makefile = format!("{} := {}", var_name, value);
            let ast = parse_makefile(&makefile).unwrap();

            let result = purify_makefile(&ast);

            prop_assert_eq!(result.transformations_applied, 0,
                           "Safe patterns should apply 0 transformations");
            prop_assert_eq!(result.issues_fixed, 0,
                           "Safe patterns should fix 0 issues");
        }

        /// Property: Nested patterns are correctly handled
        #[test]
        fn prop_PURIFY_015_nested_in_filter(
            pattern in "[a-zA-Z0-9*._-]{1,15}",
            filter_pattern in "%\\.[a-z]{1,3}",
        ) {
            let makefile = format!("OBJS := $(filter {}, $(wildcard {}))", filter_pattern, pattern);
            let ast = parse_makefile(&makefile).unwrap();

            let result = purify_makefile(&ast);

            // Should apply transformation
            prop_assert!(result.transformations_applied >= 1,
                        "Should apply at least 1 transformation");

            // Inner wildcard should be wrapped
            let purified_var = &result.ast.items[0];
            if let MakeItem::Variable { value, .. } = purified_var {
                prop_assert!(value.contains("$(sort $(wildcard"),
                           "Inner wildcard should be wrapped");
                prop_assert!(value.contains("$(filter"),
                           "Outer filter should be preserved");
            } else {
                prop_assert!(false, "Expected Variable item");
            }
        }

        /// Property: Multiple variables are all purified
        #[test]
        fn prop_PURIFY_016_multiple_variables(
            pattern1 in "[a-zA-Z0-9*._-]{1,10}",
            pattern2 in "[a-zA-Z0-9*._-]{1,10}",
        ) {
            let makefile = format!(
                "FILES1 := $(wildcard {})\nFILES2 := $(wildcard {})",
                pattern1, pattern2
            );
            let ast = parse_makefile(&makefile).unwrap();

            let result = purify_makefile(&ast);

            // Should apply at least 2 transformations (one per variable)
            prop_assert!(result.transformations_applied >= 2,
                        "Should apply at least 2 transformations");

            // Both variables should be purified
            prop_assert_eq!(result.ast.items.len(), 2,
                           "Should have 2 variables");
        }
    }
}

// ============================================================================
// Edge Case Tests for Purification (Mutation Killers)
// ============================================================================

#[test]
fn test_PURIFY_017_edge_case_variable_name_match() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::purify::purify_makefile;

    // ARRANGE: Test that && logic in wrap_variable_with_sort works correctly
    // If replaced with ||, would wrap wrong variables
    let makefile = r#"
FILES := $(wildcard *.c)
OTHER := foo.c bar.c
"#;
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Only FILES should be transformed, not OTHER
    assert_eq!(
        result.transformations_applied, 1,
        "Should apply 1 transformation"
    );

    // Check FILES is wrapped
    let files_var = &result.ast.items[0];
    if let MakeItem::Variable { value, .. } = files_var {
        assert!(
            value.contains("$(sort $(wildcard"),
            "FILES should be wrapped with sort"
        );
    }

    // Check OTHER is unchanged
    let other_var = &result.ast.items[1];
    if let MakeItem::Variable { value, .. } = other_var {
        assert_eq!(value, "foo.c bar.c", "OTHER should be unchanged");
    }
}

#[test]
fn test_PURIFY_018_edge_case_parenthesis_matching_boundary() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::purify::purify_makefile;

    // ARRANGE: Test boundary condition for parenthesis matching
    // Tests the < vs <= condition in find_matching_paren
    let makefile = "X := $(wildcard a)";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Should successfully wrap even single-char pattern
    assert!(
        result.transformations_applied >= 1,
        "Should apply at least 1 transformation"
    );
    let var = &result.ast.items[0];
    if let MakeItem::Variable { value, .. } = var {
        assert_eq!(value, "$(sort $(wildcard a))");
    }
}

#[test]
fn test_PURIFY_019_edge_case_nested_dollar_paren() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::purify::purify_makefile;

    // ARRANGE: Test $( detection logic in find_matching_paren
    // If && replaced with ||, would fail to detect nested patterns
    let makefile = "FILES := $(filter %.c, $(wildcard *.c))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Should wrap inner wildcard
    assert!(
        result.transformations_applied >= 1,
        "Should apply at least 1 transformation"
    );
    let var = &result.ast.items[0];
    if let MakeItem::Variable { value, .. } = var {
        assert!(value.contains("$(sort $(wildcard"), "Should wrap wildcard");
        assert!(value.contains("$(filter"), "Should preserve filter");
    }
}

#[test]
fn test_PURIFY_020_edge_case_empty_pattern() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::purify::purify_makefile;

    // ARRANGE: Empty wildcard pattern
    let makefile = "EMPTY := $(wildcard )";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Should still wrap even empty pattern
    assert!(
        result.transformations_applied >= 1,
        "Should apply at least 1 transformation"
    );
}

#[test]
fn test_PURIFY_021_edge_case_multiple_wildcards_same_variable() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::purify::purify_makefile;

    // ARRANGE: Multiple wildcard calls in same variable
    let makefile = "FILES := $(wildcard *.c) $(wildcard *.h)";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Currently wraps first occurrence
    // Future enhancement: wrap all occurrences
    assert!(result.transformations_applied >= 1);
}

#[test]
fn test_PURIFY_022_edge_case_already_purified_no_double_wrap() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::purify::purify_makefile;

    // ARRANGE: Already purified with $(sort $(wildcard))
    let makefile = "FILES := $(sort $(wildcard *.c))";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Purify twice
    let result1 = purify_makefile(&ast);
    let result2 = purify_makefile(&result1.ast);

    // ASSERT: Second purification should do nothing
    assert_eq!(
        result2.transformations_applied, 0,
        "Already purified should not be re-wrapped"
    );
    assert_eq!(result2.issues_fixed, 0);
}

#[test]
fn test_PURIFY_023_edge_case_shell_find_with_complex_args() {
    use crate::make_parser::parse_makefile;
    use crate::make_parser::purify::purify_makefile;

    // ARRANGE: Shell find with complex arguments
    let makefile = "FILES := $(shell find src -type f -name '*.c' -not -path '*/test/*')";
    let ast = parse_makefile(makefile).unwrap();

    // ACT: Purify
    let result = purify_makefile(&ast);

    // ASSERT: Should wrap complex shell find
    assert!(
        result.transformations_applied >= 1,
        "Should apply at least 1 transformation"
    );
    let var = &result.ast.items[0];
    if let MakeItem::Variable { value, .. } = var {
        assert!(
            value.contains("$(sort $(shell find"),
            "Should wrap shell find with sort"
        );
    }
}

// ============================================================================
// CODE GENERATOR TESTS - Sprint 68
// ============================================================================

/// RED PHASE: Test for GENERATE-001 - Simple variable generation
///
/// Tests that a simple variable assignment can be generated from AST.
///
/// Input AST:
/// ```
/// Variable { name: "CC", value: "gcc", flavor: Simple }
/// ```
///
/// Expected Output:
/// ```makefile
/// CC := gcc
/// ```
#[test]
fn test_GENERATE_001_simple_variable() {
    // ARRANGE: Create AST with simple variable
    let ast = MakeAst {
        items: vec![MakeItem::Variable {
            name: "CC".to_string(),
            value: "gcc".to_string(),
            flavor: VarFlavor::Simple,
            span: Span::dummy(),
        }],
        metadata: MakeMetadata::new(),
    };

    // ACT: Generate Makefile text
    let output = generate_purified_makefile(&ast);

    // ASSERT: Should generate variable assignment
    assert_eq!(output.trim(), "CC := gcc");
}

/// RED PHASE: Test for GENERATE-002 - All variable flavors
///
/// Tests that all 5 variable flavors can be generated correctly.
#[test]
fn test_GENERATE_002_all_variable_flavors() {
    // ARRANGE: Create AST with all variable flavors
    let ast = MakeAst {
        items: vec![
            MakeItem::Variable {
                name: "SIMPLE".to_string(),
                value: "value1".to_string(),
                flavor: VarFlavor::Simple,
                span: Span::dummy(),
            },
            MakeItem::Variable {
                name: "RECURSIVE".to_string(),
                value: "value2".to_string(),
                flavor: VarFlavor::Recursive,
                span: Span::dummy(),
            },
            MakeItem::Variable {
                name: "CONDITIONAL".to_string(),
                value: "value3".to_string(),
                flavor: VarFlavor::Conditional,
                span: Span::dummy(),
            },
            MakeItem::Variable {
                name: "APPEND".to_string(),
                value: "value4".to_string(),
                flavor: VarFlavor::Append,
                span: Span::dummy(),
            },
            MakeItem::Variable {
                name: "SHELL".to_string(),
                value: "command".to_string(),
                flavor: VarFlavor::Shell,
                span: Span::dummy(),
            },
        ],
        metadata: MakeMetadata::new(),
    };

    // ACT: Generate Makefile text
    let output = generate_purified_makefile(&ast);

    // ASSERT: Should generate all variable types with correct operators
    assert!(output.contains("SIMPLE := value1"));
    assert!(output.contains("RECURSIVE = value2"));
    assert!(output.contains("CONDITIONAL ?= value3"));
    assert!(output.contains("APPEND += value4"));
    assert!(output.contains("SHELL != command"));
}

/// RED PHASE: Test for GENERATE-003 - Target with recipe
///
/// Tests that a target with prerequisites and recipe can be generated.
///
/// Expected Output:
/// ```makefile
/// build: main.c
///     gcc -o build main.c
/// ```
#[test]
fn test_GENERATE_003_target_with_recipe() {
    // ARRANGE: Create AST with target
    let ast = MakeAst {
        items: vec![MakeItem::Target {
            name: "build".to_string(),
            prerequisites: vec!["main.c".to_string()],
            recipe: vec!["gcc -o build main.c".to_string()],
            phony: false,
            recipe_metadata: None,
            span: Span::dummy(),
        }],
        metadata: MakeMetadata::new(),
    };

    // ACT: Generate Makefile text
    let output = generate_purified_makefile(&ast);

    // ASSERT: Should generate target with tab-indented recipe
    let lines: Vec<&str> = output.trim().lines().collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "build: main.c");
    assert_eq!(lines[1], "\tgcc -o build main.c");
}

/// RED PHASE: Test for GENERATE-004 - Comment preservation
///
/// Tests that comments are preserved in generated output.
#[test]
fn test_GENERATE_004_comment_preservation() {
    // ARRANGE: Create AST with comment
    let ast = MakeAst {
        items: vec![MakeItem::Comment {
            text: "This is a comment".to_string(),
            span: Span::dummy(),
        }],
        metadata: MakeMetadata::new(),
    };

    // ACT: Generate Makefile text
    let output = generate_purified_makefile(&ast);

    // ASSERT: Should generate comment with # prefix
    assert_eq!(output.trim(), "# This is a comment");
}

/// RED PHASE: Test for GENERATE-005 - PHONY target
///
/// Tests that .PHONY targets are generated correctly.
#[test]
fn test_GENERATE_005_phony_target() {
    // ARRANGE: Create AST with phony target
    let ast = MakeAst {
        items: vec![MakeItem::Target {
            name: "clean".to_string(),
            prerequisites: vec![],
            recipe: vec!["rm -f *.o".to_string()],
            phony: true,
            recipe_metadata: None,
            span: Span::dummy(),
        }],
        metadata: MakeMetadata::new(),
    };

    // ACT: Generate Makefile text
    let output = generate_purified_makefile(&ast);

    // ASSERT: Should generate .PHONY declaration before target
    assert!(output.contains(".PHONY: clean"));
    assert!(output.contains("clean:"));
    assert!(output.contains("\trm -f *.o"));
}

/// RED PHASE: Test for GENERATE-006 - Complex Makefile
///
/// Tests generation of a complex Makefile with multiple items.
#[test]
fn test_GENERATE_006_complex_makefile() {
    // ARRANGE: Create AST with multiple items
    let ast = MakeAst {
        items: vec![
            MakeItem::Comment {
                text: "Build configuration".to_string(),
                span: Span::dummy(),
            },
            MakeItem::Variable {
                name: "CC".to_string(),
                value: "gcc".to_string(),
                flavor: VarFlavor::Simple,
                span: Span::dummy(),
            },
            MakeItem::Variable {
                name: "CFLAGS".to_string(),
                value: "-O2 -Wall".to_string(),
                flavor: VarFlavor::Simple,
                span: Span::dummy(),
            },
            MakeItem::Target {
                name: "all".to_string(),
                prerequisites: vec!["build".to_string()],
                recipe: vec![],
                phony: true,
                recipe_metadata: None,
                span: Span::dummy(),
            },
            MakeItem::Target {
                name: "build".to_string(),
                prerequisites: vec!["main.c".to_string()],
                recipe: vec!["$(CC) $(CFLAGS) -o build main.c".to_string()],
                phony: false,
                recipe_metadata: None,
                span: Span::dummy(),
            },
        ],
        metadata: MakeMetadata::new(),
    };

    // ACT: Generate Makefile text
    let output = generate_purified_makefile(&ast);

    // ASSERT: Should contain all items in order
    assert!(output.contains("# Build configuration"));
    assert!(output.contains("CC := gcc"));
    assert!(output.contains("CFLAGS := -O2 -Wall"));
    assert!(output.contains(".PHONY: all"));
    assert!(output.contains("all: build"));
    assert!(output.contains("build: main.c"));
    assert!(output.contains("\t$(CC) $(CFLAGS) -o build main.c"));
}

// ============================================================================
// GENERATOR PROPERTY TESTS - Sprint 68 Phase 2
// ============================================================================

#[cfg(test)]
mod generator_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// PROPERTY TEST: Round-trip variable generation
        ///
        /// Property: parse(generate(variable)) should preserve variable semantics
        ///
        /// This test generates 100+ random variable names and values, then verifies
        /// that generating Makefile text and parsing it back produces equivalent AST.
        #[test]
        fn prop_GENERATE_007_roundtrip_variables(
            var_name in "[A-Z_][A-Z0-9_]{0,15}",
            var_value in "[a-zA-Z0-9 ./_-]{1,30}",
        ) {
            // ARRANGE: Create variable AST
            let ast = MakeAst {
                items: vec![MakeItem::Variable {
                    name: var_name.clone(),
                    value: var_value.clone(),
                    flavor: VarFlavor::Simple,
                    span: Span::dummy(),
                }],
                metadata: MakeMetadata::new(),
            };

            // ACT: Generate and re-parse
            let generated = generate_purified_makefile(&ast);
            let reparsed = parse_makefile(&generated);

            // ASSERT: Should parse successfully
            prop_assert!(reparsed.is_ok(), "Failed to parse generated Makefile: {}", generated);

            let reparsed_ast = reparsed.unwrap();

            // ASSERT: Should have same number of items
            prop_assert_eq!(reparsed_ast.items.len(), 1);

            // ASSERT: Should preserve variable name and value
            if let MakeItem::Variable { name, value, flavor, .. } = &reparsed_ast.items[0] {
                prop_assert_eq!(name, &var_name);
                prop_assert_eq!(value.trim(), var_value.trim());
                prop_assert_eq!(flavor, &VarFlavor::Simple);
            } else {
                prop_assert!(false, "Expected Variable item, got {:?}", reparsed_ast.items[0]);
            }
        }

        /// PROPERTY TEST: Round-trip target generation
        ///
        /// Property: parse(generate(target)) should preserve target structure
        #[test]
        fn prop_GENERATE_008_roundtrip_targets(
            target_name in "[a-z][a-z0-9_-]{0,15}",
            prereq in "[a-z][a-z0-9_.]{0,15}",
        ) {
            // ARRANGE: Create target AST
            let ast = MakeAst {
                items: vec![MakeItem::Target {
                    name: target_name.clone(),
                    prerequisites: vec![prereq.clone()],
                    recipe: vec!["echo test".to_string()],
                    phony: false,
            recipe_metadata: None,
                    span: Span::dummy(),
                }],
                metadata: MakeMetadata::new(),
            };

            // ACT: Generate and re-parse
            let generated = generate_purified_makefile(&ast);
            let reparsed = parse_makefile(&generated);

            // ASSERT: Should parse successfully
            prop_assert!(reparsed.is_ok(), "Failed to parse generated Makefile: {}", generated);

            let reparsed_ast = reparsed.unwrap();

            // ASSERT: Should have same number of items
            prop_assert_eq!(reparsed_ast.items.len(), 1);

            // ASSERT: Should preserve target structure
            if let MakeItem::Target { name, prerequisites, recipe, .. } = &reparsed_ast.items[0] {
                prop_assert_eq!(name, &target_name);
                prop_assert_eq!(prerequisites.len(), 1);
                prop_assert_eq!(&prerequisites[0], &prereq);
                prop_assert_eq!(recipe.len(), 1);
                prop_assert_eq!(&recipe[0], "echo test");
            } else {
                prop_assert!(false, "Expected Target item, got {:?}", reparsed_ast.items[0]);
            }
        }

        /// PROPERTY TEST: Generation is deterministic
        ///
        /// Property: generate(ast) always produces same output for same input
        #[test]
        fn prop_GENERATE_009_deterministic_generation(
            var_name in "[A-Z_]{2,10}",
            var_value in "[a-z]{1,20}",
        ) {
            // ARRANGE: Create AST
            let ast = MakeAst {
                items: vec![MakeItem::Variable {
                    name: var_name,
                    value: var_value,
                    flavor: VarFlavor::Simple,
                    span: Span::dummy(),
                }],
                metadata: MakeMetadata::new(),
            };

            // ACT: Generate twice
            let output1 = generate_purified_makefile(&ast);
            let output2 = generate_purified_makefile(&ast);

            // ASSERT: Should be byte-identical
            prop_assert_eq!(output1, output2, "Generation is not deterministic");
        }
    }
}

// ============================================================================
// END-TO-END INTEGRATION TEST - Sprint 68
// ============================================================================

/// Integration test: Complete purification workflow
///
/// Tests the full pipeline: Parse → Analyze → Purify → Generate → Verify
///
/// This verifies the entire end-to-end workflow works correctly.
#[test]
fn test_GENERATE_010_end_to_end_purification() {
    // ARRANGE: Input Makefile with non-deterministic wildcard
    let input_makefile = r#"# Build configuration
CC := gcc
CFLAGS := -O2 -Wall

FILES := $(wildcard src/*.c)

build: $(FILES)
	$(CC) $(CFLAGS) -o build $(FILES)
"#;

    // ACT: Parse
    let ast = parse_makefile(input_makefile).expect("Failed to parse input");

    // ACT: Purify (wrap wildcard with sort)
    let purified_result = purify_makefile(&ast);

    // ASSERT: Should have applied transformations
    assert!(
        purified_result.transformations_applied > 0,
        "Expected transformations to be applied"
    );

    // ACT: Generate purified Makefile
    let purified_makefile = generate_purified_makefile(&purified_result.ast);

    // ASSERT: Should contain sorted wildcard
    assert!(
        purified_makefile.contains("$(sort $(wildcard"),
        "Generated Makefile should contain sorted wildcard"
    );

    // ASSERT: Should preserve structure
    assert!(purified_makefile.contains("CC := gcc"));
    assert!(purified_makefile.contains("CFLAGS := -O2 -Wall"));
    assert!(purified_makefile.contains("build: $(FILES)"));
    assert!(purified_makefile.contains("\t$(CC) $(CFLAGS) -o build $(FILES)"));

    // ACT: Re-parse generated Makefile to verify it's valid
    let reparsed = parse_makefile(&purified_makefile);
    assert!(
        reparsed.is_ok(),
        "Generated Makefile should be parseable: {:?}",
        reparsed.err()
    );

    // ASSERT: Re-purification should be idempotent (no changes)
    let reparsed_ast = reparsed.unwrap();
    let repurified = purify_makefile(&reparsed_ast);
    assert_eq!(
        repurified.transformations_applied, 0,
        "Second purification should apply zero transformations (idempotent)"
    );

    println!("\n=== Original Makefile ===");
    println!("{}", input_makefile);
    println!("\n=== Purified Makefile ===");
    println!("{}", purified_makefile);
    println!("\n=== End-to-End Test: PASSED ✅ ===\n");
}

// ============================================================================
// FUNC-CALL-001: Function Call Parsing Tests (Sprint 82, Day 2)
// ============================================================================
//
// These tests validate parsing of GNU Make function calls:
// - $(wildcard pattern)
// - $(patsubst pattern,replacement,text)
// - $(call function,args)
// - $(eval code)
// - $(shell command)
// - $(foreach var,list,text)
// - $(if condition,then,else)
// - $(or a,b)
// - $(and a,b)
// - $(value var)
// - $(origin var)
//
// RED PHASE: These tests are expected to FAIL initially.
// The parser currently stores function calls as raw strings in variable values.
// We need to implement explicit function call parsing.

/// Test for basic $(wildcard) function parsing
///
/// Input: SOURCES := $(wildcard src/*.c)
/// Expected: Parser stores function call in variable value, can extract it
#[test]
fn test_FUNC_CALL_001_wildcard_basic() {
    // ARRANGE: Variable with $(wildcard) function
    let makefile = "SOURCES := $(wildcard src/*.c)";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(
        result.is_ok(),
        "Should parse $(wildcard) function, got error: {:?}",
        result.err()
    );

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 variable");

    // ASSERT: Variable contains the function call
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "SOURCES");
            assert_eq!(value, "$(wildcard src/*.c)");

            // ASSERT: Can extract function calls from value
            let function_calls = extract_function_calls(value);
            assert_eq!(function_calls.len(), 1, "Should extract 1 function call");
            assert_eq!(
                function_calls[0].0, "wildcard",
                "Function name should be 'wildcard'"
            );
            assert!(
                function_calls[0].1.contains("src/*.c"),
                "Args should contain pattern"
            );
        }
        _ => panic!("Expected Variable item"),
    }
}

/// RED PHASE: Test for $(wildcard) with multiple patterns
///
/// Input: FILES := $(wildcard *.c *.h)
#[test]
fn test_FUNC_CALL_002_wildcard_multiple_patterns() {
    // ARRANGE: $(wildcard) with multiple patterns
    let makefile = "FILES := $(wildcard *.c *.h)";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 variable");

    // ASSERT: Should store as Variable and allow extraction
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "FILES");
            assert_eq!(value, "$(wildcard *.c *.h)");

            // Can extract function calls from value
            let function_calls = extract_function_calls(value);
            assert_eq!(function_calls.len(), 1, "Should extract 1 function call");
            assert_eq!(
                function_calls[0].0, "wildcard",
                "Function name should be 'wildcard'"
            );
            assert!(
                function_calls[0].1.contains("*.c"),
                "Args should contain *.c pattern"
            );
            assert!(
                function_calls[0].1.contains("*.h"),
                "Args should contain *.h pattern"
            );
        }
        _ => panic!("Expected Variable item"),
    }
}

/// RED PHASE: Test for basic $(patsubst) function
///
/// Input: OBJS := $(patsubst %.c,%.o,$(SOURCES))
#[test]
fn test_FUNC_CALL_003_patsubst_basic() {
    // ARRANGE: $(patsubst) function
    let makefile = "OBJS := $(patsubst %.c,%.o,$(SOURCES))";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 variable");

    // ASSERT: Should store as Variable and allow extraction
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "OBJS");
            assert_eq!(value, "$(patsubst %.c,%.o,$(SOURCES))");

            // Can extract function calls from value
            let function_calls = extract_function_calls(value);
            assert_eq!(function_calls.len(), 1, "Should extract 1 function call");
            assert_eq!(
                function_calls[0].0, "patsubst",
                "Function name should be 'patsubst'"
            );
            assert!(
                function_calls[0].1.contains("%.c"),
                "Args should contain %.c pattern"
            );
            assert!(
                function_calls[0].1.contains("%.o"),
                "Args should contain %.o pattern"
            );
        }
        _ => panic!("Expected Variable item"),
    }
}

/// RED PHASE: Test for $(patsubst) with nested variable
///
/// Input: OBJS := $(patsubst %.c,%.o,$(wildcard src/*.c))
#[test]
fn test_FUNC_CALL_004_patsubst_nested() {
    // ARRANGE: $(patsubst) with nested $(wildcard)
    let makefile = "OBJS := $(patsubst %.c,%.o,$(wildcard src/*.c))";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 variable");

    // ASSERT: Should store as Variable and allow extraction (outer function)
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "OBJS");
            assert_eq!(value, "$(patsubst %.c,%.o,$(wildcard src/*.c))");

            // Can extract function calls from value (extracts outermost)
            let function_calls = extract_function_calls(value);
            assert_eq!(
                function_calls.len(),
                1,
                "Should extract 1 outermost function call"
            );
            assert_eq!(
                function_calls[0].0, "patsubst",
                "Function name should be 'patsubst'"
            );
            assert!(
                function_calls[0].1.contains("$(wildcard"),
                "Args should contain nested $(wildcard)"
            );
        }
        _ => panic!("Expected Variable item"),
    }
}

/// RED PHASE: Test for $(call) function
///
/// Input: RESULT := $(call my_func,arg1,arg2)
#[test]
fn test_FUNC_CALL_005_call_basic() {
    // ARRANGE: $(call) function
    let makefile = "RESULT := $(call my_func,arg1,arg2)";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 variable");

    // ASSERT: Should store as Variable and allow extraction
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "RESULT");
            assert_eq!(value, "$(call my_func,arg1,arg2)");

            // Can extract function calls from value
            let function_calls = extract_function_calls(value);
            assert_eq!(function_calls.len(), 1, "Should extract 1 function call");
            assert_eq!(
                function_calls[0].0, "call",
                "Function name should be 'call'"
            );
            assert!(
                function_calls[0].1.contains("my_func"),
                "Args should contain my_func"
            );
            assert!(
                function_calls[0].1.contains("arg1"),
                "Args should contain arg1"
            );
        }
        _ => panic!("Expected Variable item"),
    }
}

/// RED PHASE: Test for nested $(call) function
///
/// Input: RESULT := $(call outer,$(call inner,x))
#[test]
fn test_FUNC_CALL_006_call_nested() {
    // ARRANGE: Nested $(call) functions
    let makefile = "RESULT := $(call outer,$(call inner,x))";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 variable");

    // ASSERT: Should store as Variable and allow extraction (outer call)
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "RESULT");
            assert_eq!(value, "$(call outer,$(call inner,x))");

            // Can extract function calls from value (extracts outermost)
            let function_calls = extract_function_calls(value);
            assert_eq!(
                function_calls.len(),
                1,
                "Should extract 1 outermost function call"
            );
            assert_eq!(
                function_calls[0].0, "call",
                "Function name should be 'call'"
            );
            assert!(
                function_calls[0].1.contains("outer"),
                "Args should contain outer"
            );
            assert!(
                function_calls[0].1.contains("$(call inner"),
                "Args should contain nested $(call inner)"
            );
        }
        _ => panic!("Expected Variable item"),
    }
}

/// RED PHASE: Test for $(eval) function
///
/// Input: $(eval VAR = value)
#[test]
fn test_FUNC_CALL_007_eval_basic() {
    // ARRANGE: $(eval) function (note: eval is typically standalone, not in assignment)
    let makefile = "DUMMY := $(eval NEW_VAR = value)";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 variable");

    // ASSERT: Should store as Variable and allow extraction
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "DUMMY");
            assert_eq!(value, "$(eval NEW_VAR = value)");

            // Can extract function calls from value
            let function_calls = extract_function_calls(value);
            assert_eq!(function_calls.len(), 1, "Should extract 1 function call");
            assert_eq!(
                function_calls[0].0, "eval",
                "Function name should be 'eval'"
            );
            assert!(
                function_calls[0].1.contains("NEW_VAR"),
                "Args should contain NEW_VAR"
            );
        }
        _ => panic!("Expected Variable item"),
    }
}

/// RED PHASE: Test for $(shell) function
///
/// Input: FILES := $(shell ls -la)
#[test]
fn test_FUNC_CALL_008_shell_basic() {
    // ARRANGE: $(shell) function
    let makefile = "FILES := $(shell ls -la)";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 variable");

    // ASSERT: Should store as Variable and allow extraction
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "FILES");
            assert_eq!(value, "$(shell ls -la)");

            // Can extract function calls from value
            let function_calls = extract_function_calls(value);
            assert_eq!(function_calls.len(), 1, "Should extract 1 function call");
            assert_eq!(
                function_calls[0].0, "shell",
                "Function name should be 'shell'"
            );
            assert!(
                function_calls[0].1.contains("ls -la"),
                "Args should contain shell command"
            );
        }
        _ => panic!("Expected Variable item"),
    }
}

/// RED PHASE: Test for $(foreach) function
///
/// Input: FILES := $(foreach dir,src test,$(wildcard $(dir)/*.c))
#[test]
fn test_FUNC_CALL_009_foreach_basic() {
    // ARRANGE: $(foreach) function with nested wildcard
    let makefile = "FILES := $(foreach dir,src test,$(wildcard $(dir)/*.c))";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 variable");

    // ASSERT: Should store as Variable and allow extraction
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "FILES");
            assert_eq!(value, "$(foreach dir,src test,$(wildcard $(dir)/*.c))");

            // Can extract function calls from value (extracts outermost)
            let function_calls = extract_function_calls(value);
            assert_eq!(
                function_calls.len(),
                1,
                "Should extract 1 outermost function call"
            );
            assert_eq!(
                function_calls[0].0, "foreach",
                "Function name should be 'foreach'"
            );
            assert!(
                function_calls[0].1.contains("dir,src test"),
                "Args should contain foreach parameters"
            );
        }
        _ => panic!("Expected Variable item"),
    }
}

/// RED PHASE: Test for $(if) function
///
/// Input: RESULT := $(if $(DEBUG),debug,release)
#[test]
fn test_FUNC_CALL_010_if_basic() {
    // ARRANGE: $(if) function with condition and branches
    let makefile = "RESULT := $(if $(DEBUG),debug,release)";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 variable");

    // ASSERT: Should store as Variable and allow extraction
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "RESULT");
            assert_eq!(value, "$(if $(DEBUG),debug,release)");

            // Can extract function calls from value
            let function_calls = extract_function_calls(value);
            assert_eq!(function_calls.len(), 1, "Should extract 1 function call");
            assert_eq!(function_calls[0].0, "if", "Function name should be 'if'");
            assert!(
                function_calls[0].1.contains("DEBUG"),
                "Args should contain condition"
            );
            assert!(
                function_calls[0].1.contains("debug"),
                "Args should contain then-branch"
            );
        }
        _ => panic!("Expected Variable item"),
    }
}

/// RED PHASE: Test for $(or) function
///
/// Input: ENABLED := $(or $(USE_FEATURE_A),$(USE_FEATURE_B))
#[test]
fn test_FUNC_CALL_011_or_basic() {
    // ARRANGE: $(or) function with multiple conditions
    let makefile = "ENABLED := $(or $(USE_FEATURE_A),$(USE_FEATURE_B))";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 variable");

    // ASSERT: Should store as Variable and allow extraction
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "ENABLED");
            assert_eq!(value, "$(or $(USE_FEATURE_A),$(USE_FEATURE_B))");

            // Can extract function calls from value
            let function_calls = extract_function_calls(value);
            assert_eq!(function_calls.len(), 1, "Should extract 1 function call");
            assert_eq!(function_calls[0].0, "or", "Function name should be 'or'");
            assert!(
                function_calls[0].1.contains("USE_FEATURE_A"),
                "Args should contain first condition"
            );
            assert!(
                function_calls[0].1.contains("USE_FEATURE_B"),
                "Args should contain second condition"
            );
        }
        _ => panic!("Expected Variable item"),
    }
}

/// RED PHASE: Test for $(and) function
///
/// Input: VALID := $(and $(HAS_COMPILER),$(HAS_LIBS))
#[test]
fn test_FUNC_CALL_012_and_basic() {
    // ARRANGE: $(and) function with multiple conditions
    let makefile = "VALID := $(and $(HAS_COMPILER),$(HAS_LIBS))";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 variable");

    // ASSERT: Should store as Variable and allow extraction
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "VALID");
            assert_eq!(value, "$(and $(HAS_COMPILER),$(HAS_LIBS))");

            // Can extract function calls from value
            let function_calls = extract_function_calls(value);
            assert_eq!(function_calls.len(), 1, "Should extract 1 function call");
            assert_eq!(function_calls[0].0, "and", "Function name should be 'and'");
            assert!(
                function_calls[0].1.contains("HAS_COMPILER"),
                "Args should contain first condition"
            );
            assert!(
                function_calls[0].1.contains("HAS_LIBS"),
                "Args should contain second condition"
            );
        }
        _ => panic!("Expected Variable item"),
    }
}

/// RED PHASE: Test for $(value) function
///
/// Input: VAR_CONTENT := $(value VARIABLE_NAME)
#[test]
fn test_FUNC_CALL_013_value_basic() {
    // ARRANGE: $(value) function to get variable value without expansion
    let makefile = "VAR_CONTENT := $(value VARIABLE_NAME)";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 variable");

    // ASSERT: Should store as Variable and allow extraction
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "VAR_CONTENT");
            assert_eq!(value, "$(value VARIABLE_NAME)");

            // Can extract function calls from value
            let function_calls = extract_function_calls(value);
            assert_eq!(function_calls.len(), 1, "Should extract 1 function call");
            assert_eq!(
                function_calls[0].0, "value",
                "Function name should be 'value'"
            );
            assert!(
                function_calls[0].1.contains("VARIABLE_NAME"),
                "Args should contain variable name"
            );
        }
        _ => panic!("Expected Variable item"),
    }
}

/// RED PHASE: Test for $(origin) function
///
/// Input: VAR_ORIGIN := $(origin CC)
#[test]
fn test_FUNC_CALL_014_origin_basic() {
    // ARRANGE: $(origin) function to check variable origin
    let makefile = "VAR_ORIGIN := $(origin CC)";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 variable");

    // ASSERT: Should store as Variable and allow extraction
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "VAR_ORIGIN");
            assert_eq!(value, "$(origin CC)");

            // Can extract function calls from value
            let function_calls = extract_function_calls(value);
            assert_eq!(function_calls.len(), 1, "Should extract 1 function call");
            assert_eq!(
                function_calls[0].0, "origin",
                "Function name should be 'origin'"
            );
            assert!(
                function_calls[0].1.contains("CC"),
                "Args should contain variable name"
            );
        }
        _ => panic!("Expected Variable item"),
    }
}

/// RED PHASE: Test for multiple function calls in one variable
///
/// Input: ALL := $(wildcard *.c) $(patsubst %.c,%.o,$(wildcard *.c))
#[test]
fn test_FUNC_CALL_015_multiple_functions() {
    // ARRANGE: Multiple function calls in one variable value
    let makefile = "ALL := $(wildcard *.c) $(patsubst %.c,%.o,$(wildcard *.c))";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 variable");

    // ASSERT: Should store as Variable and allow extraction of multiple calls
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "ALL");
            assert_eq!(value, "$(wildcard *.c) $(patsubst %.c,%.o,$(wildcard *.c))");

            // Can extract multiple function calls from value
            let function_calls = extract_function_calls(value);
            assert_eq!(function_calls.len(), 2, "Should extract 2 function calls");
            assert_eq!(
                function_calls[0].0, "wildcard",
                "First function should be 'wildcard'"
            );
            assert_eq!(
                function_calls[1].0, "patsubst",
                "Second function should be 'patsubst'"
            );
            assert!(
                function_calls[0].1.contains("*.c"),
                "First function args should contain *.c"
            );
            assert!(
                function_calls[1].1.contains("%.c"),
                "Second function args should contain %.c"
            );
        }
        _ => panic!("Expected Variable item"),
    }
}

// =============================================================================
// define...endef Tests (Sprint 82 Day 4-5)
// =============================================================================

/// RED PHASE: Test for basic define...endef
///
/// Input:
/// define COMPILE_RULE
/// gcc -c $< -o $@
/// endef
#[test]
fn test_DEFINE_001_basic_define() {
    // ARRANGE: Basic define...endef block
    let makefile = r#"define COMPILE_RULE
gcc -c $< -o $@
endef"#;

    // ACT: Parse makefile
    let result = parse_makefile(makefile);
    assert!(result.is_ok(), "Parsing should succeed");

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 variable");

    // ASSERT: Should store as multi-line Variable
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "COMPILE_RULE");
            assert!(
                value.contains("gcc -c $< -o $@"),
                "Value should contain command"
            );
        }
        _ => panic!("Expected Variable item for define block"),
    }
}

/// RED PHASE: Test for empty define...endef
///
/// Input:
/// define EMPTY_VAR
/// endef
#[test]
fn test_DEFINE_002_empty_define() {
    // ARRANGE: Empty define block
    let makefile = r#"define EMPTY_VAR
endef"#;

    // ACT: Parse makefile
    let result = parse_makefile(makefile);
    assert!(result.is_ok(), "Parsing should succeed");

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 variable");

    // ASSERT: Should store as Variable with empty or whitespace value
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "EMPTY_VAR");
            assert!(
                value.trim().is_empty() || value.is_empty(),
                "Value should be empty"
            );
        }
        _ => panic!("Expected Variable item"),
    }
}

/// RED PHASE: Test for multi-line define...endef
///
/// Input:
/// define HELP_TEXT
/// Usage: make [target]
/// Targets:
///   all    - Build everything
///   clean  - Remove build artifacts
/// endef
#[test]
fn test_DEFINE_003_multiline_text() {
    // ARRANGE: Multi-line define block
    let makefile = r#"define HELP_TEXT
Usage: make [target]
Targets:
  all    - Build everything
  clean  - Remove build artifacts
endef"#;

    // ACT: Parse makefile
    let result = parse_makefile(makefile);
    assert!(result.is_ok(), "Parsing should succeed");

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 variable");

    // ASSERT: Should preserve multi-line content
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "HELP_TEXT");
            assert!(
                value.contains("Usage: make [target]"),
                "Should contain first line"
            );
            assert!(value.contains("Targets:"), "Should contain second line");
            assert!(
                value.contains("all    - Build everything"),
                "Should contain third line"
            );
            assert!(
                value.contains("clean  - Remove build artifacts"),
                "Should contain fourth line"
            );
        }
        _ => panic!("Expected Variable item"),
    }
}

/// RED PHASE: Test for define with tab-indented commands
///
/// Input:
/// define BUILD_CMD
///     @echo "Building..."
///     gcc -o output main.c
/// endef
#[test]
fn test_DEFINE_004_with_tabs() {
    // ARRANGE: define block with tab-indented commands (like recipe lines)
    let makefile = "define BUILD_CMD\n\t@echo \"Building...\"\n\tgcc -o output main.c\nendef";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);
    assert!(result.is_ok(), "Parsing should succeed");

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 variable");

    // ASSERT: Should preserve tabs in multi-line value
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "BUILD_CMD");
            assert!(value.contains("echo"), "Should contain echo command");
            assert!(
                value.contains("gcc -o output main.c"),
                "Should contain gcc command"
            );
        }
        _ => panic!("Expected Variable item"),
    }
}

/// RED PHASE: Test for define with variable references
///
/// Input:
/// define INSTALL_CMD
/// install -m 755 $(BIN) $(DESTDIR)$(PREFIX)/bin
/// install -m 644 $(MAN) $(DESTDIR)$(PREFIX)/share/man
/// endef
#[test]
fn test_DEFINE_005_with_variables() {
    // ARRANGE: define block with variable references
    let makefile = r#"define INSTALL_CMD
install -m 755 $(BIN) $(DESTDIR)$(PREFIX)/bin
install -m 644 $(MAN) $(DESTDIR)$(PREFIX)/share/man
endef"#;

    // ACT: Parse makefile
    let result = parse_makefile(makefile);
    assert!(result.is_ok(), "Parsing should succeed");

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 variable");

    // ASSERT: Should preserve variable references
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "INSTALL_CMD");
            assert!(value.contains("$(BIN)"), "Should contain BIN variable");
            assert!(
                value.contains("$(DESTDIR)"),
                "Should contain DESTDIR variable"
            );
            assert!(
                value.contains("$(PREFIX)"),
                "Should contain PREFIX variable"
            );
        }
        _ => panic!("Expected Variable item"),
    }
}

/// RED PHASE: Test for define with recipe-style commands
///
/// Input:
/// define RUN_TESTS
///     cd tests && ./run_tests.sh
///     if [ $$? -ne 0 ]; then exit 1; fi
/// endef
#[test]
fn test_DEFINE_006_with_commands() {
    // ARRANGE: define block with shell commands
    let makefile = "define RUN_TESTS\n\tcd tests && ./run_tests.sh\n\tif [ $$? -ne 0 ]; then exit 1; fi\nendef";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);
    assert!(result.is_ok(), "Parsing should succeed");

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 variable");

    // ASSERT: Should preserve shell commands with $$
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "RUN_TESTS");
            assert!(value.contains("cd tests"), "Should contain cd command");
            assert!(
                value.contains("$$?") || value.contains("$?"),
                "Should contain exit code check"
            );
        }
        _ => panic!("Expected Variable item"),
    }
}

/// RED PHASE: Test for recursive expansion define (=)
///
/// Input:
/// define RECURSIVE =
/// This is $(FLAVOR) expansion
/// endef
#[test]
fn test_DEFINE_007_recursive_expansion() {
    // ARRANGE: define with recursive expansion (=)
    let makefile = r#"define RECURSIVE =
This is $(FLAVOR) expansion
endef"#;

    // ACT: Parse makefile
    let result = parse_makefile(makefile);
    assert!(result.is_ok(), "Parsing should succeed");

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 variable");

    // ASSERT: Should handle = flavor
    match &ast.items[0] {
        MakeItem::Variable {
            name,
            value,
            flavor,
            ..
        } => {
            assert_eq!(name, "RECURSIVE");
            assert!(
                value.contains("$(FLAVOR)"),
                "Should contain variable reference"
            );
            assert_eq!(
                *flavor,
                VarFlavor::Recursive,
                "Should be recursive expansion"
            );
        }
        _ => panic!("Expected Variable item"),
    }
}

/// RED PHASE: Test for simple expansion define (:=)
///
/// Input:
/// define SIMPLE :=
/// Expanded at $(shell date)
/// endef
#[test]
fn test_DEFINE_008_simple_expansion() {
    // ARRANGE: define with simple expansion (:=)
    let makefile = r#"define SIMPLE :=
Expanded at $(shell date)
endef"#;

    // ACT: Parse makefile
    let result = parse_makefile(makefile);
    assert!(result.is_ok(), "Parsing should succeed");

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 variable");

    // ASSERT: Should handle := flavor
    match &ast.items[0] {
        MakeItem::Variable {
            name,
            value,
            flavor,
            ..
        } => {
            assert_eq!(name, "SIMPLE");
            assert!(
                value.contains("shell") || value.contains("date"),
                "Should contain function call"
            );
            assert_eq!(*flavor, VarFlavor::Simple, "Should be simple expansion");
        }
        _ => panic!("Expected Variable item"),
    }
}

/// RED PHASE: Test for define with nested variable expansion
///
/// Input:
/// define COMPLEX
/// SRC = $(wildcard src/*.c)
/// OBJ = $(patsubst %.c,%.o,$(SRC))
/// endef
#[test]
fn test_DEFINE_009_nested_variables() {
    // ARRANGE: define block with nested variable assignments
    let makefile = r#"define COMPLEX
SRC = $(wildcard src/*.c)
OBJ = $(patsubst %.c,%.o,$(SRC))
endef"#;

    // ACT: Parse makefile
    let result = parse_makefile(makefile);
    assert!(result.is_ok(), "Parsing should succeed");

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 variable");

    // ASSERT: Should preserve nested content as multi-line value
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "COMPLEX");
            assert!(value.contains("SRC ="), "Should contain SRC assignment");
            assert!(value.contains("OBJ ="), "Should contain OBJ assignment");
            assert!(
                value.contains("$(wildcard"),
                "Should contain wildcard function"
            );
            assert!(
                value.contains("$(patsubst"),
                "Should contain patsubst function"
            );
        }
        _ => panic!("Expected Variable item"),
    }
}

/// RED PHASE: Test for real-world define example
///
/// Input: Complex real-world define block from Linux kernel
#[test]
fn test_DEFINE_010_real_world_example() {
    // ARRANGE: Real-world complex define block
    let makefile = r#"define COMPILE_TEMPLATE
$(1)_OBJS := $$(patsubst %.c,%.o,$$($(1)_SOURCES))
$(1)_DEPS := $$($(1)_OBJS:.o=.d)

$(1): $$($(1)_OBJS)
	$$(CC) $$(CFLAGS) -o $$@ $$^

-include $$($(1)_DEPS)
endef"#;

    // ACT: Parse makefile
    let result = parse_makefile(makefile);
    assert!(result.is_ok(), "Parsing should succeed");

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 variable");

    // ASSERT: Should preserve complex multi-line template
    match &ast.items[0] {
        MakeItem::Variable { name, value, .. } => {
            assert_eq!(name, "COMPILE_TEMPLATE");
            assert!(value.contains("_OBJS"), "Should contain OBJS assignment");
            assert!(value.contains("_DEPS"), "Should contain DEPS assignment");
            assert!(value.contains("$(CC)"), "Should contain CC variable");
            assert!(value.contains("$$@"), "Should contain automatic variable");
            assert!(
                value.contains("-include"),
                "Should contain include directive"
            );
        }
        _ => panic!("Expected Variable item"),
    }
}

// =============================================================================
// Conditional Edge Cases (Day 6)
// =============================================================================

/// RED PHASE: Test for nested conditionals (ifeq inside ifdef)
#[test]
fn test_COND_EDGE_001_nested_ifeq_ifdef() {
    let makefile = r#"
ifdef DEBUG
ifeq ($(VERBOSE),1)
CFLAGS += -DDEBUG_VERBOSE
endif
endif
"#;

    // ARRANGE: Parse Makefile with nested conditionals
    let result = parse_makefile(makefile);
    assert!(result.is_ok(), "Parsing should succeed");

    let ast = result.unwrap();

    // ASSERT: Should have conditional structure
    // Outer ifdef DEBUG should contain inner ifeq
    let has_ifdef = ast
        .items
        .iter()
        .any(|item| matches!(item, MakeItem::Conditional { .. }));

    assert!(
        has_ifdef,
        "Should have conditional items for nested structure"
    );
}

/// RED PHASE: Test for conditionals with function calls in condition
#[test]
fn test_COND_EDGE_002_conditional_with_functions() {
    let makefile = r#"
ifeq ($(shell uname),Linux)
PLATFORM = linux
else
PLATFORM = other
endif
"#;

    // ARRANGE: Parse Makefile with function call in condition
    let result = parse_makefile(makefile);
    assert!(result.is_ok(), "Parsing should succeed");

    let ast = result.unwrap();

    // ASSERT: Should parse ifeq with shell function
    let has_conditional = ast
        .items
        .iter()
        .any(|item| matches!(item, MakeItem::Conditional { .. }));

    assert!(has_conditional, "Should have conditional item");

    // ASSERT: Should have variable assignment in then or else branch
    let has_var_in_conditional = ast.items.iter().any(|item| {
        if let MakeItem::Conditional {
            then_items,
            else_items,
            ..
        } = item
        {
            let in_then = then_items
                .iter()
                .any(|i| matches!(i, MakeItem::Variable { name, .. } if name == "PLATFORM"));
            let in_else = else_items
                .as_ref()
                .map(|items| {
                    items
                        .iter()
                        .any(|i| matches!(i, MakeItem::Variable { name, .. } if name == "PLATFORM"))
                })
                .unwrap_or(false);
            in_then || in_else
        } else {
            false
        }
    });

    assert!(
        has_var_in_conditional,
        "Should have PLATFORM variable in conditional branches"
    );
}

/// RED PHASE: Test for empty conditional blocks
#[test]
fn test_COND_EDGE_003_empty_conditional_blocks() {
    let makefile = r#"
ifdef DEBUG
# Empty then block
endif

ifndef RELEASE
# Empty then block
else
# Empty else block
endif
"#;

    // ARRANGE: Parse Makefile with empty conditional blocks
    let result = parse_makefile(makefile);
    assert!(result.is_ok(), "Parsing should succeed for empty blocks");

    let ast = result.unwrap();

    // ASSERT: Should have conditional items even if empty
    let conditional_count = ast
        .items
        .iter()
        .filter(|item| matches!(item, MakeItem::Conditional { .. }))
        .count();

    assert!(
        conditional_count >= 2,
        "Should have 2 conditional items (ifdef + ifndef)"
    );
}

/// RED PHASE: Test for complex real-world nesting
#[test]
fn test_COND_EDGE_004_complex_nesting_real_world() {
    let makefile = r#"
ifdef USE_PYTHON
PYTHON := python3
ifeq ($(shell which python3),)
$(error Python 3 not found)
endif
else ifdef USE_PYTHON2
PYTHON := python2
else
PYTHON := python
endif

ifneq ($(PYTHON),)
PYTHON_CONFIG := $(PYTHON)-config
endif
"#;

    // ARRANGE: Parse complex real-world conditional nesting
    let result = parse_makefile(makefile);
    assert!(result.is_ok(), "Parsing should succeed for complex nesting");

    let ast = result.unwrap();

    // ASSERT: Should have multiple conditional items
    let conditional_count = ast
        .items
        .iter()
        .filter(|item| matches!(item, MakeItem::Conditional { .. }))
        .count();

    assert!(
        conditional_count >= 2,
        "Should have at least 2 conditional items"
    );

    // ASSERT: Should have variable assignments in conditional branches
    let has_python_var = ast.items.iter().any(|item| {
        if let MakeItem::Conditional { then_items, else_items, .. } = item {
            let check_items = |items: &[MakeItem]| {
                items.iter().any(|i| {
                    matches!(i, MakeItem::Variable { name, .. } if name == "PYTHON" || name == "PYTHON_CONFIG")
                })
            };

            let in_then = check_items(then_items);
            let in_else = else_items.as_ref().map(|items| check_items(items)).unwrap_or(false);
            in_then || in_else
        } else {
            false
        }
    });

    assert!(
        has_python_var,
        "Should have PYTHON or PYTHON_CONFIG variable in conditional branches"
    );
}

/// RED PHASE: Test for multiple nested conditional levels
#[test]
fn test_COND_EDGE_005_multiple_nested_levels() {
    let makefile = r#"
ifdef ENABLE_FEATURE_A
FEATURE_A = 1
ifdef ENABLE_FEATURE_A_VERBOSE
FEATURE_A_FLAGS = -v
else
FEATURE_A_FLAGS =
endif
else
FEATURE_A = 0
endif
"#;

    // ARRANGE: Parse conditional with multiple nesting levels
    let result = parse_makefile(makefile);
    assert!(
        result.is_ok(),
        "Parsing should succeed for multiple nesting levels"
    );

    let ast = result.unwrap();

    // ASSERT: Should have conditional item
    let has_conditional = ast
        .items
        .iter()
        .any(|item| matches!(item, MakeItem::Conditional { .. }));

    assert!(has_conditional, "Should have conditional item");

    // ASSERT: Should have variable assignments in conditional branches
    let has_var = ast.items.iter().any(|item| {
        if let MakeItem::Conditional {
            then_items,
            else_items,
            ..
        } = item
        {
            let check_items = |items: &[MakeItem]| {
                items.iter().any(|i| {
                    matches!(i, MakeItem::Variable { name, .. }
                        if name == "FEATURE_A" || name == "FEATURE_A_FLAGS")
                })
            };

            let in_then = check_items(then_items);
            let in_else = else_items
                .as_ref()
                .map(|items| check_items(items))
                .unwrap_or(false);
            in_then || in_else
        } else {
            false
        }
    });

    assert!(
        has_var,
        "Should have FEATURE_A or FEATURE_A_FLAGS variable in conditional branches"
    );
}
