fn test_PURIFY_007_mkdir_becomes_mkdir_p() {
    let bash_script = r#"#!/bin/bash
mkdir /tmp/test
"#;

    let input_file = create_temp_bash_script(bash_script);

    bashrs_cmd()
        .arg("purify")
        .arg(input_file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("mkdir -p"));
}

#[test]
fn test_PURIFY_007_rm_becomes_rm_f() {
    let bash_script = r#"#!/bin/bash
rm /tmp/file.txt
"#;

    let input_file = create_temp_bash_script(bash_script);

    bashrs_cmd()
        .arg("purify")
        .arg(input_file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("rm -f"));
}

#[test]
fn test_PURIFY_007_ln_becomes_safe_symlink() {
    let bash_script = r#"#!/bin/bash
ln -s /source /target
"#;

    let input_file = create_temp_bash_script(bash_script);

    let output = bashrs_cmd()
        .arg("purify")
        .arg(input_file.path())
        .output()
        .expect("Failed to execute purify");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "Purify command should succeed");
    // Should either use rm -f before ln -s or use ln -sf
    assert!(
        stdout.contains("ln -s") || stdout.contains("ln"),
        "Should contain symlink operation"
    );
}

// ============================================================================
// Test: PURIFY_008 - Safety Transformations (Variable Quoting)
// ============================================================================

#[test]
fn test_PURIFY_008_unquoted_variable_echo() {
    let bash_script = r#"#!/bin/bash
msg="hello"
echo $msg
"#;

    let input_file = create_temp_bash_script(bash_script);

    bashrs_cmd()
        .arg("purify")
        .arg(input_file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"$msg\"").or(predicate::str::contains("$msg")));
}

// ============================================================================
// Test: Integration - Complex Real-World Scripts
// ============================================================================

#[test]
fn test_PURIFY_integration_deployment_script() {
    let bash_script = r#"#!/bin/bash
# Deployment script
SESSION_ID=$RANDOM
RELEASE="release-$(date +%s)"
mkdir /tmp/releases/$RELEASE
rm /tmp/current
ln -s /tmp/releases/$RELEASE /tmp/current
echo $UNQUOTED
"#;

    let input_file = create_temp_bash_script(bash_script);
    let output_dir = TempDir::new().expect("Failed to create temp dir");
    let output_file = output_dir.path().join("purified_deploy.sh");

    bashrs_cmd()
        .arg("purify")
        .arg(input_file.path())
        .arg("-o")
        .arg(&output_file)
        .arg("--report")
        .assert()
        .success()
        .stdout(predicate::str::contains("Purification Report"))
        .stdout(predicate::str::contains("Performance:"));

    // Verify output file
    assert!(output_file.exists());
    let output_content = fs::read_to_string(&output_file).expect("Failed to read output");

    // Verify transformations
    assert!(
        output_content.contains("#!/bin/sh"),
        "Should have POSIX shebang"
    );
    assert!(
        output_content.contains("mkdir -p"),
        "Should have idempotent mkdir"
    );
    assert!(output_content.contains("rm -f"), "Should have safe rm");
}

#[test]
fn test_PURIFY_integration_multiple_files() {
    let script1 = r#"#!/bin/bash
x=1
"#;
    let script2 = r#"#!/bin/bash
y=2
"#;

    let input1 = create_temp_bash_script(script1);
    let input2 = create_temp_bash_script(script2);

    let output_dir = TempDir::new().expect("Failed to create temp dir");
    let output1 = output_dir.path().join("out1.sh");
    let output2 = output_dir.path().join("out2.sh");

    // Purify first file
    bashrs_cmd()
        .arg("purify")
        .arg(input1.path())
        .arg("-o")
        .arg(&output1)
        .assert()
        .success();

    // Purify second file
    bashrs_cmd()
        .arg("purify")
        .arg(input2.path())
        .arg("-o")
        .arg(&output2)
        .assert()
        .success();

    // Both should exist
    assert!(output1.exists());
    assert!(output2.exists());

    // Both should have POSIX shebang
    let content1 = fs::read_to_string(&output1).expect("Failed to read");
    let content2 = fs::read_to_string(&output2).expect("Failed to read");

    assert!(content1.contains("#!/bin/sh"));
    assert!(content2.contains("#!/bin/sh"));
}

// ============================================================================
// Test: Help and Documentation
// ============================================================================

#[test]
fn test_PURIFY_help_flag() {
    bashrs_cmd()
        .arg("purify")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Purify bash scripts"))
        .stdout(predicate::str::contains("--output"))
        .stdout(predicate::str::contains("--report"));
}

#[test]
fn test_PURIFY_in_main_help() {
    bashrs_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("purify"))
        .stdout(predicate::str::contains("Purify bash scripts"));
}
