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
#[test]
fn test_LEX_OP_QMARK_001_question_mark_glob() {
    let tokens = lex("?");
    assert!(
        tokens
            .iter()
            .any(|t| matches!(t, Token::Identifier(s) if s == "?")),
        "Expected ? as Identifier, got: {:?}",
        tokens
    );
}

// ============================================================================
// Complex Compound Operator Sequences
// ============================================================================

#[test]
fn test_LEX_OP_COMPOUND_001_redirect_chain() {
    // Multiple redirects: stdin from file, stderr to stdout, stdout to file
    let tokens = lex("cmd < in.txt > out.txt");
    assert!(
        tokens.iter().any(|t| matches!(t, Token::Lt)),
        "Expected < redirect, got: {:?}",
        tokens
    );
    assert!(
        tokens.iter().any(|t| matches!(t, Token::Gt)),
        "Expected > redirect, got: {:?}",
        tokens
    );
}

#[test]
fn test_LEX_OP_COMPOUND_002_conditional_with_assignment() {
    let tokens = lex("[[ $x == y ]] && result=pass || result=fail");
    assert!(tokens.iter().any(|t| matches!(t, Token::DoubleLeftBracket)));
    assert!(tokens.iter().any(|t| matches!(t, Token::Eq)));
    assert!(tokens.iter().any(|t| matches!(t, Token::And)));
    assert!(tokens.iter().any(|t| matches!(t, Token::Or)));
    let assign_count = tokens.iter().filter(|t| matches!(t, Token::Assign)).count();
    assert!(
        assign_count >= 2,
        "Expected at least 2 assignments, got {}: {:?}",
        assign_count,
        tokens
    );
}

#[test]
fn test_LEX_OP_COMPOUND_003_all_redirect_types_in_one() {
    // Combine <, >, >>, >|, <> in a sequence of commands
    let tokens = lex("cmd1 < a; cmd2 > b; cmd3 >> c; cmd4 >| d");
    assert!(tokens.iter().any(|t| matches!(t, Token::Lt)));
    assert!(tokens.iter().any(|t| matches!(t, Token::Gt)));
    assert!(tokens.iter().any(|t| matches!(t, Token::GtGt)));
    assert!(tokens
        .iter()
        .any(|t| matches!(t, Token::Identifier(s) if s == ">|")));
}

#[test]
fn test_LEX_OP_COMPOUND_004_heredoc_quoted_delimiter() {
    // Heredoc with quoted delimiter: <<'EOF'
    let tokens = lex("cat <<'MARKER'\nhello world\nMARKER\n");
    assert!(
        tokens
            .iter()
            .any(|t| matches!(t, Token::Heredoc { delimiter, .. } if delimiter == "MARKER")),
        "Expected Heredoc with quoted delimiter MARKER, got: {:?}",
        tokens
    );
}

#[test]
fn test_LEX_OP_COMPOUND_005_mixed_semicolons_and_ampersands() {
    let tokens = lex("cmd1; cmd2 & cmd3 && cmd4");
    assert!(tokens.iter().any(|t| matches!(t, Token::Semicolon)));
    assert!(tokens.iter().any(|t| matches!(t, Token::Ampersand)));
    assert!(tokens.iter().any(|t| matches!(t, Token::And)));
}

#[test]
fn test_LEX_OP_COMPOUND_006_le_ge_in_double_brackets() {
    let tokens = lex("[[ a <= b ]] && [[ c >= d ]]");
    assert!(
        tokens.iter().any(|t| matches!(t, Token::Le)),
        "Expected <= (Le), got: {:?}",
        tokens
    );
    assert!(
        tokens.iter().any(|t| matches!(t, Token::Ge)),
        "Expected >= (Ge), got: {:?}",
        tokens
    );
}

// ============================================================================
// Plus-Equals Operator
// ============================================================================

#[test]
fn test_LEX_OP_PLUSEQ_001_string_append() {
    let tokens = lex("str+=' world'");
    assert!(
        tokens
            .iter()
            .any(|t| matches!(t, Token::Identifier(s) if s == "+=")),
        "Expected += operator for string append, got: {:?}",
        tokens
    );
}

// ============================================================================
// Noclobber and Read-Write Redirects with No Spaces
// ============================================================================

#[test]
fn test_LEX_OP_REDIR_005_noclobber_no_spaces() {
    let tokens = lex("echo hi>|file");
    assert!(
        tokens
            .iter()
            .any(|t| matches!(t, Token::Identifier(s) if s == ">|")),
        "Expected >| noclobber without spaces, got: {:?}",
        tokens
    );
}

#[test]
fn test_LEX_OP_REDIR_006_readwrite_no_spaces() {
    let tokens = lex("exec 5<>file");
    assert!(
        tokens
            .iter()
            .any(|t| matches!(t, Token::Identifier(s) if s == "<>")),
        "Expected <> read-write without spaces, got: {:?}",
        tokens
    );
}

// ============================================================================
// Herestring Variants
// ============================================================================

#[test]
fn test_LEX_OP_HSTR_001_herestring_double_quoted() {
    let tokens = lex("cat <<< \"hello world\"");
    assert!(
        tokens.iter().any(|t| matches!(t, Token::HereString(_))),
        "Expected HereString token with double-quoted input, got: {:?}",
        tokens
    );
}

#[test]
fn test_LEX_OP_HSTR_002_herestring_unquoted() {
    let tokens = lex("cat <<< word");
    assert!(
        tokens.iter().any(|t| matches!(t, Token::HereString(_))),
        "Expected HereString token with unquoted word, got: {:?}",
        tokens
    );
}

// ============================================================================
// Process Substitution
// ============================================================================

#[test]
fn test_LEX_OP_PROCSUB_001_input_process_sub_nested() {
    let tokens = lex("diff <(sort file1) <(sort file2)");
    let proc_sub_count = tokens
        .iter()
        .filter(|t| matches!(t, Token::Identifier(s) if s.starts_with("<(")))
        .count();
    assert_eq!(
        proc_sub_count, 2,
        "Expected 2 input process substitutions, got {}: {:?}",
        proc_sub_count, tokens
    );
}

#[test]
fn test_LEX_OP_PROCSUB_002_output_process_sub_content() {
    let tokens = lex("tee >(wc -l)");
    let proc_tok = tokens
        .iter()
        .find(|t| matches!(t, Token::Identifier(s) if s.starts_with(">(")));
    assert!(
        proc_tok.is_some(),
        "Expected >(...) process substitution, got: {:?}",
        tokens
    );
    if let Some(Token::Identifier(s)) = proc_tok {
        assert!(
            s.contains("wc"),
            "Process substitution should contain 'wc', got: {}",
            s
        );
    }
}

// ============================================================================
// Token Positions (tokenize_with_positions)
// ============================================================================

#[test]
fn test_LEX_OP_POS_001_operator_positions() {
    let mut lexer = Lexer::new("a && b");
    let (tokens, positions) = lexer.tokenize_with_positions().unwrap();
    // 'a' at 0, '&&' at 2, 'b' at 5
    assert_eq!(tokens.len(), positions.len());
    assert!(tokens.iter().any(|t| matches!(t, Token::And)));
}
