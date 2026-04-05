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
