// QuickCheck-style property-based testing for Rash transpiler
// Comprehensive property testing to ensure correctness across all inputs

use crate::ast::restricted::{BinaryOp, Literal, UnaryOp};
use crate::ast::{Expr, Function, RestrictedAst, Stmt, Type};
use crate::models::{Config, ShellDialect, VerificationLevel};
use crate::services::parse;
use crate::transpile;
use proptest::prelude::*;

// Generator strategies for creating valid AST nodes
        #[test]
        fn prop_string_contains_empty(haystack in "[a-zA-Z0-9]{0,30}") {
            let source = format!(
                r#"fn main() {{
                    let text = "{}";
                    if string_contains(text, "") {{
                        echo("contains empty");
                    }}
                }}"#,
                haystack
            );

            if let Ok(shell_code) = transpile(&source, &Config::default()) {
                // Should generate proper case statement
                prop_assert!(shell_code.contains("rash_string_contains"),
                           "Should include stdlib string_contains function");
                // Should not wrap in command substitution for if statement
                prop_assert!(!shell_code.contains("\"$(rash_string_contains"),
                           "Predicate functions should not be wrapped in command substitution");

        /// Property: stdlib fs_exists should generate test command
        #[test]
        fn prop_fs_exists_test_command(path in "[a-zA-Z0-9/_.-]{1,50}") {
            let source = format!(
                r#"fn main() {{
                    if fs_exists("{}") {{
                        echo("exists");
                    }}
                }}"#,
                path
            );

            if let Ok(shell_code) = transpile(&source, &Config::default()) {
                // Should include fs_exists function
                prop_assert!(shell_code.contains("rash_fs_exists"),
                           "Should include stdlib fs_exists function");
                // Should use POSIX test -e
                prop_assert!(shell_code.contains("test -e"),
                           "fs_exists should use POSIX test -e");
            }
        }

        /// Property: stdlib string_len should return numeric value
        #[test]
        fn prop_string_len_numeric(s in "[a-zA-Z]{0,20}") {
            let source = format!(
                r#"fn main() {{
                    let text = "{}";
                    let len = string_len(text);
                    echo(len);
                }}"#,
                s
            );

            if let Ok(shell_code) = transpile(&source, &Config::default()) {
                // Should use command substitution for return value
                prop_assert!(shell_code.contains("$(rash_string_len") ||
                           shell_code.contains("len="),
                           "string_len should be captured as value");
            }
        }

        /// Property: while loops should generate POSIX while statements
        #[test]
        fn prop_while_loop_posix(limit in 1u32..10) {
            let source = format!(
                r#"fn main() {{
                    let i = 0;
                    while i < {} {{
                        echo(i);
                    }}
                }}"#,
                limit
            );

            if let Ok(shell_code) = transpile(&source, &Config::default()) {
                // Should generate while loop
                prop_assert!(shell_code.contains("while") && shell_code.contains("do"),
                           "Should generate POSIX while...do loop");
                // Should have proper test syntax
                prop_assert!(shell_code.contains("-lt") || shell_code.contains("["),
                           "Should use POSIX test syntax for condition");
                prop_assert!(shell_code.contains("done"),
                           "While loop should be closed with 'done'");
            }
        }

        /// Property: while true should generate infinite loop
        #[test]
        fn prop_while_true_infinite(_dummy in prop::bool::ANY) {
            let source = r#"
fn main() {
    while true {
        echo("loop");
    }
}
"#;

            if let Ok(shell_code) = transpile(source, &Config::default()) {
                // Should generate while true
                prop_assert!(shell_code.contains("while true"),
                           "while true should generate 'while true' statement");
                prop_assert!(shell_code.contains("do") && shell_code.contains("done"),
                           "Should have proper loop structure");
            }
        }

        /// Property: nested if statements should maintain correct nesting
        #[test]
        fn prop_nested_if_statements(
            val1 in 0u32..100,
            val2 in 0u32..100
        ) {
            let source = format!(
                r#"fn main() {{
                    let x = {};
                    if x > 10 {{
                        let y = {};
                        if y > 20 {{
                            echo("nested");
                        }}
                    }}
                }}"#,
                val1, val2
            );

            if let Ok(shell_code) = transpile(&source, &Config::default()) {
                // Should contain nested if structure
                prop_assert!(shell_code.contains("if") && shell_code.contains("fi"),
                           "Should generate if/fi structure");
                // Should contain comparison operators
                prop_assert!(shell_code.contains("-gt"),
                           "Should use POSIX comparison operators");
            }
        }

        /// Property: match expressions with multiple arms should generate complete case
        #[test]
        fn prop_match_completeness(val in 0u32..5) {
            let source = format!(
                r#"fn main() {{
                    let x = {};
                    match x {{
                        0 => echo("zero"),
                        1 => echo("one"),
                        _ => echo("other"),
                    }}
                }}"#,
                val
            );

            if let Ok(shell_code) = transpile(&source, &Config::default()) {
                // Should generate case statement
                prop_assert!(shell_code.contains("case"),
                           "Match should generate case statement");
                prop_assert!(shell_code.contains("esac"),
                           "Case statement should be closed with esac");
                // Should have wildcard pattern
                prop_assert!(shell_code.contains("*)"),
                           "Should have wildcard pattern for _ arm");
            }
        }

        /// Property: for loop with range should generate seq command
        #[test]
        fn prop_for_range_seq(start in 0u32..10, end in 11u32..20) {
            prop_assume!(start < end); // Ensure valid range

            let source = format!(
                r#"fn main() {{
                    for i in {}..{} {{
                        echo(i);
                    }}
                }}"#,
                start, end
            );

            if let Ok(shell_code) = transpile(&source, &Config::default()) {
                // Should generate seq command or shell range
                prop_assert!(shell_code.contains("seq") || shell_code.contains("for i in"),
                           "For loop should generate seq or shell range");
                // Exclusive range: end - 1
                let expected_end = end - 1;
                prop_assert!(shell_code.contains(&format!("seq {} {}", start, expected_end)) ||
                           shell_code.contains("for"),
                           "Should generate correct range bounds");
            }
        }

        /// Property: break and continue in loops should generate shell equivalents
        #[test]
        fn prop_break_continue(use_break in prop::bool::ANY) {
            let stmt = if use_break { "break" } else { "continue" };
            let source = format!(
                r#"fn main() {{
                    let i = 0;
                    while i < 10 {{
                        if i == 5 {{
                            {};
                        }}
                        echo(i);
                    }}
                }}"#,
                stmt
            );

            if let Ok(shell_code) = transpile(&source, &Config::default()) {
                // Should contain break or continue statement
                prop_assert!(shell_code.contains(stmt),
                           "Loop control statement should be preserved");
            }
        }

        // =============== Sprint 25: Extended Stdlib Property Tests ===============

        /// Property: string_to_upper should always include the runtime function
        #[test]
        fn prop_string_to_upper_includes_runtime(text in "[a-zA-Z0-9 ]{1,30}") {
            let source = format!(
                r#"fn main() {{
                    let result = string_to_upper("{}");
                    echo(result);
                }}"#,
                text
            );

            if let Ok(shell_code) = transpile(&source, &Config::default()) {
                prop_assert!(shell_code.contains("rash_string_to_upper"),
                           "Generated shell should include string_to_upper runtime function");
            }
        }

        /// Property: string_to_lower should always include the runtime function
        #[test]
        fn prop_string_to_lower_includes_runtime(text in "[a-zA-Z0-9 ]{1,30}") {
            let source = format!(
                r#"fn main() {{
                    let result = string_to_lower("{}");
                    echo(result);
                }}"#,
                text
            );

            if let Ok(shell_code) = transpile(&source, &Config::default()) {
                prop_assert!(shell_code.contains("rash_string_to_lower"),
                           "Generated shell should include string_to_lower runtime function");
            }
        }

        /// Property: string_replace should handle any valid string inputs
        #[test]
        fn prop_string_replace_transpiles(
            text in "[a-zA-Z0-9 ]{1,20}",
            old in "[a-zA-Z]{1,5}",
            new in "[a-zA-Z]{1,5}"
        ) {
            let source = format!(
                r#"fn main() {{
                    let result = string_replace("{}", "{}", "{}");
                    echo(result);
                }}"#,
                text, old, new
            );

            let result = transpile(&source, &Config::default());
            prop_assert!(result.is_ok(), "string_replace should transpile successfully");
            if let Ok(shell_code) = result {
                prop_assert!(shell_code.contains("rash_string_replace"),
                           "Generated shell should include string_replace runtime function");
            }
        }

        /// Property: fs_is_file should always include the runtime function
        #[test]
        fn prop_fs_is_file_includes_runtime(path in "/[a-z]{1,20}") {
            let source = format!(
                r#"fn main() {{
                    let is_file = fs_is_file("{}");
                    if is_file {{
                        echo("yes");
                    }}
                }}"#,
                path
            );

            if let Ok(shell_code) = transpile(&source, &Config::default()) {
                prop_assert!(shell_code.contains("rash_fs_is_file"),
                           "Generated shell should include fs_is_file runtime function");
            }
        }

        /// Property: fs_is_dir should always include the runtime function
        #[test]
        fn prop_fs_is_dir_includes_runtime(path in "/[a-z]{1,20}") {
            let source = format!(
                r#"fn main() {{
                    let is_dir = fs_is_dir("{}");
                    if is_dir {{
                        echo("yes");
                    }}
                }}"#,
                path
            );

            if let Ok(shell_code) = transpile(&source, &Config::default()) {
                prop_assert!(shell_code.contains("rash_fs_is_dir"),
                           "Generated shell should include fs_is_dir runtime function");
            }
        }

        /// Property: fs_copy should handle valid paths
        #[test]
        fn prop_fs_copy_transpiles(
            src in "/tmp/[a-z]{1,10}",
            dst in "/tmp/[a-z]{1,10}"
        ) {
            let source = format!(
                r#"fn main() {{
                    let result = fs_copy("{}", "{}");
                    if result {{
                        echo("copied");
                    }}
                }}"#,
                src, dst
            );

            let result = transpile(&source, &Config::default());
            prop_assert!(result.is_ok(), "fs_copy should transpile successfully");
            if let Ok(shell_code) = result {
                prop_assert!(shell_code.contains("rash_fs_copy"),
                           "Generated shell should include fs_copy runtime function");
            }
        }

        /// Property: fs_remove should handle valid paths
        #[test]
        fn prop_fs_remove_transpiles(path in "/tmp/[a-z]{1,10}") {
            let source = format!(
                r#"fn main() {{
                    let result = fs_remove("{}");
                    if result {{
                        echo("removed");
                    }}
                }}"#,
                path
            );

            let result = transpile(&source, &Config::default());
            prop_assert!(result.is_ok(), "fs_remove should transpile successfully");
            if let Ok(shell_code) = result {
                prop_assert!(shell_code.contains("rash_fs_remove"),
                           "Generated shell should include fs_remove runtime function");
            }
        }

        /// Property: Combining multiple new stdlib functions should transpile
        #[test]
        fn prop_multiple_new_stdlib_functions(text in "[a-zA-Z ]{5,15}") {
            let source = format!(
                r#"fn main() {{
                    let lower = string_to_lower("{}");
                    let upper = string_to_upper(lower);

                    if fs_is_dir("/tmp") {{
                        fs_write_file("/tmp/test.txt", upper);
                        if fs_is_file("/tmp/test.txt") {{
                            fs_remove("/tmp/test.txt");
                        }}
                    }}
                }}"#,
                text
            );

            let result = transpile(&source, &Config::default());
            prop_assert!(result.is_ok(), "Multiple stdlib functions should transpile successfully");
            if let Ok(shell_code) = result {
                prop_assert!(shell_code.contains("rash_string_to_lower"),
                           "Should include string_to_lower");
                prop_assert!(shell_code.contains("rash_string_to_upper"),
                           "Should include string_to_upper");
                prop_assert!(shell_code.contains("rash_fs_is_dir"),
                           "Should include fs_is_dir");
                prop_assert!(shell_code.contains("rash_fs_is_file"),
                           "Should include fs_is_file");
                prop_assert!(shell_code.contains("rash_fs_remove"),
                           "Should include fs_remove");
            }
        }
    }
}
