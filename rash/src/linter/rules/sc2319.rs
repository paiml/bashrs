// SC2319: This $? refers to a condition, not the previous command
use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static CONDITION_EXITCODE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?:if|while|until)\s+.*;\s*then").unwrap());

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let lines: Vec<&str> = source.lines().collect();

    for i in 0..lines.len() {
        let line = lines[i].trim();

        if line.starts_with('#') {
            continue;
        }

        // Check if this line is a condition
        if CONDITION_EXITCODE.is_match(line) {
            // Check if next line uses $?
            if i + 1 < lines.len() {
                let next_line = lines[i + 1].trim();
                if next_line.contains("$?") && !next_line.starts_with('#') {
                    let diagnostic = Diagnostic::new(
                        "SC2319",
                        Severity::Warning,
                        "$? refers to the condition's exit code, not the command inside it"
                            .to_string(),
                        Span::new(i + 2, 1, i + 2, next_line.len() + 1),
                    );
                    result.add(diagnostic);
                }
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2319_exitcode_after_if() {
        let code = r#"
if command; then
    echo $?
fi
"#;
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2319_save_exitcode_ok() {
        let code = r#"
command
status=$?
if [ $status -eq 0 ]; then
    echo ok
fi
"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2319_direct_test_ok() {
        let code = r#"
if command; then
    echo "success"
fi
"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2319_comment_ok() {
        let code = r#"
if command; then
    # echo $?
fi
"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2319_empty() {
        assert_eq!(check("").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2319_normal() {
        assert_eq!(check("echo test").diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2319_while_loop() {
        let code = r#"
while command; then
    echo $?
done
"#;
        assert_eq!(check(code).diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2319_no_then_ok() {
        let code = r#"
command
echo $?
"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2319_nested_command_ok() {
        let code = r#"
if command; then
    other_command
    echo $?
fi
"#;
        // This is technically ambiguous but we don't catch it
        assert_eq!(check(code).diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2319_variable_comparison_ok() {
        let code = r#"
if [ $x -eq $? ]; then
    echo ok
fi
"#;
        assert_eq!(check(code).diagnostics.len(), 0);
    }
}
