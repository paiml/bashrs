// SC2075: Escaping quotes in quotes won't work. Use \\' or \"'\"
//
// In single quotes, nothing can be escaped - not even backslashes.
// To include a single quote in a single-quoted string, you must end the string,
// add an escaped quote, and start a new string.
//
// Examples:
// Incorrect:
//   echo 'can\'t'              // Won't work - backslash is literal
//   msg='it\'s wrong'          // Syntax error
//
// Correct:
//   echo 'can'\''t'            // End string, escaped quote, new string
//   echo "can't"               // Use double quotes instead
//   msg='it'"'"'s fixed'       // End, quote in double quotes, continue
//
// Impact: Syntax errors, incorrect string values
//
// Not an escape attempt (GH-439): `tr -d '\\'` and `tr -d '\'` are complete
// strings, because inside '...' a backslash is literal and the next quote
// closes. The attempt is a `\'` that runs straight into a word, as in
// 'it\'s'. Each line is scanned with its quoting context, so a `\'` inside
// "...", $'...', a comment or unquoted code is not read as one, and a string
// is never paired with the next quoted string on the line.
//
// The scan reads an attempt the way its author meant it, as an escaped quote
// that does not close the string, so a second attempt later on the line is
// still found ('don\'t' 'won\'t' reports twice). linter::quoting reads the
// line as the shell does, closing at that `\'`, and from there on it is out
// of step with the author, so this rule does not use it.

use crate::linter::{Diagnostic, LintResult, Severity, Span};

/// The quoting context outside any single-quoted string.
#[derive(Clone, Copy, PartialEq)]
enum Ctx {
    /// Unquoted code at the top of the line.
    Code,
    /// Unquoted code inside `$( )`; a `)` at this level closes it.
    Paren,
    /// Unquoted code inside backticks.
    Backtick,
    /// Inside `"..."`.
    Double,
}

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        for (start, end) in escape_attempts(line.as_bytes()) {
            let diagnostic = Diagnostic::new(
                "SC2075",
                Severity::Error,
                "Escaping a single quote in single quotes won't work. Use '\"'\"' or double quotes"
                    .to_string(),
                Span::new(line_num, start + 1, line_num, end + 1),
            );

            result.add(diagnostic);
        }
    }

    result
}

/// Byte ranges of the single-quoted strings on `line` that try to escape a
/// quote, from the opening quote to the quote the author meant to close with.
fn escape_attempts(line: &[u8]) -> Vec<(usize, usize)> {
    let mut found = Vec::new();
    let mut stack = vec![Ctx::Code];
    let mut i = 0;
    while i < line.len() {
        let ctx = stack.last().copied().unwrap_or(Ctx::Code);
        if ctx == Ctx::Double {
            i = step_double(line, i, &mut stack);
            continue;
        }
        match step_code(line, i, ctx, &mut stack, &mut found) {
            Some(next) => i = next,
            None => break, // a comment runs to the end of the line
        }
    }
    found
}

/// One step in unquoted code. `None` when a comment starts at `i`.
fn step_code(
    line: &[u8],
    i: usize,
    ctx: Ctx,
    stack: &mut Vec<Ctx>,
    found: &mut Vec<(usize, usize)>,
) -> Option<usize> {
    match line[i] {
        b'\\' => Some(i + 2),
        b'#' if starts_word(line, i) => None,
        b'\'' => Some(single_quoted(line, i, found)),
        b'$' if line.get(i + 1) == Some(&b'\'') => Some(skip_ansi_c(line, i + 2)),
        _ => Some(step_nesting(line, i, ctx, stack)),
    }
}

/// Opens or closes `$( )`, backticks and `"..."` from unquoted code.
fn step_nesting(line: &[u8], i: usize, ctx: Ctx, stack: &mut Vec<Ctx>) -> usize {
    match (line[i], ctx) {
        (b'$', _) if line.get(i + 1) == Some(&b'(') => {
            stack.push(Ctx::Paren);
            i + 2
        }
        (b'"', _) => {
            stack.push(Ctx::Double);
            i + 1
        }
        (b'`', Ctx::Backtick) | (b')', Ctx::Paren) => {
            stack.pop();
            i + 1
        }
        (b'`', _) => {
            stack.push(Ctx::Backtick);
            i + 1
        }
        (b'(', Ctx::Paren) => {
            stack.push(Ctx::Paren);
            i + 1
        }
        _ => i + 1,
    }
}

/// One step inside `"..."`, where a single quote is text.
fn step_double(line: &[u8], i: usize, stack: &mut Vec<Ctx>) -> usize {
    match line[i] {
        b'\\' => i + 2,
        b'"' => {
            stack.pop();
            i + 1
        }
        b'$' if line.get(i + 1) == Some(&b'(') => {
            stack.push(Ctx::Paren);
            i + 2
        }
        b'`' => {
            stack.push(Ctx::Backtick);
            i + 1
        }
        _ => i + 1,
    }
}

/// Scans the single-quoted string whose opening quote is at `open` and
/// returns the index after the quote that closes it. A backslash is literal
/// and the next quote closes, except a `\'` followed by a word character:
/// that was meant as an escaped quote, so it is recorded and scanning goes
/// on the way the author meant it.
fn single_quoted(line: &[u8], open: usize, found: &mut Vec<(usize, usize)>) -> usize {
    let mut attempt = false;
    let mut j = open + 1;
    while j < line.len() {
        if line[j] == b'\'' {
            j += 1;
            break;
        }
        if line[j] == b'\\' && line.get(j + 1) == Some(&b'\'') && is_word_byte(line.get(j + 2)) {
            attempt = true;
            j += 2;
        } else {
            j += 1;
        }
    }
    if attempt {
        found.push((open, j));
    }
    j
}

/// Skips the body of `$'...'`, where `\'` is an escape, from `j` to after
/// its closing quote.
fn skip_ansi_c(line: &[u8], mut j: usize) -> usize {
    while j < line.len() {
        match line[j] {
            b'\\' => j += 2,
            b'\'' => return j + 1,
            _ => j += 1,
        }
    }
    line.len()
}

/// A `#` starts a comment only at the start of a word.
fn starts_word(line: &[u8], i: usize) -> bool {
    i == 0
        || matches!(
            line[i - 1],
            b' ' | b'\t' | b';' | b'&' | b'|' | b'(' | b'<' | b'>'
        )
}

/// A letter, digit or underscore: a closing quote followed by one of these
/// runs into the same word.
fn is_word_byte(b: Option<&u8>) -> bool {
    b.is_some_and(|b| b.is_ascii_alphanumeric() || *b == b'_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2075_escaped_quote() {
        let code = r#"echo 'can\'t'"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2075_its() {
        let code = r#"msg='it\'s broken'"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2075_double_quotes_ok() {
        let code = r#"echo "can't""#;
        let result = check(code);
        // Double quotes allow single quotes
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2075_concatenation_ok() {
        let code = r#"echo 'can'"'"'t'"#;
        let result = check(code);
        // Correct workaround (ends string, quotes, starts new)
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2075_plain_string_ok() {
        let code = r#"echo 'hello world'"#;
        let result = check(code);
        // No escaped quotes
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2075_comment_ok() {
        let code = r#"# echo 'can\'t'"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2075_double_escape() {
        let code = r#"path='C:\\Users\\file'"#;
        let result = check(code);
        // Backslashes but no quote
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2075_multiple() {
        let code = r#"echo 'don\'t' 'won\'t'"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2075_in_command_sub() {
        let code = r#"result=$(echo 'can\'t')"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2075_escaped_backslash() {
        let code = r#"path='some\\path'"#;
        let result = check(code);
        // Backslash not escaping a quote
        assert_eq!(result.diagnostics.len(), 0);
    }

    // GH-439: inside '...' a backslash is literal and the next quote closes
    // the string, so `'\\'` and `'\'` are complete strings. Another quoted
    // string later on the line is a separate word, not the rest of this one.

    #[test]
    fn test_PMAT439_sc2075_backslashes_then_another_quoted_string() {
        let code = r#"echo 'a\b' | tr -d '\\' | grep -x 'ab'"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_PMAT439_sc2075_lone_backslash_then_another_quoted_string() {
        let code = r#"echo a | tr -d '\' | grep -x 'a'"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_PMAT439_sc2075_same_pipeline_inside_quoted_substitution() {
        let code = r#"x="$(echo 'a\b' | tr -d '\\' | grep -x 'ab')""#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_PMAT439_sc2075_issue_controls_stay_clean() {
        for code in [r#"echo a | tr -d '\\'"#, r#"x='\\'"#, r#"x='\'"#] {
            assert_eq!(check(code).diagnostics.len(), 0, "{code}");
        }
    }

    #[test]
    fn test_PMAT439_sc2075_backslash_string_then_a_separate_word() {
        for code in [
            r#"printf '%s\n' '\' 'x'"#,
            r#"sed 's/\\/\//g' '\' > out"#,
            r#"a='\';b='x'"#,
            r#"[ "$c" = '\' ] && echo 'x'"#,
            r#"echo 'a\'"b""#,
            r#"echo 'a\'$x"#,
            r#"echo 'a\''b'"#,
            r#"echo '\' # 'x'"#,
        ] {
            assert_eq!(check(code).diagnostics.len(), 0, "{code}");
        }
    }

    #[test]
    fn test_PMAT439_sc2075_not_single_quoted_context() {
        for code in [
            r#"echo "it\'s""#,
            r#"echo $'it\'s'"#,
            r#"echo it\'s"#,
            r#"echo "a 'b\' c' d""#,
        ] {
            assert_eq!(check(code).diagnostics.len(), 0, "{code}");
        }
    }

    #[test]
    fn test_PMAT439_sc2075_true_positive_still_fires() {
        let result = check(r#"echo 'it\'s'"#);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].severity, Severity::Error);
        // A backslash string that closes and runs into a word is the same
        // attempt even after an earlier, valid backslash string.
        assert_eq!(check(r#"tr -d '\\' && echo 'it\'s'"#).diagnostics.len(), 1);
        assert_eq!(check(r#"echo 'a\'1"#).diagnostics.len(), 1);
    }
}
