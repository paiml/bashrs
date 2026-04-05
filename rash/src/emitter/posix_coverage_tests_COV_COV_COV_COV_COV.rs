
#[test]
fn test_COV_POSIX_078_separate_functions_non_sequence() {
    // Non-Sequence IR is treated as the whole main body
    let ir = ShellIR::Echo {
        value: ShellValue::String("standalone".to_string()),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("echo"));
    assert!(result.contains("standalone"));
}

// ---------------------------------------------------------------------------
// emit_function — known command with empty body is NOT emitted
// ---------------------------------------------------------------------------

#[test]
fn test_COV_POSIX_079_function_known_command_empty_body_not_emitted() {
    let ir = ShellIR::Function {
        name: "echo".to_string(),
        params: vec![],
        body: Box::new(ShellIR::Noop),
    };
    let result = emitter().emit(&ir).unwrap();
    // echo() function def should NOT appear since echo is a known command
    assert!(!result.contains("echo() {"));
}

#[test]
fn test_COV_POSIX_080_function_known_command_empty_sequence_not_emitted() {
    let ir = ShellIR::Function {
        name: "grep".to_string(),
        params: vec![],
        body: Box::new(ShellIR::Sequence(vec![])),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(!result.contains("grep() {"));
}

#[test]
fn test_COV_POSIX_081_function_user_defined_with_params() {
    let ir = ShellIR::Sequence(vec![ShellIR::Function {
        name: "my_func".to_string(),
        params: vec!["arg1".to_string(), "arg2".to_string()],
        body: Box::new(ShellIR::Noop),
    }]);
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("my_func()"));
    assert!(result.contains("arg1=\"$1\""));
    assert!(result.contains("arg2=\"$2\""));
}

// ---------------------------------------------------------------------------
// Return statement
// ---------------------------------------------------------------------------

#[test]
fn test_COV_POSIX_082_return_with_value() {
    let ir = ShellIR::Return {
        value: Some(ShellValue::Variable("result".to_string())),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("echo"));
    assert!(result.contains("result"));
    assert!(result.contains("return"));
}

#[test]
fn test_COV_POSIX_083_return_without_value() {
    let ir = ShellIR::Return { value: None };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("return"));
}

// ---------------------------------------------------------------------------
// ForIn emission
// ---------------------------------------------------------------------------

#[test]
fn test_COV_POSIX_084_for_in_multi_items() {
    let ir = ShellIR::ForIn {
        var: "item".to_string(),
        items: vec![
            ShellValue::String("a".to_string()),
            ShellValue::String("b".to_string()),
            ShellValue::String("c".to_string()),
        ],
        body: Box::new(ShellIR::Echo {
            value: ShellValue::Variable("item".to_string()),
        }),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("for item in"));
    assert!(result.contains("do"));
    assert!(result.contains("done"));
}

// ---------------------------------------------------------------------------
// Empty sequence uses noop
// ---------------------------------------------------------------------------

#[test]
fn test_COV_POSIX_085_empty_sequence_emits_noop() {
    let ir = ShellIR::Sequence(vec![]);
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains(':'));
}

// ---------------------------------------------------------------------------
// Case arm with guard
// ---------------------------------------------------------------------------

#[test]
fn test_COV_POSIX_086_case_arm_with_guard() {
    let ir = ShellIR::Case {
        scrutinee: ShellValue::Variable("x".to_string()),
        arms: vec![CaseArm {
            pattern: CasePattern::Literal("1".to_string()),
            guard: Some(ShellValue::Comparison {
                op: ComparisonOp::Gt,
                left: Box::new(ShellValue::Variable("y".to_string())),
                right: Box::new(ShellValue::String("0".to_string())),
            }),
            body: Box::new(ShellIR::Echo {
                value: ShellValue::String("guarded".to_string()),
            }),
        }],
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("case"));
    assert!(result.contains("if"));
    assert!(result.contains("-gt"));
    assert!(result.contains("guarded"));
    assert!(result.contains("fi"));
}

// ---------------------------------------------------------------------------
// emit_with_trace
// ---------------------------------------------------------------------------

#[test]
fn test_COV_POSIX_087_emit_with_trace_returns_trace() {
    use crate::emitter::emit_with_trace;
    let ir = ShellIR::Echo {
        value: ShellValue::String("traced".to_string()),
    };
    let (output, trace) = emit_with_trace(&ir).unwrap();
    assert!(output.contains("echo"));
    assert!(!trace.is_empty());
}

// ---------------------------------------------------------------------------
// String with single-quote in assignment
// ---------------------------------------------------------------------------

#[test]
fn test_COV_POSIX_088_assignment_string_with_single_quote() {
    // String containing a single quote — triggers escape_shell_string path
    let ir = ShellIR::Let {
        name: "msg".to_string(),
        value: ShellValue::String("it's fine".to_string()),
        effects: EffectSet::pure(),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("msg="));
    // The output should contain the text somehow (escaped)
    assert!(result.contains("it") && result.contains("fine"));
}

// ---------------------------------------------------------------------------
// Exec with runtime and builtin detection
// ---------------------------------------------------------------------------

#[test]
fn test_COV_POSIX_089_exec_runtime_rash_prefix() {
    let ir = ShellIR::Exec {
        cmd: Command {
            program: "rash_println".to_string(),
            args: vec![ShellValue::String("hello".to_string())],
        },
        effects: EffectSet::pure(),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("rash_println"));
}

#[test]
fn test_COV_POSIX_090_exec_external_command() {
    let ir = ShellIR::Exec {
        cmd: Command {
            program: "git".to_string(),
            args: vec![
                ShellValue::String("status".to_string()),
                ShellValue::String("--short".to_string()),
            ],
        },
        effects: EffectSet::pure(),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("git"));
    assert!(result.contains("--short"));
}

// ---------------------------------------------------------------------------
// StrEq and StrNe comparisons
// ---------------------------------------------------------------------------

#[test]
fn test_COV_POSIX_091_comparison_str_eq() {
    let ir = ShellIR::If {
        test: ShellValue::Comparison {
            op: ComparisonOp::StrEq,
            left: Box::new(ShellValue::Variable("name".to_string())),
            right: Box::new(ShellValue::String("alice".to_string())),
        },
        then_branch: Box::new(ShellIR::Noop),
        else_branch: None,
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains(" = "));
}

#[test]
fn test_COV_POSIX_092_comparison_str_ne() {
    let ir = ShellIR::If {
        test: ShellValue::Comparison {
            op: ComparisonOp::StrNe,
            left: Box::new(ShellValue::Variable("name".to_string())),
            right: Box::new(ShellValue::String("bob".to_string())),
        },
        then_branch: Box::new(ShellIR::Noop),
        else_branch: None,
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("!="));
}

// ---------------------------------------------------------------------------
// elif chain
// ---------------------------------------------------------------------------

#[test]
fn test_COV_POSIX_093_elif_chain() {
    let ir = ShellIR::If {
        test: ShellValue::Comparison {
            op: ComparisonOp::NumEq,
            left: Box::new(ShellValue::Variable("x".to_string())),
            right: Box::new(ShellValue::String("1".to_string())),
        },
        then_branch: Box::new(ShellIR::Echo {
            value: ShellValue::String("one".to_string()),
        }),
        else_branch: Some(Box::new(ShellIR::If {
            test: ShellValue::Comparison {
                op: ComparisonOp::NumEq,
                left: Box::new(ShellValue::Variable("x".to_string())),
                right: Box::new(ShellValue::String("2".to_string())),
            },
            then_branch: Box::new(ShellIR::Echo {
                value: ShellValue::String("two".to_string()),
            }),
            else_branch: Some(Box::new(ShellIR::Echo {
                value: ShellValue::String("other".to_string()),
            })),
        })),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("elif"));
    assert!(result.contains("else"));
    assert!(result.contains("one"));
    assert!(result.contains("two"));
    assert!(result.contains("other"));
}

// ---------------------------------------------------------------------------
// Assignment value — empty string
// ---------------------------------------------------------------------------

#[test]
fn test_COV_POSIX_094_assignment_empty_string() {
    let ir = ShellIR::Let {
        name: "empty".to_string(),
        value: ShellValue::String(String::new()),
        effects: EffectSet::pure(),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("empty=''"));
}

// ---------------------------------------------------------------------------
// emit_while_condition — Bool false
// ---------------------------------------------------------------------------

#[test]
fn test_COV_POSIX_095_while_condition_bool_false() {
    let ir = ShellIR::While {
        condition: ShellValue::Bool(false),
        body: Box::new(ShellIR::Break),
    };
    let result = emitter().emit(&ir).unwrap();
    // Bool(false) falls to the general case: [ false ]
    assert!(result.contains("while") && result.contains("false"));
}
