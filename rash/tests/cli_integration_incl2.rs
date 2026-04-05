fn test_CLI_007_compile_self_extracting() {
    let rust_code = r#"
fn main() {
    println!("Hello");
}
"#;

    let input_file = create_temp_rust_file(rust_code);
    let output_dir = TempDir::new().expect("Failed to create temp dir");
    let output_file = output_dir.path().join("installer.sh");

    bashrs_cmd()
        .arg("compile")
        .arg(input_file.path())
        .arg("--output")
        .arg(&output_file)
        .arg("--self-extracting")
        .assert()
        .success();
}

#[test]
fn test_CLI_007_compile_with_runtime_dash() {
    let rust_code = r#"
fn main() {
    println!("Hello");
}
"#;

    let input_file = create_temp_rust_file(rust_code);
    let output_dir = TempDir::new().expect("Failed to create temp dir");
    let output_file = output_dir.path().join("binary");

    bashrs_cmd()
        .arg("compile")
        .arg(input_file.path())
        .arg("--output")
        .arg(&output_file)
        .arg("--runtime")
        .arg("dash")
        .assert()
        .success();
}

// ============================================================================
// Test: CLI_008 - Lint Command
// ============================================================================

#[test]
fn test_CLI_008_lint_shell_script() {
    let shell_script = r#"#!/bin/bash
x=$RANDOM  # Non-deterministic
echo $x
"#;

    let input_file = create_temp_shell_file(shell_script);

    bashrs_cmd()
        .arg("lint")
        .arg(input_file.path())
        .assert()
        .success();
}

#[test]
fn test_CLI_008_lint_rust_source() {
    let rust_code = r#"
fn main() {
    let x = 42;
}
"#;

    let input_file = create_temp_rust_file(rust_code);

    bashrs_cmd()
        .arg("lint")
        .arg(input_file.path())
        .assert()
        .success();
}

#[test]
fn test_CLI_008_lint_with_json_format() {
    let shell_script = "#!/bin/sh\necho hello\n";
    let input_file = create_temp_shell_file(shell_script);

    bashrs_cmd()
        .arg("lint")
        .arg(input_file.path())
        .arg("--format")
        .arg("json")
        .assert()
        .success();
}

#[test]
fn test_CLI_008_lint_with_autofix() {
    let shell_script = r#"#!/bin/bash
x=$RANDOM
"#;
    let input_file = create_temp_shell_file(shell_script);

    bashrs_cmd()
        .arg("lint")
        .arg(input_file.path())
        .arg("--fix")
        .assert()
        .success();
}

#[test]
fn test_CLI_008_lint_nonexistent_file() {
    bashrs_cmd()
        .arg("lint")
        .arg("nonexistent.sh")
        .assert()
        .failure()
        .stderr(predicate::str::contains("No such file"));
}

// ============================================================================
// Test: CLI_009 - Make Parse Command
// ============================================================================

#[test]
fn test_CLI_009_make_parse_basic() {
    let makefile = r#"
.PHONY: clean

all: main.o
	gcc -o program main.o

clean:
	rm -f *.o program
"#;

    let input_file = create_temp_makefile(makefile);

    bashrs_cmd()
        .arg("make")
        .arg("parse")
        .arg(input_file.path())
        .assert()
        .success();
}

#[test]

include!("cli_integration_incl2_incl2.rs");
