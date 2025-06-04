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
                message: format!(
                    "Double quote to prevent globbing and word splitting: ${}",
                    var
                ),
                suggestion: Some(format!("Use \"${}\" instead", var)),
                auto_fix: Some(Fix {
                    description: "Add quotes around variable".to_string(),
                    replacement: format!("\"${}\"", var),
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

pub fn validate_glob_pattern(pattern: &str) -> Result<String, ValidationError> {
    if pattern.starts_with('-') {
        return Err(ValidationError {
            rule: "SC2035",
            severity: Severity::Warning,
            message: format!(
                "Use './' or -- to prevent glob patterns being interpreted as options"
            ),
            suggestion: Some(format!("Use './{}'", pattern)),
            auto_fix: Some(Fix {
                description: "Prefix with './' to prevent option interpretation".to_string(),
                replacement: format!("./{}", pattern),
            }),
            line: None,
            column: None,
        });
    }

    if pattern.contains(|c| matches!(c, '*' | '?' | '[')) {
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
                        .replace('\u{201c}', "\"")
                        .replace('\u{201d}', "\"")
                        .replace('\u{2018}', "'")
                        .replace('\u{2019}', "'"),
                }),
                line: None,
                column: None,
            });
        }
    }
    Ok(())
}

pub fn validate_all(snippet: &str) -> RashResult<()> {
    validate_backticks(snippet)?;
    validate_cd_usage(snippet)?;
    validate_read_command(snippet)?;
    validate_unicode_quotes(snippet)?;

    Ok(())
}
