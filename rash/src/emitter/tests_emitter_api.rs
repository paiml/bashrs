
#[test]
fn test_emit_public_api() {
    let config = Config::default();

    let ir = ShellIR::Let {
        name: "test".to_string(),
        value: ShellValue::String("value".to_string()),
        effects: EffectSet::pure(),
    };

    // Test the public emit function
    let result = emit(&ir).unwrap();
    // Updated: Variables are now mutable to support let-shadowing semantics
    assert!(result.contains("test='value'"));
    assert!(!result.contains("readonly"));
}

#[test]
fn test_different_shell_dialects() {
    let mut config = Config::default();

    let ir = ShellIR::Noop;

    // Test POSIX (default)
    config.target = crate::models::ShellDialect::Posix;
    let result = emit(&ir).unwrap();
    assert!(result.contains("#!/bin/sh"));

    // Test Bash (should still emit POSIX for now)
    config.target = crate::models::ShellDialect::Bash;
    let result = emit(&ir).unwrap();
    assert!(result.contains("#!/bin/sh"));
}

#[test]
fn test_indentation_consistency() {
    let config = Config::default();
    let emitter = PosixEmitter::new();

    let ir = ShellIR::If {
        test: ShellValue::Bool(true),
        then_branch: Box::new(ShellIR::If {
            test: ShellValue::Bool(false),
            then_branch: Box::new(ShellIR::Let {
                name: "nested".to_string(),
                value: ShellValue::String("deep".to_string()),
                effects: EffectSet::pure(),
            }),
            else_branch: None,
        }),
        else_branch: None,
    };

    let result = emitter.emit(&ir).unwrap();

    // Check that nested structures have proper indentation
    let lines: Vec<&str> = result.lines().collect();
    let main_function_start = lines
        .iter()
        .position(|&line| line.contains("main() {"))
        .unwrap();

    // Lines inside main() should be indented
    for line in &lines[main_function_start + 1..] {
        if line.trim().is_empty() || line.starts_with('#') || line.starts_with('}') {
            continue;
        }
        if line.contains("trap") || line.contains("main \"$@\"") {
            break;
        }
        // Should start with spaces (indentation)
        assert!(
            line.starts_with("    ") || line.starts_with("        "),
            "Line not properly indented: '{line}'"
        );
    }
}

// ============= Sprint 27a: Environment Variables Support - RED PHASE =============

/// RED TEST: env() should emit ${VAR} syntax
/// Tests that ShellValue::EnvVar without default generates "${VAR}" in shell
#[test]
fn test_env_emits_dollar_brace_syntax() {
    use crate::models::Config;

    let ir = crate::ir::shell_ir::ShellIR::Let {
        name: "home".to_string(),
        value: ShellValue::EnvVar {
            name: "HOME".to_string(),
            default: None,
        },
        effects: crate::ir::effects::EffectSet::pure(),
    };

    let config = Config::default();
    let output = super::emit(&ir).unwrap();

    // RED: This will fail until we implement EnvVar emission
    assert!(
        output.contains("\"${HOME}\""),
        "env() should emit ${{VAR}} with quotes, got: {}",
        output
    );
    assert!(
        output.contains("home=\"${HOME}\""),
        "Should assign quoted env var to variable, got: {}",
        output
    );
}

/// RED TEST: env_var_or() should emit ${VAR:-default} syntax
/// Tests that ShellValue::EnvVar with default generates "${VAR:-default}"
#[test]
fn test_env_var_or_emits_with_default() {
    use crate::models::Config;

    let ir = crate::ir::shell_ir::ShellIR::Let {
        name: "prefix".to_string(),
        value: ShellValue::EnvVar {
            name: "PREFIX".to_string(),
            default: Some("/usr/local".to_string()),
        },
        effects: crate::ir::effects::EffectSet::pure(),
    };

    let config = Config::default();
    let output = super::emit(&ir).unwrap();

    // RED: This will fail until we implement EnvVar with default emission
    assert!(
        output.contains("\"${PREFIX:-/usr/local}\""),
        "env_var_or() should emit ${{VAR:-default}} with quotes, got: {}",
        output
    );
    assert!(
        output.contains("prefix=\"${PREFIX:-/usr/local}\""),
        "Should assign quoted env var with default, got: {}",
        output
    );
}

/// RED TEST: Environment variables must be quoted for safety
/// Tests that all env var expansions include proper quoting
#[test]
fn test_env_var_quoted_for_safety() {
    use crate::models::Config;

    let ir = crate::ir::shell_ir::ShellIR::Sequence(vec![
        crate::ir::shell_ir::ShellIR::Let {
            name: "user".to_string(),
            value: ShellValue::EnvVar {
                name: "USER".to_string(),
                default: None,
            },
            effects: crate::ir::effects::EffectSet::pure(),
        },
        crate::ir::shell_ir::ShellIR::Let {
            name: "home".to_string(),
            value: ShellValue::EnvVar {
                name: "HOME".to_string(),
                default: Some("/tmp".to_string()),
            },
            effects: crate::ir::effects::EffectSet::pure(),
        },
    ]);

    let config = Config::default();
    let output = super::emit(&ir).unwrap();

    // RED: Must have quotes around ${{VAR}} for safety
    assert!(
        !output.contains("=$USER") && !output.contains("= $USER"),
        "Env vars must be quoted, found unquoted $USER: {}",
        output
    );
    assert!(
        !output.contains("=$HOME:"),
        "Env vars with defaults must be quoted, found unquoted $HOME:-...: {}",
        output
    );

    // Should have quoted versions
    assert!(
        output.contains("\"${USER}\"") || output.contains("\"$USER\""),
        "Should have quoted $USER: {}",
        output
    );
    assert!(
        output.contains("\"${HOME:-/tmp}\"") || output.contains("\"$HOME:-/tmp\""),
        "Should have quoted $HOME:-/tmp: {}",
        output
    );
}

/// RED TEST: Complex default values must be properly escaped
/// Tests that default values with special characters are handled safely
#[test]
fn test_env_complex_default_value() {
    use crate::models::Config;

    let ir = crate::ir::shell_ir::ShellIR::Let {
        name: "message".to_string(),
        value: ShellValue::EnvVar {
            name: "MESSAGE".to_string(),
            default: Some("hello world".to_string()), // Space in default
        },
        effects: crate::ir::effects::EffectSet::pure(),
    };

    let config = Config::default();
    let output = super::emit(&ir).unwrap();

    // RED: Default values with spaces must work correctly
    assert!(
        output.contains("${MESSAGE:-hello world}")
            || output.contains("${MESSAGE:-\"hello world\"}"),
        "Should handle default with spaces, got: {}",
        output
    );
}

// ============= Sprint 27b: Command-Line Arguments Support - RED PHASE =============

/// RED TEST: arg(1) should emit "$1" syntax
/// Tests that ShellValue::Arg { position: Some(1) } generates "$1" in shell
#[test]
fn test_arg_emits_positional_syntax() {
    use crate::models::Config;

    let ir = crate::ir::shell_ir::ShellIR::Let {
        name: "first".to_string(),
        value: ShellValue::Arg { position: Some(1) },
        effects: crate::ir::effects::EffectSet::pure(),
    };

    let config = Config::default();
    let output = super::emit(&ir).unwrap();

    // RED: This will fail until we implement Arg emission
    assert!(
        output.contains("\"$1\""),
        "arg(1) should emit $1 with quotes, got: {}",
        output
    );
    assert!(
        output.contains("first=\"$1\""),
        "Should assign quoted positional arg to variable, got: {}",
        output
    );
}

