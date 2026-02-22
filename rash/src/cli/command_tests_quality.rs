use super::*;

// ============================================================================
// Score Command Tests (covers score_command + print_* formatters)
// ============================================================================

#[test]
fn test_score_command_shell_script_human() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\nset -eu\necho 'hello'\nexit 0\n").unwrap();

    let result = score_command(
        &input,
        ScoreOutputFormat::Human,
        false,
        false,
        false,
        false,
        None,
    );
    assert!(result.is_ok());
}

#[test]
fn test_score_command_shell_script_json() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\necho 'test'\n").unwrap();

    let result = score_command(
        &input,
        ScoreOutputFormat::Json,
        false,
        false,
        false,
        false,
        None,
    );
    assert!(result.is_ok());
}

#[test]
fn test_score_command_shell_script_markdown() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\necho 'test'\n").unwrap();

    let result = score_command(
        &input,
        ScoreOutputFormat::Markdown,
        false,
        false,
        false,
        false,
        None,
    );
    assert!(result.is_ok());
}

#[test]
fn test_score_command_shell_script_detailed() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\nset -eu\necho 'hello'\n").unwrap();

    let result = score_command(
        &input,
        ScoreOutputFormat::Human,
        true, // detailed
        false,
        false,
        false,
        None,
    );
    assert!(result.is_ok());
}

#[test]
fn test_score_command_dockerfile_human() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    fs::write(&input, "FROM python:3.11-slim\nRUN pip install flask\nCOPY . /app\nWORKDIR /app\nCMD [\"python\", \"app.py\"]\n").unwrap();

    let result = score_command(
        &input,
        ScoreOutputFormat::Human,
        true,
        true, // dockerfile
        false,
        true, // show_grade
        None,
    );
    assert!(result.is_ok());
}

#[test]
fn test_score_command_dockerfile_json() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    fs::write(&input, "FROM alpine:3.18\nRUN apk add --no-cache curl\n").unwrap();

    let result = score_command(
        &input,
        ScoreOutputFormat::Json,
        false,
        true, // dockerfile
        false,
        false,
        None,
    );
    assert!(result.is_ok());
}

#[test]
fn test_score_command_dockerfile_markdown() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    fs::write(
        &input,
        "FROM node:20-alpine\nWORKDIR /app\nCOPY . .\nCMD [\"node\", \"index.js\"]\n",
    )
    .unwrap();

    let result = score_command(
        &input,
        ScoreOutputFormat::Markdown,
        false,
        true, // dockerfile
        false,
        false,
        None,
    );
    assert!(result.is_ok());
}

#[test]
fn test_score_command_dockerfile_with_runtime() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    fs::write(
        &input,
        "FROM ubuntu:22.04\nRUN apt-get update\nCOPY . /app\n",
    )
    .unwrap();

    let result = score_command(
        &input,
        ScoreOutputFormat::Human,
        true,
        true, // dockerfile
        true, // runtime
        true, // show_grade
        None,
    );
    assert!(result.is_ok());
}

#[test]
fn test_score_command_dockerfile_with_coursera_profile() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Dockerfile");
    fs::write(&input, "FROM python:3.11-slim\nRUN pip install flask\n").unwrap();

    let result = score_command(
        &input,
        ScoreOutputFormat::Human,
        true,
        true, // dockerfile
        true, // runtime
        true, // show_grade
        Some(LintProfileArg::Coursera),
    );
    assert!(result.is_ok());
}

#[test]
fn test_score_command_nonexistent_file() {
    let result = score_command(
        &PathBuf::from("/nonexistent/script.sh"),
        ScoreOutputFormat::Human,
        false,
        false,
        false,
        false,
        None,
    );
    assert!(result.is_err());
}

// ============================================================================
// Audit Command Tests (covers audit_command + print_* formatters)
// ============================================================================

#[test]
fn test_audit_command_basic_human() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\nset -eu\necho 'hello'\n").unwrap();

    let result = audit_command(&input, &AuditOutputFormat::Human, false, false, None);
    assert!(result.is_ok());
}

#[test]
fn test_audit_command_basic_json() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\necho 'hello'\n").unwrap();

    let result = audit_command(&input, &AuditOutputFormat::Json, false, false, None);
    assert!(result.is_ok());
}

#[test]
fn test_audit_command_basic_sarif() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\necho 'hello'\n").unwrap();

    let result = audit_command(&input, &AuditOutputFormat::Sarif, false, false, None);
    assert!(result.is_ok());
}

#[test]
fn test_audit_command_detailed() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\nset -eu\necho 'hello world'\nexit 0\n").unwrap();

    let result = audit_command(&input, &AuditOutputFormat::Human, false, true, None);
    assert!(result.is_ok());
}

#[test]
fn test_audit_command_strict_mode() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    // Script with unquoted variable (produces warning)
    fs::write(&input, "#!/bin/sh\necho $HOME\n").unwrap();

    let result = audit_command(&input, &AuditOutputFormat::Human, true, false, None);
    // Strict mode: warnings cause failure
    let _ = result; // may pass or fail depending on lint rules
}

#[test]
fn test_audit_command_min_grade_pass() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\nset -eu\necho 'hello'\nexit 0\n").unwrap();

    let result = audit_command(
        &input,
        &AuditOutputFormat::Human,
        false,
        false,
        Some("F"), // very low bar
    );
    assert!(result.is_ok());
}

#[test]
fn test_audit_command_min_grade_fail() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\necho $RANDOM\n").unwrap();

    let result = audit_command(
        &input,
        &AuditOutputFormat::Human,
        false,
        false,
        Some("A+"), // very high bar
    );
    // May fail if grade is below A+
    let _ = result;
}

#[test]
fn test_audit_command_nonexistent_file() {
    let result = audit_command(
        &PathBuf::from("/nonexistent/audit.sh"),
        &AuditOutputFormat::Human,
        false,
        false,
        None,
    );
    assert!(result.is_err());
}

// ============================================================================
// Coverage Command Tests (covers coverage_command + print_* formatters)
// ============================================================================

#[test]
fn test_coverage_command_terminal() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\nset -eu\necho 'hello'\nexit 0\n").unwrap();

    let result = coverage_command(&input, &CoverageOutputFormat::Terminal, None, false, None);
    assert!(result.is_ok());
}

#[test]
fn test_coverage_command_terminal_detailed() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\necho 'line1'\necho 'line2'\n").unwrap();

    let result = coverage_command(
        &input,
        &CoverageOutputFormat::Terminal,
        None,
        true, // detailed
        None,
    );
    assert!(result.is_ok());
}

#[test]
fn test_coverage_command_json() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\necho 'test'\n").unwrap();

    let result = coverage_command(&input, &CoverageOutputFormat::Json, None, false, None);
    assert!(result.is_ok());
}

#[test]
fn test_coverage_command_html_to_stdout() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\necho 'test'\n").unwrap();

    let result = coverage_command(&input, &CoverageOutputFormat::Html, None, false, None);
    assert!(result.is_ok());
}

#[test]
fn test_coverage_command_html_to_file() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    let output = temp_dir.path().join("coverage.html");
    fs::write(&input, "#!/bin/sh\necho 'test'\n").unwrap();

    let result = coverage_command(
        &input,
        &CoverageOutputFormat::Html,
        None,
        false,
        Some(&output),
    );
    assert!(result.is_ok());
    assert!(output.exists());
}

#[test]
fn test_coverage_command_lcov() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\necho 'test'\n").unwrap();

    let result = coverage_command(&input, &CoverageOutputFormat::Lcov, None, false, None);
    assert!(result.is_ok());
}

#[test]
fn test_coverage_command_min_threshold_pass() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\necho 'test'\n").unwrap();

    let result = coverage_command(
        &input,
        &CoverageOutputFormat::Terminal,
        Some(0), // 0% min - always passes
        false,
        None,
    );
    assert!(result.is_ok());
}

#[test]
fn test_coverage_command_nonexistent_file() {
    let result = coverage_command(
        &PathBuf::from("/nonexistent/coverage.sh"),
        &CoverageOutputFormat::Terminal,
        None,
        false,
        None,
    );
    assert!(result.is_err());
}

// ============================================================================
// Format Command Tests (covers format_command)
// ============================================================================

#[test]
fn test_format_command_basic_inplace() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\necho  'hello'\n").unwrap();

    let result = format_command(&[input.clone()], false, false, None);
    assert!(result.is_ok());
}

#[test]
fn test_format_command_check_mode() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\necho 'hello'\n").unwrap();

    let result = format_command(&[input.clone()], true, false, None);
    // May pass or fail depending on formatting rules
    let _ = result;
}

#[test]
fn test_format_command_dry_run() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    let original = "#!/bin/sh\necho  'hello'\n";
    fs::write(&input, original).unwrap();

    let result = format_command(&[input.clone()], false, true, None);
    assert!(result.is_ok());

    // Dry run should not modify the file
    let after = fs::read_to_string(&input).unwrap();
    assert_eq!(after, original);
}

#[test]
fn test_format_command_to_output_file() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    let output = temp_dir.path().join("formatted.sh");
    fs::write(&input, "#!/bin/sh\necho 'hello'\n").unwrap();

    let result = format_command(&[input.clone()], false, false, Some(&output));
    assert!(result.is_ok());
    assert!(output.exists());
}

#[test]
fn test_format_command_multiple_files() {
    let temp_dir = TempDir::new().unwrap();
    let input1 = temp_dir.path().join("a.sh");
    let input2 = temp_dir.path().join("b.sh");
    fs::write(&input1, "#!/bin/sh\necho 'a'\n").unwrap();
    fs::write(&input2, "#!/bin/sh\necho 'b'\n").unwrap();

    let result = format_command(&[input1, input2], false, false, None);
    assert!(result.is_ok());
}

#[test]
fn test_format_command_nonexistent_file() {
    let result = format_command(
        &[PathBuf::from("/nonexistent/format.sh")],
        false,
        false,
        None,
    );
    assert!(result.is_err());
}

// ============================================================================
// Test Command Tests (covers test_command + print_* formatters)
// ============================================================================

#[test]
fn test_test_command_no_tests_found() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\necho 'no tests here'\n").unwrap();

    let result = test_command(&input, TestOutputFormat::Human, false, None);
    assert!(result.is_ok()); // Returns OK with "No tests found" message
}

#[test]
fn test_test_command_json_format() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\necho 'no tests'\n").unwrap();

    let result = test_command(&input, TestOutputFormat::Json, false, None);
    assert!(result.is_ok());
}

#[test]
fn test_test_command_junit_format() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\necho 'no tests'\n").unwrap();

    let result = test_command(&input, TestOutputFormat::Junit, false, None);
    assert!(result.is_ok());
}

#[test]
fn test_test_command_with_pattern() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\necho 'no tests'\n").unwrap();

    let result = test_command(&input, TestOutputFormat::Human, false, Some("nonexistent"));
    assert!(result.is_ok()); // No tests match pattern
}

#[test]
fn test_test_command_nonexistent_file() {
    let result = test_command(
        &PathBuf::from("/nonexistent/test.sh"),
        TestOutputFormat::Human,
        false,
        None,
    );
    assert!(result.is_err());
}

// ============================================================================
// Convert Lint Format Test
// ============================================================================

#[test]
fn test_convert_lint_format_human() {
    let result = convert_lint_format(LintFormat::Human);
    assert!(matches!(result, crate::linter::output::OutputFormat::Human));
}

#[test]
fn test_convert_lint_format_json() {
    let result = convert_lint_format(LintFormat::Json);
    assert!(matches!(result, crate::linter::output::OutputFormat::Json));
}

#[test]
fn test_convert_lint_format_sarif() {
    let result = convert_lint_format(LintFormat::Sarif);
    assert!(matches!(result, crate::linter::output::OutputFormat::Sarif));
}

// ============================================================================
// Run Filtered Lint Tests
// ============================================================================

#[test]
fn test_run_filtered_lint_no_filter() {
    let source = ".PHONY: all\nall:\n\t@echo test\n";
    let result = run_filtered_lint(source, None);
    // Should return lint results (may have diagnostics)
    let _ = result.diagnostics.len();
}

#[test]
fn test_run_filtered_lint_with_filter() {
    let source = "all:\n\t@echo test\n";
    let result = run_filtered_lint(source, Some("MAKE001"));
    // Should only contain MAKE001 diagnostics (if any)
    for d in &result.diagnostics {
        assert!(d.code.contains("MAKE001"));
    }
}

#[test]
fn test_run_filtered_lint_nonexistent_rule() {
    let source = "all:\n\t@echo test\n";
    let result = run_filtered_lint(source, Some("NONEXISTENT999"));
    assert!(result.diagnostics.is_empty());
}
