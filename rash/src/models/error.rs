use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;
pub type RashResult<T> = Result<T>;
pub type RashError = Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Parse error: {0}")]
    Parse(#[from] syn::Error),

    #[error("AST validation error: {0}")]
    Validation(String),

    #[error("IR generation error: {0}")]
    IrGeneration(String),

    #[error("Verification error: {0}")]
    Verification(String),

    #[error("Code emission error: {0}")]
    Emission(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Format error: {0}")]
    Format(#[from] std::fmt::Error),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("ShellCheck validation error: {0}")]
    ShellCheckValidation(#[from] crate::validation::ValidationError),

    #[error("Validation error: {0}")]
    ValidationError(String),
}
