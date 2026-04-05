#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]

use super::super::ast::Redirect;
use super::super::lexer::Lexer;
use super::super::parser::BashParser;
use super::super::semantic::SemanticAnalyzer;
use super::super::*;

#[test]
fn test_parse_and_analyze_simple_script() {
    let script = r#"
#!/bin/bash
FOO=bar
echo $FOO
"#;

    let mut parser = BashParser::new(script).unwrap();
    let ast = parser.parse().unwrap();

    assert!(!ast.statements.is_empty());

    let mut analyzer = SemanticAnalyzer::new();
    let report = analyzer.analyze(&ast).unwrap();

    assert!(report.scope_info.variables.contains_key("FOO"));
}

#[test]
fn test_parse_function_definition() {
    let script = r#"
function greet() {
    echo "Hello, World!"
}

greet
"#;

    let mut parser = BashParser::new(script).unwrap();
    let ast = parser.parse().unwrap();

    let has_function = ast
        .statements
        .iter()
        .any(|s| matches!(s, BashStmt::Function { .. }));

    assert!(has_function);
}

#[test]
fn test_parse_if_statement() {
    let script = r#"
if [ $x == 1 ]; then
    echo "one"
elif [ $x == 2 ]; then
    echo "two"
else
    echo "other"
fi
"#;

    let mut parser = BashParser::new(script).unwrap();
    let ast = parser.parse().unwrap();

    let has_if = ast
        .statements
        .iter()
        .any(|s| matches!(s, BashStmt::If { .. }));

    assert!(has_if);
}

#[test]
fn test_parse_for_loop() {
    let script = r#"
for file in *.txt; do
    echo $file
done
"#;

    let mut parser = BashParser::new(script).unwrap();
    let ast = parser.parse().unwrap();

    let has_for = ast
        .statements
        .iter()
        .any(|s| matches!(s, BashStmt::For { .. }));

    assert!(has_for);
}

#[test]
fn test_semantic_analysis_detects_exports() {
    let script = "export PATH=/usr/bin";

    let mut parser = BashParser::new(script).unwrap();
    let ast = parser.parse().unwrap();

    let mut analyzer = SemanticAnalyzer::new();
    let report = analyzer.analyze(&ast).unwrap();

    assert!(report.effects.env_modifications.contains("PATH"));
}

/// Test: Issue #4 - Phase 2 - Basic output redirection
/// Expected behavior: Parse "echo hello > output.txt" and populate redirects field
#[test]
fn test_parse_output_redirection() {
    let script = "echo hello > output.txt";

    let mut parser = BashParser::new(script).unwrap();
    let ast = parser.parse().unwrap();

    // Should have one command statement
    assert_eq!(ast.statements.len(), 1);

    // Get the command
    if let BashStmt::Command {
        name,
        args,
        redirects,
        ..
    } = &ast.statements[0]
    {
        // Verify command name
        assert_eq!(name, "echo");

        // Verify arguments
        assert_eq!(args.len(), 1, "Expected 1 arg, got {}", args.len());
        if let BashExpr::Literal(arg) = &args[0] {
            assert_eq!(arg, "hello");
        } else {
            panic!("Expected literal argument 'hello'");
        }

        // RED PHASE: This should fail - redirects should have one Output redirection
        assert_eq!(redirects.len(), 1, "Expected one redirection");

        if let Redirect::Output { target } = &redirects[0] {
            if let BashExpr::Literal(filename) = target {
                assert_eq!(filename, "output.txt");
            } else {
                panic!("Expected literal filename 'output.txt'");
            }
        } else {
            panic!("Expected Output redirection variant");
        }
    } else {
        panic!("Expected Command statement");
    }
}

/// Test: Issue #4 - Phase 3 RED - Append redirection
/// Expected behavior: Parse "echo hello >> output.txt" and populate redirects with Append variant
#[test]
fn test_parse_append_redirection() {
    let script = "echo hello >> output.txt";

    let mut parser = BashParser::new(script).unwrap();
    let ast = parser.parse().unwrap();

    // Should have one command statement
    assert_eq!(ast.statements.len(), 1);

    // Get the command
    if let BashStmt::Command {
        name,
        args,
        redirects,
        ..
    } = &ast.statements[0]
    {
        // Verify command name
        assert_eq!(name, "echo");

        // Verify arguments
        assert_eq!(args.len(), 1, "Expected 1 arg, got {}", args.len());
        if let BashExpr::Literal(arg) = &args[0] {
            assert_eq!(arg, "hello");
        } else {
            panic!("Expected literal argument 'hello'");
        }

        // RED PHASE: This should fail - redirects should have one Append redirection
        assert_eq!(redirects.len(), 1, "Expected one redirection");

        if let Redirect::Append { target } = &redirects[0] {
            if let BashExpr::Literal(filename) = target {
                assert_eq!(filename, "output.txt");
            } else {
                panic!("Expected literal filename 'output.txt'");
            }
        } else {
            panic!(
                "Expected Append redirection variant, got {:?}",
                redirects[0]
            );
        }
    } else {
        panic!("Expected Command statement");
    }
}

/// Test: Issue #4 - Phase 4 RED - Input redirection
/// Expected behavior: Parse "cat < input.txt" and populate redirects with Input variant
#[test]
fn test_parse_input_redirection() {
    let script = "cat < input.txt";

    let mut parser = BashParser::new(script).unwrap();
    let ast = parser.parse().unwrap();

    // Should have one command statement
    assert_eq!(ast.statements.len(), 1);

    // Get the command
    if let BashStmt::Command {
        name,
        args,
        redirects,
        ..
    } = &ast.statements[0]
    {
        // Verify command name
        assert_eq!(name, "cat");

        // Verify no arguments (just the redirection)
        assert_eq!(args.len(), 0, "Expected 0 args, got {}", args.len());

        // RED PHASE: This should fail - redirects should have one Input redirection
        assert_eq!(redirects.len(), 1, "Expected one redirection");

        if let Redirect::Input { target } = &redirects[0] {
            if let BashExpr::Literal(filename) = target {
                assert_eq!(filename, "input.txt");
            } else {
                panic!("Expected literal filename 'input.txt'");
            }
        } else {
            panic!("Expected Input redirection variant, got {:?}", redirects[0]);
        }
    } else {
        panic!("Expected Command statement");
    }
}

/// Test: Issue #4 - Phase 5 RED - Error redirection (2>)
/// Expected behavior: Parse "echo hello 2> error.log" and populate redirects with Error variant
#[test]
fn test_parse_error_redirection() {
    let script = "echo hello 2> error.log";

    let mut parser = BashParser::new(script).unwrap();
    let ast = parser.parse().unwrap();

    // Should have one command statement
    assert_eq!(ast.statements.len(), 1);

    // Get the command
    if let BashStmt::Command {
        name,
        args,
        redirects,
        ..
    } = &ast.statements[0]
    {
        // Verify command name
        assert_eq!(name, "echo");

        // Verify one argument: "hello"
        assert_eq!(args.len(), 1, "Expected 1 arg, got {}", args.len());
        if let BashExpr::Literal(arg) = &args[0] {
            assert_eq!(arg, "hello");
        } else {
            panic!("Expected literal argument 'hello'");
        }

        // RED PHASE: This should fail - redirects should have one Error redirection
        assert_eq!(redirects.len(), 1, "Expected one redirection");

        if let Redirect::Error { target } = &redirects[0] {
            if let BashExpr::Literal(filename) = target {
                assert_eq!(filename, "error.log");
            } else {
                panic!("Expected literal filename 'error.log'");
            }
        } else {
            panic!("Expected Error redirection variant, got {:?}", redirects[0]);
        }
    } else {
        panic!("Expected Command statement");
    }
}

/// Test: Issue #4 - Phase 6 RED - Append error redirection (2>>)
/// Expected behavior: Parse "echo hello 2>> error.log" and populate redirects with AppendError variant
#[test]
fn test_parse_append_error_redirection() {
    let script = "echo hello 2>> error.log";

    let mut parser = BashParser::new(script).unwrap();
    let ast = parser.parse().unwrap();

    // Should have one command statement
    assert_eq!(ast.statements.len(), 1);

    // Get the command
    if let BashStmt::Command {
        name,
        args,
        redirects,
        ..
    } = &ast.statements[0]
    {
        // Verify command name
        assert_eq!(name, "echo");

        // Verify one argument: "hello"
        assert_eq!(args.len(), 1, "Expected 1 arg, got {}", args.len());
        if let BashExpr::Literal(arg) = &args[0] {
            assert_eq!(arg, "hello");
        } else {
            panic!("Expected literal argument 'hello'");
        }

        // RED PHASE: This should fail - redirects should have one AppendError redirection
        assert_eq!(redirects.len(), 1, "Expected one redirection");

        if let Redirect::AppendError { target } = &redirects[0] {
            if let BashExpr::Literal(filename) = target {
                assert_eq!(filename, "error.log");
            } else {
                panic!("Expected literal filename 'error.log'");
            }
        } else {
            panic!(
                "Expected AppendError redirection variant, got {:?}",
                redirects[0]
            );
        }
    } else {
        panic!("Expected Command statement");
    }
}

/// Test: Issue #4 - Phase 7 RED - Combined redirection (&>)
/// Expected behavior: Parse "echo hello &> output.log" and populate redirects with Combined variant
#[test]
fn test_parse_combined_redirection() {
    let script = "echo hello &> output.log";

    let mut parser = BashParser::new(script).unwrap();
    let ast = parser.parse().unwrap();

    // Should have one command statement
    assert_eq!(ast.statements.len(), 1);

    // Get the command
    if let BashStmt::Command {
        name,
        args,
        redirects,
        ..
    } = &ast.statements[0]
    {
        // Verify command name
        assert_eq!(name, "echo");

        // Verify one argument: "hello"
        assert_eq!(args.len(), 1, "Expected 1 arg, got {}", args.len());
        if let BashExpr::Literal(arg) = &args[0] {
            assert_eq!(arg, "hello");
        } else {
            panic!("Expected literal argument 'hello'");
        }

        // RED PHASE: This should fail - redirects should have one Combined redirection
        assert_eq!(redirects.len(), 1, "Expected one redirection");

        if let Redirect::Combined { target } = &redirects[0] {
            if let BashExpr::Literal(filename) = target {
                assert_eq!(filename, "output.log");
            } else {
                panic!("Expected literal filename 'output.log'");
            }
        } else {
            panic!(
                "Expected Combined redirection variant, got {:?}",
                redirects[0]
            );
        }
    } else {
        panic!("Expected Command statement");
    }
}

include!("part1_incl2.rs");
