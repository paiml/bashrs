
#[test]
fn test_COV_POSIX_014_logical_and_constant_fold_true() {
    let ir = ShellIR::Echo {
        value: ShellValue::LogicalAnd {
            left: Box::new(ShellValue::Bool(true)),
            right: Box::new(ShellValue::Bool(true)),
        },
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("true"));
}

#[test]
fn test_COV_POSIX_015_logical_or_constant_fold_false() {
    let ir = ShellIR::Echo {
        value: ShellValue::LogicalOr {
            left: Box::new(ShellValue::Bool(false)),
            right: Box::new(ShellValue::Bool(false)),
        },
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("false"));
}

#[test]
fn test_COV_POSIX_016_logical_not_constant_fold_true() {
    let ir = ShellIR::Echo {
        value: ShellValue::LogicalNot {
            operand: Box::new(ShellValue::Bool(false)),
        },
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("true"));
}

// ---------------------------------------------------------------------------
// Concatenation — Bool, EnvVar, Arg, ArgWithDefault, ArgCount, ExitCode
// ---------------------------------------------------------------------------

#[test]
fn test_COV_POSIX_017_concat_with_bool() {
    let ir = ShellIR::Let {
        name: "msg".to_string(),
        value: ShellValue::Concat(vec![
            ShellValue::String("flag=".to_string()),
            ShellValue::Bool(true),
        ]),
        effects: EffectSet::pure(),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("flag="));
    assert!(result.contains("true"));
}

#[test]
fn test_COV_POSIX_018_concat_with_env_var_no_default() {
    let ir = ShellIR::Let {
        name: "msg".to_string(),
        value: ShellValue::Concat(vec![
            ShellValue::String("home=".to_string()),
            ShellValue::EnvVar {
                name: "HOME".to_string(),
                default: None,
            },
        ]),
        effects: EffectSet::pure(),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("${HOME}"));
}

#[test]
fn test_COV_POSIX_019_concat_with_env_var_with_default() {
    let ir = ShellIR::Let {
        name: "msg".to_string(),
        value: ShellValue::Concat(vec![ShellValue::EnvVar {
            name: "EDITOR".to_string(),
            default: Some("vi".to_string()),
        }]),
        effects: EffectSet::pure(),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("${EDITOR:-vi}"));
}

#[test]
fn test_COV_POSIX_020_concat_with_arg_position() {
    let ir = ShellIR::Let {
        name: "val".to_string(),
        value: ShellValue::Concat(vec![
            ShellValue::String("arg=".to_string()),
            ShellValue::Arg { position: Some(1) },
        ]),
        effects: EffectSet::pure(),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("$1"));
}

#[test]
fn test_COV_POSIX_021_concat_with_arg_all() {
    let ir = ShellIR::Let {
        name: "all".to_string(),
        value: ShellValue::Concat(vec![ShellValue::Arg { position: None }]),
        effects: EffectSet::pure(),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("$@"));
}

#[test]
fn test_COV_POSIX_022_concat_with_arg_with_default() {
    let ir = ShellIR::Let {
        name: "val".to_string(),
        value: ShellValue::Concat(vec![ShellValue::ArgWithDefault {
            position: 1,
            default: "default".to_string(),
        }]),
        effects: EffectSet::pure(),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("${1:-default}"));
}

#[test]
fn test_COV_POSIX_023_concat_with_arg_count() {
    let ir = ShellIR::Let {
        name: "cnt".to_string(),
        value: ShellValue::Concat(vec![ShellValue::ArgCount]),
        effects: EffectSet::pure(),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("$#"));
}

#[test]
fn test_COV_POSIX_024_concat_with_exit_code() {
    let ir = ShellIR::Let {
        name: "ret".to_string(),
        value: ShellValue::Concat(vec![ShellValue::ExitCode]),
        effects: EffectSet::pure(),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("$?"));
}

#[test]
fn test_COV_POSIX_025_concat_with_dynamic_array() {
    let ir = ShellIR::Let {
        name: "val".to_string(),
        value: ShellValue::Concat(vec![ShellValue::DynamicArrayAccess {
            array: "arr".to_string(),
            index: Box::new(ShellValue::Variable("i".to_string())),
        }]),
        effects: EffectSet::pure(),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("arr"));
}

#[test]
fn test_COV_POSIX_026_concat_with_nested_concat() {
    let ir = ShellIR::Let {
        name: "msg".to_string(),
        value: ShellValue::Concat(vec![
            ShellValue::String("outer".to_string()),
            ShellValue::Concat(vec![
                ShellValue::String("inner1".to_string()),
                ShellValue::String("inner2".to_string()),
            ]),
        ]),
        effects: EffectSet::pure(),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("outer"));
    assert!(result.contains("inner1"));
    assert!(result.contains("inner2"));
}

#[test]
fn test_COV_POSIX_027_concat_comparison_returns_error() {
    let e = emitter();
    let value = ShellValue::Concat(vec![ShellValue::Comparison {
        op: ComparisonOp::NumEq,
        left: Box::new(ShellValue::Variable("x".to_string())),
        right: Box::new(ShellValue::String("1".to_string())),
    }]);
    let ir = ShellIR::Let {
        name: "v".to_string(),
        value,
        effects: EffectSet::pure(),
    };
    // Comparison in concat is an error
    assert!(e.emit(&ir).is_err());
}

#[test]
fn test_COV_POSIX_028_concat_logical_and_returns_error() {
    let e = emitter();
    let value = ShellValue::Concat(vec![ShellValue::LogicalAnd {
        left: Box::new(ShellValue::Bool(true)),
        right: Box::new(ShellValue::Bool(false)),
    }]);
    // LogicalAnd in concat is an error (even for constant values the concat
    // path hits the error arm before constant-folding)
    let ir = ShellIR::Let {
        name: "v".to_string(),
        value,
        effects: EffectSet::pure(),
    };
    assert!(e.emit(&ir).is_err());
}

// ---------------------------------------------------------------------------
// Arithmetic operand — DynamicArrayAccess and CommandSubst
// ---------------------------------------------------------------------------

include!("posix_coverage_tests_COV_COV.rs");
