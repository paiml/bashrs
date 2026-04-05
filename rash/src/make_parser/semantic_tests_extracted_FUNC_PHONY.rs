
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

include!("semantic_tests_extracted_FUNC_PHONY_FUNC.rs");
