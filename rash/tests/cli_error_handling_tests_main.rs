fn test_check_subcommand_valid_file() {
    let rust_code = r#"
        fn main() {
            let x = 42;
            println!("Hello");
        }
    "#;

    let mut file = NamedTempFile::new().unwrap();
    file.write_all(rust_code.as_bytes()).unwrap();

    assert_cmd::cargo_bin_cmd!("bashrs")
        .arg("check")
        .arg(file.path())
        .assert()
        .success();
}

#[test]
fn test_check_subcommand_invalid_file() {
    // Use a construct that still fails validation (unsafe block inside function)
    let rust_code = r#"
        fn main() {
            unsafe { let ptr = std::ptr::null::<i32>(); }
        }
    "#;

    let mut file = NamedTempFile::new().unwrap();
    file.write_all(rust_code.as_bytes()).unwrap();

    assert_cmd::cargo_bin_cmd!("bashrs")
        .arg("check")
        .arg(file.path())
        .assert()
        .failure();
}

// ============================================================================
// File Not Found Tests
// ============================================================================

#[test]
fn test_missing_input_file_error() {
    assert_cmd::cargo_bin_cmd!("bashrs")
        .arg("build")
        .arg("nonexistent_file.rs")
        .assert()
        .failure()
        .code(2) // IO errors return 2
        .stderr(predicate::str::contains("error").or(predicate::str::contains("Error")))
        .stderr(predicate::str::contains("No such file or directory"));
}

// ============================================================================
// Error Message Quality Tests
// ============================================================================

#[test]
fn test_error_message_quality_baseline() {
    // Only test constructs that actually produce errors
    let unsupported_features = vec![
        ("async", "async fn test() { let x = foo().await; }"),
        ("unsafe", "unsafe { let ptr = std::ptr::null::<i32>(); }"),
    ];

    for (feature, code) in unsupported_features {
        let full_code = format!("fn main() {{ {} }}", code);
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(full_code.as_bytes()).unwrap();

        let output = assert_cmd::cargo_bin_cmd!("bashrs")
            .arg("build")
            .arg(file.path())
            .output()
            .unwrap();

        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let combined = format!("{}{}", stdout, stderr);
        let quality = ErrorMessageQuality::from_stderr(&combined);

        // Verify we get SOME error indication
        assert!(
            quality.has_error_prefix || !output.status.success(),
            "Error message for '{}' should have error prefix or non-zero exit. Quality: {:?}",
            feature,
            quality
        );
    }
}

// ============================================================================
// Multiple Error Reporting Tests
// ============================================================================

#[test]
fn test_multiple_errors_detected() {
    // Use constructs that still fail: async inside main, unsafe block
    let rust_code = r#"
        fn main() {
            let data = fetch_data().await;
            unsafe { let p = std::ptr::null::<i32>(); }
        }
    "#;

    let mut file = NamedTempFile::new().unwrap();
    file.write_all(rust_code.as_bytes()).unwrap();

    let output = assert_cmd::cargo_bin_cmd!("bashrs")
        .arg("build")
        .arg(file.path())
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should detect at least one error
    assert!(
        stderr.contains("error")
            || stderr.contains("Error")
            || stdout.contains("error")
            || !output.status.success(),
        "Should detect errors. stderr: {}, stdout: {}",
        stderr,
        stdout
    );

    assert!(
        !output.status.success(),
        "Should fail with non-zero exit code"
    );
}

#[test]
fn test_macro_definition_error_message() {
    let rust_code = r#"
        macro_rules! my_macro {
            () => { println!("hello"); }
        }

        fn main() {
            my_macro!();
        }
    "#;

    let mut file = NamedTempFile::new().unwrap();
    file.write_all(rust_code.as_bytes()).unwrap();

    let output = assert_cmd::cargo_bin_cmd!("bashrs")
        .arg("build")
        .arg(file.path())
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);

    // macro_rules! is detected as unsupported (not a function definition)
    assert!(
        !output.status.success(),
        "macro_rules! should fail. Exit: {:?}",
        output.status.code(),
    );
    assert!(
        stderr.contains("error") || stderr.contains("Only functions"),
        "Should mention error or function restriction. Got: {}",
        stderr
    );
}
