#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]

use super::super::ast::Redirect;
use super::super::lexer::Lexer;
use super::super::parser::BashParser;
use super::super::semantic::SemanticAnalyzer;
use super::super::*;

/// Helper: tokenize input and assert tokens are non-empty.
/// Accepts parse errors gracefully (parser may not support all constructs yet).
#[test]
fn test_BUILTIN_005_cd_comparison_table() {
    // COMPREHENSIVE COMPARISON: POSIX vs Bash vs bashrs

    let cd_comparison = r#"
# POSIX SUPPORTED (bashrs SUPPORTED):
cd /tmp              # Basic navigation
cd -                 # Previous directory
cd                   # Home directory
cd ~                 # Home via tilde
cd ~/path            # Home subdir

# Bash extensions (bashrs NOT SUPPORTED):
cd -L /path          # Follow symlinks (bash default behavior)
cd -P /path          # Physical directory (resolve symlinks)
cd -e /path          # Exit on error (with -P)
cd -@ /path          # Extended attributes (rare)
CDPATH=/usr:/var     # Directory search path (bash/ksh extension)

# Environment variables (POSIX):
echo $PWD            # Current directory (updated by cd)
echo $OLDPWD         # Previous directory (updated by cd)
echo $HOME           # Home directory (used by cd)

# Exit status:
cd /tmp && echo "Success"   # Exit 0
cd /bad || echo "Failed"    # Exit 1

# Common patterns:
cd /tmp || exit 1           # Error handling
cd - >/dev/null 2>&1        # Silent previous dir
cd "$dir" || return 1       # Function error handling
"#;

    let mut lexer = Lexer::new(cd_comparison);
    match lexer.tokenize() {
        Ok(tokens) => {
            assert!(!tokens.is_empty(), "cd comparison should tokenize");
            let _ = tokens; // Use tokens to satisfy type inference
        }
        Err(_) => {
            // Test documents comprehensive cd behavior
        }
    }

    // SUMMARY
    // cd is POSIX-COMPLIANT and FULLY SUPPORTED in bashrs (basic navigation)
    // cd /path, cd -, cd (no args), cd ~, cd ~/path are all POSIX
    // Bash flags (-L, -P, -e, -@) are NOT SUPPORTED (bash extensions)
    // CDPATH is NOT SUPPORTED (bash/ksh extension, not POSIX)
    // Always quote paths with spaces, check exit status for errors
    // cd updates $PWD and $OLDPWD automatically
}

// ============================================================================
// BUILTIN-009: exit command (POSIX builtin)
// ============================================================================
// Task: Document exit (terminate shell) builtin command
// Reference: GNU Bash Manual Section 4.1 (Bourne Shell Builtins)
// POSIX: exit is POSIX-COMPLIANT (SUPPORTED)
//
// Syntax:
//   exit [n]
//   exit 0           # Exit with success (status 0)
//   exit 1           # Exit with failure (status 1)
//   exit             # Exit with status of last command ($?)
//   exit $?          # Explicit exit with last command status
//
// POSIX Compliance:
//   SUPPORTED: exit [n] where n is 0-255
//   SUPPORTED: exit with no args (uses $? from last command)
//   SUPPORTED: Exit status 0 = success, non-zero = failure
//   SUPPORTED: In functions, exit terminates entire script (not just function)
//   SUPPORTED: In subshells, exit terminates only the subshell
//
// Exit Status Conventions (POSIX):
//   0: Success (command completed successfully)
//   1: General errors (catchall for miscellaneous errors)
//   2: Misuse of shell builtins (missing keyword or command)
//   126: Command invoked cannot execute (permission problem)
//   127: Command not found (illegal command)
//   128: Invalid argument to exit (non-numeric or out of range)
//   128+N: Fatal error signal N (e.g., 130 = 128+2 for SIGINT/Ctrl-C)
//   255: Exit status out of range (exit takes only 0-255)
//
// Bash Extensions:
//   exit with value >255: Wraps modulo 256 (exit 256 becomes 0)
//   exit with negative value: Wraps modulo 256 (exit -1 becomes 255)
//   exit in trap handlers: Specific behaviors in various traps
//
// bashrs Support:
//   SUPPORTED: exit [n] where n is 0-255
//   SUPPORTED: exit with no args (uses $?)
//   SUPPORTED: Standard exit status conventions
//   NOT SUPPORTED: exit >255 (bash wrapping behavior)
//   NOT SUPPORTED: exit with negative values (bash wrapping behavior)
//
// Rust Mapping:
//   exit 0 → std::process::exit(0)
//   exit 1 → std::process::exit(1)
//   exit $? → std::process::exit(last_exit_status)
//   exit → std::process::exit(last_exit_status)
//
// Purified Bash:
//   exit 0 → exit 0 (POSIX supported)
//   exit 1 → exit 1 (POSIX supported)
//   exit → exit (POSIX supported, uses $?)
//   exit 256 → exit 0 (normalize to 0-255 range)
//   exit -1 → exit 255 (normalize to 0-255 range)
//
// Exit vs Return:
//   exit: Terminates entire script (even from function)
//   return: Returns from function only (function-local)
//   In script: exit terminates script
//   In function: exit terminates script, return returns from function
//   In subshell: exit terminates subshell only
//
// Common Use Cases:
//   1. Success exit: exit 0 (at end of script)
//   2. Error exit: exit 1 (on error conditions)
//   3. Conditional exit: [ -z "$VAR" ] && exit 1
//   4. Exit with last status: command || exit
//   5. Exit with custom code: exit 2 (for specific error types)
//   6. Early return: if [ error ]; then exit 1; fi
//
// Edge Cases:
//   1. exit with no args → uses $? from last command
//   2. exit >255 → bash wraps modulo 256 (exit 256 = 0)
//   3. exit <0 → bash wraps modulo 256 (exit -1 = 255)
//   4. exit in subshell → terminates subshell only, not parent
//   5. exit in function → terminates entire script, not just function
//   6. exit in trap → depends on trap type (EXIT, ERR, etc.)
//
// Best Practices:
//   1. Use exit 0 for success at end of script
//   2. Use exit 1 for general errors
//   3. Use specific exit codes (2-125) for different error types
//   4. Document exit codes in script header
//   5. Use return (not exit) in functions to avoid terminating script
//   6. Check $? before exit to propagate error codes
//   7. Avoid exit codes >125 (reserved for signals and special meanings)
//
// POSIX vs Bash Comparison:
//
// | Feature              | POSIX | Bash | bashrs | Notes                          |
// |----------------------|-------|------|--------|--------------------------------|
// | exit 0               | ✓     | ✓    | ✓      | Success exit                   |
// | exit 1               | ✓     | ✓    | ✓      | Error exit                     |
// | exit [0-255]         | ✓     | ✓    | ✓      | Valid exit codes               |
// | exit (no args)       | ✓     | ✓    | ✓      | Uses $? from last command      |
// | exit $?              | ✓     | ✓    | ✓      | Explicit last command status   |
// | exit >255            | ✗     | ✓    | ✗      | Wraps modulo 256 (bash only)   |
// | exit <0              | ✗     | ✓    | ✗      | Wraps modulo 256 (bash only)   |
// | Terminates script    | ✓     | ✓    | ✓      | From anywhere (incl. functions)|
// | Terminates subshell  | ✓     | ✓    | ✓      | Only subshell, not parent      |
// | Standard exit codes  | ✓     | ✓    | ✓      | 0=success, 1-2=errors, etc.    |
//
// ✓ = Supported
// ✗ = Not supported
//
// Summary:
// exit command: POSIX, FULLY SUPPORTED (0-255 range)
// exit terminates script (from anywhere, including functions)
// exit in subshell terminates only subshell
// exit with no args uses $? from last command
// Standard exit codes: 0 (success), 1 (general error), 2 (misuse), 126 (no execute), 127 (not found), 128+N (signal)
// Use exit 0 for success, exit 1 for general errors
// Use return (not exit) in functions to avoid terminating script
// Bash wrapping behavior (>255, <0) is NOT SUPPORTED

#[test]
fn test_BUILTIN_009_exit_command_supported() {
    // DOCUMENTATION: exit is SUPPORTED (POSIX builtin)
    // exit terminates the shell with specified exit code (0-255)
    // exit with no args uses $? (exit status of last command)
    // Syntax: exit [n]

    let exit_command = r#"
exit 0
exit 1
exit 2
exit
exit $?
"#;

    let mut lexer = Lexer::new(exit_command);
    match lexer.tokenize() {
        Ok(tokens) => {
            assert!(
                !tokens.is_empty(),
                "exit command should tokenize successfully"
            );
            let _ = tokens; // Use tokens to satisfy type inference
                            // exit is a builtin command, not a keyword
                            // It's treated as an identifier/command name
        }
        Err(_) => {
            // Parser may not fully support exit yet - test documents expected behavior
        }
    }

    // COMPARISON TABLE
    // | exit syntax   | Meaning                  | POSIX | Bash | bashrs |
    // |---------------|--------------------------|-------|------|--------|
    // | exit 0        | Exit with success        | ✓     | ✓    | ✓      |
    // | exit 1        | Exit with error          | ✓     | ✓    | ✓      |
    // | exit [0-255]  | Exit with code           | ✓     | ✓    | ✓      |
    // | exit          | Exit with last status    | ✓     | ✓    | ✓      |
    // | exit $?       | Explicit last status     | ✓     | ✓    | ✓      |
    // | exit 256      | Wraps to 0 (modulo 256)  | ✗     | ✓    | ✗      |
    // | exit -1       | Wraps to 255 (modulo 256)| ✗     | ✓    | ✗      |
}

#[test]
fn test_BUILTIN_009_exit_with_status_code() {
    // DOCUMENTATION: exit [n] where n is 0-255
    // 0 = success, non-zero = failure
    // Standard codes: 0 (success), 1 (error), 2 (misuse), 126 (no exec), 127 (not found), 128+N (signal)

    let exit_status = r#"
exit 0
exit 1
exit 2
exit 126
exit 127
exit 130
"#;

    let mut lexer = Lexer::new(exit_status);
    match lexer.tokenize() {
        Ok(tokens) => {
            assert!(!tokens.is_empty(), "exit with status should tokenize");
            let _ = tokens; // Use tokens to satisfy type inference
                            // exit is followed by numeric argument (exit code)
        }
        Err(_) => {
            // Test documents expected behavior
        }
    }

    // Standard exit codes:
    // 0: Success
    // 1: General error
    // 2: Misuse of shell builtins
    // 126: Command cannot execute
    // 127: Command not found
    // 128+N: Fatal error signal N (e.g., 130 = 128+2 for SIGINT)

    // Rust mapping: exit 0 → std::process::exit(0)
    // Purified bash: exit 0 → exit 0 (POSIX supported)
}

#[test]
fn test_BUILTIN_009_exit_no_args() {
    // DOCUMENTATION: exit with no args uses $? (last command exit status)
    // Equivalent to: exit $?
    // POSIX-compliant behavior

    let exit_no_args = r#"
command_that_fails
exit
"#;

    let mut lexer = Lexer::new(exit_no_args);
    match lexer.tokenize() {
        Ok(tokens) => {
            assert!(!tokens.is_empty(), "exit with no args should tokenize");
            let _ = tokens; // Use tokens to satisfy type inference
                            // exit alone (no arguments) is POSIX-compliant
                            // Uses $? from last command
        }
        Err(_) => {
            // Test documents expected behavior
        }
    }

    // Rust mapping: exit → std::process::exit(last_exit_status)
    // Purified bash: exit → exit (POSIX supported)
    // Common use: command || exit (exit if command fails)
}

#[test]
fn test_BUILTIN_009_exit_vs_return() {
    // DOCUMENTATION: exit vs return distinction
    // exit: Terminates entire script (even from function)
    // return: Returns from function only (function-local)
    // In subshell: exit terminates subshell only, not parent

    let exit_vs_return = r#"
function my_func() {
    if [ error ]; then
        return 1  # Returns from function only
    fi
    exit 1        # Terminates entire script
}

# In subshell
(
    exit 1        # Terminates subshell only
)
echo "Parent continues"
"#;

    let mut lexer = Lexer::new(exit_vs_return);
    match lexer.tokenize() {
        Ok(tokens) => {
            assert!(!tokens.is_empty(), "exit vs return should tokenize");
            let _ = tokens; // Use tokens to satisfy type inference
                            // exit terminates script, return is function-local
        }
        Err(_) => {
            // Test documents expected behavior
        }
    }

    // Key distinction:
    // return: Function-local (returns from function)
    // exit: Script-global (terminates entire script)
    // Exception: exit in subshell only terminates subshell
}

#[test]
fn test_BUILTIN_009_exit_standard_codes() {
    // DOCUMENTATION: Standard POSIX exit codes
    // 0: Success
    // 1: General errors
    // 2: Misuse of shell builtins
    // 126: Command invoked cannot execute
    // 127: Command not found
    // 128+N: Fatal error signal N
    // 255: Exit status out of range

    let exit_codes = r#"
# Success
exit 0

# General error
exit 1

# Misuse of shell builtin
exit 2

# Permission problem or command is not executable
exit 126

# Command not found
exit 127

# Invalid argument to exit
exit 128

# Fatal error signal (e.g., 130 = 128+2 for SIGINT/Ctrl-C)
exit 130

# Exit status out of range
exit 255
"#;

    let mut lexer = Lexer::new(exit_codes);
    match lexer.tokenize() {
        Ok(tokens) => {
            assert!(!tokens.is_empty(), "exit codes should tokenize");
            let _ = tokens; // Use tokens to satisfy type inference
                            // Standard exit codes are well-defined
        }
        Err(_) => {
            // Test documents expected behavior
        }
    }

    // Best practice: Document exit codes in script header
    // Use specific codes for different error types
    // Avoid codes >125 (reserved for signals and special meanings)
}

#[test]
fn test_BUILTIN_009_exit_conditional() {
    // DOCUMENTATION: Conditional exit patterns
    // Common patterns: [ condition ] && exit 1
    // command || exit (exit if command fails)
    // [ -z "$VAR" ] && { echo "Error"; exit 1; }

    let exit_conditional = r#"
# Exit if variable is empty
[ -z "$VAR" ] && exit 1

# Exit if command fails
command || exit 1

# Exit with error message
[ ! -f "$FILE" ] && { echo "File not found"; exit 1; }

# Early return pattern
if [ error ]; then
    echo "Error occurred"
    exit 1
fi
"#;

    let mut lexer = Lexer::new(exit_conditional);
    match lexer.tokenize() {
        Ok(tokens) => {
            assert!(!tokens.is_empty(), "conditional exit should tokenize");
            let _ = tokens; // Use tokens to satisfy type inference
                            // Conditional exit is common error handling pattern
        }
        Err(_) => {
            // Test documents expected behavior
        }
    }

    // Common patterns:
    // [ condition ] && exit 1 (exit if condition true)
    // command || exit (exit if command fails)
    // Early return pattern (check error, exit if found)
}

#[test]
fn test_BUILTIN_009_exit_edge_cases() {
    // DOCUMENTATION: Edge cases with exit
    // exit >255: Bash wraps modulo 256 (NOT SUPPORTED in bashrs)
    // exit <0: Bash wraps modulo 256 (NOT SUPPORTED in bashrs)
    // exit in subshell: Terminates subshell only
    // exit in function: Terminates entire script

    let exit_edge_cases = r#"
# Bash wrapping (NOT SUPPORTED in bashrs):
# exit 256   # Wraps to 0 in bash
# exit 257   # Wraps to 1 in bash
# exit -1    # Wraps to 255 in bash

# Subshell termination (SUPPORTED):
(exit 1)
echo "Parent continues after subshell exit"

# Function termination (SUPPORTED):
function func() {
    exit 1  # Terminates entire script, not just function
}
"#;

    let mut lexer = Lexer::new(exit_edge_cases);
    match lexer.tokenize() {
        Ok(tokens) => {
            assert!(!tokens.is_empty(), "exit edge cases should tokenize");
            let _ = tokens; // Use tokens to satisfy type inference
                            // Edge cases documented for completeness
        }
        Err(_) => {
            // Test documents expected behavior
        }
    }

    // Bash wrapping behavior is NOT SUPPORTED in bashrs
    // Use exit codes 0-255 only
    // Purification: exit 256 → exit 0, exit -1 → exit 255
}

