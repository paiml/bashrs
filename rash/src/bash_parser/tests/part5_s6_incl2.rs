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
