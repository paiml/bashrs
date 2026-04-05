fn test_POSIX_COV_017_emit_while_condition_recursive() {
    use crate::ir::shell_ir::ComparisonOp;
    let config = Config::default();
    let emitter = PosixEmitter::new();
    // Nested: (a < 10) && (! (b > 5))
    let ir = ShellIR::While {
        condition: ShellValue::LogicalAnd {
            left: Box::new(ShellValue::Comparison {
                op: ComparisonOp::Lt,
                left: Box::new(ShellValue::Variable("a".to_string())),
                right: Box::new(ShellValue::String("10".to_string())),
            }),
            right: Box::new(ShellValue::LogicalNot {
                operand: Box::new(ShellValue::Comparison {
                    op: ComparisonOp::Gt,
                    left: Box::new(ShellValue::Variable("b".to_string())),
                    right: Box::new(ShellValue::String("5".to_string())),
                }),
            }),
        },
        body: Box::new(ShellIR::Noop),
    };
    let result = emitter.emit(&ir).expect("emit should succeed");
    assert!(result.contains("&&"));
    assert!(result.contains("!"));
}

#[test]
fn test_POSIX_COV_018_emit_while_condition_bool_false() {
    let config = Config::default();
    let emitter = PosixEmitter::new();
    // while (true && false) - tests Bool(false) in emit_while_condition
    let ir = ShellIR::While {
        condition: ShellValue::LogicalAnd {
            left: Box::new(ShellValue::Bool(true)),
            right: Box::new(ShellValue::Bool(false)),
        },
        body: Box::new(ShellIR::Noop),
    };
    let result = emitter.emit(&ir).expect("emit should succeed");
    assert!(result.contains("true && false"));
}

#[test]
fn test_POSIX_COV_019_emit_while_condition_or_recursive() {
    let config = Config::default();
    let emitter = PosixEmitter::new();
    // while (a || (b && c)) - tests LogicalOr path in emit_while_condition
    let ir = ShellIR::While {
        condition: ShellValue::LogicalOr {
            left: Box::new(ShellValue::Variable("a".to_string())),
            right: Box::new(ShellValue::LogicalAnd {
                left: Box::new(ShellValue::Variable("b".to_string())),
                right: Box::new(ShellValue::Variable("c".to_string())),
            }),
        },
        body: Box::new(ShellIR::Noop),
    };
    let result = emitter.emit(&ir).expect("emit should succeed");
    assert!(result.contains("||"));
    assert!(result.contains("&&"));
}

#[test]
fn test_POSIX_COV_020_emit_while_condition_general() {
    let config = Config::default();
    let emitter = PosixEmitter::new();
    // General expression in recursive condition via LogicalAnd
    let ir = ShellIR::While {
        condition: ShellValue::LogicalAnd {
            left: Box::new(ShellValue::String("test_val".to_string())),
            right: Box::new(ShellValue::Bool(true)),
        },
        body: Box::new(ShellIR::Noop),
    };
    let result = emitter.emit(&ir).expect("emit should succeed");
    assert!(result.contains("[ test_val ]"));
}

#[test]
fn test_POSIX_COV_021_case_statement() {
    use crate::ir::shell_ir::{CaseArm, CasePattern};
    let config = Config::default();
    let emitter = PosixEmitter::new();
    let ir = ShellIR::Case {
        scrutinee: ShellValue::Variable("opt".to_string()),
        arms: vec![
            CaseArm {
                pattern: CasePattern::Literal("start".to_string()),
                guard: None,
                body: Box::new(ShellIR::Exec {
                    cmd: Command {
                        program: "echo".to_string(),
                        args: vec![ShellValue::String("starting".to_string())],
                    },
                    effects: Default::default(),
                }),
            },
            CaseArm {
                pattern: CasePattern::Wildcard,
                guard: None,
                body: Box::new(ShellIR::Exec {
                    cmd: Command {
                        program: "echo".to_string(),
                        args: vec![ShellValue::String("unknown".to_string())],
                    },
                    effects: Default::default(),
                }),
            },
        ],
    };
    let result = emitter.emit(&ir).expect("emit should succeed");
    assert!(result.contains("case"));
    assert!(result.contains("start)"));
    assert!(result.contains("*)"));
    assert!(result.contains(";;"));
    assert!(result.contains("esac"));
}

#[test]
fn test_POSIX_COV_022_case_with_guard() {
    use crate::ir::shell_ir::{CaseArm, CasePattern};
    let config = Config::default();
    let emitter = PosixEmitter::new();
    let ir = ShellIR::Case {
        scrutinee: ShellValue::Variable("x".to_string()),
        arms: vec![CaseArm {
            pattern: CasePattern::Literal("yes".to_string()),
            guard: Some(ShellValue::Bool(true)),
            body: Box::new(ShellIR::Noop),
        }],
    };
    let result = emitter.emit(&ir).expect("emit should succeed");
    assert!(result.contains("if true; then"));
    assert!(result.contains("fi"));
}

#[test]
fn test_POSIX_COV_023_concat_with_bool() {
    let config = Config::default();
    let emitter = PosixEmitter::new();
    let ir = ShellIR::Let {
        name: "msg".to_string(),
        value: ShellValue::Concat(vec![
            ShellValue::String("status=".to_string()),
            ShellValue::Bool(true),
        ]),
        effects: Default::default(),
    };
    let result = emitter.emit(&ir).expect("emit should succeed");
    assert!(result.contains("status=true"));
}

#[test]
fn test_POSIX_COV_024_concat_with_env_var() {
    let config = Config::default();
    let emitter = PosixEmitter::new();
    let ir = ShellIR::Let {
        name: "path".to_string(),
        value: ShellValue::Concat(vec![
            ShellValue::EnvVar {
                name: "HOME".to_string(),
                default: None,
            },
            ShellValue::String("/bin".to_string()),
        ]),
        effects: Default::default(),
    };
    let result = emitter.emit(&ir).expect("emit should succeed");
    assert!(result.contains("${HOME}"));
}

#[test]
fn test_POSIX_COV_025_concat_with_env_var_default() {
    let config = Config::default();
    let emitter = PosixEmitter::new();
    let ir = ShellIR::Let {
        name: "path".to_string(),
        value: ShellValue::Concat(vec![ShellValue::EnvVar {
            name: "PREFIX".to_string(),
            default: Some("/usr/local".to_string()),
        }]),
        effects: Default::default(),
    };
    let result = emitter.emit(&ir).expect("emit should succeed");
    assert!(result.contains("${PREFIX:-/usr/local}"));
}

#[test]
fn test_POSIX_COV_026_concat_with_command_subst() {
    let config = Config::default();
    let emitter = PosixEmitter::new();
    let ir = ShellIR::Let {
        name: "msg".to_string(),
        value: ShellValue::Concat(vec![
            ShellValue::String("date: ".to_string()),
            ShellValue::CommandSubst(Command {
                program: "date".to_string(),
                args: vec![],
            }),
        ]),
        effects: Default::default(),
    };
    let result = emitter.emit(&ir).expect("emit should succeed");
    assert!(result.contains("$(date)"));
}

#[test]
fn test_POSIX_COV_027_concat_comparison_error() {
    use crate::ir::shell_ir::ComparisonOp;
    let config = Config::default();
    let emitter = PosixEmitter::new();
    let ir = ShellIR::Let {
        name: "bad".to_string(),
        value: ShellValue::Concat(vec![ShellValue::Comparison {
            op: ComparisonOp::NumEq,
            left: Box::new(ShellValue::String("1".to_string())),
            right: Box::new(ShellValue::String("2".to_string())),
        }]),
        effects: Default::default(),
    };
    let result = emitter.emit(&ir);
    assert!(result.is_err());
}

#[test]
fn test_POSIX_COV_028_concat_with_arithmetic() {
    use crate::ir::shell_ir::ArithmeticOp;
    let config = Config::default();
    let emitter = PosixEmitter::new();
    let ir = ShellIR::Let {
        name: "msg".to_string(),
        value: ShellValue::Concat(vec![
            ShellValue::String("count=".to_string()),
            ShellValue::Arithmetic {
                op: ArithmeticOp::Add,
                left: Box::new(ShellValue::Variable("n".to_string())),
                right: Box::new(ShellValue::String("1".to_string())),
            },
        ]),
        effects: Default::default(),
    };
    let result = emitter.emit(&ir).expect("emit should succeed");
    assert!(result.contains("$(("));
}

#[test]
fn test_POSIX_COV_029_concat_logical_error() {
    let config = Config::default();
    let emitter = PosixEmitter::new();
    let ir = ShellIR::Let {
        name: "bad".to_string(),
        value: ShellValue::Concat(vec![ShellValue::LogicalAnd {
            left: Box::new(ShellValue::Bool(true)),
            right: Box::new(ShellValue::Bool(false)),
        }]),
        effects: Default::default(),
    };
    let result = emitter.emit(&ir);
    assert!(result.is_err());
}

#[test]
fn test_POSIX_COV_030_concat_with_arg() {
    let config = Config::default();
    let emitter = PosixEmitter::new();
    let ir = ShellIR::Let {
        name: "msg".to_string(),
        value: ShellValue::Concat(vec![
            ShellValue::String("arg1=".to_string()),
            ShellValue::Arg { position: Some(1) },
            ShellValue::String(" all=".to_string()),
            ShellValue::Arg { position: None },
        ]),
        effects: Default::default(),
    };
    let result = emitter.emit(&ir).expect("emit should succeed");
    assert!(result.contains("$1"));
    assert!(result.contains("$@"));
}

#[test]
fn test_POSIX_COV_031_concat_with_arg_default_and_count() {
    let config = Config::default();
    let emitter = PosixEmitter::new();
    let ir = ShellIR::Let {
        name: "msg".to_string(),
        value: ShellValue::Concat(vec![
            ShellValue::ArgWithDefault {
                position: 2,
                default: "def".to_string(),
            },
            ShellValue::String(" n=".to_string()),
            ShellValue::ArgCount,
        ]),
        effects: Default::default(),
    };
    let result = emitter.emit(&ir).expect("emit should succeed");
    assert!(result.contains("${2:-def}"));
    assert!(result.contains("$#"));
}

#[test]
fn test_POSIX_COV_032_concat_nested_flatten() {
    let config = Config::default();
    let emitter = PosixEmitter::new();
    // Nested concatenation - triggers append_flattened_content
    let ir = ShellIR::Let {
        name: "msg".to_string(),
        value: ShellValue::Concat(vec![
            ShellValue::String("prefix_".to_string()),
            ShellValue::Concat(vec![
                ShellValue::String("inner_".to_string()),
                ShellValue::Variable("x".to_string()),
            ]),
        ]),
        effects: Default::default(),
    };
    let result = emitter.emit(&ir).expect("emit should succeed");
    assert!(result.contains("prefix_"));
    assert!(result.contains("inner_"));
}

#[test]
fn test_POSIX_COV_033a_test_expression_string() {
    let config = Config::default();
    let emitter = PosixEmitter::new();
    // String in test position is treated as truthy/falsy
    let ir = ShellIR::If {
        test: ShellValue::String("something".to_string()),
        then_branch: Box::new(ShellIR::Noop),
        else_branch: None,
    };
    let result = emitter.emit(&ir).expect("emit should succeed");
    // Non-"true"/Non-"0" string → "false"
    assert!(result.contains("if false; then"));
}

#[test]
fn test_POSIX_COV_033b_test_expression_other() {
    let config = Config::default();
    let emitter = PosixEmitter::new();
    // EnvVar in test position triggers "other =>" fallback branch
    let ir = ShellIR::If {
        test: ShellValue::EnvVar {
            name: "DEBUG".to_string(),
            default: None,
        },
        then_branch: Box::new(ShellIR::Noop),
        else_branch: None,
    };
    let result = emitter.emit(&ir).expect("emit should succeed");
    assert!(result.contains("test -n"));
}

#[test]
fn test_POSIX_COV_034_is_known_command_skip() {
    let config = Config::default();
    let emitter = PosixEmitter::new();
    // Function with known command name and empty body should be skipped
    let ir = ShellIR::Function {
        name: "echo".to_string(),
        params: vec![],
        body: Box::new(ShellIR::Noop),
    };
    let result = emitter.emit(&ir).expect("emit should succeed");
    // Should NOT contain echo() { since it's a known command with empty body
    assert!(!result.contains("echo() {"));
}

#[test]
fn test_POSIX_COV_035_while_comparison_condition() {
    use crate::ir::shell_ir::ComparisonOp;
    let config = Config::default();
    let emitter = PosixEmitter::new();
    let ir = ShellIR::While {
        condition: ShellValue::Comparison {
            op: ComparisonOp::Lt,
            left: Box::new(ShellValue::Variable("i".to_string())),
            right: Box::new(ShellValue::String("10".to_string())),
        },
        body: Box::new(ShellIR::Noop),
    };
    let result = emitter.emit(&ir).expect("emit should succeed");
    assert!(result.contains("-lt"));
    assert!(result.contains("while"));
}

#[test]
fn test_POSIX_COV_036_emit_shell_value_logical_not() {
    let config = Config::default();
    let emitter = PosixEmitter::new();
    let ir = ShellIR::Let {
        name: "x".to_string(),
        value: ShellValue::LogicalNot {
            operand: Box::new(ShellValue::Variable("flag".to_string())),
        },
        effects: Default::default(),
    };
    let result = emitter.emit(&ir).expect("emit should succeed");
    assert!(result.contains("!"));
}

#[test]
fn test_POSIX_COV_037_elif_chain_with_else() {
    use crate::ir::shell_ir::ComparisonOp;
    let config = Config::default();
    let emitter = PosixEmitter::new();
    // if/elif/else chain
    let ir = ShellIR::If {
        test: ShellValue::Comparison {
            op: ComparisonOp::NumEq,
            left: Box::new(ShellValue::Variable("x".to_string())),
            right: Box::new(ShellValue::String("1".to_string())),
        },
        then_branch: Box::new(ShellIR::Exec {
            cmd: Command {
                program: "echo".to_string(),
                args: vec![ShellValue::String("one".to_string())],
            },
            effects: Default::default(),
        }),
        else_branch: Some(Box::new(ShellIR::Sequence(vec![ShellIR::If {
            test: ShellValue::Comparison {
                op: ComparisonOp::NumEq,
                left: Box::new(ShellValue::Variable("x".to_string())),
                right: Box::new(ShellValue::String("2".to_string())),
            },
            then_branch: Box::new(ShellIR::Exec {
                cmd: Command {
                    program: "echo".to_string(),
                    args: vec![ShellValue::String("two".to_string())],
                },
                effects: Default::default(),
            }),
            else_branch: Some(Box::new(ShellIR::Exec {
                cmd: Command {
                    program: "echo".to_string(),
                    args: vec![ShellValue::String("other".to_string())],
                },
                effects: Default::default(),
            })),
        }]))),
    };
    let result = emitter.emit(&ir).expect("emit should succeed");
    assert!(result.contains("elif"));
    assert!(result.contains("else"));
}
