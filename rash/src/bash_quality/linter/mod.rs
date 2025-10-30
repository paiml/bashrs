// Linter module with smart suppression support

pub mod suppressions;

pub use suppressions::{known_external_vars, should_suppress_sc2154, FileType, LintContext};
