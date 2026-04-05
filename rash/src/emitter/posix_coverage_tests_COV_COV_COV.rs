
#[test]
fn test_COV_POSIX_042_while_logical_and_condition() {
    let ir = ShellIR::While {
        condition: ShellValue::LogicalAnd {
            left: Box::new(ShellValue::Comparison {
                op: ComparisonOp::Lt,
                left: Box::new(ShellValue::Variable("i".to_string())),
                right: Box::new(ShellValue::String("10".to_string())),
            }),
            right: Box::new(ShellValue::Comparison {
                op: ComparisonOp::Gt,
                left: Box::new(ShellValue::Variable("j".to_string())),
                right: Box::new(ShellValue::String("0".to_string())),
            }),
        },
        body: Box::new(ShellIR::Break),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("while"));
    assert!(result.contains("&&"));
    assert!(result.contains("-lt") || result.contains("-gt"));
}

#[test]
fn test_COV_POSIX_043_while_logical_or_condition() {
    let ir = ShellIR::While {
        condition: ShellValue::LogicalOr {
            left: Box::new(ShellValue::Comparison {
                op: ComparisonOp::NumEq,
                left: Box::new(ShellValue::Variable("x".to_string())),
                right: Box::new(ShellValue::String("0".to_string())),
            }),
            right: Box::new(ShellValue::Comparison {
                op: ComparisonOp::NumEq,
                left: Box::new(ShellValue::Variable("y".to_string())),
                right: Box::new(ShellValue::String("0".to_string())),
            }),
        },
        body: Box::new(ShellIR::Break),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("||"));
}

#[test]
fn test_COV_POSIX_044_while_logical_not_condition() {
    let ir = ShellIR::While {
        condition: ShellValue::LogicalNot {
            operand: Box::new(ShellValue::Comparison {
                op: ComparisonOp::NumEq,
                left: Box::new(ShellValue::Variable("done_flag".to_string())),
                right: Box::new(ShellValue::String("1".to_string())),
            }),
        },
        body: Box::new(ShellIR::Break),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("!"));
    assert!(result.contains("-eq"));
}

#[test]
fn test_COV_POSIX_045_while_general_condition() {
    // Falls into the general "[ cond ]" path
    let ir = ShellIR::While {
        condition: ShellValue::Variable("running".to_string()),
        body: Box::new(ShellIR::Break),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("while"));
    assert!(result.contains("running"));
}

// ---------------------------------------------------------------------------
// emit_test_expression — string "0" → true, CommandSubst predicate
// ---------------------------------------------------------------------------

#[test]
fn test_COV_POSIX_046_test_string_true_value() {
    let ir = ShellIR::If {
        test: ShellValue::String("true".to_string()),
        then_branch: Box::new(ShellIR::Noop),
        else_branch: None,
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("if true"));
}

#[test]
fn test_COV_POSIX_047_test_string_zero_value() {
    let ir = ShellIR::If {
        test: ShellValue::String("0".to_string()),
        then_branch: Box::new(ShellIR::Noop),
        else_branch: None,
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("if true"));
}

#[test]
fn test_COV_POSIX_048_test_string_other_value() {
    let ir = ShellIR::If {
        test: ShellValue::String("foo".to_string()),
        then_branch: Box::new(ShellIR::Noop),
        else_branch: None,
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("if false"));
}

#[test]
fn test_COV_POSIX_049_test_command_subst_predicate_function() {
    let ir = ShellIR::If {
        test: ShellValue::CommandSubst(Command {
            program: "rash_string_contains".to_string(),
            args: vec![
                ShellValue::Variable("s".to_string()),
                ShellValue::String("x".to_string()),
            ],
        }),
        then_branch: Box::new(ShellIR::Noop),
        else_branch: None,
    };
    let result = emitter().emit(&ir).unwrap();
    // predicate functions are emitted directly (no "test -n")
    assert!(result.contains("rash_string_contains"));
}

#[test]
fn test_COV_POSIX_050_test_command_subst_value_function() {
    let ir = ShellIR::If {
        test: ShellValue::CommandSubst(Command {
            program: "compute_value".to_string(),
            args: vec![],
        }),
        then_branch: Box::new(ShellIR::Noop),
        else_branch: None,
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("test -n"));
}

#[test]
fn test_COV_POSIX_051_test_other_value_uses_test_n() {
    // An Arg value in test context falls to the "other" branch
    let ir = ShellIR::If {
        test: ShellValue::Arg { position: Some(1) },
        then_branch: Box::new(ShellIR::Noop),
        else_branch: None,
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("test -n"));
}

// ---------------------------------------------------------------------------
// Test expression — LogicalNot constant fold
// ---------------------------------------------------------------------------

#[test]
fn test_COV_POSIX_052_test_logical_not_constant_fold() {
    let ir = ShellIR::If {
        test: ShellValue::LogicalNot {
            operand: Box::new(ShellValue::Bool(true)),
        },
        then_branch: Box::new(ShellIR::Noop),
        else_branch: None,
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("if false"));
}

#[test]
fn test_COV_POSIX_053_test_logical_not_variable() {
    let ir = ShellIR::If {
        test: ShellValue::LogicalNot {
            operand: Box::new(ShellValue::Variable("flag".to_string())),
        },
        then_branch: Box::new(ShellIR::Noop),
        else_branch: None,
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("! \"$flag\""));
}

#[test]
fn test_COV_POSIX_054_test_logical_not_other_folded() {
    // Non-bool, non-variable falls through to emit_test_expression inner
    let ir = ShellIR::If {
        test: ShellValue::LogicalNot {
            operand: Box::new(ShellValue::Comparison {
                op: ComparisonOp::NumEq,
                left: Box::new(ShellValue::Variable("x".to_string())),
                right: Box::new(ShellValue::String("0".to_string())),
            }),
        },
        then_branch: Box::new(ShellIR::Noop),
        else_branch: None,
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("!"));
    assert!(result.contains("-eq"));
}

// ---------------------------------------------------------------------------
// Test logical and/or constant folding in test context
// ---------------------------------------------------------------------------

include!("posix_coverage_tests_COV_COV_COV_COV.rs");
