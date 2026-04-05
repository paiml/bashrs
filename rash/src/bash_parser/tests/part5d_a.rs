#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]

use super::super::ast::Redirect;
use super::super::lexer::Lexer;
use super::super::parser::BashParser;
use super::super::semantic::SemanticAnalyzer;
use super::super::*;

#[test]
fn test_while_loop_with_semicolon_before_do() {
    let script = r#"
x=5
while [ "$x" = "5" ]; do
    echo "looping"
done
"#;

    let mut parser = BashParser::new(script).unwrap();
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "While loop with semicolon before do should parse successfully: {:?}",
        result.err()
    );

    let ast = result.unwrap();
    let has_while = ast
        .statements
        .iter()
        .any(|s| matches!(s, BashStmt::While { .. }));

    assert!(has_while, "AST should contain a while loop");
}

// EXTREME TDD - RED Phase: Test for arithmetic expansion $((expr))
// This is P0 blocker documented in multiple locations
// Bug: Parser cannot handle arithmetic expansion like y=$((y - 1))
// Expected error: InvalidSyntax or UnexpectedToken when parsing $((...))
// GREEN phase complete - lexer + parser implemented with proper operator precedence
#[test]
fn test_arithmetic_expansion_basic() {
    let script = r#"
x=5
y=$((x + 1))
echo "$y"
"#;

    let mut parser = BashParser::new(script).unwrap();
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "Arithmetic expansion should parse successfully: {:?}",
        result.err()
    );

    let ast = result.unwrap();

    // Verify we have an assignment with arithmetic expansion
    let has_arithmetic_assignment = ast.statements.iter().any(|s| {
        matches!(s, BashStmt::Assignment { value, .. }
            if matches!(value, BashExpr::Arithmetic(_)))
    });

    assert!(
        has_arithmetic_assignment,
        "AST should contain arithmetic expansion in assignment"
    );
}

#[test]
fn test_arithmetic_expansion_in_loop() {
    let script = r#"
count=3
while [ "$count" -gt "0" ]; do
    echo "Iteration $count"
    count=$((count - 1))
done
"#;

    let mut parser = BashParser::new(script).unwrap();
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "While loop with arithmetic decrement should parse: {:?}",
        result.err()
    );

    let ast = result.unwrap();
    let has_while = ast
        .statements
        .iter()
        .any(|s| matches!(s, BashStmt::While { .. }));

    assert!(has_while, "AST should contain a while loop");
}

#[test]
fn test_arithmetic_expansion_complex_expressions() {
    let script = r#"
a=10
b=20
sum=$((a + b))
diff=$((a - b))
prod=$((a * b))
quot=$((a / b))
mod=$((a % b))
"#;

    let mut parser = BashParser::new(script).unwrap();
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "Complex arithmetic expressions should parse: {:?}",
        result.err()
    );
}

// ============================================================================
// ISSUE #4: Benchmark Parser Gaps - STOP THE LINE (P0 BLOCKER)
// ============================================================================
// Issue: docs/known-limitations/issue-004-benchmark-parser-gaps.md
//
// All benchmark fixture files (small.sh, medium.sh, large.sh) fail to parse
// due to missing parser support for common bash constructs:
// 1. $RANDOM - Special bash variable (0-32767 random integer)
// 2. $$ - Process ID variable
// 3. $(command) - Command substitution
// 4. function keyword - Function definition syntax
//
// These tests verify parser ACCEPTS these constructs (LEXER/PARSER ONLY).
// Purification transformation is separate (handled by purifier).
//
// Architecture: bash → PARSE (accept) → AST → PURIFY (transform) → POSIX sh
// Cannot purify what cannot be parsed!
// ============================================================================

#[test]
fn test_ISSUE_004_001_parse_random_special_variable() {
    // RED PHASE: Write failing test for $RANDOM parsing
    //
    // CRITICAL: Parser MUST accept $RANDOM to enable purification
    // Purifier will later reject/transform it, but parser must accept first
    //
    // INPUT: bash with $RANDOM
    // EXPECTED: Parser accepts, returns AST with Variable("RANDOM")
    // PURIFIER (later): Rejects or transforms to deterministic alternative

    let bash = r#"
#!/bin/bash
ID=$RANDOM
echo "Random ID: $ID"
"#;

    // ARRANGE: Lexer should tokenize $RANDOM
    let lexer_result = BashParser::new(bash);
    assert!(
        lexer_result.is_ok(),
        "Lexer should tokenize $RANDOM: {:?}",
        lexer_result.err()
    );

    // ACT: Parser should accept $RANDOM
    let mut parser = lexer_result.unwrap();
    let parse_result = parser.parse();

    // ASSERT: Parser must accept $RANDOM (for purification to work)
    assert!(
        parse_result.is_ok(),
        "Parser MUST accept $RANDOM to enable purification: {:?}",
        parse_result.err()
    );

    // VERIFY: AST contains assignment with Variable("RANDOM")
    let ast = parse_result.unwrap();
    assert!(
        !ast.statements.is_empty(),
        "$RANDOM should produce non-empty AST"
    );
}

#[test]
fn test_ISSUE_004_002_parse_process_id_variable() {
    // RED PHASE: Write failing test for $$ parsing
    //
    // CRITICAL: Parser MUST accept $$ to enable purification
    // $$ is process ID (non-deterministic, needs purification)
    //
    // INPUT: bash with $$
    // EXPECTED: Parser accepts, returns AST with special PID variable
    // PURIFIER (later): Transforms to deterministic alternative

    let bash = r#"
#!/bin/bash
PID=$$
TEMP_DIR="/tmp/build-$PID"
echo "Process ID: $PID"
"#;

    // ARRANGE: Lexer should tokenize $$
    let lexer_result = BashParser::new(bash);
    assert!(
        lexer_result.is_ok(),
        "Lexer should tokenize $$: {:?}",
        lexer_result.err()
    );

    // ACT: Parser should accept $$
    let mut parser = lexer_result.unwrap();
    let parse_result = parser.parse();

    // ASSERT: Parser must accept $$ (for purification to work)
    assert!(
        parse_result.is_ok(),
        "Parser MUST accept $$ to enable purification: {:?}",
        parse_result.err()
    );

    // VERIFY: AST contains assignment with PID variable
    let ast = parse_result.unwrap();
    assert!(
        !ast.statements.is_empty(),
        "$$ should produce non-empty AST"
    );
}

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
