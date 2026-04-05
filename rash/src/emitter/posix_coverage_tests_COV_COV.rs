
#[test]
fn test_COV_POSIX_029_arithmetic_with_cmd_subst_operand() {
    let ir = ShellIR::Let {
        name: "n".to_string(),
        value: ShellValue::Arithmetic {
            op: ArithmeticOp::Add,
            left: Box::new(ShellValue::CommandSubst(Command {
                program: "wc".to_string(),
                args: vec![ShellValue::String("-l".to_string())],
            })),
            right: Box::new(ShellValue::String("0".to_string())),
        },
        effects: EffectSet::pure(),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("$(wc"));
    assert!(result.contains("+"));
}

#[test]
fn test_COV_POSIX_030_arithmetic_with_dynamic_array_operand() {
    let ir = ShellIR::Let {
        name: "sum".to_string(),
        value: ShellValue::Arithmetic {
            op: ArithmeticOp::Add,
            left: Box::new(ShellValue::DynamicArrayAccess {
                array: "nums".to_string(),
                index: Box::new(ShellValue::Variable("i".to_string())),
            }),
            right: Box::new(ShellValue::String("0".to_string())),
        },
        effects: EffectSet::pure(),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("nums"));
    assert!(result.contains("+"));
}

#[test]
fn test_COV_POSIX_031_arithmetic_unsupported_operand_returns_error() {
    let e = emitter();
    let ir = ShellIR::Let {
        name: "n".to_string(),
        value: ShellValue::Arithmetic {
            op: ArithmeticOp::Add,
            left: Box::new(ShellValue::Bool(true)), // Bool unsupported in arithmetic
            right: Box::new(ShellValue::String("1".to_string())),
        },
        effects: EffectSet::pure(),
    };
    assert!(e.emit(&ir).is_err());
}

// ---------------------------------------------------------------------------
// Arithmetic operators — all variants
// ---------------------------------------------------------------------------

#[test]
fn test_COV_POSIX_032_arithmetic_sub() {
    let ir = ShellIR::Let {
        name: "diff".to_string(),
        value: ShellValue::Arithmetic {
            op: ArithmeticOp::Sub,
            left: Box::new(ShellValue::String("10".to_string())),
            right: Box::new(ShellValue::String("3".to_string())),
        },
        effects: EffectSet::pure(),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("10 - 3"));
}

#[test]
fn test_COV_POSIX_033_arithmetic_mul() {
    let ir = ShellIR::Let {
        name: "prod".to_string(),
        value: ShellValue::Arithmetic {
            op: ArithmeticOp::Mul,
            left: Box::new(ShellValue::String("4".to_string())),
            right: Box::new(ShellValue::String("5".to_string())),
        },
        effects: EffectSet::pure(),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("4 * 5"));
}

#[test]
fn test_COV_POSIX_034_arithmetic_div() {
    let ir = ShellIR::Let {
        name: "quot".to_string(),
        value: ShellValue::Arithmetic {
            op: ArithmeticOp::Div,
            left: Box::new(ShellValue::String("20".to_string())),
            right: Box::new(ShellValue::String("4".to_string())),
        },
        effects: EffectSet::pure(),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("20 / 4"));
}

#[test]
fn test_COV_POSIX_035_arithmetic_mod() {
    let ir = ShellIR::Let {
        name: "rem".to_string(),
        value: ShellValue::Arithmetic {
            op: ArithmeticOp::Mod,
            left: Box::new(ShellValue::String("7".to_string())),
            right: Box::new(ShellValue::String("3".to_string())),
        },
        effects: EffectSet::pure(),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("7 % 3"));
}

#[test]
fn test_COV_POSIX_036_arithmetic_bitand() {
    let ir = ShellIR::Let {
        name: "r".to_string(),
        value: ShellValue::Arithmetic {
            op: ArithmeticOp::BitAnd,
            left: Box::new(ShellValue::String("15".to_string())),
            right: Box::new(ShellValue::String("9".to_string())),
        },
        effects: EffectSet::pure(),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("15 & 9"));
}

#[test]
fn test_COV_POSIX_037_arithmetic_bitor() {
    let ir = ShellIR::Let {
        name: "r".to_string(),
        value: ShellValue::Arithmetic {
            op: ArithmeticOp::BitOr,
            left: Box::new(ShellValue::String("6".to_string())),
            right: Box::new(ShellValue::String("3".to_string())),
        },
        effects: EffectSet::pure(),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("6 | 3"));
}

#[test]
fn test_COV_POSIX_038_arithmetic_bitxor() {
    let ir = ShellIR::Let {
        name: "r".to_string(),
        value: ShellValue::Arithmetic {
            op: ArithmeticOp::BitXor,
            left: Box::new(ShellValue::String("5".to_string())),
            right: Box::new(ShellValue::String("3".to_string())),
        },
        effects: EffectSet::pure(),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("5 ^ 3"));
}

#[test]
fn test_COV_POSIX_039_arithmetic_shl() {
    let ir = ShellIR::Let {
        name: "r".to_string(),
        value: ShellValue::Arithmetic {
            op: ArithmeticOp::Shl,
            left: Box::new(ShellValue::String("1".to_string())),
            right: Box::new(ShellValue::String("4".to_string())),
        },
        effects: EffectSet::pure(),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("1 << 4"));
}

#[test]
fn test_COV_POSIX_040_arithmetic_shr() {
    let ir = ShellIR::Let {
        name: "r".to_string(),
        value: ShellValue::Arithmetic {
            op: ArithmeticOp::Shr,
            left: Box::new(ShellValue::String("64".to_string())),
            right: Box::new(ShellValue::String("2".to_string())),
        },
        effects: EffectSet::pure(),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("64 >> 2"));
}

// ---------------------------------------------------------------------------
// Nested arithmetic with precedence-based parenthesization
// ---------------------------------------------------------------------------

#[test]
fn test_COV_POSIX_041_nested_arithmetic_parens() {
    // (1 + 2) * 3 — the Add sub-expression must be wrapped in parens
    let ir = ShellIR::Let {
        name: "r".to_string(),
        value: ShellValue::Arithmetic {
            op: ArithmeticOp::Mul,
            left: Box::new(ShellValue::Arithmetic {
                op: ArithmeticOp::Add,
                left: Box::new(ShellValue::String("1".to_string())),
                right: Box::new(ShellValue::String("2".to_string())),
            }),
            right: Box::new(ShellValue::String("3".to_string())),
        },
        effects: EffectSet::pure(),
    };
    let result = emitter().emit(&ir).unwrap();
    assert!(result.contains("(1 + 2)"));
    assert!(result.contains("* 3"));
}

// ---------------------------------------------------------------------------
// While loop — compound conditions
// ---------------------------------------------------------------------------

include!("posix_coverage_tests_COV_COV_COV.rs");
