#[allow(clippy::expect_used)] // Parser uses expect() for internal invariants
pub mod parser;

#[cfg(test)]
mod tests;

pub use parser::parse;
