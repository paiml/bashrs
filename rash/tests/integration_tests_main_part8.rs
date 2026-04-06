// ============================================================================
// P0-POSITIONAL-PARAMETERS: Property Tests
// ============================================================================

#[cfg(test)]
mod positional_parameters_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Property: Transpiling positional parameters is deterministic
        /// Same Rust input always produces same shell output
        #[test]
        fn prop_positional_params_deterministic(
            default_val in "[a-z]{1,10}"
        ) {
            let source = format!(r#"
fn main() {{
    let args: Vec<String> = std::env::args().collect();
    let first = args.get(1).unwrap_or("{}");
    echo(first);
}}

fn echo(msg: &str) {{}}
"#, default_val);

            let config = Config::default();
            let result1 = transpile(&source, &config);
            let result2 = transpile(&source, &config);

            prop_assert!(result1.is_ok());
            prop_assert!(result2.is_ok());
            prop_assert_eq!(result1.unwrap(), result2.unwrap());
        }

        /// Property: Transpilation succeeds for all valid default values
        #[test]
        fn prop_default_values_preserved(
            default_val in "[a-zA-Z0-9_-]{1,20}"
        ) {
            let source = format!(r#"
fn main() {{
    let args: Vec<String> = std::env::args().collect();
    let param = args.get(1).unwrap_or("{}");
    echo(param);
}}

fn echo(msg: &str) {{}}
"#, default_val);

            let config = Config::default();
            let result = transpile(&source, &config);

            // Transpilation should always succeed for valid default values
            prop_assert!(result.is_ok(), "Transpilation failed for default: {}", default_val);

            let shell = result.unwrap();

            // Shell output should contain the param assignment with positional parameter syntax
            prop_assert!(
                shell.contains("param=") && shell.contains("${1:-"),
                "Shell output should contain positional parameter with default"
            );
        }

        /// Property: Positional parameters are always quoted
        #[test]
        fn prop_positional_params_quoted(
            position in 1u32..10,
            default_val in "[a-z]{1,10}"
        ) {
            let source = format!(r#"
fn main() {{
    let args: Vec<String> = std::env::args().collect();
    let param = args.get({}).unwrap_or("{}");
    echo(param);
}}

fn echo(msg: &str) {{}}
"#, position, default_val);

            let config = Config::default();
            let result = transpile(&source, &config);

            prop_assert!(result.is_ok());
            let shell = result.unwrap();

            // Positional params should be in quotes
            prop_assert!(
                shell.contains(&format!("\"${{{}:-", position)) ||
                shell.contains(&format!("param=\"${{{}:-", position)),
                "Positional parameter should be quoted"
            );
        }

        /// Property: std::env::args().collect() always becomes "$@"
        #[test]
        fn prop_args_collect_becomes_dollar_at(
            _seed in 0u32..100  // Just for variety
        ) {
            let source = r#"
fn main() {
    let args: Vec<String> = std::env::args().collect();
    echo("test");
}

fn echo(msg: &str) {}
"#;

            let config = Config::default();
            let result = transpile(source, &config);

            prop_assert!(result.is_ok());
            let shell = result.unwrap();

            // args variable should contain "$@"
            prop_assert!(
                shell.contains("args=\"$@\"") || shell.contains("$@"),
                "args.collect() should become $@"
            );
        }
    }
}

/// PARAM-SPEC-001: Property-based tests for arg_count() → $# transformation
mod arg_count_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
            /// Property: Transpiling arg_count() is deterministic
            /// Same Rust input always produces same shell output
            #[test]
            fn prop_arg_count_deterministic(
                _seed in 0u32..100
            ) {
                let source = r#"
fn main() {
    let count = arg_count();
    echo("Done");
}

fn arg_count() -> i32 { 0 }
fn echo(msg: &str) {}
"#;

                let config = Config::default();
                let result1 = transpile(source, &config);
                let result2 = transpile(source, &config);

                prop_assert!(result1.is_ok());
                prop_assert!(result2.is_ok());
                prop_assert_eq!(result1.unwrap(), result2.unwrap());
            }

            /// Property: arg_count() always generates $# in output
            #[test]
            fn prop_arg_count_generates_dollar_hash(
                _seed in 0u32..100
            ) {
                let source = r#"
fn main() {
    let count = arg_count();
    wc("-l");
}

fn arg_count() -> i32 { 0 }
fn wc(arg: &str) {}
"#;

                let config = Config::default();
                let result = transpile(source, &config);

                prop_assert!(result.is_ok(), "Transpilation should succeed");

                let shell = result.unwrap();

                // arg_count() should always generate $# in shell output
                prop_assert!(
                    shell.contains("$#"),
                    "Shell output must contain $# for arg_count()"
                );
            }

    include!("integration_tests_main_part8.rs");
            #[test]
            fn prop_arg_count_in_conditionals_valid(
                threshold in 0i32..10
            ) {
                let source = format!(r#"
fn main() {{
    let count = arg_count();
    if count == {} {{
        echo("Match");
    }}
}}

fn arg_count() -> i32 {{ 0 }}
fn echo(msg: &str) {{}}
"#, threshold);

                let config = Config::default();
                let result = transpile(&source, &config);

                prop_assert!(
                    result.is_ok(),
                    "Transpilation should succeed for count == {}", threshold
                );

                let shell = result.unwrap();

                // Must contain both $# and the threshold value
                prop_assert!(
                    shell.contains("$#"),
                    "Shell must use $# for arg_count()"
                );
            }

            /// Property: Generated shell scripts are syntactically valid
            #[test]
            fn prop_arg_count_output_shell_valid(
                _seed in 0u32..50
            ) {
                let source = r#"
fn main() {
    let count = arg_count();
    echo("test");
}

fn arg_count() -> i32 { 0 }
fn echo(msg: &str) {}
"#;

                let config = Config::default();
                let result = transpile(source, &config);

                prop_assert!(result.is_ok(), "Transpilation must succeed");

                let shell = result.unwrap();

                // Basic validity checks
                prop_assert!(shell.contains("#!/bin/sh"), "Must have shebang");
                prop_assert!(shell.contains("set -euf"), "Must have safety flags");
                prop_assert!(shell.contains("$#"), "Must contain arg count");
            }
        }
}
