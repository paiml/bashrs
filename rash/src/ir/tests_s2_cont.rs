/// RED TEST: env() call should convert to EnvVar variant in IR
/// Tests that env("HOME") is properly recognized and converted to ShellValue::EnvVar
#[test]
fn test_env_call_converts_to_ir() {
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Str,
            body: vec![Stmt::Let {
                name: "home".to_string(),
                value: Expr::FunctionCall {
                    name: "env".to_string(),
                    args: vec![Expr::Literal(Literal::Str("HOME".to_string()))],
                },
                declaration: true,
            }],
        }],
        entry_point: "main".to_string(),
    };

    let ir = from_ast(&ast).unwrap();

    match ir {
        ShellIR::Sequence(stmts) => {
            match &stmts[0] {
                ShellIR::Let { name, value, .. } => {
                    assert_eq!(name, "home");
                    // RED: This will fail until we implement EnvVar variant
                    match value {
                        ShellValue::EnvVar { name, default } => {
                            assert_eq!(name, "HOME");
                            assert_eq!(default, &None);
                        }
                        other => panic!("Expected EnvVar, got {:?}", other),
                    }
                }
                _ => panic!("Expected Let statement"),
            }
        }
        _ => panic!("Expected Sequence"),
    }
}

/// RED TEST: env_var_or() call should convert to EnvVar with default value
/// Tests that env_var_or("PREFIX", "/usr/local") converts to EnvVar with Some(default)
#[test]
fn test_env_var_or_call_converts_to_ir() {
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Str,
            body: vec![Stmt::Let {
                name: "prefix".to_string(),
                value: Expr::FunctionCall {
                    name: "env_var_or".to_string(),
                    args: vec![
                        Expr::Literal(Literal::Str("PREFIX".to_string())),
                        Expr::Literal(Literal::Str("/usr/local".to_string())),
                    ],
                },
                declaration: true,
            }],
        }],
        entry_point: "main".to_string(),
    };

    let ir = from_ast(&ast).unwrap();

    match ir {
        ShellIR::Sequence(stmts) => {
            match &stmts[0] {
                ShellIR::Let { name, value, .. } => {
                    assert_eq!(name, "prefix");
                    // RED: This will fail until we implement EnvVar variant with default
                    match value {
                        ShellValue::EnvVar { name, default } => {
                            assert_eq!(name, "PREFIX");
                            assert_eq!(default, &Some("/usr/local".to_string()));
                        }
                        other => panic!("Expected EnvVar with default, got {:?}", other),
                    }
                }
                _ => panic!("Expected Let statement"),
            }
        }
        _ => panic!("Expected Sequence"),
    }
}

/// RED TEST: env() in variable assignment context
/// Tests that env() works in typical variable assignment patterns
#[test]
fn test_env_in_assignment() {
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "setup".to_string(),
            params: vec![],
            return_type: Type::Str,
            body: vec![
                Stmt::Let {
                    name: "user".to_string(),
                    value: Expr::FunctionCall {
                        name: "env".to_string(),
                        args: vec![Expr::Literal(Literal::Str("USER".to_string()))],
                    },
                    declaration: true,
                },
                Stmt::Let {
                    name: "path".to_string(),
                    value: Expr::FunctionCall {
                        name: "env".to_string(),
                        args: vec![Expr::Literal(Literal::Str("PATH".to_string()))],
                    },
                    declaration: true,
                },
            ],
        }],
        entry_point: "setup".to_string(),
    };

    let ir = from_ast(&ast).unwrap();

    // RED: This will fail until EnvVar variant exists
    match ir {
        ShellIR::Sequence(stmts) => {
            assert_eq!(stmts.len(), 2);

            // Check first env() call
            match &stmts[0] {
                ShellIR::Let { value, .. } => {
                    assert!(
                        matches!(value, ShellValue::EnvVar { name, default }
                            if name == "USER" && default.is_none()),
                        "First env() should be EnvVar for USER"
                    );
                }
                _ => panic!("Expected Let statement"),
            }

            // Check second env() call
            match &stmts[1] {
                ShellIR::Let { value, .. } => {
                    assert!(
                        matches!(value, ShellValue::EnvVar { name, default }
                            if name == "PATH" && default.is_none()),
                        "Second env() should be EnvVar for PATH"
                    );
                }
                _ => panic!("Expected Let statement"),
            }
        }
        _ => panic!("Expected Sequence"),
    }
}

// ============= Sprint 27b: Command-Line Arguments Support - RED PHASE =============
