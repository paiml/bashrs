// SC2035: Use ./* so names with dashes won't become options
//
// When using globs like *.txt with commands, files starting with dashes
// can be interpreted as options. Using ./*.txt prevents this.
//
// Examples:
// Bad:
//   rm *.txt           # File named "-rf.txt" would expand to "rm -rf.txt"
//   cat *.log          # File named "-n.log" would be treated as option
//   grep pattern *.sh  # File named "-v.sh" could cause issues
//
// Good:
//   rm ./*.txt         # Safe: "./-rf.txt" is clearly a file
//   cat ./*.log        # Safe: files won't be treated as options
//   grep pattern ./*.sh

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static UNSAFE_COMMAND: Lazy<Regex> = Lazy::new(|| {
    // Match commands that take file arguments
    Regex::new(r"^(?:.*\s+)?(rm|cat|grep|ls|mv|cp|chmod|chown|find|xargs|echo)\b").unwrap()
});

static BARE_GLOB: Lazy<Regex> = Lazy::new(|| {
    // Match bare globs (*.ext) that aren't prefixed with ./ or / or $
    Regex::new(r"\*\.[a-zA-Z0-9]+\b").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Check if line has a relevant command
        if !UNSAFE_COMMAND.is_match(line) {
            continue;
        }

        // Find all bare globs on this line
        for mat in BARE_GLOB.find_iter(line) {
            let glob_start = mat.start();

            // Skip if preceded by ./ or / or $
            if glob_start > 0 {
                let before = &line[..glob_start];
                if before.ends_with("./") || before.ends_with('/') || before.ends_with('$') {
                    continue;
                }
            }

            let start_col = glob_start + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2035",
                Severity::Warning,
                "Use ./* so names with dashes won't become options. Example: rm ./*.txt instead of rm *.txt",
                Span::new(line_num, start_col, line_num, end_col),
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
    fn test_sc2035_rm_glob() {
        let code = r#"rm *.txt"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2035");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
        assert!(result.diagnostics[0].message.contains("./*"));
    }

    #[test]
    fn test_sc2035_cat_glob() {
        let code = r#"cat *.log"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2035_grep_glob() {
        let code = r#"grep pattern *.sh"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2035_safe_dotslash_ok() {
        let code = r#"rm ./*.txt"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2035_absolute_path_ok() {
        let code = r#"rm /tmp/*.txt"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2035_variable_path_ok() {
        let code = r#"rm "$dir"/*.txt"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2035_multiple_globs() {
        let code = r#"rm *.txt *.log"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_sc2035_mv_glob() {
        let code = r#"mv *.bak /backup/"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2035_chmod_glob() {
        let code = r#"chmod 644 *.conf"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2035_no_glob_ok() {
        let code = r#"rm file.txt"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
