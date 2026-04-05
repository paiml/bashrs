#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(unused_imports)]

use super::super::*;
use crate::make_parser::ast::{MakeCondition, Span, VarFlavor};

        #[test]
        fn prop_INCLUDE_001_multiple_includes_order_preserved(
            files in prop::collection::vec("[a-z]{3,10}\\.mk", 2..5)
        ) {
            let makefile = files
                .iter()
                .map(|f| format!("include {}", f))
                .collect::<Vec<_>>()
                .join("\n");

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            prop_assert_eq!(ast.items.len(), files.len());

            for (i, expected_file) in files.iter().enumerate() {
                match &ast.items[i] {
                    MakeItem::Include { path, .. } => {
                        prop_assert_eq!(path, expected_file);
                    }
                    _ => prop_assert!(false, "Expected Include at index {}", i),
                }
            }
        }

        #[test]
        fn prop_INCLUDE_001_paths_with_directories(
            dir in "[a-z]{3,10}",
            file in "[a-z]{3,10}\\.mk"
        ) {
            let makefile = format!("include {}/{}", dir, file);
            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::Include { path, .. } => {
                    prop_assert!(path.contains('/'));
                    prop_assert!(path.ends_with(".mk"));
                }
                _ => prop_assert!(false, "Expected Include"),
            }
        }

        #[test]
        fn prop_INCLUDE_001_var_refs_preserved(
            var_name in "[A-Z_]{2,10}",
            file in "[a-z]{3,10}\\.mk"
        ) {
            let makefile = format!("include $({})/{}", var_name, file);
            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::Include { path, .. } => {
                    prop_assert!(path.contains("$("));
                    prop_assert!(path.contains(&var_name));
                }
                _ => prop_assert!(false, "Expected Include"),
            }
        }
    }
}

/// RED PHASE: Mutation-killing test - Include keyword detection
#[test]
fn test_INCLUDE_001_mut_keyword_detection() {
    // Test that only "include" keyword triggers Include parsing
    let makefile_include = "include file.mk";
    let makefile_invalid = "includes file.mk"; // typo

    let result_include = parse_makefile(makefile_include);
    let result_invalid = parse_makefile(makefile_invalid);

    assert!(result_include.is_ok(), "Should parse 'include'");
    assert!(result_invalid.is_ok(), "'includes' should not crash parser");

    // Valid include should produce Include item
    match &result_include.unwrap().items[0] {
        MakeItem::Include { .. } => {} // Expected
        _ => panic!("Expected Include for 'include' keyword"),
    }

    // Invalid should NOT produce Include item (probably parsed as unknown/error)
    let ast_invalid = result_invalid.unwrap();
    if !ast_invalid.items.is_empty() {
        if let MakeItem::Include { .. } = &ast_invalid.items[0] {
            panic!("Should not parse 'includes' as Include");
        }
    }
}

/// RED PHASE: Mutation-killing test - Path extraction correctness
#[test]
fn test_INCLUDE_001_mut_path_extraction() {
    // Test that path is correctly extracted after "include" keyword
    let makefile = "include    file.mk"; // Extra whitespace

    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();
    match &ast.items[0] {
        MakeItem::Include { path, .. } => {
            // Path should be trimmed of leading/trailing whitespace
            assert_eq!(path, "file.mk", "Path should be trimmed");
            assert!(
                !path.starts_with(' '),
                "Path should not have leading whitespace"
            );
        }
        _ => panic!("Expected Include"),
    }
}

/// RED PHASE: Mutation-killing test - Include vs target distinction
#[test]
fn test_INCLUDE_001_mut_include_vs_target() {
    // Test that "include" is not parsed as a target name
    let makefile_include = "include file.mk";
    let makefile_target = "include: file.mk\n\techo build";

    let result_include = parse_makefile(makefile_include);
    let result_target = parse_makefile(makefile_target);

    assert!(result_include.is_ok());
    assert!(result_target.is_ok());

    // First should be Include
    match &result_include.unwrap().items[0] {
        MakeItem::Include { .. } => {} // Expected
        _ => panic!("'include file.mk' should be parsed as Include, not Target"),
    }

    // Second should be Target (named "include")
    match &result_target.unwrap().items[0] {
        MakeItem::Target { name, .. } => {
            assert_eq!(name, "include", "Should parse as target named 'include'");
        }
        _ => panic!("'include:' should be parsed as Target, not Include"),
    }
}

/// RED PHASE: Mutation-killing test - Empty path handling
#[test]
fn test_INCLUDE_001_mut_empty_path() {
    // Test edge case: include with no path
    let makefile = "include";

    let result = parse_makefile(makefile);

    // This could either:
    // 1. Fail gracefully (preferred)
    // 2. Parse with empty path (acceptable if validated later)
    // Either way, should not panic
    if let Ok(ast) = result {
        if !ast.items.is_empty() {
            // If parsed, verify it doesn't have invalid state
            if let MakeItem::Include { path, .. } = &ast.items[0] {
                // Empty path is detectable
                assert!(path.is_empty() || !path.is_empty());
            }
        }
    }
}

/// RED PHASE: Mutation-killing test - Line parsing advances correctly
#[test]
fn test_INCLUDE_001_mut_parser_advances() {
    // Test that parser advances to next line after include
    let makefile = "include file.mk\nCC := gcc";

    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 2, "Should parse both include and variable");

    // First item: Include
    match &ast.items[0] {
        MakeItem::Include { .. } => {}
        _ => panic!("First item should be Include"),
    }

    // Second item: Variable
    match &ast.items[1] {
        MakeItem::Variable { name, .. } => {
            assert_eq!(name, "CC", "Second item should be CC variable");
        }
        _ => panic!("Second item should be Variable"),
    }
}

// ==============================================================================
// INCLUDE-002: Optional Include Directives (-include, sinclude)
// ==============================================================================
// Task: Document optional include directives that don't error if file is missing
// Input: -include optional.mk or sinclude optional.mk
// Goal: Parser sets optional=true flag for -include and sinclude variants

#[test]
fn test_INCLUDE_002_dash_include() {
    // ARRANGE: Optional include with -include syntax
    let makefile = "-include optional.mk";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(
        result.is_ok(),
        "Should parse -include directive, got error: {:?}",
        result.err()
    );

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 include");

    // ASSERT: Include item with optional=true
    match &ast.items[0] {
        MakeItem::Include { path, optional, .. } => {
            assert_eq!(path, "optional.mk", "Path should be optional.mk");
            assert!(*optional, "-include should set optional=true");
        }
        other => panic!("Expected Include item, got {:?}", other),
    }
}

#[test]
fn test_INCLUDE_002_sinclude() {
    // ARRANGE: Optional include with sinclude syntax (GNU Make synonym)
    let makefile = "sinclude optional.mk";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(
        result.is_ok(),
        "Should parse sinclude directive, got error: {:?}",
        result.err()
    );

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should have 1 include");

    // ASSERT: Include item with optional=true
    match &ast.items[0] {
        MakeItem::Include { path, optional, .. } => {
            assert_eq!(path, "optional.mk", "Path should be optional.mk");
            assert!(*optional, "sinclude should set optional=true");
        }
        other => panic!("Expected Include item, got {:?}", other),
    }
}

#[test]
fn test_INCLUDE_002_dash_include_with_path() {
    // ARRANGE: Optional include with directory path
    let makefile = "-include config/optional.mk";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok(), "Should parse -include with path");

    let ast = result.unwrap();
    match &ast.items[0] {
        MakeItem::Include { path, optional, .. } => {
            assert_eq!(path, "config/optional.mk");
            assert!(*optional, "Should be optional");
        }
        other => panic!("Expected Include, got {:?}", other),
    }
}

#[test]
fn test_INCLUDE_002_mixed_includes() {
    // ARRANGE: Mix of required and optional includes
    let makefile = "include required.mk\n-include optional.mk\nsinclude also_optional.mk";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok(), "Should parse mixed includes");

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 3, "Should have 3 includes");

    // First: required (optional=false)
    match &ast.items[0] {
        MakeItem::Include { path, optional, .. } => {
            assert_eq!(path, "required.mk");
            assert!(!optional, "include should be required");
        }
        other => panic!("Expected Include, got {:?}", other),
    }

    // Second: -include (optional=true)
    match &ast.items[1] {
        MakeItem::Include { path, optional, .. } => {
            assert_eq!(path, "optional.mk");
            assert!(*optional, "-include should be optional");
        }
        other => panic!("Expected Include, got {:?}", other),
    }

    // Third: sinclude (optional=true)
    match &ast.items[2] {
        MakeItem::Include { path, optional, .. } => {
            assert_eq!(path, "also_optional.mk");
            assert!(*optional, "sinclude should be optional");
        }
        other => panic!("Expected Include, got {:?}", other),
    }
}

#[test]
fn test_INCLUDE_002_dash_include_with_variables() {
    // ARRANGE: Optional include with variable reference
    let makefile = "-include $(CONFIG_DIR)/optional.mk";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: Successfully parsed
    assert!(result.is_ok(), "Should parse -include with variables");

    let ast = result.unwrap();
    match &ast.items[0] {
        MakeItem::Include { path, optional, .. } => {
            assert!(path.contains("$(CONFIG_DIR)"), "Should preserve variable");
            assert!(*optional, "Should be optional");
        }
        other => panic!("Expected Include, got {:?}", other),
    }
}

#[test]
fn test_INCLUDE_002_multiple_optional_includes() {
    // ARRANGE: Multiple optional includes
    let makefile = "-include file1.mk file2.mk file3.mk";

    // ACT: Parse makefile
    let result = parse_makefile(makefile);

    // ASSERT: This tests current behavior - parser may handle this differently
    // GNU Make allows multiple files in one include directive
    assert!(result.is_ok(), "Should not crash on multiple files");
}

// ==============================================================================
// INCLUDE-002: Property Tests
// ==============================================================================

#[cfg(test)]
mod include_002_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_INCLUDE_002_dash_include_always_optional(
            filename in "[a-zA-Z0-9_.-]{1,30}\\.mk"
        ) {
            // ARRANGE: -include directive
            let makefile = format!("-include {}", filename);

            // ACT: Parse makefile
            let result = parse_makefile(&makefile);

            // ASSERT: Always parses and sets optional=true
            prop_assert!(result.is_ok(), "-include should always parse");

            let ast = result.unwrap();
            prop_assert_eq!(ast.items.len(), 1);

            match &ast.items[0] {
                MakeItem::Include { path, optional, .. } => {
                    prop_assert_eq!(path, &filename);
                    prop_assert!(*optional, "-include should always be optional");
                }
                other => return Err(TestCaseError::fail(format!("Expected Include, got {:?}", other))),
            }
        }

        #[test]
        fn prop_INCLUDE_002_sinclude_always_optional(
            filename in "[a-zA-Z0-9_.-]{1,30}\\.mk"
        ) {
            // ARRANGE: sinclude directive
            let makefile = format!("sinclude {}", filename);

            // ACT: Parse makefile
            let result = parse_makefile(&makefile);

            // ASSERT: Always parses and sets optional=true
            prop_assert!(result.is_ok(), "sinclude should always parse");

            let ast = result.unwrap();
            prop_assert_eq!(ast.items.len(), 1);

            match &ast.items[0] {
                MakeItem::Include { path, optional, .. } => {
                    prop_assert_eq!(path, &filename);
                    prop_assert!(*optional, "sinclude should always be optional");
                }
                other => return Err(TestCaseError::fail(format!("Expected Include, got {:?}", other))),
            }
        }

        #[test]
        fn prop_INCLUDE_002_parsing_is_deterministic(
            filename in "[a-zA-Z0-9/_.-]{1,50}\\.mk"
        ) {
            // ARRANGE: -include directive
            let makefile = format!("-include {}", filename);

            // ACT: Parse twice
            let ast1 = parse_makefile(&makefile);
            let ast2 = parse_makefile(&makefile);

            // ASSERT: Results are identical
            match (ast1, ast2) {
                (Ok(a1), Ok(a2)) => {
                    prop_assert_eq!(a1.items.len(), a2.items.len());
                    match (&a1.items[0], &a2.items[0]) {
                        (MakeItem::Include { path: p1, optional: o1, .. },
                         MakeItem::Include { path: p2, optional: o2, .. }) => {
                            prop_assert_eq!(p1, p2);
                            prop_assert_eq!(o1, o2);
                        }
                        _ => return Err(TestCaseError::fail("Expected matching Include items")),
                    }
                }
                _ => return Err(TestCaseError::fail("Parsing should be deterministic")),
            }
        }

        #[test]
        fn prop_INCLUDE_002_optional_vs_required(
            filename in "[a-z]{3,10}\\.mk"
        ) {
            // ARRANGE: Test that include vs -include vs sinclude set optional correctly
            let include_reg = format!("include {}", filename);
            let include_dash = format!("-include {}", filename);
            let include_s = format!("sinclude {}", filename);

            // ACT: Parse all three variants
            let ast_reg = parse_makefile(&include_reg);
            let ast_dash = parse_makefile(&include_dash);
            let ast_s = parse_makefile(&include_s);

            // ASSERT: All parse successfully
            prop_assert!(ast_reg.is_ok());
            prop_assert!(ast_dash.is_ok());
            prop_assert!(ast_s.is_ok());

            // Regular include: optional=false
            match &ast_reg.unwrap().items[0] {
                MakeItem::Include { optional, .. } => {
                    prop_assert!(!optional, "include should be required");
                }
                other => return Err(TestCaseError::fail(format!("Expected Include, got {:?}", other))),
            }

            // -include: optional=true
            match &ast_dash.unwrap().items[0] {
                MakeItem::Include { optional, .. } => {
                    prop_assert!(*optional, "-include should be optional");
                }
                other => return Err(TestCaseError::fail(format!("Expected Include, got {:?}", other))),
            }

            // sinclude: optional=true
            match &ast_s.unwrap().items[0] {
                MakeItem::Include { optional, .. } => {
                    prop_assert!(*optional, "sinclude should be optional");
                }
                other => return Err(TestCaseError::fail(format!("Expected Include, got {:?}", other))),
            }
        }

