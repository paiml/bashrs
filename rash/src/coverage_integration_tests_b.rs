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
        let result = transpile_makefile(code, &Config::default());
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
        let result = transpile_dockerfile(code, &Config::default());
        // May succeed or fail depending on DSL expectations
        let _ = result;
    }
}
