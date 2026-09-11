// SC2317: Command appears to be unreachable (dead code)
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static EXIT_OR_RETURN: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"(?:exit|return)\s+\d+").unwrap());

/// Issue #93: Check if exit/return is conditional (part of || or && chain)
/// `cmd || exit 1` - exit only runs if cmd fails, code after IS reachable
/// `cmd && exit 1` - exit only runs if cmd succeeds, code after IS reachable
fn is_conditional_exit(line: &str) -> bool {
    // Check if exit/return is preceded by || or &&
    if let Some(pos) = line.find("exit") {
        let before = &line[..pos];
        if before.contains("||") || before.contains("&&") {
            return true;
        }
    }
    if let Some(pos) = line.find("return") {
        let before = &line[..pos];
        if before.contains("||") || before.contains("&&") {
            return true;
        }
    }
    false
}

/// Issue #108: `;;`, `;&` and `;;&` are case-terminator syntax, not code.
fn is_case_terminator(trimmed: &str) -> bool {
    trimmed == ";;" || trimmed == ";&" || trimmed == ";;&"
}

/// Reset points: a block closer (`}`, `fi`, `done`, `esac`) or a case clause
/// pattern (`--help|-h)`, `*)`, `a)`) each start a fresh reachability
/// context. `$(...)`/`(...)` are excluded so a real subshell isn't mistaken
/// for a clause pattern (Issue #108).
fn resets_reachability(trimmed: &str) -> bool {
    let is_block_closer = trimmed.starts_with('}')
        || trimmed.starts_with("fi")
        || trimmed.starts_with("done")
        || trimmed.starts_with("esac");
    let is_case_clause =
        trimmed.ends_with(')') && !trimmed.contains("$(") && !trimmed.starts_with('(');
    is_block_closer || is_case_clause
}

/// Issue #93: a real, unconditional `exit N` / `return N`.
fn starts_unreachable_run(trimmed: &str) -> bool {
    EXIT_OR_RETURN.is_match(trimmed) && !is_conditional_exit(trimmed)
}

fn unreachable_diagnostic(line_num_1indexed: usize, line: &str, exit_line: usize) -> Diagnostic {
    Diagnostic::new(
        "SC2317",
        Severity::Warning,
        format!(
            "Command appears to be unreachable (code after exit/return on line {})",
            exit_line + 1
        ),
        Span::new(line_num_1indexed, 1, line_num_1indexed, line.len() + 1),
    )
}

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    let mut found_exit = false;
    let mut exit_line = 0;

    for (line_num, line) in source.lines().enumerate() {
        let line_num_1indexed = line_num + 1;
        let trimmed = line.trim();

        if trimmed.starts_with('#') || trimmed.is_empty() || is_case_terminator(trimmed) {
            continue;
        }

        if resets_reachability(trimmed) {
            found_exit = false;
            continue;
        }

        if !found_exit {
            if starts_unreachable_run(trimmed) {
                found_exit = true;
                exit_line = line_num;
            }
            continue;
        }

        // Found code after exit/return.
        result.add(unreachable_diagnostic(line_num_1indexed, line, exit_line));
        break; // Only warn once per function/block
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2317_unreachable_after_exit() {
        let code = r#"
exit 1
echo "unreachable"
"#;
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2317_unreachable_after_return() {
        let code = r#"
return 0
echo "unreachable"
"#;
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2317_in_function_ok() {
        let code = r#"
foo() {
    return 0
}
echo "reachable"
"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2317_comment_ok() {
        let code = r#"
exit 1
# echo "commented"
"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2317_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2317_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2317_if_block_ok() {
        let code = r#"
if [ $x -eq 1 ]; then
    exit 1
fi
echo "reachable"
"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2317_empty_line_ok() {
        let code = r#"
exit 1

"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2317_just_exit_ok() {
        let code = "exit 0";
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2317_multiple_commands() {
        let code = r#"
return 1
cmd1
cmd2
"#;
        // Only warns once
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    // Issue #93: Conditional exit (cmd || exit) should NOT flag subsequent code
    #[test]
    fn test_issue_93_conditional_exit_or_ok() {
        // From issue #93: cd /tmp || exit 1 followed by code
        let code = r#"
cd /tmp || exit 1
echo "reachable"
"#;
        assert_eq!(
            check(code).diagnostics.len(),
            0,
            "SC2317 must NOT flag code after cmd || exit 1"
        );
    }

    #[test]
    fn test_issue_93_conditional_exit_and_ok() {
        // cmd && exit 1 - exit only on success, code after is reachable
        let code = r#"
test -f /nonexistent && exit 1
echo "reachable"
"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_issue_93_conditional_return_ok() {
        let code = r#"
check_something || return 1
echo "continue"
"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_issue_93_standalone_exit_still_flags() {
        // Standalone exit 1 (not conditional) SHOULD flag subsequent code
        let code = r#"
exit 1
echo "unreachable"
"#;
        assert_eq!(
            check(code).diagnostics.len(),
            1,
            "Standalone exit should still flag unreachable code"
        );
    }

    // Issue #108: Case statement ;; after exit should NOT be flagged
    #[test]
    fn test_issue_108_case_terminator_after_exit() {
        // The ;; is required syntax, not unreachable code
        let code = r#"
case "$1" in
    --help|-h)
        show_help
        exit 0
        ;;
    --dry-run)
        DRY_RUN=true
        ;;
esac
echo "reachable"
"#;
        assert_eq!(
            check(code).diagnostics.len(),
            0,
            "SC2317 must NOT flag ;; after exit in case statement"
        );
    }

    #[test]
    fn test_issue_108_case_clause_resets_context() {
        // Each case clause is a new reachability context
        let code = r#"
case "$1" in
    a)
        exit 1
        ;;
    b)
        echo "reachable in different clause"
        ;;
esac
"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_issue_108_esac_resets_context() {
        // Code after esac should be reachable
        let code = r#"
case "$1" in
    *)
        exit 1
        ;;
esac
echo "reachable after case"
"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_issue_108_fallthrough_terminators() {
        // ;& and ;;& are also valid case terminators
        let code = r#"
case "$1" in
    a)
        exit 1
        ;&
    b)
        echo "fallthrough"
        ;;&
esac
"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    // GH-312: the word `exit` inside a quoted argument is text, not a
    // command.
    #[test]
    fn test_PMAT257_gh312_sc2317_exit_inside_a_quoted_word_is_text() {
        let code = r#"
echo "exit 0 to stop the script early"
echo "reachable"
"#;
        assert_eq!(
            check(code).diagnostics.len(),
            0,
            "SC2317 must not flag a quoted mention of exit as a command"
        );
    }

    // GH-312: the companion half of the same defect - a real, unconditional
    // `exit 0` still makes the following code unreachable.
    #[test]
    fn test_PMAT257_gh312_sc2317_real_unreachable_code_after_exit_is_reported() {
        let code = r#"
exit 0
echo "really unreachable"
"#;
        assert_eq!(
            check(code).diagnostics.len(),
            1,
            "SC2317 must still fire on code after a real exit 0"
        );
    }
}
