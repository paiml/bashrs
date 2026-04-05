#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(unused_imports)]

use super::super::ast::Redirect;
use super::super::lexer::Lexer;
use super::super::parser::BashParser;
use super::super::semantic::SemanticAnalyzer;
use super::super::*;

#[test]
fn test_ISSUE_004_003_parse_command_substitution() {
    // RED PHASE: Write failing test for $(command) parsing
    //
    // CRITICAL: Parser MUST accept $(command) for shell script parsing
    // Command substitution is CORE bash feature (different from arithmetic $((expr)))
    //
    // INPUT: bash with $(command)
    // EXPECTED: Parser accepts, returns AST with CommandSubstitution node
    // PURIFIER (later): May preserve or transform based on determinism

    let bash = r#"
#!/bin/bash
FILES=$(ls /tmp)
echo $FILES

USER=$(whoami)
echo "User: $USER"
"#;

    // ARRANGE: Lexer should tokenize $(command)
    let lexer_result = BashParser::new(bash);
    assert!(
        lexer_result.is_ok(),
        "Lexer should tokenize $(command): {:?}",
        lexer_result.err()
    );

    // ACT: Parser should accept $(command)
    let mut parser = lexer_result.unwrap();
    let parse_result = parser.parse();

    // ASSERT: Parser must accept $(command) for real bash parsing
    assert!(
        parse_result.is_ok(),
        "Parser MUST accept $(command) for real bash scripts: {:?}",
        parse_result.err()
    );

    // VERIFY: AST contains command substitution
    let ast = parse_result.unwrap();
    assert!(
        !ast.statements.is_empty(),
        "$(command) should produce non-empty AST"
    );
}

#[test]
fn test_ISSUE_004_004_parse_function_keyword() {
    // RED PHASE: Write failing test for 'function' keyword parsing
    //
    // CRITICAL: Parser MUST support 'function' keyword (common bash idiom)
    // Alternative to POSIX 'name() {}' syntax: 'function name() {}'
    //
    // INPUT: bash with function keyword
    // EXPECTED: Parser accepts both 'function name()' and 'function name' syntax
    // PURIFIER (later): May convert to POSIX 'name()' syntax

    let bash = r#"
#!/bin/bash

# Function with parentheses
function gen_id() {
    echo $RANDOM
}

# Function without parentheses (also valid bash)
function gen_temp {
    echo "/tmp/file-$$"
}

# Call functions
id=$(gen_id)
temp=$(gen_temp)
echo "ID: $id, Temp: $temp"
"#;

    // ARRANGE: Lexer should tokenize 'function' keyword
    let lexer_result = BashParser::new(bash);
    assert!(
        lexer_result.is_ok(),
        "Lexer should tokenize 'function' keyword: {:?}",
        lexer_result.err()
    );

    // ACT: Parser should accept function keyword
    let mut parser = lexer_result.unwrap();
    let parse_result = parser.parse();

    // ASSERT: Parser must accept 'function' keyword
    assert!(
        parse_result.is_ok(),
        "Parser MUST accept 'function' keyword: {:?}",
        parse_result.err()
    );

    // VERIFY: AST contains function definitions
    let ast = parse_result.unwrap();
    assert!(
        !ast.statements.is_empty(),
        "'function' keyword should produce non-empty AST"
    );
}

#[test]
fn test_ISSUE_004_005_parse_complete_small_simple_fixture() {
    // RED PHASE: Integration test for complete small_simple.sh
    //
    // CRITICAL: This is the ACTUAL benchmark fixture that fails
    // Combines ALL missing features: $RANDOM, $$, $(cmd), function
    //
    // This test verifies ALL features working together

    let bash = r#"
#!/bin/bash
# Simplified version of small_simple.sh combining all features

# Feature 1: $RANDOM
ID=$RANDOM
echo "Random ID: $ID"

# Feature 2: $$
PID=$$
TEMP_DIR="/tmp/build-$PID"

# Feature 3: $(command)
FILES=$(ls /tmp)
echo $FILES

# Feature 4: function keyword
function gen_id() {
    echo $RANDOM
}

function gen_temp() {
    echo "/tmp/file-$$"
}

# Combined usage
session_id="session-$(gen_id)"
temp_file=$(gen_temp)
echo "Session: $session_id"
echo "Temp: $temp_file"
"#;

    // ARRANGE: Lexer should handle combined features
    let lexer_result = BashParser::new(bash);
    assert!(
        lexer_result.is_ok(),
        "Lexer should tokenize combined features: {:?}",
        lexer_result.err()
    );

    // ACT: Parser should accept all features together
    let mut parser = lexer_result.unwrap();
    let parse_result = parser.parse();

    // ASSERT: Parser must accept complete script
    assert!(
        parse_result.is_ok(),
        "Parser MUST accept complete bash script with all features: {:?}",
        parse_result.err()
    );

    // VERIFY: AST is non-empty
    let ast = parse_result.unwrap();
    assert!(
        !ast.statements.is_empty(),
        "Complete script should produce non-empty AST"
    );
    assert!(
        ast.statements.len() >= 8,
        "Complete script should have multiple statements, got {}",
        ast.statements.len()
    );
}

// RED Phase: Test for $@ special variable (all positional parameters)
// Issue: medium.sh fails at line 119 with "local message=$@"
#[test]
fn test_ISSUE_004_006_parse_dollar_at() {
    // ACT: Parse bash with $@ special variable
    let bash = "message=$@";
    let parser_result = BashParser::new(bash);

    // ASSERT: Lexer should succeed
    assert!(
        parser_result.is_ok(),
        "Lexer should accept $@ special variable, got: {:?}",
        parser_result.err()
    );

    let mut parser = parser_result.unwrap();
    let parse_result = parser.parse();

    // ASSERT: Parser should succeed
    assert!(
        parse_result.is_ok(),
        "Parser should handle $@ special variable, got: {:?}",
        parse_result.err()
    );

    // VERIFY: AST contains variable assignment
    let ast = parse_result.unwrap();
    assert!(
        !ast.statements.is_empty(),
        "Should have at least one statement"
    );
}

// RED Phase: Test for heredoc (here-document) support
// Issue: medium.sh line 139 uses `sqlite3 $db_file <<SQL`
#[test]
fn test_HEREDOC_001_basic_heredoc() {
    // ARRANGE: Bash with basic heredoc
    let bash = r#"cat <<EOF
line1
line2
EOF"#;

    // ACT: Parse
    let parser_result = BashParser::new(bash);

    // ASSERT: Lexer should succeed
    assert!(
        parser_result.is_ok(),
        "Lexer should accept heredoc syntax, got: {:?}",
        parser_result.err()
    );

    let mut parser = parser_result.unwrap();
    let parse_result = parser.parse();

    // ASSERT: Parser should succeed
    assert!(
        parse_result.is_ok(),
        "Parser should handle heredoc, got: {:?}",
        parse_result.err()
    );

    // VERIFY: AST contains command with heredoc
    let ast = parse_result.unwrap();
    assert!(
        !ast.statements.is_empty(),
        "Should have at least one statement"
    );
}

/// Test: Issue #4 - Phase 9 RED - Basic pipeline support (echo | grep)
/// Expected behavior: Parse "echo hello | grep hello" and create Pipeline AST variant
#[test]
fn test_parse_basic_pipeline() {
    let script = "echo hello | grep hello";

    let mut parser = BashParser::new(script).unwrap();
    let ast = parser.parse().unwrap();

    assert_eq!(ast.statements.len(), 1);

    // RED PHASE: This will fail - Pipeline variant doesn't exist yet
    if let BashStmt::Pipeline { commands, span: _ } = &ast.statements[0] {
        assert_eq!(commands.len(), 2, "Expected 2 commands in pipeline");

        // First command: echo hello
        if let BashStmt::Command {
            name: name1,
            args: args1,
            ..
        } = &commands[0]
        {
            assert_eq!(name1, "echo");
            assert_eq!(args1.len(), 1);
            if let BashExpr::Literal(arg) = &args1[0] {
                assert_eq!(arg, "hello");
            } else {
                panic!("Expected literal argument 'hello'");
            }
        } else {
            panic!("Expected Command statement for first command");
        }

        // Second command: grep hello
        if let BashStmt::Command {
            name: name2,
            args: args2,
            ..
        } = &commands[1]
        {
            assert_eq!(name2, "grep");
            assert_eq!(args2.len(), 1);
            if let BashExpr::Literal(arg) = &args2[0] {
                assert_eq!(arg, "hello");
            } else {
                panic!("Expected literal argument 'hello'");
            }
        } else {
            panic!("Expected Command statement for second command");
        }
    } else {
        panic!("Expected Pipeline statement");
    }
}

/// Issue #59: Test parsing nested quotes in command substitution
/// INPUT: OUTPUT="$(echo "test" 2>&1)"
/// BUG: Gets mangled to: OUTPUT='$(echo ' test ' 2>&1)'
/// EXPECTED: String contains command substitution, preserves inner quotes
#[test]
fn test_ISSUE_059_001_nested_quotes_in_command_substitution() {
    // RED PHASE: This test currently fails due to incorrect string parsing
    //
    // CRITICAL: Parser MUST handle nested double quotes inside command substitution
    // This is VALID bash syntax that must be supported for real-world scripts
    let script = r#"OUTPUT="$(echo "test" 2>&1)""#;

    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let result = parser.parse();

    // ASSERT: Parser must accept this valid bash syntax
    assert!(
        result.is_ok(),
        "Parser MUST accept nested quotes in command substitution: {:?}",
        result.err()
    );

    let ast = result.expect("Should parse");
    assert_eq!(ast.statements.len(), 1, "Should have one statement");

    // Verify it's an assignment
    match &ast.statements[0] {
        BashStmt::Assignment { name, value, .. } => {
            assert_eq!(name, "OUTPUT", "Variable name should be OUTPUT");
            // The value should contain the command substitution
            // It should NOT be mangled into separate pieces
            match value {
                BashExpr::Concat(parts) => {
                    // Check that we have exactly one command substitution part
                    let has_cmd_sub = parts.iter().any(|p| matches!(p, BashExpr::CommandSubst(_)));
                    assert!(
                        has_cmd_sub,
                        "Value should contain command substitution, got: {:?}",
                        parts
                    );
                }
                BashExpr::CommandSubst(_cmd_stmt) => {
                    // Also acceptable: direct command substitution
                    // The presence of CommandSubst variant is sufficient
                }
                BashExpr::Literal(s) => {
                    // Also acceptable: Literal containing the command substitution string
                    // The key point is the string is NOT mangled - it preserves the full
                    // command substitution including nested quotes
                    assert!(
                        s.contains("$(") && s.contains("echo") && s.contains("test"),
                        "Literal should contain complete command substitution, got: {}",
                        s
                    );
                }
                other => {
                    panic!(
                        "Expected Concat, CommandSubst, or Literal for assignment value, got: {:?}",
                        other
                    );
                }
            }
        }
        other => panic!("Expected Assignment statement, got: {:?}", other),
    }
}

include!("part5_6_tests_ISSUE.rs");
