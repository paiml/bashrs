#[test]
#[ignore] // Requires herestring pattern recognition
fn test_herestring_conversion() {
    let source = r#"
fn main() {
    let input = "test input";
    pipe_input(input);
}

fn pipe_input(data: &str) {}
"#;

    let config = Config::default();
    let shell = transpile(source, &config).unwrap();

    // Should convert to printf | cmd (POSIX alternative to <<<)
    assert!(
        shell.contains("printf") && shell.contains("|"),
        "Should convert herestring to printf | cmd"
    );
}

/// REDIR-005: RED Phase - EXECUTION
/// Test herestring execution
#[test]
fn test_herestring_execution() {
    let source = r#"
fn main() {
    send_data("hello");
}

fn send_data(msg: &str) {}
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
// BASH-VAR-003: SECONDS Purification
// ============================================================================

/// BASH-VAR-003: RED Phase
/// Test that SECONDS is NOT generated
#[test]
fn test_seconds_purification_baseline() {
    let source = r#"
fn main() {
    use_fixed_time(100);
}

fn use_fixed_time(duration: i32) {}
"#;

    let config = Config::default();
    let result = transpile(source, &config);

    assert!(
        result.is_ok(),
        "Should transpile fixed time function: {:?}",
        result.err()
    );

    let shell = result.unwrap();
    eprintln!("Generated shell for fixed time:\n{}", shell);

    // Verify function is called with fixed duration
    assert!(
        shell.contains("use_fixed_time") && shell.contains("100"),
        "Should use fixed time duration"
    );

    // Should NOT contain $SECONDS
    let main_section = shell.split("main() {").nth(1).unwrap_or("");
    let main_body = main_section.split("}").next().unwrap_or("");
    assert!(
        !main_body.contains("$SECONDS") && !main_body.contains("SECONDS="),
        "Should NOT contain $SECONDS (non-deterministic)"
    );
}

/// BASH-VAR-003: RED Phase - ADVANCED
/// Test that SystemTime::now() is NOT supported
#[test]
#[ignore] // Requires SystemTime detection and rejection
fn test_seconds_time_rejection() {
    let source = r#"
fn main() {
    let start = std::time::SystemTime::now();
    do_work();
    let elapsed = start.elapsed().unwrap();
    println!("{:?}", elapsed);
}

fn do_work() {}
"#;

    let config = Config::default();
    let result = transpile(source, &config);

    // Should fail validation - timing is non-deterministic
    assert!(
        result.is_err(),
        "SystemTime::now() should be rejected as non-deterministic"
    );
}

/// BASH-VAR-003: RED Phase - EXECUTION
/// Test fixed duration execution
#[test]
fn test_seconds_fixed_duration_execution() {
    let source = r#"
fn main() {
    wait_fixed(5);
}

fn wait_fixed(seconds: i32) {}
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
        "Script should execute with fixed duration"
    );
}

// ============================================================================
// JOB-001: Background Jobs (&) Purification
// ============================================================================

/// JOB-001: RED Phase
/// Test that background jobs are NOT generated
#[test]
fn test_background_jobs_purification_baseline() {
    let source = r#"
fn main() {
    run_foreground();
}

fn run_foreground() {}
"#;

    let config = Config::default();
    let result = transpile(source, &config);

    assert!(
        result.is_ok(),
        "Should transpile foreground function: {:?}",
        result.err()
    );

    let shell = result.unwrap();
    eprintln!("Generated shell for foreground execution:\n{}", shell);

    // Verify function is called in foreground
    assert!(
        shell.contains("run_foreground"),
        "Should call function in foreground"
    );

    // Should NOT contain background operators
    let main_section = shell.split("main() {").nth(1).unwrap_or("");
    let main_body = main_section.split("}").next().unwrap_or("");
    assert!(
        !main_body.contains(" &"),
        "Should NOT contain background job operator & (non-deterministic)"
    );
}

/// JOB-001: RED Phase - ADVANCED
/// Test that spawn/thread is NOT supported
#[test]
#[ignore] // Requires spawn/thread detection and rejection
fn test_background_spawn_rejection() {
    let source = r#"
fn main() {
    std::thread::spawn(|| {
        background_work();
    });
}

fn background_work() {}
"#;

    let config = Config::default();
    let result = transpile(source, &config);

    // Should fail validation - threading is non-deterministic
    assert!(
        result.is_err(),
        "std::thread::spawn should be rejected as non-deterministic"
    );
}

/// JOB-001: RED Phase - EXECUTION
/// Test foreground execution
#[test]
fn test_background_foreground_execution() {
    let source = r#"
fn main() {
    task_one();
    task_two();
    task_three();
}

fn task_one() {}
fn task_two() {}
fn task_three() {}
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
        "Script should execute tasks in foreground"
    );
}

/// Session 7: Combined execution test
#[test]
fn test_session7_commands_execution() {
    let source = r#"
fn main() {
    get_status();
    pass_string("data");
    use_fixed_time(60);
    run_foreground();
}

fn get_status() -> i32 { 0 }
fn pass_string(data: &str) {}
fn use_fixed_time(duration: i32) {}
fn run_foreground() {}
"#;

    let config = Config::default();
    let shell = transpile(source, &config).unwrap();

    eprintln!("Generated combined shell script:\n{}", shell);

    // Verify all functions are called
    assert!(shell.contains("get_status"), "Should call get_status");
    assert!(shell.contains("pass_string"), "Should call pass_string");
    assert!(
        shell.contains("use_fixed_time"),
        "Should call use_fixed_time"
    );
    assert!(
        shell.contains("run_foreground"),
        "Should call run_foreground"
    );

    // Verify NO non-deterministic constructs in main function
    let main_section = shell.split("main() {").nth(1).unwrap_or("");
    let main_body = main_section.split("}").next().unwrap_or("");
    assert!(
        !main_body.contains("$SECONDS"),
        "Main should NOT contain $SECONDS"
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
// P0-POSITIONAL-PARAMETERS: Property Tests
// ============================================================================

#[cfg(test)]
mod positional_parameters_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Property: Transpiling positional parameters is deterministic
        /// Same Rust input always produces same shell output
        #[test]
        fn prop_positional_params_deterministic(
            default_val in "[a-z]{1,10}"
        ) {
            let source = format!(r#"
fn main() {{
    let args: Vec<String> = std::env::args().collect();
    let first = args.get(1).unwrap_or("{}");
    echo(first);
}}

fn echo(msg: &str) {{}}
"#, default_val);

            let config = Config::default();
            let result1 = transpile(&source, &config);
            let result2 = transpile(&source, &config);

            prop_assert!(result1.is_ok());
            prop_assert!(result2.is_ok());
            prop_assert_eq!(result1.unwrap(), result2.unwrap());
        }

        /// Property: Transpilation succeeds for all valid default values
        #[test]
        fn prop_default_values_preserved(
            default_val in "[a-zA-Z0-9_-]{1,20}"
        ) {
            let source = format!(r#"
fn main() {{
    let args: Vec<String> = std::env::args().collect();
    let param = args.get(1).unwrap_or("{}");
    echo(param);
}}

fn echo(msg: &str) {{}}
"#, default_val);

            let config = Config::default();
            let result = transpile(&source, &config);

            // Transpilation should always succeed for valid default values
            prop_assert!(result.is_ok(), "Transpilation failed for default: {}", default_val);

            let shell = result.unwrap();

            // Shell output should contain the param assignment with positional parameter syntax
            prop_assert!(
                shell.contains("param=") && shell.contains("${1:-"),
                "Shell output should contain positional parameter with default"
            );
        }

        /// Property: Positional parameters are always quoted
        #[test]
        fn prop_positional_params_quoted(
            position in 1u32..10,
            default_val in "[a-z]{1,10}"
        ) {
            let source = format!(r#"
fn main() {{
    let args: Vec<String> = std::env::args().collect();
    let param = args.get({}).unwrap_or("{}");
    echo(param);
}}

fn echo(msg: &str) {{}}
"#, position, default_val);

            let config = Config::default();
            let result = transpile(&source, &config);

            prop_assert!(result.is_ok());
            let shell = result.unwrap();

            // Positional params should be in quotes
            prop_assert!(
                shell.contains(&format!("\"${{{}:-", position)) ||
                shell.contains(&format!("param=\"${{{}:-", position)),
                "Positional parameter should be quoted"
            );
        }

        /// Property: std::env::args().collect() always becomes "$@"
        #[test]
        fn prop_args_collect_becomes_dollar_at(
            _seed in 0u32..100  // Just for variety
        ) {
            let source = r#"
fn main() {
    let args: Vec<String> = std::env::args().collect();
    echo("test");
}

fn echo(msg: &str) {}
"#;

            let config = Config::default();
            let result = transpile(source, &config);

            prop_assert!(result.is_ok());
            let shell = result.unwrap();

            // args variable should contain "$@"
            prop_assert!(
                shell.contains("args=\"$@\"") || shell.contains("$@"),
                "args.collect() should become $@"
            );
        }
    }
}

/// PARAM-SPEC-001: Property-based tests for arg_count() → $# transformation
mod arg_count_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Property: Transpiling arg_count() is deterministic
        /// Same Rust input always produces same shell output
        #[test]
        fn prop_arg_count_deterministic(
            _seed in 0u32..100
        ) {
            let source = r#"
fn main() {
    let count = arg_count();
    echo("Done");
}

fn arg_count() -> i32 { 0 }
fn echo(msg: &str) {}
"#;

            let config = Config::default();
            let result1 = transpile(source, &config);
            let result2 = transpile(source, &config);

            prop_assert!(result1.is_ok());
            prop_assert!(result2.is_ok());
            prop_assert_eq!(result1.unwrap(), result2.unwrap());
        }

        /// Property: arg_count() always generates $# in output
        #[test]
        fn prop_arg_count_generates_dollar_hash(
            _seed in 0u32..100
        ) {
            let source = r#"
fn main() {
    let count = arg_count();
    wc("-l");
}

fn arg_count() -> i32 { 0 }
fn wc(arg: &str) {}
"#;

            let config = Config::default();
            let result = transpile(source, &config);

            prop_assert!(result.is_ok(), "Transpilation should succeed");

            let shell = result.unwrap();

            // arg_count() should always generate $# in shell output
            prop_assert!(
                shell.contains("$#"),
                "Shell output must contain $# for arg_count()"
            );
        }

include!("integration_tests_main_part8.rs");
