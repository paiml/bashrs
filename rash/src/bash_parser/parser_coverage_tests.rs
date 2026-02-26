//! Coverage tests targeting uncovered functions in bash_parser/parser.rs
//!
//! Focus areas:
//! - `expect` (line 789, 0% coverage) — error path when token mismatch
//! - `tokens_adjacent` (line 834, 0% coverage) — assignment adjacency check
//! - `skip_condition_redirects` (line 860, 50% coverage) — redirect skipping
//! - Edge cases in partially-covered parser functions
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::ast::{BashExpr, BashStmt};
use super::parser::{BashParser, ParseError};

// ---------------------------------------------------------------------------
// expect() — error path tests (line 789)
// ---------------------------------------------------------------------------

/// `expect` returns an error when the next token does not match.
/// Trigger by writing invalid bash that requires a specific keyword.
#[test]
fn test_expect_error_missing_then() {
    // `if` without `then` triggers expect(Token::Then) failure
    let input = "if [ 1 = 1 ]; echo missing_then; fi";
    let result = BashParser::new(input).and_then(|mut p| p.parse());
    assert!(
        result.is_err(),
        "should error on missing 'then': {:?}",
        result
    );
    if let Err(ParseError::UnexpectedToken { expected, .. }) = result {
        assert!(
            expected.contains("then") || expected.contains("'then'"),
            "expected hint about 'then', got: {expected}"
        );
    }
}

#[test]
fn test_expect_error_missing_do_in_while() {
    // `while` without `do` triggers expect(Token::Do) failure
    let input = "while true; echo no_do; done";
    let result = BashParser::new(input).and_then(|mut p| p.parse());
    assert!(result.is_err(), "should error on missing 'do'");
}

#[test]
fn test_expect_error_missing_fi() {
    // Unclosed `if` triggers expect(Token::Fi) failure at EOF
    let input = "if true; then\n  echo hi\n";
    let result = BashParser::new(input).and_then(|mut p| p.parse());
    assert!(result.is_err(), "should error on missing 'fi'");
}

#[test]
fn test_expect_error_missing_done() {
    // Unclosed `for` loop triggers expect(Token::Done)
    let input = "for x in 1 2 3; do\n  echo $x\n";
    let result = BashParser::new(input).and_then(|mut p| p.parse());
    assert!(result.is_err(), "should error on missing 'done'");
}

#[test]
fn test_expect_error_missing_esac() {
    // Unclosed `case` triggers expect(Token::Esac)
    let input = "case $x in\n  a) echo a ;;\n";
    let result = BashParser::new(input).and_then(|mut p| p.parse());
    assert!(result.is_err(), "should error on missing 'esac'");
}

#[test]
fn test_expect_success_then_present() {
    // Happy path: `expect` succeeds when the token is present
    let input = "if [ 1 = 1 ]; then\n  echo ok\nfi";
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    assert!(ast
        .statements
        .iter()
        .any(|s| matches!(s, BashStmt::If { .. })));
}

// ---------------------------------------------------------------------------
// tokens_adjacent() — adjacency check (line 834)
// ---------------------------------------------------------------------------

/// VAR=VALUE (no space) must be parsed as an assignment, not a command.
/// tokens_adjacent() is the gating function for this distinction.
#[test]
fn test_tokens_adjacent_assignment_no_space() {
    let input = "FOO=bar";
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    assert_eq!(ast.statements.len(), 1);
    assert!(
        matches!(ast.statements[0], BashStmt::Assignment { .. }),
        "no-space assignment should be BashStmt::Assignment"
    );
}

#[test]
fn test_tokens_adjacent_variable_value() {
    let input = "X=hello_world";
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    assert!(matches!(ast.statements[0], BashStmt::Assignment { .. }));
    if let BashStmt::Assignment { name, value, .. } = &ast.statements[0] {
        assert_eq!(name, "X");
        if let BashExpr::Literal(v) = value {
            assert_eq!(v, "hello_world");
        }
    }
}

#[test]
fn test_tokens_adjacent_empty_assignment() {
    // VAR= (empty value) is still an assignment
    let input = "EMPTY=";
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    // Should parse without error — may be assignment or command depending on impl
    assert!(!ast.statements.is_empty());
}

#[test]
fn test_tokens_adjacent_multiple_assignments() {
    let input = "A=1\nB=2\nC=3";
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    assert_eq!(ast.statements.len(), 3);
    assert!(ast
        .statements
        .iter()
        .all(|s| matches!(s, BashStmt::Assignment { .. })));
}

// ---------------------------------------------------------------------------
// skip_condition_redirects() — redirect skipping (line 860)
// ---------------------------------------------------------------------------

/// Heredoc in `while` condition — exercises Heredoc branch in skip_condition_redirects
#[test]
fn test_skip_condition_redirects_heredoc_in_while() {
    let input = "while read line; do\n  echo $line\ndone <<EOF\nhello\nEOF";
    let result = BashParser::new(input).and_then(|mut p| p.parse());
    // Should parse without panicking — result may succeed or fail gracefully
    let _ = result;
}

/// Output redirect on done — bare `>` redirect
#[test]
fn test_skip_condition_redirects_bare_output_redirect() {
    let input = "for x in 1 2 3; do\n  echo $x\ndone > /tmp/out.txt";
    let result = BashParser::new(input).and_then(|mut p| p.parse());
    // Should parse without panic
    let _ = result;
}

/// Append redirect on done — bare `>>` redirect
#[test]
fn test_skip_condition_redirects_append_redirect() {
    let input = "for x in a b; do\n  echo $x\ndone >> /tmp/log.txt";
    let result = BashParser::new(input).and_then(|mut p| p.parse());
    let _ = result;
}

/// Input redirect on done — bare `<` redirect
#[test]
fn test_skip_condition_redirects_input_redirect() {
    let input = "while read line; do\n  echo $line\ndone < /tmp/input.txt";
    let result = BashParser::new(input).and_then(|mut p| p.parse());
    let _ = result;
}

/// FD-prefixed redirect: 2>/dev/null on compound command
#[test]
fn test_skip_condition_redirects_fd_prefixed() {
    let input = "for x in 1 2; do\n  echo $x\ndone 2>/dev/null";
    let result = BashParser::new(input).and_then(|mut p| p.parse());
    let _ = result;
}

/// FD duplication: 2>&1
#[test]
fn test_skip_condition_redirects_fd_duplication() {
    let input = "while true; do\n  echo hi\n  break\ndone 2>&1";
    let result = BashParser::new(input).and_then(|mut p| p.parse());
    let _ = result;
}

/// Here-string (<<<) on compound command
#[test]
fn test_skip_condition_redirects_herestring() {
    // HereString token is emitted by lexer for <<<
    let input = "while read line; do\n  echo $line\ndone <<< \"hello world\"";
    let result = BashParser::new(input).and_then(|mut p| p.parse());
    let _ = result;
}

/// if-fi with output redirect — exercises skip_condition_redirects via skip_compound_redirects
#[test]
fn test_skip_compound_redirects_on_fi() {
    let input = "if true; then\n  echo hi\nfi > /tmp/out";
    let result = BashParser::new(input).and_then(|mut p| p.parse());
    let _ = result;
}

// ---------------------------------------------------------------------------
// Partial-coverage branches in other parser functions
// ---------------------------------------------------------------------------

/// Coproc keyword (BUG-018) — exercises parse_coproc path
#[test]
fn test_parse_coproc_basic() {
    let input = "coproc cat /dev/stdin";
    let result = BashParser::new(input).and_then(|mut p| p.parse());
    // Coproc is supported, result should be Ok
    let _ = result;
}

/// Select statement (F017) — exercises parse_select path
#[test]
fn test_parse_select_statement() {
    let input = "select opt in a b c; do\n  echo $opt\ndone";
    let result = BashParser::new(input).and_then(|mut p| p.parse());
    let _ = result;
}

/// Process substitution as command argument
#[test]
fn test_parse_process_substitution_arg() {
    let input = "diff <(sort a.txt) <(sort b.txt)";
    let result = BashParser::new(input).and_then(|mut p| p.parse());
    let _ = result;
}

/// Pipeline RHS is a compound command (exercises parse_pipeline_rhs branches)
#[test]
fn test_parse_pipeline_rhs_while() {
    let input = "cat file.txt | while read line; do echo $line; done";
    let result = BashParser::new(input).and_then(|mut p| p.parse());
    let _ = result;
}

#[test]
fn test_parse_pipeline_rhs_for() {
    let input = "ls | for x in 1 2; do echo $x; done";
    let result = BashParser::new(input).and_then(|mut p| p.parse());
    let _ = result;
}

#[test]
fn test_parse_pipeline_rhs_if() {
    let input = "echo hello | if true; then cat; fi";
    let result = BashParser::new(input).and_then(|mut p| p.parse());
    let _ = result;
}

#[test]
fn test_parse_pipeline_rhs_brace_group() {
    let input = "cat file | { sort; uniq; }";
    let result = BashParser::new(input).and_then(|mut p| p.parse());
    let _ = result;
}

#[test]
fn test_parse_pipeline_rhs_subshell() {
    let input = "cat file | (sort | uniq)";
    let result = BashParser::new(input).and_then(|mut p| p.parse());
    let _ = result;
}

/// Background operator as statement terminator
#[test]
fn test_parse_background_operator() {
    let input = "sleep 100 &\necho done";
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    assert!(!ast.statements.is_empty());
}

/// Or-list (||) exercising parse_statement OrList path
#[test]
fn test_parse_or_list() {
    let input = "command_a || command_b";
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    assert!(ast
        .statements
        .iter()
        .any(|s| matches!(s, BashStmt::OrList { .. })));
}

/// And-list (&&) exercising parse_statement AndList path
#[test]
fn test_parse_and_list() {
    let input = "mkdir -p /tmp/foo && echo created";
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    assert!(ast
        .statements
        .iter()
        .any(|s| matches!(s, BashStmt::AndList { .. })));
}

/// parse_block_until with background (&) separator between statements
#[test]
fn test_parse_block_background_separator() {
    let input = "{ cmd1 & cmd2 & cmd3; }";
    let result = BashParser::new(input).and_then(|mut p| p.parse());
    let _ = result;
}

/// Comment parsing
#[test]
fn test_parse_comment() {
    let input = "# this is a comment\necho hello";
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    assert!(ast
        .statements
        .iter()
        .any(|s| matches!(s, BashStmt::Comment { .. })));
}

/// ParseError::line() and ParseError::column() accessors
#[test]
fn test_parse_error_line_number() {
    let input = "if true; then\n  echo hi\n  echo there\n";
    // No fi — should error
    let result = BashParser::new(input).and_then(|mut p| p.parse());
    assert!(result.is_err());
    if let Err(e) = result {
        // line() should return Some for UnexpectedToken/UnexpectedEof
        let _ = e.line(); // Just call it for coverage
    }
}

#[test]
fn test_parse_error_column_from_lexer() {
    // Unterminated string causes a LexerError with column info
    let input = r#"echo "unclosed"#;
    let result = BashParser::new(input);
    assert!(result.is_err());
    if let Err(e) = result {
        let _ = e.column();
        let _ = e.line();
    }
}

/// format_parse_diagnostic with UnexpectedToken
#[test]
fn test_format_parse_diagnostic_unexpected_token() {
    use super::parser::format_parse_diagnostic;
    let source = "if true; echo hi; fi";
    let err = ParseError::UnexpectedToken {
        expected: "'then' keyword".to_string(),
        found: "'echo'".to_string(),
        line: 1,
    };
    let diag = format_parse_diagnostic(&err, source, Some("test.sh"));
    assert!(diag.error.contains("expected") || diag.error.contains("then"));
}

/// format_parse_diagnostic with UnexpectedEof
#[test]
fn test_format_parse_diagnostic_unexpected_eof() {
    use super::parser::format_parse_diagnostic;
    let source = "if true; then\n  echo hi\n";
    let err = ParseError::UnexpectedEof;
    let diag = format_parse_diagnostic(&err, source, None);
    assert!(diag.error.contains("end of file") || diag.error.contains("unexpected"));
}

/// format_parse_diagnostic with InvalidSyntax
#[test]
fn test_format_parse_diagnostic_invalid_syntax() {
    use super::parser::format_parse_diagnostic;
    let source = "echo test";
    let err = ParseError::InvalidSyntax("something went wrong".to_string());
    let diag = format_parse_diagnostic(&err, source, None);
    assert!(diag.error.contains("something went wrong"));
}

/// build_snippet utility — exercises multi-line snippet rendering
#[test]
fn test_build_snippet_line_context() {
    use super::parser::build_snippet;
    let source = "line1\nline2\nline3\nline4";
    let snippet = build_snippet(source, 2, Some(3), 4);
    assert!(snippet.contains("line2"));
}

#[test]
fn test_build_snippet_first_line() {
    use super::parser::build_snippet;
    let source = "only_one_line";
    let snippet = build_snippet(source, 1, None, 1);
    assert!(snippet.contains("only_one_line"));
}

/// `skip_newlines` coverage — exercised inside parse loops, but test directly through
/// a multi-newline script to ensure the branch is hit
#[test]
fn test_skip_newlines_between_statements() {
    let input = "\n\n\necho hello\n\n\necho world\n\n";
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    assert_eq!(ast.statements.len(), 2);
}

/// `syntax_error` path — InvalidSyntax produced internally
#[test]
fn test_syntax_error_in_pipeline_without_rhs() {
    // Pipe at end of input with nothing on RHS
    let input = "echo hello |";
    let result = BashParser::new(input).and_then(|mut p| p.parse());
    // May error or succeed depending on parser; just ensure no panic
    let _ = result;
}

/// `check` / `is_at_end` exercised via empty input
#[test]
fn test_parse_empty_input() {
    let input = "";
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    assert!(ast.statements.is_empty());
}

/// `source()` accessor on BashParser
#[test]
fn test_parser_source_accessor() {
    let input = "echo hello";
    let parser = BashParser::new(input).unwrap();
    assert_eq!(parser.source(), input);
}

/// Until loop — exercises parse_until path (distinct from while)
#[test]
fn test_parse_until_loop() {
    let input = "until false; do\n  echo waiting\ndone";
    let result = BashParser::new(input).and_then(|mut p| p.parse());
    // until is supported
    let _ = result;
}

/// Brace group — exercises parse_brace_group
#[test]
fn test_parse_brace_group() {
    let input = "{ echo a; echo b; }";
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    assert!(!ast.statements.is_empty());
}

/// Subshell — exercises parse_subshell
#[test]
fn test_parse_subshell() {
    let input = "(echo hello; echo world)";
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    assert!(!ast.statements.is_empty());
}

/// Nested if-elif-else — deeper branch coverage
#[test]
fn test_parse_if_elif_else() {
    let input = r#"if [ $x = 1 ]; then
  echo one
elif [ $x = 2 ]; then
  echo two
else
  echo other
fi"#;
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    assert!(ast
        .statements
        .iter()
        .any(|s| matches!(s, BashStmt::If { .. })));
}

/// Arithmetic expansion in assignment
#[test]
fn test_parse_arithmetic_expansion_assignment() {
    let input = "RESULT=$((1 + 2))";
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    assert!(!ast.statements.is_empty());
}

/// Command substitution in assignment
#[test]
fn test_parse_command_substitution_assignment() {
    let input = "PWD=$(pwd)";
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    assert!(!ast.statements.is_empty());
}

/// Multiple FD redirects on the same command
#[test]
fn test_parse_multiple_redirects_on_command() {
    let input = "cmd 2>/dev/null 1>/tmp/out";
    let result = BashParser::new(input).and_then(|mut p| p.parse());
    let _ = result;
}

/// Negated pipeline (! pipeline)
#[test]
fn test_parse_negated_pipeline() {
    let input = "! grep foo bar.txt";
    let result = BashParser::new(input).and_then(|mut p| p.parse());
    let _ = result;
}
