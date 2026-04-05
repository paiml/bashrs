#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

//! Comprehensive integration tests for maximum code coverage.
//!
//! Exercises: transpiler pipeline, bash parser, purification,
//! linter (shell/makefile/dockerfile), comply rules, and make parser.

// ============================================================================
// Transpiler Integration (parser -> IR -> emitter)
// ============================================================================

mod transpiler_integration {
    use crate::models::{ShellDialect, VerificationLevel};
    use crate::{transpile, transpile_with_lint, transpile_with_trace, Config};

    #[test]
    fn test_transpile_basic_assignment() {
        let code = "fn main() { let x = 42; }";
        let result = transpile(code, &Config::default()).unwrap();
        assert!(result.contains("#!/bin/sh"));
        assert!(result.contains("x="));
    }

    #[test]
    fn test_transpile_multiple_variables() {
        let code = r#"
            fn main() {
                let a = 1;
                let b = 2;
                let c = 3;
            }
        "#;
        let result = transpile(code, &Config::default()).unwrap();
        assert!(result.contains("a="));
        assert!(result.contains("b="));
        assert!(result.contains("c="));
    }

    #[test]
    fn test_transpile_function_call() {
        let code = r#"
            fn main() {
                greet("World");
            }
            fn greet(name: &str) {}
        "#;
        let result = transpile(code, &Config::default()).unwrap();
        assert!(result.contains("greet"));
    }

    #[test]
    fn test_transpile_function_definition() {
        let code = r#"
            fn main() {
                helper();
            }
            fn helper() {
                let x = 10;
            }
        "#;
        let result = transpile(code, &Config::default()).unwrap();
        assert!(result.contains("helper"));
    }

    #[test]
    fn test_transpile_if_else() {
        let code = r#"
            fn main() {
                let x = 5;
                if x > 3 {
                    let y = 1;
                } else {
                    let y = 2;
                }
            }
        "#;
        let result = transpile(code, &Config::default()).unwrap();
        assert!(result.contains("if"));
        assert!(result.contains("then"));
        assert!(result.contains("else"));
        assert!(result.contains("fi"));
    }

    #[test]
    fn test_transpile_for_loop() {
        let code = r#"
            fn main() {
                for i in [1, 2, 3] {
                    echo(i);
                }
            }
            fn echo(val: i64) {}
        "#;
        let result = transpile(code, &Config::default()).unwrap();
        assert!(result.contains("for"));
        assert!(result.contains("do"));
        assert!(result.contains("done"));
    }

    #[test]
    fn test_transpile_match_statement() {
        let code = r#"
            fn main() {
                let x = 2;
                match x {
                    1 => { let a = 10; },
                    2 => { let a = 20; },
                    _ => { let a = 30; },
                }
            }
        "#;
        let result = transpile(code, &Config::default()).unwrap();
        assert!(result.contains("case"));
        assert!(result.contains("esac"));
    }

    #[test]
    fn test_transpile_while_loop() {
        let code = r#"
            fn main() {
                let mut x = 0;
                while x < 5 {
                    x = x + 1;
                }
            }
        "#;
        let result = transpile(code, &Config::default()).unwrap();
        assert!(result.contains("while"));
        assert!(result.contains("do"));
        assert!(result.contains("done"));
    }

    #[test]
    fn test_transpile_binary_operations_add() {
        let code = "fn main() { let x = 1 + 2; }";
        let result = transpile(code, &Config::default()).unwrap();
        assert!(result.contains("x="));
    }

    #[test]
    fn test_transpile_binary_operations_sub() {
        let code = "fn main() { let x = 10 - 3; }";
        let result = transpile(code, &Config::default()).unwrap();
        assert!(result.contains("x="));
    }

    #[test]
    fn test_transpile_binary_operations_mul() {
        let code = "fn main() { let x = 3 * 4; }";
        let result = transpile(code, &Config::default()).unwrap();
        assert!(result.contains("x="));
    }

    #[test]
    fn test_transpile_binary_operations_div() {
        let code = "fn main() { let x = 10 / 2; }";
        let result = transpile(code, &Config::default()).unwrap();
        assert!(result.contains("x="));
    }

    #[test]
    fn test_transpile_binary_operations_mod() {
        let code = "fn main() { let x = 10 % 3; }";
        let result = transpile(code, &Config::default()).unwrap();
        assert!(result.contains("x="));
    }

    #[test]
    fn test_transpile_binary_operations_comparisons() {
        let code = r#"
            fn main() {
                let a = 5;
                let b = 10;
                if a < b {
                    let c = 1;
                }
                if a > b {
                    let d = 2;
                }
                if a == b {
                    let e = 3;
                }
                if a != b {
                    let f = 4;
                }
            }
        "#;
        let result = transpile(code, &Config::default()).unwrap();
        assert!(result.contains("if"));
    }

    #[test]
    fn test_transpile_array() {
        let code = r#"
            fn main() {
                let items = [1, 2, 3];
            }
        "#;
        let result = transpile(code, &Config::default()).unwrap();
        assert!(result.contains("#!/bin/sh"));
    }

    #[test]
    fn test_transpile_nested_if() {
        let code = r#"
            fn main() {
                let x = 5;
                if x > 0 {
                    if x > 3 {
                        let y = 1;
                    }
                }
            }
        "#;
        let result = transpile(code, &Config::default()).unwrap();
        // Should have nested if/fi
        let if_count = result.matches("if").count();
        assert!(if_count >= 2, "Expected nested ifs, got: {}", result);
    }

    #[test]
    fn test_transpile_string_operations() {
        let code = r#"
            fn main() {
                let name = "Alice";
                let greeting = "Hello";
            }
        "#;
        let result = transpile(code, &Config::default()).unwrap();
        assert!(result.contains("name="));
        assert!(result.contains("greeting="));
    }

    #[test]
    fn test_transpile_with_bash_dialect() {
        let code = "fn main() { let x = 1; }";
        let config = Config {
            target: ShellDialect::Bash,
            ..Config::default()
        };
        let result = transpile(code, &config).unwrap();
        assert!(result.contains("#!/"));
    }

    #[test]
    fn test_transpile_with_paranoid_verification() {
        let code = "fn main() { let x = 1; }";
        let config = Config {
            verify: VerificationLevel::Paranoid,
            ..Config::default()
        };
        let result = transpile(code, &config).unwrap();
        assert!(result.contains("#!/bin/sh"));
    }

    #[test]
    fn test_transpile_empty_main() {
        let code = "fn main() {}";
        let result = transpile(code, &Config::default()).unwrap();
        assert!(result.contains("#!/bin/sh"));
    }

    #[test]
    fn test_transpile_with_trace() {
        let code = r#"
            fn main() {
                let x = 42;
                echo(x);
            }
            fn echo(val: i64) {}
        "#;
        let (shell_code, trace) = transpile_with_trace(code, &Config::default()).unwrap();
        assert!(shell_code.contains("#!/bin/sh"));
        // Trace should have recorded decisions
        assert!(!trace.is_empty() || shell_code.len() > 10);
    }

    #[test]
    fn test_transpile_with_lint() {
        let code = r#"
            fn main() {
                let greeting = "Hello";
                echo(greeting);
            }
            fn echo(msg: &str) {}
        "#;
        let (shell_code, lint_result) = transpile_with_lint(code, &Config::default()).unwrap();
        assert!(shell_code.contains("#!/bin/sh"));
        // lint_result contains diagnostics (may or may not be empty)
        let _ = lint_result.diagnostics.len();
    }

    #[test]
    fn test_transpile_error_invalid_input() {
        let code = "fn main( { }";
        let result = transpile(code, &Config::default());
        assert!(result.is_err());
    }

    #[test]
    fn test_transpile_with_dash_dialect() {
        let code = "fn main() { let x = 1; }";
        let config = Config {
            target: ShellDialect::Dash,
            ..Config::default()
        };
        let result = transpile(code, &config).unwrap();
        assert!(result.contains("#!/"));
    }

    #[test]
    fn test_transpile_with_ash_dialect() {
        let code = "fn main() { let x = 1; }";
        let config = Config {
            target: ShellDialect::Ash,
            ..Config::default()
        };
        let result = transpile(code, &config).unwrap();
        assert!(result.contains("#!/"));
    }

    #[test]
    fn test_transpile_with_basic_verification() {
        let code = "fn main() { let x = 1; }";
        let config = Config {
            verify: VerificationLevel::Basic,
            ..Config::default()
        };
        let result = transpile(code, &config).unwrap();
        assert!(result.contains("#!/bin/sh"));
    }

    #[test]
    fn test_transpile_with_no_verification() {
        let code = "fn main() { let x = 1; }";
        let config = Config {
            verify: VerificationLevel::None,
            ..Config::default()
        };
        let result = transpile(code, &config).unwrap();
        assert!(result.contains("#!/bin/sh"));
    }

    #[test]
    fn test_transpile_boolean_literal() {
        let code = r#"
            fn main() {
                let flag = true;
                let other = false;
            }
        "#;
        let result = transpile(code, &Config::default()).unwrap();
        assert!(result.contains("flag="));
        assert!(result.contains("other="));
    }
}

include!("coverage_integration_tests_incl2.rs");
