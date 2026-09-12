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

/// Issue #18 as re-specified by PMAT-251 (GH-256, GH-257).
///
/// MAKE010 asks for `|| exit 1` only where a command's exit status would
/// otherwise be masked. A critical command that is the ONLY command in its
/// logical recipe is not flagged: its status IS the recipe's status, so Make
/// already aborts the target. The rule fires when another command follows it
/// in the same logical recipe, which is the case the original `restore:`
/// report was about.
#[test]
fn test_issue_018_make010_masked_command_is_reported() {
    let makefile = "setup:\n\tcp a b; echo done\n";

    let result = lint_makefile(makefile);

    let make010_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.code == "MAKE010")
        .collect();

    assert_eq!(
        make010_errors.len(),
        1,
        "a critical command followed by another in the same recipe is masked and must be reported. Found {} errors: {:?}",
        make010_errors.len(),
        make010_errors
    );
}

/// The other half of the same contract: alone, it needs nothing.
#[test]
fn test_issue_018_make010_lone_command_needs_no_guard() {
    let makefile = "install-tools:\n\tcargo install foo\n";

    let result = lint_makefile(makefile);

    let make010_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.code == "MAKE010")
        .collect();

    assert!(
        make010_errors.is_empty(),
        "a lone command carries its own exit status to Make; `|| exit 1` there is a no-op. Found {:?}",
        make010_errors
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

/// Issue #18: the text inside an echo is never a command, while a real
/// masked command on the next line still is.
#[test]
fn test_issue_018_make010_real_command_vs_echo() {
    let makefile = "setup:\n\t@echo \"Run: cp a b\"\n\tcp a b; echo done\n";

    let result = lint_makefile(makefile);

    let make010_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.code == "MAKE010")
        .collect();

    assert_eq!(
        make010_errors.len(),
        1,
        "exactly the real cp, never the cp inside the echo. Found {} errors: {:?}",
        make010_errors.len(),
        make010_errors
    );

    let diag = make010_errors[0];
    assert!(
        diag.span.start_line >= 3,
        "the report belongs to the real command line, not the echo. Found line {}",
        diag.span.start_line
    );
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

/// Issue #18: a real Makefile from the ruchy-docker project.
///
/// Under the PMAT-251 contract every candidate here is exempt, and for a
/// stated reason rather than by accident:
///
/// - `cargo install bashrs` and `cargo install cargo-llvm-cov` are each the
///   only command in their own physical recipe line, so Make already sees
///   their status;
/// - `docker rm -f test-container` and `rm -rf target/` declare the
///   tolerance MAKE010 would ask for, in the `-f` and `-rf` flags;
/// - the two `echo` lines are text, which is what issue #18 was about.
///
/// So the whole file is clean, and no echo is ever the subject of a report.
#[test]
fn test_issue_018_make010_comprehensive_ruchy_docker_example() {
    let makefile = r#"
PROJECT := ruchy

.PHONY: check-deps
check-deps:
	@if ! command -v bashrs > /dev/null 2>&1; then \
		echo "bashrs not installed. Run: make install-tools"; \
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

    let make010_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.code == "MAKE010")
        .collect();

    assert!(
        make010_errors.is_empty(),
        "every candidate in this file is exempt for a stated reason. Found {:?}",
        make010_errors
    );

    for diag in &result.diagnostics {
        assert!(
            !diag.message.to_lowercase().contains("echo"),
            "no rule may take an echo for a command. Found: {}",
            diag.message
        );
    }
}
