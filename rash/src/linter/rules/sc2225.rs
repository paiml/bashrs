// SC2225: Backticks in assignments can interfere with line breaks
//
// GH-370: assignments are found by word POSITION (`shell_assignments::assignments`),
// not by the line regex `\b\w+\s*=\s*\``, which read `ps -o pid= \`...\`` and
// `cmd --out=\`pwd\`` as assignments.
use crate::linter::shell_assignments::assignments;
use crate::linter::{Diagnostic, LintResult, Severity, Span};

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;
        if line.trim_start().starts_with('#') {
            continue;
        }

        if assignments(line).iter().any(|a| a.value.starts_with('`')) {
            let diagnostic = Diagnostic::new(
                "SC2225",
                Severity::Info,
                "Use $(...) instead of backticks for command substitution in assignments"
                    .to_string(),
                Span::new(line_num, 1, line_num, line.len() + 1),
            );
            result.add(diagnostic);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sc2225_backtick_assignment() {
        let code = r#"result=`date`"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2225_dollar_paren_ok() {
        let code = r#"result=$(date)"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2225_backtick_command_substitution() {
        let code = r#"output=`ls -la`"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2225_nested_backticks() {
        let code = r#"var=`echo \`date\``"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2225_comment_skipped() {
        let code = r#"# var=`date`"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2225_no_assignment() {
        let code = r#"echo `date`"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // Not an assignment
    }
    #[test]
    fn test_sc2225_multiple_assignments() {
        let code = "a=`cmd1`\nb=`cmd2`";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }
    #[test]
    fn test_sc2225_with_space() {
        let code = r#"x = `pwd`"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
    #[test]
    fn test_sc2225_no_code() {
        let code = "";
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
    #[test]
    fn test_sc2225_literal_string() {
        let code = r#"var="literal value""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}

/// GH-370: same class as SC2210 — `\b\w+\s*=\s*\`` let `=` and the backtick
/// sit in different words, so `ps -o pid= \`...\`` read as an assignment.
#[cfg(test)]
mod gh370_tests {
    use super::*;

    fn count(code: &str) -> usize {
        check(code).diagnostics.len()
    }

    #[test]
    fn test_gh370_sc2225_ps_format_then_backtick_is_not_an_assignment() {
        assert_eq!(count("ps -o pid= `echo 1`"), 0);
    }

    #[test]
    fn test_gh370_sc2225_option_value_is_not_an_assignment() {
        assert_eq!(count("cmd --out=`pwd`"), 0);
    }

    #[test]
    fn test_gh370_sc2225_control_local_still_fires() {
        assert_eq!(count("local v=`date`"), 1);
    }

    #[test]
    fn test_gh370_sc2225_control_prefix_before_command_still_fires() {
        assert_eq!(count("v=`date` cmd"), 1);
    }
}
