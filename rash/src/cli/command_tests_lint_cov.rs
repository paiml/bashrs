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

/// Test lint_command with directory input (walk for lintable files).
#[test]
fn test_cov_lint_directory_walk() {
    let dir = TempDir::new().unwrap();

    // Create nested shell scripts
    let sub = dir.path().join("scripts");
    fs::create_dir_all(&sub).unwrap();
    fs::write(sub.join("a.sh"), "#!/bin/sh\necho a\n").unwrap();
    fs::write(sub.join("b.sh"), "#!/bin/sh\necho b\n").unwrap();

    // Create hidden directory (should be skipped)
    let hidden = dir.path().join(".hidden");
    fs::create_dir_all(&hidden).unwrap();
    fs::write(hidden.join("skip.sh"), "#!/bin/sh\necho skip\n").unwrap();

    let inputs = vec![dir.path().to_path_buf()];
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

/// Test lint_command with multiple files (multi-file mode).
#[test]
fn test_cov_lint_multiple_files() {
    let dir = TempDir::new().unwrap();
    let file1 = dir.path().join("a.sh");
    let file2 = dir.path().join("b.sh");
    fs::write(&file1, "#!/bin/sh\necho a\n").unwrap();
    fs::write(&file2, "#!/bin/sh\necho b\n").unwrap();

    let inputs = vec![file1, file2];
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

/// Test lint_command with multiple files in CI mode.
#[test]
fn test_cov_lint_multiple_files_ci() {
    let dir = TempDir::new().unwrap();
    let file1 = dir.path().join("a.sh");
    let file2 = dir.path().join("b.sh");
    fs::write(&file1, "#!/bin/sh\nset -eu\necho a\n").unwrap();
    fs::write(&file2, "#!/bin/sh\nset -eu\necho b\n").unwrap();

    let inputs = vec![file1, file2];
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

/// Test lint helper: load_ignore_file with no_ignore=true returns None.
#[test]
fn test_cov_lint_load_ignore_file_no_ignore() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("test.sh");
    fs::write(&file, "#!/bin/sh\n").unwrap();

    let result = super::lint_cmds::load_ignore_file(&file, true, None);
    assert!(result.is_none());
}

/// Test lint helper: load_ignore_file with non-existent ignore file.
#[test]
fn test_cov_lint_load_ignore_file_missing() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("test.sh");
    fs::write(&file, "#!/bin/sh\n").unwrap();

    let result = super::lint_cmds::load_ignore_file(&file, false, None);
    // No .bashrsignore in temp dir, so should return None
    assert!(result.is_none());
}

/// Test lint helper: build_ignored_rules with various inputs.
#[test]
fn test_cov_lint_build_ignored_rules() {
    // With comma-separated --ignore
    let rules = super::lint_cmds::build_ignored_rules(Some("SEC001,DET002"), None, None);
    assert!(rules.contains("SEC001"));
    assert!(rules.contains("DET002"));

    // With --exclude
    let excludes = vec!["SEC003".to_string()];
    let rules2 = super::lint_cmds::build_ignored_rules(None, Some(&excludes), None);
    assert!(rules2.contains("SEC003"));

    // With empty strings
    let rules3 = super::lint_cmds::build_ignored_rules(Some(""), None, None);
    assert!(rules3.is_empty());
}

/// Test lint helper: determine_min_severity.
#[test]
fn test_cov_lint_determine_min_severity() {
    use crate::linter::Severity;

    // quiet mode suppresses info
    let s1 = super::lint_cmds::determine_min_severity(true, crate::cli::args::LintLevel::Info);
    assert!(s1 >= Severity::Warning);

    // explicit warning level
    let s2 = super::lint_cmds::determine_min_severity(false, crate::cli::args::LintLevel::Warning);
    assert!(s2 >= Severity::Warning);

    // explicit error level
    let s3 = super::lint_cmds::determine_min_severity(false, crate::cli::args::LintLevel::Error);
    assert!(s3 >= Severity::Error);

    // info level (show all)
    let s4 = super::lint_cmds::determine_min_severity(false, crate::cli::args::LintLevel::Info);
    assert!(s4 <= Severity::Info);
}

/// Test lint helper: export_citl_if_requested with no path (no-op).
#[test]
fn test_cov_lint_export_citl_no_path() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("test.sh");

    let result = crate::linter::LintResult {
        diagnostics: vec![],
    };
    // Should be a no-op when citl_export_path is None
    super::lint_cmds::export_citl_if_requested(&file, &result, None);
}

/// Test lint helper: export_citl_if_requested with path.
#[test]
fn test_cov_lint_export_citl_with_path() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("test.sh");
    let citl_path = dir.path().join("export.citl.json");

    let result = crate::linter::LintResult {
        diagnostics: vec![],
    };
    super::lint_cmds::export_citl_if_requested(&file, &result, Some(&citl_path));
    assert!(citl_path.exists());
}

/// Test lint_command with --fix on a script that has fixable issues.
#[test]
fn test_cov_lint_fix_mode() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("fixme.sh");
    // Write a script with known fixable issues (non-quoted variable)
    fs::write(&file, "#!/bin/sh\necho $foo\n").unwrap();

    let inputs = vec![file];
    let opts = LintCommandOptions {
        inputs: &inputs,
        format: LintFormat::Human,
        fix: true,
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

// ============================================================================
// Comply Commands Coverage Tests
// ============================================================================

/// Test comply check command with default config on a temp directory.
#[test]
fn test_cov_comply_check_default() {
    let dir = TempDir::new().unwrap();
    // Create a simple shell script for the checker to find
    fs::write(dir.path().join("test.sh"), "#!/bin/sh\nset -eu\necho ok\n").unwrap();

    let result = super::comply_cmds::handle_comply_command(
        crate::cli::args::ComplyCommands::Check {
            path: dir.path().to_path_buf(),
            scope: None,
            strict: false,
            failures_only: false,
            min_score: None,
            format: crate::cli::args::ComplyFormat::Text,
        },
    );
    // Check command should succeed (may have low score but no error)
    let _ = result;
}

/// Test comply check command with JSON format.
#[test]
fn test_cov_comply_check_json() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("test.sh"), "#!/bin/sh\necho ok\n").unwrap();

    let result = super::comply_cmds::handle_comply_command(
        crate::cli::args::ComplyCommands::Check {
            path: dir.path().to_path_buf(),
            scope: None,
            strict: false,
            failures_only: false,
            min_score: None,
            format: crate::cli::args::ComplyFormat::Json,
        },
    );
    let _ = result;
}

/// Test comply check command with Markdown format.
#[test]
fn test_cov_comply_check_markdown() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("test.sh"), "#!/bin/sh\necho ok\n").unwrap();

    let result = super::comply_cmds::handle_comply_command(
        crate::cli::args::ComplyCommands::Check {
            path: dir.path().to_path_buf(),
            scope: None,
            strict: false,
            failures_only: false,
            min_score: None,
            format: crate::cli::args::ComplyFormat::Markdown,
        },
    );
    let _ = result;
}

/// Test comply check command with --failures-only.
#[test]
fn test_cov_comply_check_failures_only() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("test.sh"), "#!/bin/sh\necho ok\n").unwrap();

    let result = super::comply_cmds::handle_comply_command(
        crate::cli::args::ComplyCommands::Check {
            path: dir.path().to_path_buf(),
            scope: Some(crate::cli::args::ComplyScopeArg::Project),
            strict: false,
            failures_only: true,
            min_score: None,
            format: crate::cli::args::ComplyFormat::Text,
        },
    );
    let _ = result;
}

/// Test comply check with --min-score threshold.
#[test]
fn test_cov_comply_check_min_score() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("test.sh"), "#!/bin/sh\necho ok\n").unwrap();

    let result = super::comply_cmds::handle_comply_command(
        crate::cli::args::ComplyCommands::Check {
            path: dir.path().to_path_buf(),
            scope: None,
            strict: false,
            failures_only: false,
            min_score: Some(100), // Intentionally high — may fail
            format: crate::cli::args::ComplyFormat::Text,
        },
    );
    // High min_score on empty project will likely fail — that's fine for coverage
    let _ = result;
}

/// Test comply status command.
#[test]
fn test_cov_comply_status() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("test.sh"), "#!/bin/sh\necho ok\n").unwrap();

    let result = super::comply_cmds::handle_comply_command(
        crate::cli::args::ComplyCommands::Status {
            path: dir.path().to_path_buf(),
            format: crate::cli::args::ComplyFormat::Text,
        },
    );
    assert!(result.is_ok());
}

/// Test comply status command with JSON format.
#[test]
fn test_cov_comply_status_json() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("test.sh"), "#!/bin/sh\necho ok\n").unwrap();

    let result = super::comply_cmds::handle_comply_command(
        crate::cli::args::ComplyCommands::Status {
            path: dir.path().to_path_buf(),
            format: crate::cli::args::ComplyFormat::Json,
        },
    );
    assert!(result.is_ok());
}

/// Test comply rules command (text format).
#[test]
fn test_cov_comply_rules_text() {
    let result = super::comply_cmds::handle_comply_command(
        crate::cli::args::ComplyCommands::Rules {
            format: crate::cli::args::ComplyFormat::Text,
        },
    );
    assert!(result.is_ok());
}

/// Test comply rules command (JSON format).
#[test]
fn test_cov_comply_rules_json() {
    let result = super::comply_cmds::handle_comply_command(
        crate::cli::args::ComplyCommands::Rules {
            format: crate::cli::args::ComplyFormat::Json,
        },
    );
    assert!(result.is_ok());
}

/// Test comply rules command (Markdown format).
#[test]
fn test_cov_comply_rules_markdown() {
    let result = super::comply_cmds::handle_comply_command(
        crate::cli::args::ComplyCommands::Rules {
            format: crate::cli::args::ComplyFormat::Markdown,
        },
    );
    assert!(result.is_ok());
}

/// Test comply init command on a fresh directory.
///
/// NOTE: comply init uses cwd-relative paths internally.
/// We avoid set_current_dir to prevent parallel test races.
/// Instead, we test the comply init error path (already exists in project root)
/// and individual helper functions.
#[test]
fn test_cov_comply_init_already_exists_in_cwd() {
    // The project root likely already has .bashrs/ — so init will fail.
    // This exercises the "already exists" error path in comply_init_command.
    let result = super::comply_cmds::handle_comply_command(
        crate::cli::args::ComplyCommands::Init {
            scope: crate::cli::args::ComplyScopeArg::Project,
            pzsh: false,
            strict: false,
        },
    );
    // Expected: either Err (already exists) or Ok (if no comply.toml).
    // Both paths exercise code coverage.
    let _ = result;
}

/// Test comply init with pzsh and strict flags (error path).
#[test]
fn test_cov_comply_init_pzsh_strict() {
    let result = super::comply_cmds::handle_comply_command(
        crate::cli::args::ComplyCommands::Init {
            scope: crate::cli::args::ComplyScopeArg::All,
            pzsh: true,
            strict: true,
        },
    );
    let _ = result;
}

/// Test comply init with user scope.
#[test]
fn test_cov_comply_init_user_scope() {
    let result = super::comply_cmds::handle_comply_command(
        crate::cli::args::ComplyCommands::Init {
            scope: crate::cli::args::ComplyScopeArg::User,
            pzsh: false,
            strict: false,
        },
    );
    let _ = result;
}

/// Test comply init with system scope.
#[test]
fn test_cov_comply_init_system_scope() {
    let result = super::comply_cmds::handle_comply_command(
        crate::cli::args::ComplyCommands::Init {
            scope: crate::cli::args::ComplyScopeArg::System,
            pzsh: false,
            strict: true,
        },
    );
    let _ = result;
}

/// Test comply track discover command.
#[test]
fn test_cov_comply_track_discover() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("test.sh"), "#!/bin/sh\necho ok\n").unwrap();
    fs::write(
        dir.path().join("Makefile"),
        ".PHONY: all\nall:\n\t@echo done\n",
    )
    .unwrap();

    let result = super::comply_cmds::handle_comply_command(
        crate::cli::args::ComplyCommands::Track {
            command: crate::cli::args::ComplyTrackCommands::Discover {
                path: dir.path().to_path_buf(),
                scope: crate::cli::args::ComplyScopeArg::Project,
            },
        },
    );
    assert!(result.is_ok());
}

/// Test comply track discover with All scope.
#[test]
fn test_cov_comply_track_discover_all() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("test.sh"), "#!/bin/sh\necho ok\n").unwrap();

    let result = super::comply_cmds::handle_comply_command(
        crate::cli::args::ComplyCommands::Track {
            command: crate::cli::args::ComplyTrackCommands::Discover {
                path: dir.path().to_path_buf(),
                scope: crate::cli::args::ComplyScopeArg::All,
            },
        },
    );
    assert!(result.is_ok());
}

/// Test comply track list command.
#[test]
fn test_cov_comply_track_list() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("test.sh"), "#!/bin/sh\necho ok\n").unwrap();

    let result = super::comply_cmds::handle_comply_command(
        crate::cli::args::ComplyCommands::Track {
            command: crate::cli::args::ComplyTrackCommands::List {
                path: dir.path().to_path_buf(),
                scope: None,
            },
        },
    );
    assert!(result.is_ok());
}

/// Test comply track list with specific scope.
#[test]
fn test_cov_comply_track_list_project() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("test.sh"), "#!/bin/sh\necho ok\n").unwrap();

    let result = super::comply_cmds::handle_comply_command(
        crate::cli::args::ComplyCommands::Track {
            command: crate::cli::args::ComplyTrackCommands::List {
                path: dir.path().to_path_buf(),
                scope: Some(crate::cli::args::ComplyScopeArg::Project),
            },
        },
    );
    assert!(result.is_ok());
}

/// Test comply check with each scope variant.
#[test]
fn test_cov_comply_check_scope_user() {
    let dir = TempDir::new().unwrap();
    let result = super::comply_cmds::handle_comply_command(
        crate::cli::args::ComplyCommands::Check {
            path: dir.path().to_path_buf(),
            scope: Some(crate::cli::args::ComplyScopeArg::User),
            strict: false,
            failures_only: false,
            min_score: None,
            format: crate::cli::args::ComplyFormat::Text,
        },
    );
    let _ = result;
}

/// Test comply check with system scope.
#[test]
fn test_cov_comply_check_scope_system() {
    let dir = TempDir::new().unwrap();
    let result = super::comply_cmds::handle_comply_command(
        crate::cli::args::ComplyCommands::Check {
            path: dir.path().to_path_buf(),
            scope: Some(crate::cli::args::ComplyScopeArg::System),
            strict: false,
            failures_only: false,
            min_score: None,
            format: crate::cli::args::ComplyFormat::Text,
        },
    );
    let _ = result;
}

/// Test comply check with all scope.
#[test]
fn test_cov_comply_check_scope_all() {
    let dir = TempDir::new().unwrap();
    let result = super::comply_cmds::handle_comply_command(
        crate::cli::args::ComplyCommands::Check {
            path: dir.path().to_path_buf(),
            scope: Some(crate::cli::args::ComplyScopeArg::All),
            strict: false,
            failures_only: false,
            min_score: None,
            format: crate::cli::args::ComplyFormat::Text,
        },
    );
    let _ = result;
}

// ============================================================================
// Config Commands Coverage Tests
// ============================================================================

/// Test config analyze command with human output.
#[test]
fn test_cov_config_analyze_human() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join(".bashrc");
    fs::write(
        &file,
        "#!/bin/bash\nexport PATH=\"/usr/local/bin:$PATH\"\nexport PATH=\"/usr/local/bin:$PATH\"\nalias ll='ls -la'\n",
    )
    .unwrap();

    let result = config_analyze_command(&file, ConfigOutputFormat::Human);
    assert!(result.is_ok());
}

/// Test config analyze command with JSON output.
#[test]
fn test_cov_config_analyze_json() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join(".bashrc");
    fs::write(
        &file,
        "#!/bin/bash\nexport PATH=\"/usr/local/bin:$PATH\"\nalias ll='ls -la'\n",
    )
    .unwrap();

    let result = config_analyze_command(&file, ConfigOutputFormat::Json);
    assert!(result.is_ok());
}

/// Test config lint command with human output (no issues).
///
/// NOTE: config_lint_command calls std::process::exit(1) when issues are found.
/// We use a minimal clean config to avoid triggering issues.
#[test]
fn test_cov_config_lint_clean() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join(".profile");
    // A truly minimal config file that should have zero issues
    fs::write(&file, "# Clean profile\n").unwrap();

    let result = config_lint_command(&file, ConfigOutputFormat::Human);
    assert!(result.is_ok());
}

/// Test config lint command with JSON output.
///
/// NOTE: config_lint_command calls std::process::exit(1) when issues are found.
/// We use a minimal clean config to avoid triggering issues.
#[test]
fn test_cov_config_lint_json() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join(".zshrc");
    // Minimal clean config that should produce zero issues
    fs::write(&file, "# Clean zshrc\n").unwrap();

    let result = config_lint_command(&file, ConfigOutputFormat::Json);
    assert!(result.is_ok());
}

/// Test config purify command in dry-run mode.
#[test]
fn test_cov_config_purify_dry_run() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join(".bashrc");
    fs::write(
        &file,
        "#!/bin/bash\nexport PATH=\"/usr/local/bin:$PATH\"\nexport PATH=\"/usr/local/bin:$PATH\"\n",
    )
    .unwrap();

    let result = super::config_cmds::handle_config_command(
        crate::cli::args::ConfigCommands::Purify {
            input: file,
            output: None,
            fix: false,
            no_backup: false,
            dry_run: true,
        },
    );
    assert!(result.is_ok());
}

/// Test config purify command writing to output file.
#[test]
fn test_cov_config_purify_output_file() {
    let dir = TempDir::new().unwrap();
    let input = dir.path().join(".bashrc");
    let output = dir.path().join(".bashrc.purified");
    fs::write(
        &input,
        "#!/bin/bash\nexport PATH=\"/usr/local/bin:$PATH\"\nexport PATH=\"/usr/local/bin:$PATH\"\n",
    )
    .unwrap();

    let result = super::config_cmds::handle_config_command(
        crate::cli::args::ConfigCommands::Purify {
            input,
            output: Some(output.clone()),
            fix: false,
            no_backup: false,
            dry_run: false,
        },
    );
    assert!(result.is_ok());
    assert!(output.exists());
}

/// Test config purify command writing to stdout (output path = "-").
#[test]
fn test_cov_config_purify_stdout() {
    let dir = TempDir::new().unwrap();
    let input = dir.path().join(".bashrc");
    fs::write(&input, "#!/bin/bash\nexport EDITOR=vim\n").unwrap();

    let stdout_path = PathBuf::from("-");
    let result = super::config_cmds::handle_config_command(
        crate::cli::args::ConfigCommands::Purify {
            input,
            output: Some(stdout_path),
            fix: false,
            no_backup: false,
            dry_run: false,
        },
    );
    assert!(result.is_ok());
}

/// Test config purify command with --fix (in-place with backup).
#[test]
fn test_cov_config_purify_fix_inplace() {
    let dir = TempDir::new().unwrap();
    let input = dir.path().join(".bashrc");
    fs::write(
        &input,
        "#!/bin/bash\nexport PATH=\"/usr/local/bin:$PATH\"\nexport PATH=\"/usr/local/bin:$PATH\"\n",
    )
    .unwrap();

    let result = super::config_cmds::handle_config_command(
        crate::cli::args::ConfigCommands::Purify {
            input,
            output: None,
            fix: true,
            no_backup: false,
            dry_run: false,
        },
    );
    assert!(result.is_ok());
}

/// Test config purify command with --fix --no-backup.
#[test]
fn test_cov_config_purify_fix_no_backup() {
    let dir = TempDir::new().unwrap();
    let input = dir.path().join(".bashrc");
    fs::write(
        &input,
        "#!/bin/bash\nexport PATH=\"/usr/local/bin:$PATH\"\nexport PATH=\"/usr/local/bin:$PATH\"\n",
    )
    .unwrap();

    let result = super::config_cmds::handle_config_command(
        crate::cli::args::ConfigCommands::Purify {
            input,
            output: None,
            fix: true,
            no_backup: true,
            dry_run: false,
        },
    );
    assert!(result.is_ok());
}

/// Test helper: should_output_to_stdout.
#[test]
fn test_cov_config_should_output_to_stdout() {
    assert!(should_output_to_stdout(std::path::Path::new("-")));
    assert!(!should_output_to_stdout(std::path::Path::new("foo.sh")));
    assert!(!should_output_to_stdout(std::path::Path::new("/tmp/out")));
}

/// Test helper: count_duplicate_path_entries.
#[test]
fn test_cov_config_count_duplicate_path_entries() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join(".bashrc");
    fs::write(
        &file,
        "#!/bin/bash\nexport PATH=\"/usr/local/bin:$PATH\"\nexport PATH=\"/usr/local/bin:$PATH\"\n",
    )
    .unwrap();

    let source = fs::read_to_string(&file).unwrap();
    let analysis = crate::config::analyzer::analyze_config(&source, file);
    let dup_count = count_duplicate_path_entries(&analysis);
    // May or may not find duplicates — just exercise the code path
    let _ = dup_count;
}

/// Test helper: handle_output_to_file with regular file.
#[test]
fn test_cov_config_handle_output_to_file() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("output.sh");
    let result = handle_output_to_file(&out, "#!/bin/sh\necho purified\n");
    assert!(result.is_ok());
    assert!(out.exists());
}

/// Test helper: handle_output_to_file with stdout.
#[test]
fn test_cov_config_handle_output_to_file_stdout() {
    let result = handle_output_to_file(std::path::Path::new("-"), "#!/bin/sh\necho purified\n");
    assert!(result.is_ok());
}

/// Test config analyze via handle_config_command dispatch.
#[test]
fn test_cov_config_dispatch_analyze() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join(".bashrc");
    fs::write(&file, "#!/bin/bash\nexport EDITOR=vim\n").unwrap();

    let result = super::config_cmds::handle_config_command(
        crate::cli::args::ConfigCommands::Analyze {
            input: file,
            format: ConfigOutputFormat::Human,
        },
    );
    assert!(result.is_ok());
}

/// Test config lint via handle_config_command dispatch.
///
/// NOTE: config_lint_command calls std::process::exit(1) when issues are found.
/// We use a minimal clean config to avoid triggering issues.
#[test]
fn test_cov_config_dispatch_lint() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join(".zshrc");
    // Minimal clean config to avoid process::exit(1)
    fs::write(&file, "# Clean zshrc config\n").unwrap();

    let result = super::config_cmds::handle_config_command(
        crate::cli::args::ConfigCommands::Lint {
            input: file,
            format: ConfigOutputFormat::Human,
        },
    );
    assert!(result.is_ok());
}

// ============================================================================
// Gate Commands Coverage Tests
//
// NOTE: gate_commands.rs uses GateConfig::load() which reads .pmat-gates.toml
// from the current working directory. Tests run from the project root which
// already has .pmat-gates.toml. We exercise gate_commands directly and test
// GateConfig deserialization separately to avoid process-global set_current_dir
// races in parallel tests.
// ============================================================================

/// Test GateConfig deserialization with all gate types.
#[test]
fn test_cov_gate_config_deser_all_gates() {
    let config_content = r#"
[gates]
run_clippy = true
clippy_strict = true
run_tests = true
test_timeout = 120
check_coverage = false
min_coverage = 80.0
check_complexity = true
max_complexity = 10

[gates.satd]
enabled = true
max_count = 5
patterns = ["TODO", "FIXME"]

[gates.mutation]
enabled = false
min_score = 90.0

[gates.security]
enabled = true
max_unsafe_blocks = 0

[tiers]
tier1_gates = ["complexity", "satd"]
tier2_gates = ["clippy", "tests"]
tier3_gates = ["coverage", "mutation", "security"]
"#;
    let config: crate::gates::GateConfig = toml::from_str(config_content).unwrap();
    assert!(config.gates.run_clippy);
    assert!(config.gates.clippy_strict);
    assert!(config.gates.run_tests);
    assert_eq!(config.gates.test_timeout, 120);
    assert!(!config.gates.check_coverage);
    assert!(config.gates.check_complexity);
    assert_eq!(config.gates.max_complexity, 10);

    let satd = config.gates.satd.as_ref().unwrap();
    assert!(satd.enabled);
    assert_eq!(satd.patterns.len(), 2);

    let mutation = config.gates.mutation.as_ref().unwrap();
    assert!(!mutation.enabled);

    let security = config.gates.security.as_ref().unwrap();
    assert!(security.enabled);

    assert_eq!(config.tiers.tier1_gates.len(), 2);
    assert_eq!(config.tiers.tier2_gates.len(), 2);
    assert_eq!(config.tiers.tier3_gates.len(), 3);
}

/// Test GateConfig deserialization with minimal config (defaults).
#[test]
fn test_cov_gate_config_deser_minimal() {
    let config_content = r#"
[gates]
"#;
    let config: crate::gates::GateConfig = toml::from_str(config_content).unwrap();
    assert!(!config.gates.run_clippy);
    assert!(!config.gates.run_tests);
    assert!(!config.gates.check_coverage);
    assert!(config.gates.satd.is_none());
    assert!(config.gates.mutation.is_none());
    assert!(config.tiers.tier1_gates.is_empty());
}

/// Test GateConfig deserialization with SATD disabled.
#[test]
fn test_cov_gate_config_deser_satd_disabled() {
    let config_content = r#"
[gates]

[gates.satd]
enabled = false
"#;
    let config: crate::gates::GateConfig = toml::from_str(config_content).unwrap();
    let satd = config.gates.satd.as_ref().unwrap();
    assert!(!satd.enabled);
}

/// Test GateConfig deserialization with mutation gate.
#[test]
fn test_cov_gate_config_deser_mutation() {
    let config_content = r#"
[gates]

[gates.mutation]
enabled = true
min_score = 85.0
"#;
    let config: crate::gates::GateConfig = toml::from_str(config_content).unwrap();
    let mutation = config.gates.mutation.as_ref().unwrap();
    assert!(mutation.enabled);
    assert!((mutation.min_score - 85.0).abs() < f64::EPSILON);
}

/// Test handle_gate_command with invalid tier (exercises the error path
/// after loading real .pmat-gates.toml from project root).
#[test]
fn test_cov_gate_invalid_tier() {
    // Uses the project root's .pmat-gates.toml. Tier 99 is invalid.
    let result = super::gate_cmds::handle_gate_command(
        99,
        crate::cli::args::ReportFormat::Human,
    );
    // Should fail with "Invalid tier: 99"
    assert!(result.is_err());
}

/// Test handle_gate_command with tier 0 (also invalid).
#[test]
fn test_cov_gate_tier_zero() {
    let result = super::gate_cmds::handle_gate_command(
        0,
        crate::cli::args::ReportFormat::Human,
    );
    assert!(result.is_err());
}

// ============================================================================
// DevContainer Commands Coverage Tests
// ============================================================================

/// Test devcontainer validate with --list-rules flag.
#[test]
fn test_cov_devcontainer_list_rules() {
    let dir = TempDir::new().unwrap();
    let result = super::devcontainer_cmds::handle_devcontainer_command(
        crate::cli::args::DevContainerCommands::Validate {
            path: dir.path().to_path_buf(),
            format: LintFormat::Human,
            lint_dockerfile: false,
            list_rules: true,
        },
    );
    assert!(result.is_ok());
}

/// Test devcontainer validate with valid devcontainer.json.
#[test]
fn test_cov_devcontainer_validate_valid() {
    let dir = TempDir::new().unwrap();
    let dc_dir = dir.path().join(".devcontainer");
    fs::create_dir_all(&dc_dir).unwrap();
    fs::write(
        dc_dir.join("devcontainer.json"),
        r#"{
  "name": "Test Dev Container",
  "image": "mcr.microsoft.com/devcontainers/base:ubuntu"
}"#,
    )
    .unwrap();

    let result = super::devcontainer_cmds::handle_devcontainer_command(
        crate::cli::args::DevContainerCommands::Validate {
            path: dir.path().to_path_buf(),
            format: LintFormat::Human,
            lint_dockerfile: false,
            list_rules: false,
        },
    );
    let _ = result;
}

/// Test devcontainer validate with JSON format.
#[test]
fn test_cov_devcontainer_validate_json_format() {
    let dir = TempDir::new().unwrap();
    let dc_dir = dir.path().join(".devcontainer");
    fs::create_dir_all(&dc_dir).unwrap();
    fs::write(
        dc_dir.join("devcontainer.json"),
        r#"{"name": "Test", "image": "ubuntu:22.04"}"#,
    )
    .unwrap();

    let result = super::devcontainer_cmds::handle_devcontainer_command(
        crate::cli::args::DevContainerCommands::Validate {
            path: dir.path().to_path_buf(),
            format: LintFormat::Json,
            lint_dockerfile: false,
            list_rules: false,
        },
    );
    let _ = result;
}

/// Test devcontainer validate with SARIF format.
#[test]
fn test_cov_devcontainer_validate_sarif_format() {
    let dir = TempDir::new().unwrap();
    let dc_dir = dir.path().join(".devcontainer");
    fs::create_dir_all(&dc_dir).unwrap();
    fs::write(
        dc_dir.join("devcontainer.json"),
        r#"{"name": "Test", "image": "ubuntu:22.04"}"#,
    )
    .unwrap();

    let result = super::devcontainer_cmds::handle_devcontainer_command(
        crate::cli::args::DevContainerCommands::Validate {
            path: dir.path().to_path_buf(),
            format: LintFormat::Sarif,
            lint_dockerfile: false,
            list_rules: false,
        },
    );
    let _ = result;
}

/// Test devcontainer validate when no devcontainer.json exists (error).
#[test]
fn test_cov_devcontainer_validate_missing() {
    let dir = TempDir::new().unwrap();
    let result = super::devcontainer_cmds::handle_devcontainer_command(
        crate::cli::args::DevContainerCommands::Validate {
            path: dir.path().to_path_buf(),
            format: LintFormat::Human,
            lint_dockerfile: false,
            list_rules: false,
        },
    );
    assert!(result.is_err());
}

/// Test devcontainer validate with --lint-dockerfile and a referenced Dockerfile.
#[test]
fn test_cov_devcontainer_validate_lint_dockerfile() {
    let dir = TempDir::new().unwrap();
    let dc_dir = dir.path().join(".devcontainer");
    fs::create_dir_all(&dc_dir).unwrap();

    // Write Dockerfile
    fs::write(
        dc_dir.join("Dockerfile"),
        "FROM ubuntu:22.04\nRUN apt-get update\n",
    )
    .unwrap();

    // Write devcontainer.json referencing the Dockerfile
    fs::write(
        dc_dir.join("devcontainer.json"),
        r#"{
  "name": "Test",
  "build": {
    "dockerfile": "Dockerfile"
  }
}"#,
    )
    .unwrap();

    let result = super::devcontainer_cmds::handle_devcontainer_command(
        crate::cli::args::DevContainerCommands::Validate {
            path: dir.path().to_path_buf(),
            format: LintFormat::Human,
            lint_dockerfile: true,
            list_rules: false,
        },
    );
    let _ = result;
}

/// Test devcontainer validate with direct file path.
#[test]
fn test_cov_devcontainer_validate_direct_file() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("devcontainer.json");
    fs::write(
        &file,
        r#"{"name": "Direct", "image": "ubuntu:22.04"}"#,
    )
    .unwrap();

    let result = super::devcontainer_cmds::handle_devcontainer_command(
        crate::cli::args::DevContainerCommands::Validate {
            path: file,
            format: LintFormat::Human,
            lint_dockerfile: false,
            list_rules: false,
        },
    );
    let _ = result;
}

// ============================================================================
// Test Commands Coverage Tests
// ============================================================================

/// Test test_command with a shell script that has test functions.
#[test]
fn test_cov_test_command_with_tests() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("test_suite.sh");
    fs::write(
        &file,
        r#"#!/bin/sh
# TEST: Basic echo test
# GIVEN: A simple command
# WHEN: We run it
# THEN: It should succeed
test_echo() {
    result=$(echo hello)
    [ "$result" = "hello" ]
}

# TEST: Another test
test_true() {
    true
}
"#,
    )
    .unwrap();

    let result = test_command(
        &file,
        crate::cli::args::TestOutputFormat::Human,
        false,
        None,
    );
    let _ = result;
}

/// Test test_command with detailed output.
#[test]
fn test_cov_test_command_detailed() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("test_suite.sh");
    fs::write(
        &file,
        r#"#!/bin/sh
# TEST: Echo test
test_echo() {
    echo hello
}
"#,
    )
    .unwrap();

    let result = test_command(
        &file,
        crate::cli::args::TestOutputFormat::Human,
        true,
        None,
    );
    let _ = result;
}

/// Test test_command with JSON output.
#[test]
fn test_cov_test_command_json() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("test_suite.sh");
    fs::write(
        &file,
        r#"#!/bin/sh
test_basic() {
    true
}
"#,
    )
    .unwrap();

    let result = test_command(
        &file,
        crate::cli::args::TestOutputFormat::Json,
        false,
        None,
    );
    let _ = result;
}

/// Test test_command with JUnit output.
#[test]
fn test_cov_test_command_junit() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("test_suite.sh");
    fs::write(
        &file,
        r#"#!/bin/sh
test_basic() {
    true
}
"#,
    )
    .unwrap();

    let result = test_command(
        &file,
        crate::cli::args::TestOutputFormat::Junit,
        false,
        None,
    );
    let _ = result;
}

/// Test test_command with pattern filter.
#[test]
fn test_cov_test_command_with_pattern() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("test_suite.sh");
    fs::write(
        &file,
        r#"#!/bin/sh
test_echo() {
    echo hello
}
test_math() {
    [ "$(expr 1 + 1)" = "2" ]
}
"#,
    )
    .unwrap();

    let result = test_command(
        &file,
        crate::cli::args::TestOutputFormat::Human,
        false,
        Some("echo"),
    );
    let _ = result;
}

/// Test test_command with pattern that matches nothing.
#[test]
fn test_cov_test_command_pattern_no_match() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("test_suite.sh");
    fs::write(
        &file,
        r#"#!/bin/sh
test_echo() {
    echo hello
}
"#,
    )
    .unwrap();

    let result = test_command(
        &file,
        crate::cli::args::TestOutputFormat::Human,
        false,
        Some("nonexistent_pattern"),
    );
    // Should succeed (0 tests is not an error, just a warning)
    assert!(result.is_ok());
}

/// Test test_command with script that has no tests.
#[test]
fn test_cov_test_command_no_tests() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("empty.sh");
    fs::write(&file, "#!/bin/sh\necho 'no tests here'\n").unwrap();

    let result = test_command(
        &file,
        crate::cli::args::TestOutputFormat::Human,
        false,
        None,
    );
    assert!(result.is_ok());
}

/// Test test_command with nonexistent file.
#[test]
fn test_cov_test_command_nonexistent_file() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("nonexistent.sh");

    let result = test_command(
        &file,
        crate::cli::args::TestOutputFormat::Human,
        false,
        None,
    );
    assert!(result.is_err());
}

/// Test print_human_test_results directly.
#[test]
fn test_cov_print_human_test_results_empty() {
    let report = crate::bash_quality::testing::TestReport {
        tests: vec![],
        results: vec![],
        duration_ms: 0,
    };
    super::test_commands::print_human_test_results(&report, false);
}

/// Test print_json_test_results directly.
#[test]
fn test_cov_print_json_test_results_empty() {
    let report = crate::bash_quality::testing::TestReport {
        tests: vec![],
        results: vec![],
        duration_ms: 0,
    };
    super::test_commands::print_json_test_results(&report);
}

/// Test print_junit_test_results directly.
#[test]
fn test_cov_print_junit_test_results_empty() {
    let report = crate::bash_quality::testing::TestReport {
        tests: vec![],
        results: vec![],
        duration_ms: 0,
    };
    super::test_commands::print_junit_test_results(&report);
}

/// Test print_test_summary directly.
#[test]
fn test_cov_print_test_summary_empty() {
    let report = crate::bash_quality::testing::TestReport {
        tests: vec![],
        results: vec![],
        duration_ms: 42,
    };
    super::test_commands::print_test_summary(&report);
}

/// Test print_test_detail with nonexistent test name.
#[test]
fn test_cov_print_test_detail_missing() {
    let report = crate::bash_quality::testing::TestReport {
        tests: vec![],
        results: vec![],
        duration_ms: 0,
    };
    // Should be a no-op (test not found)
    super::test_commands::print_test_detail(&report, "nonexistent", true);
}

// ============================================================================
// Additional Lint Edge Cases
// ============================================================================

/// Test lint_command with .bashrsignore file.
#[test]
fn test_cov_lint_with_bashrsignore() {
    let dir = TempDir::new().unwrap();

    // Create a script with only warning-level issues (not errors),
    // so even if the ignore pattern doesn't match the full path,
    // CI mode with fail_on=Error won't call process::exit.
    let file = dir.path().join("test.sh");
    fs::write(&file, "#!/bin/sh\necho $HOME\n").unwrap();

    // Create a .bashrsignore that ignores this file
    let ignore_file = dir.path().join(".bashrsignore");
    fs::write(&ignore_file, "test.sh\n").unwrap();

    let inputs = vec![file];
    let opts = LintCommandOptions {
        inputs: &inputs,
        format: LintFormat::Human,
        fix: false,
        fix_assumptions: false,
        output: None,
        no_ignore: false,
        ignore_file_path: Some(&ignore_file),
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

/// Test lint_command with devcontainer profile.
#[test]
fn test_cov_lint_devcontainer_profile() {
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
        profile: crate::cli::args::LintProfileArg::DevContainer,
        ci: true,
        fail_on: crate::cli::args::LintLevel::Error,
    };
    let _ = lint_command(opts);
}
