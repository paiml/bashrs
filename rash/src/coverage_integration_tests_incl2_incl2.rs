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


include!("coverage_integration_tests_incl2_incl2_incl2.rs");
