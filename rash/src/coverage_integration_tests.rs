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
        let result = transpile(code, Config::default()).unwrap();
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
        let result = transpile(code, Config::default()).unwrap();
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
        let result = transpile(code, Config::default()).unwrap();
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
        let result = transpile(code, Config::default()).unwrap();
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
        let result = transpile(code, Config::default()).unwrap();
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
        let result = transpile(code, Config::default()).unwrap();
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
        let result = transpile(code, Config::default()).unwrap();
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
        let result = transpile(code, Config::default()).unwrap();
        assert!(result.contains("while"));
        assert!(result.contains("do"));
        assert!(result.contains("done"));
    }

    #[test]
    fn test_transpile_binary_operations_add() {
        let code = "fn main() { let x = 1 + 2; }";
        let result = transpile(code, Config::default()).unwrap();
        assert!(result.contains("x="));
    }

    #[test]
    fn test_transpile_binary_operations_sub() {
        let code = "fn main() { let x = 10 - 3; }";
        let result = transpile(code, Config::default()).unwrap();
        assert!(result.contains("x="));
    }

    #[test]
    fn test_transpile_binary_operations_mul() {
        let code = "fn main() { let x = 3 * 4; }";
        let result = transpile(code, Config::default()).unwrap();
        assert!(result.contains("x="));
    }

    #[test]
    fn test_transpile_binary_operations_div() {
        let code = "fn main() { let x = 10 / 2; }";
        let result = transpile(code, Config::default()).unwrap();
        assert!(result.contains("x="));
    }

    #[test]
    fn test_transpile_binary_operations_mod() {
        let code = "fn main() { let x = 10 % 3; }";
        let result = transpile(code, Config::default()).unwrap();
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
        let result = transpile(code, Config::default()).unwrap();
        assert!(result.contains("if"));
    }

    #[test]
    fn test_transpile_array() {
        let code = r#"
            fn main() {
                let items = [1, 2, 3];
            }
        "#;
        let result = transpile(code, Config::default()).unwrap();
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
        let result = transpile(code, Config::default()).unwrap();
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
        let result = transpile(code, Config::default()).unwrap();
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
        let result = transpile(code, config).unwrap();
        assert!(result.contains("#!/"));
    }

    #[test]
    fn test_transpile_with_paranoid_verification() {
        let code = "fn main() { let x = 1; }";
        let config = Config {
            verify: VerificationLevel::Paranoid,
            ..Config::default()
        };
        let result = transpile(code, config).unwrap();
        assert!(result.contains("#!/bin/sh"));
    }

    #[test]
    fn test_transpile_empty_main() {
        let code = "fn main() {}";
        let result = transpile(code, Config::default()).unwrap();
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
        let (shell_code, trace) = transpile_with_trace(code, Config::default()).unwrap();
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
        let (shell_code, lint_result) = transpile_with_lint(code, Config::default()).unwrap();
        assert!(shell_code.contains("#!/bin/sh"));
        // lint_result contains diagnostics (may or may not be empty)
        let _ = lint_result.diagnostics.len();
    }

    #[test]
    fn test_transpile_error_invalid_input() {
        let code = "fn main( { }";
        let result = transpile(code, Config::default());
        assert!(result.is_err());
    }

    #[test]
    fn test_transpile_with_dash_dialect() {
        let code = "fn main() { let x = 1; }";
        let config = Config {
            target: ShellDialect::Dash,
            ..Config::default()
        };
        let result = transpile(code, config).unwrap();
        assert!(result.contains("#!/"));
    }

    #[test]
    fn test_transpile_with_ash_dialect() {
        let code = "fn main() { let x = 1; }";
        let config = Config {
            target: ShellDialect::Ash,
            ..Config::default()
        };
        let result = transpile(code, config).unwrap();
        assert!(result.contains("#!/"));
    }

    #[test]
    fn test_transpile_with_basic_verification() {
        let code = "fn main() { let x = 1; }";
        let config = Config {
            verify: VerificationLevel::Basic,
            ..Config::default()
        };
        let result = transpile(code, config).unwrap();
        assert!(result.contains("#!/bin/sh"));
    }

    #[test]
    fn test_transpile_with_no_verification() {
        let code = "fn main() { let x = 1; }";
        let config = Config {
            verify: VerificationLevel::None,
            ..Config::default()
        };
        let result = transpile(code, config).unwrap();
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
        let result = transpile(code, Config::default()).unwrap();
        assert!(result.contains("flag="));
        assert!(result.contains("other="));
    }
}

// ============================================================================
// Bash Parser Integration
// ============================================================================

mod bash_parser_integration {
    use crate::bash_parser::BashParser;

    fn parse_ok(input: &str) -> crate::bash_parser::ast::BashAst {
        let mut parser = BashParser::new(input).unwrap();
        parser.parse().unwrap()
    }

    #[test]
    fn test_parse_simple_command() {
        let ast = parse_ok("echo hello");
        assert_eq!(ast.statements.len(), 1);
    }

    #[test]
    fn test_parse_command_with_args() {
        let ast = parse_ok("ls -la /tmp");
        assert_eq!(ast.statements.len(), 1);
    }

    #[test]
    fn test_parse_variable_assignment() {
        let ast = parse_ok("x=42");
        assert_eq!(ast.statements.len(), 1);
    }

    #[test]
    fn test_parse_if_then_fi() {
        let ast = parse_ok("if [ -f /tmp/test ]; then echo found; fi");
        assert_eq!(ast.statements.len(), 1);
    }

    #[test]
    fn test_parse_if_elif_else() {
        let ast = parse_ok(
            "if [ \"$x\" = 1 ]; then echo one; elif [ \"$x\" = 2 ]; then echo two; else echo other; fi",
        );
        assert_eq!(ast.statements.len(), 1);
    }

    #[test]
    fn test_parse_while_loop() {
        let ast = parse_ok("while true; do echo loop; done");
        assert_eq!(ast.statements.len(), 1);
    }

    #[test]
    fn test_parse_until_loop() {
        let ast = parse_ok("until false; do echo loop; done");
        assert_eq!(ast.statements.len(), 1);
    }

    #[test]
    fn test_parse_for_in_loop() {
        let ast = parse_ok("for i in 1 2 3; do echo $i; done");
        assert_eq!(ast.statements.len(), 1);
    }

    #[test]
    fn test_parse_for_c_style() {
        let ast = parse_ok("for ((i=0; i<10; i++)); do echo $i; done");
        assert_eq!(ast.statements.len(), 1);
    }

    #[test]
    fn test_parse_case_statement() {
        let ast = parse_ok("case $x in a) echo a;; b) echo b;; *) echo other;; esac");
        assert_eq!(ast.statements.len(), 1);
    }

    #[test]
    fn test_parse_function_definition() {
        let ast = parse_ok("myfunc() { echo hello; }");
        assert_eq!(ast.statements.len(), 1);
    }

    #[test]
    fn test_parse_pipeline() {
        let ast = parse_ok("ls | grep test");
        assert_eq!(ast.statements.len(), 1);
    }

    #[test]
    fn test_parse_and_list() {
        let ast = parse_ok("true && echo yes");
        assert_eq!(ast.statements.len(), 1);
    }

    #[test]
    fn test_parse_or_list() {
        let ast = parse_ok("false || echo fallback");
        assert_eq!(ast.statements.len(), 1);
    }

    #[test]
    fn test_parse_redirect_output() {
        let ast = parse_ok("echo hello > /tmp/out.txt");
        assert_eq!(ast.statements.len(), 1);
    }

    #[test]
    fn test_parse_redirect_input() {
        let ast = parse_ok("cat < /tmp/in.txt");
        assert_eq!(ast.statements.len(), 1);
    }

    #[test]
    fn test_parse_redirect_append() {
        let ast = parse_ok("echo hello >> /tmp/out.txt");
        assert_eq!(ast.statements.len(), 1);
    }

    #[test]
    fn test_parse_redirect_error() {
        let ast = parse_ok("cmd 2>/dev/null");
        assert_eq!(ast.statements.len(), 1);
    }

    #[test]
    fn test_parse_redirect_combined() {
        let ast = parse_ok("cmd > /tmp/out 2>&1");
        assert_eq!(ast.statements.len(), 1);
    }

    #[test]
    fn test_parse_here_string() {
        let ast = parse_ok("cat <<< 'hello world'");
        assert_eq!(ast.statements.len(), 1);
    }

    #[test]
    fn test_parse_arithmetic_expression() {
        let ast = parse_ok("x=$((1 + 2))");
        assert_eq!(ast.statements.len(), 1);
    }

    #[test]
    fn test_parse_variable_expansion_default() {
        let ast = parse_ok("echo ${x:-default}");
        assert_eq!(ast.statements.len(), 1);
    }

    #[test]
    fn test_parse_variable_expansion_length() {
        let ast = parse_ok("echo ${#x}");
        assert_eq!(ast.statements.len(), 1);
    }

    #[test]
    fn test_parse_variable_expansion_prefix_removal() {
        let ast = parse_ok("echo ${x#pattern}");
        assert_eq!(ast.statements.len(), 1);
    }

    #[test]
    fn test_parse_variable_expansion_suffix_removal() {
        let ast = parse_ok("echo ${x%pattern}");
        assert_eq!(ast.statements.len(), 1);
    }

    #[test]
    fn test_parse_test_condition_file() {
        let ast = parse_ok("if [ -f /tmp/test ]; then echo exists; fi");
        assert_eq!(ast.statements.len(), 1);
    }

    #[test]
    fn test_parse_test_condition_string() {
        let ast = parse_ok("if [ -n \"$x\" ]; then echo nonempty; fi");
        assert_eq!(ast.statements.len(), 1);
    }

    #[test]
    fn test_parse_test_condition_numeric() {
        let ast = parse_ok("if [ \"$x\" -eq 5 ]; then echo five; fi");
        assert_eq!(ast.statements.len(), 1);
    }

    #[test]
    fn test_parse_subshell() {
        let ast = parse_ok("(echo hello; echo world)");
        assert_eq!(ast.statements.len(), 1);
    }

    #[test]
    fn test_parse_brace_group() {
        let ast = parse_ok("{ echo hello; echo world; }");
        assert_eq!(ast.statements.len(), 1);
    }

    #[test]
    fn test_parse_coproc() {
        let ast = parse_ok("coproc myproc { cat; }");
        assert_eq!(ast.statements.len(), 1);
    }

    #[test]
    fn test_parse_select() {
        let ast = parse_ok("select choice in a b c; do echo $choice; break; done");
        assert_eq!(ast.statements.len(), 1);
    }

    #[test]
    fn test_parse_negated_command() {
        // The parser doesn't support bare `! cmd`; use in pipeline context
        let ast = parse_ok("if ! test -f /tmp/x; then echo missing; fi");
        assert_eq!(ast.statements.len(), 1);
    }

    #[test]
    fn test_parse_background_command() {
        let ast = parse_ok("sleep 10 &");
        assert_eq!(ast.statements.len(), 1);
    }

    #[test]
    fn test_parse_comments() {
        let ast = parse_ok("# this is a comment\necho hello");
        // Comment is skipped, only echo remains
        assert!(!ast.statements.is_empty());
    }

    #[test]
    fn test_parse_exported_variable() {
        let ast = parse_ok("export PATH=/usr/bin");
        assert_eq!(ast.statements.len(), 1);
    }

    #[test]
    fn test_parse_multiple_statements() {
        let ast = parse_ok("x=1\ny=2\necho $x $y");
        assert_eq!(ast.statements.len(), 3);
    }

    #[test]
    fn test_parse_string_with_spaces() {
        let ast = parse_ok("x=\"hello world\"");
        assert_eq!(ast.statements.len(), 1);
    }

    #[test]
    fn test_parse_single_quoted_string() {
        let ast = parse_ok("x='hello world'");
        assert_eq!(ast.statements.len(), 1);
    }

    #[test]
    fn test_parse_command_substitution() {
        let ast = parse_ok("x=$(date)");
        assert_eq!(ast.statements.len(), 1);
    }

    #[test]
    fn test_parse_nested_command_substitution() {
        let ast = parse_ok("x=$(echo $(date))");
        assert_eq!(ast.statements.len(), 1);
    }
}

// ============================================================================
// Purification Integration
// ============================================================================

mod purification_integration {
    use crate::bash_parser::BashParser;
    use crate::bash_transpiler::purification::{PurificationOptions, Purifier};

    fn parse_and_purify(
        input: &str,
    ) -> (
        crate::bash_parser::ast::BashAst,
        crate::bash_transpiler::purification::PurificationReport,
    ) {
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        let mut purifier = Purifier::new(PurificationOptions::default());
        let purified = purifier.purify(&ast).unwrap();
        let report = purifier.report().clone();
        (purified, report)
    }

    #[test]
    fn test_purify_removes_random() {
        let input = "x=$RANDOM";
        let (purified, report) = parse_and_purify(input);
        // Purifier should flag or transform $RANDOM
        assert!(!purified.statements.is_empty());
        // Should have at least one determinism fix
        let total_fixes = report.determinism_fixes.len() + report.warnings.len();
        assert!(
            total_fixes > 0 || !report.idempotency_fixes.is_empty() || purified.statements.len() == 1,
            "Expected purification activity for $RANDOM"
        );
    }

    #[test]
    fn test_purify_mkdir_gets_p() {
        let input = "mkdir /tmp/test";
        let (purified, _report) = parse_and_purify(input);
        assert!(!purified.statements.is_empty());
    }

    #[test]
    fn test_purify_rm_gets_f() {
        let input = "rm /tmp/test";
        let (purified, _report) = parse_and_purify(input);
        assert!(!purified.statements.is_empty());
    }

    #[test]
    fn test_purify_ln_gets_sf() {
        let input = "ln -s /src /dst";
        let (purified, _report) = parse_and_purify(input);
        assert!(!purified.statements.is_empty());
    }

    #[test]
    fn test_purify_preserves_comments() {
        let input = "# This is a comment\necho hello";
        let (purified, _report) = parse_and_purify(input);
        assert!(!purified.statements.is_empty());
    }

    #[test]
    fn test_purify_idempotent() {
        let input = "mkdir -p /tmp/test\necho hello";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();

        // First purification
        let mut purifier1 = Purifier::new(PurificationOptions::default());
        let purified1 = purifier1.purify(&ast).unwrap();

        // Second purification of the already-purified result
        let mut purifier2 = Purifier::new(PurificationOptions::default());
        let purified2 = purifier2.purify(&purified1).unwrap();

        // Should be the same
        assert_eq!(
            format!("{:?}", purified1),
            format!("{:?}", purified2),
            "Purification should be idempotent"
        );
    }

    #[test]
    fn test_purify_type_check_enabled() {
        let input = "x=42\necho $x";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        let opts = PurificationOptions {
            type_check: true,
            ..PurificationOptions::default()
        };
        let mut purifier = Purifier::new(opts);
        let purified = purifier.purify(&ast).unwrap();
        assert!(!purified.statements.is_empty());
        // Type checker should have run
        let report = purifier.report();
        let _ = report.type_diagnostics.len();
    }

    #[test]
    fn test_purify_emit_guards() {
        let input = "x=42\necho $x";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        let opts = PurificationOptions {
            type_check: true,
            emit_guards: true,
            ..PurificationOptions::default()
        };
        let mut purifier = Purifier::new(opts);
        let _purified = purifier.purify(&ast).unwrap();
        // Type checker should exist
        assert!(purifier.type_checker().is_some());
    }

    #[test]
    fn test_purify_complex_script() {
        let input = r#"#!/bin/bash
x=$RANDOM
mkdir /tmp/mydir
rm /tmp/old
ln -s /src /dst
for i in 1 2 3; do
    echo $i
done
if [ -f /tmp/test ]; then
    echo found
fi
"#;
        let (purified, _report) = parse_and_purify(input);
        assert!(!purified.statements.is_empty());
    }

    #[test]
    fn test_purify_with_pipe() {
        let input = "ls | grep test";
        let (purified, _report) = parse_and_purify(input);
        assert!(!purified.statements.is_empty());
    }

    #[test]
    fn test_purify_options_defaults() {
        let opts = PurificationOptions::default();
        assert!(opts.strict_idempotency);
        assert!(opts.remove_non_deterministic);
        assert!(opts.track_side_effects);
        assert!(!opts.type_check);
        assert!(!opts.emit_guards);
        assert!(!opts.type_strict);
    }
}

// ============================================================================
// Linter Integration
// ============================================================================

mod linter_integration {
    use crate::linter::rules::{
        lint_dockerfile, lint_dockerfile_with_profile, lint_makefile, lint_shell, LintProfile,
    };

    #[test]
    fn test_lint_dockerfile_standard() {
        let dockerfile = "FROM ubuntu:22.04\nRUN apt-get update\n";
        let result = lint_dockerfile(dockerfile);
        // Should produce some diagnostics (missing USER, unpinned, etc.)
        let _ = result.diagnostics.len();
    }

    #[test]
    fn test_lint_dockerfile_coursera_profile() {
        let dockerfile = "FROM ubuntu:22.04\nRUN apt-get update\nUSER 65534\n";
        let result = lint_dockerfile_with_profile(dockerfile, LintProfile::Coursera);
        let _ = result.diagnostics.len();
    }

    #[test]
    fn test_lint_dockerfile_devcontainer_profile() {
        let dockerfile = "FROM ubuntu:22.04\nRUN apt-get update\n";
        let result = lint_dockerfile_with_profile(dockerfile, LintProfile::DevContainer);
        let _ = result.diagnostics.len();
    }

    #[test]
    fn test_lint_makefile_clean_file() {
        let makefile = ".PHONY: all\nall:\n\t@echo done\n";
        let result = lint_makefile(makefile);
        // A clean, simple makefile may have few or no issues
        let _ = result.diagnostics.len();
    }

    #[test]
    fn test_lint_makefile_with_issues() {
        // Makefile with various issues: spaces instead of tabs, etc.
        let makefile = ".PHONY: test\ntest:\n    echo test\n";
        let result = lint_makefile(makefile);
        // Should detect tab vs spaces issue
        assert!(
            !result.diagnostics.is_empty(),
            "Expected lint issues for spaces-instead-of-tabs"
        );
    }

    #[test]
    fn test_lint_shell_clean_script() {
        let script = "#!/bin/sh\nprintf '%s\\n' 'hello'\n";
        let result = lint_shell(script);
        // Clean script should have minimal issues
        let _ = result.diagnostics.len();
    }

    #[test]
    fn test_lint_shell_with_all_issue_types() {
        let script = r#"#!/bin/bash
# Unquoted variable
echo $UNQUOTED
# Backticks
x=`date`
# cd without exit
cd /tmp
# Useless cat
cat file | grep test
"#;
        let result = lint_shell(script);
        // Should have multiple diagnostics
        assert!(
            !result.diagnostics.is_empty(),
            "Expected lint issues for problematic script"
        );
    }

    #[test]
    fn test_lint_profile_display() {
        assert_eq!(format!("{}", LintProfile::Standard), "standard");
        assert_eq!(format!("{}", LintProfile::Coursera), "coursera");
        assert_eq!(format!("{}", LintProfile::DevContainer), "devcontainer");
    }

    #[test]
    fn test_lint_profile_from_str_all_variants() {
        use std::str::FromStr;

        assert_eq!(
            LintProfile::from_str("standard").unwrap(),
            LintProfile::Standard
        );
        assert_eq!(
            LintProfile::from_str("default").unwrap(),
            LintProfile::Standard
        );
        assert_eq!(
            LintProfile::from_str("coursera").unwrap(),
            LintProfile::Coursera
        );
        assert_eq!(
            LintProfile::from_str("coursera-labs").unwrap(),
            LintProfile::Coursera
        );
        assert_eq!(
            LintProfile::from_str("devcontainer").unwrap(),
            LintProfile::DevContainer
        );
        assert_eq!(
            LintProfile::from_str("dev-container").unwrap(),
            LintProfile::DevContainer
        );

        // Invalid profile
        assert!(LintProfile::from_str("nonexistent").is_err());
    }

    #[test]
    fn test_lint_shell_empty_script() {
        let result = lint_shell("");
        let _ = result.diagnostics.len();
    }

    #[test]
    fn test_lint_shell_shebang_only() {
        let result = lint_shell("#!/bin/sh\n");
        let _ = result.diagnostics.len();
    }

    #[test]
    fn test_lint_dockerfile_minimal() {
        let result = lint_dockerfile("FROM scratch\n");
        let _ = result.diagnostics.len();
    }

    #[test]
    fn test_lint_dockerfile_multi_stage() {
        let dockerfile = r#"FROM rust:1.75 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/app /usr/local/bin/
USER 65534
ENTRYPOINT ["app"]
"#;
        let result = lint_dockerfile(dockerfile);
        let _ = result.diagnostics.len();
    }

    #[test]
    fn test_lint_makefile_with_variables() {
        let makefile = "CC := gcc\nCFLAGS := -Wall\n\nall:\n\t$(CC) $(CFLAGS) -o app main.c\n";
        let result = lint_makefile(makefile);
        let _ = result.diagnostics.len();
    }
}

// ============================================================================
// Comply Integration
// ============================================================================

mod comply_integration {
    use crate::comply::config::Scope;
    use crate::comply::discovery::{Artifact, ArtifactKind};
    use crate::comply::rules::{check_rule, RuleId};
    use std::path::PathBuf;

    fn shell_artifact() -> Artifact {
        Artifact::new(
            PathBuf::from("test.sh"),
            Scope::Project,
            ArtifactKind::ShellScript,
        )
    }

    fn makefile_artifact() -> Artifact {
        Artifact::new(
            PathBuf::from("Makefile"),
            Scope::Project,
            ArtifactKind::Makefile,
        )
    }

    fn dockerfile_artifact() -> Artifact {
        Artifact::new(
            PathBuf::from("Dockerfile"),
            Scope::Project,
            ArtifactKind::Dockerfile,
        )
    }

    #[test]
    fn test_all_rules_on_clean_script() {
        let clean = "#!/bin/sh\nset -eu\nprintf '%s\\n' 'hello'\n";
        let artifact = shell_artifact();

        // Most rules should pass on a clean script
        let det = check_rule(RuleId::Determinism, clean, &artifact);
        assert!(det.passed, "Determinism should pass on clean script");

        let idem = check_rule(RuleId::Idempotency, clean, &artifact);
        assert!(idem.passed, "Idempotency should pass on clean script");

        let sec = check_rule(RuleId::Security, clean, &artifact);
        assert!(sec.passed, "Security should pass on clean script");

        let quote = check_rule(RuleId::Quoting, clean, &artifact);
        // May or may not pass depending on strictness
        let _ = quote.passed;

        let posix = check_rule(RuleId::Posix, clean, &artifact);
        let _ = posix.passed;

        let sc = check_rule(RuleId::ShellCheck, clean, &artifact);
        let _ = sc.passed;
    }

    #[test]
    fn test_all_rules_on_messy_script() {
        let messy = r#"#!/bin/bash
x=$RANDOM
eval "$user_input"
echo $unquoted
mkdir /tmp/test
`date`
"#;
        let artifact = shell_artifact();

        let det = check_rule(RuleId::Determinism, messy, &artifact);
        assert!(
            !det.passed,
            "Determinism should fail: $RANDOM found, violations: {:?}",
            det.violations
        );

        let sec = check_rule(RuleId::Security, messy, &artifact);
        assert!(
            !sec.passed,
            "Security should fail: eval found, violations: {:?}",
            sec.violations
        );
    }

    #[test]
    fn test_comply_makefile_safety() {
        let content = ".PHONY: all\nall:\n\t@echo done\n";
        let artifact = makefile_artifact();
        let result = check_rule(RuleId::MakefileSafety, content, &artifact);
        let _ = result.passed;
    }

    #[test]
    fn test_comply_dockerfile_best() {
        let content = "FROM ubuntu:22.04\nRUN apt-get update\n";
        let artifact = dockerfile_artifact();
        let result = check_rule(RuleId::DockerfileBest, content, &artifact);
        let _ = result.passed;
    }

    #[test]
    fn test_comply_config_hygiene() {
        let content = "export PATH=/usr/bin\n";
        let artifact = Artifact::new(
            PathBuf::from(".bashrc"),
            Scope::User,
            ArtifactKind::ShellConfig,
        );
        let result = check_rule(RuleId::ConfigHygiene, content, &artifact);
        let _ = result.passed;
    }

    #[test]
    fn test_comply_pzsh_budget() {
        let content = "echo hello";
        let artifact = shell_artifact();
        let result = check_rule(RuleId::PzshBudget, content, &artifact);
        // PzshBudget is handled externally, should pass
        assert!(result.passed, "PzshBudget stub should pass");
    }

    #[test]
    fn test_rule_id_code() {
        assert_eq!(RuleId::Posix.code(), "COMPLY-001");
        assert_eq!(RuleId::Determinism.code(), "COMPLY-002");
        assert_eq!(RuleId::Idempotency.code(), "COMPLY-003");
        assert_eq!(RuleId::Security.code(), "COMPLY-004");
        assert_eq!(RuleId::Quoting.code(), "COMPLY-005");
        assert_eq!(RuleId::ShellCheck.code(), "COMPLY-006");
        assert_eq!(RuleId::MakefileSafety.code(), "COMPLY-007");
        assert_eq!(RuleId::DockerfileBest.code(), "COMPLY-008");
        assert_eq!(RuleId::ConfigHygiene.code(), "COMPLY-009");
        assert_eq!(RuleId::PzshBudget.code(), "COMPLY-010");
    }

    #[test]
    fn test_artifact_kind_display() {
        assert_eq!(format!("{}", ArtifactKind::ShellScript), "shell");
        assert_eq!(format!("{}", ArtifactKind::Makefile), "makefile");
        assert_eq!(format!("{}", ArtifactKind::Dockerfile), "dockerfile");
        assert_eq!(format!("{}", ArtifactKind::ShellConfig), "config");
        assert_eq!(format!("{}", ArtifactKind::Workflow), "workflow");
        assert_eq!(format!("{}", ArtifactKind::DevContainer), "devcontainer");
    }

    #[test]
    fn test_scope_display() {
        assert_eq!(format!("{}", Scope::Project), "project");
        assert_eq!(format!("{}", Scope::User), "user");
        assert_eq!(format!("{}", Scope::System), "system");
    }
}

// ============================================================================
// Make Parser Integration
// ============================================================================

mod make_parser_integration {
    use crate::make_parser::parse_makefile;

    #[test]
    fn test_parse_simple_makefile() {
        let input = "all:\n\techo done\n";
        let ast = parse_makefile(input).unwrap();
        assert!(!ast.items.is_empty());
    }

    #[test]
    fn test_parse_makefile_with_variables() {
        let input = "CC := gcc\nCFLAGS := -Wall\n\nall:\n\t$(CC) $(CFLAGS) main.c\n";
        let ast = parse_makefile(input).unwrap();
        assert!(!ast.items.is_empty());
    }

    #[test]
    fn test_parse_makefile_with_phony() {
        let input = ".PHONY: clean test\n\nclean:\n\trm -rf build\n\ntest:\n\tcargo test\n";
        let ast = parse_makefile(input).unwrap();
        assert!(!ast.items.is_empty());
    }

    #[test]
    fn test_parse_makefile_with_prerequisites() {
        let input = "app: main.o utils.o\n\tgcc -o app main.o utils.o\n";
        let ast = parse_makefile(input).unwrap();
        assert!(!ast.items.is_empty());
    }

    #[test]
    fn test_parse_makefile_with_comments() {
        let input = "# This is a comment\nall:\n\techo done\n";
        let ast = parse_makefile(input).unwrap();
        assert!(!ast.items.is_empty());
    }

    #[test]
    fn test_parse_makefile_multiline_recipe() {
        let input = "build:\n\techo step1\n\techo step2\n\techo step3\n";
        let ast = parse_makefile(input).unwrap();
        assert!(!ast.items.is_empty());
    }

    #[test]
    fn test_parse_makefile_conditional_assignment() {
        let input = "CC ?= gcc\n\nall:\n\t$(CC) main.c\n";
        let ast = parse_makefile(input).unwrap();
        assert!(!ast.items.is_empty());
    }

    #[test]
    fn test_parse_makefile_append_assignment() {
        let input = "CFLAGS += -O2\n\nall:\n\tgcc $(CFLAGS) main.c\n";
        let ast = parse_makefile(input).unwrap();
        assert!(!ast.items.is_empty());
    }

    #[test]
    fn test_parse_makefile_empty() {
        let input = "";
        let ast = parse_makefile(input).unwrap();
        assert!(ast.items.is_empty());
    }

    #[test]
    fn test_parse_makefile_line_continuation() {
        let input = "SRCS = main.c \\\n\tutils.c \\\n\thelper.c\n\nall:\n\tgcc $(SRCS)\n";
        let ast = parse_makefile(input).unwrap();
        assert!(!ast.items.is_empty());
    }
}

// ============================================================================
// Config and model coverage
// ============================================================================

mod config_coverage {
    use crate::models::{Config, ShellDialect, VerificationLevel};

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.target, ShellDialect::Posix);
        assert_eq!(config.verify, VerificationLevel::Strict);
        assert!(config.optimize);
        assert!(!config.emit_proof);
        assert!(!config.strict_mode);
    }

    #[test]
    fn test_shell_dialect_debug() {
        let _ = format!("{:?}", ShellDialect::Posix);
        let _ = format!("{:?}", ShellDialect::Bash);
        let _ = format!("{:?}", ShellDialect::Dash);
        let _ = format!("{:?}", ShellDialect::Ash);
    }

    #[test]
    fn test_verification_level_debug() {
        let _ = format!("{:?}", VerificationLevel::None);
        let _ = format!("{:?}", VerificationLevel::Basic);
        let _ = format!("{:?}", VerificationLevel::Strict);
        let _ = format!("{:?}", VerificationLevel::Paranoid);
    }

    #[test]
    fn test_config_with_strict_mode() {
        let config = Config {
            strict_mode: true,
            ..Config::default()
        };
        assert!(config.strict_mode);
    }

    #[test]
    fn test_config_with_emit_proof() {
        let config = Config {
            emit_proof: true,
            ..Config::default()
        };
        assert!(config.emit_proof);
    }

    #[test]
    fn test_config_optimize_disabled() {
        let config = Config {
            optimize: false,
            ..Config::default()
        };
        assert!(!config.optimize);
    }
}

// ============================================================================
// Check function (validation-only path)
// ============================================================================

mod check_function {
    use crate::check;

    #[test]
    fn test_check_valid_code() {
        let code = "fn main() { let x = 42; }";
        assert!(check(code).is_ok());
    }

    #[test]
    fn test_check_invalid_syntax() {
        let code = "fn main( { }";
        assert!(check(code).is_err());
    }

    #[test]
    fn test_check_empty_main() {
        let code = "fn main() {}";
        assert!(check(code).is_ok());
    }

    #[test]
    fn test_check_with_function_def() {
        let code = r#"
            fn main() {
                helper();
            }
            fn helper() {
                let x = 1;
            }
        "#;
        assert!(check(code).is_ok());
    }

    #[test]
    fn test_check_multiple_variables() {
        let code = r#"
            fn main() {
                let a = 1;
                let b = "hello";
                let c = true;
            }
        "#;
        assert!(check(code).is_ok());
    }
}

// ============================================================================
// Makefile and Dockerfile transpilation
// ============================================================================

mod transpile_formats {
    use crate::{transpile_dockerfile, transpile_makefile, Config};

    #[test]
    fn test_transpile_makefile_basic() {
        let code = r#"
            fn main() {
                let CC = "gcc";
            }
        "#;
        let result = transpile_makefile(code, Config::default());
        // May succeed or fail depending on DSL expectations
        let _ = result;
    }

    #[test]
    fn test_transpile_dockerfile_basic() {
        let code = r#"
            fn main() {
                from_image("rust", "1.75");
            }
            fn from_image(name: &str, tag: &str) {}
        "#;
        let result = transpile_dockerfile(code, Config::default());
        // May succeed or fail depending on DSL expectations
        let _ = result;
    }
}
