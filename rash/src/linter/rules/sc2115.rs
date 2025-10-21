// SC2115: Use "${var:?}" to ensure this never expands to /* or /
//
// When using rm -r with paths starting with /, ensure variables are set
// to prevent accidentally deleting from root.
//
// Examples:
// Bad:
//   rm -r "$prefix"/data     # If $prefix is empty, becomes "rm -r /data"
//   rm -r /"$dir"/*          # If $dir is empty, becomes "rm -r //*"
//
// Good:
//   rm -r "${prefix:?}"/data  # Fails if $prefix is unset/empty
//   rm -r "${prefix:-/tmp}"/data  # Uses default if empty
//   [[ -n "$prefix" ]] && rm -r "$prefix"/data  # Guard check

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static RM_SLASH_VAR: Lazy<Regex> = Lazy::new(|| {
    // Match: rm -r ... /$var or rm -r ... "$var"/...
    // Where the pattern could expand to root if var is empty
    Regex::new(r#"rm\s+-[a-zA-Z]*r[a-zA-Z]*\s+.*["']?\$\{?([A-Za-z_][A-Za-z0-9_]*)\}?["']?/|rm\s+-[a-zA-Z]*r[a-zA-Z]*\s+/["']?\$\{?([A-Za-z_][A-Za-z0-9_]*)\}?["']?"#).unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Skip if already has guards (:?, :-, etc.)
        if line.contains(":?") || line.contains(":-") || line.contains(":=") {
            continue;
        }

        // Skip if there's an explicit guard check
        if line.contains("[[ -n") || line.contains("[ -n") {
            continue;
        }

        for cap in RM_SLASH_VAR.captures_iter(line) {
            // Get the variable name from either capture group
            let var_name = cap.get(1).or_else(|| cap.get(2)).map(|m| m.as_str());

            if let Some(var) = var_name {
                let start_col = cap.get(0).unwrap().start() + 1;
                let end_col = cap.get(0).unwrap().end() + 1;

                let diagnostic = Diagnostic::new(
                    "SC2115",
                    Severity::Warning,
                    format!(
                        "Use \"${{{}:?}}\" to ensure this never expands to / or /*",
                        var
                    ),
                    Span::new(line_num, start_col, line_num, end_col),
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
    fn test_sc2115_rm_prefix_slash() {
        let code = r#"rm -r "$prefix"/data"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2115");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
        assert!(result.diagnostics[0].message.contains(":?"));
    }

    #[test]
    fn test_sc2115_rm_slash_var() {
        let code = r#"rm -r /"$dir"/*"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2115_guarded_with_question_ok() {
        let code = r#"rm -r "${prefix:?}"/data"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2115_guarded_with_default_ok() {
        let code = r#"rm -r "${prefix:-/tmp}"/data"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2115_explicit_guard_ok() {
        let code = r#"[[ -n "$prefix" ]] && rm -r "$prefix"/data"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2115_safe_path_ok() {
        let code = r#"rm -r /tmp/data"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2115_rm_rf_variant() {
        let code = r#"rm -rf "$build_dir"/output"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2115_multiple_flags() {
        let code = r#"rm -rfv "$tmpdir"/cache"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2115_no_slash_ok() {
        let code = r#"rm -r "$file""#;
        let result = check(code);
        // No slash pattern, so no warning
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2115_rm_without_r_ok() {
        let code = r#"rm "$prefix"/file.txt"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
