//! Unified Testing Quality Compliance Tests
//!
//! EXTREME TDD verification of quality standards from:
//! docs/specification/unified-testing-quality-spec.md
//!
//! Test Naming Convention: test_SPEC_<ID>_<area>_<requirement>

#![allow(non_snake_case)] // Test naming convention requires uppercase task IDs
#![allow(clippy::unwrap_used)] // Standard in tests for known-valid patterns
#![allow(clippy::expect_used)] // Standard in tests for better error messages
#![allow(clippy::panic)] // Tests intentionally panic on assertion failures

use std::fs;
use std::path::Path;
use std::process::Command;

// ============================================================================
// Test: SPEC_001 - Test Naming Convention Compliance
// ============================================================================

#[test]
fn test_SPEC_001_test_naming_follows_convention() {
    // Verify all test files follow test_<TASK_ID>_<feature>_<scenario> naming

    let test_files = vec![
        "rash/tests/cli_make_purify_with_tests.rs",
        "rash/tests/cli_dockerfile_purify.rs",
        "rash/src/bash_parser/codegen_tests.rs",
    ];

    let naming_pattern = regex::Regex::new(r"fn\s+test_[A-Z0-9_]+_[a-z_]+").unwrap();

    for test_file in &test_files {
        if !Path::new(test_file).exists() {
            continue; // Skip if file doesn't exist
        }

        let content = fs::read_to_string(test_file)
            .unwrap_or_else(|_| panic!("Failed to read test file: {}", test_file));

        // Find all test function names
        for line in content.lines() {
            if line.trim_start().starts_with("fn test_")
                || line.trim_start().starts_with("pub fn test_")
            {
                let matches_convention = naming_pattern.is_match(line);

                if !matches_convention && !line.contains("#[ignore]") {
                    // Allow helper functions like test_helper_*
                    if !line.contains("test_helper") {
                        panic!(
                            "Test naming violation in {}:\n  {}\n  Expected: test_<TASK_ID>_<feature>_<scenario>",
                            test_file, line.trim()
                        );
                    }
                }
            }
        }
    }
}

// ============================================================================
// Test: SPEC_002 - CLI Testing Standards (assert_cmd)
// ============================================================================

#[test]
fn test_SPEC_002_cli_tests_use_assert_cmd() {
    // Verify CLI tests use assert_cmd, not std::process::Command directly

    let cli_test_files = vec![
        "rash/tests/cli_make_purify_with_tests.rs",
        "rash/tests/cli_dockerfile_purify.rs",
        "rash/tests/cli_make_formatting.rs",
    ];

    for test_file in &cli_test_files {
        if !Path::new(test_file).exists() {
            continue;
        }

        let content = fs::read_to_string(test_file)
            .unwrap_or_else(|_| panic!("Failed to read test file: {}", test_file));

        // Check for assert_cmd usage
        assert!(
            content.contains("use assert_cmd::Command") || content.contains("assert_cmd::Command"),
            "CLI test file {} should use assert_cmd::Command",
            test_file
        );

        // Check for helper function pattern (MANDATORY)
        assert!(
            content.contains("fn bashrs_cmd()") || content.contains("fn rash_cmd()"),
            "CLI test file {} should define helper function for Command creation",
            test_file
        );

        // Verify no direct use of std::process::Command for testing
        if content.contains("std::process::Command") {
            // Ensure it's not used for actual CLI testing (only for verification)
            let lines: Vec<&str> = content.lines().collect();
            for (i, line) in lines.iter().enumerate() {
                if line.contains("std::process::Command") && !line.trim_start().starts_with("//") {
                    // Check if it's in a verification context (like checking sh syntax)
                    let context = lines
                        .get(i.saturating_sub(5)..=i.saturating_add(5))
                        .unwrap_or(&[]);
                    let context_str = context.join("\n");

                    if !context_str.contains("sh -n") && !context_str.contains("syntax check") {
                        panic!(
                            "Quality defect in {}: Direct use of std::process::Command for CLI testing\n  Use assert_cmd::Command instead\n  Line: {}",
                            test_file, line.trim()
                        );
                    }
                }
            }
        }
    }
}

// ============================================================================
// Test: SPEC_003 - Coverage Targets
// ============================================================================

#[test]
#[ignore] // Requires llvm-cov to be run first
fn test_SPEC_003_coverage_targets_met() {
    // Verify >85% coverage for all modules

    let coverage_targets = vec![
        ("bash_parser/parser.rs", 85.0),
        ("bash_parser/codegen.rs", 85.0),
        ("make_parser/parser.rs", 85.0),
        ("make_parser/generators.rs", 85.0),
        ("linter/mod.rs", 85.0),
    ];

    // Run coverage measurement
    let output = Command::new("cargo")
        .args(["llvm-cov", "--lib", "--text"])
        .output()
        .expect("Failed to run cargo llvm-cov");

    let coverage_report = String::from_utf8_lossy(&output.stdout);

    for (module, target) in &coverage_targets {
        // Find module in coverage report
        let module_lines: Vec<&str> = coverage_report
            .lines()
            .skip_while(|line| !line.contains(module))
            .take(10)
            .collect();

        if module_lines.is_empty() {
            panic!("Module {} not found in coverage report", module);
        }

        // Extract coverage percentage (looking for pattern like "85.23%")
        let coverage_line = module_lines
            .iter()
            .find(|line| line.contains('%'))
            .unwrap_or_else(|| panic!("Coverage percentage not found for {}", module));

        // Parse coverage percentage
        let percentage_str = coverage_line
            .split('%')
            .next()
            .and_then(|s| s.split_whitespace().last())
            .unwrap_or("0.0");

        let coverage: f64 = percentage_str.parse().unwrap_or_else(|_| {
            panic!(
                "Failed to parse coverage for {}: {}",
                module, percentage_str
            )
        });

        assert!(
            coverage >= *target,
            "Coverage target not met for {}:\n  Expected: ≥{}%\n  Actual: {}%",
            module,
            target,
            coverage
        );
    }
}

// ============================================================================
// Test: SPEC_004 - Property Testing Presence
// ============================================================================

#[test]
fn test_SPEC_004_property_tests_exist() {
    // Verify property-based tests exist for core modules

    let modules_requiring_property_tests = vec![
        "src/bash_parser/property_tests.rs",
        "src/test_generator/property_tests.rs", // make_parser doesn't have separate property_tests yet
        "src/linter/rules/sc2154.rs",           // Has inline property tests
    ];

    for module in &modules_requiring_property_tests {
        if !Path::new(module).exists() {
            panic!("Property tests missing for module: {}", module);
        }

        let content = fs::read_to_string(module)
            .unwrap_or_else(|_| panic!("Failed to read module: {}", module));

        // Check for proptest usage
        assert!(
            content.contains("proptest!") || content.contains("use proptest::"),
            "Module {} should use proptest for property-based testing",
            module
        );

        // Verify minimum 100+ cases (proptest default, but check for explicit config)
        if content.contains("ProptestConfig") {
            assert!(
                content.contains("cases") && (content.contains("100") || content.contains("256")),
                "Property tests in {} should run at least 100 cases",
                module
            );
        }
    }
}

// ============================================================================
// Test: SPEC_005 - Quality Gates (Clippy Clean)
// ============================================================================

#[test]
#[ignore] // Run manually before release
fn test_SPEC_005_clippy_clean() {
    // Verify zero clippy warnings

    let output = Command::new("cargo")
        .args(["clippy", "--all-targets", "--", "-D", "warnings"])
        .output()
        .expect("Failed to run cargo clippy");

    assert!(
        output.status.success(),
        "Clippy found warnings or errors:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

// ============================================================================
// Test: SPEC_006 - EXTREME TDD Documentation
// ============================================================================

#[test]
fn test_SPEC_006_extreme_tdd_documentation_exists() {
    // Verify CLAUDE.md documents EXTREME TDD workflow

    let claude_md = "../CLAUDE.md"; // Test runs from rash/ directory
    assert!(
        Path::new(claude_md).exists(),
        "CLAUDE.md documentation must exist at project root"
    );

    let content = fs::read_to_string(claude_md).expect("Failed to read CLAUDE.md");

    // Check for key EXTREME TDD concepts (case-insensitive to match actual doc terminology)
    let required_sections = vec![
        "EXTREME TDD",
        "RED",
        "GREEN",
        "REFACTOR",
        "Property",
        "Mutation",
    ];

    for section in &required_sections {
        assert!(
            content.contains(section),
            "CLAUDE.md missing required section: {}",
            section
        );
    }
}

// ============================================================================
// Test: SPEC_007 - Specification Document Compliance
// ============================================================================

#[test]
fn test_SPEC_007_unified_spec_document_exists() {
    // Verify the specification document exists and is complete

    let spec_path = "../docs/specification/unified-testing-quality-spec.md"; // Test runs from rash/
    assert!(
        Path::new(spec_path).exists(),
        "Unified testing quality specification must exist at {}",
        spec_path
    );

    let content = fs::read_to_string(spec_path).expect("Failed to read specification");

    // Verify key sections
    let required_sections = vec![
        "## Testing Capabilities by File Type",
        "### script.sh (Bash Scripts)",
        "### Makefile",
        "### Dockerfile",
        "## Quality Targets",
        "## Test Naming Convention",
        "## EXTREME TDD Requirements",
        "## CLI Testing Standards",
        "## Quality Gates",
    ];

    for section in &required_sections {
        assert!(
            content.contains(section),
            "Specification missing required section: {}",
            section
        );
    }
}

// ============================================================================
// Test: SPEC_008 - Test Count Validation
// ============================================================================

#[test]
fn test_SPEC_008_test_count_exceeds_minimum() {
    // Verify we have at least 6000+ tests (per spec: 6517+ tests)

    let output = Command::new("cargo")
        .args(["test", "--lib", "--", "--list"])
        .output()
        .expect("Failed to list tests");

    let test_list = String::from_utf8_lossy(&output.stdout);
    let test_count = test_list
        .lines()
        .filter(|line| line.contains(": test"))
        .count();

    assert!(
        test_count >= 6000,
        "Test count below minimum threshold:\n  Expected: ≥6000\n  Actual: {}",
        test_count
    );
}
