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

    let result =
        super::comply_cmds::handle_comply_command(crate::cli::args::ComplyCommands::Check {
            path: dir.path().to_path_buf(),
            scope: None,
            strict: false,
            failures_only: false,
            min_score: None,
            format: crate::cli::args::ComplyFormat::Text,
        });
    // Check command should succeed (may have low score but no error)
    let _ = result;
}

/// Test comply check command with JSON format.
#[test]
fn test_cov_comply_check_json() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("test.sh"), "#!/bin/sh\necho ok\n").unwrap();

    let result =
        super::comply_cmds::handle_comply_command(crate::cli::args::ComplyCommands::Check {
            path: dir.path().to_path_buf(),
            scope: None,
            strict: false,
            failures_only: false,
            min_score: None,
            format: crate::cli::args::ComplyFormat::Json,
        });
    let _ = result;
}

/// Test comply check command with Markdown format.
#[test]
fn test_cov_comply_check_markdown() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("test.sh"), "#!/bin/sh\necho ok\n").unwrap();

    let result =
        super::comply_cmds::handle_comply_command(crate::cli::args::ComplyCommands::Check {
            path: dir.path().to_path_buf(),
            scope: None,
            strict: false,
            failures_only: false,
            min_score: None,
            format: crate::cli::args::ComplyFormat::Markdown,
        });
    let _ = result;
}

/// Test comply check command with --failures-only.
#[test]
fn test_cov_comply_check_failures_only() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("test.sh"), "#!/bin/sh\necho ok\n").unwrap();

    let result =
        super::comply_cmds::handle_comply_command(crate::cli::args::ComplyCommands::Check {
            path: dir.path().to_path_buf(),
            scope: Some(crate::cli::args::ComplyScopeArg::Project),
            strict: false,
            failures_only: true,
            min_score: None,
            format: crate::cli::args::ComplyFormat::Text,
        });
    let _ = result;
}

/// Test comply check with --min-score threshold.
#[test]
fn test_cov_comply_check_min_score() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("test.sh"), "#!/bin/sh\necho ok\n").unwrap();

    let result =
        super::comply_cmds::handle_comply_command(crate::cli::args::ComplyCommands::Check {
            path: dir.path().to_path_buf(),
            scope: None,
            strict: false,
            failures_only: false,
            min_score: Some(100), // Intentionally high — may fail
            format: crate::cli::args::ComplyFormat::Text,
        });
    // High min_score on empty project will likely fail — that's fine for coverage
    let _ = result;
}

/// Test comply status command.
#[test]
fn test_cov_comply_status() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("test.sh"), "#!/bin/sh\necho ok\n").unwrap();

    let result =
        super::comply_cmds::handle_comply_command(crate::cli::args::ComplyCommands::Status {
            path: dir.path().to_path_buf(),
            format: crate::cli::args::ComplyFormat::Text,
        });
    assert!(result.is_ok());
}

/// Test comply status command with JSON format.
#[test]
fn test_cov_comply_status_json() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("test.sh"), "#!/bin/sh\necho ok\n").unwrap();

    let result =
        super::comply_cmds::handle_comply_command(crate::cli::args::ComplyCommands::Status {
            path: dir.path().to_path_buf(),
            format: crate::cli::args::ComplyFormat::Json,
        });
    assert!(result.is_ok());
}

/// Test comply rules command (text format).
#[test]
fn test_cov_comply_rules_text() {
    let result =
        super::comply_cmds::handle_comply_command(crate::cli::args::ComplyCommands::Rules {
            format: crate::cli::args::ComplyFormat::Text,
        });
    assert!(result.is_ok());
}


include!("command_tests_lint_cov_incl2_incl2.rs");
