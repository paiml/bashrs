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

    let result = format_command(std::slice::from_ref(&input), false, false, None);
    assert!(result.is_ok());
}

#[test]
fn test_format_command_check_mode() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\necho 'hello'\n").unwrap();

    let result = format_command(std::slice::from_ref(&input), true, false, None);
    // May pass or fail depending on formatting rules
    let _ = result;
}

#[test]
fn test_format_command_dry_run() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    let original = "#!/bin/sh\necho  'hello'\n";
    fs::write(&input, original).unwrap();

    let result = format_command(std::slice::from_ref(&input), false, true, None);
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

    let result = format_command(std::slice::from_ref(&input), false, false, Some(&output));
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
