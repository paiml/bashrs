// SC2317: Command appears to be unreachable (dead code)
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static EXIT_OR_RETURN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?:exit|return)\s+\d+").unwrap()
});

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

        if !found_exit && EXIT_OR_RETURN.is_match(trimmed) {
            found_exit = true;
            exit_line = line_num;
        } else if found_exit {
            // Found code after exit/return
            let diagnostic = Diagnostic::new(
                "SC2317",
                Severity::Warning,
                format!("Command appears to be unreachable (code after exit/return on line {})", exit_line + 1),
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
}
