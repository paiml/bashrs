use super::*;
use crate::cli::args::{
    AuditOutputFormat, CompileRuntime, ContainerFormatArg, CoverageOutputFormat, LintProfileArg,
    MutateFormat, PlaybookFormat, ReportFormat, ScoreOutputFormat, SimulateFormat,
    TestOutputFormat,
};
use crate::models::{ShellDialect, VerificationLevel};
use crate::validation::ValidationLevel;
use std::path::PathBuf;
use tempfile::TempDir;

#[path = "command_tests_build.rs"]
mod build;

#[path = "command_tests_helpers.rs"]
mod helpers;

#[path = "command_tests_dockerfile.rs"]
mod dockerfile;

#[allow(clippy::expect_used)]
#[path = "command_tests_quality.rs"]
mod quality;

#[path = "command_tests_tools.rs"]
mod tools;
