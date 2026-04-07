//! Parser error display helpers — extracted from parser.rs for file health.

use super::lexer::{LexerError, Token};
use super::parser::ParseError;

/// Human-friendly name for a token (not Debug format)
pub(crate) fn token_display(tok: &Token) -> String {
    match tok {
        Token::Identifier(s) => format!("'{s}'"),
        Token::String(s) => format!("\"{}\"", s.chars().take(30).collect::<String>()),
        Token::Number(n) => format!("'{n}'"),
        Token::Variable(v) => format!("'${v}'"),
        Token::Assign => "'='".to_string(),
        Token::Semicolon => "';'".to_string(),
        Token::Pipe => "'|'".to_string(),
        Token::Ampersand => "'&'".to_string(),
        Token::LeftParen => "'('".to_string(),
        Token::RightParen => "')'".to_string(),
        Token::LeftBrace => "'{'".to_string(),
        Token::RightBrace => "'}'".to_string(),
        Token::LeftBracket => "'['".to_string(),
        Token::RightBracket => "']'".to_string(),
        Token::Newline => "newline".to_string(),
        Token::If => "'if'".to_string(),
        Token::Then => "'then'".to_string(),
        Token::Else => "'else'".to_string(),
        Token::Elif => "'elif'".to_string(),
        Token::Fi => "'fi'".to_string(),
        Token::For => "'for'".to_string(),
        Token::While => "'while'".to_string(),
        Token::Until => "'until'".to_string(),
        Token::Do => "'do'".to_string(),
        Token::Done => "'done'".to_string(),
        Token::Case => "'case'".to_string(),
        Token::Esac => "'esac'".to_string(),
        Token::In => "'in'".to_string(),
        Token::Function => "'function'".to_string(),
        Token::Return => "'return'".to_string(),
        Token::Local => "'local'".to_string(),
        Token::Export => "'export'".to_string(),
        Token::Dollar => "'$'".to_string(),
        Token::Heredoc { delimiter, .. } => format!("heredoc '<<{delimiter}'"),
        Token::HereString(s) => {
            format!("herestring '<<<{}'", s.chars().take(20).collect::<String>())
        }
        Token::CommandSubstitution(s) => format!("'$({s})'"),
        Token::ArithmeticExpansion(s) => format!("'$(({s}))'"),
        Token::Comment(_) => "comment".to_string(),
        _ => format!("{tok:?}"),
    }
}

/// Human-friendly expected token description
pub(crate) fn expected_display(tok: &Token) -> &'static str {
    match tok {
        Token::Then => "'then' keyword",
        Token::Do => "'do' keyword",
        Token::Fi => "'fi' keyword",
        Token::Done => "'done' keyword",
        Token::Esac => "'esac' keyword",
        Token::In => "'in' keyword",
        Token::LeftBrace => "'{'",
        Token::RightBrace => "'}'",
        Token::LeftParen => "'('",
        Token::RightParen => "')'",
        Token::LeftBracket => "'['",
        Token::RightBracket => "']'",
        Token::Semicolon => "';'",
        _ => "token",
    }
}

/// Contextual help suggestion based on what was expected vs found
pub(crate) fn suggest_fix(expected: &Token, found: Option<&Token>) -> Option<String> {
    match (expected, found) {
        (Token::Then, Some(Token::Identifier(_) | Token::Variable(_))) => {
            Some("add 'then' after the condition: `if [ ... ]; then`".to_string())
        }
        (Token::Then, _) => Some("'if' requires 'then' after the condition".to_string()),
        (Token::Do, Some(Token::Identifier(_) | Token::Variable(_))) => Some(
            "add 'do' after the loop condition: `while [ ... ]; do` or `for x in ...; do`"
                .to_string(),
        ),
        (Token::Do, _) => Some("loops require 'do' after the condition/iterator".to_string()),
        (Token::Fi, _) => Some("'if' block must be closed with 'fi'".to_string()),
        (Token::Done, _) => Some("loop must be closed with 'done'".to_string()),
        (Token::RightBrace, _) => Some("unmatched '{' — did you forget '}'?".to_string()),
        (Token::RightParen, _) => Some("unmatched '(' — did you forget ')'?".to_string()),
        (Token::In, _) => Some("'for' loop requires 'in': `for var in list; do`".to_string()),
        _ => None,
    }
}

/// Build a source snippet showing the error location with surrounding context.
///
/// Returns a rustc-style snippet:
/// ```text
///   2 | if [ "$x" = "y" ]
///   3 |    echo missing then
///     |    ^^^^ expected 'then', found 'echo'
/// ```
pub(crate) fn build_snippet(
    source: &str,
    line: usize,
    col: Option<usize>,
    highlight_len: usize,
) -> String {
    let lines: Vec<&str> = source.lines().collect();
    let line_idx = line.saturating_sub(1);
    let gutter_width = format!("{}", line.min(lines.len()) + 1).len();

    let mut snippet = String::new();

    // Show 1 line before for context (if available)
    if line_idx > 0 {
        let prev = line_idx - 1;
        let _ = std::fmt::Write::write_fmt(
            &mut snippet,
            format_args!(
                "{:>width$} | {}\n",
                prev + 1,
                lines.get(prev).unwrap_or(&""),
                width = gutter_width
            ),
        );
    }

    // Show the error line
    if let Some(src_line) = lines.get(line_idx) {
        let _ = std::fmt::Write::write_fmt(
            &mut snippet,
            format_args!("{:>width$} | {}\n", line, src_line, width = gutter_width),
        );

        // Show the caret indicator
        let caret_col = col.unwrap_or(1).saturating_sub(1);
        let caret_len = if highlight_len > 0 { highlight_len } else { 1 };
        let padding = " ".repeat(gutter_width);
        let spaces = " ".repeat(caret_col);
        let carets = "^".repeat(caret_len);
        let _ = std::fmt::Write::write_fmt(
            &mut snippet,
            format_args!("{padding} | {spaces}{carets}\n"),
        );
    }

    // Show 1 line after for context (if available)
    if let Some(next_line) = lines.get(line_idx + 1) {
        let _ = std::fmt::Write::write_fmt(
            &mut snippet,
            format_args!(
                "{:>width$} | {}\n",
                line + 1,
                next_line,
                width = gutter_width
            ),
        );
    }

    snippet
}

/// Derive contextual help text from an expected-token description.
fn unexpected_token_help(expected: &str) -> Option<String> {
    const HELP_TABLE: &[(&str, &str)] = &[
        ("then", "add 'then' after the condition: `if [ ... ]; then`"),
        (
            "do",
            "add 'do' after the loop header: `while [ ... ]; do` or `for x in ...; do`",
        ),
        ("fi", "every 'if' must be closed with 'fi'"),
        (
            "done",
            "every 'while'/'for'/'until' loop must be closed with 'done'",
        ),
        ("esac", "every 'case' must be closed with 'esac'"),
        (
            "in",
            "'for' and 'case' require 'in': `for var in list` / `case $x in`",
        ),
        ("}", "unmatched '{' — did you forget the closing '}'?"),
        (")", "unmatched '(' — did you forget the closing ')'?"),
    ];
    HELP_TABLE
        .iter()
        .find(|(keyword, _)| expected.contains(keyword))
        .map(|(_, help)| help.to_string())
}

/// Build a full Diagnostic from a `LexerError`.
pub(crate) fn lexer_error_diagnostic(
    lex_err: &LexerError,
    source: &str,
    file: Option<&str>,
) -> crate::models::diagnostic::Diagnostic {
    use crate::models::diagnostic::{Diagnostic, ErrorCategory};

    let (line, col) = match lex_err {
        LexerError::UnexpectedChar(_, l, c) | LexerError::UnterminatedString(l, c) => {
            (Some(*l), Some(*c))
        }
        LexerError::InvalidNumber(_) => (None, None),
    };
    let snippet = line.map(|l| build_snippet(source, l, col, 1));
    let help = match lex_err {
        LexerError::UnterminatedString(_, _) => {
            Some("close the string with a matching quote character".to_string())
        }
        LexerError::UnexpectedChar(ch, _, _) => {
            Some(format!("'{ch}' is not valid in this context"))
        }
        LexerError::InvalidNumber(s) => Some(format!("'{s}' is not a valid number")),
    };
    Diagnostic {
        error: format!("{lex_err}"),
        file: file.map(String::from),
        line,
        column: col,
        category: ErrorCategory::Syntax,
        note: None,
        help,
        snippet,
    }
}

/// Convert a ParseError into a rich Diagnostic for CLI display.
pub fn format_parse_diagnostic(
    error: &ParseError,
    source: &str,
    file: Option<&str>,
) -> crate::models::diagnostic::Diagnostic {
    use crate::models::diagnostic::{Diagnostic, ErrorCategory};

    match error {
        ParseError::UnexpectedToken {
            expected,
            found,
            line,
        } => {
            let snippet = build_snippet(source, *line, None, found.len().min(20));
            let help = unexpected_token_help(expected);
            Diagnostic {
                error: format!("expected {expected}, found {found}"),
                file: file.map(String::from),
                line: Some(*line),
                column: None,
                category: ErrorCategory::Syntax,
                note: Some(format!("the parser expected {expected} at this point")),
                help,
                snippet: Some(snippet),
            }
        }
        ParseError::UnexpectedEof => {
            let total_lines = source.lines().count();
            let snippet = build_snippet(source, total_lines, None, 1);
            Diagnostic {
                error: "unexpected end of file".to_string(),
                file: file.map(String::from),
                line: Some(total_lines),
                column: None,
                category: ErrorCategory::Syntax,
                note: Some(
                    "the file ended while the parser was still expecting more input".to_string(),
                ),
                help: Some(
                    "check for unclosed quotes, brackets, or missing keywords (fi, done, esac)"
                        .to_string(),
                ),
                snippet: Some(snippet),
            }
        }
        ParseError::InvalidSyntax(msg) => Diagnostic {
            error: msg.clone(),
            file: file.map(String::from),
            line: None,
            column: None,
            category: ErrorCategory::Syntax,
            note: None,
            help: None,
            snippet: None,
        },
        ParseError::LexerError(lex_err) => lexer_error_diagnostic(lex_err, source, file),
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    #![allow(clippy::expect_used)]
    use super::*;
    use crate::bash_parser::lexer::{LexerError, Token};
    use crate::bash_parser::parser::ParseError;

    // -----------------------------------------------------------------------
    // token_display — all Token variants
    // -----------------------------------------------------------------------

    #[test]
    fn test_token_display_identifier() {
        assert_eq!(token_display(&Token::Identifier("foo".into())), "'foo'");
    }

    #[test]
    fn test_token_display_string_short() {
        assert_eq!(token_display(&Token::String("hello".into())), "\"hello\"");
    }

    #[test]
    fn test_token_display_string_truncated() {
        let long = "a".repeat(50);
        let result = token_display(&Token::String(long));
        // Should truncate to first 30 chars
        assert!(result.len() < 40);
        assert!(result.starts_with('"'));
    }

    #[test]
    fn test_token_display_number() {
        assert_eq!(token_display(&Token::Number(42)), "'42'");
    }

    #[test]
    fn test_token_display_variable() {
        assert_eq!(token_display(&Token::Variable("HOME".into())), "'$HOME'");
    }

    #[test]
    fn test_token_display_assign() {
        assert_eq!(token_display(&Token::Assign), "'='");
    }

    #[test]
    fn test_token_display_semicolon() {
        assert_eq!(token_display(&Token::Semicolon), "';'");
    }

    #[test]
    fn test_token_display_pipe() {
        assert_eq!(token_display(&Token::Pipe), "'|'");
    }

    #[test]
    fn test_token_display_ampersand() {
        assert_eq!(token_display(&Token::Ampersand), "'&'");
    }

    #[test]
    fn test_token_display_parens() {
        assert_eq!(token_display(&Token::LeftParen), "'('");
        assert_eq!(token_display(&Token::RightParen), "')'");
    }

    #[test]
    fn test_token_display_braces() {
        assert_eq!(token_display(&Token::LeftBrace), "'{'");
        assert_eq!(token_display(&Token::RightBrace), "'}'");
    }

    #[test]
    fn test_token_display_brackets() {
        assert_eq!(token_display(&Token::LeftBracket), "'['");
        assert_eq!(token_display(&Token::RightBracket), "']'");
    }

    #[test]
    fn test_token_display_newline() {
        assert_eq!(token_display(&Token::Newline), "newline");
    }

    #[test]
    fn test_token_display_keywords() {
        assert_eq!(token_display(&Token::If), "'if'");
        assert_eq!(token_display(&Token::Then), "'then'");
        assert_eq!(token_display(&Token::Else), "'else'");
        assert_eq!(token_display(&Token::Elif), "'elif'");
        assert_eq!(token_display(&Token::Fi), "'fi'");
        assert_eq!(token_display(&Token::For), "'for'");
        assert_eq!(token_display(&Token::While), "'while'");
        assert_eq!(token_display(&Token::Until), "'until'");
        assert_eq!(token_display(&Token::Do), "'do'");
        assert_eq!(token_display(&Token::Done), "'done'");
        assert_eq!(token_display(&Token::Case), "'case'");
        assert_eq!(token_display(&Token::Esac), "'esac'");
        assert_eq!(token_display(&Token::In), "'in'");
        assert_eq!(token_display(&Token::Function), "'function'");
        assert_eq!(token_display(&Token::Return), "'return'");
        assert_eq!(token_display(&Token::Local), "'local'");
        assert_eq!(token_display(&Token::Export), "'export'");
    }

    #[test]
    fn test_token_display_dollar() {
        assert_eq!(token_display(&Token::Dollar), "'$'");
    }

    #[test]
    fn test_token_display_heredoc() {
        let tok = Token::Heredoc {
            delimiter: "EOF".into(),
            content: "stuff".into(),
        };
        assert_eq!(token_display(&tok), "heredoc '<<EOF'");
    }

    #[test]
    fn test_token_display_herestring() {
        let tok = Token::HereString("short".into());
        assert_eq!(token_display(&tok), "herestring '<<<short'");
    }

    #[test]
    fn test_token_display_herestring_truncated() {
        let long = "x".repeat(50);
        let result = token_display(&Token::HereString(long));
        // Should truncate content to 20 chars
        assert!(result.contains("<<<"));
        assert!(result.len() < 40);
    }

    #[test]
    fn test_token_display_command_substitution() {
        assert_eq!(
            token_display(&Token::CommandSubstitution("ls -la".into())),
            "'$(ls -la)'"
        );
    }

    #[test]
    fn test_token_display_arithmetic_expansion() {
        assert_eq!(
            token_display(&Token::ArithmeticExpansion("1+2".into())),
            "'$((1+2))'"
        );
    }

    #[test]
    fn test_token_display_comment() {
        assert_eq!(token_display(&Token::Comment("a note".into())), "comment");
    }

    #[test]
    fn test_token_display_fallback_debug() {
        // Tokens not in the match arms fall through to Debug format
        let result = token_display(&Token::Eof);
        assert!(result.contains("Eof"));
    }

    // -----------------------------------------------------------------------
    // expected_display — all branches
    // -----------------------------------------------------------------------

    #[test]
    fn test_expected_display_keywords() {
        assert_eq!(expected_display(&Token::Then), "'then' keyword");
        assert_eq!(expected_display(&Token::Do), "'do' keyword");
        assert_eq!(expected_display(&Token::Fi), "'fi' keyword");
        assert_eq!(expected_display(&Token::Done), "'done' keyword");
        assert_eq!(expected_display(&Token::Esac), "'esac' keyword");
        assert_eq!(expected_display(&Token::In), "'in' keyword");
    }

    #[test]
    fn test_expected_display_delimiters() {
        assert_eq!(expected_display(&Token::LeftBrace), "'{'");
        assert_eq!(expected_display(&Token::RightBrace), "'}'");
        assert_eq!(expected_display(&Token::LeftParen), "'('");
        assert_eq!(expected_display(&Token::RightParen), "')'");
        assert_eq!(expected_display(&Token::LeftBracket), "'['");
        assert_eq!(expected_display(&Token::RightBracket), "']'");
        assert_eq!(expected_display(&Token::Semicolon), "';'");
    }

    #[test]
    fn test_expected_display_fallback() {
        assert_eq!(expected_display(&Token::Eof), "token");
        assert_eq!(expected_display(&Token::Pipe), "token");
    }

    // -----------------------------------------------------------------------
    // suggest_fix — all branches
    // -----------------------------------------------------------------------

    #[test]
    fn test_suggest_fix_then_with_identifier() {
        let fix = suggest_fix(&Token::Then, Some(&Token::Identifier("echo".into())));
        assert!(fix.expect("should have suggestion").contains("then"));
    }

    #[test]
    fn test_suggest_fix_then_with_variable() {
        let fix = suggest_fix(&Token::Then, Some(&Token::Variable("x".into())));
        assert!(fix.expect("should have suggestion").contains("then"));
    }

    #[test]
    fn test_suggest_fix_then_with_other() {
        let fix = suggest_fix(&Token::Then, Some(&Token::Semicolon));
        assert!(fix
            .expect("should have suggestion")
            .contains("'if' requires 'then'"));
    }

    #[test]
    fn test_suggest_fix_then_with_none() {
        let fix = suggest_fix(&Token::Then, None);
        assert!(fix.is_some());
    }

    #[test]
    fn test_suggest_fix_do_with_identifier() {
        let fix = suggest_fix(&Token::Do, Some(&Token::Identifier("cmd".into())));
        assert!(fix.expect("should have suggestion").contains("do"));
    }

    #[test]
    fn test_suggest_fix_do_with_variable() {
        let fix = suggest_fix(&Token::Do, Some(&Token::Variable("v".into())));
        assert!(fix.expect("should have suggestion").contains("do"));
    }

    #[test]
    fn test_suggest_fix_do_with_other() {
        let fix = suggest_fix(&Token::Do, Some(&Token::Pipe));
        assert!(fix
            .expect("should have suggestion")
            .contains("loops require 'do'"));
    }

    #[test]
    fn test_suggest_fix_fi() {
        let fix = suggest_fix(&Token::Fi, None);
        assert!(fix.expect("should have suggestion").contains("fi"));
    }

    #[test]
    fn test_suggest_fix_done() {
        let fix = suggest_fix(&Token::Done, None);
        assert!(fix.expect("should have suggestion").contains("done"));
    }

    #[test]
    fn test_suggest_fix_right_brace() {
        let fix = suggest_fix(&Token::RightBrace, None);
        assert!(fix.expect("should have suggestion").contains("}"));
    }

    #[test]
    fn test_suggest_fix_right_paren() {
        let fix = suggest_fix(&Token::RightParen, None);
        assert!(fix.expect("should have suggestion").contains(")"));
    }

    #[test]
    fn test_suggest_fix_in() {
        let fix = suggest_fix(&Token::In, None);
        assert!(fix.expect("should have suggestion").contains("in"));
    }

    #[test]
    fn test_suggest_fix_no_match() {
        let fix = suggest_fix(&Token::Eof, None);
        assert!(fix.is_none());
    }

    // -----------------------------------------------------------------------
    // build_snippet — context, carets, edge cases
    // -----------------------------------------------------------------------

    #[test]
    fn test_build_snippet_single_line() {
        let source = "echo hello";
        let snippet = build_snippet(source, 1, Some(1), 4);
        assert!(snippet.contains("echo hello"), "Should contain the line");
        assert!(snippet.contains("^^^^"), "Should have carets");
    }

    #[test]
    fn test_build_snippet_with_context_before() {
        let source = "line1\nline2\nline3";
        let snippet = build_snippet(source, 2, Some(1), 3);
        assert!(snippet.contains("line1"), "Should have context before");
        assert!(snippet.contains("line2"), "Should have the error line");
        assert!(snippet.contains("line3"), "Should have context after");
    }

    #[test]
    fn test_build_snippet_first_line_no_context_before() {
        let source = "first line\nsecond line";
        let snippet = build_snippet(source, 1, Some(1), 1);
        // No previous line to show
        assert!(snippet.contains("first line"));
        assert!(snippet.contains("second line"), "Should have context after");
    }

    #[test]
    fn test_build_snippet_last_line_no_context_after() {
        let source = "first\nlast";
        let snippet = build_snippet(source, 2, Some(1), 1);
        assert!(snippet.contains("first"), "Should have context before");
        assert!(snippet.contains("last"), "Should have error line");
    }

    #[test]
    fn test_build_snippet_col_none_defaults_to_1() {
        let source = "echo hello";
        let snippet = build_snippet(source, 1, None, 3);
        assert!(snippet.contains("^^^"));
    }

    #[test]
    fn test_build_snippet_highlight_len_zero_uses_1() {
        let source = "echo hello";
        let snippet = build_snippet(source, 1, Some(3), 0);
        assert!(snippet.contains("^"), "Should have at least one caret");
    }

    #[test]
    fn test_build_snippet_line_zero_saturates() {
        // line=0 should saturate to 0 in saturating_sub(1), producing line_idx=0
        let source = "only line";
        let snippet = build_snippet(source, 0, Some(1), 1);
        // Should not panic — the function handles line=0 gracefully
        assert!(!snippet.is_empty());
    }

    #[test]
    fn test_build_snippet_column_offset() {
        let source = "echo hello world";
        let snippet = build_snippet(source, 1, Some(6), 5);
        // Caret should be offset 5 characters (col=6, sub 1 = 5 spaces)
        assert!(snippet.contains("^^^^^"));
    }

    // -----------------------------------------------------------------------
    // unexpected_token_help — all keyword matches
    // -----------------------------------------------------------------------

    #[test]
    fn test_unexpected_token_help_then() {
        let help = unexpected_token_help("expected 'then' keyword");
        assert!(help.is_some());
        assert!(help.unwrap().contains("then"));
    }

    #[test]
    fn test_unexpected_token_help_do() {
        let help = unexpected_token_help("expected 'do' keyword");
        assert!(help.is_some());
        assert!(help.unwrap().contains("do"));
    }

    #[test]
    fn test_unexpected_token_help_fi() {
        let help = unexpected_token_help("fi");
        assert!(help.is_some());
    }

    #[test]
    fn test_unexpected_token_help_done() {
        let help = unexpected_token_help("done");
        assert!(help.is_some());
    }

    #[test]
    fn test_unexpected_token_help_esac() {
        let help = unexpected_token_help("esac");
        assert!(help.is_some());
    }

    #[test]
    fn test_unexpected_token_help_in() {
        let help = unexpected_token_help("expected 'in' after variable");
        assert!(help.is_some());
    }

    #[test]
    fn test_unexpected_token_help_close_brace() {
        let help = unexpected_token_help("expected }");
        assert!(help.is_some());
    }

    #[test]
    fn test_unexpected_token_help_close_paren() {
        let help = unexpected_token_help("expected )");
        assert!(help.is_some());
    }

    #[test]
    fn test_unexpected_token_help_no_match() {
        let help = unexpected_token_help("xyz abc");
        assert!(help.is_none());
    }

    // -----------------------------------------------------------------------
    // lexer_error_diagnostic — all LexerError variants
    // -----------------------------------------------------------------------

    #[test]
    fn test_lexer_error_diagnostic_unexpected_char() {
        let err = LexerError::UnexpectedChar('@', 3, 7);
        let diag = lexer_error_diagnostic(&err, "line1\nline2\n@invalid", Some("test.sh"));
        assert_eq!(diag.line, Some(3));
        assert_eq!(diag.column, Some(7));
        assert!(diag.help.expect("should have help").contains("@"));
        assert_eq!(diag.file, Some("test.sh".to_string()));
        assert!(diag.snippet.is_some());
    }

    #[test]
    fn test_lexer_error_diagnostic_unterminated_string() {
        let err = LexerError::UnterminatedString(2, 5);
        let diag = lexer_error_diagnostic(&err, "x='hello\ny=1", None);
        assert_eq!(diag.line, Some(2));
        assert_eq!(diag.column, Some(5));
        assert!(diag.help.expect("should have help").contains("quote"));
        assert_eq!(diag.file, None);
    }

    #[test]
    fn test_lexer_error_diagnostic_invalid_number() {
        let err = LexerError::InvalidNumber("0xZZ".into());
        let diag = lexer_error_diagnostic(&err, "x=0xZZ", Some("file.sh"));
        assert_eq!(diag.line, None);
        assert_eq!(diag.column, None);
        assert!(diag.help.expect("should have help").contains("0xZZ"));
        assert!(diag.snippet.is_none());
    }

    // -----------------------------------------------------------------------
    // format_parse_diagnostic — all ParseError variants
    // -----------------------------------------------------------------------

    #[test]
    fn test_format_parse_diagnostic_unexpected_token() {
        let err = ParseError::UnexpectedToken {
            expected: "'then' keyword".into(),
            found: "echo".into(),
            line: 2,
        };
        let source = "if [ -f /tmp ]\necho hello";
        let diag = format_parse_diagnostic(&err, source, Some("script.sh"));

        assert!(diag.error.contains("expected"));
        assert!(diag.error.contains("echo"));
        assert_eq!(diag.line, Some(2));
        assert_eq!(diag.file, Some("script.sh".to_string()));
        assert!(diag.note.is_some());
        assert!(diag.snippet.is_some());
        // "then" in expected triggers help
        assert!(diag.help.is_some());
    }

    #[test]
    fn test_format_parse_diagnostic_unexpected_eof() {
        let source = "if [ -f /tmp ]; then\necho hello";
        let diag = format_parse_diagnostic(&ParseError::UnexpectedEof, source, None);

        assert_eq!(diag.error, "unexpected end of file");
        assert_eq!(diag.line, Some(2)); // total_lines
        assert!(diag.note.expect("should have note").contains("ended"));
        assert!(diag.help.expect("should have help").contains("unclosed"));
        assert!(diag.snippet.is_some());
    }

    #[test]
    fn test_format_parse_diagnostic_invalid_syntax() {
        let msg = "invalid heredoc syntax".to_string();
        let diag = format_parse_diagnostic(
            &ParseError::InvalidSyntax(msg.clone()),
            "<<OOPS",
            Some("test.sh"),
        );
        assert_eq!(diag.error, msg);
        assert_eq!(diag.line, None);
        assert!(diag.help.is_none());
        assert!(diag.snippet.is_none());
    }

    #[test]
    fn test_format_parse_diagnostic_lexer_error() {
        let lex_err = LexerError::UnexpectedChar('~', 1, 3);
        let diag =
            format_parse_diagnostic(&ParseError::LexerError(lex_err), "ab~cd", Some("lex.sh"));
        assert_eq!(diag.line, Some(1));
        assert_eq!(diag.column, Some(3));
    }

    #[test]
    fn test_format_parse_diagnostic_no_file() {
        let err = ParseError::InvalidSyntax("bad".into());
        let diag = format_parse_diagnostic(&err, "", None);
        assert_eq!(diag.file, None);
    }

    #[test]
    fn test_format_parse_diagnostic_unexpected_token_no_help() {
        // An expected string that doesn't match any help keyword
        // Note: unexpected_token_help uses contains(), so avoid substrings of
        // "then", "do", "fi", "done", "esac", "in", "}", ")"
        let err = ParseError::UnexpectedToken {
            expected: "xyz abc".into(),
            found: "x".into(),
            line: 1,
        };
        let diag = format_parse_diagnostic(&err, "x", None);
        assert!(diag.help.is_none());
    }
}
