#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]

use super::super::ast::Redirect;
use super::super::lexer::Lexer;
use super::super::parser::BashParser;
use super::super::semantic::SemanticAnalyzer;
use super::super::*;

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

/// Issue #59: Test parsing || true after command substitution
/// INPUT: OUTPUT="$(echo "test" 2>&1)" || true
/// BUG: Fails with "Invalid syntax: Expected expression"
/// EXPECTED: Parses as OrList with assignment and 'true' command
#[test]
fn test_ISSUE_059_002_or_true_after_command_substitution() {
    // RED PHASE: This test currently fails because || is not handled after assignment
    //
    // CRITICAL: Parser MUST handle || (logical OR) after command substitution
    // This pattern is EXTREMELY common in real bash scripts for error handling
    let script = r#"OUTPUT="$(echo "test" 2>&1)" || true"#;

    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let result = parser.parse();

    // ASSERT: Parser must accept || after command substitution
    assert!(
        result.is_ok(),
        "Parser MUST accept '|| true' after command substitution: {:?}",
        result.err()
    );

    let ast = result.expect("Should parse");
    assert!(
        !ast.statements.is_empty(),
        "Should have at least one statement"
    );

    // The statement should be some kind of logical OR construct
    // Either as a dedicated OrList variant or as a wrapper
    // The exact structure depends on how we choose to implement it
}

/// Issue #59: Test simpler case - || true after simple command
/// This helps isolate whether the bug is in || parsing or command substitution
#[test]
fn test_ISSUE_059_003_or_true_after_simple_command() {
    // Simpler case: does || work after a simple command?
    let script = "echo hello || true";

    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let result = parser.parse();

    // ASSERT: Parser must accept || after simple command
    assert!(
        result.is_ok(),
        "Parser MUST accept '|| true' after simple command: {:?}",
        result.err()
    );

    let ast = result.expect("Should parse");
    assert!(
        !ast.statements.is_empty(),
        "Should have at least one statement"
    );
}

/// Issue #59: Test && operator after command (related to ||)
/// If || doesn't work, && probably doesn't either
#[test]
fn test_ISSUE_059_004_and_operator_after_command() {
    let script = "mkdir -p /tmp/test && echo success";

    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let result = parser.parse();

    // ASSERT: Parser must accept && between commands
    assert!(
        result.is_ok(),
        "Parser MUST accept '&&' between commands: {:?}",
        result.err()
    );

    let ast = result.expect("Should parse");
    assert!(
        !ast.statements.is_empty(),
        "Should have at least one statement"
    );
}

/// Issue #60: Test parsing brace groups after || operator
/// INPUT: cargo fmt --check || { echo "error"; exit 1; }
/// BUG: Fails with "Invalid syntax: Expected command name"
/// EXPECTED: Parses as OrList with command and brace group
#[test]
fn test_ISSUE_060_001_brace_group_after_or() {
    // RED PHASE: This test currently fails because brace groups aren't parsed
    let script = r#"cargo fmt --check || { echo "error"; exit 1; }"#;

    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let result = parser.parse();

    // ASSERT: Parser must accept brace groups after ||
    assert!(
        result.is_ok(),
        "Parser MUST accept brace group after ||: {:?}",
        result.err()
    );

    let ast = result.expect("Should parse");
    assert!(
        !ast.statements.is_empty(),
        "Should have at least one statement"
    );

    // Should be an OrList
    match &ast.statements[0] {
        BashStmt::OrList { left, right, .. } => {
            // Left should be a command
            assert!(
                matches!(**left, BashStmt::Command { .. }),
                "Left side should be a command, got: {:?}",
                left
            );
            // Right should be a brace group
            assert!(
                matches!(**right, BashStmt::BraceGroup { .. }),
                "Right side should be a brace group, got: {:?}",
                right
            );
        }
        other => panic!("Expected OrList statement, got: {:?}", other),
    }
}

/// Issue #60: Test parsing standalone brace group
/// INPUT: { echo "hello"; echo "world"; }
#[test]
fn test_ISSUE_060_002_standalone_brace_group() {
    let script = r#"{ echo "hello"; echo "world"; }"#;

    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let result = parser.parse();

    // ASSERT: Parser must accept standalone brace groups
    assert!(
        result.is_ok(),
        "Parser MUST accept standalone brace group: {:?}",
        result.err()
    );

    let ast = result.expect("Should parse");
    assert!(
        !ast.statements.is_empty(),
        "Should have at least one statement"
    );

    // Should be a BraceGroup
    match &ast.statements[0] {
        BashStmt::BraceGroup { body, .. } => {
            assert!(
                body.len() >= 2,
                "Brace group should have at least 2 statements, got: {}",
                body.len()
            );
        }
        other => panic!("Expected BraceGroup statement, got: {:?}", other),
    }
}

/// Issue #60: Test parsing brace group after && operator
/// INPUT: test -f file && { echo "exists"; cat file; }
#[test]
fn test_ISSUE_060_003_brace_group_after_and() {
    let script = r#"test -f file && { echo "exists"; cat file; }"#;

    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let result = parser.parse();

    // ASSERT: Parser must accept brace groups after &&
    assert!(
        result.is_ok(),
        "Parser MUST accept brace group after &&: {:?}",
        result.err()
    );
}

// ============================================================================
// Issue #62: Extended test [[ ]] conditionals
// ============================================================================
// Bug: Parser fails on bash [[ ]] extended test syntax
// Root cause: Parser only handles POSIX [ ] tests, not bash [[ ]] tests

/// Issue #62: Test basic [[ ]] conditional in if statement
/// INPUT: if [[ -f file ]]; then echo exists; fi
/// EXPECTED: Parse successfully with ExtendedTest expression
#[test]
fn test_ISSUE_062_001_extended_test_file_exists() {
    let script = r#"if [[ -f /tmp/test.txt ]]; then echo exists; fi"#;

    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let result = parser.parse();

    // ASSERT: Parser must accept [[ ]] extended test syntax
    assert!(
        result.is_ok(),
        "Parser MUST accept [[ ]] extended test: {:?}",
        result.err()
    );
}

/// Issue #62: Test [[ ]] with negation
/// INPUT: if [[ ! -s file ]]; then echo empty; fi
/// EXPECTED: Parse successfully with negated test
#[test]
fn test_ISSUE_062_002_extended_test_negation() {
    let script = r#"if [[ ! -s /tmp/file.txt ]]; then echo "File is empty"; exit 1; fi"#;

    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "Parser MUST accept [[ ! ... ]] negated test: {:?}",
        result.err()
    );
}

/// Issue #62: Test [[ ]] with string comparison
/// INPUT: if [[ "$var" == "value" ]]; then ...; fi
/// EXPECTED: Parse successfully
#[test]
fn test_ISSUE_062_003_extended_test_string_comparison() {
    let script = r#"if [[ "$total" -eq 0 ]]; then echo "No data"; exit 1; fi"#;

    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "Parser MUST accept [[ ]] string comparison: {:?}",
        result.err()
    );
}

/// Issue #62: Test standalone [[ ]] as condition
/// INPUT: [[ -d /tmp ]] && echo "exists"
/// EXPECTED: Parse successfully
#[test]
fn test_ISSUE_062_004_extended_test_standalone() {
    let script = r#"[[ -d /tmp ]] && echo "directory exists""#;

    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "Parser MUST accept standalone [[ ]] test: {:?}",
        result.err()
    );
}

// ============================================================================
// Issue #61: Parser error with here-strings (<<<)
// ============================================================================
// Here-strings are a bash feature that provide a string to a command's stdin.
// Syntax: cmd <<< "string"
// This is NOT a heredoc (<<), it's a simpler single-line input mechanism.
//
// Master Ticket: #63 (Bash Syntax Coverage Gaps)
// ============================================================================

/// Test: Issue #61 - Basic here-string with variable
/// Input: `read line <<< "$data"`
/// Expected: Parser accepts here-string redirection
#[test]
fn test_ISSUE_061_001_herestring_basic() {
    let script = r#"data="hello world"
read line <<< "$data"
echo "$line""#;

    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "Parser MUST accept here-string <<<: {:?}",
        result.err()
    );
}

/// Test: Issue #61 - Here-string with literal string
/// Input: `cat <<< "hello world"`
/// Expected: Parser accepts here-string with literal
#[test]
fn test_ISSUE_061_002_herestring_literal() {
    let script = r#"cat <<< "hello world""#;

    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "Parser MUST accept here-string with literal: {:?}",
        result.err()
    );
}

/// Test: Issue #61 - Here-string with unquoted word
/// Input: `read word <<< hello`
/// Expected: Parser accepts here-string with unquoted word
#[test]
fn test_ISSUE_061_003_herestring_unquoted() {
    let script = r#"read word <<< hello"#;

    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "Parser MUST accept here-string with unquoted word: {:?}",
        result.err()
    );
}

/// Test: Issue #61 - Here-string in pipeline
/// Input: `cat <<< "test" | grep t`
/// Expected: Parser accepts here-string in pipeline
#[test]
fn test_ISSUE_061_004_herestring_pipeline() {
    let script = r#"cat <<< "test" | grep t"#;

    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "Parser MUST accept here-string in pipeline: {:?}",
        result.err()
    );
}

// =============================================================================
// F001-F020: Parser Falsification Tests (Issue #93, #103)
// Specification: docs/specifications/unix-runtime-improvements-docker-mac-bash-zsh-daemons.md
// =============================================================================

/// F001: Parser handles inline if/then/else/fi
/// Issue #93: Parser fails on valid inline if/then/else/fi syntax
/// Falsification: If this test fails, the hypothesis "parser handles inline if" is falsified
#[test]
fn test_F001_inline_if_then_else_fi() {
    let script = r#"if grep -q "pattern" "$FILE"; then echo "found"; else echo "not found"; fi"#;

    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "F001 FALSIFIED: Parser MUST handle inline if/then/else/fi. Error: {:?}",
        result.err()
    );

    let ast = result.unwrap();
    assert_eq!(
        ast.statements.len(),
        1,
        "F001 FALSIFIED: Should produce exactly one If statement"
    );

    match &ast.statements[0] {
        BashStmt::If {
            then_block,
            else_block,
            ..
        } => {
            assert!(
                !then_block.is_empty(),
                "F001 FALSIFIED: then_block should not be empty"
            );
            assert!(
                else_block.is_some(),
                "F001 FALSIFIED: else_block should be present"
            );
        }
        other => panic!("F001 FALSIFIED: Expected If statement, got {:?}", other),
    }
}

/// F001 variant: Inline if with command condition (Issue #93 exact reproduction)
#[test]
fn test_F001_issue93_exact_reproduction() {
    // Exact test case from Issue #93
    let script =
        r#"if grep -q "MAX_QUEUE_DEPTH.*=.*3" "$BRIDGE"; then pass "1"; else fail "2"; fi"#;

    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "F001 FALSIFIED: Issue #93 exact case must parse. Error: {:?}",
        result.err()
    );
}

/// F002: Parser handles empty array initialization
/// Issue #103: Parser fails on common bash array syntax
#[test]
fn test_F002_empty_array_initialization() {
    let script = r#"local arr=()"#;

    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "F002 FALSIFIED: Parser MUST handle empty array initialization. Error: {:?}",
        result.err()
    );
}

/// F003: Parser handles array append operator
/// Issue #103: Parser fails on arr+=("item") syntax
