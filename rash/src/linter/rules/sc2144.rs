// SC2144: -e doesn't work with globs in [[ ]]. Use a for loop or find
//
// The -e test in [[ ]] checks if the literal pattern exists as a filename,
// it does NOT expand the glob. This is almost never the intended behavior.
//
// Examples:
// Bad:
//   [[ -e *.txt ]]             # Checks if file named "*.txt" exists (literal)
//   [[ -f /path/*.log ]]       # Checks if file named "*.log" exists (literal)
//
// Good:
//   for f in *.txt; do [[ -e $f ]] && ...; done  # Loop over matches
//   files=(*.txt); [[ -e ${files[0]} ]]          # Check first match
//   shopt -s nullglob; files=(*.txt); [[ ${#files[@]} -gt 0 ]]  # Check if any exist

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static FILE_TEST_WITH_GLOB: Lazy<Regex> = Lazy::new(|| {
    // Match: -e or -f or -d followed by pattern with * or ?
    Regex::new(r"-[efd]\s+([^\s\]]*[\*\?][^\s\]]*)").unwrap()
});

static DOUBLE_BRACKET: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[\[.*?\]\]").unwrap());

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Only check lines with [[ ]]
        if !line.contains("[[") {
            continue;
        }

        // Extract [[ ]] blocks
        for bracket_match in DOUBLE_BRACKET.find_iter(line) {
            let bracket_text = bracket_match.as_str();

            // Check for file tests with globs
            if let Some(cap) = FILE_TEST_WITH_GLOB.captures(bracket_text) {
                let pattern = cap.get(1).unwrap().as_str();
                let start_col = line.find(bracket_text).unwrap_or(0) + 1;
                let end_col = start_col + bracket_text.len();

                let diagnostic = Diagnostic::new(
                    "SC2144",
                    Severity::Warning,
                    format!(
                        "-e doesn't expand globs in [[ ]]. Use a for loop to check if any '{}' exist",
                        pattern
                    ),
                    Span::new(line_num, start_col, line_num, end_col),
                );

                result.add(diagnostic);
                break; // Only warn once per [[ ]] block
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sc2144_glob_with_e() {
        let code = r#"[[ -e *.txt ]]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2144");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
        assert!(result.diagnostics[0].message.contains("glob"));
    }

    #[test]
    fn test_sc2144_glob_with_f() {
        let code = r#"[[ -f /path/*.log ]]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2144_glob_with_d() {
        let code = r#"[[ -d dir_* ]]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2144_question_mark_glob() {
        let code = r#"[[ -e file?.txt ]]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2144_specific_file_ok() {
        let code = r#"[[ -e file.txt ]]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2144_variable_ok() {
        let code = r#"[[ -e $file ]]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2144_for_loop_ok() {
        let code = r#"for f in *.txt; do [[ -e $f ]] && echo "$f"; done"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2144_single_bracket_ok() {
        let code = r#"[ -e *.txt ]"#;
        let result = check(code);
        // Single bracket also doesn't expand, but that's documented
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2144_multiple_globs() {
        let code = r#"[[ -e *.txt || -e *.log ]]"#;
        let result = check(code);
        // Only warns once per [[ ]]
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2144_path_with_glob() {
        let code = r#"[[ -f /var/log/app*.log ]]"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }
}
