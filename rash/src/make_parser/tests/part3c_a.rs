#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(unused_imports)]

use super::super::*;
use crate::make_parser::ast::{MakeCondition, Span, VarFlavor};

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
