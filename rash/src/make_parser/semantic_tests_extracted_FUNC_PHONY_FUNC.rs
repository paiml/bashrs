    #[test]
    fn test_FUNC_SHELL_002_analyze_detects_shell_find() {
        use crate::make_parser::parse_makefile;

        let makefile = "FILES := $(shell find src -name '*.c')";
        let ast = parse_makefile(makefile).unwrap();
        let issues = analyze_makefile(&ast);

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].rule, "NO_UNORDERED_FIND");
        assert_eq!(issues[0].severity, IssueSeverity::High);
        assert!(issues[0].message.contains("FILES"));
        assert!(issues[0].suggestion.is_some());
    }

    #[test]
    fn test_FUNC_SHELL_002_analyze_no_issues_clean_makefile() {
        use crate::make_parser::parse_makefile;

        let makefile = "FILES := src/a.c src/b.c";
        let ast = parse_makefile(makefile).unwrap();
        let issues = analyze_makefile(&ast);

        assert_eq!(issues.len(), 0);
    }

    // Integration tests for analyze_makefile() with auto-PHONY detection (PHONY-002)
    #[test]
    fn test_PHONY_002_analyze_detects_missing_phony() {
        use crate::make_parser::parse_makefile;

        // RED: Test target without .PHONY
        let makefile = "test:\n\tcargo test";
        let ast = parse_makefile(makefile).unwrap();
        let issues = analyze_makefile(&ast);

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].rule, "AUTO_PHONY");
        assert_eq!(issues[0].severity, IssueSeverity::High);
        assert!(issues[0].message.contains("test"));
        assert!(issues[0].suggestion.is_some());
        assert!(issues[0]
            .suggestion
            .as_ref()
            .unwrap()
            .contains(".PHONY: test"));
    }

    #[test]
    fn test_PHONY_002_analyze_no_issue_with_phony() {
        use crate::make_parser::parse_makefile;

        // GREEN: Test target WITH .PHONY should not trigger issue
        let makefile = ".PHONY: test\ntest:\n\tcargo test";
        let ast = parse_makefile(makefile).unwrap();
        let issues = analyze_makefile(&ast);

        // Should not report missing .PHONY
        let phony_issues: Vec<_> = issues.iter().filter(|i| i.rule == "AUTO_PHONY").collect();
        assert_eq!(phony_issues.len(), 0);
    }

    #[test]
    fn test_PHONY_002_analyze_multiple_missing_phony() {
        use crate::make_parser::parse_makefile;

        let makefile = r#"test:
	cargo test

clean:
	rm -f *.o

build:
	cargo build"#;
        let ast = parse_makefile(makefile).unwrap();
        let issues = analyze_makefile(&ast);

        // Should detect all 3 missing .PHONY declarations
        let phony_issues: Vec<_> = issues.iter().filter(|i| i.rule == "AUTO_PHONY").collect();
        assert_eq!(phony_issues.len(), 3);
    }

    #[test]
    fn test_PHONY_002_analyze_file_target_no_issue() {
        use crate::make_parser::parse_makefile;

        // Real file targets should NOT trigger AUTO_PHONY
        let makefile = "main.o: main.c\n\tgcc -c main.c";
        let ast = parse_makefile(makefile).unwrap();
        let issues = analyze_makefile(&ast);

        let phony_issues: Vec<_> = issues.iter().filter(|i| i.rule == "AUTO_PHONY").collect();
        assert_eq!(phony_issues.len(), 0);
    }

    #[test]
    fn test_PHONY_002_analyze_mixed_targets() {
        use crate::make_parser::parse_makefile;

        let makefile = r#".PHONY: clean
clean:
	rm -f *.o

main.o: main.c
	gcc -c main.c

test:
	cargo test"#;
        let ast = parse_makefile(makefile).unwrap();
        let issues = analyze_makefile(&ast);

        // Only 'test' is missing .PHONY
        let phony_issues: Vec<_> = issues.iter().filter(|i| i.rule == "AUTO_PHONY").collect();
        assert_eq!(phony_issues.len(), 1);
        assert!(phony_issues[0].message.contains("test"));
    }

    // Unit tests for $RANDOM detection (FUNC-SHELL-003)
    #[test]
    fn test_FUNC_SHELL_003_detect_random_basic() {
        // Should detect $RANDOM
        assert!(detect_random("BUILD_ID := $RANDOM"));
    }

    #[test]
    fn test_FUNC_SHELL_003_detect_double_dollar_random() {
        // Should detect $$RANDOM (shell syntax)
        assert!(detect_random("ID := $(shell echo $$RANDOM)"));
    }

    #[test]
    fn test_FUNC_SHELL_003_no_false_positive() {
        // Should NOT detect when no $RANDOM
        assert!(!detect_random("VERSION := 1.0.0"));
    }

    #[test]
    fn test_FUNC_SHELL_003_detect_in_variable_context() {
        // Should detect in full variable assignment context
        let value = "SESSION_ID := $RANDOM";
        assert!(detect_random(value));
    }

    // Edge cases
    #[test]
    fn test_FUNC_SHELL_003_empty_string() {
        assert!(!detect_random(""));
    }

    #[test]
    fn test_FUNC_SHELL_003_random_text_not_variable() {
        // Just the word "random" is not a problem
        assert!(!detect_random("# Generate random numbers"));
    }

    #[test]
    fn test_FUNC_SHELL_003_randomize_not_detected() {
        // "randomize" or "RANDOMIZE" should not be detected
        assert!(!detect_random("randomize_data()"));
    }

    #[test]
    fn test_FUNC_SHELL_003_multiple_randoms() {
        // Should detect if ANY contain $RANDOM
        assert!(detect_random("A=fixed B=$RANDOM"));
    }

    #[test]
    fn test_FUNC_SHELL_003_case_sensitive() {
        // Should be case-sensitive - $random is not the same as $RANDOM
        assert!(!detect_random("$random"));
    }

    #[test]
    fn test_FUNC_SHELL_003_detect_both_variants() {
        // Should detect both $RANDOM and $$RANDOM
        assert!(detect_random("$RANDOM"));
        assert!(detect_random("$$RANDOM"));
    }

    // Mutation-killing tests
    #[test]
    fn test_FUNC_SHELL_003_mut_contains_must_check_substring() {
        // Ensures we use .contains() not .eq()
        assert!(detect_random("prefix $RANDOM suffix"));
    }

    #[test]
    fn test_FUNC_SHELL_003_mut_exact_pattern() {
        // Ensures we check for "$RANDOM" not just "RANDOM"
        assert!(!detect_random("RANDOM_SEED := 42"));
    }

    #[test]
    fn test_FUNC_SHELL_003_mut_non_empty_check() {
        // Ensures we don't crash on empty strings
        let result = detect_random("");
        assert!(!result);
    }

    // Property-based tests for $RANDOM detection (FUNC-SHELL-003)
    #[cfg(test)]
    mod random_property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn prop_FUNC_SHELL_003_any_string_no_panic(s in "\\PC*") {
                // Should never panic on any string
                let _ = detect_random(&s);
            }

            #[test]
            fn prop_FUNC_SHELL_003_random_always_detected(
                prefix in "[A-Z_]{3,10}"
            ) {
                // $RANDOM should always be detected
                let input = format!("{} := $RANDOM", prefix);
                prop_assert!(detect_random(&input));
            }

            #[test]
            fn prop_FUNC_SHELL_003_double_dollar_random_always_detected(
                prefix in "[A-Z_]{3,10}"
            ) {
                // $$RANDOM should always be detected
                let input = format!("{} := $$RANDOM", prefix);
                prop_assert!(detect_random(&input));
            }

            #[test]
            fn prop_FUNC_SHELL_003_no_dollar_never_detected(
                s in "[^$]*"
            ) {
                // Strings without $ should never be detected
                prop_assert!(!detect_random(&s));
            }

            #[test]
            fn prop_FUNC_SHELL_003_deterministic(s in "\\PC*") {
                // Same input always gives same output
                let result1 = detect_random(&s);
                let result2 = detect_random(&s);
                prop_assert_eq!(result1, result2);
            }
        }
    }

    // Integration tests for analyze_makefile() with $RANDOM (FUNC-SHELL-003)
    #[test]
    fn test_FUNC_SHELL_003_analyze_detects_random() {
        use crate::make_parser::parse_makefile;

        let makefile = "BUILD_ID := $RANDOM";
        let ast = parse_makefile(makefile).unwrap();
        let issues = analyze_makefile(&ast);

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].rule, "NO_RANDOM");
        assert_eq!(issues[0].severity, IssueSeverity::Critical);
        assert!(issues[0].message.contains("BUILD_ID"));
        assert!(issues[0].suggestion.is_some());
    }

    #[test]
    fn test_FUNC_SHELL_003_analyze_detects_double_dollar_random() {
        use crate::make_parser::parse_makefile;

        let makefile = "SESSION := $(shell echo $$RANDOM)";
        let ast = parse_makefile(makefile).unwrap();
        let issues = analyze_makefile(&ast);

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].rule, "NO_RANDOM");
        assert_eq!(issues[0].severity, IssueSeverity::Critical);
    }

    #[test]
    fn test_FUNC_SHELL_003_analyze_no_issues_clean_makefile() {
        use crate::make_parser::parse_makefile;

        let makefile = "BUILD_ID := 42\nVERSION := 1.0.0";
        let ast = parse_makefile(makefile).unwrap();
        let issues = analyze_makefile(&ast);

        assert_eq!(issues.len(), 0);
    }

    #[test]
    fn test_FUNC_SHELL_003_analyze_multiple_issues() {
        use crate::make_parser::parse_makefile;

        let makefile = r#"SESSION_ID := $RANDOM
BUILD_ID := $$RANDOM
VERSION := 1.0.0"#;
        let ast = parse_makefile(makefile).unwrap();
        let issues = analyze_makefile(&ast);

        // Should detect both $RANDOM occurrences
        let random_issues: Vec<_> = issues.iter().filter(|i| i.rule == "NO_RANDOM").collect();
        assert_eq!(random_issues.len(), 2);
        assert!(random_issues[0].message.contains("SESSION_ID"));
        assert!(random_issues[1].message.contains("BUILD_ID"));
    }

    #[test]
    fn test_FUNC_SHELL_003_analyze_mixed_issues() {
        use crate::make_parser::parse_makefile;

        // Mix of $RANDOM (Critical) and $(wildcard) (High)
        let makefile = r#"BUILD_ID := $RANDOM
SOURCES := $(wildcard *.c)"#;
        let ast = parse_makefile(makefile).unwrap();
        let issues = analyze_makefile(&ast);

        assert_eq!(issues.len(), 2);
        // First issue is $RANDOM (Critical)
        assert_eq!(issues[0].rule, "NO_RANDOM");
        assert_eq!(issues[0].severity, IssueSeverity::Critical);
        // Second issue is wildcard (High)
        assert_eq!(issues[1].rule, "NO_WILDCARD");
        assert_eq!(issues[1].severity, IssueSeverity::High);
    }

    #[test]
    fn test_FUNC_SHELL_003_analyze_suggestion_format() {
        use crate::make_parser::parse_makefile;

        let makefile = "RANDOM_ID := $RANDOM";
        let ast = parse_makefile(makefile).unwrap();
        let issues = analyze_makefile(&ast);

        assert_eq!(issues.len(), 1);
        let suggestion = issues[0].suggestion.as_ref().unwrap();
        assert!(suggestion.contains("RANDOM_ID"));
        assert!(suggestion.contains(":="));
        assert!(suggestion.contains("42"));
    }
}
