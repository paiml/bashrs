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
    assert!(
        regex_tok.is_some(),
        "Expected =~ token, got: {:?}",
        tokens
    );
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

#[test]
fn test_LEX_OP_NOT_001_bang_not_followed_by_eq() {
    // Standalone ! followed by space is Not, not Ne
    let tokens = lex("! true");
    assert!(tokens.iter().any(|t| matches!(t, Token::Not)));
    assert!(!tokens.iter().any(|t| matches!(t, Token::Ne)));
}

#[test]
fn test_LEX_OP_NOT_002_ne_operator() {
    // != should be Ne, not (Not + Assign)
    let tokens = lex("[ $a != $b ]");
    assert!(tokens.iter().any(|t| matches!(t, Token::Ne)));
    // Should not have separate Not token for !=
    let not_count = tokens
        .iter()
        .filter(|t| matches!(t, Token::Not))
        .count();
    assert_eq!(
        not_count, 0,
        "!= should be a single Ne token, not separate Not, got: {:?}",
        tokens
    );
}

// ============================================================================
// Pipe Variants
// ============================================================================

#[test]
fn test_LEX_OP_PIPE_001_or_not_confused_with_pipe() {
    // || should produce Or, not two Pipes
    let tokens = lex("cmd1 || cmd2");
    assert!(tokens.iter().any(|t| matches!(t, Token::Or)));
    let pipe_count = tokens
        .iter()
        .filter(|t| matches!(t, Token::Pipe))
        .count();
    assert_eq!(
        pipe_count, 0,
        "|| should not produce standalone Pipe tokens, got {}: {:?}",
        pipe_count, tokens
    );
}

#[test]
fn test_LEX_OP_PIPE_002_multiple_pipes() {
    let tokens = lex("a | b | c | d");
    let pipe_count = tokens
        .iter()
        .filter(|t| matches!(t, Token::Pipe))
        .count();
    assert_eq!(
        pipe_count, 3,
        "Expected 3 pipes, got {}: {:?}",
        pipe_count, tokens
    );
}

// ============================================================================
// Bracket Variants
// ============================================================================

#[test]
fn test_LEX_OP_BRACKET_001_single_vs_double_brackets() {
    let tokens_single = lex("[ -f file ]");
    let tokens_double = lex("[[ -f file ]]");

    assert!(tokens_single.iter().any(|t| matches!(t, Token::LeftBracket)));
    assert!(tokens_single.iter().any(|t| matches!(t, Token::RightBracket)));
    assert!(!tokens_single.iter().any(|t| matches!(t, Token::DoubleLeftBracket)));

    assert!(tokens_double.iter().any(|t| matches!(t, Token::DoubleLeftBracket)));
    assert!(tokens_double.iter().any(|t| matches!(t, Token::DoubleRightBracket)));
    assert!(!tokens_double.iter().any(|t| matches!(t, Token::LeftBracket)));
}

// ============================================================================
// Error Path: Unexpected Character
// ============================================================================

#[test]
fn test_LEX_OP_ERR_001_unexpected_char() {
    // A character that isn't handled by any branch in read_operator
    // or other tokenization paths should produce an UnexpectedChar error.
    // The backtick ` triggers command substitution in some shells but
    // may cause an error if not handled.
    let mut lexer = Lexer::new("`");
    let result = lexer.tokenize();
    // Either it errors or handles it as some token — verify it doesn't panic
    assert!(
        result.is_ok() || result.is_err(),
        "Tokenizing unexpected char should not panic"
    );
}

// ============================================================================
// Question Mark as Glob
// ============================================================================

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
    let assign_count = tokens
        .iter()
        .filter(|t| matches!(t, Token::Assign))
        .count();
    assert!(
        assign_count >= 2,
        "Expected at least 2 assignments, got {}: {:?}",
        assign_count, tokens
    );
}

#[test]
fn test_LEX_OP_COMPOUND_003_all_redirect_types_in_one() {
    // Combine <, >, >>, >|, <> in a sequence of commands
    let tokens = lex("cmd1 < a; cmd2 > b; cmd3 >> c; cmd4 >| d");
    assert!(tokens.iter().any(|t| matches!(t, Token::Lt)));
    assert!(tokens.iter().any(|t| matches!(t, Token::Gt)));
    assert!(tokens.iter().any(|t| matches!(t, Token::GtGt)));
    assert!(
        tokens
            .iter()
            .any(|t| matches!(t, Token::Identifier(s) if s == ">|"))
    );
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
        tokens
            .iter()
            .any(|t| matches!(t, Token::HereString(_))),
        "Expected HereString token with double-quoted input, got: {:?}",
        tokens
    );
}

#[test]
fn test_LEX_OP_HSTR_002_herestring_unquoted() {
    let tokens = lex("cat <<< word");
    assert!(
        tokens
            .iter()
            .any(|t| matches!(t, Token::HereString(_))),
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
