use super::rules::*;
use super::*;

#[test]
fn test_sc2086_quoted_variables_pass() {
    let cases = vec![
        VariableExpansion::Quoted("USER".to_string()),
        VariableExpansion::WordSplit("FILES".to_string()),
        VariableExpansion::ArrayExpansion("array[@]".to_string()),
    ];

    for case in cases {
        assert!(case.validate().is_ok(), "Failed on: {case:?}");
    }
}

#[test]
fn test_sc2086_unquoted_variables_fail() {
    let var = VariableExpansion::Unquoted("USER".to_string());
    let result = var.validate();

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.rule, "SC2086");
    assert_eq!(err.severity, Severity::Error);
    assert!(err.auto_fix.is_some());
}

#[test]
fn test_sc2046_command_substitution_quoted_pass() {
    let cases = vec![
        CommandSubstitution {
            command: "date".to_string(),
            context: SubstitutionContext::Assignment,
        },
        CommandSubstitution {
            command: "ls".to_string(),
            context: SubstitutionContext::Quoted,
        },
        CommandSubstitution {
            command: "find . -name '*.txt'".to_string(),
            context: SubstitutionContext::ArrayInit,
        },
    ];

    for case in cases {
        assert!(case.validate().is_ok(), "Failed on: {case:?}");
    }
}

#[test]
fn test_sc2046_command_substitution_unquoted_fail() {
    let cmd = CommandSubstitution {
        command: "ls -l".to_string(),
        context: SubstitutionContext::Unquoted,
    };
    let result = cmd.validate();

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.rule, "SC2046");
    assert_eq!(err.severity, Severity::Error);
    assert!(err.auto_fix.is_some());
}

#[test]
fn test_sc2035_glob_protection() {
    // Should pass
    assert!(validate_glob_pattern("file.txt").is_ok());
    assert!(validate_glob_pattern("*.txt").is_ok());
    assert!(validate_glob_pattern("./file").is_ok());

    // Should fail
    let result = validate_glob_pattern("-rf");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.rule, "SC2035");
    assert_eq!(err.severity, Severity::Warning);
    assert!(err.auto_fix.is_some());
    assert_eq!(err.auto_fix.unwrap().replacement, "./-rf");
}

#[test]
fn test_sc2181_exit_code_preservation() {
    // Good: checks immediately after command
    let good = CommandSequence {
        commands: vec!["ls".to_string(), "grep foo".to_string()],
        exit_code_checks: vec![
            ExitCodeCheck { command_index: 0 },
            ExitCodeCheck { command_index: 1 },
        ],
    };
    assert!(good.validate().is_ok());

    // Bad: delayed check
    let bad = CommandSequence {
        commands: vec!["ls".to_string(), "echo done".to_string()],
        exit_code_checks: vec![
            ExitCodeCheck { command_index: 0 },
            ExitCodeCheck { command_index: 0 }, // Checking old command
        ],
    };
    let result = bad.validate();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.rule, "SC2181");
    assert_eq!(err.severity, Severity::Style);
}

#[test]
fn test_conditional_expression_validation() {
    use crate::ir::ShellExpression;

    // Good: both sides quoted
    let good = ConditionalExpression::StringComparison {
        left: Box::new(ShellExpression::Variable("VAR".to_string(), true)),
        op: ComparisonOp::Eq,
        right: Box::new(ShellExpression::String("\"value\"".to_string())),
    };
    assert!(good.validate().is_ok());

    // Bad: unquoted variable
    let bad = ConditionalExpression::StringComparison {
        left: Box::new(ShellExpression::Variable("VAR".to_string(), false)),
        op: ComparisonOp::Eq,
        right: Box::new(ShellExpression::String("value".to_string())),
    };
    let result = bad.validate();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.rule, "SC2086");
}

#[test]
fn test_sc2006_backticks() {
    assert!(validate_backticks("echo $(date)").is_ok());

    let result = validate_backticks("echo `date`");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.rule, "SC2006");
    assert_eq!(err.severity, Severity::Style);
    assert!(err.auto_fix.is_some());
}

#[test]
fn test_sc2164_cd_usage() {
    assert!(validate_cd_usage("cd /tmp || exit 1").is_ok());
    assert!(validate_cd_usage("cd /tmp || return").is_ok());
    assert!(validate_cd_usage("echo 'not a cd command'").is_ok());

    let result = validate_cd_usage("cd /tmp");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.rule, "SC2164");
    assert_eq!(err.severity, Severity::Warning);
    assert!(err.auto_fix.is_some());
    assert_eq!(err.auto_fix.unwrap().replacement, "cd /tmp || exit 1");
}

#[test]
fn test_sc2162_read_command() {
    assert!(validate_read_command("read -r var").is_ok());
    assert!(validate_read_command("echo 'not a read'").is_ok());

    let result = validate_read_command("read var");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.rule, "SC2162");
    assert_eq!(err.severity, Severity::Warning);
    assert!(err.auto_fix.is_some());
    assert_eq!(err.auto_fix.unwrap().replacement, "read -r var");
}

#[test]
fn test_sc2220_unicode_quotes() {
    assert!(validate_unicode_quotes("echo \"hello\"").is_ok());
    assert!(validate_unicode_quotes("echo 'world'").is_ok());

    let result = validate_unicode_quotes("echo \u{201c}hello\u{201d}");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.rule, "SC2220");
    assert_eq!(err.severity, Severity::Error);
    assert!(err.auto_fix.is_some());

    let result = validate_unicode_quotes("echo \u{2018}world\u{2019}");
    assert!(result.is_err());
}

#[test]
fn test_validate_all() {
    // Should pass
    assert!(validate_all("echo \"hello world\"").is_ok());
    assert!(validate_all("cd /tmp || exit 1").is_ok());
    assert!(validate_all("read -r var").is_ok());

    // Should fail on backticks
    assert!(validate_all("echo `date`").is_err());

    // Should fail on unprotected cd
    assert!(validate_all("cd /tmp").is_err());

    // Should fail on unicode quotes
    assert!(validate_all("echo \u{201c}test\u{201d}").is_err());
}

#[test]
fn test_validation_pipeline_integration() {
    use crate::ir::{ShellIR, ShellValue};
    use crate::models::config::Config;

    let config = Config::default();
    let pipeline = pipeline::ValidationPipeline::new(&config);

    // Test AST validation - create a simple AST
    let ast = crate::ast::RestrictedAst {
        functions: vec![crate::ast::Function {
            name: "main".to_string(),
            params: vec![],
            body: vec![crate::ast::Stmt::Expr(crate::ast::Expr::FunctionCall {
                name: "echo".to_string(),
                args: vec![crate::ast::Expr::Literal(
                    crate::ast::restricted::Literal::Str("test".to_string()),
                )],
            })],
            return_type: crate::ast::Type::Void,
        }],
        entry_point: "main".to_string(),
    };
    assert!(pipeline.validate_ast(&ast).is_ok());

    // Test IR validation
    let ir = ShellIR::Exec {
        cmd: crate::ir::Command {
            program: "echo".to_string(),
            args: vec![ShellValue::String("test".to_string())],
        },
        effects: crate::ir::EffectSet::pure(),
    };
    assert!(pipeline.validate_ir(&ir).is_ok());

    // Test output validation (only in debug mode)
    #[cfg(debug_assertions)]
    {
        assert!(pipeline.validate_output("echo \"test\"").is_ok());
        assert!(pipeline.validate_output("echo `test`").is_err());
    }
}

#[test]
fn test_validation_error_reporting() {
    let error = ValidationError {
        rule: "SC2086",
        severity: Severity::Error,
        message: "Unquoted variable".to_string(),
        suggestion: Some("Add quotes".to_string()),
        auto_fix: Some(Fix {
            description: "Quote variable".to_string(),
            replacement: "\"$VAR\"".to_string(),
        }),
        line: Some(10),
        column: Some(5),
    };

    let display = format!("{error}");
    assert!(display.contains("SC2086"));
    assert!(display.contains("error"));
    assert!(display.contains("Unquoted variable"));
    assert!(display.contains("Add quotes"));
}

// ===== COVERAGE IMPROVEMENT TESTS =====

#[test]
fn test_severity_as_str() {
    assert_eq!(Severity::Error.as_str(), "error");
    assert_eq!(Severity::Warning.as_str(), "warning");
    assert_eq!(Severity::Style.as_str(), "style");
}

#[test]
fn test_validation_level_default() {
    let level: ValidationLevel = Default::default();
    assert_eq!(level, ValidationLevel::Minimal);
}

#[test]
fn test_validation_level_ordering() {
    assert!(ValidationLevel::None < ValidationLevel::Minimal);
    assert!(ValidationLevel::Minimal < ValidationLevel::Strict);
    assert!(ValidationLevel::Strict < ValidationLevel::Paranoid);
}

#[test]
fn test_validation_error_without_suggestion() {
    let error = ValidationError {
        rule: "SC2006",
        severity: Severity::Warning,
        message: "Use $(...) notation".to_string(),
        suggestion: None,
        auto_fix: None,
        line: None,
        column: None,
    };

    let display = format!("{error}");
    assert!(display.contains("SC2006"));
    assert!(display.contains("warning"));
    assert!(!display.contains("Suggestion:"));
}

#[test]
fn test_validate_shell_snippet_valid() {
    use super::validate_shell_snippet;

    // Valid shell snippets should pass
    let result = validate_shell_snippet("echo \"hello world\"");
    assert!(result.is_ok());
}

#[test]
fn test_validate_shell_snippet_invalid() {
    use super::validate_shell_snippet;

    // Backticks should fail
    let result = validate_shell_snippet("echo `date`");
    assert!(result.is_err());
}

#[test]
fn test_implemented_rules_count() {
    use super::IMPLEMENTED_RULES;

    // Ensure we have at least 20 rules implemented
    assert!(IMPLEMENTED_RULES.len() >= 20);
    assert!(IMPLEMENTED_RULES.contains(&"SC2086"));
    assert!(IMPLEMENTED_RULES.contains(&"SC2164"));
}
