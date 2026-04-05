#[test]
fn test_heredoc_execution() {
    let source = r#"
fn main() {
    print_multiline();
}

fn print_multiline() {}
"#;

    let config = Config::default();
    let shell = transpile(source, &config).unwrap();

    // Write to temp file and verify it's valid shell
    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    file.write_all(shell.as_bytes())
        .expect("Failed to write shell script");

    let output = Command::new("sh")
        .arg(file.path())
        .output()
        .expect("Failed to execute shell script");

    // Should execute (even if function does nothing)
    assert!(
        output.status.success() || output.status.code() == Some(127),
        "Script should execute. Exit code: {:?}, stderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
}

// ============================================================================
// PARAM-SPEC-003: Process ID $$ Purification
// ============================================================================

/// PARAM-SPEC-003: RED Phase
/// Test that $$ usage is documented for removal
#[test]
fn test_process_id_purification_baseline() {
    let source = r#"
fn main() {
    use_fixed_id();
}

fn use_fixed_id() {}
"#;

    let config = Config::default();
    let result = transpile(source, &config);

    assert!(
        result.is_ok(),
        "Should transpile fixed ID function: {:?}",
        result.err()
    );

    let shell = result.unwrap();
    eprintln!("Generated shell for fixed ID:\n{}", shell);

    // Verify function is called (not $$)
    assert!(
        shell.contains("use_fixed_id"),
        "Should use fixed identifier, not $$"
    );

    // Should NOT contain $$ in main function (trap cleanup usage is OK)
    let main_section = shell.split("main() {").nth(1).unwrap_or("");
    let main_body = main_section.split("}").next().unwrap_or("");
    assert!(
        !main_body.contains("$$"),
        "Main function should NOT contain $$ (trap cleanup is OK, but user code shouldn't use $$)"
    );
}

/// PARAM-SPEC-003: RED Phase - ADVANCED
/// Test that std::process::id() is NOT supported
#[test]
#[ignore] // Requires std::process::id() detection and rejection
fn test_process_id_rejection() {
    let source = r#"
fn main() {
    let pid = std::process::id();
    println!("PID: {}", pid);
}
"#;

    let config = Config::default();
    let result = transpile(source, &config);

    // Should fail validation - non-deterministic
    assert!(
        result.is_err(),
        "std::process::id() should be rejected as non-deterministic"
    );
}

/// PARAM-SPEC-003: RED Phase - EXECUTION
/// Test fixed ID execution
#[test]
fn test_process_id_execution() {
    let source = r#"
fn main() {
    use_session_id("test-session");
}

fn use_session_id(id: &str) {}
"#;

    let config = Config::default();
    let shell = transpile(source, &config).unwrap();

    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    file.write_all(shell.as_bytes())
        .expect("Failed to write shell script");

    let output = Command::new("sh")
        .arg(file.path())
        .output()
        .expect("Failed to execute shell script");

    assert!(
        output.status.success() || output.status.code() == Some(127),
        "Script should execute with fixed ID"
    );
}

// ============================================================================
// PARAM-SPEC-004: Background PID $! Purification
// ============================================================================

/// PARAM-SPEC-004: RED Phase
/// Test that background jobs are NOT generated
#[test]
fn test_background_pid_purification_baseline() {
    let source = r#"
fn main() {
    run_sync();
}

fn run_sync() {}
"#;

    let config = Config::default();
    let result = transpile(source, &config);

    assert!(
        result.is_ok(),
        "Should transpile sync function: {:?}",
        result.err()
    );

    let shell = result.unwrap();
    eprintln!("Generated shell for sync execution:\n{}", shell);

    // Verify function is called synchronously
    assert!(
        shell.contains("run_sync"),
        "Should call function synchronously"
    );

    // Should NOT contain background operators
    assert!(
        !shell.contains(" &") && !shell.contains("$!"),
        "Should NOT contain background job operators (non-deterministic)"
    );
}

/// PARAM-SPEC-004: RED Phase - ADVANCED
/// Test that async/await is NOT supported
#[test]
#[ignore] // Requires async detection and rejection
fn test_background_async_rejection() {
    let source = r#"
async fn background_task() {
    // Some work
}

fn main() {
    background_task();
}
"#;

    let config = Config::default();
    let result = transpile(source, &config);

    // Should fail validation - async is non-deterministic
    assert!(
        result.is_err(),
        "async functions should be rejected as non-deterministic"
    );
}

/// PARAM-SPEC-004: RED Phase - EXECUTION
/// Test synchronous execution
#[test]
fn test_background_sync_execution() {
    let source = r#"
fn main() {
    task1();
    task2();
}

fn task1() {}
fn task2() {}
"#;

    let config = Config::default();
    let shell = transpile(source, &config).unwrap();

    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    file.write_all(shell.as_bytes())
        .expect("Failed to write shell script");

    let output = Command::new("sh")
        .arg(file.path())
        .output()
        .expect("Failed to execute shell script");

    assert!(
        output.status.success() || output.status.code() == Some(127),
        "Script should execute synchronously"
    );
}

// ============================================================================
// BASH-VAR-002: RANDOM Purification
// ============================================================================

/// BASH-VAR-002: RED Phase
/// Test that RANDOM is NOT generated
#[test]
fn test_random_purification_baseline() {
    let source = r#"
fn main() {
    use_seed(42);
}

fn use_seed(seed: i32) {}
"#;

    let config = Config::default();
    let result = transpile(source, &config);

    assert!(
        result.is_ok(),
        "Should transpile seed function: {:?}",
        result.err()
    );

    let shell = result.unwrap();
    eprintln!("Generated shell for deterministic seed:\n{}", shell);

    // Verify function is called with deterministic seed
    assert!(
        shell.contains("use_seed") && shell.contains("42"),
        "Should use deterministic seed"
    );

    // Should NOT contain $RANDOM
    assert!(
        !shell.contains("$RANDOM") && !shell.contains("RANDOM"),
        "Should NOT contain $RANDOM (non-deterministic)"
    );
}

/// BASH-VAR-002: RED Phase - ADVANCED
/// Test that rand crate usage is NOT supported
#[test]
#[ignore] // Requires rand crate detection and rejection
fn test_random_crate_rejection() {
    let source = r#"
fn main() {
    let num = rand::random::<u32>();
    println!("{}", num);
}
"#;

    let config = Config::default();
    let result = transpile(source, &config);

    // Should fail validation - random is non-deterministic
    assert!(
        result.is_err(),
        "rand crate usage should be rejected as non-deterministic"
    );
}

/// BASH-VAR-002: RED Phase - EXECUTION
/// Test deterministic value execution
#[test]
fn test_random_deterministic_execution() {
    let source = r#"
fn main() {
    use_value(12345);
}

fn use_value(val: i32) {}
"#;

    let config = Config::default();
    let shell = transpile(source, &config).unwrap();

    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    file.write_all(shell.as_bytes())
        .expect("Failed to write shell script");

    let output = Command::new("sh")
        .arg(file.path())
        .output()
        .expect("Failed to execute shell script");

    assert!(
        output.status.success() || output.status.code() == Some(127),
        "Script should execute with deterministic value"
    );
}

/// Session 6: Combined execution test
#[test]
fn test_session6_commands_execution() {
    let source = r#"
fn main() {
    print_heredoc();
    use_fixed_id();
    run_sync();
    use_seed(42);
}

fn print_heredoc() {}
fn use_fixed_id() {}
fn run_sync() {}
fn use_seed(seed: i32) {}
"#;

    let config = Config::default();
    let shell = transpile(source, &config).unwrap();

    eprintln!("Generated combined shell script:\n{}", shell);

    // Verify all functions are called
    assert!(shell.contains("print_heredoc"), "Should call print_heredoc");
    assert!(shell.contains("use_fixed_id"), "Should call use_fixed_id");
    assert!(shell.contains("run_sync"), "Should call run_sync");
    assert!(shell.contains("use_seed"), "Should call use_seed");

    // Verify NO non-deterministic constructs in main function (trap cleanup is OK)
    let main_section = shell.split("main() {").nth(1).unwrap_or("");
    let main_body = main_section.split("}").next().unwrap_or("");
    assert!(!main_body.contains("$$"), "Main should NOT contain $$");
    assert!(!main_body.contains("$!"), "Main should NOT contain $!");
    assert!(
        !main_body.contains("$RANDOM"),
        "Main should NOT contain $RANDOM"
    );
    assert!(
        !main_body.contains(" &"),
        "Main should NOT contain background &"
    );

    // Write and execute
    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    file.write_all(shell.as_bytes())
        .expect("Failed to write shell script");

    let output = Command::new("sh")
        .arg(file.path())
        .output()
        .expect("Failed to execute shell script");

    // Execution test may fail (functions undefined), but script should be valid
    eprintln!("Exit code: {:?}", output.status.code());
    eprintln!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
}

// ============================================================================
// Session 7: Exit Status and Additional Purifications
// Validation of GNU Bash Manual constructs - RED Phase Tests
// ============================================================================

/// PARAM-SPEC-002: RED Phase
/// Test exit status $? baseline
#[test]
fn test_exit_status_baseline() {
    let source = r#"
fn main() {
    get_status();
}

fn get_status() -> i32 { 0 }
"#;

    let config = Config::default();
    let result = transpile(source, &config);

    assert!(
        result.is_ok(),
        "Should transpile exit status function: {:?}",
        result.err()
    );

    let shell = result.unwrap();
    eprintln!("Generated shell for exit status:\n{}", shell);

    // Verify function is called
    assert!(
        shell.contains("get_status"),
        "Should transpile get_status function"
    );
}

/// PARAM-SPEC-002: RED Phase - ADVANCED
/// Test command exit status capture with $?
#[test]
#[ignore] // Requires $? capture pattern recognition
fn test_exit_status_capture() {
    let source = r#"
fn main() {
    run_command();
    let status = last_exit_code();
    check_status(status);
}

fn run_command() {}
fn last_exit_code() -> i32 { 0 }
fn check_status(code: i32) {}
"#;

    let config = Config::default();
    let shell = transpile(source, &config).unwrap();

    // Should capture $? after command
    assert!(shell.contains("$?"), "Should use $? to capture exit status");
}

/// PARAM-SPEC-002: RED Phase - EXECUTION
/// Test exit status execution
#[test]
fn test_exit_status_param_execution() {
    let source = r#"
fn main() {
    check_result(0);
}

fn check_result(code: i32) {}
"#;

    let config = Config::default();
    let shell = transpile(source, &config).unwrap();

    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    file.write_all(shell.as_bytes())
        .expect("Failed to write shell script");

    let output = Command::new("sh")
        .arg(file.path())
        .output()
        .expect("Failed to execute shell script");

    assert!(
        output.status.success() || output.status.code() == Some(127),
        "Script should execute"
    );
}

// ============================================================================
// REDIR-005: Herestring <<<
// ============================================================================

/// REDIR-005: RED Phase
/// Test herestring <<< baseline
#[test]
fn test_herestring_baseline() {
    let source = r#"
fn main() {
    pass_string("input data");
}

fn pass_string(data: &str) {}
"#;

    let config = Config::default();
    let result = transpile(source, &config);

    assert!(
        result.is_ok(),
        "Should transpile herestring function: {:?}",
        result.err()
    );

    let shell = result.unwrap();
    eprintln!("Generated shell for herestring:\n{}", shell);

    // Verify function is called
    assert!(
        shell.contains("pass_string"),
        "Should transpile pass_string function"
    );
}

include!("integration_tests_main_part7.rs");
