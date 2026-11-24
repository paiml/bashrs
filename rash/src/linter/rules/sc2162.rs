//! SC2162: read without -r will mangle backslashes
//!
//! # Examples
//!
//! Bad:
//! ```bash
//! read line
//! ```
//!
//! Good:
//! ```bash
//! read -r line
//! ```
//!
//! # Rationale
//!
//! Without -r flag, read treats backslashes as escape characters:
//! - `\n` becomes `n` (not newline)
//! - `\\` becomes `\`
//! - Unexpected behavior for most use cases
//!
//! Always use `read -r` unless you specifically need backslash processing.
//!
//! # Auto-fix
//!
//! Add -r flag to read command

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};
use regex::Regex;

/// Check for read without -r flag
static PATTERN: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"\bread\s+([A-Za-z_][A-Za-z0-9_]*)").unwrap());

/// Options of `read` that consume the next word as their argument.
const ARG_OPTS: &str = "pnNdtuia";

/// Split the text after `read` into words, stopping at a command terminator.
/// Quoted sections are kept inside their word.
fn split_words(rest: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut word = String::new();
    let mut quote: Option<char> = None;
    for c in rest.chars() {
        if quote.is_some() {
            if Some(c) == quote {
                quote = None;
            }
            word.push(c);
        } else if c == '"' || c == '\'' {
            quote = Some(c);
            word.push(c);
        } else if c.is_whitespace() || ";&|<>".contains(c) {
            if !word.is_empty() {
                words.push(std::mem::take(&mut word));
            }
            if !c.is_whitespace() {
                return words;
            }
        } else {
            word.push(c);
        }
    }
    if !word.is_empty() {
        words.push(word);
    }
    words
}

/// True when the leading option words of a `read` include `-r`.
fn options_have_r(words: &[String]) -> bool {
    let mut expect_arg = false;
    let mut has_r = false;
    for w in words {
        if std::mem::take(&mut expect_arg) {
            continue;
        }
        let flags = match w.strip_prefix('-') {
            Some(f) if !f.is_empty() && !f.starts_with('-') => f,
            _ => break,
        };
        has_r |= flags.contains('r');
        expect_arg = flags.chars().last().is_some_and(|c| ARG_OPTS.contains(c));
    }
    has_r
}

/// Flags `read` commands whose first word is an option (`read -p "x" var`),
/// which the plain `read VAR` pattern misses, when they lack `-r`.
fn check_read_with_options(line: &str, line_num: usize, result: &mut LintResult) {
    static START: std::sync::LazyLock<Regex> =
        std::sync::LazyLock::new(|| Regex::new(r"\bread\s+-").unwrap());
    for m in START.find_iter(line) {
        if options_have_r(&split_words(&line[m.end() - 1..])) {
            continue;
        }
        let start = m.start() + 1;
        result.add(
            Diagnostic::new(
                "SC2162",
                Severity::Info,
                "read without -r will mangle backslashes",
                Span::new(line_num, start, line_num, start + 4),
            )
            .with_fix(Fix::new("read -r".to_string())),
        );
    }
}

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Pattern: read (without -r) followed by variable name
    let pattern = &*PATTERN;

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        check_read_with_options(line, line_num, &mut result);

        // Skip if line contains "read -r"
        if line.contains("read -r") {
            continue;
        }

        for cap in pattern.captures_iter(line) {
            let full_match = cap.get(0).unwrap();
            let var_name = cap.get(1).unwrap().as_str();

            let start_col = full_match.start() + 1;
            let end_col = full_match.end() + 1;

            let fix_text = format!("read -r {}", var_name);

            let diagnostic = Diagnostic::new(
                "SC2162",
                Severity::Info,
                "read without -r will mangle backslashes",
                Span::new(line_num, start_col, line_num, end_col),
            )
            .with_fix(Fix::new(fix_text));

            result.add(diagnostic);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2162_basic_detection() {
        let script = "read line";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2162");
    }

    #[test]
    fn test_sc2162_autofix() {
        let script = "read line";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].fix.is_some());
        assert_eq!(
            result.diagnostics[0].fix.as_ref().unwrap().replacement,
            "read -r line"
        );
    }

    #[test]
    fn test_sc2162_in_while_loop() {
        let script = "while read line; do echo \"$line\"; done";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2162_multiple_variables() {
        let script = "read var1 var2";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1); // Only matches first var
    }

    #[test]
    fn test_sc2162_false_positive_with_r_flag() {
        let script = "read -r line";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2162_false_positive_in_comment() {
        let script = "# read line";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2162_with_prompt() {
        let script = "read input";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2162_underscore_variable() {
        let script = "read user_input";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2162_in_function() {
        let script = "get_input() { read value; }";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2162_with_ifs() {
        let script = "IFS= read line";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2162_read_p_prompt() {
        for s in [
            "read -p \"x\" var",
            "read -s -p \"x\" var",
            "read -p x var",
            "read -n 1 var",
        ] {
            let r = check(s);
            assert_eq!(r.diagnostics.len(), 1, "{s}");
            assert_eq!(r.diagnostics[0].code, "SC2162");
        }
    }

    #[test]
    fn test_sc2162_read_p_with_r_not_flagged() {
        for s in [
            "read -r -p \"x\" var",
            "read -rp \"x\" var",
            "read -p \"x\" -r var",
            "read -s -r var",
        ] {
            assert_eq!(check(s).diagnostics.len(), 0, "{s}");
        }
    }
}
