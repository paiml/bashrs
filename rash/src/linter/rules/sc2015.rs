// SC2015: Note that A && B || C is not if-then-else. C may run when A is true.
//
// A common mistake is thinking `A && B || C` means "if A then B else C".
// It actually means "(A and B) or C". If B fails, C will run even if A succeeded.
//
// Examples:
// Bad:
//   test && success || failure      // failure runs if success fails
//   [[ -f file ]] && cat file || cleanup  // wrong logic
//   command && build || rollback  // misleading
//
// Good:
//   if test; then success; else failure; fi  // Clear if-then-else
//   if [[ -f file ]]; then cat file; else cleanup; fi
//   if command; then build; else rollback; fi
//
// Why it's wrong:
//   true && false || cleanup        // Runs even though true succeeded
//   The || sees the false from && and runs
//
// Note: Use proper if-then-else for conditional logic to avoid surprises.
//
// Exempt (shellcheck 0.8.0 parity): a fallback of echo/exit/return/printf, or a
// bare assignment, is not flagged.

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static AND_OR_PATTERN: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: command && command || command
    // Looking for pattern with both && and ||. `;` ends a chain, so a match never spans it.
    Regex::new(r"[^|&;]+&&[^|&;]+\|\|[^|&;]+").unwrap()
});

/// True when the fallback continues as a pipeline (`C | tee`, `C |& tee`), which
/// ShellCheck does not exempt. `rest` is the fallback text plus what follows the match.
fn fallback_is_pipeline(rest: &str) -> bool {
    let cleaned = rest
        .replace(">&", ">")
        .replace("<&", "<")
        .replace("&>", ">")
        .replace(">|", ">");
    let mut chars = cleaned.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            ';' | '&' => return false,
            '|' => return chars.peek() != Some(&'|'),
            _ => {}
        }
    }
    false
}

/// `NAME=value` (or `NAME+=value`): the part before `=` is a shell identifier.
fn is_assignment(word: &str) -> bool {
    let Some((name, _)) = word.split_once('=') else {
        return false;
    };
    let name = name.strip_suffix('+').unwrap_or(name);
    let mut chars = name.chars();
    chars
        .next()
        .is_some_and(|c| c == '_' || c.is_ascii_alphabetic())
        && chars.all(|c| c == '_' || c.is_ascii_alphanumeric())
}

/// Name of the command a fallback runs, skipping leading `VAR=value` words and
/// `command`/`builtin` prefixes, with quotes and any directory removed.
/// `None` when the fallback is only assignments.
/// Structure is read from `masked` (quoted text is `_`); the name is taken from `original`
/// at the same byte offset, so a quoted command such as `"echo"` keeps its name.
fn fallback_command(masked: &str, original: &str) -> Option<String> {
    let base = masked.as_ptr() as usize;
    let words: Vec<(usize, &str)> = masked
        .split_whitespace()
        .map(|w| (w.as_ptr() as usize - base, w))
        .collect();
    let mut i = 0;
    while i < words.len() && is_assignment(words[i].1) {
        i += 1;
    }
    let prefix_start = i;
    // Only one `command`/`builtin` prefix is skipped, as in ShellCheck 0.8.0.
    if i < words.len() && matches!(words[i].1, "command" | "builtin") {
        i += 1;
    }
    let Some(&(off, w)) = words.get(i) else {
        // A bare `command`/`builtin` runs that command, so it is not exempt.
        return (i > prefix_start).then(|| "command".to_string());
    };
    // Redirection is split on the masked word, so a `<` or `>` inside quotes is literal.
    let masked_word = w.split(['<', '>']).next().unwrap_or("");
    let word = original.get(off..off + masked_word.len())?;
    // `\echo` runs `echo`: a leading backslash is quote removal, as in ShellCheck.
    let word = word.strip_prefix('\\').unwrap_or(word);
    let word = word.trim_matches(['"', '\'']);
    Some(word.rsplit('/').next().unwrap_or("").to_string())
}

/// ShellCheck 0.8.0 does not flag `A && B || C` when C is a single error-reporting or
/// terminating command (the usual "A && B || die" idiom) or a bare assignment.
/// `masked` and `original` are the same chain (quotes masked, or not); `after` is masked.
fn fallback_is_exempt(masked: &str, original: &str, after: &str) -> bool {
    let fallback = masked.rsplit("||").next().unwrap_or("").trim_start();
    let original_fallback = &original[original.len() - fallback.len()..];
    if fallback_is_pipeline(&format!("{fallback}{after}")) {
        return false;
    }
    match fallback_command(fallback, original_fallback) {
        None => true,
        Some(cmd) => matches!(cmd.as_str(), "echo" | "exit" | "return" | "printf"),
    }
}

/// Copy of `line` with quoted text, `$( )` bodies and a trailing `#` comment masked by
/// `_` (same byte length), so `;`, `|` and `&` inside them cannot split or join a chain.
fn mask_quoted(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut mask = Mask::default();
    for c in line.chars() {
        mask.step(&mut out, c);
    }
    out
}

#[derive(Default)]
struct Mask {
    quote: Option<char>,
    depth: usize,
    comment: bool,
    escaped: bool,
}

impl Mask {
    fn step(&mut self, out: &mut String, c: char) {
        if self.comment {
            push_mask(out, c);
        } else if self.escaped {
            // The char after a backslash is literal: it cannot open, close or split anything.
            self.escaped = false;
            push_mask(out, c);
        } else if c == '\\' && self.quote != Some('\'') {
            // Backslash is literal inside single quotes only.
            self.escaped = true;
            out.push(c);
        } else if let Some(q) = self.quote {
            self.in_quote(out, q, c);
        } else if self.depth > 0 {
            self.in_subst(out, c);
        } else {
            self.plain(out, c);
        }
    }

    fn in_quote(&mut self, out: &mut String, quote: char, c: char) {
        if c == quote {
            self.quote = None;
            out.push(c);
        } else {
            push_mask(out, c);
        }
    }

    fn in_subst(&mut self, out: &mut String, c: char) {
        match c {
            '(' => self.depth += 1,
            ')' => self.depth -= 1,
            _ => {}
        }
        if c == ')' && self.depth == 0 {
            out.push(c);
        } else {
            push_mask(out, c);
        }
    }

    fn plain(&mut self, out: &mut String, c: char) {
        match c {
            '"' | '\'' => {
                self.quote = Some(c);
                out.push(c);
            }
            '(' if out.ends_with('$') => {
                self.depth = 1;
                out.push(c);
            }
            '#' if out.chars().last().is_none_or(char::is_whitespace) => {
                self.comment = true;
                push_mask(out, c);
            }
            _ => out.push(c),
        }
    }
}

fn push_mask(out: &mut String, c: char) {
    for _ in 0..c.len_utf8() {
        out.push('_');
    }
}

fn diagnostic_for(line_num: usize, start: usize, end: usize) -> Diagnostic {
    Diagnostic::new(
        "SC2015",
        Severity::Info,
        "Note that 'A && B || C' is not if-then-else. C may run when A is true. Use 'if A; then B; else C; fi' for proper conditional logic".to_string(),
        Span::new(line_num, start + 1, line_num, end + 1),
    )
}

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            continue;
        }

        // Skip if line is inside a comment or string
        // Simple heuristic: skip lines that start with if/elif/while (proper control flow)
        if trimmed.starts_with("if ")
            || trimmed.starts_with("elif ")
            || trimmed.starts_with("while ")
        {
            continue;
        }

        // Look for && followed by ||
        // Examine every chain on the line; one exempt chain must not hide another.
        // Matching runs on a masked copy so quoted or commented text cannot split a chain.
        let masked = mask_quoted(line);
        let mut pos = 0;
        while let Some(m) = AND_OR_PATTERN.find(&masked[pos..]) {
            let (start, end) = (pos + m.start(), pos + m.end());
            let matched = &masked[start..end];
            if !fallback_is_exempt(matched, &line[start..end], &masked[end..]) {
                result.add(diagnostic_for(line_num, start, end));
                pos = end;
            } else {
                // Resume at the last `||`, so a following `&& ... || ...` chain is still examined.
                pos = start + matched.rfind("||").unwrap_or(matched.len() - 1) + 2;
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2015_and_or_pattern() {
        let code = r#"test && success || failure"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2015");
        assert_eq!(result.diagnostics[0].severity, Severity::Info);
        assert!(result.diagnostics[0].message.contains("not if-then-else"));
    }

    #[test]
    fn test_sc2015_file_check() {
        let code = r#"[[ -f file ]] && cat file || handle_missing"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2015_command_check() {
        let code = r#"command && echo "ok" || cleanup"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2015_complex_commands() {
        let code = r#"grep pattern file && process data || handle_error"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2015_if_statement_ok() {
        let code = r#"if test; then success; else failure; fi"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2015_if_with_and_ok() {
        let code = r#"if command && other; then action; fi"#;
        let result = check(code);
        // Inside if statement, && || is fine
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2015_only_and_ok() {
        let code = r#"test && success && another"#;
        let result = check(code);
        // Only &&, no ||
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2015_only_or_ok() {
        let code = r#"test || fallback || default"#;
        let result = check(code);
        // Only ||, no &&
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2015_multiple_issues() {
        let code = r#"
check1 && ok1 || fail1
check2 && ok2 || fail2
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2015_parity_echo_exit_return_printf_fallback_exempt() {
        // shellcheck 0.8.0 does not flag these fallbacks (error-handling idiom)
        for c in [
            "foo && bar || echo b",
            "foo && bar || exit 1",
            "foo && bar || return 1",
            "foo && bar || printf x",
            "[ -f x ] && echo a || echo b >&2",
        ] {
            assert_eq!(check(c).diagnostics.len(), 0, "{c}");
        }
    }

    #[test]
    fn test_sc2015_parity_other_fallback_still_flagged() {
        for c in [
            "foo && bar || true",
            "foo && bar || :",
            "foo && bar || cd d",
            "foo && bar || continue",
        ] {
            assert_eq!(check(c).diagnostics.len(), 1, "{c}");
        }
    }

    #[test]
    fn test_sc2015_pipeline_fallback_still_flagged() {
        // shellcheck 0.8.0 flags these: the fallback is a pipeline, not a single command
        for c in [
            "foo && bar || echo b | tee log",
            "foo && bar || echo b 2>&1 | tee log",
            "foo && bar || echo b |& tee log",
            "foo && bar || exit | x",
        ] {
            assert_eq!(check(c).diagnostics.len(), 1, "{c}");
        }
    }

    #[test]
    fn test_sc2015_list_fallback_exempt() {
        // shellcheck 0.8.0 does not flag when the exempt command is followed by & / && / ||
        for c in [
            "foo && bar || echo b &",
            "foo && bar || echo b && baz",
            "foo && bar || echo b || baz",
        ] {
            assert_eq!(check(c).diagnostics.len(), 0, "{c}");
        }
    }

    #[test]
    fn test_sc2015_every_chain_on_line_examined() {
        assert_eq!(
            check("foo && bar || echo ok; baz && qux || rm -f x")
                .diagnostics
                .len(),
            1
        );
        assert_eq!(
            check("foo && bar || echo ok; baz && qux || echo z")
                .diagnostics
                .len(),
            0
        );
        assert_eq!(check("a && b || c1; d && e || c2").diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2015_parity_extra_exempt_forms() {
        for c in [
            "foo && bar || /bin/echo b",
            "foo && bar || \"echo\" b",
            "foo && bar || echo>&2 b",
            "foo && bar || command echo b",
            "foo && bar || builtin echo b",
            "foo && bar || X=1",
            "foo && bar || X=1 echo b",
        ] {
            assert_eq!(check(c).diagnostics.len(), 0, "{c}");
        }
    }

    #[test]
    fn test_sc2015_extra_forms_not_over_exempt() {
        for c in [
            "foo && bar || X=1 true",
            "foo && bar || command cleanup",
            "foo && bar || /bin/cleanup",
            "foo && bar || cleanup &",
            "foo && bar || cleanup | tee",
        ] {
            assert_eq!(check(c).diagnostics.len(), 1, "{c}");
        }
    }

    #[test]
    fn test_sc2015_quote_aware_chain_flagged() {
        // ';' or '|' inside quotes or $(...) does not end the chain (shellcheck 0.8.0 flags these)
        for c in [
            r#"foo && echo "a;b" || cleanup"#,
            "foo && x=$(a; b) || cleanup",
            r#"foo && bar || MSG="a echo" cleanup"#,
            r#"foo && bar || echo "a;b" | tee log"#,
            "foo && bar || echo ok && baz || cleanup",
        ] {
            assert!(!check(c).diagnostics.is_empty(), "{c}");
        }
    }

    #[test]
    fn test_sc2015_quoted_pipe_and_noclobber_exempt() {
        // shellcheck 0.8.0 exempts these: the '|' is quoted or is '>|' redirection
        for c in [
            r#"foo && bar || echo "a | b""#,
            "foo && bar || echo 'x|y'",
            "foo && bar || echo err >| /tmp/err.log",
            r#"foo && bar || X="a b" echo hi"#,
        ] {
            assert_eq!(check(c).diagnostics.len(), 0, "{c}");
        }
    }

    #[test]
    fn test_sc2015_non_assignment_and_bare_prefix_flagged() {
        // '=' inside a non-assignment word, and a bare command/builtin, are not exempt
        for c in [
            "foo && bar || ((count+=1))",
            "foo && bar || ./script=1",
            "foo && bar || command",
            "foo && bar || builtin",
        ] {
            assert_eq!(check(c).diagnostics.len(), 1, "{c}");
        }
    }

    #[test]
    fn test_sc2015_backslash_escape_chain_flagged() {
        // A backslash escapes the next char: `\"` and `\'` do not open quotes, `\;` does not split
        for c in [
            r#"foo && echo "a\";b" || cleanup"#,
            r#"foo && echo \" || cleanup"#,
            r#"foo && echo \' || cleanup"#,
            r#"foo && echo a\;b || cleanup"#,
        ] {
            assert_eq!(check(c).diagnostics.len(), 1, "{c}");
        }
    }

    #[test]
    fn test_sc2015_backslash_escape_not_split_exempt() {
        // An escaped `|` is literal, not a pipeline; `\echo` is the command echo
        for c in [
            r#"foo && bar || echo "a\" | b""#,
            r#"foo && bar || echo a\|b"#,
            r#"foo && bar || \echo b"#,
        ] {
            assert_eq!(check(c).diagnostics.len(), 0, "{c}");
        }
    }

    #[test]
    fn test_sc2015_quoted_redirect_char_flagged() {
        // '<' or '>' inside quotes is literal: the quoted word is not `echo` with a redirect
        for c in [
            r#"foo && bar || "echo>foo""#,
            "foo && bar || 'echo>foo'",
            r#"foo && bar || "echo<foo""#,
        ] {
            assert_eq!(check(c).diagnostics.len(), 1, "{c}");
        }
    }

    #[test]
    fn test_sc2015_unquoted_redirect_still_exempt() {
        for c in ["foo && bar || echo>foo", r#"foo && bar || "echo">foo"#] {
            assert_eq!(check(c).diagnostics.len(), 0, "{c}");
        }
    }

    #[test]
    fn test_sc2015_stacked_prefix_flagged() {
        // shellcheck 0.8.0 strips only one command/builtin prefix, so a second one is the command
        for c in [
            "foo && bar || command command echo hi",
            "foo && bar || command builtin echo hi",
            "foo && bar || builtin command echo hi",
        ] {
            assert_eq!(check(c).diagnostics.len(), 1, "{c}");
        }
    }

    #[test]
    fn test_sc2015_single_prefix_still_exempt() {
        for c in [
            "foo && bar || command echo hi",
            "foo && bar || builtin echo hi",
        ] {
            assert_eq!(check(c).diagnostics.len(), 0, "{c}");
        }
    }

    #[test]
    fn test_sc2015_in_function() {
        let code = r#"
my_func() {
    validate && process || error
}
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
