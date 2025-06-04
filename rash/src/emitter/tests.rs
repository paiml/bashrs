use super::*;
use crate::ir::{ShellIR, ShellValue, Command, EffectSet};
use crate::models::Config;
use proptest::prelude::*;
use rstest::*;

#[test]
fn test_simple_let_emission() {
    let config = Config::default();
    let emitter = PosixEmitter::new(config);
    
    let ir = ShellIR::Let {
        name: "test_var".to_string(),
        value: ShellValue::String("hello world".to_string()),
        effects: EffectSet::pure(),
    };
    
    let result = emitter.emit(&ir).unwrap();
    assert!(result.contains("readonly test_var='hello world'"));
    assert!(result.contains("#!/bin/sh"));
    assert!(result.contains("set -euf"));
}

#[test]
fn test_command_emission() {
    let config = Config::default();
    let emitter = PosixEmitter::new(config);
    
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
    let emitter = PosixEmitter::new(config);
    
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
    let emitter = PosixEmitter::new(config);
    
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
    assert!(result.contains("readonly greeting=hello"));
    assert!(result.contains("echo \"$greeting\""));
}

#[test]
fn test_exit_statement_emission() {
    let config = Config::default();
    let emitter = PosixEmitter::new(config);
    
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
    let emitter = PosixEmitter::new(config);
    
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
    let emitter = PosixEmitter::new(config);
    
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
    let emitter = PosixEmitter::new(config);
    
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
    let emitter = PosixEmitter::new(config);
    
    let ir = ShellIR::Noop;
    let result = emitter.emit(&ir).unwrap();
    assert!(result.contains("# noop"));
}

#[test]
fn test_header_and_footer_structure() {
    let config = Config::default();
    let emitter = PosixEmitter::new(config);
    
    let ir = ShellIR::Noop;
    let result = emitter.emit(&ir).unwrap();
    
    // Check header
    assert!(result.starts_with("#!/bin/sh"));
    assert!(result.contains("# Generated by Rash"));
    assert!(result.contains("set -euf"));
    assert!(result.contains("IFS=$'\\n\\t'"));
    assert!(result.contains("export LC_ALL=C"));
    
    // Check runtime functions
    assert!(result.contains("rash_require()"));
    assert!(result.contains("rash_download_verified()"));
    
    // Check footer
    assert!(result.contains("main() {"));
    assert!(result.contains("trap 'rm -rf"));
    assert!(result.contains("main \"$@\""));
}

#[test]
fn test_runtime_functions_included() {
    let config = Config::default();
    let emitter = PosixEmitter::new(config);
    
    let ir = ShellIR::Noop;
    let result = emitter.emit(&ir).unwrap();
    
    // Verify essential runtime functions are present
    assert!(result.contains("rash_require() {"));
    assert!(result.contains("rash_download_verified() {"));
    
    // Verify they contain expected functionality
    assert!(result.contains("curl -fsSL"));
    assert!(result.contains("sha256sum"));
    assert!(result.contains("wget"));
}

#[test]
fn test_test_expression_emission() {
    let config = Config::default();
    let emitter = PosixEmitter::new(config);
    
    // Boolean true
    let result = emitter.emit_test_expression(&ShellValue::Bool(true)).unwrap();
    assert_eq!(result, "true");
    
    // Boolean false
    let result = emitter.emit_test_expression(&ShellValue::Bool(false)).unwrap();
    assert_eq!(result, "false");
    
    // Variable test
    let result = emitter.emit_test_expression(&ShellValue::Variable("var".to_string())).unwrap();
    assert_eq!(result, "test -n \"$var\"");
    
    // String literal
    let result = emitter.emit_test_expression(&ShellValue::String("true".to_string())).unwrap();
    assert_eq!(result, "true");
    
    let result = emitter.emit_test_expression(&ShellValue::String("false".to_string())).unwrap();
    assert_eq!(result, "false");
}

// Test escape module functionality
#[test]
fn test_string_escaping() {
    use super::escape::*;
    
    // Simple strings don't need escaping
    assert_eq!(escape_shell_string("hello"), "hello");
    assert_eq!(escape_shell_string("simple123"), "simple123");
    
    // Strings with spaces need quotes
    assert_eq!(escape_shell_string("hello world"), "'hello world'");
    
    // Empty strings
    assert_eq!(escape_shell_string(""), "''");
    
    // Strings with single quotes
    assert_eq!(escape_shell_string("don't"), "'don'\"'\"'t'");
}

#[test]
fn test_variable_name_escaping() {
    use super::escape::*;
    
    // Valid identifiers
    assert_eq!(escape_variable_name("valid_name"), "valid_name");
    assert_eq!(escape_variable_name("_underscore"), "_underscore");
    assert_eq!(escape_variable_name("name123"), "name123");
    
    // Invalid characters converted to underscores
    assert_eq!(escape_variable_name("invalid-name"), "invalid_name");
    assert_eq!(escape_variable_name("123invalid"), "_23invalid");
    assert_eq!(escape_variable_name("my.var"), "my_var");
}

#[test]
fn test_command_name_escaping() {
    use super::escape::*;
    
    // Simple commands
    assert_eq!(escape_command_name("ls"), "ls");
    assert_eq!(escape_command_name("/bin/ls"), "/bin/ls");
    assert_eq!(escape_command_name("my-tool"), "my-tool");
    
    // Commands with spaces need quoting
    assert_eq!(escape_command_name("my command"), "'my command'");
}

// Property-based tests
proptest! {
    #[test]
    fn test_string_escaping_preserves_content(s in ".*") {
        use super::escape::*;
        
        let escaped = escape_shell_string(&s);
        
        // Escaped strings should either be the original (if safe) or quoted
        if s.chars().all(|c| c.is_alphanumeric() || "_.-/+=:@".contains(c)) && !s.is_empty() {
            // Safe strings might be unquoted
            assert!(escaped == s || escaped == format!("'{}'", s));
        } else {
            // Unsafe strings should be quoted
            assert!(escaped.starts_with('\'') && escaped.ends_with('\'') || escaped == "''");
        }
    }

    #[test]
    fn test_variable_name_escaping_produces_valid_identifiers(name in "[a-zA-Z_][a-zA-Z0-9_-]*") {
        use super::escape::*;
        
        let escaped = escape_variable_name(&name);
        
        // Should start with letter or underscore
        assert!(escaped.chars().next().unwrap().is_alphabetic() || escaped.starts_with('_'));
        
        // Should only contain valid characters
        assert!(escaped.chars().all(|c| c.is_alphanumeric() || c == '_'));
    }
}

#[rstest]
#[case(ShellValue::String("test".to_string()), "test")]
#[case(ShellValue::Bool(true), "true")]
#[case(ShellValue::Bool(false), "false")]
#[case(ShellValue::Variable("var".to_string()), "\"$var\"")]
fn test_shell_value_emission_cases(
    #[case] value: ShellValue,
    #[case] expected: &str
) {
    let config = Config::default();
    let emitter = PosixEmitter::new(config);
    
    let result = emitter.emit_shell_value(&value).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_complex_nested_emission() {
    let config = Config::default();
    let emitter = PosixEmitter::new(config);
    
    let ir = ShellIR::Sequence(vec![
        ShellIR::Let {
            name: "prefix".to_string(),
            value: ShellValue::String("/usr/local".to_string()),
            effects: EffectSet::pure(),
        },
        ShellIR::If {
            test: ShellValue::Variable("install_mode".to_string()),
            then_branch: Box::new(ShellIR::Sequence(vec![
                ShellIR::Exec {
                    cmd: Command {
                        program: "mkdir".to_string(),
                        args: vec![ShellValue::Variable("prefix".to_string())],
                    },
                    effects: EffectSet::default(),
                },
                ShellIR::Exec {
                    cmd: Command {
                        program: "echo".to_string(),
                        args: vec![ShellValue::Concat(vec![
                            ShellValue::String("Installing to ".to_string()),
                            ShellValue::Variable("prefix".to_string()),
                        ])],
                    },
                    effects: EffectSet::pure(),
                },
            ])),
            else_branch: Some(Box::new(ShellIR::Exit {
                code: 1,
                message: Some("Installation cancelled".to_string()),
            })),
        },
    ]);
    
    let result = emitter.emit(&ir).unwrap();
    
    // Verify structure
    assert!(result.contains("readonly prefix=/usr/local"));
    assert!(result.contains("if test -n \"$install_mode\"; then"));
    assert!(result.contains("mkdir \"$prefix\""));
    assert!(result.contains("echo \"Installing to ${prefix}\""));
    assert!(result.contains("else"));
    assert!(result.contains("echo 'Installation cancelled' >&2"));
    assert!(result.contains("exit 1"));
    assert!(result.contains("fi"));
}

#[test]
fn test_emit_public_api() {
    let config = Config::default();
    
    let ir = ShellIR::Let {
        name: "test".to_string(),
        value: ShellValue::String("value".to_string()),
        effects: EffectSet::pure(),
    };
    
    // Test the public emit function
    let result = emit(&ir, &config).unwrap();
    assert!(result.contains("readonly test=value"));
}

#[test]
fn test_different_shell_dialects() {
    let mut config = Config::default();
    
    let ir = ShellIR::Noop;
    
    // Test POSIX (default)
    config.target = crate::models::ShellDialect::Posix;
    let result = emit(&ir, &config).unwrap();
    assert!(result.contains("#!/bin/sh"));
    
    // Test Bash (should still emit POSIX for now)
    config.target = crate::models::ShellDialect::Bash;
    let result = emit(&ir, &config).unwrap();
    assert!(result.contains("#!/bin/sh"));
}

#[test] 
fn test_indentation_consistency() {
    let config = Config::default();
    let emitter = PosixEmitter::new(config);
    
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
    let main_function_start = lines.iter().position(|&line| line.contains("main() {")).unwrap();
    
    // Lines inside main() should be indented
    for line in &lines[main_function_start + 1..] {
        if line.trim().is_empty() || line.starts_with('#') || line.starts_with('}') {
            continue;
        }
        if line.contains("trap") || line.contains("main \"$@\"") {
            break;
        }
        // Should start with spaces (indentation)
        assert!(line.starts_with("    ") || line.starts_with("        "), 
                "Line not properly indented: '{}'", line);
    }
}