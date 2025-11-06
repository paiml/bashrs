//! Test for Issue #16: Makefile linting false positives
//!
//! GitHub Issue: https://github.com/paiml/bashrs/issues/16
//! Bug Report: "Makefile linting: False positives and improvement suggestions"
//!
//! Issues to fix:
//! 1. SC2168: False positive on "local" in quoted strings
//! 2. SC2082/SC2086/SC2154: Variable expansion mishandling with $$VAR
//! 3. SC2102: Regex pattern misinterpretation in awk commands
//! 4. SC2095: Context-insensitive redirection advice
//!
//! Test methodology: EXTREME TDD (RED → GREEN → REFACTOR)

use bashrs::linter::rules::lint_makefile;
use bashrs::linter::Severity;

/// Issue #16.1: SC2168 false positive on "local" in quoted strings
///
/// PROBLEM: The word "local" in `@printf 'Starting local server...'`
/// incorrectly triggers SC2168 (local outside function)
///
/// EXPECTED: Keywords within quoted strings should not be flagged
#[test]
fn test_issue_016_sc2168_local_in_quoted_string() {
    let makefile = r#"
serve:
	@printf 'Starting local server on port 8080...\n'
	@echo "Connecting to local database"
"#;

    let result = lint_makefile(makefile);

    // Should NOT report SC2168 for "local" in quoted strings
    let sc2168_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.code == "SC2168")
        .collect();

    assert_eq!(
        sc2168_errors.len(),
        0,
        "SC2168 should not trigger on 'local' in quoted strings. Found {} errors: {:?}",
        sc2168_errors.len(),
        sc2168_errors
    );
}

/// Issue #16.1b: SC2168 should only trigger on actual 'local' keyword usage
#[test]
fn test_issue_016_sc2168_local_in_various_contexts() {
    let makefile = r#"
test:
	@echo "local variable"
	@printf 'local server\n'
	@echo 'localhost'
	@echo "locale settings"
"#;

    let result = lint_makefile(makefile);

    let sc2168_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.code == "SC2168")
        .collect();

    assert_eq!(
        sc2168_errors.len(),
        0,
        "SC2168 should not trigger on 'local' as part of other words or in strings. Found {} errors: {:?}",
        sc2168_errors.len(),
        sc2168_errors
    );
}

/// Issue #16.2: SC2082 false positive on $$VAR (proper Makefile syntax)
///
/// PROBLEM: bashrs misinterprets `$$VAR` as indirection when it's
/// actually proper Makefile escaping for shell variables
///
/// EXPECTED: $$VAR in Makefile recipes should not trigger SC2082
#[test]
fn test_issue_016_sc2082_dollar_dollar_var_in_makefile() {
    let makefile = r#"
deploy:
	@VERSION=1.0.0
	@echo "Deploying version $$VERSION"
	@IMAGE=myapp:$$VERSION
"#;

    let result = lint_makefile(makefile);

    // SC2082 should NOT trigger on $$VAR in Makefiles
    let sc2082_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.code == "SC2082")
        .collect();

    assert_eq!(
        sc2082_errors.len(),
        0,
        "SC2082 should not trigger on $$VAR in Makefile recipes. Found {} errors: {:?}",
        sc2082_errors.len(),
        sc2082_errors
    );
}

/// Issue #16.2b: SC2086 false positive on unquoted $$VAR
///
/// PROBLEM: After preprocessing $$VAR → $VAR, the tool warns about
/// unquoted variables, but $$VAR is proper Makefile syntax
///
/// EXPECTED: No SC2086 warnings for $$VAR in Makefile recipes
#[test]
fn test_issue_016_sc2086_unquoted_dollar_dollar_var() {
    let makefile = r#"
build:
	@CORES=$$(nproc)
	@echo "Using $$CORES cores"
	@make -j$$CORES
"#;

    let result = lint_makefile(makefile);

    // SC2086 should NOT trigger on $$CORES in echo/make commands
    let sc2086_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.code == "SC2086")
        .collect();

    assert_eq!(
        sc2086_errors.len(),
        0,
        "SC2086 should not trigger on unquoted $$VAR in Makefiles. Found {} errors: {:?}",
        sc2086_errors.len(),
        sc2086_errors
    );
}

/// Issue #16.2c: SC2154 false positive on undefined $$VAR
///
/// PROBLEM: bashrs warns about undefined variables for $$VAR usage
/// even though they're defined in the same recipe
///
/// EXPECTED: No SC2154 warnings for $$VAR in Makefile recipes
#[test]
fn test_issue_016_sc2154_undefined_dollar_dollar_var() {
    let makefile = r#"
check:
	@STATUS=$$(curl -s -o /dev/null -w "%{http_code}" http://localhost:8080)
	@if [ "$$STATUS" = "200" ]; then echo "OK"; fi
"#;

    let result = lint_makefile(makefile);

    // SC2154 should NOT trigger on $$STATUS
    let sc2154_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.code == "SC2154")
        .collect();

    assert_eq!(
        sc2154_errors.len(),
        0,
        "SC2154 should not trigger on $$VAR in Makefiles. Found {} errors: {:?}",
        sc2154_errors.len(),
        sc2154_errors
    );
}

/// Issue #16.3: SC2102 false positive on awk regex patterns
///
/// PROBLEM: bashrs warns about [a-zA-Z_-]+ thinking it's invalid glob syntax,
/// but this is valid awk regex syntax
///
/// EXPECTED: Regex patterns in awk/sed commands should not trigger SC2102
#[test]
fn test_issue_016_sc2102_awk_regex_pattern() {
    let makefile = r#"
analyze:
	@awk '/[a-zA-Z_-]+/ {print $$1}' input.txt
	@sed 's/[0-9]+/NUM/g' data.txt
"#;

    let result = lint_makefile(makefile);

    // SC2102 should NOT trigger on awk/sed regex patterns
    let sc2102_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.code == "SC2102")
        .collect();

    assert_eq!(
        sc2102_errors.len(),
        0,
        "SC2102 should not trigger on regex patterns in awk/sed. Found {} errors: {:?}",
        sc2102_errors.len(),
        sc2102_errors
    );
}

/// Issue #16.4: SC2095 context-insensitive redirection advice
///
/// PROBLEM: bashrs suggests moving redirections after 'fi', but in Makefiles
/// each recipe line executes independently, making the pattern intentional
///
/// EXPECTED: SC2095 should not apply to Makefile recipes or should be suppressed
#[test]
fn test_issue_016_sc2095_makefile_redirection_context() {
    let makefile = r#"
backup:
	@if [ -f data.txt ]; then
	@	cat data.txt > backup.txt
	@fi
"#;

    let result = lint_makefile(makefile);

    // SC2095 should NOT trigger on Makefile recipe redirections
    let sc2095_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.code == "SC2095")
        .collect();

    assert_eq!(
        sc2095_errors.len(),
        0,
        "SC2095 should not trigger on Makefile recipe patterns. Found {} errors: {:?}",
        sc2095_errors.len(),
        sc2095_errors
    );
}

/// Comprehensive test: All Issue #16 false positives together
#[test]
fn test_issue_016_all_false_positives_comprehensive() {
    let makefile = r#"
PROJECT := myapp

serve:
	@printf 'Starting local server on port 8080...\n'
	@echo "Connecting to local database"

deploy:
	@VERSION=1.0.0
	@echo "Deploying version $$VERSION"
	@IMAGE=myapp:$$VERSION

build:
	@CORES=$$(nproc)
	@echo "Using $$CORES cores"
	@make -j$$CORES

analyze:
	@awk '/[a-zA-Z_-]+/ {print $$1}' input.txt
	@sed 's/[0-9]+/NUM/g' data.txt

backup:
	@if [ -f data.txt ]; then cat data.txt > backup.txt; fi
"#;

    let result = lint_makefile(makefile);

    // Count false positives by rule
    let sc2168_count = result
        .diagnostics
        .iter()
        .filter(|d| d.code == "SC2168")
        .count();
    let sc2082_count = result
        .diagnostics
        .iter()
        .filter(|d| d.code == "SC2082")
        .count();
    let sc2086_count = result
        .diagnostics
        .iter()
        .filter(|d| d.code == "SC2086")
        .count();
    let sc2154_count = result
        .diagnostics
        .iter()
        .filter(|d| d.code == "SC2154")
        .count();
    let sc2102_count = result
        .diagnostics
        .iter()
        .filter(|d| d.code == "SC2102")
        .count();
    let sc2095_count = result
        .diagnostics
        .iter()
        .filter(|d| d.code == "SC2095")
        .count();

    println!("\n=== Issue #16 False Positives Count ===");
    println!("SC2168 ('local' in strings): {}", sc2168_count);
    println!("SC2082 ($$VAR indirection): {}", sc2082_count);
    println!("SC2086 (unquoted $$VAR): {}", sc2086_count);
    println!("SC2154 (undefined $$VAR): {}", sc2154_count);
    println!("SC2102 (awk regex): {}", sc2102_count);
    println!("SC2095 (redirection): {}", sc2095_count);
    println!("=======================================\n");

    // All should be zero after fix
    assert_eq!(sc2168_count, 0, "SC2168 false positives remain");
    assert_eq!(sc2082_count, 0, "SC2082 false positives remain");
    assert_eq!(sc2086_count, 0, "SC2086 false positives remain");
    assert_eq!(sc2154_count, 0, "SC2154 false positives remain");
    assert_eq!(sc2102_count, 0, "SC2102 false positives remain");
    assert_eq!(sc2095_count, 0, "SC2095 false positives remain");

    // Verify no ERROR-level diagnostics for these patterns
    let error_count = result
        .diagnostics
        .iter()
        .filter(|d| {
            d.severity == Severity::Error
                && ["SC2168", "SC2082", "SC2086", "SC2154", "SC2102", "SC2095"]
                    .contains(&d.code.as_str())
        })
        .count();

    assert_eq!(
        error_count, 0,
        "Found {} ERROR-level diagnostics for Issue #16 patterns",
        error_count
    );
}
