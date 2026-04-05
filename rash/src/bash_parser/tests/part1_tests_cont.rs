fn test_error_if_unset_expansion() {
    use crate::bash_parser::ast::*;

    // INPUT: Parameter expansion with error if unset
    // echo "${VAR:?Variable VAR is required}"
    // If VAR is unset or null, exit with error message

    // Manually construct AST with error-if-unset expansion
    let error_if_unset_expr = BashExpr::ErrorIfUnset {
        variable: "VAR".to_string(),
        message: Box::new(BashExpr::Literal("Variable VAR is required".to_string())),
    };

    let ast = BashAst {
        statements: vec![BashStmt::Command {
            name: "echo".to_string(),
            args: vec![error_if_unset_expr],
            redirects: vec![],
            span: Span::dummy(),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };

    // Generate purified bash
    let purified = generators::generate_purified_bash(&ast);

    // EXPECTED: Purified bash should preserve ${VAR:?message} syntax
    // echo "${VAR:?Variable VAR is required}"

    // ASSERT: Should contain parameter expansion syntax with :?
    assert!(
        purified.contains("$") && purified.contains("VAR") && purified.contains(":?"),
        "Purified output should preserve ${{VAR:?message}} syntax, got: {}",
        purified
    );

    // ASSERT: Should contain error message
    assert!(
        purified.contains("Variable VAR is required") || purified.contains("required"),
        "Purified output should contain error message, got: {}",
        purified
    );

    // ASSERT: Should contain the command
    assert!(
        purified.contains("echo"),
        "Purified output should contain echo command, got: {}",
        purified
    );

    // PROPERTY: Deterministic output
    let purified2 = generators::generate_purified_bash(&ast);
    assert_eq!(purified, purified2, "Purification must be deterministic");

    // TODO: Test Rust transpilation
    // Expected: let val = var.expect("Variable VAR is required");
}

// BASH MANUAL VALIDATION - Task EXP-PARAM-004: Alternative Value Expansion
// EXTREME TDD RED Phase - This test MUST fail first

#[test]
fn test_alternative_value_expansion() {
    use crate::bash_parser::ast::*;

    // INPUT: Parameter expansion with alternative value
    // echo "${VAR:+is_set}"
    // If VAR is set and non-null, use "is_set", otherwise empty string

    // Manually construct AST with alternative value expansion
    let alternative_value_expr = BashExpr::AlternativeValue {
        variable: "VAR".to_string(),
        alternative: Box::new(BashExpr::Literal("is_set".to_string())),
    };

    let ast = BashAst {
        statements: vec![BashStmt::Command {
            name: "echo".to_string(),
            args: vec![alternative_value_expr],
            redirects: vec![],
            span: Span::dummy(),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };

    // Generate purified bash
    let purified = generators::generate_purified_bash(&ast);

    // EXPECTED: Purified bash should preserve ${VAR:+is_set} syntax
    // echo "${VAR:+is_set}"

    // ASSERT: Should contain parameter expansion syntax with :+
    assert!(
        purified.contains("$") && purified.contains("VAR") && purified.contains(":+"),
        "Purified output should preserve ${{VAR:+alternative}} syntax, got: {}",
        purified
    );

    // ASSERT: Should contain alternative value
    assert!(
        purified.contains("is_set"),
        "Purified output should contain alternative value, got: {}",
        purified
    );

    // ASSERT: Should contain the command
    assert!(
        purified.contains("echo"),
        "Purified output should contain echo command, got: {}",
        purified
    );

    // PROPERTY: Deterministic output
    let purified2 = generators::generate_purified_bash(&ast);
    assert_eq!(purified, purified2, "Purification must be deterministic");

    // TODO: Test Rust transpilation
    // Expected: let val = if var.is_some() { "is_set" } else { "" };
    // or: var.map(|_| "is_set").unwrap_or("")
}

// BASH MANUAL VALIDATION - Task EXP-PARAM-005: String Length Expansion
// EXTREME TDD RED Phase - This test MUST fail first

#[test]
fn test_string_length_expansion() {
    use crate::bash_parser::ast::*;

    // INPUT: Parameter expansion with string length
    // echo "${#VAR}"
    // Get the length of the string value of VAR

    // Manually construct AST with string length expansion
    let string_length_expr = BashExpr::StringLength {
        variable: "VAR".to_string(),
    };

    let ast = BashAst {
        statements: vec![BashStmt::Command {
            name: "echo".to_string(),
            args: vec![string_length_expr],
            redirects: vec![],
            span: Span::dummy(),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };

    // Generate purified bash
    let purified = generators::generate_purified_bash(&ast);

    // EXPECTED: Purified bash should preserve ${#VAR} syntax
    // echo "${#VAR}"

    // ASSERT: Should contain parameter expansion syntax with #
    assert!(
        purified.contains("$") && purified.contains("#") && purified.contains("VAR"),
        "Purified output should preserve ${{#VAR}} syntax, got: {}",
        purified
    );

    // ASSERT: Should contain the command
    assert!(
        purified.contains("echo"),
        "Purified output should contain echo command, got: {}",
        purified
    );

    // PROPERTY: Deterministic output
    let purified2 = generators::generate_purified_bash(&ast);
    assert_eq!(purified, purified2, "Purification must be deterministic");

    // TODO: Test Rust transpilation
    // Expected: let len = var.len();
}

// BASH MANUAL VALIDATION - Task EXP-PARAM-006: Remove Suffix Expansion
// EXTREME TDD RED Phase - This test MUST fail first

#[test]
fn test_remove_suffix_expansion() {
    use crate::bash_parser::ast::*;

    // INPUT: Parameter expansion with suffix removal
    // file="test.txt"; echo "${file%.txt}"
    // Remove shortest matching suffix pattern from variable

    // Manually construct AST with remove suffix expansion
    let remove_suffix_expr = BashExpr::RemoveSuffix {
        variable: "file".to_string(),
        pattern: Box::new(BashExpr::Literal(".txt".to_string())),
    };

    let ast = BashAst {
        statements: vec![BashStmt::Command {
            name: "echo".to_string(),
            args: vec![remove_suffix_expr],
            redirects: vec![],
            span: Span::dummy(),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };

    // Generate purified bash
    let purified = generators::generate_purified_bash(&ast);

    // EXPECTED: Purified bash should preserve ${file%.txt} syntax
    // echo "${file%.txt}"

    // ASSERT: Should contain parameter expansion syntax with %
    assert!(
        purified.contains("$") && purified.contains("file") && purified.contains("%"),
        "Purified output should preserve ${{file%.txt}} syntax, got: {}",
        purified
    );

    // ASSERT: Should contain pattern
    assert!(
        purified.contains(".txt") || purified.contains("txt"),
        "Purified output should contain pattern, got: {}",
        purified
    );

    // ASSERT: Should contain the command
    assert!(
        purified.contains("echo"),
        "Purified output should contain echo command, got: {}",
        purified
    );

    // PROPERTY: Deterministic output
    let purified2 = generators::generate_purified_bash(&ast);
    assert_eq!(purified, purified2, "Purification must be deterministic");

    // TODO: Test Rust transpilation
    // Expected: let name = file.strip_suffix(".txt").unwrap_or(&file);
}

// BASH MANUAL VALIDATION - Task EXP-PARAM-007: Remove Prefix Expansion
// EXTREME TDD RED Phase - This test MUST fail first

#[test]
fn test_remove_prefix_expansion() {
    use crate::bash_parser::ast::*;

    // INPUT: Parameter expansion with prefix removal
    // path="/usr/local/bin"; echo "${path#/usr/}"
    // Remove shortest matching prefix pattern from variable

    // Manually construct AST with remove prefix expansion
    let remove_prefix_expr = BashExpr::RemovePrefix {
        variable: "path".to_string(),
        pattern: Box::new(BashExpr::Literal("/usr/".to_string())),
    };

    let ast = BashAst {
        statements: vec![BashStmt::Command {
            name: "echo".to_string(),
            args: vec![remove_prefix_expr],
            redirects: vec![],
            span: Span::dummy(),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };

    // Generate purified bash
    let purified = generators::generate_purified_bash(&ast);

    // EXPECTED: Purified bash should preserve ${path#/usr/} syntax
    // echo "${path#/usr/}"

    // ASSERT: Should contain parameter expansion syntax with #
    assert!(
        purified.contains("$") && purified.contains("path") && purified.contains("#"),
        "Purified output should preserve ${{path#/usr/}} syntax, got: {}",
        purified
    );

    // ASSERT: Should contain pattern
    assert!(
        purified.contains("/usr/") || purified.contains("usr"),
        "Purified output should contain pattern, got: {}",
        purified
    );

    // ASSERT: Should contain the command
    assert!(
        purified.contains("echo"),
        "Purified output should contain echo command, got: {}",
        purified
    );

    // PROPERTY: Deterministic output
    let purified2 = generators::generate_purified_bash(&ast);
    assert_eq!(purified, purified2, "Purification must be deterministic");

    // TODO: Test Rust transpilation
    // Expected: let name = path.strip_prefix("/usr/").unwrap_or(&path);
}

// BASH MANUAL VALIDATION - Task EXP-PARAM-008: Remove Longest Prefix Expansion
// EXTREME TDD RED Phase - This test MUST fail first

#[test]
fn test_remove_longest_prefix_expansion() {
    use crate::bash_parser::ast::*;

    // INPUT: Parameter expansion with longest prefix removal (greedy)
    // path="/usr/local/bin"; echo "${path##*/}"
    // Remove longest matching prefix pattern from variable
    // ${path##*/} removes everything up to the last / - gets just "bin"

    // Manually construct AST with remove longest prefix expansion
    let remove_longest_prefix_expr = BashExpr::RemoveLongestPrefix {
        variable: "path".to_string(),
        pattern: Box::new(BashExpr::Literal("*/".to_string())),
    };

    let ast = BashAst {
        statements: vec![BashStmt::Command {
            name: "echo".to_string(),
            args: vec![remove_longest_prefix_expr],
            redirects: vec![],
            span: Span::dummy(),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };

    // Generate purified bash
    let purified = generators::generate_purified_bash(&ast);

    // EXPECTED: Purified bash should preserve ${path##*/} syntax
    // echo "${path##*/}"

    // ASSERT: Should contain parameter expansion syntax with ##
    assert!(
        purified.contains("$") && purified.contains("path") && purified.contains("##"),
        "Purified output should preserve ${{path##*/}} syntax, got: {}",
        purified
    );

    // ASSERT: Should contain pattern
    assert!(
        purified.contains("*/") || purified.contains("*"),
        "Purified output should contain pattern, got: {}",
        purified
    );

    // ASSERT: Should contain the command
    assert!(
        purified.contains("echo"),
        "Purified output should contain echo command, got: {}",
        purified
    );

    // PROPERTY: Deterministic output
    let purified2 = generators::generate_purified_bash(&ast);
    assert_eq!(purified, purified2, "Purification must be deterministic");

    // TODO: Test Rust transpilation
    // Expected: let name = path.rsplit_once('/').map_or(&path, |(_, name)| name);
}

// BASH MANUAL VALIDATION - Task EXP-PARAM-009: Remove Longest Suffix Expansion
// EXTREME TDD RED Phase - This test MUST fail first

#[test]
fn test_remove_longest_suffix_expansion() {
    use crate::bash_parser::ast::*;

    // INPUT: Parameter expansion with longest suffix removal (greedy)
    // file="archive.tar.gz"; echo "${file%%.*}"
    // Remove longest matching suffix pattern from variable
    // ${file%%.*} removes everything from the first . - gets just "archive"

    // Manually construct AST with remove longest suffix expansion
    let remove_longest_suffix_expr = BashExpr::RemoveLongestSuffix {
        variable: "file".to_string(),
        pattern: Box::new(BashExpr::Literal(".*".to_string())),
    };

    let ast = BashAst {
        statements: vec![BashStmt::Command {
            name: "echo".to_string(),
            args: vec![remove_longest_suffix_expr],
            redirects: vec![],
            span: Span::dummy(),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };

    // Generate purified bash
    let purified = generators::generate_purified_bash(&ast);

    // EXPECTED: Purified bash should preserve ${file%%.*} syntax
    // echo "${file%%.*}"

    // ASSERT: Should contain parameter expansion syntax with %%
    assert!(
        purified.contains("$") && purified.contains("file") && purified.contains("%%"),
        "Purified output should preserve ${{file%%.*}} syntax, got: {}",
        purified
    );

    // ASSERT: Should contain pattern
    assert!(
        purified.contains(".*") || purified.contains("*"),
        "Purified output should contain pattern, got: {}",
        purified
    );

    // ASSERT: Should contain the command
    assert!(
        purified.contains("echo"),
        "Purified output should contain echo command, got: {}",
        purified
    );

    // PROPERTY: Deterministic output
    let purified2 = generators::generate_purified_bash(&ast);
    assert_eq!(purified, purified2, "Purification must be deterministic");

    // TODO: Test Rust transpilation
    // Expected: let name = file.split_once('.').map_or(&file, |(name, _)| name);
}

// PROPERTY TESTING: Until Loop Transformation
// Verify until→while transformation properties hold across all valid inputs

#[cfg(test)]
#[path = "part1_tests_extracted.rs"]
mod tests_extracted;
