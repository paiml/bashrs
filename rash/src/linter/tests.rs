//! Integration tests for the linter module

#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use crate::linter::rules::lint_shell;
    use crate::linter::Severity;

    #[test]
    fn test_lint_integration_safe_script() {
        let safe_script = "#!/bin/sh\nFILES=\"$1\"\necho \"$FILES\"\n";

        let result = lint_shell(safe_script);
        let errors_and_warnings: Vec<_> = result
            .diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error || d.severity == Severity::Warning)
            .collect();
        assert_eq!(
            errors_and_warnings.len(),
            0,
            "Safe script should have no errors or warnings"
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

    // bashrs#677: `arr=($(cmd))` is an array assignment whose elements are
    // word-split. ShellCheck 0.8.0 reports only SC2207 for it.
    #[test]
    fn test_lint_677_array_cmdsub_emits_only_sc2207() {
        let result = lint_shell("#!/bin/bash\ndeclare -a arr=($(ls))\n");
        let codes: Vec<&str> = result.diagnostics.iter().map(|d| d.code.as_str()).collect();
        assert!(codes.contains(&"SC2207"), "SC2207 must stay: {codes:?}");
        assert!(!codes.contains(&"SC2046"), "extra SC2046: {codes:?}");
        assert!(!codes.contains(&"SC2206"), "extra SC2206: {codes:?}");
    }

    // bashrs#677: a plain scalar command substitution assignment is not touched.
    #[test]
    fn test_lint_677_plain_cmdsub_assignment_keeps_sc2046() {
        let result = lint_shell("#!/bin/bash\nx=$(ls)\n");
        let codes: Vec<&str> = result.diagnostics.iter().map(|d| d.code.as_str()).collect();
        assert!(codes.contains(&"SC2046"), "SC2046 must stay: {codes:?}");
        assert!(!codes.contains(&"SC2207"), "unexpected SC2207: {codes:?}");
    }
}
