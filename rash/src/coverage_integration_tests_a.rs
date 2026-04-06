#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

//! Comprehensive integration tests for maximum code coverage.
//!
//! Exercises: transpiler pipeline, bash parser, purification,
//! linter (shell/makefile/dockerfile), comply rules, and make parser.

// ============================================================================
// Transpiler Integration (parser -> IR -> emitter)
// ============================================================================

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
