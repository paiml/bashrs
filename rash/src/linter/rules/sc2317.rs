// SC2317: Command appears to be unreachable (dead code)
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static EXIT_OR_RETURN: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?:exit|return)\s+\d+").unwrap());

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

        // Reset found_exit when encountering block closers
        if trimmed.starts_with('}') || trimmed.starts_with("fi") || trimmed.starts_with("done") {
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
}
