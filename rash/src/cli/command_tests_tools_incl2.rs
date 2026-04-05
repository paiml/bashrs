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
