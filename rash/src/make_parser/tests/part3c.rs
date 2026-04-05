#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(unused_imports)]

use super::super::*;
use crate::make_parser::ast::{MakeCondition, Span, VarFlavor};

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
