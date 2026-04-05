fn test_ECOV_040_test_bool_false() {
    let r = e()
        .emit(&if_ir(ShellValue::Bool(false), ShellIR::Noop, None))
        .unwrap();
    assert!(r.contains("if false"), "Bool false: {r}");
}

// --- Comparison operators: Ge, Le, Ne, Gt, Lt ---

#[test]
fn test_ECOV_016_comparison_ge() {
    let r = e()
        .emit(&if_ir(
            cmp(ComparisonOp::Ge, var("x"), s("5")),
            ShellIR::Noop,
            None,
        ))
        .unwrap();
    assert!(r.contains("-ge"), "Ge: {r}");
}

#[test]
fn test_ECOV_017_comparison_le() {
    let r = e()
        .emit(&if_ir(
            cmp(ComparisonOp::Le, var("y"), s("100")),
            ShellIR::Noop,
            None,
        ))
        .unwrap();
    assert!(r.contains("-le"), "Le: {r}");
}

#[test]
fn test_ECOV_018_comparison_ne() {
    let r = e()
        .emit(&if_ir(
            cmp(ComparisonOp::NumNe, var("z"), s("0")),
            ShellIR::Noop,
            None,
        ))
        .unwrap();
    assert!(r.contains("-ne"), "Ne: {r}");
}

#[test]
fn test_ECOV_019_comparison_gt() {
    let r = e()
        .emit(&if_ir(
            cmp(ComparisonOp::Gt, var("n"), s("42")),
            ShellIR::Noop,
            None,
        ))
        .unwrap();
    assert!(r.contains("-gt"), "Gt: {r}");
}

#[test]
fn test_ECOV_020_comparison_lt() {
    let r = e()
        .emit(&if_ir(
            cmp(ComparisonOp::Lt, var("m"), s("10")),
            ShellIR::Noop,
            None,
        ))
        .unwrap();
    assert!(r.contains("-lt"), "Lt: {r}");
}

// --- For range, ForIn, Case wildcard ---

#[test]
fn test_ECOV_021_for_range() {
    let ir = ShellIR::For {
        var: "i".into(),
        start: s("0"),
        end: s("5"),
        body: Box::new(echo(var("i"))),
    };
    let r = e().emit(&ir).unwrap();
    assert!(
        r.contains("for i in $(seq") && r.contains("done"),
        "Seq loop: {r}"
    );
}

#[test]
fn test_ECOV_034_for_in_with_variable_items() {
    let ir = ShellIR::ForIn {
        var: "file".into(),
        items: vec![var("src_dir"), s("extra.txt")],
        body: Box::new(echo(var("file"))),
    };
    let r = e().emit(&ir).unwrap();
    assert!(
        r.contains("for file in") && r.contains("$src_dir"),
        "ForIn: {r}"
    );
}

#[test]
fn test_ECOV_022_case_wildcard_pattern() {
    let ir = ShellIR::Case {
        scrutinee: var("cmd"),
        arms: vec![
            CaseArm {
                pattern: CasePattern::Literal("start".into()),
                guard: None,
                body: Box::new(echo(s("starting"))),
            },
            CaseArm {
                pattern: CasePattern::Wildcard,
                guard: None,
                body: Box::new(echo(s("unknown"))),
            },
        ],
    };
    let r = e().emit(&ir).unwrap();
    assert!(
        r.contains("start)") && r.contains("*)") && r.contains("esac"),
        "Case: {r}"
    );
}

// --- Exit, Continue, Break, Noop ---

#[test]
fn test_ECOV_023_exit_with_message() {
    let r = e()
        .emit(&ShellIR::Exit {
            code: 1,
            message: Some("fatal error".into()),
        })
        .unwrap();
    assert!(
        r.contains("fatal error") && r.contains(">&2") && r.contains("exit 1"),
        "{r}"
    );
}

#[test]
fn test_ECOV_024_exit_without_message() {
    let r = e()
        .emit(&ShellIR::Exit {
            code: 0,
            message: None,
        })
        .unwrap();
    assert!(r.contains("exit 0") && !r.contains(">&2"), "{r}");
}

#[test]
fn test_ECOV_025_continue_in_while() {
    let r = e()
        .emit(&ShellIR::While {
            condition: ShellValue::Bool(true),
            body: Box::new(ShellIR::Continue),
        })
        .unwrap();
    assert!(r.contains("continue"), "Continue: {r}");
}

#[test]
fn test_ECOV_026_break_in_for() {
    let ir = ShellIR::For {
        var: "i".into(),
        start: s("0"),
        end: s("10"),
        body: Box::new(ShellIR::Break),
    };
    let r = e().emit(&ir).unwrap();
    assert!(r.contains("break"), "Break: {r}");
}

#[test]
fn test_ECOV_027_noop_standalone() {
    let r = e().emit(&ShellIR::Noop).unwrap();
    assert!(r.contains(':'), "Noop emits colon: {r}");
}

// --- Arithmetic right-side parens ---

#[test]
fn test_ECOV_031_arithmetic_right_side_parens() {
    let ir = ShellIR::Let {
        name: "r".into(),
        value: ShellValue::Arithmetic {
            op: ArithmeticOp::Div,
            left: Box::new(s("10")),
            right: Box::new(ShellValue::Arithmetic {
                op: ArithmeticOp::Sub,
                left: Box::new(s("3")),
                right: Box::new(s("1")),
            }),
        },
        effects: EffectSet::pure(),
    };
    let r = e().emit(&ir).unwrap();
    assert!(r.contains("(3 - 1)"), "Right side parenthesized: {r}");
}

// --- Flattened concat, Bash dialect ---

#[test]
fn test_ECOV_037_flattened_content_no_quotes() {
    let val = ShellValue::Concat(vec![s("a"), ShellValue::Concat(vec![var("x")])]);
    let r = e().emit_shell_value(&val).unwrap();
    assert!(r.contains("a") && r.contains("${x}"), "Flattened: {r}");
}

#[test]
fn test_ECOV_039_emit_bash_dialect() {
    use crate::emitter::emit;
    let r = emit(&echo(s("hello"))).unwrap();
    assert!(r.contains("echo"), "Bash dialect emits: {r}");
}

// --- Makefile additional branch coverage ---

#[test]
fn test_ECOV_035_makefile_target_with_recipes() {
    use crate::ast::restricted::{Function, Literal, Type};
    use crate::ast::{Expr, RestrictedAst, Stmt};
    use crate::emitter::makefile::emit_makefile;
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".into(),
            params: vec![],
            return_type: Type::Void,
            body: vec![Stmt::Expr(Expr::FunctionCall {
                name: "target".into(),
                args: vec![
                    Expr::Literal(Literal::Str("build".into())),
                    Expr::Array(vec![Expr::Literal(Literal::Str("main.c".into()))]),
                    Expr::Array(vec![Expr::Literal(Literal::Str(
                        "gcc -o build main.c".into(),
                    ))]),
                ],
            })],
        }],
        entry_point: "main".into(),
    };
    let r = emit_makefile(&ast).unwrap();
    assert!(r.contains("build"), "Target present: {r}");
}

#[test]
fn test_ECOV_036_makefile_non_main_function_no_echo() {
    use crate::ast::restricted::{Function, Literal, Type};
    use crate::ast::{Expr, RestrictedAst, Stmt};
    use crate::emitter::makefile::emit_makefile;
    let ast = RestrictedAst {
        functions: vec![
            Function {
                name: "main".into(),
                params: vec![],
                return_type: Type::Void,
                body: vec![Stmt::Let {
                    name: "x".into(),
                    value: Expr::Literal(Literal::Str("val".into())),
                    declaration: true,
                }],
            },
            Function {
                name: "helper".into(),
                params: vec![],
                return_type: Type::Void,
                body: vec![Stmt::Let {
                    name: "y".into(),
                    value: Expr::Literal(Literal::Str("inner".into())),
                    declaration: true,
                }],
            },
        ],
        entry_point: "main".into(),
    };
    let r = emit_makefile(&ast).unwrap();
    assert!(!r.contains("helper:"), "No helper target: {r}");
}
