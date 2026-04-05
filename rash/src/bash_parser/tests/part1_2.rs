#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]

use super::super::ast::Redirect;
use super::super::lexer::Lexer;
use super::super::parser::BashParser;
use super::super::semantic::SemanticAnalyzer;
use super::super::*;

#[test]
fn test_until_to_while_transformation() {
    use crate::bash_parser::ast::*;

    // INPUT: Until loop in bash
    // until [ $i -gt 5 ]; do echo $i; i=$((i+1)); done

    // Manually construct AST for until loop (parser doesn't support it yet)
    let until_condition = BashExpr::Test(Box::new(TestExpr::IntGt(
        BashExpr::Variable("i".to_string()),
        BashExpr::Literal("5".to_string()),
    )));

    let until_body = vec![
        BashStmt::Command {
            name: "echo".to_string(),
            args: vec![BashExpr::Variable("i".to_string())],
            redirects: vec![],
            span: Span::dummy(),
        },
        BashStmt::Assignment {
            name: "i".to_string(),
            index: None,
            value: BashExpr::Arithmetic(Box::new(ArithExpr::Add(
                Box::new(ArithExpr::Variable("i".to_string())),
                Box::new(ArithExpr::Number(1)),
            ))),
            exported: false,
            span: Span::dummy(),
        },
    ];

    // Create Until statement (this will fail - variant doesn't exist yet)
    let ast = BashAst {
        statements: vec![BashStmt::Until {
            condition: until_condition,
            body: until_body,
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

    // EXPECTED: Until loop transformed to while with negated condition
    // while [ ! "$i" -gt 5 ]; do printf '%s\n' "$i"; i=$((i+1)); done

    // ASSERT: Should contain "while" not "until"
    assert!(
        purified.contains("while"),
        "Until loop should be transformed to while, got: {}",
        purified
    );

    // ASSERT: Should contain negation "!"
    assert!(
        purified.contains("!"),
        "Until loop condition should be negated in while, got: {}",
        purified
    );

    // ASSERT: Should NOT contain "until"
    assert!(
        !purified.contains("until"),
        "Purified output should not contain 'until', got: {}",
        purified
    );

    // PROPERTY: Deterministic output
    let purified2 = generators::generate_purified_bash(&ast);
    assert_eq!(purified, purified2, "Purification must be deterministic");
}

// BASH MANUAL VALIDATION - Task EXP-GLOB-001: Glob Pattern Transformation
// EXTREME TDD RED Phase - This test MUST fail first

#[test]
fn test_glob_pattern_transformation() {
    use crate::bash_parser::ast::*;

    // INPUT: for loop with glob pattern
    // for f in *.txt; do echo $f; done

    // Manually construct AST with glob pattern in for loop
    let ast = BashAst {
        statements: vec![BashStmt::For {
            variable: "f".to_string(),
            items: BashExpr::Glob("*.txt".to_string()),
            body: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Variable("f".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }],
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

    // EXPECTED: Purified bash should preserve glob pattern
    // for f in *.txt; do echo "$f"; done

    // ASSERT: Should contain the glob pattern
    assert!(
        purified.contains("*.txt"),
        "Purified output should preserve glob pattern *.txt, got: {}",
        purified
    );

    // ASSERT: Should contain for loop structure
    assert!(
        purified.contains("for f in"),
        "Purified output should contain 'for f in', got: {}",
        purified
    );

    // ASSERT: Should contain do/done
    assert!(
        purified.contains("do") && purified.contains("done"),
        "Purified output should contain do/done, got: {}",
        purified
    );

    // ASSERT: Variable should be quoted in purified output
    assert!(
        purified.contains("\"$f\""),
        "Purified output should quote variable $f, got: {}",
        purified
    );

    // PROPERTY: Deterministic output
    let purified2 = generators::generate_purified_bash(&ast);
    assert_eq!(purified, purified2, "Purification must be deterministic");

    // TODO: Test Rust transpilation
    // Expected: for f in glob("*.txt") { println!("{}", f); }
}

// BASH MANUAL VALIDATION - Task EXP-PARAM-002: Assign Default Value Expansion
// EXTREME TDD RED Phase - This test MUST fail first

#[test]
fn test_assign_default_value_expansion() {
    use crate::bash_parser::ast::*;

    // INPUT: Parameter expansion with assign default
    // echo "${VAR:=default}"
    // If VAR is unset or null, assign "default" to VAR and use it

    // Manually construct AST with assign default expansion
    let assign_default_expr = BashExpr::AssignDefault {
        variable: "VAR".to_string(),
        default: Box::new(BashExpr::Literal("default".to_string())),
    };

    let ast = BashAst {
        statements: vec![BashStmt::Command {
            name: "echo".to_string(),
            args: vec![assign_default_expr],
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

    // EXPECTED: Purified bash should preserve ${VAR:=default} syntax
    // echo "${VAR:=default}"

    // ASSERT: Should contain parameter expansion syntax with :=
    assert!(
        purified.contains("$")
            && purified.contains("VAR")
            && purified.contains(":=")
            && purified.contains("default"),
        "Purified output should preserve ${{VAR:=default}} syntax, got: {}",
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
    // Expected: let val = var.get_or_insert("default");
    // or: if var.is_none() { var = Some("default"); }
}

// BASH MANUAL VALIDATION - Task EXP-PARAM-001: Default Value Expansion
// EXTREME TDD RED Phase - This test MUST fail first

#[test]
fn test_default_value_expansion() {
    use crate::bash_parser::ast::*;

    // INPUT: Parameter expansion with default value
    // echo "${VAR:-default}"
    // If VAR is unset or null, use "default"

    // Manually construct AST with default value expansion
    let default_value_expr = BashExpr::DefaultValue {
        variable: "VAR".to_string(),
        default: Box::new(BashExpr::Literal("default".to_string())),
    };

    let ast = BashAst {
        statements: vec![BashStmt::Command {
            name: "echo".to_string(),
            args: vec![default_value_expr],
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

    // EXPECTED: Purified bash should preserve ${VAR:-default} syntax
    // printf '%s\n' "${VAR:-default}"

    // ASSERT: Should contain parameter expansion syntax
    assert!(
        purified.contains("$")
            && purified.contains("VAR")
            && purified.contains(":-")
            && purified.contains("default"),
        "Purified output should preserve ${{VAR:-default}} syntax, got: {}",
        purified
    );

    // ASSERT: Should contain the command (echo in this case - printf transformation is separate)
    assert!(
        purified.contains("echo"),
        "Purified output should contain echo command, got: {}",
        purified
    );

    // PROPERTY: Deterministic output
    let purified2 = generators::generate_purified_bash(&ast);
    assert_eq!(purified, purified2, "Purification must be deterministic");

    // TODO: Test Rust transpilation
    // Expected: let val = var.unwrap_or("default");
}

// BASH MANUAL VALIDATION - Task EXP-PARAM-003: Error If Unset Expansion
// EXTREME TDD RED Phase - This test MUST fail first

#[test]
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

