//! Additional emitter coverage tests for posix.rs and makefile.rs uncovered branches.
//!
//! Targets branches NOT already tested in posix_coverage_tests.rs / makefile_coverage_tests.rs:
//! emit_logical_operand (nested Or/Not/Arithmetic/Comparison/Bool/String), concat with
//! Arithmetic/CommandSubst/LogicalOr/LogicalNot error paths, emit_while_condition nested
//! compound, triple elif, Sequence-wrapped elif, function user-defined, test LogicalOr/Variable,
//! all comparison operators, ForIn, Case wildcard, Exit, Continue/Break, Noop, Makefile targets,
//! emit_shell_value flattened concat, Bash dialect fallback, Bool(false) in test.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use crate::emitter::posix::PosixEmitter;
use crate::ir::shell_ir::{ArithmeticOp, CaseArm, CasePattern, ComparisonOp};
use crate::ir::{Command, EffectSet, ShellIR, ShellValue};
use crate::models::Config;

fn e() -> PosixEmitter { PosixEmitter::new(Config::default()) }
fn s(v: &str) -> ShellValue { ShellValue::String(v.to_string()) }
fn var(v: &str) -> ShellValue { ShellValue::Variable(v.to_string()) }
fn cmp(op: ComparisonOp, l: ShellValue, r: ShellValue) -> ShellValue {
    ShellValue::Comparison { op, left: Box::new(l), right: Box::new(r) }
}
fn echo(v: ShellValue) -> ShellIR { ShellIR::Echo { value: v } }
fn if_ir(test: ShellValue, then: ShellIR, el: Option<ShellIR>) -> ShellIR {
    ShellIR::If { test, then_branch: Box::new(then), else_branch: el.map(Box::new) }
}

// --- emit_logical_operand: nested Or, Not, Arithmetic, Comparison, Bool, String ---

#[test]
fn test_ECOV_001_logical_operand_nested_or() {
    let ir = echo(ShellValue::LogicalAnd {
        left: Box::new(ShellValue::LogicalOr { left: Box::new(var("a")), right: Box::new(var("b")) }),
        right: Box::new(var("c")),
    });
    let r = e().emit(&ir).unwrap();
    assert!(r.contains("||") && r.contains("&&"), "Nested OR in AND: {r}");
}

#[test]
fn test_ECOV_002_logical_operand_nested_not() {
    let ir = echo(ShellValue::LogicalAnd {
        left: Box::new(ShellValue::LogicalNot { operand: Box::new(var("flag")) }),
        right: Box::new(var("ok")),
    });
    let r = e().emit(&ir).unwrap();
    assert!(r.contains("!flag"), "Negated in logical operand: {r}");
}

#[test]
fn test_ECOV_003_logical_operand_with_arithmetic() {
    let ir = echo(ShellValue::LogicalAnd {
        left: Box::new(ShellValue::Arithmetic {
            op: ArithmeticOp::Add, left: Box::new(var("x")), right: Box::new(s("1")),
        }),
        right: Box::new(ShellValue::Bool(true)),
    });
    let r = e().emit(&ir).unwrap();
    assert!(r.contains("x + 1"), "Arithmetic in logical: {r}");
}

#[test]
fn test_ECOV_004_logical_operand_with_comparison() {
    let ir = echo(ShellValue::LogicalOr {
        left: Box::new(cmp(ComparisonOp::Gt, var("x"), s("0"))),
        right: Box::new(ShellValue::Bool(false)),
    });
    let r = e().emit(&ir).unwrap();
    assert!(r.contains("-gt"), "Comparison in logical: {r}");
}

#[test]
fn test_ECOV_005_logical_operand_bool_values() {
    let ir = echo(ShellValue::LogicalAnd {
        left: Box::new(ShellValue::Bool(false)), right: Box::new(var("v")),
    });
    let r = e().emit(&ir).unwrap();
    assert!(r.contains("0"), "Bool(false) emits 0: {r}");
}

#[test]
fn test_ECOV_033_logical_operand_string_fallback() {
    let ir = echo(ShellValue::LogicalAnd {
        left: Box::new(s("1")), right: Box::new(s("0")),
    });
    let r = e().emit(&ir).unwrap();
    assert!(r.contains("&&"), "Logical AND with strings: {r}");
}

// --- append_concat_part: Arithmetic, CommandSubst, LogicalOr/Not error paths ---

#[test]
fn test_ECOV_006_concat_with_arithmetic() {
    let ir = ShellIR::Let {
        name: "msg".into(),
        value: ShellValue::Concat(vec![
            s("count="),
            ShellValue::Arithmetic { op: ArithmeticOp::Add, left: Box::new(var("n")), right: Box::new(s("1")) },
        ]),
        effects: EffectSet::pure(),
    };
    let r = e().emit(&ir).unwrap();
    assert!(r.contains("count=") && r.contains("$(("), "Arith in concat: {r}");
}

#[test]
fn test_ECOV_007_concat_with_command_subst() {
    let ir = ShellIR::Let {
        name: "info".into(),
        value: ShellValue::Concat(vec![
            s("user="),
            ShellValue::CommandSubst(Command { program: "whoami".into(), args: vec![] }),
        ]),
        effects: EffectSet::pure(),
    };
    let r = e().emit(&ir).unwrap();
    assert!(r.contains("$(whoami)"), "Command subst in concat: {r}");
}

#[test]
fn test_ECOV_029_concat_logical_or_returns_error() {
    let ir = ShellIR::Let {
        name: "v".into(),
        value: ShellValue::Concat(vec![ShellValue::LogicalOr {
            left: Box::new(ShellValue::Bool(true)), right: Box::new(ShellValue::Bool(false)),
        }]),
        effects: EffectSet::pure(),
    };
    assert!(e().emit(&ir).is_err(), "LogicalOr in concat is error");
}

#[test]
fn test_ECOV_030_concat_logical_not_returns_error() {
    let ir = ShellIR::Let {
        name: "v".into(),
        value: ShellValue::Concat(vec![ShellValue::LogicalNot {
            operand: Box::new(ShellValue::Bool(true)),
        }]),
        effects: EffectSet::pure(),
    };
    assert!(e().emit(&ir).is_err(), "LogicalNot in concat is error");
}

// --- emit_while_condition: nested compound, Bool(false), String, Comparison ---

#[test]
fn test_ECOV_008_while_condition_nested_and_in_or() {
    let ir = ShellIR::While {
        condition: ShellValue::LogicalOr {
            left: Box::new(ShellValue::LogicalAnd {
                left: Box::new(cmp(ComparisonOp::Lt, var("i"), s("10"))),
                right: Box::new(cmp(ComparisonOp::Gt, var("j"), s("0"))),
            }),
            right: Box::new(ShellValue::Bool(true)),
        },
        body: Box::new(ShellIR::Break),
    };
    let r = e().emit(&ir).unwrap();
    assert!(r.contains("&&") && r.contains("||"), "Nested AND in OR: {r}");
}

#[test]
fn test_ECOV_009_while_condition_not_wrapping_and() {
    let ir = ShellIR::While {
        condition: ShellValue::LogicalNot {
            operand: Box::new(ShellValue::LogicalAnd {
                left: Box::new(cmp(ComparisonOp::NumEq, var("x"), s("1"))),
                right: Box::new(cmp(ComparisonOp::NumEq, var("y"), s("1"))),
            }),
        },
        body: Box::new(ShellIR::Break),
    };
    let r = e().emit(&ir).unwrap();
    assert!(r.contains("! ") && r.contains("&&"), "NOT wrapping AND: {r}");
}

#[test]
fn test_ECOV_028_while_condition_string_value() {
    let ir = ShellIR::While { condition: s("1"), body: Box::new(ShellIR::Break) };
    let r = e().emit(&ir).unwrap();
    assert!(r.contains("while [ 1 ]"), "String as condition: {r}");
}

#[test]
fn test_ECOV_032_while_condition_bool_false_nested() {
    let ir = ShellIR::While {
        condition: ShellValue::LogicalOr {
            left: Box::new(ShellValue::Bool(false)),
            right: Box::new(cmp(ComparisonOp::Lt, var("i"), s("5"))),
        },
        body: Box::new(ShellIR::Break),
    };
    let r = e().emit(&ir).unwrap();
    assert!(r.contains("false") && r.contains("||"), "Bool false in nested: {r}");
}

#[test]
fn test_ECOV_038_while_comparison_condition() {
    let ir = ShellIR::While {
        condition: cmp(ComparisonOp::Lt, var("i"), s("10")),
        body: Box::new(ShellIR::Noop),
    };
    let r = e().emit(&ir).unwrap();
    assert!(r.contains("while [ ") && r.contains("-lt"), "Comparison in while: {r}");
}

// --- Triple elif, Sequence-wrapped elif ---

#[test]
fn test_ECOV_010_triple_elif_chain() {
    let ir = if_ir(
        cmp(ComparisonOp::NumEq, var("x"), s("1")), echo(s("one")),
        Some(if_ir(
            cmp(ComparisonOp::NumEq, var("x"), s("2")), echo(s("two")),
            Some(if_ir(
                cmp(ComparisonOp::NumEq, var("x"), s("3")), echo(s("three")),
                Some(echo(s("other"))),
            )),
        )),
    );
    let r = e().emit(&ir).unwrap();
    assert_eq!(r.matches("elif").count(), 2, "Two elif: {r}");
    assert!(r.contains("else"), "Final else: {r}");
}

#[test]
fn test_ECOV_011_elif_via_sequence_wrapped_if() {
    let ir = if_ir(
        ShellValue::Bool(true), echo(s("first")),
        Some(ShellIR::Sequence(vec![if_ir(ShellValue::Bool(false), echo(s("second")), None)])),
    );
    let r = e().emit(&ir).unwrap();
    assert!(r.contains("elif"), "Sequence-wrapped If becomes elif: {r}");
}

// --- Functions ---

#[test]
fn test_ECOV_012_function_user_defined_with_body() {
    let ir = ShellIR::Sequence(vec![ShellIR::Function {
        name: "greet".into(), params: vec!["name".into()],
        body: Box::new(echo(var("name"))),
    }]);
    let r = e().emit(&ir).unwrap();
    assert!(r.contains("greet()") && r.contains("name=\"$1\"") && r.contains("echo"), "{r}");
}

#[test]
fn test_ECOV_013_function_unknown_command_empty_body() {
    let ir = ShellIR::Function { name: "my_custom_cmd".into(), params: vec![], body: Box::new(ShellIR::Noop) };
    let r = e().emit(&ir).unwrap();
    assert!(r.contains("my_custom_cmd()"), "Custom func emitted: {r}");
}

// --- Test expressions: LogicalOr runtime, Variable, Bool(false) ---

#[test]
fn test_ECOV_014_test_logical_or_runtime() {
    let r = e().emit(&if_ir(
        ShellValue::LogicalOr { left: Box::new(var("a")), right: Box::new(var("b")) },
        ShellIR::Noop, None,
    )).unwrap();
    assert!(r.contains("||"), "Runtime LogicalOr in test: {r}");
}

#[test]
fn test_ECOV_015_test_variable() {
    let r = e().emit(&if_ir(var("flag"), ShellIR::Noop, None)).unwrap();
    assert!(r.contains("test -n") && r.contains("$flag"), "Variable test: {r}");
}

#[test]
fn test_ECOV_040_test_bool_false() {
    let r = e().emit(&if_ir(ShellValue::Bool(false), ShellIR::Noop, None)).unwrap();
    assert!(r.contains("if false"), "Bool false: {r}");
}

// --- Comparison operators: Ge, Le, Ne, Gt, Lt ---

#[test]
fn test_ECOV_016_comparison_ge() {
    let r = e().emit(&if_ir(cmp(ComparisonOp::Ge, var("x"), s("5")), ShellIR::Noop, None)).unwrap();
    assert!(r.contains("-ge"), "Ge: {r}");
}

#[test]
fn test_ECOV_017_comparison_le() {
    let r = e().emit(&if_ir(cmp(ComparisonOp::Le, var("y"), s("100")), ShellIR::Noop, None)).unwrap();
    assert!(r.contains("-le"), "Le: {r}");
}

#[test]
fn test_ECOV_018_comparison_ne() {
    let r = e().emit(&if_ir(cmp(ComparisonOp::NumNe, var("z"), s("0")), ShellIR::Noop, None)).unwrap();
    assert!(r.contains("-ne"), "Ne: {r}");
}

#[test]
fn test_ECOV_019_comparison_gt() {
    let r = e().emit(&if_ir(cmp(ComparisonOp::Gt, var("n"), s("42")), ShellIR::Noop, None)).unwrap();
    assert!(r.contains("-gt"), "Gt: {r}");
}

#[test]
fn test_ECOV_020_comparison_lt() {
    let r = e().emit(&if_ir(cmp(ComparisonOp::Lt, var("m"), s("10")), ShellIR::Noop, None)).unwrap();
    assert!(r.contains("-lt"), "Lt: {r}");
}

// --- For range, ForIn, Case wildcard ---

#[test]
fn test_ECOV_021_for_range() {
    let ir = ShellIR::For { var: "i".into(), start: s("0"), end: s("5"), body: Box::new(echo(var("i"))) };
    let r = e().emit(&ir).unwrap();
    assert!(r.contains("for i in $(seq") && r.contains("done"), "Seq loop: {r}");
}

#[test]
fn test_ECOV_034_for_in_with_variable_items() {
    let ir = ShellIR::ForIn {
        var: "file".into(),
        items: vec![var("src_dir"), s("extra.txt")],
        body: Box::new(echo(var("file"))),
    };
    let r = e().emit(&ir).unwrap();
    assert!(r.contains("for file in") && r.contains("$src_dir"), "ForIn: {r}");
}

#[test]
fn test_ECOV_022_case_wildcard_pattern() {
    let ir = ShellIR::Case {
        scrutinee: var("cmd"),
        arms: vec![
            CaseArm { pattern: CasePattern::Literal("start".into()), guard: None,
                       body: Box::new(echo(s("starting"))) },
            CaseArm { pattern: CasePattern::Wildcard, guard: None,
                       body: Box::new(echo(s("unknown"))) },
        ],
    };
    let r = e().emit(&ir).unwrap();
    assert!(r.contains("start)") && r.contains("*)") && r.contains("esac"), "Case: {r}");
}

// --- Exit, Continue, Break, Noop ---

#[test]
fn test_ECOV_023_exit_with_message() {
    let r = e().emit(&ShellIR::Exit { code: 1, message: Some("fatal error".into()) }).unwrap();
    assert!(r.contains("fatal error") && r.contains(">&2") && r.contains("exit 1"), "{r}");
}

#[test]
fn test_ECOV_024_exit_without_message() {
    let r = e().emit(&ShellIR::Exit { code: 0, message: None }).unwrap();
    assert!(r.contains("exit 0") && !r.contains(">&2"), "{r}");
}

#[test]
fn test_ECOV_025_continue_in_while() {
    let r = e().emit(&ShellIR::While { condition: ShellValue::Bool(true), body: Box::new(ShellIR::Continue) }).unwrap();
    assert!(r.contains("continue"), "Continue: {r}");
}

#[test]
fn test_ECOV_026_break_in_for() {
    let ir = ShellIR::For { var: "i".into(), start: s("0"), end: s("10"), body: Box::new(ShellIR::Break) };
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
            op: ArithmeticOp::Div, left: Box::new(s("10")),
            right: Box::new(ShellValue::Arithmetic {
                op: ArithmeticOp::Sub, left: Box::new(s("3")), right: Box::new(s("1")),
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
    let val = ShellValue::Concat(vec![
        s("a"), ShellValue::Concat(vec![var("x")]),
    ]);
    let r = e().emit_shell_value(&val).unwrap();
    assert!(r.contains("a") && r.contains("${x}"), "Flattened: {r}");
}

#[test]
fn test_ECOV_039_emit_bash_dialect() {
    use crate::emitter::emit;
    use crate::models::config::ShellDialect;
    let mut config = Config::default();
    config.target = ShellDialect::Bash;
    let r = emit(&echo(s("hello")), &config).unwrap();
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
            name: "main".into(), params: vec![], return_type: Type::Void,
            body: vec![Stmt::Expr(Expr::FunctionCall {
                name: "target".into(),
                args: vec![
                    Expr::Literal(Literal::Str("build".into())),
                    Expr::Array(vec![Expr::Literal(Literal::Str("main.c".into()))]),
                    Expr::Array(vec![Expr::Literal(Literal::Str("gcc -o build main.c".into()))]),
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
            Function { name: "main".into(), params: vec![], return_type: Type::Void,
                body: vec![Stmt::Let { name: "x".into(), value: Expr::Literal(Literal::Str("val".into())), declaration: true }] },
            Function { name: "helper".into(), params: vec![], return_type: Type::Void,
                body: vec![Stmt::Let { name: "y".into(), value: Expr::Literal(Literal::Str("inner".into())), declaration: true }] },
        ],
        entry_point: "main".into(),
    };
    let r = emit_makefile(&ast).unwrap();
    assert!(!r.contains("helper:"), "No helper target: {r}");
}
