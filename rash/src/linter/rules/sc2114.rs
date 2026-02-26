// SC2114: Warning: rm -rf can be dangerous. Consider using safer patterns.
//
// Using rm -rf with variables or paths requires extra care to avoid
// accidentally deleting important data. This rule warns about risky patterns.
//
// Examples:
// Bad:
//   rm -rf "$dir"              # If $dir is empty, becomes "rm -rf"
//   rm -rf /                   # Deletes everything!
//   rm -rf /*                  # Also deletes everything
//   rm -rf /$var               # If $var is empty, deletes root
//
// Good:
//   rm -rf "${dir:?}"          # Fails if $dir is unset/empty
//   rm -rf "$dir/"*            # Only deletes contents, not root
//   [[ -n "$dir" ]] && rm -rf "$dir"  # Guard check

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use regex::Regex;

static DANGEROUS_RM_RF: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match dangerous rm -rf patterns - support -rf or -fr
    Regex::new(r#"rm\s+-[a-zA-Z]*[rf][a-zA-Z]*[rf][a-zA-Z]*\s+(["']?/["']?$|/\*|/\$)"#).unwrap()
});

static UNGUARDED_RM_RF_VAR: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Match: rm -rf "$var" without :? guard - support -rf or -fr
    Regex::new(r#"rm\s+-[a-zA-Z]*[rf][a-zA-Z]*[rf][a-zA-Z]*\s+"\$\{?([A-Za-z_][A-Za-z0-9_]*)\}?""#)
        .unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        // Check for extremely dangerous patterns (rm -rf /, rm -rf /*, rm -rf /$var)
        for mat in DANGEROUS_RM_RF.find_iter(line) {
            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2114",
                Severity::Error,
                "CRITICAL: rm -rf on root or root-like path is extremely dangerous and likely a bug",
                Span::new(line_num, start_col, line_num, end_col),
            );

            result.add(diagnostic);
        }

        // Check for unguarded rm -rf with variables
        for cap in UNGUARDED_RM_RF_VAR.captures_iter(line) {
            // Skip if it has :? guard
            if line.contains(":?") {
                continue;
            }

            if let Some(var_match) = cap.get(0) {
                let start_col = var_match.start() + 1;
                let end_col = var_match.end() + 1;

                let var_name = cap.get(1).unwrap().as_str();

                let diagnostic = Diagnostic::new(
                    "SC2114",
                    Severity::Warning,
                    format!(
                        "Use \"${{{}:?}}\" to ensure variable is set before rm -rf",
                        var_name
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
    fn test_sc2114_rm_rf_root() {
        let code = r#"rm -rf /"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2114");
        assert_eq!(result.diagnostics[0].severity, Severity::Error);
        assert!(result.diagnostics[0].message.contains("CRITICAL"));
    }

    #[test]
    fn test_sc2114_rm_rf_root_wildcard() {
        let code = r#"rm -rf /*"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].severity, Severity::Error);
    }

    #[test]
    fn test_sc2114_rm_rf_unguarded_var() {
        let code = r#"rm -rf "$dir""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
        assert!(result.diagnostics[0].message.contains(":?"));
    }

    #[test]
    fn test_sc2114_rm_rf_guarded_var_ok() {
        let code = r#"rm -rf "${dir:?}""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2114_rm_rf_safe_path_ok() {
        let code = r#"rm -rf /tmp/mydir"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2114_rm_rf_relative_ok() {
        let code = r#"rm -rf ./build"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2114_rm_flags_different_order() {
        let code = r#"rm -fr /"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2114_rm_multiple_flags() {
        let code = r#"rm -rfv "$tmpdir""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
    }

    #[test]
    fn test_sc2114_rm_rf_root_var() {
        let code = r#"rm -rf /$var"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].severity, Severity::Error);
    }

    #[test]
    fn test_sc2114_rm_without_rf_ok() {
        let code = r#"rm "$file""#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
