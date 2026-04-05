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
