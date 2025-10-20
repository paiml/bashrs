// SC2174: When used with -p, -m only applies to the deepest directory
//
// mkdir -p -m with permissions only applies to the final directory created,
// not to parent directories. This can lead to unexpected permission issues.
//
// Examples:
// Bad:
//   mkdir -p -m 700 /tmp/a/b/c  # Only c gets 700, a and b may get 755
//   mkdir -pm700 /var/secure    # Parent dirs may have wrong perms
//
// Good:
//   mkdir -p /tmp/a/b/c && chmod 700 /tmp/a/b/c  # Explicit
//   mkdir -p /tmp/a/b && mkdir -m 700 /tmp/a/b/c  # Create parents separately
//   install -d -m 700 /tmp/a/b/c  # install command handles this better

use crate::linter::{Diagnostic, LintResult, Severity, Span};
use once_cell::sync::Lazy;
use regex::Regex;

static MKDIR_P_WITH_MODE: Lazy<Regex> = Lazy::new(|| {
    // Match: mkdir -p ... -m mode or mkdir -pm mode or mkdir -m ... -p
    Regex::new(r"\bmkdir\s+(-[a-zA-Z]*p[a-zA-Z]*\s+.*-m\s+[0-7]+|-[a-zA-Z]*p[a-zA-Z]*m\s*[0-7]+|-m\s+[0-7]+\s+.*-[a-zA-Z]*p[a-zA-Z]*)").unwrap()
});

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num + 1;

        if line.trim_start().starts_with('#') {
            continue;
        }

        for mat in MKDIR_P_WITH_MODE.find_iter(line) {
            let start_col = mat.start() + 1;
            let end_col = mat.end() + 1;

            let diagnostic = Diagnostic::new(
                "SC2174",
                Severity::Warning,
                "When used with -p, -m only applies to the deepest directory. Use install -d or chmod separately.",
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
    fn test_sc2174_mkdir_p_m() {
        let code = r#"mkdir -p -m 700 /tmp/a/b/c"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "SC2174");
        assert_eq!(result.diagnostics[0].severity, Severity::Warning);
        assert!(result.diagnostics[0].message.contains("deepest directory"));
    }

    #[test]
    fn test_sc2174_mkdir_pm_combined() {
        let code = r#"mkdir -pm 755 /var/data"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2174_mkdir_pm700() {
        let code = r#"mkdir -pm700 /secure/path"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2174_mkdir_p_ok() {
        let code = r#"mkdir -p /tmp/a/b/c"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2174_mkdir_m_without_p_ok() {
        let code = r#"mkdir -m 700 /tmp/single"#;
        let result = check(code);
        // No -p flag, so warning doesn't apply
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2174_install_d_ok() {
        let code = r#"install -d -m 700 /tmp/a/b/c"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2174_separate_chmod_ok() {
        let code = r#"mkdir -p /tmp/a/b/c && chmod 700 /tmp/a/b/c"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_sc2174_flags_reversed_order() {
        let code = r#"mkdir -m 755 -p /var/lib/app"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2174_verbose_flag() {
        let code = r#"mkdir -pv -m 644 /tmp/output"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_sc2174_no_mkdir_ok() {
        let code = r#"install -d /tmp/path"#;
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0);
    }
}
