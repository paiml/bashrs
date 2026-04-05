use super::*;
use crate::ir::{Command, EffectSet, ShellIR, ShellValue};
use crate::models::Config;
use proptest::prelude::*;
use rstest::*;

#[test]
fn test_simple_let_emission() {
    let config = Config::default();
    let emitter = PosixEmitter::new();

    let ir = ShellIR::Let {
        name: "test_var".to_string(),
        value: ShellValue::String("hello world".to_string()),
        effects: EffectSet::pure(),
    };

    let result = emitter.emit(&ir).unwrap();
    // Updated: Variables are now mutable to support let-shadowing semantics
    assert!(result.contains("test_var='hello world'"));
    assert!(!result.contains("readonly"));
    assert!(result.contains("#!/bin/sh"));
    assert!(result.contains("set -euf"));
}

#[test]
fn test_command_emission() {
    let config = Config::default();
    let emitter = PosixEmitter::new();

    let cmd = Command {
        program: "echo".to_string(),
        args: vec![ShellValue::String("hello".to_string())],
    };

    let ir = ShellIR::Exec {
        cmd,
        effects: EffectSet::pure(),
    };

    let result = emitter.emit(&ir).unwrap();
    assert!(result.contains("echo hello"));
}

#[test]
fn test_if_statement_emission() {
    let config = Config::default();
    let emitter = PosixEmitter::new();

    let ir = ShellIR::If {
        test: ShellValue::Bool(true),
        then_branch: Box::new(ShellIR::Exec {
            cmd: Command {
                program: "echo".to_string(),
                args: vec![ShellValue::String("true branch".to_string())],
            },
            effects: EffectSet::pure(),
        }),
        else_branch: Some(Box::new(ShellIR::Exec {
            cmd: Command {
                program: "echo".to_string(),
                args: vec![ShellValue::String("false branch".to_string())],
            },
            effects: EffectSet::pure(),
        })),
    };

    let result = emitter.emit(&ir).unwrap();
    assert!(result.contains("if true; then"));
    assert!(result.contains("echo 'true branch'"));
    assert!(result.contains("else"));
    assert!(result.contains("echo 'false branch'"));
    assert!(result.contains("fi"));
}

#[test]
fn test_sequence_emission() {
    let config = Config::default();
    let emitter = PosixEmitter::new();

    let ir = ShellIR::Sequence(vec![
        ShellIR::Let {
            name: "greeting".to_string(),
            value: ShellValue::String("hello".to_string()),
            effects: EffectSet::pure(),
        },
        ShellIR::Exec {
            cmd: Command {
                program: "echo".to_string(),
                args: vec![ShellValue::Variable("greeting".to_string())],
            },
            effects: EffectSet::pure(),
        },
    ]);

    let result = emitter.emit(&ir).unwrap();
    // Updated: Variables are now mutable to support let-shadowing semantics
    assert!(result.contains("greeting='hello'"));
    assert!(!result.contains("readonly"));
    assert!(result.contains("echo \"$greeting\""));
}

#[test]
fn test_exit_statement_emission() {
    let config = Config::default();
    let emitter = PosixEmitter::new();

    let ir = ShellIR::Exit {
        code: 1,
        message: Some("Error occurred".to_string()),
    };

    let result = emitter.emit(&ir).unwrap();
    assert!(result.contains("echo 'Error occurred' >&2"));
    assert!(result.contains("exit 1"));
}

#[test]
fn test_shell_value_emission() {
    let config = Config::default();
    let emitter = PosixEmitter::new();

    // String value
    let string_val = ShellValue::String("hello world".to_string());
    let result = emitter.emit_shell_value(&string_val).unwrap();
    assert_eq!(result, "'hello world'");

    // Boolean values
    let bool_val = ShellValue::Bool(true);
    let result = emitter.emit_shell_value(&bool_val).unwrap();
    assert_eq!(result, "true");

    let bool_val = ShellValue::Bool(false);
    let result = emitter.emit_shell_value(&bool_val).unwrap();
    assert_eq!(result, "false");

    // Variable reference
    let var_val = ShellValue::Variable("test_var".to_string());
    let result = emitter.emit_shell_value(&var_val).unwrap();
    assert_eq!(result, "\"$test_var\"");
}

#[test]
fn test_concatenation_emission() {
    let config = Config::default();
    let emitter = PosixEmitter::new();

    let concat_val = ShellValue::Concat(vec![
        ShellValue::String("Hello ".to_string()),
        ShellValue::Variable("name".to_string()),
        ShellValue::String("!".to_string()),
    ]);

    let result = emitter.emit_shell_value(&concat_val).unwrap();
    assert_eq!(result, "\"Hello ${name}!\"");
}

#[test]
fn test_command_substitution_emission() {
    let config = Config::default();
    let emitter = PosixEmitter::new();

    let cmd_subst = ShellValue::CommandSubst(Command {
        program: "date".to_string(),
        args: vec![ShellValue::String("+%Y".to_string())],
    });

    let result = emitter.emit_shell_value(&cmd_subst).unwrap();
    assert_eq!(result, "\"$(date '+%Y')\"");
}

#[test]
fn test_noop_emission() {
    let config = Config::default();
    let emitter = PosixEmitter::new();

    let ir = ShellIR::Noop;
    let result = emitter.emit(&ir).unwrap();
    // Updated: Noop now emits ':' for valid POSIX syntax instead of comment
    assert!(result.contains(":"));
}

#[test]

include!("tests_incl2.rs");
