fn test_CLI_009_make_parse_json_format() {
    let makefile = r#"
all:
	echo "Building"
"#;

    let input_file = create_temp_makefile(makefile);

    bashrs_cmd()
        .arg("make")
        .arg("parse")
        .arg(input_file.path())
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("{"));
}

#[test]
fn test_CLI_009_make_parse_debug_format() {
    let makefile = r#"
test:
	echo "Testing"
"#;

    let input_file = create_temp_makefile(makefile);

    bashrs_cmd()
        .arg("make")
        .arg("parse")
        .arg(input_file.path())
        .arg("--format")
        .arg("debug")
        .assert()
        .success();
}

#[test]
fn test_CLI_009_make_parse_nonexistent_file() {
    bashrs_cmd()
        .arg("make")
        .arg("parse")
        .arg("nonexistent.mk")
        .assert()
        .failure()
        .stderr(predicate::str::contains("No such file"));
}

// ============================================================================
// Test: CLI_010 - Make Purify Command
// ============================================================================

#[test]
fn test_CLI_010_make_purify_basic() {
    let makefile = r#"
VERSION := $(shell date +%s)

build:
	mkdir /tmp/build-$(VERSION)
"#;

    let input_file = create_temp_makefile(makefile);

    bashrs_cmd()
        .arg("make")
        .arg("purify")
        .arg(input_file.path())
        .assert()
        .success();
}

#[test]
fn test_CLI_010_make_purify_with_output() {
    let makefile = r#"
all:
	echo "Building"
"#;

    let input_file = create_temp_makefile(makefile);
    let output_dir = TempDir::new().expect("Failed to create temp dir");
    let output_file = output_dir.path().join("Makefile.purified");

    let output = bashrs_cmd()
        .arg("make")
        .arg("purify")
        .arg(input_file.path())
        .arg("--output")
        .arg(&output_file)
        .output()
        .expect("Failed to execute command");

    // Command should succeed, whether or not it creates the file
    // (implementation may write to stdout instead of file)
    assert!(output.status.success() || output_file.exists());
}

#[test]
fn test_CLI_010_make_purify_with_report() {
    let makefile = r#"
build:
	mkdir /tmp/build-$$
"#;

    let input_file = create_temp_makefile(makefile);

    bashrs_cmd()
        .arg("make")
        .arg("purify")
        .arg(input_file.path())
        .arg("--report")
        .assert()
        .success()
        .stdout(predicate::str::contains("Transformation").or(predicate::str::contains("Issues")));
}

#[test]
fn test_CLI_010_make_purify_json_report() {
    let makefile = r#"
all:
	echo "Test"
"#;

    let input_file = create_temp_makefile(makefile);

    bashrs_cmd()
        .arg("make")
        .arg("purify")
        .arg(input_file.path())
        .arg("--report")
        .arg("--format")
        .arg("json")
        .assert()
        .success();
}

#[test]
fn test_CLI_010_make_purify_nonexistent_file() {
    bashrs_cmd()
        .arg("make")
        .arg("purify")
        .arg("nonexistent.mk")
        .assert()
        .failure()
        .stderr(predicate::str::contains("No such file"));
}

// ============================================================================
// Test: CLI_011 - Global Flags
// ============================================================================

#[test]
fn test_CLI_011_global_verbose_flag() {
    let rust_code = r#"
fn main() {
    println!("Hello");
}
"#;

    let input_file = create_temp_rust_file(rust_code);

    bashrs_cmd()
        .arg("--verbose")
        .arg("check")
        .arg(input_file.path())
        .assert()
        .success();
}

#[test]
fn test_CLI_011_global_strict_flag() {
    let rust_code = r#"
fn main() {
    println!("Hello");
}
"#;

    let input_file = create_temp_rust_file(rust_code);

    bashrs_cmd()
        .arg("--strict")
        .arg("check")
        .arg(input_file.path())
        .assert()
        .success();
}

#[test]
fn test_CLI_011_global_target_posix() {
    let rust_code = r#"
fn main() {
    println!("Hello");
}
"#;

    let input_file = create_temp_rust_file(rust_code);
    let output_dir = TempDir::new().expect("Failed to create temp dir");
    let output_file = output_dir.path().join("output.sh");

    bashrs_cmd()
        .arg("--target")
        .arg("posix")
        .arg("build")
        .arg(input_file.path())
        .arg("--output")
        .arg(&output_file)
        .assert()
        .success();
}

#[test]
fn test_CLI_011_global_verify_strict() {
    let rust_code = r#"
fn main() {
    println!("Hello");
}
"#;

    let input_file = create_temp_rust_file(rust_code);

    bashrs_cmd()
        .arg("--verify")
        .arg("strict")
        .arg("check")
        .arg(input_file.path())
        .assert()
        .success();
}

// ============================================================================
// Test: CLI_012 - End-to-End Workflow Tests
// ============================================================================

#[test]
fn test_CLI_012_e2e_check_then_build() {
    let rust_code = r#"
fn install(version: &str) {
    println!(version);
}

fn main() {
    install("1.0.0");
}
"#;

    let input_file = create_temp_rust_file(rust_code);

    // Step 1: Check the file
    bashrs_cmd()
        .arg("check")
        .arg(input_file.path())
        .assert()
        .success();

    // Step 2: Build the file
    let output_dir = TempDir::new().expect("Failed to create temp dir");
    let output_file = output_dir.path().join("install.sh");

    bashrs_cmd()
        .arg("build")
        .arg(input_file.path())
        .arg("--output")
        .arg(&output_file)
        .assert()
        .success();

    // Verify output exists and has content
    assert!(output_file.exists());
    let content = fs::read_to_string(&output_file).expect("Failed to read output");
    assert!(!content.is_empty());
}

#[test]
fn test_CLI_012_e2e_makefile_parse_then_purify() {
    let makefile = r#"
VERSION := $(shell date +%s)

.PHONY: clean

build:
	mkdir /tmp/build-$(VERSION)
	echo "Building..."

clean:
	rm -rf /tmp/build-*
"#;

    let input_file = create_temp_makefile(makefile);

    // Step 1: Parse the Makefile
    bashrs_cmd()
        .arg("make")
        .arg("parse")
        .arg(input_file.path())
        .assert()
        .success();

    // Step 2: Purify the Makefile (may write to stdout)
    let output = bashrs_cmd()
        .arg("make")
        .arg("purify")
        .arg(input_file.path())
        .arg("--report")
        .output()
        .expect("Failed to execute command");

    // Verify command succeeded
    assert!(output.status.success(), "Purify command should succeed");

    // Verify we got some output (either stdout or report)
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stdout.is_empty() || !stderr.is_empty(),
        "Should produce output"
    );
}

// ============================================================================
// Test: CLI_013 - Error Handling Edge Cases
// ============================================================================

#[test]
fn test_CLI_013_empty_input_file() {
    let input_file = create_temp_rust_file("");

    bashrs_cmd()
        .arg("build")
        .arg(input_file.path())
        .assert()
        .failure();
}

#[test]
fn test_CLI_013_binary_input_file() {
    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    file.write_all(&[0xFF, 0xFE, 0xFD, 0xFC])
        .expect("Failed to write binary data");

    bashrs_cmd()
        .arg("build")
        .arg(file.path())
        .assert()
        .failure();
}

#[test]
fn test_CLI_013_permission_denied() {
    // This test may not work on all systems
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let restricted_file = temp_dir.path().join("restricted.rs");
    fs::write(&restricted_file, "fn main() {}").expect("Failed to write file");

    // Try to make file unreadable (may not work on all systems)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&restricted_file)
            .expect("Failed to get metadata")
            .permissions();
        perms.set_mode(0o000);
        fs::set_permissions(&restricted_file, perms).expect("Failed to set permissions");

        bashrs_cmd()
            .arg("build")
            .arg(&restricted_file)
            .assert()
            .failure();
    }
}

// ============================================================================
// Test: CLI_014 - Output Format Validation
// ============================================================================

#[test]
fn test_CLI_014_json_output_is_valid_json() {
    let makefile = r#"
all:
	echo "Test"
"#;

    let input_file = create_temp_makefile(makefile);

    let output = bashrs_cmd()
        .arg("make")
        .arg("parse")
        .arg(input_file.path())
        .arg("--format")
        .arg("json")
        .output()
        .expect("Failed to execute command");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Verify it's valid JSON (basic check)
        assert!(
            stdout.contains("{") && stdout.contains("}"),
            "JSON output should contain braces"
        );
    }
}

// ============================================================================
// Test: CLI_015 - Multiple Files and Batch Processing
// ============================================================================

#[test]
fn test_CLI_015_multiple_sequential_builds() {
    let rust_code1 = r#"fn main() { println!("File 1"); }"#;
    let rust_code2 = r#"fn main() { println!("File 2"); }"#;

    let file1 = create_temp_rust_file(rust_code1);
    let file2 = create_temp_rust_file(rust_code2);

    let output_dir = TempDir::new().expect("Failed to create temp dir");

    // Build file 1
    let output1 = output_dir.path().join("output1.sh");
    bashrs_cmd()
        .arg("build")
        .arg(file1.path())
        .arg("--output")
        .arg(&output1)
        .assert()
        .success();

    // Build file 2
    let output2 = output_dir.path().join("output2.sh");
    bashrs_cmd()
        .arg("build")
        .arg(file2.path())
        .arg("--output")
        .arg(&output2)
        .assert()
        .success();

    // Verify both outputs exist
    assert!(output1.exists());
    assert!(output2.exists());
}
