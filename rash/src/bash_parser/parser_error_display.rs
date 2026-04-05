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
