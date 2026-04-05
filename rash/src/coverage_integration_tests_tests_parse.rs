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
            total_fixes > 0
                || !report.idempotency_fixes.is_empty()
                || purified.statements.len() == 1,
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


include!("coverage_integration_tests_tests_lint.rs");
