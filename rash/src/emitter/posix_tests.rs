use super::*;
use crate::ir::{Command, ShellIR, ShellValue};
use crate::models::Config;

#[test]
fn test_emit_simple_let() {
    let config = Config::default();
    let emitter = PosixEmitter::new();

    let ir = ShellIR::Let {
        name: "test_var".to_string(),
        value: ShellValue::String("hello world".to_string()),
        effects: Default::default(),
    };

    let result = emitter.emit(&ir).unwrap();
    // Updated: Variables are now mutable to support let-shadowing semantics
    assert!(result.contains("test_var='hello world'"));
    assert!(!result.contains("readonly"));
}

#[test]
fn test_emit_command() {
    let config = Config::default();
    let emitter = PosixEmitter::new();

    let cmd = Command {
        program: "echo".to_string(),
        args: vec![ShellValue::String("hello".to_string())],
    };

    let ir = ShellIR::Exec {
        cmd,
        effects: Default::default(),
    };

    let result = emitter.emit(&ir).unwrap();
    assert!(result.contains("echo hello"));
}

#[test]
fn test_emit_if_statement() {
    let config = Config::default();
    let emitter = PosixEmitter::new();

    let ir = ShellIR::If {
        test: ShellValue::Bool(true),
        then_branch: Box::new(ShellIR::Exec {
            cmd: Command {
                program: "echo".to_string(),
                args: vec![ShellValue::String("true branch".to_string())],
            },
            effects: Default::default(),
        }),
        else_branch: None,
    };

    let result = emitter.emit(&ir).unwrap();
    assert!(result.contains("if true; then"));
    assert!(result.contains("echo 'true branch'"));
    assert!(result.contains("fi"));
}

#[test]
fn test_POSIX_COV_001_write_footer() {
    // write_footer produces closing brace, cleanup trap, and main call
    let config = Config::default();
    let emitter = PosixEmitter::new();
    let ir = ShellIR::Noop;
    let result = emitter.emit(&ir).expect("emit should succeed");
    assert!(result.contains("main() {"));
    assert!(result.contains("}"));
    assert!(result.contains("trap"));
    assert!(result.contains("main \"$@\""));
}

#[test]
fn test_POSIX_COV_002_write_println_function() {
    // rash_println used in IR triggers runtime function emission
    let config = Config::default();
    let emitter = PosixEmitter::new();
    let ir = ShellIR::Exec {
        cmd: Command {
            program: "rash_println".to_string(),
            args: vec![ShellValue::String("hello".to_string())],
        },
        effects: Default::default(),
    };
    let result = emitter.emit(&ir).expect("emit should succeed");
    assert!(result.contains("rash_println()"));
    assert!(result.contains("printf '%s\\n' \"$1\""));
}

#[test]
fn test_POSIX_COV_003_write_eprintln_function() {
    let config = Config::default();
    let emitter = PosixEmitter::new();
    let ir = ShellIR::Exec {
        cmd: Command {
            program: "rash_eprintln".to_string(),
            args: vec![ShellValue::String("error".to_string())],
        },
        effects: Default::default(),
    };
    let result = emitter.emit(&ir).expect("emit should succeed");
    assert!(result.contains("rash_eprintln()"));
    assert!(result.contains(">&2"));
}

#[test]
fn test_POSIX_COV_004_write_fs_read_file_function() {
    let config = Config::default();
    let emitter = PosixEmitter::new();
    let ir = ShellIR::Exec {
        cmd: Command {
            program: "rash_fs_read_file".to_string(),
            args: vec![ShellValue::String("/tmp/test".to_string())],
        },
        effects: Default::default(),
    };
    let result = emitter.emit(&ir).expect("emit should succeed");
    assert!(result.contains("rash_fs_read_file()"));
    assert!(result.contains("cat \"$path\""));
}

#[test]
fn test_POSIX_COV_005_write_fs_exists_function() {
    let config = Config::default();
    let emitter = PosixEmitter::new();
    let ir = ShellIR::Exec {
        cmd: Command {
            program: "rash_fs_exists".to_string(),
            args: vec![ShellValue::String("/tmp".to_string())],
        },
        effects: Default::default(),
    };
    let result = emitter.emit(&ir).expect("emit should succeed");
    assert!(result.contains("rash_fs_exists()"));
}

#[test]
fn test_POSIX_COV_006_emit_comparison() {
    use crate::ir::shell_ir::ComparisonOp;
    let config = Config::default();
    let emitter = PosixEmitter::new();
    // Comparison in an if test
    let ir = ShellIR::If {
        test: ShellValue::Comparison {
            op: ComparisonOp::NumEq,
            left: Box::new(ShellValue::Variable("x".to_string())),
            right: Box::new(ShellValue::String("5".to_string())),
        },
        then_branch: Box::new(ShellIR::Noop),
        else_branch: None,
    };
    let result = emitter.emit(&ir).expect("emit should succeed");
    assert!(result.contains("-eq"));
}

#[test]
fn test_POSIX_COV_007_emit_arithmetic() {
    use crate::ir::shell_ir::ArithmeticOp;
    let config = Config::default();
    let emitter = PosixEmitter::new();
    let ir = ShellIR::Let {
        name: "result".to_string(),
        value: ShellValue::Arithmetic {
            op: ArithmeticOp::Add,
            left: Box::new(ShellValue::Variable("a".to_string())),
            right: Box::new(ShellValue::String("1".to_string())),
        },
        effects: Default::default(),
    };
    let result = emitter.emit(&ir).expect("emit should succeed");
    assert!(result.contains("$(("));
    assert!(result.contains("+"));
}

#[test]
fn test_POSIX_COV_008_emit_arithmetic_all_ops() {
    use crate::ir::shell_ir::ArithmeticOp;
    let config = Config::default();
    let emitter = PosixEmitter::new();
    for (op, sym) in [
        (ArithmeticOp::Sub, "-"),
        (ArithmeticOp::Mul, "*"),
        (ArithmeticOp::Div, "/"),
        (ArithmeticOp::Mod, "%"),
    ] {
        let ir = ShellIR::Let {
            name: "r".to_string(),
            value: ShellValue::Arithmetic {
                op,
                left: Box::new(ShellValue::String("10".to_string())),
                right: Box::new(ShellValue::String("3".to_string())),
            },
            effects: Default::default(),
        };
        let result = emitter.emit(&ir).expect("emit should succeed");
        assert!(result.contains(sym), "expected '{sym}' in output: {result}");
    }
}

#[test]
fn test_POSIX_COV_009_emit_arithmetic_operand_nested() {
    use crate::ir::shell_ir::ArithmeticOp;
    let config = Config::default();
    let emitter = PosixEmitter::new();
    // Nested arithmetic: (a + 1) * 2
    let ir = ShellIR::Let {
        name: "r".to_string(),
        value: ShellValue::Arithmetic {
            op: ArithmeticOp::Mul,
            left: Box::new(ShellValue::Arithmetic {
                op: ArithmeticOp::Add,
                left: Box::new(ShellValue::Variable("a".to_string())),
                right: Box::new(ShellValue::String("1".to_string())),
            }),
            right: Box::new(ShellValue::String("2".to_string())),
        },
        effects: Default::default(),
    };
    let result = emitter.emit(&ir).expect("emit should succeed");
    // Should contain nested expression
    assert!(result.contains("("));
    assert!(result.contains("+"));
    assert!(result.contains("*"));
}

#[test]
fn test_POSIX_COV_010_emit_arithmetic_operand_command_subst() {
    use crate::ir::shell_ir::ArithmeticOp;
    let config = Config::default();
    let emitter = PosixEmitter::new();
    // Arithmetic with command substitution: $(wc -l) + 1
    let ir = ShellIR::Let {
        name: "r".to_string(),
        value: ShellValue::Arithmetic {
            op: ArithmeticOp::Add,
            left: Box::new(ShellValue::CommandSubst(Command {
                program: "wc".to_string(),
                args: vec![ShellValue::String("-l".to_string())],
            })),
            right: Box::new(ShellValue::String("1".to_string())),
        },
        effects: Default::default(),
    };
    let result = emitter.emit(&ir).expect("emit should succeed");
    assert!(result.contains("$(wc"));
}

#[test]
fn test_POSIX_COV_011_emit_arithmetic_operand_unsupported() {
    use crate::ir::shell_ir::ArithmeticOp;
    let config = Config::default();
    let emitter = PosixEmitter::new();
    // Bool in arithmetic context should error
    let ir = ShellIR::Let {
        name: "r".to_string(),
        value: ShellValue::Arithmetic {
            op: ArithmeticOp::Add,
            left: Box::new(ShellValue::Bool(true)),
            right: Box::new(ShellValue::String("1".to_string())),
        },
        effects: Default::default(),
    };
    let result = emitter.emit(&ir);
    assert!(result.is_err());
}

#[test]
fn test_POSIX_COV_012_emit_shell_value_arg_with_default() {
    let config = Config::default();
    let emitter = PosixEmitter::new();
    let ir = ShellIR::Exec {
        cmd: Command {
            program: "echo".to_string(),
            args: vec![ShellValue::ArgWithDefault {
                position: 1,
                default: "fallback".to_string(),
            }],
        },
        effects: Default::default(),
    };
    let result = emitter.emit(&ir).expect("emit should succeed");
    assert!(result.contains("${1:-fallback}"));
}

#[test]
fn test_POSIX_COV_013_while_logical_and_condition() {
    use crate::ir::shell_ir::ComparisonOp;
    let config = Config::default();
    let emitter = PosixEmitter::new();
    let ir = ShellIR::While {
        condition: ShellValue::LogicalAnd {
            left: Box::new(ShellValue::Comparison {
                op: ComparisonOp::Lt,
                left: Box::new(ShellValue::Variable("i".to_string())),
                right: Box::new(ShellValue::String("10".to_string())),
            }),
            right: Box::new(ShellValue::Bool(true)),
        },
        body: Box::new(ShellIR::Noop),
    };
    let result = emitter.emit(&ir).expect("emit should succeed");
    assert!(result.contains("&&"));
    assert!(result.contains("while"));
    assert!(result.contains("done"));
}

#[test]
fn test_POSIX_COV_014_while_logical_or_condition() {
    let config = Config::default();
    let emitter = PosixEmitter::new();
    let ir = ShellIR::While {
        condition: ShellValue::LogicalOr {
            left: Box::new(ShellValue::Bool(true)),
            right: Box::new(ShellValue::Bool(false)),
        },
        body: Box::new(ShellIR::Noop),
    };
    let result = emitter.emit(&ir).expect("emit should succeed");
    assert!(result.contains("||"));
}

#[test]
fn test_POSIX_COV_015_while_logical_not_condition() {
    let config = Config::default();
    let emitter = PosixEmitter::new();
    let ir = ShellIR::While {
        condition: ShellValue::LogicalNot {
            operand: Box::new(ShellValue::Bool(false)),
        },
        body: Box::new(ShellIR::Noop),
    };
    let result = emitter.emit(&ir).expect("emit should succeed");
    assert!(result.contains("! false"));
}

#[test]
fn test_POSIX_COV_016_while_general_expression() {
    let config = Config::default();
    let emitter = PosixEmitter::new();
    let ir = ShellIR::While {
        condition: ShellValue::Variable("running".to_string()),
        body: Box::new(ShellIR::Noop),
    };
    let result = emitter.emit(&ir).expect("emit should succeed");
    assert!(result.contains("[ "));
}

#[test]

include!("posix_tests_tests_POSIX.rs");
