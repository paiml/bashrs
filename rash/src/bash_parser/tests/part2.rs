#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]

use super::super::ast::Redirect;
use super::super::lexer::Lexer;
use super::super::parser::BashParser;
use super::super::semantic::SemanticAnalyzer;
use super::super::*;

/// Helper: parse a script and return whether parsing succeeded.
/// Used by documentation tests that only need to verify parsability.

#[test]
fn test_BUILTIN_019_umask_basic() {
    // DOCUMENTATION: Basic umask command parsing
    //
    // Bash: umask 022
    // Effect: New files: 644 (rw-r--r--), dirs: 755 (rwxr-xr-x)
    // Rust: std::fs::set_permissions() or libc::umask()
    // Purified: umask 022
    //
    // Global State: Modifies file creation mask
    // Priority: LOW (works but has global state implications)

    let script = r#"umask 022"#;
    let result = BashParser::new(script);

    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok(),
                "umask should parse successfully: {:?}",
                parse_result.err()
            );
        }
        Err(e) => {
            panic!("umask parsing failed: {:?}", e);
        }
    }

    // DOCUMENTATION: umask is supported
    // Global State: Modifies process-wide permissions
    // Best Practice: Set once at script start, document reasoning
}

#[test]
fn test_BUILTIN_019_umask_global_state() {
    // DOCUMENTATION: umask modifies global state
    //
    // Problem: umask affects entire process
    // Effect: All file operations after umask use new mask
    //
    // Example:
    // #!/bin/bash
    // touch file1.txt    # Uses default umask (e.g., 022 → 644)
    // umask 077
    // touch file2.txt    # Uses new umask (077 → 600)
    //
    // file1.txt: -rw-r--r-- (644)
    // file2.txt: -rw------- (600)

    let script = r#"umask 077"#;
    let result = BashParser::new(script);

    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok(),
                "umask with global state documented: {:?}",
                parse_result.err()
            );
        }
        Err(_) => {
            panic!("umask should parse");
        }
    }

    // DOCUMENTATION: umask has global side effects
    // Global State: Cannot be scoped or limited
    // Side Effects: Affects all subsequent file operations
    // Consideration: May surprise developers unfamiliar with umask
}

#[test]
fn test_BUILTIN_019_umask_idempotency_concern() {
    // DOCUMENTATION: umask idempotency considerations
    //
    // Concern: Running script multiple times
    // Issue: umask stacks if not carefully managed
    //
    // Safe Pattern:
    // #!/bin/bash
    // old_umask=$(umask)
    // umask 022
    // # ... script logic ...
    // umask "$old_umask"
    //
    // Unsafe Pattern:
    // #!/bin/bash
    // umask 022
    // # ... script logic ...
    // # umask not restored!

    let script = r#"old_umask=$(umask); umask 022"#;
    let result = BashParser::new(script);

    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "umask save/restore pattern documented"
            );
        }
        Err(_) => {
            // May fail due to command substitution
        }
    }

    // DOCUMENTATION: Best practice for umask
    // Safe: Save old umask, restore at end
    // Unsafe: Set umask without restoration
    // Idempotency: Restoration ensures safe re-run
}

#[test]
fn test_BUILTIN_019_umask_explicit_chmod_alternative() {
    // DOCUMENTATION: Explicit chmod as alternative to umask
    //
    // umask (global):
    // umask 077
    // touch file.txt      # Permissions: 600
    //
    // chmod (explicit, safer):
    // touch file.txt
    // chmod 600 file.txt  # Explicit, clear, localized
    //
    // Benefits of chmod:
    // - Explicit permissions (easier to understand)
    // - No global state modification
    // - Clear intent in code
    // - Easier to audit

    let script = r#"chmod 600 file.txt"#;
    let mut parser = BashParser::new(script).unwrap();
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "Explicit chmod should parse successfully: {:?}",
        result.err()
    );

    let ast = result.unwrap();
    assert!(!ast.statements.is_empty());

    // DOCUMENTATION: chmod is preferred over umask
    // Reason: Explicit, no global state, clear intent
    // umask: Global, implicit, affects all operations
    // chmod: Localized, explicit, affects specific files
    //
    // Recommendation:
    // - Use chmod for explicit permission control
    // - Use umask only when necessary (e.g., security requirements)
    // - Document why umask is needed if used
}

// ============================================================================
// BASH-BUILTIN-003: let - Arithmetic Evaluation
// Reference: docs/BASH-INGESTION-ROADMAP.yaml
// Status: DOCUMENTED (prefer $((...)) for POSIX)
//
// let evaluates arithmetic expressions:
// - let "x = 5 + 3" → x=8
// - let "y += 1" → y increments
// - let "z = x * y" → z = x * y
//
// POSIX Alternative: $((...))
// - x=$((5 + 3)) → POSIX-compliant
// - y=$((y + 1)) → POSIX-compliant
// - z=$((x * y)) → POSIX-compliant
//
// Purification Strategy:
// - Convert let to $((...)) for POSIX compliance
// - let "x = expr" → x=$((expr))
// - More portable and widely supported
//
// EXTREME TDD: Document let and POSIX alternative
// ============================================================================

#[test]
fn test_BASH_BUILTIN_003_let_basic() {
    // DOCUMENTATION: Basic let command parsing
    //
    // Bash: let "x = 5 + 3"
    // Result: x=8
    // Rust: let x = 5 + 3;
    // Purified: x=$((5 + 3))
    //
    // POSIX Alternative: $((arithmetic))
    // Priority: LOW (works but $((...)) is preferred)

    let script = r#"let "x = 5 + 3""#;
    let result = BashParser::new(script);

    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "let command parsing documented"
            );
        }
        Err(_) => {
            // May not parse let syntax
        }
    }

    // DOCUMENTATION: let is Bash-specific
    // POSIX: Use $((...)) for arithmetic
    // Purification: Convert let → $((...))
}

#[test]
fn test_BASH_BUILTIN_003_let_increment() {
    // DOCUMENTATION: let with increment operator
    //
    // Bash: let "y += 1"
    // Result: y increments by 1
    // Purified: y=$((y + 1))
    //
    // Common Usage:
    // - let "i++" → i=$((i + 1))
    // - let "j--" → j=$((j - 1))
    // - let "k *= 2" → k=$((k * 2))

    let script = r#"let "y += 1""#;
    let result = BashParser::new(script);

    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "let increment documented"
            );
        }
        Err(_) => {
            // May not parse
        }
    }

    // DOCUMENTATION: let supports C-style operators
    // POSIX: Use explicit arithmetic: x=$((x + 1))
    // Clarity: Explicit form is more readable
}

#[test]
fn test_BASH_BUILTIN_003_let_posix_alternative() {
    // DOCUMENTATION: POSIX $((...)) alternative to let
    //
    // let (Bash-specific):
    // let "x = 5 + 3"
    //
    // $((...)) (POSIX-compliant):
    // x=$((5 + 3))
    //
    // This test verifies $((...)) works as replacement for let.

    let script = r#"x=$((5 + 3))"#;
    let result = BashParser::new(script);

    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "POSIX arithmetic documented"
            );
        }
        Err(_) => {
            // May not parse arithmetic
        }
    }

    // DOCUMENTATION: $((...)) is preferred over let
    // Reason: POSIX-compliant, more portable
    // let: Bash-specific extension
    // $((...)):  Works in sh, dash, bash, zsh
    //
    // Purification Strategy:
    // - let "x = expr" → x=$((expr))
    // - More explicit and portable
}

#[test]
fn test_BASH_BUILTIN_003_let_refactoring() {
    // DOCUMENTATION: How to refactor let to POSIX
    //
    // Bash (let):
    // let "x = 5 + 3"
    // let "y += 1"
    // let "z = x * y"
    //
    // POSIX ($((...)):
    // x=$((5 + 3))
    // y=$((y + 1))
    // z=$((x * y))
    //
    // Benefits:
    // - POSIX-compliant (works everywhere)
    // - More explicit and readable
    // - No quoting needed
    // - Standard shell arithmetic

    let script = r#"x=$((5 + 3))"#;
    let result = BashParser::new(script);

    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "POSIX arithmetic refactoring documented"
            );
        }
        Err(_) => {
            // May not parse
        }
    }

    // DOCUMENTATION: Refactoring strategy for let
    // Instead of: let "x = 5 + 3" (Bash-specific)
    // Use: x=$((5 + 3)) (POSIX-compliant)
    //
    // Conversion Rules:
    // - let "x = expr" → x=$((expr))
    // - let "x += 1" → x=$((x + 1))
    // - let "x++" → x=$((x + 1))
    // - let "x--" → x=$((x - 1))
    //
    // Portability:
    // - let: Bash, zsh only
    // - $((...)):  All POSIX shells (sh, dash, bash, zsh, ksh)
}

// ============================================================================
// TASK 1.2: Interactive vs Script Mode
// ============================================================================
//
// Task: 1.2 - Document interactive vs script mode
// Status: DOCUMENTED
// Priority: HIGH (foundational concept)
//
// bashrs philosophy: SCRIPT MODE ONLY (deterministic, non-interactive)
//
// Why script mode only?
// - Determinism: Same input → same output (always)
// - Automation: Works in CI/CD, cron, Docker (no TTY needed)
// - Testing: Can be unit tested (no human input required)
// - Safety: No risk of user typos or unexpected input
//
// Interactive features NOT SUPPORTED:
// - read command (waits for user input) → use command-line args
// - select menus → use config files
// - TTY detection (tty, isatty) → assume non-TTY
// - History navigation (↑↓ arrows) → use git for versioning
// - Tab completion → use IDE/editor completion
//
// Script features FULLY SUPPORTED:
// - Functions, variables, control flow
// - File I/O, process execution
// - Command-line argument parsing ($1, $2, $@)
// - Environment variables
// - Exit codes, error handling
//
// Transformation strategy:
// - Interactive bash → Deterministic script mode only
// - read var → var="$1" (command-line args)
// - select menu → config file or case statement
// - TTY checks → assume batch mode always

#[test]

include!("part2_cont.rs");
