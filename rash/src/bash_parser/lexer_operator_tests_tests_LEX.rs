fn test_LEX_OP_BRACE_003_nested_brace_expansion() {
    let tokens = lex("echo {a,{b,c}}");
    assert!(
        tokens
            .iter()
            .any(|t| matches!(t, Token::Identifier(s) if s.starts_with("{"))),
        "Expected nested brace expansion, got: {:?}",
        tokens
    );
}

// ============================================================================
// Plain Assignment and Comparison Operators
// ============================================================================

#[test]
fn test_LEX_OP_ASSIGN_001_plain_assign() {
    let tokens = lex("x=5");
    assert!(
        tokens.iter().any(|t| matches!(t, Token::Assign)),
        "Expected Assign token, got: {:?}",
        tokens
    );
}

#[test]
fn test_LEX_OP_ASSIGN_002_eq_operator() {
    let tokens = lex("[[ a == b ]]");
    assert!(
        tokens.iter().any(|t| matches!(t, Token::Eq)),
        "Expected Eq (==) token, got: {:?}",
        tokens
    );
}

#[test]
fn test_LEX_OP_ASSIGN_003_assign_vs_eq_disambiguation() {
    // Single = is Assign, double == is Eq
    let tokens_assign = lex("x=hello");
    let tokens_eq = lex("[[ x == y ]]");

    assert!(tokens_assign.iter().any(|t| matches!(t, Token::Assign)));
    assert!(!tokens_assign.iter().any(|t| matches!(t, Token::Eq)));

    assert!(tokens_eq.iter().any(|t| matches!(t, Token::Eq)));
}

// ============================================================================
// Plain Redirect Operators: < and >
// ============================================================================

#[test]
fn test_LEX_OP_REDIR_001_plain_input_redirect() {
    let tokens = lex("sort < input.txt");
    assert!(
        tokens.iter().any(|t| matches!(t, Token::Lt)),
        "Expected Lt (<) input redirect, got: {:?}",
        tokens
    );
}

#[test]
fn test_LEX_OP_REDIR_002_plain_output_redirect() {
    let tokens = lex("echo hi > output.txt");
    assert!(
        tokens.iter().any(|t| matches!(t, Token::Gt)),
        "Expected Gt (>) output redirect, got: {:?}",
        tokens
    );
}

#[test]
fn test_LEX_OP_REDIR_003_lt_not_confused_with_heredoc() {
    // Single < should be Lt, not start of heredoc
    let tokens = lex("cmd < file");
    assert!(tokens.iter().any(|t| matches!(t, Token::Lt)));
    assert!(!tokens.iter().any(|t| matches!(t, Token::Heredoc { .. })));
}

#[test]
fn test_LEX_OP_REDIR_004_gt_not_confused_with_append() {
    // Single > should be Gt, not >>
    let tokens = lex("cmd > file");
    assert!(tokens.iter().any(|t| matches!(t, Token::Gt)));
    assert!(!tokens.iter().any(|t| matches!(t, Token::GtGt)));
}

// ============================================================================
// Semicolon Variants
// ============================================================================

#[test]
fn test_LEX_OP_SEMI_001_plain_semicolon() {
    let tokens = lex("echo a; echo b; echo c");
    let count = tokens
        .iter()
        .filter(|t| matches!(t, Token::Semicolon))
        .count();
    assert_eq!(
        count, 2,
        "Expected 2 semicolons, got {}: {:?}",
        count, tokens
    );
}

#[test]
fn test_LEX_OP_SEMI_002_double_semicolon_in_case() {
    let tokens = lex("case $x in\na) echo yes;;\nesac");
    assert!(
        tokens
            .iter()
            .any(|t| matches!(t, Token::Identifier(s) if s == ";;")),
        "Expected ;; case terminator in case statement, got: {:?}",
        tokens
    );
}

#[test]
fn test_LEX_OP_SEMI_003_semicolon_ampersand_in_case() {
    let tokens = lex("case $x in\na) echo yes;&\nb) echo also;;\nesac");
    assert!(
        tokens
            .iter()
            .any(|t| matches!(t, Token::Identifier(s) if s == ";&")),
        "Expected ;& fall-through in case, got: {:?}",
        tokens
    );
}

#[test]
fn test_LEX_OP_SEMI_004_double_semicolon_ampersand_resume() {
    let tokens = lex("case $x in\na) echo yes;;&\nb) echo also;;\nesac");
    assert!(
        tokens
            .iter()
            .any(|t| matches!(t, Token::Identifier(s) if s == ";;&")),
        "Expected ;;& case resume in case, got: {:?}",
        tokens
    );
}

// ============================================================================
// Grouping Operators in Context
// ============================================================================

#[test]
fn test_LEX_OP_GROUP_001_subshell_parens() {
    let tokens = lex_ok("(cd /tmp && ls)");
    assert!(tokens.iter().any(|t| matches!(t, Token::LeftParen)));
    assert!(tokens.iter().any(|t| matches!(t, Token::RightParen)));
    assert!(tokens.iter().any(|t| matches!(t, Token::And)));
}

#[test]
fn test_LEX_OP_GROUP_002_brace_group_not_expansion() {
    // { cmd; } is a brace group, NOT a brace expansion
    let tokens = lex("{ echo hello; }");
    assert!(
        tokens.iter().any(|t| matches!(t, Token::LeftBrace)),
        "Expected LeftBrace, got: {:?}",
        tokens
    );
    assert!(
        tokens.iter().any(|t| matches!(t, Token::RightBrace)),
        "Expected RightBrace, got: {:?}",
        tokens
    );
}

// ============================================================================
// Background and And/Or
// ============================================================================

#[test]
fn test_LEX_OP_BG_001_ampersand_background() {
    let tokens = lex("cmd &");
    assert!(
        tokens.iter().any(|t| matches!(t, Token::Ampersand)),
        "Expected Ampersand for background, got: {:?}",
        tokens
    );
}

#[test]
fn test_LEX_OP_BG_002_and_not_confused_with_background() {
    // && should produce And, not two Ampersands
    let tokens = lex("cmd1 && cmd2");
    assert!(tokens.iter().any(|t| matches!(t, Token::And)));
    let amp_count = tokens
        .iter()
        .filter(|t| matches!(t, Token::Ampersand))
        .count();
    assert_eq!(
        amp_count, 0,
        "&& should not produce standalone Ampersand tokens, got {}: {:?}",
        amp_count, tokens
    );
}

// ============================================================================
// Not (!) Operator Variants
// ============================================================================

include!("lexer_operator_tests_tests_LEX_LEX.rs");
