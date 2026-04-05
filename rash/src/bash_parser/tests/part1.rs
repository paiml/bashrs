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

/// Test: Issue #4 - Phase 8 RED - File descriptor duplication (2>&1)
/// Expected behavior: Parse "echo hello 2>&1" and populate redirects with Duplicate variant
#[test]
fn test_parse_fd_duplication() {
    let script = "echo hello 2>&1";

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

        // RED PHASE: This should fail - redirects should have one Duplicate redirection
        assert_eq!(redirects.len(), 1, "Expected one redirection");

        if let Redirect::Duplicate { from_fd, to_fd } = &redirects[0] {
            assert_eq!(*from_fd, 2, "Expected from_fd=2 (stderr)");
            assert_eq!(*to_fd, 1, "Expected to_fd=1 (stdout)");
        } else {
            panic!(
                "Expected Duplicate redirection variant, got {:?}",
                redirects[0]
            );
        }
    } else {
        panic!("Expected Command statement");
    }
}

#[test]
fn test_semantic_analysis_detects_file_operations() {
    let script = "cat /etc/passwd";

    let mut parser = BashParser::new(script).unwrap();
    let ast = parser.parse().unwrap();

    let mut analyzer = SemanticAnalyzer::new();
    let report = analyzer.analyze(&ast).unwrap();

    assert!(!report.effects.file_reads.is_empty());
}

// BASH MANUAL VALIDATION - Task 1.1: Shebang Transformation
// EXTREME TDD RED Phase - This test MUST fail first

#[test]
fn test_shebang_transformation() {
    // INPUT: Bash script with bash shebang
    let bash_script = "#!/bin/bash\necho 'Hello'";

    // Parse bash
    let mut parser = BashParser::new(bash_script).unwrap();
    let ast = parser.parse().unwrap();

    // Generate purified bash
    let purified = generators::generate_purified_bash(&ast);

    // ASSERT: Shebang should be transformed to POSIX sh
    assert!(
        purified.starts_with("#!/bin/sh"),
        "Purified bash must use POSIX sh shebang, got: {}",
        purified.lines().next().unwrap_or("")
    );

    // PROPERTY: Purified output must be deterministic
    let purified2 = generators::generate_purified_bash(&ast);
    assert_eq!(purified, purified2, "Purification must be deterministic");
}

// BASH MANUAL VALIDATION - Task LOOP-001: Until Loop Transformation
// EXTREME TDD RED Phase - This test MUST fail first

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
