//! Additional tests for posix.rs - improving coverage from 86% to 92%+

use super::posix::PosixEmitter;
use crate::ir::shell_ir::CaseArm;
use crate::ir::{shell_ir::CasePattern, Command, EffectSet, ShellIR, ShellValue};
use crate::models::Config;

// ============================================================================
// IR Type Coverage Tests
// ============================================================================

#[test]
fn test_exit_statement_with_message() {
    let config = Config::default();
    let emitter = PosixEmitter::new(config);

    let ir = ShellIR::Exit {
        code: 1,
        message: Some("Error occurred".to_string()),
    };

    let result = emitter.emit(&ir).unwrap();
    assert!(result.contains("exit 1"));
    assert!(result.contains("Error occurred"));
}

#[test]
fn test_exit_statement_without_message() {
    let config = Config::default();
    let emitter = PosixEmitter::new(config);

    let ir = ShellIR::Exit {
        code: 0,
        message: None,
    };

    let result = emitter.emit(&ir).unwrap();
    assert!(result.contains("exit 0"));
}

#[test]
fn test_noop_emission() {
    let config = Config::default();
    let emitter = PosixEmitter::new(config);

    let ir = ShellIR::Noop;

    let result = emitter.emit(&ir).unwrap();
    // Noop should produce a valid script but with minimal content
    assert!(result.contains("#!/bin/sh"));
}

#[test]
fn test_function_emission() {
    let config = Config::default();
    let emitter = PosixEmitter::new(config);

    let func = ShellIR::Function {
        name: "my_func".to_string(),
        params: vec!["arg1".to_string(), "arg2".to_string()],
        body: Box::new(ShellIR::Echo {
            value: ShellValue::String("hello".to_string()),
        }),
    };

    let result = emitter.emit(&func).unwrap();
    assert!(result.contains("my_func()"));
    assert!(result.contains("echo hello"));
}

#[test]
fn test_echo_statement_emission() {
    let config = Config::default();
    let emitter = PosixEmitter::new(config);

    let ir = ShellIR::Echo {
        value: ShellValue::String("test output".to_string()),
    };

    let result = emitter.emit(&ir).unwrap();
    assert!(result.contains("echo 'test output'"));
}

#[test]
fn test_echo_with_variable() {
    let config = Config::default();
    let emitter = PosixEmitter::new(config);

    let ir = ShellIR::Echo {
        value: ShellValue::Variable("my_var".to_string()),
    };

    let result = emitter.emit(&ir).unwrap();
    assert!(result.contains("echo \"$my_var\""));
}

#[test]
fn test_for_loop_emission() {
    let config = Config::default();
    let emitter = PosixEmitter::new(config);

    let ir = ShellIR::For {
        var: "i".to_string(),
        start: ShellValue::String("1".to_string()),
        end: ShellValue::String("10".to_string()),
        body: Box::new(ShellIR::Echo {
            value: ShellValue::Variable("i".to_string()),
        }),
    };

    let result = emitter.emit(&ir).unwrap();
    assert!(result.contains("for i in $(seq 1 10)"));
    assert!(result.contains("do"));
    assert!(result.contains("done"));
    assert!(result.contains("echo \"$i\""));
}

#[test]
fn test_while_loop_emission() {
    let config = Config::default();
    let emitter = PosixEmitter::new(config);

    let ir = ShellIR::While {
        condition: ShellValue::Bool(true),
        body: Box::new(ShellIR::Break),
    };

    let result = emitter.emit(&ir).unwrap();
    assert!(result.contains("while true"));
    assert!(result.contains("do"));
    assert!(result.contains("break"));
    assert!(result.contains("done"));
}

#[test]
fn test_while_with_comparison() {
    let config = Config::default();
    let emitter = PosixEmitter::new(config);

    let ir = ShellIR::While {
        condition: ShellValue::Comparison {
            op: crate::ir::shell_ir::ComparisonOp::Lt,
            left: Box::new(ShellValue::Variable("count".to_string())),
            right: Box::new(ShellValue::String("10".to_string())),
        },
        body: Box::new(ShellIR::Noop),
    };

    let result = emitter.emit(&ir).unwrap();
    assert!(result.contains("while"));
    assert!(result.contains("-lt"));
}

#[test]
fn test_case_statement_emission() {
    let config = Config::default();
    let emitter = PosixEmitter::new(config);

    let ir = ShellIR::Case {
        scrutinee: ShellValue::Variable("choice".to_string()),
        arms: vec![
            CaseArm {
                pattern: CasePattern::Literal("1".to_string()),
                guard: None,
                body: Box::new(ShellIR::Echo {
                    value: ShellValue::String("One".to_string()),
                }),
            },
            CaseArm {
                pattern: CasePattern::Wildcard,
                guard: None,
                body: Box::new(ShellIR::Echo {
                    value: ShellValue::String("Other".to_string()),
                }),
            },
        ],
    };

    let result = emitter.emit(&ir).unwrap();
    assert!(result.contains("case"));
    assert!(result.contains("choice"));
    assert!(result.contains("1)"));
    assert!(result.contains("echo"));
    assert!(result.contains("One"));
    assert!(result.contains("*)"));
    assert!(result.contains("Other"));
    assert!(result.contains("esac"));
}

#[test]
fn test_break_emission() {
    let config = Config::default();
    let emitter = PosixEmitter::new(config);

    let ir = ShellIR::Break;

    let result = emitter.emit(&ir).unwrap();
    assert!(result.contains("break"));
}

#[test]
fn test_continue_emission() {
    let config = Config::default();
    let emitter = PosixEmitter::new(config);

    let ir = ShellIR::Continue;

    let result = emitter.emit(&ir).unwrap();
    assert!(result.contains("continue"));
}

// ============================================================================
// ShellValue Type Coverage Tests
// ============================================================================

#[test]
fn test_shell_value_bool_true() {
    let config = Config::default();
    let emitter = PosixEmitter::new(config);

    let ir = ShellIR::Let {
        name: "flag".to_string(),
        value: ShellValue::Bool(true),
        effects: EffectSet::pure(),
    };

    let result = emitter.emit(&ir).unwrap();
    assert!(result.contains("flag=true"));
}

#[test]
fn test_shell_value_bool_false() {
    let config = Config::default();
    let emitter = PosixEmitter::new(config);

    let ir = ShellIR::Let {
        name: "flag".to_string(),
        value: ShellValue::Bool(false),
        effects: EffectSet::pure(),
    };

    let result = emitter.emit(&ir).unwrap();
    assert!(result.contains("flag=false"));
}

#[test]
fn test_shell_value_concat() {
    let config = Config::default();
    let emitter = PosixEmitter::new(config);

    let ir = ShellIR::Let {
        name: "result".to_string(),
        value: ShellValue::Concat(vec![
            ShellValue::String("hello".to_string()),
            ShellValue::String(" ".to_string()),
            ShellValue::Variable("name".to_string()),
        ]),
        effects: EffectSet::pure(),
    };

    let result = emitter.emit(&ir).unwrap();
    assert!(result.contains("result="));
    assert!(result.contains("hello"));
    // Variable expansion in concat - may be quoted
    assert!(result.contains("name") || result.contains("$name"));
}

#[test]
fn test_shell_value_command_subst() {
    let config = Config::default();
    let emitter = PosixEmitter::new(config);

    let ir = ShellIR::Let {
        name: "date".to_string(),
        value: ShellValue::CommandSubst(Command {
            program: "date".to_string(),
            args: vec![ShellValue::String("+%Y-%m-%d".to_string())],
        }),
        effects: EffectSet::pure(),
    };

    let result = emitter.emit(&ir).unwrap();
    assert!(result.contains("date="));
    assert!(result.contains("$(date"));
    assert!(result.contains("+%Y-%m-%d"));
}

#[test]
fn test_shell_value_comparison_eq() {
    let config = Config::default();
    let emitter = PosixEmitter::new(config);

    let ir = ShellIR::If {
        test: ShellValue::Comparison {
            op: crate::ir::shell_ir::ComparisonOp::NumEq,
            left: Box::new(ShellValue::Variable("x".to_string())),
            right: Box::new(ShellValue::String("5".to_string())),
        },
        then_branch: Box::new(ShellIR::Noop),
        else_branch: None,
    };

    let result = emitter.emit(&ir).unwrap();
    assert!(result.contains("if"));
    assert!(result.contains("-eq"));
}

#[test]
fn test_shell_value_comparison_ne() {
    let config = Config::default();
    let emitter = PosixEmitter::new(config);

    let ir = ShellIR::If {
        test: ShellValue::Comparison {
            op: crate::ir::shell_ir::ComparisonOp::NumNe,
            left: Box::new(ShellValue::Variable("x".to_string())),
            right: Box::new(ShellValue::String("5".to_string())),
        },
        then_branch: Box::new(ShellIR::Noop),
        else_branch: None,
    };

    let result = emitter.emit(&ir).unwrap();
    assert!(result.contains("-ne"));
}

#[test]
fn test_shell_value_comparison_gt() {
    let config = Config::default();
    let emitter = PosixEmitter::new(config);

    let ir = ShellIR::If {
        test: ShellValue::Comparison {
            op: crate::ir::shell_ir::ComparisonOp::Gt,
            left: Box::new(ShellValue::Variable("x".to_string())),
            right: Box::new(ShellValue::String("5".to_string())),
        },
        then_branch: Box::new(ShellIR::Noop),
        else_branch: None,
    };

    let result = emitter.emit(&ir).unwrap();
    assert!(result.contains("-gt"));
}

#[test]
fn test_shell_value_comparison_ge() {
    let config = Config::default();
    let emitter = PosixEmitter::new(config);

    let ir = ShellIR::If {
        test: ShellValue::Comparison {
            op: crate::ir::shell_ir::ComparisonOp::Ge,
            left: Box::new(ShellValue::Variable("x".to_string())),
            right: Box::new(ShellValue::String("5".to_string())),
        },
        then_branch: Box::new(ShellIR::Noop),
        else_branch: None,
    };

    let result = emitter.emit(&ir).unwrap();
    assert!(result.contains("-ge"));
}

#[test]
fn test_shell_value_comparison_le() {
    let config = Config::default();
    let emitter = PosixEmitter::new(config);

    let ir = ShellIR::If {
        test: ShellValue::Comparison {
            op: crate::ir::shell_ir::ComparisonOp::Le,
            left: Box::new(ShellValue::Variable("x".to_string())),
            right: Box::new(ShellValue::String("5".to_string())),
        },
        then_branch: Box::new(ShellIR::Noop),
        else_branch: None,
    };

    let result = emitter.emit(&ir).unwrap();
    assert!(result.contains("-le"));
}

#[test]
fn test_shell_value_arithmetic_add() {
    let config = Config::default();
    let emitter = PosixEmitter::new(config);

    let ir = ShellIR::Let {
        name: "sum".to_string(),
        value: ShellValue::Arithmetic {
            op: crate::ir::shell_ir::ArithmeticOp::Add,
            left: Box::new(ShellValue::String("10".to_string())),
            right: Box::new(ShellValue::String("5".to_string())),
        },
        effects: EffectSet::pure(),
    };

    let result = emitter.emit(&ir).unwrap();
    assert!(result.contains("sum="));
    assert!(result.contains("$(("));
}

#[test]
fn test_shell_value_arithmetic_sub() {
    let config = Config::default();
    let emitter = PosixEmitter::new(config);

    let ir = ShellIR::Let {
        name: "diff".to_string(),
        value: ShellValue::Arithmetic {
            op: crate::ir::shell_ir::ArithmeticOp::Sub,
            left: Box::new(ShellValue::String("10".to_string())),
            right: Box::new(ShellValue::String("3".to_string())),
        },
        effects: EffectSet::pure(),
    };

    let result = emitter.emit(&ir).unwrap();
    assert!(result.contains("diff="));
}

#[test]
fn test_shell_value_arithmetic_mul() {
    let config = Config::default();
    let emitter = PosixEmitter::new(config);

    let ir = ShellIR::Let {
        name: "product".to_string(),
        value: ShellValue::Arithmetic {
            op: crate::ir::shell_ir::ArithmeticOp::Mul,
            left: Box::new(ShellValue::String("10".to_string())),
            right: Box::new(ShellValue::String("3".to_string())),
        },
        effects: EffectSet::pure(),
    };

    let result = emitter.emit(&ir).unwrap();
    assert!(result.contains("product="));
}

#[test]
fn test_shell_value_arithmetic_div() {
    let config = Config::default();
    let emitter = PosixEmitter::new(config);

    let ir = ShellIR::Let {
        name: "quotient".to_string(),
        value: ShellValue::Arithmetic {
            op: crate::ir::shell_ir::ArithmeticOp::Div,
            left: Box::new(ShellValue::String("10".to_string())),
            right: Box::new(ShellValue::String("2".to_string())),
        },
        effects: EffectSet::pure(),
    };

    let result = emitter.emit(&ir).unwrap();
    assert!(result.contains("quotient="));
}

#[test]
fn test_shell_value_arithmetic_mod() {
    let config = Config::default();
    let emitter = PosixEmitter::new(config);

    let ir = ShellIR::Let {
        name: "remainder".to_string(),
        value: ShellValue::Arithmetic {
            op: crate::ir::shell_ir::ArithmeticOp::Mod,
            left: Box::new(ShellValue::String("10".to_string())),
            right: Box::new(ShellValue::String("3".to_string())),
        },
        effects: EffectSet::pure(),
    };

    let result = emitter.emit(&ir).unwrap();
    assert!(result.contains("remainder="));
}

// ============================================================================
// Complex Scenarios
// ============================================================================

#[test]
fn test_nested_for_loops() {
    let config = Config::default();
    let emitter = PosixEmitter::new(config);

    let ir = ShellIR::For {
        var: "i".to_string(),
        start: ShellValue::String("1".to_string()),
        end: ShellValue::String("3".to_string()),
        body: Box::new(ShellIR::For {
            var: "j".to_string(),
            start: ShellValue::String("1".to_string()),
            end: ShellValue::String("2".to_string()),
            body: Box::new(ShellIR::Echo {
                value: ShellValue::Concat(vec![
                    ShellValue::Variable("i".to_string()),
                    ShellValue::String(",".to_string()),
                    ShellValue::Variable("j".to_string()),
                ]),
            }),
        }),
    };

    let result = emitter.emit(&ir).unwrap();
    assert!(result.contains("for i in"));
    assert!(result.contains("for j in"));
    assert!(result.contains("echo"));
}

#[test]
fn test_case_with_guard() {
    let config = Config::default();
    let emitter = PosixEmitter::new(config);

    let ir = ShellIR::Case {
        scrutinee: ShellValue::Variable("num".to_string()),
        arms: vec![
            CaseArm {
                pattern: CasePattern::Literal("1".to_string()),
                guard: Some(ShellValue::Bool(true)),
                body: Box::new(ShellIR::Echo {
                    value: ShellValue::String("One with guard".to_string()),
                }),
            },
            CaseArm {
                pattern: CasePattern::Wildcard,
                guard: None,
                body: Box::new(ShellIR::Echo {
                    value: ShellValue::String("Default".to_string()),
                }),
            },
        ],
    };

    let result = emitter.emit(&ir).unwrap();
    assert!(result.contains("case"));
    assert!(result.contains("1)"));
    // Guards may be emitted as conditionals
    assert!(result.contains("if") || result.contains("true"));
}

#[test]
fn test_empty_main_body() {
    let config = Config::default();
    let emitter = PosixEmitter::new(config);

    let ir = ShellIR::Sequence(vec![]);

    let result = emitter.emit(&ir).unwrap();
    // Empty main should have a no-op (:)
    assert!(result.contains(":"));
}

#[test]
fn test_sequence_with_function_and_main() {
    let config = Config::default();
    let emitter = PosixEmitter::new(config);

    let ir = ShellIR::Sequence(vec![
        ShellIR::Function {
            name: "helper".to_string(),
            params: vec![],
            body: Box::new(ShellIR::Echo {
                value: ShellValue::String("helper called".to_string()),
            }),
        },
        ShellIR::Exec {
            cmd: Command {
                program: "helper".to_string(),
                args: vec![],
            },
            effects: EffectSet::pure(),
        },
    ]);

    let result = emitter.emit(&ir).unwrap();
    // Function should be at global scope
    assert!(result.contains("helper()"));
    // Main should call it
    assert!(result.contains("main() {"));
    assert!(result.contains("helper"));
}
