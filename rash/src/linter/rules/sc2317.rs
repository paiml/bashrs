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

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let lines: Vec<&str> = source.lines().collect();

    let mut found_exit = false;
    let mut exit_line = 0;

    for (line_num, line) in lines.iter().enumerate() {
        let line_num_1indexed = line_num + 1;
        let trimmed = line.trim();

        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }

        // Issue #108: Skip case statement terminators - ;; is syntax, not code
        // Also skip ;& and ;;& (fall-through terminators)
        if trimmed == ";;" || trimmed == ";&" || trimmed == ";;&" {
            continue;
        }

        // Reset found_exit when encountering block closers
        // Also reset on esac (end of case statement)
        if trimmed.starts_with('}')
            || trimmed.starts_with("fi")
            || trimmed.starts_with("done")
            || trimmed.starts_with("esac")
        {
            found_exit = false;
            continue;
        }

        // Issue #108: Reset on case clause patterns (lines ending with ))
        // e.g., --help|-h) or *) or a) patterns start new reachability context
        // Exclude subshell syntax: $(...) or standalone (...)
        if trimmed.ends_with(')') && !trimmed.contains("$(") && !trimmed.starts_with('(') {
            found_exit = false;
            continue;
        }

        // Issue #93: Check for exit/return
        if !found_exit && EXIT_OR_RETURN.is_match(trimmed) {
            // Skip if exit/return is conditional (part of || or && chain)
            if is_conditional_exit(trimmed) {
                continue;
            }
            found_exit = true;
            exit_line = line_num;
        } else if found_exit {
            // Found code after exit/return
            let diagnostic = Diagnostic::new(
                "SC2317",
                Severity::Warning,
                format!(
                    "Command appears to be unreachable (code after exit/return on line {})",
                    exit_line + 1
                ),
                Span::new(line_num_1indexed, 1, line_num_1indexed, line.len() + 1),
            );
            result.add(diagnostic);
            break; // Only warn once per function/block
        }
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
}
