use crate::ir::{Command, Effect, EffectSet, ShellIR, ShellValue};
use crate::models::VerificationLevel;
use crate::verifier::verify;

#[test]
fn test_verify_basic() {
    let ir = ShellIR::Sequence(vec![ShellIR::Let {
        name: "x".to_string(),
        value: ShellValue::String("42".to_string()),
        effects: EffectSet::pure(),
    }]);

    let result = verify(&ir, VerificationLevel::Basic);
    assert!(result.is_ok());
}

#[test]
fn test_verify_strict() {
    let ir = ShellIR::Sequence(vec![
        ShellIR::Let {
            name: "x".to_string(),
            value: ShellValue::String("42".to_string()),
            effects: EffectSet::pure(),
        },
        ShellIR::Exec {
            cmd: Command::new("echo").arg(ShellValue::Variable("x".to_string())),
            effects: EffectSet::pure(),
        },
    ]);

    let result = verify(&ir, VerificationLevel::Strict);
    assert!(result.is_ok());
}

#[test]
fn test_verify_paranoid() {
    let ir = ShellIR::Sequence(vec![ShellIR::Let {
        name: "x".to_string(),
        value: ShellValue::String("safe_value".to_string()),
        effects: EffectSet::pure(),
    }]);

    let result = verify(&ir, VerificationLevel::Paranoid);
    assert!(result.is_ok());
}

#[test]
fn test_verify_command_injection() {
    // Test that command injection is detected
    let mut effects = EffectSet::pure();
    effects.add(Effect::ProcessExec);

    let ir = ShellIR::Exec {
        cmd: Command::new("eval").arg(ShellValue::Variable("user_input".to_string())),
        effects,
    };

    let result = verify(&ir, VerificationLevel::Basic);
    assert!(result.is_err());
}

#[test]
fn test_verify_nested_sequence() {
    let ir = ShellIR::Sequence(vec![
        ShellIR::Let {
            name: "x".to_string(),
            value: ShellValue::String("1".to_string()),
            effects: EffectSet::pure(),
        },
        ShellIR::If {
            test: ShellValue::Variable("x".to_string()),
            then_branch: Box::new(ShellIR::Exec {
                cmd: Command::new("echo").arg(ShellValue::String("x is 1".to_string())),
                effects: EffectSet::pure(),
            }),
            else_branch: None,
        },
    ]);

    let result = verify(&ir, VerificationLevel::Strict);
    assert!(result.is_ok());
}

#[test]
fn test_verify_concat_operations() {
    let ir = ShellIR::Let {
        name: "result".to_string(),
        value: ShellValue::Concat(vec![
            ShellValue::String("Hello".to_string()),
            ShellValue::String(" ".to_string()),
            ShellValue::String("World".to_string()),
        ]),
        effects: EffectSet::pure(),
    };

    let result = verify(&ir, VerificationLevel::Basic);
    assert!(result.is_ok());
}

#[test]
fn test_verify_command_substitution() {
    let mut effects = EffectSet::pure();
    effects.add(Effect::SystemModification);

    let ir = ShellIR::Let {
        name: "output".to_string(),
        value: ShellValue::CommandSubst(Command::new("date")),
        effects,
    };

    // Non-deterministic commands should fail in strict mode
    let result = verify(&ir, VerificationLevel::Strict);
    assert!(result.is_err());

    // But pass in basic mode
    let result = verify(&ir, VerificationLevel::Basic);
    assert!(result.is_ok());
}

#[test]
fn test_verify_exit_codes() {
    let ir = ShellIR::Sequence(vec![
        ShellIR::Exec {
            cmd: Command::new("test")
                .arg(ShellValue::String("-f".to_string()))
                .arg(ShellValue::String("file.txt".to_string())),
            effects: EffectSet::pure(),
        },
        ShellIR::Exit {
            code: 1,
            message: Some("File not found".to_string()),
        },
    ]);

    let result = verify(&ir, VerificationLevel::Basic);
    assert!(result.is_ok());
}

#[test]
fn test_verify_empty_ir() {
    let ir = ShellIR::Noop;
    let result = verify(&ir, VerificationLevel::Paranoid);
    assert!(result.is_ok());
}

#[test]
fn test_verify_none_level() {
    // With None verification level, everything should pass
    let ir = ShellIR::Exec {
        cmd: Command::new("eval").arg(ShellValue::Variable("dangerous".to_string())),
        effects: EffectSet::pure(),
    };

    let result = verify(&ir, VerificationLevel::None);
    assert!(result.is_ok());
}
