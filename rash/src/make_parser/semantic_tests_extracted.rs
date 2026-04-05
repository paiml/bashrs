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

    #[test]
    fn test_PHONY_002_is_common_phony_target_clean() {
        assert!(is_common_phony_target("clean"));
    }

    #[test]
    fn test_PHONY_002_is_common_phony_target_install() {
        assert!(is_common_phony_target("install"));
    }

    #[test]
    fn test_PHONY_002_is_common_phony_target_deploy() {
        assert!(is_common_phony_target("deploy"));
    }

    #[test]
    fn test_PHONY_002_is_common_phony_target_build() {
        assert!(is_common_phony_target("build"));
    }

    #[test]
    fn test_PHONY_002_is_common_phony_target_all() {
        assert!(is_common_phony_target("all"));
    }

    #[test]
    fn test_PHONY_002_is_common_phony_target_help() {
        assert!(is_common_phony_target("help"));
    }

    #[test]
    fn test_PHONY_002_not_common_phony_target_file() {
        assert!(!is_common_phony_target("main.o"));
    }

    #[test]
    fn test_PHONY_002_not_common_phony_target_program() {
        assert!(!is_common_phony_target("program"));
    }

    #[test]
    fn test_PHONY_002_empty_string() {
        assert!(!is_common_phony_target(""));
    }

    #[test]
    fn test_PHONY_002_case_sensitive() {
        // Should be case-sensitive - "TEST" is not the same as "test"
        assert!(!is_common_phony_target("TEST"));
    }

    // Mutation-killing tests
    #[test]
    fn test_PHONY_002_mut_contains_check() {
        // Ensures we use .contains() to check the list
        assert!(is_common_phony_target("test"));
        assert!(is_common_phony_target("clean"));
    }

    #[test]
    fn test_PHONY_002_mut_exact_match() {
        // Ensures we match exact target names, not substrings
        assert!(!is_common_phony_target("testing"));
        assert!(!is_common_phony_target("cleanup"));
    }

    #[test]
    fn test_PHONY_002_mut_non_empty_list() {
        // Ensures COMMON_PHONY_TARGETS is not empty
        assert!(is_common_phony_target("test") || is_common_phony_target("clean"));
    }

    // Property-based tests for auto-PHONY detection
    #[cfg(test)]
    mod phony_property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn prop_PHONY_002_any_string_no_panic(s in "\\PC*") {
                // Should never panic on any string
                let _ = is_common_phony_target(&s);
            }

            #[test]
            fn prop_PHONY_002_deterministic(s in "\\PC*") {
                // Same input always gives same output
                let result1 = is_common_phony_target(&s);
                let result2 = is_common_phony_target(&s);
                prop_assert_eq!(result1, result2);
            }

            #[test]
            fn prop_PHONY_002_known_targets_always_detected(
                target in "(test|clean|install|deploy|build|all|help)"
            ) {
                // Known common targets should always be detected
                prop_assert!(is_common_phony_target(&target));
            }

            #[test]
            fn prop_PHONY_002_file_extensions_not_phony(
                ext in "(c|h|o|a|so|rs|py|js|java|go)"
            ) {
                // Files with extensions should not be phony
                let filename = format!("file.{}", ext);
                prop_assert!(!is_common_phony_target(&filename));
            }

            #[test]
            fn prop_PHONY_002_uppercase_not_detected(
                target in "(TEST|CLEAN|INSTALL|DEPLOY|BUILD|ALL|HELP)"
            ) {
                // Uppercase versions should not be detected (case-sensitive)
                prop_assert!(!is_common_phony_target(&target));
            }
        }
    }

    // Unit tests for shell find detection (FUNC-SHELL-002)
    #[test]
    fn test_FUNC_SHELL_002_detect_shell_find_basic() {
        // Should detect $(shell find . -name '*.c')
        assert!(detect_shell_find("$(shell find . -name '*.c')"));
    }

    #[test]
    fn test_FUNC_SHELL_002_detect_shell_find_with_type() {
        // Should detect $(shell find src -type f)
        assert!(detect_shell_find("$(shell find src -type f)"));
    }

    #[test]
    fn test_FUNC_SHELL_002_no_false_positive() {
        // Should NOT detect when no shell find
        assert!(!detect_shell_find("FILES := main.c util.c"));
    }

    #[test]
    fn test_FUNC_SHELL_002_detect_in_variable_context() {
        // Should detect in full variable assignment context
        let value = "FILES := $(shell find src -name '*.c')";
        assert!(detect_shell_find(value));
    }

    // Edge cases
    #[test]
    fn test_FUNC_SHELL_002_empty_string() {
        assert!(!detect_shell_find(""));
    }

    #[test]
    fn test_FUNC_SHELL_002_no_shell_command() {
        assert!(!detect_shell_find("$(CC) -o output"));
    }

    #[test]
    fn test_FUNC_SHELL_002_shell_but_not_find() {
        assert!(!detect_shell_find("$(shell pwd)"));
    }

    #[test]
    fn test_FUNC_SHELL_002_multiple_shell_commands() {
        // Should detect if ANY contain shell find
        assert!(detect_shell_find(
            "A=$(shell pwd) B=$(shell find . -name '*.c')"
        ));
    }

    #[test]
    fn test_FUNC_SHELL_002_find_without_shell() {
        // "find" alone is not a problem
        assert!(!detect_shell_find("# Use find to locate files"));
    }

    #[test]
    fn test_FUNC_SHELL_002_case_sensitive() {
        // Should be case-sensitive (shell commands are case-sensitive)
        assert!(!detect_shell_find("$(SHELL FIND)"));
    }

    // Mutation-killing tests
    #[test]
    fn test_FUNC_SHELL_002_mut_contains_must_check_substring() {
        // Ensures we use .contains() not .eq()
        assert!(detect_shell_find(
            "prefix $(shell find . -name '*.c') suffix"
        ));
    }

    #[test]
    fn test_FUNC_SHELL_002_mut_exact_pattern() {
        // Ensures we check for "$(shell find" not just "find"
        assert!(!detect_shell_find("findutils"));
    }

    #[test]
    fn test_FUNC_SHELL_002_mut_non_empty_check() {
        // Ensures we don't crash on empty strings
        let result = detect_shell_find("");
        assert!(!result);
    }

    // Property-based tests for shell find detection (FUNC-SHELL-002)
    #[cfg(test)]
    mod shell_find_property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn prop_FUNC_SHELL_002_any_string_no_panic(s in "\\PC*") {
                // Should never panic on any string
                let _ = detect_shell_find(&s);
            }

            #[test]
            fn prop_FUNC_SHELL_002_shell_find_always_detected(
                args in "[a-zA-Z0-9/. -]*"
            ) {
                let input = format!("$(shell find {})", args);
                prop_assert!(detect_shell_find(&input));
            }

            #[test]
            fn prop_FUNC_SHELL_002_no_dollar_never_detected(
                s in "[^$]*"
            ) {
                // Strings without $ should never be detected
                prop_assert!(!detect_shell_find(&s));
            }

            #[test]
            fn prop_FUNC_SHELL_002_deterministic(s in "\\PC*") {
                // Same input always gives same output
                let result1 = detect_shell_find(&s);
                let result2 = detect_shell_find(&s);
                prop_assert_eq!(result1, result2);
            }

            #[test]
            fn prop_FUNC_SHELL_002_shell_without_find_not_detected(
                cmd in "(pwd|date|echo|ls|cat|grep|awk|sed)"
            ) {
                // $(shell <non-find-command>) should not be detected
                let input = format!("$(shell {})", cmd);
                prop_assert!(!detect_shell_find(&input));
            }
        }
    }

    // Integration tests for analyze_makefile() with shell find (FUNC-SHELL-002)
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
