//! Tests for `read_operator` and its helper functions in `lexer.rs`.
//!
//! Focuses on less-common operator branches that existing inline tests
//! do not exercise: extended globs (`@(`, `+(`, `?(`), regex match (`=~`),
//! standalone arithmetic `((…))`, brace expansion, plain assignment `=`,
//! plain `<`/`>`, FD redirects, the error path for unexpected characters,
//! and edge-case interactions between operators.
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::lexer::{Lexer, LexerError, Token};

/// Helper: tokenize input and return the token vector.
fn lex(input: &str) -> Vec<Token> {
    let mut lexer = Lexer::new(input);
    lexer.tokenize().unwrap_or_default()
}

/// Helper: tokenize input, expecting success, and return the token vector.
fn lex_ok(input: &str) -> Vec<Token> {
    let mut lexer = Lexer::new(input);
    lexer.tokenize().expect("tokenization should succeed")
}

// ============================================================================
// Extended Glob Operators: @(...), +(...), ?(...)
// ============================================================================

#[test]
fn test_LEX_OP_EXT_001_at_glob_pattern() {
    // @(pattern) — matches exactly one of the patterns
    let tokens = lex("echo @(foo|bar)");
    assert!(
        tokens
            .iter()
            .any(|t| matches!(t, Token::Identifier(s) if s == "@(foo|bar)")),
        "Expected @(foo|bar) extended glob, got: {:?}",
        tokens
    );
}

#[test]
fn test_LEX_OP_EXT_002_plus_glob_pattern() {
    // +(pattern) — matches one or more of the patterns
    let tokens = lex("echo +(abc|def)");
    assert!(
        tokens
            .iter()
            .any(|t| matches!(t, Token::Identifier(s) if s == "+(abc|def)")),
        "Expected +(abc|def) extended glob, got: {:?}",
        tokens
    );
}

#[test]
fn test_LEX_OP_EXT_003_question_glob_pattern() {
    // ?(pattern) — matches zero or one of the patterns
    let tokens = lex("echo ?(opt1|opt2)");
    assert!(
        tokens
            .iter()
            .any(|t| matches!(t, Token::Identifier(s) if s == "?(opt1|opt2)")),
        "Expected ?(opt1|opt2) extended glob, got: {:?}",
        tokens
    );
}

#[test]
fn test_LEX_OP_EXT_004_at_glob_nested_parens() {
    // Nested parens inside extended glob
    let tokens = lex("@(a|(b|c))");
    assert!(
        tokens
            .iter()
            .any(|t| matches!(t, Token::Identifier(s) if s == "@(a|(b|c))")),
        "Expected nested @(a|(b|c)), got: {:?}",
        tokens
    );
}

#[test]
fn test_LEX_OP_EXT_005_plus_glob_single_pattern() {
    let tokens = lex("+(single)");
    assert!(
        tokens
            .iter()
            .any(|t| matches!(t, Token::Identifier(s) if s == "+(single)")),
        "Expected +(single), got: {:?}",
        tokens
    );
}

// ============================================================================
// Regex Match Operator: =~
// ============================================================================

#[test]
fn test_LEX_OP_REGEX_001_basic_regex_match() {
    let tokens = lex("[[ $str =~ ^[0-9]+$ ]]");
    assert!(
        tokens
            .iter()
            .any(|t| matches!(t, Token::Identifier(s) if s.starts_with("=~ "))),
        "Expected =~ regex match operator, got: {:?}",
        tokens
    );
}

#[test]
fn test_LEX_OP_REGEX_002_regex_pattern_content() {
    let tokens = lex("[[ $var =~ foo.*bar ]]");
    let regex_tok = tokens
        .iter()
        .find(|t| matches!(t, Token::Identifier(s) if s.starts_with("=~ ")));
    assert!(regex_tok.is_some(), "Expected =~ token, got: {:?}", tokens);
    if let Some(Token::Identifier(s)) = regex_tok {
        assert!(
            s.contains("foo.*bar"),
            "Regex pattern should contain 'foo.*bar', got: {}",
            s
        );
    }
}

#[test]
fn test_LEX_OP_REGEX_003_regex_with_bracket_class() {
    // =~ with character class [[:alpha:]] — bracket depth tracking matters
    let tokens = lex("[[ $x =~ [[:alpha:]]+ ]]");
    let regex_tok = tokens
        .iter()
        .find(|t| matches!(t, Token::Identifier(s) if s.starts_with("=~ ")));
    assert!(
        regex_tok.is_some(),
        "Expected =~ regex token with bracket class, got: {:?}",
        tokens
    );
}

// ============================================================================
// Standalone Arithmetic: ((expr))
// ============================================================================

#[test]
fn test_LEX_OP_ARITH_001_standalone_arithmetic() {
    let tokens = lex("((x + 1))");
    assert!(
        tokens
            .iter()
            .any(|t| matches!(t, Token::ArithmeticExpansion(_))),
        "Expected ArithmeticExpansion token, got: {:?}",
        tokens
    );
}

#[test]
fn test_LEX_OP_ARITH_002_standalone_arithmetic_content() {
    let tokens = lex("((i++))");
    let arith = tokens
        .iter()
        .find(|t| matches!(t, Token::ArithmeticExpansion(_)));
    assert!(
        arith.is_some(),
        "Expected ArithmeticExpansion, got: {:?}",
        tokens
    );
}

#[test]
fn test_LEX_OP_ARITH_003_arithmetic_nested_parens() {
    let tokens = lex("((a * (b + c)))");
    assert!(
        tokens
            .iter()
            .any(|t| matches!(t, Token::ArithmeticExpansion(_))),
        "Expected ArithmeticExpansion with nested parens, got: {:?}",
        tokens
    );
}

// ============================================================================
// Brace Expansion: {a,b,c} and {1..10}
// ============================================================================

#[test]
fn test_LEX_OP_BRACE_001_comma_expansion() {
    let tokens = lex("echo {a,b,c}");
    assert!(
        tokens
            .iter()
            .any(|t| matches!(t, Token::Identifier(s) if s.contains(","))),
        "Expected brace expansion with commas, got: {:?}",
        tokens
    );
}

#[test]
fn test_LEX_OP_BRACE_002_range_expansion() {
    let tokens = lex("echo {1..10}");
    assert!(
        tokens
            .iter()
            .any(|t| matches!(t, Token::Identifier(s) if s.contains(".."))),
        "Expected brace expansion with range, got: {:?}",
        tokens
    );
}

#[test]

include!("lexer_operator_tests_tests_LEX.rs");
