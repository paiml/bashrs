// SC2103: Use cd ... || exit instead of cd ..; cd ..
//
// Detects attempts to return to a previous directory using multiple cd .. commands
// instead of proper directory stack usage (pushd/popd) or error handling.
//
// Examples:
// Bad:
//   cd /some/deep/path
//   # ... do work ...
//   cd ..
//   cd ..
//
// Good:
//   cd /some/deep/path || exit
//   # ... do work ...
//   cd - || exit  # or use pushd/popd

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static CONSECUTIVE_CD_UP: Lazy<Regex> = Lazy::new(|| Regex::new(r"cd\s+\.\.").unwrap());

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let lines: Vec<&str> = source.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        let line_num = i + 1;

        // Skip comments
        if line.trim_start().starts_with('#') {
            continue;
        }

        // Detect cd .. on current line
        if !CONSECUTIVE_CD_UP.is_match(line) {
            continue;
        }

        // Check if next line also has cd ..
        if i + 1 < lines.len() {
            let next_line = lines[i + 1];
            if !next_line.trim_start().starts_with('#') && CONSECUTIVE_CD_UP.is_match(next_line) {
                let diagnostic = Diagnostic::new(
                    "SC2103",
                    Severity::Warning,
                    "Use pushd/popd or cd - instead of multiple cd .. commands",
                    Span::new(line_num, 1, line_num, line.len() + 1),
                );

                result.add(diagnostic);
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2103_consecutive_cd_up() {
        let code = r#"
cd /some/deep/path
# do work
cd ..
cd ..
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2103");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
    }

    #[test]
    fn test_sc2103_single_cd_up_ok() {
        let code = r#"
cd /some/path
# do work
cd ..
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2103_cd_up_with_other_commands() {
        let code = r#"
cd /some/deep/path
# do work
cd ..
echo "Back one level"
cd ..
"#;
        let result = check(code);
        // Not consecutive lines, so no warning
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2103_three_consecutive_cd_up() {
        let code = r#"
cd /very/deep/path
cd ..
cd ..
cd ..
"#;
        let result = check(code);
        // Should detect first pair and second pair
        assert!(result.diagnostics.len() >= 1);
    }

    #[test]
    fn test_sc2103_cd_to_named_directory() {
        let code = r#"
cd /some/path
cd /another/path
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2103_comment_between_cd() {
        let code = r#"
cd ..
# comment
cd ..
"#;
        let result = check(code);
        // Comment separates them, but they're still consecutive non-comment lines
        // Current implementation checks immediate next line, so this won't trigger
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2103_cd_dash_ok() {
        let code = r#"
cd /some/path
cd -
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2103_pushd_popd_ok() {
        let code = r#"
pushd /some/path
# do work
popd
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2103_no_cd_commands() {
        let code = r#"
echo "No cd commands here"
ls -la
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2103_cd_up_in_function() {
        let code = r#"
function cleanup() {
    cd ..
    cd ..
}
"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
