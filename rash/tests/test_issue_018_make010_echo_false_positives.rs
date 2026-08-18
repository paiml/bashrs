#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

//! Test for Issue #18: MAKE010 false positives on echo statements
//!
//! GitHub Issue: https://github.com/paiml/bashrs/issues/18
//! Bug Report: "MAKE010 false positives on echo statements containing command keywords"
//!
//! PROBLEM:
//! bashrs reports MAKE010 warnings for echo/printf statements that contain
//! command keywords like "install" in quoted strings. These are not actual
//! commands being executed, just help messages for users.
//!
//! Example false positive:
//! ```makefile
//! check-deps:
//!     @echo "foo not installed. Run: make install-tools"
//! ```
//!
//! bashrs incorrectly warns:
//! ⚠ MAKE010: Command 'install' missing error handling
//!
//! EXPECTED BEHAVIOR:
//! bashrs should distinguish between:
//! 1. Actual commands: `cargo install foo` → ✅ Should warn
//! 2. String literals: `echo "Run: cargo install foo"` → ❌ Should NOT warn
//!
//! Test methodology: EXTREME TDD (RED → GREEN → REFACTOR)

use bashrs::linter::rules::lint_makefile;

/// Issue #18: MAKE010 false positive on echo with "install" keyword
///
/// RED PHASE: This test should FAIL initially, proving the bug exists
#[test]
fn test_issue_018_make010_echo_install_false_positive() {
    let makefile = r#"
.PHONY: check-deps
check-deps:
	@if ! command -v foo > /dev/null 2>&1; then \
		echo "foo not installed. Run: make install-tools"; \
		exit 1; \
	fi
"#;

    let result = lint_makefile(makefile);

    // Should NOT report MAKE010 for "install" inside echo string
    let make010_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.code == "MAKE010")
        .collect();

    assert_eq!(
        make010_errors.len(),
        0,
        "MAKE010 should not trigger on 'install' in echo string literal. Found {} errors: {:?}",
        make010_errors.len(),
        make010_errors
    );
}

/// Issue #18: MAKE010 should still warn on actual install commands
///
/// This test ensures we don't break the valid use case
#[test]
fn test_issue_018_make010_actual_install_command() {
    let makefile = r#"
install-tools:
	cargo install foo
"#;

    let result = lint_makefile(makefile);

    // SHOULD report MAKE010 for actual install command
    let make010_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.code == "MAKE010")
        .collect();

    assert_eq!(
        make010_errors.len(),
        1,
        "MAKE010 should trigger on actual 'cargo install' command. Found {} errors",
        make010_errors.len()
    );
}

/// Issue #18: Multiple echo patterns with command keywords
#[test]
fn test_issue_018_make010_various_echo_patterns() {
    let makefile = r#"
help:
	@echo "bashrs not installed. Run: make install-tools"
	@echo 'cargo-llvm-cov not installed. Run: cargo install cargo-llvm-cov'
	@printf "Use: cp file dest\n"
	@printf 'Run rm -rf /tmp/foo\n'
"#;

    let result = lint_makefile(makefile);

    // Should NOT report MAKE010 for any of these echo/printf statements
    let make010_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.code == "MAKE010")
        .collect();

    assert_eq!(
        make010_errors.len(),
        0,
        "MAKE010 should not trigger on command keywords in echo/printf strings. Found {} errors: {:?}",
        make010_errors.len(),
        make010_errors
    );
}

/// Issue #18: Distinguish echo from actual commands in mixed recipe
#[test]
fn test_issue_018_make010_mixed_echo_and_commands() {
    let makefile = r#"
deploy:
	@echo "Installing package..."
	cargo install myapp || exit 1
	@echo "Installation complete"
"#;

    let result = lint_makefile(makefile);

    // Should report MAKE010 ONLY for the actual cargo install, not the echo
    let make010_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.code == "MAKE010")
        .collect();

    // The cargo install already has || exit 1, so should be 0
    assert_eq!(
        make010_errors.len(),
        0,
        "MAKE010 should only trigger on actual commands, not echo statements. Found {} errors: {:?}",
        make010_errors.len(),
        make010_errors
    );
}

/// Issue #18: Test with actual command missing error handling vs echo
#[test]
fn test_issue_018_make010_real_command_vs_echo() {
    let makefile = r#"
setup:
	@echo "Run: rm -rf /tmp/data"
	rm -rf /tmp/data
"#;

    let result = lint_makefile(makefile);

    // Should report MAKE010 ONLY for the actual rm command (line 2)
    let make010_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.code == "MAKE010")
        .collect();

    assert_eq!(
        make010_errors.len(),
        1,
        "MAKE010 should trigger once for actual 'rm' command, not for 'rm' in echo. Found {} errors: {:?}",
        make010_errors.len(),
        make010_errors
    );

    // Verify it's the actual rm command, not the echo
    if let Some(diag) = make010_errors.first() {
        // The actual rm is on line 4 (after blank line, .PHONY, recipe header, echo)
        assert!(
            diag.span.start_line >= 4,
            "Error should be on the actual rm command line (line ≥4), not echo. Found line {}",
            diag.span.start_line
        );
    }
}

/// Issue #18: Heredoc with command keywords should not trigger MAKE010
#[test]
fn test_issue_018_make010_heredoc_with_commands() {
    let makefile = r#"
docs:
	@cat << EOF
To install dependencies:
  Run: cargo install foo
  Run: make install
EOF
"#;

    let result = lint_makefile(makefile);

    // Should NOT report MAKE010 for keywords in heredoc
    let make010_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.code == "MAKE010")
        .collect();

    assert_eq!(
        make010_errors.len(),
        0,
        "MAKE010 should not trigger on command keywords in heredocs. Found {} errors: {:?}",
        make010_errors.len(),
        make010_errors
    );
}

/// Issue #18: Variable assignments with command keywords
#[test]
fn test_issue_018_make010_variable_assignment() {
    let makefile = r#"
config:
	@MSG="install here"
	@HELP='Use: rm -rf /tmp'
	@echo "$$MSG"
"#;

    let result = lint_makefile(makefile);

    // Should NOT report MAKE010 for keywords in variable assignments
    let make010_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.code == "MAKE010")
        .collect();

    assert_eq!(
        make010_errors.len(),
        0,
        "MAKE010 should not trigger on command keywords in variable assignments. Found {} errors: {:?}",
        make010_errors.len(),
        make010_errors
    );
}

/// Issue #18: Comprehensive test with all patterns from the issue
#[test]
fn test_issue_018_make010_comprehensive_ruchy_docker_example() {
    // Real-world Makefile from ruchy-docker project
    let makefile = r#"
PROJECT := ruchy

.PHONY: check-deps
check-deps:
	@if ! command -v bashrs > /dev/null 2>&1; then \
		echo "bashrs not installed. Run: make install-tools"; \
		exit 1; \
	fi
	@if ! command -v cargo-llvm-cov > /dev/null 2>&1; then \
		echo "cargo-llvm-cov not installed. Run: cargo install cargo-llvm-cov"; \
		exit 1; \
	fi

.PHONY: install-tools
install-tools:
	cargo install bashrs
	cargo install cargo-llvm-cov

.PHONY: clean
clean:
	docker rm -f test-container
	rm -rf target/
"#;

    let result = lint_makefile(makefile);

    // Count MAKE010 errors
    let make010_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.code == "MAKE010")
        .collect();

    // Expected MAKE010 warnings:
    // 1. cargo install bashrs (line in install-tools)
    // 2. cargo install cargo-llvm-cov (line in install-tools)
    // 3. docker rm -f test-container (line in clean)
    // 4. rm -rf target/ (line in clean)
    //
    // Should NOT warn on:
    // - echo "bashrs not installed. Run: make install-tools"
    // - echo "cargo-llvm-cov not installed. Run: cargo install cargo-llvm-cov"
    //
    // Total expected: 4 warnings (not 8 with false positives)

    println!("\n=== Issue #18 MAKE010 Analysis ===");
    println!("Total MAKE010 warnings: {}", make010_errors.len());
    for (i, diag) in make010_errors.iter().enumerate() {
        println!(
            "  {}: Line {} - {}",
            i + 1,
            diag.span.start_line,
            diag.message
        );
    }
    println!("===================================\n");

    assert_eq!(
        make010_errors.len(),
        4,
        "Expected 4 MAKE010 warnings (actual commands only, not echo statements). Found {}",
        make010_errors.len()
    );

    // Verify none of the errors are on the echo lines
    for diag in &make010_errors {
        let message_lower = diag.message.to_lowercase();
        assert!(
            !message_lower.contains("echo"),
            "MAKE010 should not trigger on echo statements. Found error: {}",
            diag.message
        );
    }
}
