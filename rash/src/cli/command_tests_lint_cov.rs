#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

//! Coverage tests for CLI command modules with 0% coverage:
//!   - lint_commands.rs (lint_command, expand_inputs, walk_for_lintable_files, ...)
//!   - comply_commands.rs (handle_comply_command, comply_check_command, ...)
//!   - config_commands.rs (config_analyze_command, config_lint_command, ...)
//!   - gate_commands.rs (handle_gate_command, run_*_gate, ...)
//!   - devcontainer_commands.rs (handle_devcontainer_command, devcontainer_validate, ...)
//!   - test_commands.rs (test_command, print_human_test_results, ...)

use super::*;
use std::path::PathBuf;
use tempfile::TempDir;

// ============================================================================
// Lint Commands Coverage Tests
// ============================================================================

/// Test lint_command with a single clean shell script (no lint warnings expected).
///
/// NOTE: All lint_command tests use `ci: true` + `fail_on: Error` to prevent
/// output_lint_results from calling std::process::exit(1) on warnings, which
/// would kill the entire test harness.
#[test]
fn test_cov_lint_single_clean_shell_file() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("clean.sh");
    fs::write(&file, "#!/bin/sh\nset -eu\necho hello\n").unwrap();

    let inputs = vec![file];
    let opts = LintCommandOptions {
        inputs: &inputs,
        format: LintFormat::Human,
        fix: false,
        fix_assumptions: false,
        output: None,
        no_ignore: true,
        ignore_file_path: None,
        quiet: false,
        level: crate::cli::args::LintLevel::Info,
        ignore_rules: None,
        exclude_rules: None,
        citl_export_path: None,
        profile: crate::cli::args::LintProfileArg::Standard,
        ci: true,
        fail_on: crate::cli::args::LintLevel::Error,
    };
    let _ = lint_command(opts);
}

/// Test lint_command with JSON output format.
#[test]
fn test_cov_lint_json_format() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("test.sh");
    fs::write(&file, "#!/bin/sh\necho hello\n").unwrap();

    let inputs = vec![file];
    let opts = LintCommandOptions {
        inputs: &inputs,
        format: LintFormat::Json,
        fix: false,
        fix_assumptions: false,
        output: None,
        no_ignore: true,
        ignore_file_path: None,
        quiet: false,
        level: crate::cli::args::LintLevel::Info,
        ignore_rules: None,
        exclude_rules: None,
        citl_export_path: None,
        profile: crate::cli::args::LintProfileArg::Standard,
        ci: true,
        fail_on: crate::cli::args::LintLevel::Error,
    };
    let _ = lint_command(opts);
}

/// Test lint_command with SARIF output format.
#[test]
fn test_cov_lint_sarif_format() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("test.sh");
    fs::write(&file, "#!/bin/sh\necho hello\n").unwrap();

    let inputs = vec![file];
    let opts = LintCommandOptions {
        inputs: &inputs,
        format: LintFormat::Sarif,
        fix: false,
        fix_assumptions: false,
        output: None,
        no_ignore: true,
        ignore_file_path: None,
        quiet: false,
        level: crate::cli::args::LintLevel::Info,
        ignore_rules: None,
        exclude_rules: None,
        citl_export_path: None,
        profile: crate::cli::args::LintProfileArg::Standard,
        ci: true,
        fail_on: crate::cli::args::LintLevel::Error,
    };
    let _ = lint_command(opts);
}

/// Test lint_command with --quiet flag (suppress info-level).
#[test]
fn test_cov_lint_quiet_mode() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("test.sh");
    fs::write(&file, "#!/bin/sh\necho hello\n").unwrap();

    let inputs = vec![file];
    let opts = LintCommandOptions {
        inputs: &inputs,
        format: LintFormat::Human,
        fix: false,
        fix_assumptions: false,
        output: None,
        no_ignore: true,
        ignore_file_path: None,
        quiet: true,
        level: crate::cli::args::LintLevel::Warning,
        ignore_rules: None,
        exclude_rules: None,
        citl_export_path: None,
        profile: crate::cli::args::LintProfileArg::Standard,
        ci: true,
        fail_on: crate::cli::args::LintLevel::Error,
    };
    let _ = lint_command(opts);
}

/// Test lint_command with --level error (only show errors).
#[test]
fn test_cov_lint_level_error() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("test.sh");
    fs::write(&file, "#!/bin/sh\necho hello\n").unwrap();

    let inputs = vec![file];
    let opts = LintCommandOptions {
        inputs: &inputs,
        format: LintFormat::Human,
        fix: false,
        fix_assumptions: false,
        output: None,
        no_ignore: true,
        ignore_file_path: None,
        quiet: false,
        level: crate::cli::args::LintLevel::Error,
        ignore_rules: None,
        exclude_rules: None,
        citl_export_path: None,
        profile: crate::cli::args::LintProfileArg::Standard,
        ci: true,
        fail_on: crate::cli::args::LintLevel::Error,
    };
    let _ = lint_command(opts);
}

/// Test lint_command with --ignore rules (comma-separated codes).
#[test]
fn test_cov_lint_ignore_rules() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("test.sh");
    fs::write(&file, "#!/bin/sh\necho $RANDOM\n").unwrap();

    let inputs = vec![file];
    let opts = LintCommandOptions {
        inputs: &inputs,
        format: LintFormat::Human,
        fix: false,
        fix_assumptions: false,
        output: None,
        no_ignore: true,
        ignore_file_path: None,
        quiet: false,
        level: crate::cli::args::LintLevel::Info,
        ignore_rules: Some("DET001,SEC001"),
        exclude_rules: None,
        citl_export_path: None,
        profile: crate::cli::args::LintProfileArg::Standard,
        ci: true,
        fail_on: crate::cli::args::LintLevel::Error,
    };
    let _ = lint_command(opts);
}

/// Test lint_command with --exclude rules (-e flag).
#[test]
fn test_cov_lint_exclude_rules() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("test.sh");
    fs::write(&file, "#!/bin/sh\necho $RANDOM\n").unwrap();

    let excludes = vec!["DET001".to_string(), "SEC001".to_string()];
    let inputs = vec![file];
    let opts = LintCommandOptions {
        inputs: &inputs,
        format: LintFormat::Human,
        fix: false,
        fix_assumptions: false,
        output: None,
        no_ignore: true,
        ignore_file_path: None,
        quiet: false,
        level: crate::cli::args::LintLevel::Info,
        ignore_rules: None,
        exclude_rules: Some(&excludes),
        citl_export_path: None,
        profile: crate::cli::args::LintProfileArg::Standard,
        ci: true,
        fail_on: crate::cli::args::LintLevel::Error,
    };
    let _ = lint_command(opts);
}

/// Test lint_command with CITL export path.
#[test]
fn test_cov_lint_citl_export() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("test.sh");
    // Use a script with a warning (unquoted var) but no error-level diagnostic,
    // so CI mode with fail_on=Error does not call process::exit.
    fs::write(&file, "#!/bin/sh\necho $HOME\n").unwrap();
    let citl_path = dir.path().join("diags.citl.json");

    let inputs = vec![file];
    let opts = LintCommandOptions {
        inputs: &inputs,
        format: LintFormat::Human,
        fix: false,
        fix_assumptions: false,
        output: None,
        no_ignore: true,
        ignore_file_path: None,
        quiet: false,
        level: crate::cli::args::LintLevel::Info,
        ignore_rules: None,
        exclude_rules: None,
        citl_export_path: Some(&citl_path),
        profile: crate::cli::args::LintProfileArg::Standard,
        ci: true,
        fail_on: crate::cli::args::LintLevel::Error,
    };
    let _ = lint_command(opts);
    // Verify CITL file was created
    assert!(citl_path.exists());
}

/// Test lint_command with CI mode (GitHub Actions annotations).
#[test]
fn test_cov_lint_ci_mode() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("test.sh");
    fs::write(&file, "#!/bin/sh\nset -eu\necho hello\n").unwrap();

    let inputs = vec![file];
    let opts = LintCommandOptions {
        inputs: &inputs,
        format: LintFormat::Human,
        fix: false,
        fix_assumptions: false,
        output: None,
        no_ignore: true,
        ignore_file_path: None,
        quiet: false,
        level: crate::cli::args::LintLevel::Info,
        ignore_rules: None,
        exclude_rules: None,
        citl_export_path: None,
        profile: crate::cli::args::LintProfileArg::Standard,
        ci: true,
        fail_on: crate::cli::args::LintLevel::Error,
    };
    let _ = lint_command(opts);
}

/// Test lint_command with no inputs returns validation error.
#[test]
fn test_cov_lint_no_inputs() {
    let inputs: Vec<PathBuf> = vec![];
    let opts = LintCommandOptions {
        inputs: &inputs,
        format: LintFormat::Human,
        fix: false,
        fix_assumptions: false,
        output: None,
        no_ignore: true,
        ignore_file_path: None,
        quiet: false,
        level: crate::cli::args::LintLevel::Info,
        ignore_rules: None,
        exclude_rules: None,
        citl_export_path: None,
        profile: crate::cli::args::LintProfileArg::Standard,
        ci: true,
        fail_on: crate::cli::args::LintLevel::Error,
    };
    let result = lint_command(opts);
    assert!(result.is_err());
}

/// Test lint_command on a Makefile.
#[test]
fn test_cov_lint_makefile() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("Makefile");
    fs::write(
        &file,
        ".PHONY: all\nall:\n\t@echo building\n\tgcc -o main main.c\n",
    )
    .unwrap();

    let inputs = vec![file];
    let opts = LintCommandOptions {
        inputs: &inputs,
        format: LintFormat::Human,
        fix: false,
        fix_assumptions: false,
        output: None,
        no_ignore: true,
        ignore_file_path: None,
        quiet: false,
        level: crate::cli::args::LintLevel::Info,
        ignore_rules: None,
        exclude_rules: None,
        citl_export_path: None,
        profile: crate::cli::args::LintProfileArg::Standard,
        ci: true,
        fail_on: crate::cli::args::LintLevel::Error,
    };
    let _ = lint_command(opts);
}

/// Test lint_command on a Dockerfile (CI mode to avoid process::exit on warnings).
#[test]
fn test_cov_lint_dockerfile() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("Dockerfile");
    fs::write(&file, "FROM ubuntu:22.04\nRUN apt-get update\n").unwrap();

    let inputs = vec![file];
    let opts = LintCommandOptions {
        inputs: &inputs,
        format: LintFormat::Human,
        fix: false,
        fix_assumptions: false,
        output: None,
        no_ignore: true,
        ignore_file_path: None,
        quiet: false,
        level: crate::cli::args::LintLevel::Info,
        ignore_rules: None,
        exclude_rules: None,
        citl_export_path: None,
        profile: crate::cli::args::LintProfileArg::Coursera,
        ci: true, // CI mode avoids process::exit on warnings
        fail_on: crate::cli::args::LintLevel::Error, // only exit on errors
    };
    let _ = lint_command(opts);
}

include!("command_tests_lint_cov_incl2.rs");
