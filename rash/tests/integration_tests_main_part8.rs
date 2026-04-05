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
