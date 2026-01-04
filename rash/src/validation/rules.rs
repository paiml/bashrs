use super::{Fix, Severity, Validate, ValidationError};
use crate::emitter::escape::shell_escape;
use crate::ir::ShellExpression;
use crate::models::error::RashResult;

#[derive(Debug, Clone)]
pub enum VariableExpansion {
    Quoted(String),
    Unquoted(String),
    WordSplit(String),
    ArrayExpansion(String),
}

impl Validate for VariableExpansion {
    fn validate(&self) -> Result<(), ValidationError> {
        match self {
            VariableExpansion::Unquoted(var) => Err(ValidationError {
                rule: "SC2086",
                severity: Severity::Error,
                message: format!("Double quote to prevent globbing and word splitting: ${var}"),
                suggestion: Some(format!("Use \"${var}\" instead")),
                auto_fix: Some(Fix {
                    description: "Add quotes around variable".to_string(),
                    replacement: format!("\"${var}\""),
                }),
                line: None,
                column: None,
            }),
            _ => Ok(()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CommandSubstitution {
    pub command: String,
    pub context: SubstitutionContext,
}

#[derive(Debug, Clone)]
pub enum SubstitutionContext {
    Assignment,
    ArrayInit,
    Quoted,
    Unquoted,
}

impl Validate for CommandSubstitution {
    fn validate(&self) -> Result<(), ValidationError> {
        match self.context {
            SubstitutionContext::Unquoted => Err(ValidationError {
                rule: "SC2046",
                severity: Severity::Error,
                message: format!(
                    "Quote command substitution to prevent word splitting: $({})",
                    self.command
                ),
                suggestion: Some(format!("Use \"$({})\"", self.command)),
                auto_fix: Some(Fix {
                    description: "Add quotes around command substitution".to_string(),
                    replacement: format!("\"$({})\"", self.command),
                }),
                line: None,
                column: None,
            }),
            _ => Ok(()),
        }
    }
}

#[allow(clippy::result_large_err)]
pub fn validate_glob_pattern(pattern: &str) -> Result<String, ValidationError> {
    if pattern.starts_with('-') {
        return Err(ValidationError {
            rule: "SC2035",
            severity: Severity::Warning,
            message: "Use './' or -- to prevent glob patterns being interpreted as options"
                .to_string(),
            suggestion: Some(format!("Use './{pattern}'")),
            auto_fix: Some(Fix {
                description: "Prefix with './' to prevent option interpretation".to_string(),
                replacement: format!("./{pattern}"),
            }),
            line: None,
            column: None,
        });
    }

    if pattern.contains(['*', '?', '[']) {
        Ok(pattern.to_string())
    } else {
        Ok(shell_escape(pattern))
    }
}

#[derive(Debug, Clone)]
pub struct CommandSequence {
    pub commands: Vec<String>,
    pub exit_code_checks: Vec<ExitCodeCheck>,
}

#[derive(Debug, Clone)]
pub struct ExitCodeCheck {
    pub command_index: usize,
}

impl Validate for CommandSequence {
    fn validate(&self) -> Result<(), ValidationError> {
        for (i, check) in self.exit_code_checks.iter().enumerate() {
            if check.command_index != i {
                return Err(ValidationError {
                    rule: "SC2181",
                    severity: Severity::Style,
                    message: "Check exit code directly with 'if mycmd;', not indirectly with $?"
                        .to_string(),
                    suggestion: Some("Check $? immediately after command".to_string()),
                    auto_fix: None,
                    line: None,
                    column: None,
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum ConditionalExpression {
    StringComparison {
        left: Box<ShellExpression>,
        op: ComparisonOp,
        right: Box<ShellExpression>,
    },
    FileTest {
        op: FileTestOp,
        path: Box<ShellExpression>,
    },
}

#[derive(Debug, Clone)]
pub enum ComparisonOp {
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
}

#[derive(Debug, Clone)]
pub enum FileTestOp {
    Exists,
    IsFile,
    IsDir,
    IsReadable,
    IsWritable,
    IsExecutable,
}

impl Validate for ConditionalExpression {
    fn validate(&self) -> Result<(), ValidationError> {
        match self {
            ConditionalExpression::StringComparison { left, right, .. } => {
                if !left.is_quoted() || !right.is_quoted() {
                    return Err(ValidationError {
                        rule: "SC2086",
                        severity: Severity::Error,
                        message: "Quote variables in conditionals to prevent word splitting"
                            .to_string(),
                        suggestion: Some("Both sides of comparison must be quoted".to_string()),
                        auto_fix: None,
                        line: None,
                        column: None,
                    });
                }
                Ok(())
            }
            ConditionalExpression::FileTest { path, .. } => {
                if !path.is_quoted() {
                    return Err(ValidationError {
                        rule: "SC2086",
                        severity: Severity::Error,
                        message: "Quote file path to prevent word splitting".to_string(),
                        suggestion: Some("File paths in tests must be quoted".to_string()),
                        auto_fix: None,
                        line: None,
                        column: None,
                    });
                }
                Ok(())
            }
        }
    }
}

#[allow(clippy::result_large_err)]
pub fn validate_backticks(command: &str) -> Result<(), ValidationError> {
    if command.contains('`') {
        Err(ValidationError {
            rule: "SC2006",
            severity: Severity::Style,
            message: "Use $(...) notation instead of legacy backticks".to_string(),
            suggestion: Some("Replace backticks with $()".to_string()),
            auto_fix: Some(Fix {
                description: "Convert backticks to modern syntax".to_string(),
                replacement: command.replace('`', "$()"),
            }),
            line: None,
            column: None,
        })
    } else {
        Ok(())
    }
}

#[allow(clippy::result_large_err)]
pub fn validate_cd_usage(command: &str) -> Result<(), ValidationError> {
    if command.trim().starts_with("cd ") && !command.contains("||") {
        Err(ValidationError {
            rule: "SC2164",
            severity: Severity::Warning,
            message: "Use 'cd ... || exit' or 'cd ... || return' in case cd fails".to_string(),
            suggestion: Some("Add error handling for cd command".to_string()),
            auto_fix: Some(Fix {
                description: "Add error handling".to_string(),
                replacement: format!("{} || exit 1", command.trim()),
            }),
            line: None,
            column: None,
        })
    } else {
        Ok(())
    }
}

#[allow(clippy::result_large_err)]
pub fn validate_read_command(command: &str) -> Result<(), ValidationError> {
    if command.contains("read ") && !command.contains("-r") {
        Err(ValidationError {
            rule: "SC2162",
            severity: Severity::Warning,
            message: "read without -r will mangle backslashes".to_string(),
            suggestion: Some("Use 'read -r' to preserve backslashes".to_string()),
            auto_fix: Some(Fix {
                description: "Add -r flag".to_string(),
                replacement: command.replace("read ", "read -r "),
            }),
            line: None,
            column: None,
        })
    } else {
        Ok(())
    }
}

#[allow(clippy::result_large_err)]
pub fn validate_unicode_quotes(text: &str) -> Result<(), ValidationError> {
    let unicode_quotes = ['\u{201c}', '\u{201d}', '\u{2018}', '\u{2019}'];

    for quote in &unicode_quotes {
        if text.contains(*quote) {
            return Err(ValidationError {
                rule: "SC2220",
                severity: Severity::Error,
                message: "Unicode quotes must be replaced with ASCII quotes".to_string(),
                suggestion: Some("Use standard ASCII quotes".to_string()),
                auto_fix: Some(Fix {
                    description: "Replace Unicode quotes".to_string(),
                    replacement: text
                        .replace(['\u{201c}', '\u{201d}'], "\"")
                        .replace(['\u{2018}', '\u{2019}'], "'"),
                }),
                line: None,
                column: None,
            });
        }
    }
    Ok(())
}

pub fn validate_all(snippet: &str) -> RashResult<()> {
    validate_backticks(snippet)
        .map_err(|e| crate::models::error::Error::ShellCheckValidation(Box::new(e)))?;
    validate_cd_usage(snippet)
        .map_err(|e| crate::models::error::Error::ShellCheckValidation(Box::new(e)))?;
    validate_read_command(snippet)
        .map_err(|e| crate::models::error::Error::ShellCheckValidation(Box::new(e)))?;
    validate_unicode_quotes(snippet)
        .map_err(|e| crate::models::error::Error::ShellCheckValidation(Box::new(e)))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== VariableExpansion tests =====

    #[test]
    fn test_variable_expansion_quoted() {
        let exp = VariableExpansion::Quoted("var".to_string());
        assert!(exp.validate().is_ok());
    }

    #[test]
    fn test_variable_expansion_unquoted() {
        let exp = VariableExpansion::Unquoted("var".to_string());
        let result = exp.validate();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.rule, "SC2086");
    }

    #[test]
    fn test_variable_expansion_word_split() {
        let exp = VariableExpansion::WordSplit("var".to_string());
        assert!(exp.validate().is_ok());
    }

    #[test]
    fn test_variable_expansion_array() {
        let exp = VariableExpansion::ArrayExpansion("arr".to_string());
        assert!(exp.validate().is_ok());
    }

    // ===== CommandSubstitution tests =====

    #[test]
    fn test_command_substitution_assignment() {
        let cmd = CommandSubstitution {
            command: "ls".to_string(),
            context: SubstitutionContext::Assignment,
        };
        assert!(cmd.validate().is_ok());
    }

    #[test]
    fn test_command_substitution_array_init() {
        let cmd = CommandSubstitution {
            command: "find .".to_string(),
            context: SubstitutionContext::ArrayInit,
        };
        assert!(cmd.validate().is_ok());
    }

    #[test]
    fn test_command_substitution_quoted() {
        let cmd = CommandSubstitution {
            command: "pwd".to_string(),
            context: SubstitutionContext::Quoted,
        };
        assert!(cmd.validate().is_ok());
    }

    #[test]
    fn test_command_substitution_unquoted() {
        let cmd = CommandSubstitution {
            command: "date".to_string(),
            context: SubstitutionContext::Unquoted,
        };
        let result = cmd.validate();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.rule, "SC2046");
    }

    // ===== validate_glob_pattern tests =====

    #[test]
    fn test_glob_pattern_normal() {
        let result = validate_glob_pattern("*.txt");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "*.txt");
    }

    #[test]
    fn test_glob_pattern_with_question() {
        let result = validate_glob_pattern("file?.txt");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "file?.txt");
    }

    #[test]
    fn test_glob_pattern_with_brackets() {
        let result = validate_glob_pattern("[abc].txt");
        assert!(result.is_ok());
    }

    #[test]
    fn test_glob_pattern_starts_with_dash() {
        let result = validate_glob_pattern("-rf");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.rule, "SC2035");
    }

    #[test]
    fn test_glob_pattern_no_glob_chars() {
        let result = validate_glob_pattern("file.txt");
        assert!(result.is_ok());
    }

    // ===== CommandSequence tests =====

    #[test]
    fn test_command_sequence_valid() {
        let seq = CommandSequence {
            commands: vec!["cmd1".to_string(), "cmd2".to_string()],
            exit_code_checks: vec![
                ExitCodeCheck { command_index: 0 },
                ExitCodeCheck { command_index: 1 },
            ],
        };
        assert!(seq.validate().is_ok());
    }

    #[test]
    fn test_command_sequence_invalid() {
        let seq = CommandSequence {
            commands: vec!["cmd1".to_string(), "cmd2".to_string()],
            exit_code_checks: vec![
                ExitCodeCheck { command_index: 0 },
                ExitCodeCheck { command_index: 0 }, // Wrong index
            ],
        };
        let result = seq.validate();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.rule, "SC2181");
    }

    #[test]
    fn test_command_sequence_empty() {
        let seq = CommandSequence {
            commands: vec![],
            exit_code_checks: vec![],
        };
        assert!(seq.validate().is_ok());
    }

    // ===== validate_backticks tests =====

    #[test]
    fn test_backticks_present() {
        let result = validate_backticks("echo `date`");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.rule, "SC2006");
    }

    #[test]
    fn test_backticks_absent() {
        let result = validate_backticks("echo $(date)");
        assert!(result.is_ok());
    }

    // ===== validate_cd_usage tests =====

    #[test]
    fn test_cd_without_error_handling() {
        let result = validate_cd_usage("cd /tmp");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.rule, "SC2164");
    }

    #[test]
    fn test_cd_with_error_handling() {
        let result = validate_cd_usage("cd /tmp || exit 1");
        assert!(result.is_ok());
    }

    #[test]
    fn test_not_cd_command() {
        let result = validate_cd_usage("ls /tmp");
        assert!(result.is_ok());
    }

    // ===== validate_read_command tests =====

    #[test]
    fn test_read_without_r() {
        let result = validate_read_command("read var");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.rule, "SC2162");
    }

    #[test]
    fn test_read_with_r() {
        let result = validate_read_command("read -r var");
        assert!(result.is_ok());
    }

    #[test]
    fn test_not_read_command() {
        let result = validate_read_command("echo hello");
        assert!(result.is_ok());
    }

    // ===== validate_unicode_quotes tests =====

    #[test]
    fn test_unicode_left_double_quote() {
        let result = validate_unicode_quotes("echo \u{201c}hello\u{201d}");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.rule, "SC2220");
    }

    #[test]
    fn test_unicode_single_quote() {
        let result = validate_unicode_quotes("echo \u{2018}hello\u{2019}");
        assert!(result.is_err());
    }

    #[test]
    fn test_ascii_quotes() {
        let result = validate_unicode_quotes("echo \"hello\"");
        assert!(result.is_ok());
    }

    // ===== validate_all tests =====

    #[test]
    fn test_validate_all_valid() {
        let result = validate_all("echo $(date)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_all_backticks() {
        let result = validate_all("echo `date`");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_all_cd() {
        let result = validate_all("cd /tmp");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_all_read() {
        let result = validate_all("read var");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_all_unicode() {
        let result = validate_all("echo \u{201c}hi\u{201d}");
        assert!(result.is_err());
    }

    // ===== ConditionalExpression tests =====

    #[test]
    fn test_conditional_string_comparison_quoted() {
        let expr = ConditionalExpression::StringComparison {
            left: Box::new(ShellExpression::Variable("x".to_string(), true)),
            op: ComparisonOp::Eq,
            right: Box::new(ShellExpression::Variable("y".to_string(), true)),
        };
        assert!(expr.validate().is_ok());
    }

    #[test]
    fn test_conditional_string_comparison_unquoted() {
        let expr = ConditionalExpression::StringComparison {
            left: Box::new(ShellExpression::Variable("x".to_string(), false)),
            op: ComparisonOp::Eq,
            right: Box::new(ShellExpression::Variable("y".to_string(), true)),
        };
        let result = expr.validate();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.rule, "SC2086");
    }

    #[test]
    fn test_conditional_file_test_quoted() {
        let expr = ConditionalExpression::FileTest {
            op: FileTestOp::Exists,
            path: Box::new(ShellExpression::Variable("path".to_string(), true)),
        };
        assert!(expr.validate().is_ok());
    }

    #[test]
    fn test_conditional_file_test_unquoted() {
        let expr = ConditionalExpression::FileTest {
            op: FileTestOp::IsFile,
            path: Box::new(ShellExpression::Variable("path".to_string(), false)),
        };
        let result = expr.validate();
        assert!(result.is_err());
    }

    // ===== Clone/Debug trait tests =====

    #[test]
    fn test_variable_expansion_clone() {
        let exp = VariableExpansion::Quoted("var".to_string());
        let cloned = exp.clone();
        matches!(cloned, VariableExpansion::Quoted(_));
    }

    #[test]
    fn test_command_substitution_clone() {
        let cmd = CommandSubstitution {
            command: "ls".to_string(),
            context: SubstitutionContext::Assignment,
        };
        let cloned = cmd.clone();
        assert_eq!(cloned.command, "ls");
    }

    #[test]
    fn test_comparison_op_clone() {
        let ops = [
            ComparisonOp::Eq,
            ComparisonOp::Ne,
            ComparisonOp::Lt,
            ComparisonOp::Gt,
            ComparisonOp::Le,
            ComparisonOp::Ge,
        ];
        for op in ops {
            let _ = op.clone();
        }
    }

    #[test]
    fn test_file_test_op_clone() {
        let ops = [
            FileTestOp::Exists,
            FileTestOp::IsFile,
            FileTestOp::IsDir,
            FileTestOp::IsReadable,
            FileTestOp::IsWritable,
            FileTestOp::IsExecutable,
        ];
        for op in ops {
            let _ = op.clone();
        }
    }

    #[test]
    fn test_substitution_context_clone() {
        let contexts = [
            SubstitutionContext::Assignment,
            SubstitutionContext::ArrayInit,
            SubstitutionContext::Quoted,
            SubstitutionContext::Unquoted,
        ];
        for ctx in contexts {
            let _ = ctx.clone();
        }
    }

    #[test]
    fn test_variable_expansion_debug() {
        let exp = VariableExpansion::Quoted("var".to_string());
        let debug = format!("{:?}", exp);
        assert!(debug.contains("Quoted"));
    }
}
