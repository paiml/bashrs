// SC2015: Note that A && B || C is not if-then-else. C may run when A is true.
//
// A common mistake is thinking `A && B || C` means "if A then B else C".
// It actually means "(A and B) or C". If B fails, C will run even if A succeeded.
//
// Examples:
// Bad:
//   test && success || failure      // failure runs if success fails
//   [[ -f file ]] && cat file || echo "missing"  // wrong logic
//   command && echo "ok" || echo "fail"  // misleading
//
// Good:
//   if test; then success; else failure; fi  // Clear if-then-else
//   if [[ -f file ]]; then cat file; else echo "missing"; fi
//   if command; then echo "ok"; else echo "fail"; fi
//
// Why it's wrong:
//   true && false || echo "runs"    // Runs even though true succeeded
//   The || sees the false from && and runs
//
// Note: Use proper if-then-else for conditional logic to avoid surprises.

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

/// `NAME=value`, `NAME+=value`, `NAME[sub]=value` or `NAME[sub]+=value`: the name
/// before `=` (with any `+` and `[subscript]` removed) is a shell identifier.
fn is_assignment(word: &str) -> bool {
    let Some((name, _)) = word.split_once('=') else {
        return false;
    };
    let name = name.strip_suffix('+').unwrap_or(name);
    let base = match name.split_once('[') {
        None => name,
        Some((base, _)) if name.ends_with(']') => base,
        Some(_) => return false,
    };
    is_identifier(base)
}

fn is_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    chars
        .next()
        .is_some_and(|c| c == '_' || c.is_ascii_alphabetic())
        && chars.all(|c| c == '_' || c.is_ascii_alphanumeric())
}

/// Words of a fallback as `(masked, original)` pairs at the same byte offset.
fn fallback_words<'a>(masked: &'a str, original: &'a str) -> Vec<(&'a str, &'a str)> {
    let base = masked.as_ptr() as usize;
    masked
        .split_whitespace()
        .map(|w| {
            let off = w.as_ptr() as usize - base;
            (w, original.get(off..off + w.len()).unwrap_or(""))
        })
        .collect()
}

/// Index just past the assignment word at `i`. An array `X=( ... )` spans words up to
/// the one that ends in `)`.
fn skip_assignment(words: &[(&str, &str)], i: usize) -> usize {
    let value = words[i].0.split_once('=').map_or("", |(_, v)| v);
    let mut next = i + 1;
    if value.starts_with('(') && !value.ends_with(')') {
        while next < words.len() && !words[next - 1].0.ends_with(')') {
            next += 1;
        }
    }
    next
}

/// A redirection word: `>x`, `2>&1`, `&>f`, or a bare `>` / `2>` whose target is the next word.
fn is_redirection(word: &str) -> bool {
    let rest = word.trim_start_matches(|c: char| c.is_ascii_digit());
    rest.starts_with(['<', '>']) || word.starts_with("&>")
}

/// True when a redirection's target is the following word (`>`, `2>`, `>&`).
fn redirect_takes_next(word: &str) -> bool {
    word.trim_start_matches(|c: char| c.is_ascii_digit())
        .trim_start_matches(['<', '>', '&', '|'])
        .is_empty()
}

/// Command name of one original word: quotes, redirection and directory removed.
fn command_name(original_word: &str) -> String {
    let word = original_word.split(['<', '>']).next().unwrap_or("");
    let word = word.trim_matches(['"', '\'']);
    word.rsplit('/').next().unwrap_or("").to_string()
}

/// Name of the command a fallback runs, skipping leading assignments, redirections and
/// `command`/`builtin` prefixes. `None` when the fallback has no command word.
/// Structure is read from `masked` (quoted text is `_`); names come from `original`.
fn fallback_command(masked: &str, original: &str) -> Option<String> {
    let words = fallback_words(masked, original);
    let mut i = 0;
    let mut saw_prefix = false;
    while i < words.len() {
        let (m, o) = words[i];
        if is_assignment(m) {
            i = skip_assignment(&words, i);
        } else if is_redirection(m) {
            i += if redirect_takes_next(m) { 2 } else { 1 };
        } else if matches!(command_name(o).as_str(), "command" | "builtin") {
            saw_prefix = true;
            i += 1;
        } else {
            return Some(command_name(o));
        }
    }
    // A bare `command`/`builtin` runs that command, so it is not exempt.
    saw_prefix.then(|| "command".to_string())
}

/// The fallback command text: from the start of `tail` (just after the chain's last `||`)
/// up to the next `;`, or up to the `)` that closes a subshell around the chain.
fn fallback_body(tail: &str) -> &str {
    let mut depth = 0usize;
    for (idx, c) in tail.char_indices() {
        match c {
            ';' => return &tail[..idx],
            '(' => depth += 1,
            ')' if depth == 0 => return &tail[..idx],
            ')' => depth -= 1,
            _ => {}
        }
    }
    tail
}

/// ShellCheck 0.8.0 does not flag `A && B || C` when C is a single error-reporting or
/// terminating command (the usual "A && B || die" idiom) or a bare assignment.
/// `masked_tail` and `original_tail` are the line from just after the chain's `||`
/// (quotes masked, or not).
fn fallback_is_exempt(masked_tail: &str, original_tail: &str) -> bool {
    let body = fallback_body(masked_tail);
    let lead = body.len() - body.trim_start().len();
    let (masked, original) = (&body[lead..], &original_tail[lead..body.len()]);
    if fallback_is_pipeline(masked) {
        return false;
    }
    match fallback_command(masked, original) {
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
}

impl Mask {
    fn step(&mut self, out: &mut String, c: char) {
        if self.comment {
            push_mask(out, c);
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
            // The fallback starts after the chain's last `||`; the match itself is a prefix of it.
            let fallback = start + masked[start..end].rfind("||").unwrap_or(0) + 2;
            if !fallback_is_exempt(&masked[fallback..], &line[fallback..]) {
                result.add(diagnostic_for(line_num, start, end));
                pos = end;
            } else {
                // Resume at the last `||`, so a following `&& ... || ...` chain is still examined.
                pos = fallback;
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
            assert!(check(c).diagnostics.len() >= 1, "{c}");
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
    fn test_sc2015_leading_redirect_exempt() {
        // shellcheck 0.8.0 skips leading redirections before the exempt command
        for c in [
            "foo && bar || >&2 echo hi",
            "foo && bar || 2>&1 echo hi",
            "foo && bar || >err echo hi",
            "foo && bar || 2>/dev/null echo hi",
            "foo && bar || > err echo hi",
        ] {
            assert_eq!(check(c).diagnostics.len(), 0, "{c}");
        }
    }

    #[test]
    fn test_sc2015_leading_redirect_not_over_exempt() {
        for c in [
            "foo && bar || >err cleanup",
            "foo && bar || 2>&1 cleanup",
            "foo && bar || > err cleanup",
        ] {
            assert_eq!(check(c).diagnostics.len(), 1, "{c}");
        }
    }

    #[test]
    fn test_sc2015_subshell_fallback_exempt() {
        // shellcheck 0.8.0 reads the fallback up to the ')' that closes the subshell
        for c in [
            "(foo && bar || exit)",
            "(foo && bar || echo)",
            "(foo && bar || echo a) | tee log",
        ] {
            assert_eq!(check(c).diagnostics.len(), 0, "{c}");
        }
    }

    #[test]
    fn test_sc2015_subshell_fallback_not_over_exempt() {
        for c in [
            "(foo && bar || cleanup)",
            "(foo && bar || echo b | tee log)",
        ] {
            assert_eq!(check(c).diagnostics.len(), 1, "{c}");
        }
    }

    #[test]
    fn test_sc2015_array_assignment_fallback_exempt() {
        for c in [
            "foo && bar || X=(1 2)",
            "foo && bar || X=( 1 2 )",
            "foo && bar || X=()",
            "foo && bar || arr[0]=1",
            "foo && bar || arr[$i]+=(1 2)",
        ] {
            assert_eq!(check(c).diagnostics.len(), 0, "{c}");
        }
    }

    #[test]
    fn test_sc2015_array_assignment_not_over_exempt() {
        for c in [
            "foo && bar || X=(1 2) cleanup",
            "foo && bar || arr[0]=1 cleanup",
            "foo && bar || arr[0]x=1 cleanup",
        ] {
            assert_eq!(check(c).diagnostics.len(), 1, "{c}");
        }
    }

    #[test]
    fn test_sc2015_quoted_command_prefix_exempt() {
        // a quoted command word is still the command: "command" and "echo" are shell words
        for c in [
            "foo && bar || \"command\" echo hi",
            "foo && bar || 'command' echo hi",
            "foo && bar || \"builtin\" echo hi",
        ] {
            assert_eq!(check(c).diagnostics.len(), 0, "{c}");
        }
    }

    #[test]
    fn test_sc2015_quoted_command_prefix_not_over_exempt() {
        assert_eq!(
            check("foo && bar || \"command\" cleanup").diagnostics.len(),
            1
        );
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
