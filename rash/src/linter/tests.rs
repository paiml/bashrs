//! Integration tests for the linter module

#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use crate::linter::rules::lint_shell;
    use crate::linter::Severity;

    #[test]
    fn test_lint_integration_safe_script() {
        let safe_script = r#"
#!/bin/sh
FILES="$1"
echo "$FILES"
"#;

        let result = lint_shell(safe_script);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "Safe script should have no diagnostics"
        );
    }

    #[test]
    fn test_lint_integration_unsafe_script() {
        let unsafe_script = r#"
#!/bin/bash
FILES=$1
ls $FILES
result=$(echo $FILES)
"#;

        let result = lint_shell(unsafe_script);
        assert!(
            !result.diagnostics.is_empty(),
            "Unsafe script should have diagnostics"
        );

        // Should detect SC2086 (unquoted variables)
        assert!(result.diagnostics.iter().any(|d| d.code == "SC2086"));

        // Should detect SC2116 (useless echo)
        assert!(result.diagnostics.iter().any(|d| d.code == "SC2116"));
    }

    #[test]
    fn test_lint_integration_all_severities() {
        let mixed_script = r#"
#!/bin/bash
# Multiple issues with different severities
ls $FILES           # Warning: SC2086
result=$(echo foo)  # Info: SC2116
"#;

        let result = lint_shell(mixed_script);

        assert!(result.has_warnings(), "Should have warnings");
        assert!(result.count_by_severity(Severity::Warning) >= 1);
        assert!(result.count_by_severity(Severity::Info) >= 1);
    }
}
