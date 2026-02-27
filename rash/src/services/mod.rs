#[allow(clippy::expect_used)] // Parser uses expect() for internal invariants
pub mod parser;

#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "parser_coverage_tests.rs"]
mod parser_coverage_tests;

#[cfg(test)]
#[path = "parser_coverage_tests2.rs"]
mod parser_coverage_tests2;

#[cfg(test)]
#[path = "parser_coverage_tests3.rs"]
mod parser_coverage_tests3;

#[cfg(test)]
#[path = "parser_coverage_tests4.rs"]
mod parser_coverage_tests4;

#[cfg(test)]
#[path = "parser_coverage_tests5.rs"]
mod parser_coverage_tests5;

pub use parser::parse;
