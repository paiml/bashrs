#[cfg(test)]
mod proptest_generative {
    use super::*;
    use proptest::prelude::*;

    // Strategy for bash commands
    fn bash_command() -> impl Strategy<Value = String> {
        prop::sample::select(vec![
            "echo", "ls", "cat", "grep", "sed", "awk", "mkdir", "rm", "cp", "mv", "find", "chmod",
            "chown", "pwd", "cd", "test",
        ])
        .prop_map(|s| s.to_string())
    }

    // Strategy for bash keywords (reserved for future property tests)
    fn _bash_keyword() -> impl Strategy<Value = String> {
        prop::sample::select(vec![
            "if", "then", "else", "elif", "fi", "for", "while", "do", "done", "until", "case",
            "esac", "function",
        ])
        .prop_map(|s| s.to_string())
    }

    // Strategy for simple arguments (no special chars, no shell keywords)
    fn simple_arg() -> impl Strategy<Value = String> {
        "[a-z0-9_-]{1,10}".prop_filter_map("filter shell keywords", |s| {
            const SHELL_KEYWORDS: &[&str] = &[
                "do", "done", "then", "else", "elif", "fi", "for", "while", "until", "case",
                "esac", "if", "in", "select",
            ];
            if SHELL_KEYWORDS.contains(&s.as_str()) {
                None
            } else {
                Some(s)
            }
        })
    }

    // Strategy for file paths
    fn file_path() -> impl Strategy<Value = String> {
        "[a-z0-9_/-]{1,20}".prop_map(|s| s.to_string())
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property: Simple commands always complete
        #[test]
        fn prop_gen_simple_commands_complete(cmd in bash_command(), arg in simple_arg()) {
            let input = format!("{} {}", cmd, arg);
            prop_assert!(!is_incomplete(&input));
        }

        /// Property: Unclosed single quote always incomplete
        #[test]
        fn prop_gen_unclosed_single_quote(cmd in bash_command(), text in simple_arg()) {
            let input = format!("{} '{}", cmd, text);
            prop_assert!(is_incomplete(&input));
        }

        /// Property: Closed single quote complete
        #[test]
        fn prop_gen_closed_single_quote(cmd in bash_command(), text in simple_arg()) {
            let input = format!("{} '{}'", cmd, text);
            prop_assert!(!is_incomplete(&input));
        }

        /// Property: Unclosed double quote always incomplete
        #[test]
        fn prop_gen_unclosed_double_quote(cmd in bash_command(), text in simple_arg()) {
            let input = format!("{} \"{}", cmd, text);
            prop_assert!(is_incomplete(&input));
        }

        /// Property: Closed double quote complete
        #[test]
        fn prop_gen_closed_double_quote(cmd in bash_command(), text in simple_arg()) {
            let input = format!("{} \"{}\"", cmd, text);
            prop_assert!(!is_incomplete(&input));
        }

        /// Property: Unclosed brace always incomplete
        #[test]
        fn prop_gen_unclosed_brace(cmd in bash_command()) {
            let input = format!("{} {{", cmd);
            prop_assert!(is_incomplete(&input));
        }

        /// Property: Balanced braces complete
        #[test]
        fn prop_gen_balanced_braces(cmd in bash_command()) {
            let input = format!("{{ {}; }}", cmd);
            prop_assert!(!is_incomplete(&input));
        }

        /// Property: Unclosed paren always incomplete
        #[test]
        fn prop_gen_unclosed_paren(cmd in bash_command()) {
            let input = format!("({}", cmd);
            prop_assert!(is_incomplete(&input));
        }

        /// Property: Balanced parens complete
        #[test]
        fn prop_gen_balanced_parens(cmd in bash_command()) {
            let input = format!("({})", cmd);
            prop_assert!(!is_incomplete(&input));
        }

        /// Property: Unclosed bracket always incomplete
        #[test]
        fn prop_gen_unclosed_bracket(file in file_path()) {
            let input = format!("[ -f {}", file);
            prop_assert!(is_incomplete(&input));
        }

        /// Property: Balanced brackets complete
        #[test]
        fn prop_gen_balanced_brackets(file in file_path()) {
            let input = format!("[ -f {} ]", file);
            prop_assert!(!is_incomplete(&input));
        }

        /// Property: If without fi incomplete
        #[test]
        fn prop_gen_if_incomplete(cmd in bash_command()) {
            let input = format!("if true; then {}", cmd);
            prop_assert!(is_incomplete(&input));
        }

        /// Property: Complete if statement complete
        #[test]
        fn prop_gen_if_complete(cmd in bash_command()) {
            let input = format!("if true; then {}; fi", cmd);
            prop_assert!(!is_incomplete(&input));
        }

        /// Property: For without done incomplete
        #[test]
        fn prop_gen_for_incomplete(var in simple_arg()) {
            let input = format!("for {} in 1 2 3; do", var);
            prop_assert!(is_incomplete(&input));
        }

        /// Property: Complete for loop complete
        #[test]
        fn prop_gen_for_complete(var in simple_arg(), cmd in bash_command()) {
            let input = format!("for {} in 1 2 3; do {}; done", var, cmd);
            prop_assert!(!is_incomplete(&input));
        }

        /// Property: While without done incomplete
        #[test]
        fn prop_gen_while_incomplete(cmd in bash_command()) {
            let input = format!("while true; do {}", cmd);
            prop_assert!(is_incomplete(&input));
        }

        /// Property: Complete while loop complete
        #[test]
        fn prop_gen_while_complete(cmd in bash_command()) {
            let input = format!("while true; do {}; done", cmd);
            prop_assert!(!is_incomplete(&input));
        }

        /// Property: Case without esac incomplete
        #[test]
        fn prop_gen_case_incomplete(var in simple_arg()) {
            let input = format!("case ${} in", var);
            prop_assert!(is_incomplete(&input));
        }

        /// Property: Complete case statement complete
        #[test]
        fn prop_gen_case_complete(var in simple_arg()) {
            let input = format!("case ${} in foo) echo bar;; esac", var);
            prop_assert!(!is_incomplete(&input));
        }

        /// Property: Backslash continuation always incomplete
        #[test]
        fn prop_gen_backslash_continuation(cmd in bash_command(), arg in simple_arg()) {
            let input = format!("{} {} \\", cmd, arg);
            prop_assert!(is_incomplete(&input));
        }

        /// Property: Empty input always complete
        #[test]
        fn prop_gen_empty_complete(_n in 0u8..100u8) {
            prop_assert!(!is_incomplete(""));
        }

        /// Property: Whitespace-only input always complete
        #[test]
        fn prop_gen_whitespace_complete(n in 1usize..20usize) {
            let input = " ".repeat(n);
            prop_assert!(!is_incomplete(&input));
        }
    }
}
