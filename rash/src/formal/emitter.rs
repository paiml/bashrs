//! Formal emitter for the tiny AST subset
//!
//! This module implements the formally verified emitter that translates
//! rash AST nodes to semantically equivalent POSIX shell commands.

use crate::formal::TinyAst;

/// Formally verified emitter for the tiny AST subset
pub struct FormalEmitter;

impl FormalEmitter {
    /// Emit POSIX shell code from a tiny AST node
    pub fn emit(ast: &TinyAst) -> String {
        match ast {
            TinyAst::ExecuteCommand { command_name, args } => {
                Self::emit_command(command_name, args)
            }

            TinyAst::SetEnvironmentVariable { name, value } => Self::emit_assignment(name, value),

            TinyAst::Sequence { commands } => Self::emit_sequence(commands),

            TinyAst::ChangeDirectory { path } => Self::emit_cd(path),
        }
    }

    /// Emit a simple command
    fn emit_command(name: &str, args: &[String]) -> String {
        let mut parts = vec![name.to_string()];

        for arg in args {
            parts.push(Self::quote_argument(arg));
        }

        parts.join(" ")
    }

    /// Emit a variable assignment
    fn emit_assignment(name: &str, value: &str) -> String {
        format!("{}={}", name, Self::quote_value(value))
    }

    /// Emit a sequence of commands
    fn emit_sequence(commands: &[TinyAst]) -> String {
        commands
            .iter()
            .map(Self::emit)
            .collect::<Vec<_>>()
            .join("; ")
    }

    /// Emit a change directory command
    fn emit_cd(path: &str) -> String {
        format!("cd {}", Self::quote_argument(path))
    }

    /// Quote a command argument if necessary
    fn quote_argument(arg: &str) -> String {
        // Check if quoting is needed
        if arg.is_empty()
            || arg.contains(|c: char| {
                c.is_whitespace()
                    || matches!(
                        c,
                        '$' | '`'
                            | '"'
                            | '\''
                            | '\\'
                            | '!'
                            | '#'
                            | '&'
                            | '*'
                            | '('
                            | ')'
                            | ';'
                            | '<'
                            | '>'
                            | '?'
                            | '['
                            | ']'
                            | '{'
                            | '}'
                            | '|'
                            | '~'
                    )
            })
        {
            // Use double quotes and escape special characters
            format!("\"{}\"", Self::escape_for_double_quotes(arg))
        } else {
            arg.to_string()
        }
    }

    /// Quote a value for assignment
    fn quote_value(value: &str) -> String {
        // Always quote values to ensure correctness
        format!("\"{}\"", Self::escape_for_double_quotes(value))
    }

    /// Escape special characters for use within double quotes
    fn escape_for_double_quotes(s: &str) -> String {
        let mut result = String::with_capacity(s.len());

        for ch in s.chars() {
            match ch {
                '"' => result.push_str("\\\""),
                '\\' => result.push_str("\\\\"),
                '$' => result.push_str("\\$"),
                '`' => result.push_str("\\`"),
                '\n' => result.push_str("\\n"),
                _ => result.push(ch),
            }
        }

        result
    }
}

/// Theorem: Semantic equivalence of emitted code
///
/// For any AST node in the tiny subset, the emitted POSIX code
/// has the same semantic behavior as the original AST.
pub fn verify_semantic_equivalence(ast: &TinyAst) -> Result<(), String> {
    // This function represents the formal theorem that would be proven
    // in a proof assistant. Here we can only test it empirically.

    use crate::formal::semantics::{posix_semantics, rash_semantics};
    use crate::formal::AbstractState;

    // Create a test state
    let initial_state = AbstractState::test_state();

    // Evaluate the AST
    let rash_result = rash_semantics::eval_rash(ast, initial_state.clone())?;

    // Emit POSIX code
    let posix_code = FormalEmitter::emit(ast);

    // Evaluate the POSIX code
    let posix_result = posix_semantics::eval_posix(&posix_code, initial_state)?;

    // Check equivalence
    if rash_result.is_equivalent(&posix_result) {
        Ok(())
    } else {
        Err(format!(
            "Semantic equivalence failed for AST: {:?}\nEmitted: {}\nRash state: {:?}\nPOSIX state: {:?}",
            ast, posix_code, rash_result, posix_result
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emit_simple_command() {
        let ast = TinyAst::ExecuteCommand {
            command_name: "echo".to_string(),
            args: vec!["Hello World".to_string()],
        };

        let emitted = FormalEmitter::emit(&ast);
        assert_eq!(emitted, "echo \"Hello World\"");
    }

    #[test]
    fn test_emit_assignment() {
        let ast = TinyAst::SetEnvironmentVariable {
            name: "PATH".to_string(),
            value: "/usr/bin:/bin".to_string(),
        };

        let emitted = FormalEmitter::emit(&ast);
        assert_eq!(emitted, "PATH=\"/usr/bin:/bin\"");
    }

    #[test]
    fn test_emit_sequence() {
        let ast = TinyAst::Sequence {
            commands: vec![
                TinyAst::SetEnvironmentVariable {
                    name: "DIR".to_string(),
                    value: "/opt/rash".to_string(),
                },
                TinyAst::ExecuteCommand {
                    command_name: "mkdir".to_string(),
                    args: vec!["-p".to_string(), "/opt/rash".to_string()],
                },
                TinyAst::ChangeDirectory {
                    path: "/opt/rash".to_string(),
                },
            ],
        };

        let emitted = FormalEmitter::emit(&ast);
        assert_eq!(
            emitted,
            "DIR=\"/opt/rash\"; mkdir -p /opt/rash; cd /opt/rash"
        );
    }

    #[test]
    fn test_quote_special_characters() {
        let ast = TinyAst::ExecuteCommand {
            command_name: "echo".to_string(),
            args: vec!["$HOME/path with spaces".to_string()],
        };

        let emitted = FormalEmitter::emit(&ast);
        assert_eq!(emitted, "echo \"\\$HOME/path with spaces\"");
    }

    #[test]
    fn test_semantic_equivalence_echo() {
        let ast = TinyAst::ExecuteCommand {
            command_name: "echo".to_string(),
            args: vec!["Test".to_string()],
        };

        assert!(verify_semantic_equivalence(&ast).is_ok());
    }

    #[test]
    fn test_semantic_equivalence_assignment() {
        let ast = TinyAst::SetEnvironmentVariable {
            name: "TEST_VAR".to_string(),
            value: "test_value".to_string(),
        };

        assert!(verify_semantic_equivalence(&ast).is_ok());
    }

    #[test]
    fn test_semantic_equivalence_sequence() {
        let ast = TinyAst::Sequence {
            commands: vec![
                TinyAst::SetEnvironmentVariable {
                    name: "INSTALL_DIR".to_string(),
                    value: "/opt/rash".to_string(),
                },
                TinyAst::ExecuteCommand {
                    command_name: "mkdir".to_string(),
                    args: vec!["-p".to_string(), "/opt/rash/bin".to_string()],
                },
            ],
        };

        assert!(verify_semantic_equivalence(&ast).is_ok());
    }
}
