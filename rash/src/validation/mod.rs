use crate::models::error::RashResult;
use std::fmt;

pub mod pipeline;
pub mod rules;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod pipeline_tests;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    Default,
)]
pub enum ValidationLevel {
    None,
    #[default]
    Minimal,
    Strict,
    Paranoid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Style,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Style => "style",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub rule: &'static str,
    pub severity: Severity,
    pub message: String,
    pub suggestion: Option<String>,
    pub auto_fix: Option<Fix>,
    pub line: Option<usize>,
    pub column: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct Fix {
    pub description: String,
    pub replacement: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {}: {}",
            self.severity.as_str(),
            self.rule,
            self.message
        )?;
        if let Some(ref suggestion) = self.suggestion {
            write!(f, "\n  Suggestion: {}", suggestion)?;
        }
        Ok(())
    }
}

impl std::error::Error for ValidationError {}

pub trait Validate {
    #[allow(clippy::result_large_err)]
    fn validate(&self) -> Result<(), ValidationError>;
}

pub trait ShellCheckValidation {
    type Error: std::error::Error;

    fn validate(&self) -> Result<(), Self::Error>;
    fn emit_safe(&self) -> String;
}

#[repr(C)]
pub struct ValidatedNode {
    node_type: u16,
    rule_mask: u16,
    validation: u32,
}

static_assertions::const_assert_eq!(std::mem::size_of::<ValidatedNode>(), 8);

pub const IMPLEMENTED_RULES: &[&str] = &[
    "SC2086", "SC2046", "SC2035", "SC2181", "SC2006", "SC2016", "SC2034", "SC2154", "SC2129",
    "SC2164", "SC2103", "SC2115", "SC2162", "SC2219", "SC2220", "SC2088", "SC2068", "SC2145",
    "SC2053", "SC2010",
];

pub fn validate_shell_snippet(snippet: &str) -> RashResult<()> {
    rules::validate_all(snippet)
}
