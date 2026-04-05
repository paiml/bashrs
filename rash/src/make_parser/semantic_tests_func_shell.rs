#[cfg(test)]
mod tests {
    use super::*;

    // Unit tests for shell date detection
    #[test]
    fn test_FUNC_SHELL_001_detect_shell_date_basic() {
        // Should detect $(shell date +%s)
        assert!(detect_shell_date("$(shell date +%s)"));
    }

    #[test]
    fn test_FUNC_SHELL_001_detect_shell_date_with_format() {
        // Should detect $(shell date +%Y%m%d-%H%M%S)
        assert!(detect_shell_date("$(shell date +%Y%m%d-%H%M%S)"));
    }

    #[test]
    fn test_FUNC_SHELL_001_no_false_positive() {
        // Should NOT detect when no shell date
        assert!(!detect_shell_date("VERSION := 1.0.0"));
    }

    #[test]
    fn test_FUNC_SHELL_001_detect_in_variable_context() {
        // Should detect in full variable assignment context
        let value = "RELEASE := $(shell date +%s)";
        assert!(detect_shell_date(value));
    }

    // Edge cases
    #[test]
    fn test_FUNC_SHELL_001_empty_string() {
        assert!(!detect_shell_date(""));
    }

    #[test]
    fn test_FUNC_SHELL_001_no_shell_command() {
        assert!(!detect_shell_date("$(CC) -o output"));
    }

    #[test]
    fn test_FUNC_SHELL_001_shell_but_not_date() {
        assert!(!detect_shell_date("$(shell pwd)"));
    }

    #[test]
    fn test_FUNC_SHELL_001_multiple_shell_commands() {
        // Should detect if ANY contain shell date
        assert!(detect_shell_date("A=$(shell pwd) B=$(shell date +%s)"));
    }

    #[test]
    fn test_FUNC_SHELL_001_date_without_shell() {
        // "date" alone is not a problem
        assert!(!detect_shell_date("# Update date: 2025-10-16"));
    }

    #[test]
    fn test_FUNC_SHELL_001_case_sensitive() {
        // Should be case-sensitive (shell commands are case-sensitive)
        assert!(!detect_shell_date("$(SHELL DATE)"));
    }

    // Mutation-killing tests
    #[test]
    fn test_FUNC_SHELL_001_mut_contains_must_check_substring() {
        // Ensures we use .contains() not .eq()
        assert!(detect_shell_date("prefix $(shell date +%s) suffix"));
    }

    #[test]
    fn test_FUNC_SHELL_001_mut_exact_pattern() {
        // Ensures we check for "$(shell date" not just "date"
        assert!(!detect_shell_date("datestamp"));
    }

    #[test]
    fn test_FUNC_SHELL_001_mut_non_empty_check() {
        // Ensures we don't crash on empty strings
        let result = detect_shell_date("");
        assert!(!result);
    }

    // Property-based tests
    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn prop_FUNC_SHELL_001_any_string_no_panic(s in "\\PC*") {
                // Should never panic on any string
                let _ = detect_shell_date(&s);
            }

            #[test]
            fn prop_FUNC_SHELL_001_shell_date_always_detected(
                format in "[+%a-zA-Z0-9-]*"
            ) {
                let input = format!("$(shell date {})", format);
                prop_assert!(detect_shell_date(&input));
            }

            #[test]
            fn prop_FUNC_SHELL_001_no_shell_never_detected(
                s in "[^$]*"
            ) {
                // Strings without $ should never be detected
                prop_assert!(!detect_shell_date(&s));
            }

            #[test]
            fn prop_FUNC_SHELL_001_deterministic(s in "\\PC*") {
                // Same input always gives same output
                let result1 = detect_shell_date(&s);
                let result2 = detect_shell_date(&s);
                prop_assert_eq!(result1, result2);
            }

            #[test]
            fn prop_FUNC_SHELL_001_shell_without_date_not_detected(
                cmd in "[a-z]{3,10}"
            ) {
                // $(shell <non-date-command>) should not be detected
                if cmd != "date" {
                    let input = format!("$(shell {})", cmd);
                    prop_assert!(!detect_shell_date(&input));
                }
            }
        }
    }

    // Integration tests for analyze_makefile()
    #[test]
    fn test_FUNC_SHELL_001_analyze_detects_shell_date() {
        use crate::make_parser::parse_makefile;

        let makefile = "RELEASE := $(shell date +%s)";
        let ast = parse_makefile(makefile).unwrap();
        let issues = analyze_makefile(&ast);

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].rule, "NO_TIMESTAMPS");
        assert_eq!(issues[0].severity, IssueSeverity::Critical);
        assert!(issues[0].message.contains("RELEASE"));
        assert!(issues[0].suggestion.is_some());
    }

    #[test]
    fn test_FUNC_SHELL_001_analyze_no_issues_clean_makefile() {
        use crate::make_parser::parse_makefile;

        let makefile = "VERSION := 1.0.0\nCC := gcc";
        let ast = parse_makefile(makefile).unwrap();
        let issues = analyze_makefile(&ast);

        assert_eq!(issues.len(), 0);
    }

    #[test]
    fn test_FUNC_SHELL_001_analyze_multiple_issues() {
        use crate::make_parser::parse_makefile;

        let makefile = r#"RELEASE := $(shell date +%s)
VERSION := 1.0.0
BUILD_TIME := $(shell date +%Y%m%d)"#;
        let ast = parse_makefile(makefile).unwrap();
        let issues = analyze_makefile(&ast);

        assert_eq!(issues.len(), 2);
        assert!(issues[0].message.contains("RELEASE"));
        assert!(issues[1].message.contains("BUILD_TIME"));
    }

    #[test]
    fn test_FUNC_SHELL_001_analyze_suggestion_format() {
        use crate::make_parser::parse_makefile;

        let makefile = "TIMESTAMP := $(shell date +%s)";
        let ast = parse_makefile(makefile).unwrap();
        let issues = analyze_makefile(&ast);

        assert_eq!(issues.len(), 1);
        let suggestion = issues[0].suggestion.as_ref().unwrap();
        assert!(suggestion.contains("TIMESTAMP"));
        assert!(suggestion.contains(":="));
    }

    #[test]
    fn test_FUNC_SHELL_001_analyze_ignores_targets() {
        use crate::make_parser::parse_makefile;

        // Should NOT detect shell date in recipe commands (only in variables)
        // But WILL detect missing .PHONY for "build" target
        let makefile = "build:\n\techo $(shell date +%s)";
        let ast = parse_makefile(makefile).unwrap();
        let issues = analyze_makefile(&ast);

        // Should only detect AUTO_PHONY (not NO_TIMESTAMPS)
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].rule, "AUTO_PHONY");
    }

}

#[cfg(test)]
mod semantic_tests_extracted_FUNC {
    use super::*;
    include!("semantic_tests_extracted_FUNC.rs");
}
