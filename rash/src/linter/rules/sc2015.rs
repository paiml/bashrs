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
        .replace("&>", ">");
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

/// Name of the command a fallback runs, skipping leading `VAR=value` words and
/// `command`/`builtin` prefixes, with quotes and any directory removed.
/// `None` when the fallback is only assignments.
fn fallback_command(fallback: &str) -> Option<String> {
    let mut words = fallback
        .split_whitespace()
        .skip_while(|w| w.contains('=') && !w.starts_with('=') && !w.starts_with(['"', '\'']))
        .skip_while(|w| matches!(*w, "command" | "builtin"));
    let word = words.next()?;
    let word = word.split(['<', '>']).next().unwrap_or("");
    let word = word.trim_matches(['"', '\'']);
    Some(word.rsplit('/').next().unwrap_or("").to_string())
}

/// ShellCheck 0.8.0 does not flag `A && B || C` when C is a single error-reporting or
/// terminating command (the usual "A && B || die" idiom) or a bare assignment.
fn fallback_is_exempt(matched: &str, after: &str) -> bool {
    let fallback = matched.rsplit("||").next().unwrap_or("").trim_start();
    if fallback_is_pipeline(&format!("{fallback}{after}")) {
        return false;
    }
    match fallback_command(fallback) {
        None => true,
        Some(cmd) => matches!(cmd.as_str(), "echo" | "exit" | "return" | "printf"),
    }
}

fn diagnostic_for(line_num: usize, m: &regex::Match) -> Diagnostic {
    Diagnostic::new(
        "SC2015",
        Severity::Info,
        "Note that 'A && B || C' is not if-then-else. C may run when A is true. Use 'if A; then B; else C; fi' for proper conditional logic".to_string(),
        Span::new(line_num, m.start() + 1, line_num, m.end() + 1),
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
        for m in AND_OR_PATTERN.find_iter(line) {
            if fallback_is_exempt(m.as_str(), &line[m.end()..]) {
                continue;
            }
            result.add(diagnostic_for(line_num, &m));
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
