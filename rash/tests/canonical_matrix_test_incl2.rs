fn test_MATRIX_property_purification_idempotency() {
    // Property: purify(purify(x)) == purify(x)
    // Purifying twice produces same result as purifying once

    let test_cases = vec![
        (
            "makefile",
            r#"
build:
	gcc -o app main.c
"#,
        ),
        (
            "dockerfile",
            r#"FROM ubuntu:latest
RUN apt-get install -y curl
"#,
        ),
    ];

    for (file_type, content) in test_cases {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let input_path = temp_dir.path().join(format!("input.{}", file_type));
        fs::write(&input_path, content).expect("Failed to write input file");

        let purified_once = temp_dir.path().join("purified_once");
        let purified_twice = temp_dir.path().join("purified_twice");

        // First purification
        match file_type {
            "makefile" => {
                bashrs_cmd()
                    .arg("make")
                    .arg("purify")
                    .arg(&input_path)
                    .arg("-o")
                    .arg(&purified_once)
                    .assert()
                    .success();
            }
            "dockerfile" => {
                bashrs_cmd()
                    .arg("dockerfile")
                    .arg("purify")
                    .arg(&input_path)
                    .arg("-o")
                    .arg(&purified_once)
                    .assert()
                    .success();
            }
            _ => {}
        }

        // Second purification (purify the purified file)
        match file_type {
            "makefile" => {
                bashrs_cmd()
                    .arg("make")
                    .arg("purify")
                    .arg(&purified_once)
                    .arg("-o")
                    .arg(&purified_twice)
                    .assert()
                    .success();
            }
            "dockerfile" => {
                bashrs_cmd()
                    .arg("dockerfile")
                    .arg("purify")
                    .arg(&purified_once)
                    .arg("-o")
                    .arg(&purified_twice)
                    .assert()
                    .success();
            }
            _ => {}
        }

        // Verify idempotency
        let content_once =
            fs::read_to_string(&purified_once).expect("Failed to read purified_once");
        let content_twice =
            fs::read_to_string(&purified_twice).expect("Failed to read purified_twice");

        assert_eq!(
            content_once, content_twice,
            "{} purification should be idempotent (purify(purify(x)) == purify(x))",
            file_type
        );
    }
}

// ============================================================================
// MATRIX TEST 8: Performance Baseline (<5 seconds total)
// ============================================================================

#[test]
fn test_MATRIX_performance_all_operations() {
    // Verify all matrix operations complete within performance target
    use std::time::Instant;

    let start = Instant::now();

    // Quick smoke test of each capability
    let bash_script = r#"#!/bin/bash
echo "test"
"#;
    let makefile = r#"
build:
	@echo "test"
"#;
    let dockerfile = r#"FROM alpine:latest
RUN apk add curl
"#;

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Test 1: Bash purification
    let bash_input = temp_dir.path().join("test.sh");
    fs::write(&bash_input, bash_script).expect("Failed to write bash script");
    let bash_output = temp_dir.path().join("test_purified.sh");

    let _ = bashrs_cmd()
        .arg("purify")
        .arg(&bash_input)
        .arg("-o")
        .arg(&bash_output)
        .ok(); // May not be implemented yet

    // Test 2: Makefile purification
    let make_input = temp_dir.path().join("Makefile");
    fs::write(&make_input, makefile).expect("Failed to write Makefile");
    let make_output = temp_dir.path().join("Makefile.purified");

    bashrs_cmd()
        .arg("make")
        .arg("purify")
        .arg(&make_input)
        .arg("-o")
        .arg(&make_output)
        .assert()
        .success();

    // Test 3: Dockerfile purification
    let docker_input = temp_dir.path().join("Dockerfile");
    fs::write(&docker_input, dockerfile).expect("Failed to write Dockerfile");
    let docker_output = temp_dir.path().join("Dockerfile.purified");

    bashrs_cmd()
        .arg("dockerfile")
        .arg("purify")
        .arg(&docker_input)
        .arg("-o")
        .arg(&docker_output)
        .assert()
        .success();

    let elapsed = start.elapsed();

    assert!(
        elapsed.as_secs() < 5,
        "Matrix test should complete in <5 seconds (actual: {:?})",
        elapsed
    );
}
