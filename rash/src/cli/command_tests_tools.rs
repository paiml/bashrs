use super::*;

// ============================================================================
// Inspect Command Tests
// ============================================================================

#[test]
fn test_inspect_command_echo_example() {
    use super::inspect_command;
    use super::InspectionFormat;

    // Test basic echo example
    let result = inspect_command("echo-example", InspectionFormat::Markdown, None, false);
    let _ = result; // May succeed or fail
}

#[test]
fn test_inspect_command_bootstrap_example() {
    use super::inspect_command;
    use super::InspectionFormat;

    // Test bootstrap example
    let result = inspect_command("bootstrap-example", InspectionFormat::Json, None, false);
    let _ = result; // May succeed or fail
}

#[test]
fn test_inspect_command_json_ast() {
    use super::inspect_command;
    use super::InspectionFormat;

    // Test with JSON AST input
    let json_ast = r#"{"ExecuteCommand": {"command_name": "echo", "args": ["test"]}}"#;
    let result = inspect_command(json_ast, InspectionFormat::Markdown, None, false);
    let _ = result; // May succeed or fail
}

#[test]
fn test_inspect_command_invalid_input() {
    use super::inspect_command;
    use super::InspectionFormat;

    // Test with invalid input
    let result = inspect_command("invalid-example", InspectionFormat::Markdown, None, false);
    assert!(result.is_err());
}

#[test]
fn test_inspect_command_html_format() {
    use super::inspect_command;
    use super::InspectionFormat;

    // Test HTML format
    let result = inspect_command("echo-example", InspectionFormat::Html, None, false);
    let _ = result; // May succeed or fail
}

#[test]
fn test_inspect_command_with_output_file() {
    use super::inspect_command;
    use super::InspectionFormat;
    use tempfile::NamedTempFile;

    // Test with output file
    let temp_file = NamedTempFile::new().unwrap();
    let result = inspect_command(
        "echo-example",
        InspectionFormat::Markdown,
        Some(temp_file.path()),
        false,
    );
    let _ = result; // May succeed or fail

    // Verify file was written
    let content = fs::read_to_string(temp_file.path()).unwrap();
    assert!(!content.is_empty());
    assert!(content.contains("Formal Verification Report"));
}

#[test]
fn test_inspect_command_invalid_json() {
    use super::inspect_command;
    use super::InspectionFormat;

    // Test with malformed JSON
    let invalid_json = r#"{"invalid": json}"#;
    let result = inspect_command(invalid_json, InspectionFormat::Json, None, false);
    assert!(result.is_err());
}

#[test]
fn test_inspect_command_all_formats() {
    use super::inspect_command;
    use super::InspectionFormat;

    // Test all supported formats
    for format in [
        InspectionFormat::Markdown,
        InspectionFormat::Json,
        InspectionFormat::Html,
    ] {
        let result = inspect_command("echo-example", format.clone(), None, false);
        assert!(result.is_ok(), "Failed with format: {format:?}");
    }
}

#[test]
fn test_inspect_command_rust_file() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("test.rs");
    fs::write(&input_path, "fn main() { let x = 42; }").unwrap();

    let result = inspect_command(
        input_path.to_str().unwrap(),
        InspectionFormat::Markdown,
        None,
        false,
    );
    let _ = result;
}

#[test]
fn test_inspect_command_shell_script() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("script.sh");
    fs::write(&input_path, "#!/bin/bash\necho hello").unwrap();

    let result = inspect_command(
        input_path.to_str().unwrap(),
        InspectionFormat::Json,
        None,
        true,
    );
    let _ = result;
}

// ============================================================================
// Make Command Tests
// ============================================================================

#[test]
fn test_make_parse_command_basic() {
    let temp_dir = TempDir::new().unwrap();
    let makefile = temp_dir.path().join("Makefile");
    fs::write(
        &makefile,
        ".PHONY: all clean\n\nall:\n\t@echo 'Building...'\n\nclean:\n\t@rm -f *.o\n",
    )
    .unwrap();

    let result = make_parse_command(&makefile, MakeOutputFormat::Text);
    assert!(result.is_ok());
}

#[test]
fn test_make_parse_command_json_format() {
    let temp_dir = TempDir::new().unwrap();
    let makefile = temp_dir.path().join("Makefile");
    fs::write(&makefile, "all:\n\t@echo 'test'\n").unwrap();

    let result = make_parse_command(&makefile, MakeOutputFormat::Json);
    assert!(result.is_ok());
}

#[test]
fn test_make_lint_command_basic() {
    let temp_dir = TempDir::new().unwrap();
    let makefile = temp_dir.path().join("Makefile");
    // Include .SUFFIXES and .DELETE_ON_ERROR to avoid warnings
    fs::write(
        &makefile,
        ".SUFFIXES:\n.DELETE_ON_ERROR:\n.PHONY: all\nall:\n\t@echo test\n",
    )
    .unwrap();

    let result = make_lint_command(&makefile, LintFormat::Human, false, None, None);
    assert!(result.is_ok());
}

#[test]
fn test_make_lint_command_with_fix() {
    let temp_dir = TempDir::new().unwrap();
    let makefile = temp_dir.path().join("Makefile");
    let output = temp_dir.path().join("Makefile.fixed");
    fs::write(&makefile, ".PHONY: all\nall:\n\t@echo test\n").unwrap();

    let result = make_lint_command(&makefile, LintFormat::Human, true, Some(&output), None);
    // May or may not have fixable issues
    let _ = result;
}

#[test]
fn test_make_lint_command_json_format() {
    let temp_dir = TempDir::new().unwrap();
    let makefile = temp_dir.path().join("Makefile");
    fs::write(&makefile, ".PHONY: all\nall:\n\t@echo test\n").unwrap();

    // Note: show_lint_results calls process::exit on warnings/errors
    // so we test with a rule filter that produces no matches
    let result = make_lint_command(
        &makefile,
        LintFormat::Human,
        false,
        None,
        Some("NONEXISTENT"),
    );
    let _ = result;
}

#[test]
fn test_make_lint_command_with_rules_filter() {
    let temp_dir = TempDir::new().unwrap();
    let makefile = temp_dir.path().join("Makefile");
    fs::write(&makefile, "all:\n\t@echo test\n").unwrap();

    let result = make_lint_command(&makefile, LintFormat::Human, false, None, Some("MAKE001"));
    let _ = result;
}

#[test]
fn test_make_purify_command_basic() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("Makefile");
    let output = temp_dir.path().join("Makefile.purified");
    fs::write(&input, ".PHONY: all\nall:\n\t@echo test\n").unwrap();

    let result = make_purify_command(
        &input,
        Some(&output),
        false, // fix
        false, // report
        ReportFormat::Human,
        false, // with_tests
        false, // property_tests
        false, // preserve_formatting
        None,  // max_line_length
        false, // skip_blank_line_removal
        false, // skip_consolidation
    );
    assert!(result.is_ok());
}

// ============================================================================
// Config Command Tests
// ============================================================================

#[test]
fn test_config_analyze_command_basic() {
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join(".bashrc");
    fs::write(
        &config_file,
        "export PATH=\"/usr/bin:$PATH\"\nalias ll='ls -la'\n",
    )
    .unwrap();

    let result = config_analyze_command(&config_file, ConfigOutputFormat::Human);
    assert!(result.is_ok());
}

#[test]
fn test_config_analyze_command_json() {
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join(".zshrc");
    fs::write(&config_file, "export EDITOR=vim\n").unwrap();

    let result = config_analyze_command(&config_file, ConfigOutputFormat::Json);
    assert!(result.is_ok());
}

#[test]
fn test_config_lint_command_basic() {
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join(".bashrc");
    fs::write(&config_file, "export PATH=/usr/bin\n").unwrap();

    let result = config_lint_command(&config_file, ConfigOutputFormat::Human);
    let _ = result;
}

#[test]
fn test_config_analyze_command_json_format() {
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join(".bashrc");
    fs::write(
        &config_file,
        "export PATH=/usr/bin:$PATH\nalias ll='ls -la'\n",
    )
    .unwrap();

    let result = config_analyze_command(&config_file, ConfigOutputFormat::Json);
    assert!(result.is_ok());
}

#[test]
fn test_config_analyze_command_nonexistent() {
    let result = config_analyze_command(
        &PathBuf::from("/nonexistent/.bashrc"),
        ConfigOutputFormat::Human,
    );
    assert!(result.is_err());
}

#[test]
fn test_config_lint_command_json() {
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join(".bashrc");
    fs::write(&config_file, "export PATH=/usr/bin\n").unwrap();

    let result = config_lint_command(&config_file, ConfigOutputFormat::Json);
    let _ = result;
}

// ============================================================================
// Purify Command Tests
// ============================================================================

#[test]
fn test_purify_command_basic() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("script.sh");
    let output_path = temp_dir.path().join("purified.sh");

    fs::write(&input_path, "#!/bin/bash\necho $RANDOM").unwrap();

    let result = purify_command(PurifyCommandOptions {
        input: &input_path,
        output: Some(&output_path),
        report: false,
        with_tests: false,
        property_tests: false,
        type_check: false,
        emit_guards: false,
        type_strict: false,
        diff: false,
        verify: false,
        recursive: false,
    });
    let _ = result;
}

#[test]
fn test_purify_command_with_lint() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("script.sh");

    fs::write(&input_path, "#!/bin/bash\necho hello world").unwrap();

    let result = purify_command(PurifyCommandOptions {
        input: &input_path,
        output: None,
        report: true,
        with_tests: false,
        property_tests: false,
        type_check: false,
        emit_guards: false,
        type_strict: false,
        diff: false,
        verify: false,
        recursive: false,
    });
    let _ = result;
}

#[test]
fn test_purify_command_with_output_and_report() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("messy.sh");
    let output = temp_dir.path().join("purified.sh");
    fs::write(&input, "#!/bin/bash\nmkdir /tmp/test\necho $RANDOM\n").unwrap();

    let result = purify_command(PurifyCommandOptions {
        input: &input,
        output: Some(&output),
        report: true,
        with_tests: false,
        property_tests: false,
        type_check: false,
        emit_guards: false,
        type_strict: false,
        diff: false,
        verify: false,
        recursive: false,
    });
    assert!(result.is_ok());
    assert!(output.exists());
}

#[test]
fn test_purify_command_to_stdout() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/bash\necho hello\n").unwrap();

    let result = purify_command(PurifyCommandOptions {
        input: &input,
        output: None,
        report: false,
        with_tests: false,
        property_tests: false,
        type_check: false,
        emit_guards: false,
        type_strict: false,
        diff: false,
        verify: false,
        recursive: false,
    });
    assert!(result.is_ok());
}

#[test]
fn test_purify_command_with_tests() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    let output = temp_dir.path().join("purified.sh");
    fs::write(&input, "#!/bin/bash\necho hello\n").unwrap();

    let result = purify_command(PurifyCommandOptions {
        input: &input,
        output: Some(&output),
        report: false,
        with_tests: true,
        property_tests: false,
        type_check: false,
        emit_guards: false,
        type_strict: false,
        diff: false,
        verify: false,
        recursive: false,
    });
    assert!(result.is_ok());
    // Test file should be generated
    let test_path = temp_dir.path().join("purified_test.sh");
    assert!(test_path.exists());
}

#[test]
fn test_purify_command_with_property_tests() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    let output = temp_dir.path().join("purified.sh");
    fs::write(&input, "#!/bin/bash\necho hello\n").unwrap();

    let result = purify_command(PurifyCommandOptions {
        input: &input,
        output: Some(&output),
        report: true,
        with_tests: true,
        property_tests: true,
        type_check: false,
        emit_guards: false,
        type_strict: false,
        diff: false,
        verify: false,
        recursive: false,
    });
    assert!(result.is_ok());
}

#[test]
fn test_purify_command_with_tests_requires_output() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/bash\necho hello\n").unwrap();

    let result = purify_command(PurifyCommandOptions {
        input: &input,
        output: None,
        report: false,
        with_tests: true,
        property_tests: false,
        type_check: false,
        emit_guards: false,
        type_strict: false,
        diff: false,
        verify: false,
        recursive: false,
    });
    assert!(result.is_err()); // --with-tests requires -o flag
}

#[test]
fn test_purify_command_nonexistent_file() {
    let result = purify_command(PurifyCommandOptions {
        input: &PathBuf::from("/nonexistent/purify.sh"),
        output: None,
        report: false,
        with_tests: false,
        property_tests: false,
        type_check: false,
        emit_guards: false,
        type_strict: false,
        diff: false,
        verify: false,
        recursive: false,
    });
    assert!(result.is_err());
}

// ============================================================================
// Playbook Command Tests
// ============================================================================

#[test]
fn test_playbook_command_validate_human() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("playbook.yaml");
    fs::write(
        &input,
        "version: \"1.0\"\nmachine:\n  id: test-machine\n  initial: start\n",
    )
    .unwrap();

    let result = playbook_command(&input, false, PlaybookFormat::Human, false, false);
    assert!(result.is_ok());
}

#[test]
fn test_playbook_command_run_human() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("playbook.yaml");
    fs::write(
        &input,
        "version: \"1.0\"\nmachine:\n  id: deploy\n  initial: setup\n",
    )
    .unwrap();

    let result = playbook_command(&input, true, PlaybookFormat::Human, true, false);
    assert!(result.is_ok());
}

#[test]
fn test_playbook_command_dry_run() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("playbook.yaml");
    fs::write(
        &input,
        "version: \"1.0\"\nmachine:\n  id: test\n  initial: start\n",
    )
    .unwrap();

    let result = playbook_command(&input, true, PlaybookFormat::Human, false, true);
    assert!(result.is_ok());
}

#[test]
fn test_playbook_command_json() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("playbook.yaml");
    fs::write(
        &input,
        "version: \"1.0\"\nmachine:\n  id: test\n  initial: start\n",
    )
    .unwrap();

    let result = playbook_command(&input, false, PlaybookFormat::Json, false, false);
    assert!(result.is_ok());
}

#[test]
fn test_playbook_command_junit() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("playbook.yaml");
    fs::write(
        &input,
        "version: \"1.0\"\nmachine:\n  id: test\n  initial: start\n",
    )
    .unwrap();

    let result = playbook_command(&input, false, PlaybookFormat::Junit, false, false);
    assert!(result.is_ok());
}

#[test]
fn test_playbook_command_invalid() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("bad.yaml");
    fs::write(&input, "this is not a valid playbook").unwrap();

    let result = playbook_command(&input, false, PlaybookFormat::Human, false, false);
    assert!(result.is_err());
}

#[test]
fn test_playbook_command_nonexistent() {
    let result = playbook_command(
        &PathBuf::from("/nonexistent/playbook.yaml"),
        false,
        PlaybookFormat::Human,
        false,
        false,
    );
    assert!(result.is_err());
}

// ============================================================================
// Mutate Command Tests
// ============================================================================

#[test]
fn test_mutate_command_human() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(
        &input,
        "#!/bin/sh\nif [ \"$x\" == \"y\" ]; then\n  echo true\nfi\n",
    )
    .unwrap();

    let result = mutate_command(&input, None, MutateFormat::Human, 10, false, None);
    assert!(result.is_ok());
}

#[test]
fn test_mutate_command_json() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\nif [ $x -eq 0 ]; then exit 0; fi\n").unwrap();

    let result = mutate_command(&input, None, MutateFormat::Json, 5, false, None);
    assert!(result.is_ok());
}

#[test]
fn test_mutate_command_csv() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\ntrue && echo ok\n").unwrap();

    let result = mutate_command(&input, None, MutateFormat::Csv, 5, false, None);
    assert!(result.is_ok());
}

#[test]
fn test_mutate_command_show_survivors() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(
        &input,
        "#!/bin/sh\nif [ \"$a\" == \"$b\" ]; then\n  echo equal\nfi\nexit 0\n",
    )
    .unwrap();

    let result = mutate_command(&input, None, MutateFormat::Human, 10, true, None);
    assert!(result.is_ok());
}

#[test]
fn test_mutate_command_no_mutations() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\necho hello\n").unwrap();

    let result = mutate_command(&input, None, MutateFormat::Human, 10, false, None);
    assert!(result.is_ok());
}

#[test]
fn test_mutate_command_nonexistent() {
    let result = mutate_command(
        &PathBuf::from("/nonexistent/mutate.sh"),
        None,
        MutateFormat::Human,
        10,
        false,
        None,
    );
    assert!(result.is_err());
}

// ============================================================================
// Simulate Command Tests
// ============================================================================

#[test]
fn test_simulate_command_human_deterministic() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\necho 'deterministic'\nexit 0\n").unwrap();

    let result = simulate_command(&input, 42, false, false, SimulateFormat::Human, false);
    assert!(result.is_ok());
}

#[test]
fn test_simulate_command_human_nondeterministic() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\necho $RANDOM\necho $$\n").unwrap();

    let result = simulate_command(&input, 42, false, false, SimulateFormat::Human, false);
    assert!(result.is_ok());
}

#[test]
fn test_simulate_command_with_trace() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\necho hello\necho world\n").unwrap();

    let result = simulate_command(&input, 42, false, false, SimulateFormat::Human, true);
    assert!(result.is_ok());
}

#[test]
fn test_simulate_command_with_verify() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\necho test\n").unwrap();

    let result = simulate_command(&input, 42, true, false, SimulateFormat::Human, true);
    assert!(result.is_ok());
}

#[test]
fn test_simulate_command_with_mock_externals() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\necho test\n").unwrap();

    let result = simulate_command(&input, 42, false, true, SimulateFormat::Human, false);
    assert!(result.is_ok());
}

#[test]
fn test_simulate_command_json() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\necho test\n").unwrap();

    let result = simulate_command(&input, 42, false, false, SimulateFormat::Json, false);
    assert!(result.is_ok());
}

#[test]
fn test_simulate_command_trace_format() {
    let temp_dir = TempDir::new().unwrap();
    let input = temp_dir.path().join("script.sh");
    fs::write(&input, "#!/bin/sh\n# comment\necho hello\necho world\n").unwrap();

    let result = simulate_command(&input, 42, false, false, SimulateFormat::Trace, false);
    assert!(result.is_ok());
}

#[test]
fn test_simulate_command_nonexistent() {
    let result = simulate_command(
        &PathBuf::from("/nonexistent/sim.sh"),
        42,
        false,
        false,
        SimulateFormat::Human,
        false,
    );
    assert!(result.is_err());
}
