
    #[test]
    fn test_FUNC_SHELL_001_analyze_span_preserved() {
        use crate::make_parser::parse_makefile;

        let makefile = "RELEASE := $(shell date +%s)";
        let ast = parse_makefile(makefile).unwrap();
        let issues = analyze_makefile(&ast);

        assert_eq!(issues.len(), 1);
        // Span should be non-zero
        assert!(issues[0].span.end > issues[0].span.start);
        assert!(issues[0].span.line > 0);
    }

    // Unit tests for wildcard detection (FUNC-WILDCARD-001)
    #[test]
    fn test_FUNC_WILDCARD_001_detect_wildcard_basic() {
        // Should detect $(wildcard *.c)
        assert!(detect_wildcard("$(wildcard *.c)"));
    }

    #[test]
    fn test_FUNC_WILDCARD_001_detect_wildcard_with_path() {
        // Should detect $(wildcard src/*.c)
        assert!(detect_wildcard("$(wildcard src/*.c)"));
    }

    #[test]
    fn test_FUNC_WILDCARD_001_no_false_positive() {
        // Should NOT detect when no wildcard
        assert!(!detect_wildcard("SOURCES := main.c util.c"));
    }

    #[test]
    fn test_FUNC_WILDCARD_001_detect_in_variable_context() {
        // Should detect in full variable assignment context
        let value = "SOURCES := $(wildcard *.c)";
        assert!(detect_wildcard(value));
    }

    // Edge cases
    #[test]
    fn test_FUNC_WILDCARD_001_empty_string() {
        assert!(!detect_wildcard(""));
    }

    #[test]
    fn test_FUNC_WILDCARD_001_wildcard_text_not_function() {
        // Just the word "wildcard" is not a problem
        assert!(!detect_wildcard("# Use wildcard to find files"));
    }

    #[test]
    fn test_FUNC_WILDCARD_001_multiple_wildcards() {
        // Should detect if ANY contain wildcard
        assert!(detect_wildcard("A=*.c B=$(wildcard *.h)"));
    }

    #[test]
    fn test_FUNC_WILDCARD_001_case_sensitive() {
        // Should be case-sensitive (Make functions are case-sensitive)
        assert!(!detect_wildcard("$(WILDCARD *.c)"));
    }

    // Mutation-killing tests
    #[test]
    fn test_FUNC_WILDCARD_001_mut_contains_must_check_substring() {
        // Ensures we use .contains() not .eq()
        assert!(detect_wildcard("prefix $(wildcard *.c) suffix"));
    }

    #[test]
    fn test_FUNC_WILDCARD_001_mut_exact_pattern() {
        // Ensures we check for "$(wildcard" not just "wildcard"
        assert!(!detect_wildcard("wildcard_var"));
    }

    #[test]
    fn test_FUNC_WILDCARD_001_mut_non_empty_check() {
        // Ensures we don't crash on empty strings
        let result = detect_wildcard("");
        assert!(!result);
    }

    // Property-based tests for wildcard detection
    #[cfg(test)]
    mod wildcard_property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn prop_FUNC_WILDCARD_001_any_string_no_panic(s in "\\PC*") {
                // Should never panic on any string
                let _ = detect_wildcard(&s);
            }

            #[test]
            fn prop_FUNC_WILDCARD_001_wildcard_always_detected(
                pattern in "[*.a-zA-Z0-9/_-]*"
            ) {
                let input = format!("$(wildcard {})", pattern);
                prop_assert!(detect_wildcard(&input));
            }

            #[test]
            fn prop_FUNC_WILDCARD_001_no_dollar_never_detected(
                s in "[^$]*"
            ) {
                // Strings without $ should never be detected
                prop_assert!(!detect_wildcard(&s));
            }

            #[test]
            fn prop_FUNC_WILDCARD_001_deterministic(s in "\\PC*") {
                // Same input always gives same output
                let result1 = detect_wildcard(&s);
                let result2 = detect_wildcard(&s);
                prop_assert_eq!(result1, result2);
            }

            #[test]
            fn prop_FUNC_WILDCARD_001_other_functions_not_detected(
                func in "(shell|subst|patsubst|filter|sort|dir|notdir|basename|suffix)"
            ) {
                // $(other_function ...) should not be detected as wildcard
                let input = format!("$({} test)", func);
                prop_assert!(!detect_wildcard(&input));
            }
        }
    }

    // Integration tests for analyze_makefile() with wildcard
    #[test]
    fn test_FUNC_WILDCARD_001_analyze_detects_wildcard() {
        use crate::make_parser::parse_makefile;

        let makefile = "SOURCES := $(wildcard *.c)";
        let ast = parse_makefile(makefile).unwrap();
        let issues = analyze_makefile(&ast);

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].rule, "NO_WILDCARD");
        assert_eq!(issues[0].severity, IssueSeverity::High);
        assert!(issues[0].message.contains("SOURCES"));
        assert!(issues[0].suggestion.is_some());
    }

    #[test]
    fn test_FUNC_WILDCARD_001_analyze_wildcard_severity_high() {
        use crate::make_parser::parse_makefile;

        let makefile = "FILES := $(wildcard src/*.c)";
        let ast = parse_makefile(makefile).unwrap();
        let issues = analyze_makefile(&ast);

        assert_eq!(issues.len(), 1);
        // Wildcard is High severity (less critical than timestamps)
        assert_eq!(issues[0].severity, IssueSeverity::High);
    }

    #[test]
    fn test_FUNC_WILDCARD_001_analyze_multiple_issues() {
        use crate::make_parser::parse_makefile;

        let makefile = r#"SOURCES := $(wildcard *.c)
HEADERS := $(wildcard *.h)"#;
        let ast = parse_makefile(makefile).unwrap();
        let issues = analyze_makefile(&ast);

        assert_eq!(issues.len(), 2);
        assert!(issues[0].message.contains("SOURCES"));
        assert!(issues[1].message.contains("HEADERS"));
    }

    #[test]
    fn test_FUNC_WILDCARD_001_analyze_mixed_issues() {
        use crate::make_parser::parse_makefile;

        // Both shell date (Critical) and wildcard (High)
        let makefile = r#"RELEASE := $(shell date +%s)
SOURCES := $(wildcard *.c)"#;
        let ast = parse_makefile(makefile).unwrap();
        let issues = analyze_makefile(&ast);

        assert_eq!(issues.len(), 2);
        // First issue is shell date (Critical)
        assert_eq!(issues[0].rule, "NO_TIMESTAMPS");
        assert_eq!(issues[0].severity, IssueSeverity::Critical);
        // Second issue is wildcard (High)
        assert_eq!(issues[1].rule, "NO_WILDCARD");
        assert_eq!(issues[1].severity, IssueSeverity::High);
    }

    // Unit tests for auto-PHONY detection (PHONY-002)
    #[test]
    fn test_PHONY_002_is_common_phony_target_test() {
        assert!(is_common_phony_target("test"));
    }

include!("semantic_tests_extracted_FUNC_PHONY.rs");
