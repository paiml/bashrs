fn test_CLI_MAKE_005_purify_report_json_format() {
    ensure_fixtures_dir();
    let makefile = "tests/fixtures/cli_report_json.mk";
    fs::write(makefile, "FILES := $(wildcard *.c)").unwrap();

    bashrs_cmd()
        .arg("make")
        .arg("purify")
        .arg("--report")
        .arg("--format")
        .arg("json")
        .arg(makefile)
        .assert()
        .success()
        .stdout(predicate::str::contains("transformations_applied"))
        .stdout(predicate::str::contains("{"));

    cleanup(makefile);
}

#[test]
fn test_CLI_MAKE_005_purify_report_no_changes() {
    ensure_fixtures_dir();
    let makefile = "tests/fixtures/cli_report_clean.mk";
    fs::write(makefile, "FILES := $(sort $(wildcard *.c))").unwrap();

    bashrs_cmd()
        .arg("make")
        .arg("purify")
        .arg("--report")
        .arg(makefile)
        .assert()
        .success()
        .stdout(predicate::str::contains("Transformations Applied: 0"));

    cleanup(makefile);
}

// ============================================================================
// RED-006: Error handling tests
// ============================================================================

#[test]
fn test_CLI_MAKE_006_parse_invalid_makefile() {
    ensure_fixtures_dir();
    let makefile = "tests/fixtures/cli_invalid.mk";
    // Invalid syntax: completely malformed
    // Note: Parser is lenient and returns empty AST rather than failing
    fs::write(makefile, "this is not a makefile at all!!! $$$$").unwrap();

    bashrs_cmd()
        .arg("make")
        .arg("parse")
        .arg(makefile)
        .assert()
        .success()
        .stdout(predicate::str::contains("items: []"));

    cleanup(makefile);
}

#[test]
fn test_CLI_MAKE_006_parse_nonexistent_file() {
    bashrs_cmd()
        .arg("make")
        .arg("parse")
        .arg("tests/fixtures/nonexistent.mk")
        .assert()
        .failure()
        .stderr(predicate::str::contains("error")); // Error message contains lowercase "error"
}

#[test]
fn test_CLI_MAKE_006_purify_invalid_makefile() {
    ensure_fixtures_dir();
    let makefile = "tests/fixtures/cli_purify_invalid.mk";
    // Note: Parser is lenient and returns empty AST rather than failing
    fs::write(makefile, "completely invalid !!!! $$$$ not a makefile").unwrap();

    bashrs_cmd()
        .arg("make")
        .arg("purify")
        .arg(makefile)
        .assert()
        .success();

    cleanup(makefile);
}

// ============================================================================
// Additional edge case tests
// ============================================================================

#[test]
fn test_CLI_MAKE_007_purify_multiple_wildcards() {
    ensure_fixtures_dir();
    let makefile = "tests/fixtures/cli_multi_wildcard.mk";
    fs::write(
        makefile,
        "SOURCES := $(wildcard src/*.c)\nHEADERS := $(wildcard inc/*.h)\nOBJECTS := $(wildcard obj/*.o)",
    )
    .unwrap();

    let output = "tests/fixtures/cli_multi_wildcard_out.mk";
    bashrs_cmd()
        .arg("make")
        .arg("purify")
        .arg("--fix")
        .arg("-o")
        .arg(output)
        .arg(makefile)
        .assert()
        .success();

    // Verify all wildcards wrapped
    let content = fs::read_to_string(output).unwrap();
    assert!(content.contains("$(sort $(wildcard src/*.c))"));
    assert!(content.contains("$(sort $(wildcard inc/*.h))"));
    assert!(content.contains("$(sort $(wildcard obj/*.o))"));

    cleanup(makefile);
    cleanup(output);
}

#[test]
fn test_CLI_MAKE_008_purify_complex_makefile() {
    ensure_fixtures_dir();
    let makefile = "tests/fixtures/cli_complex.mk";
    fs::write(
        makefile,
        r#"# Build configuration
CC := gcc
CFLAGS := -O2 -Wall

SOURCES := $(wildcard src/*.c)
OBJECTS := $(SOURCES:.c=.o)

.PHONY: build clean

build: $(OBJECTS)
	$(CC) $(CFLAGS) -o myapp $(OBJECTS)

clean:
	rm -f $(OBJECTS) myapp
"#,
    )
    .unwrap();

    bashrs_cmd()
        .arg("make")
        .arg("purify")
        .arg("--report")
        .arg(makefile)
        .assert()
        .success()
        .stdout(predicate::str::contains("Transformations Applied"));

    cleanup(makefile);
}

// ============================================================================
// Integration test: End-to-end workflow
// ============================================================================

#[test]
fn test_CLI_MAKE_009_integration_full_workflow() {
    ensure_fixtures_dir();
    let input = "tests/fixtures/cli_integration.mk";
    let purified = "tests/fixtures/cli_integration_purified.mk";

    // Create a Makefile with multiple issues
    let content = r#"# Build System
CC := gcc
CFLAGS := -O2 -Wall

# Non-deterministic wildcard (will be purified)
SOURCES := $(wildcard src/*.c)
HEADERS := $(wildcard inc/*.h)
OBJECTS := $(wildcard obj/*.o)

.PHONY: build clean

build: $(OBJECTS)
	$(CC) $(CFLAGS) -o myapp $(OBJECTS)

clean:
	rm -f $(OBJECTS) myapp
"#;
    fs::write(input, content).unwrap();

    // Step 1: Parse to verify valid Makefile
    bashrs_cmd()
        .arg("make")
        .arg("parse")
        .arg(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("Target"));

    // Step 2: Purify with report
    let report_output = bashrs_cmd()
        .arg("make")
        .arg("purify")
        .arg("--report")
        .arg(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("Transformations Applied"))
        .get_output()
        .clone();

    let report = String::from_utf8(report_output.stdout).unwrap();
    assert!(
        report.contains("wildcard"),
        "Report should mention wildcard transformations"
    );

    // Step 3: Purify to output file
    bashrs_cmd()
        .arg("make")
        .arg("purify")
        .arg("--fix")
        .arg("-o")
        .arg(purified)
        .arg(input)
        .assert()
        .success();

    // Step 4: Verify purified content
    let purified_content = fs::read_to_string(purified).unwrap();
    assert!(
        purified_content.contains("$(sort $(wildcard src/*.c))"),
        "Should wrap wildcards with sort"
    );
    assert!(
        purified_content.contains("$(sort $(wildcard inc/*.h))"),
        "Should wrap all wildcards"
    );
    assert!(
        purified_content.contains("$(sort $(wildcard obj/*.o))"),
        "Should wrap all wildcards"
    );
    assert!(
        purified_content.contains(".PHONY: build clean"),
        "Should preserve .PHONY"
    );

    // Step 5: Parse purified file (should succeed)
    bashrs_cmd()
        .arg("make")
        .arg("parse")
        .arg(purified)
        .assert()
        .success();

    // Step 6: Re-purify should show 0 transformations (idempotent)
    let second_purify = bashrs_cmd()
        .arg("make")
        .arg("purify")
        .arg("--report")
        .arg(purified)
        .assert()
        .success()
        .get_output()
        .clone();

    let second_report = String::from_utf8(second_purify.stdout).unwrap();
    assert!(
        second_report.contains("Transformations Applied: 0"),
        "Should be idempotent"
    );

    cleanup(input);
    cleanup(purified);
}
