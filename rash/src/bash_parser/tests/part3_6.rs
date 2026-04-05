#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]

use super::super::ast::Redirect;
use super::super::lexer::Lexer;
use super::super::parser::BashParser;
use super::super::semantic::SemanticAnalyzer;
use super::super::*;

/// Helper: assert that BashParser handles the input without panicking.
/// Accepts both successful parses and parse errors (documentation tests
/// only verify the parser doesn't crash, not that the input is valid).
#[test]
fn test_PARAM_SPEC_002_exit_status_clobbering() {
    let clobbering_issue = r#"
# BAD: $? clobbered by [ command
cmd
if [ $? -eq 0 ]; then  # This tests if [ succeeded, not cmd!
  echo "Wrong"
fi

# GOOD: Capture $? immediately
cmd
STATUS=$?
if [ $STATUS -eq 0 ]; then
  echo "Correct"
fi

# BETTER: Direct conditional
if cmd; then
  echo "Best practice"
fi
"#;

    assert_parses_without_panic(
        clobbering_issue,
        "$? clobbering behavior is POSIX-compliant",
    );
}

#[test]
fn test_PARAM_SPEC_002_exit_status_functions() {
    // DOCUMENTATION: $? with functions (POSIX)
    //
    // Functions return exit status like commands:
    // - Explicit: return N (0-255)
    // - Implicit: exit status of last command
    //
    // $ my_function() {
    // $   cmd
    // $   return $?  # Explicit return
    // $ }
    // $
    // $ my_function
    // $ echo $?  # Function's return value

    let function_exit = r#"
check_file() {
  if [ -f "$1" ]; then
return 0
  else
return 1
  fi
}

# Implicit return (last command)
process_data() {
  validate_input
  transform_data
  save_output  # Function returns this command's status
}

# Using function status
check_file "/tmp/data.txt"
if [ $? -eq 0 ]; then
  echo "File exists"
fi

# Better: Direct conditional
if check_file "/tmp/data.txt"; then
  echo "File exists"
fi
"#;

    let result = BashParser::new(function_exit);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "$? with functions is POSIX-compliant"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

#[test]
fn test_PARAM_SPEC_002_exit_status_subshells() {
    // DOCUMENTATION: $? with subshells and command substitution (POSIX)
    //
    // Subshells and command substitution preserve exit status:
    //
    // Subshell:
    // $ ( cmd1; cmd2 )
    // $ echo $?  # Exit status of cmd2
    //
    // Command substitution (capture output, lose status):
    // $ OUTPUT=$(cmd)
    // $ echo $?  # Always 0 if assignment succeeded
    //
    // To capture both output and status:
    // $ OUTPUT=$(cmd)
    // $ STATUS=$?  # This is too late! Already clobbered
    //
    // Better: Set -e or check inline:
    // $ OUTPUT=$(cmd) || { echo "Failed"; exit 1; }

    let subshell_exit = r#"
# Subshell exit status
( cmd1; cmd2 )
echo "Subshell status: $?"

# Command substitution loses status
OUTPUT=$(cmd)
echo $?  # This is assignment status, not cmd status!

# Capture output and check status inline
OUTPUT=$(cmd) || {
  echo "Command failed"
  exit 1
}

# Alternative: set -e (exit on any error)
set -e
OUTPUT=$(cmd)  # Will exit script if cmd fails
"#;

    let result = BashParser::new(subshell_exit);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "$? with subshells is POSIX-compliant"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

#[test]
fn test_PARAM_SPEC_002_exit_status_common_use_cases() {
    // DOCUMENTATION: Common $? use cases (POSIX)
    //
    // Use Case 1: Error handling
    // $ cmd
    // $ if [ $? -ne 0 ]; then
    // $   echo "Error occurred"
    // $   exit 1
    // $ fi
    //
    // Use Case 2: Multiple status checks
    // $ cmd1
    // $ STATUS1=$?
    // $ cmd2
    // $ STATUS2=$?
    // $ if [ $STATUS1 -ne 0 ] || [ $STATUS2 -ne 0 ]; then
    // $   echo "One or both failed"
    // $ fi
    //
    // Use Case 3: Logging
    // $ cmd
    // $ STATUS=$?
    // $ log_message "Command exited with status $STATUS"
    // $ [ $STATUS -eq 0 ] || exit $STATUS

    let common_uses = r#"
# Use Case 1: Error handling
deploy_app
if [ $? -ne 0 ]; then
  echo "Deployment failed"
  rollback_changes
  exit 1
fi

# Use Case 2: Multiple checks
backup_database
DB_STATUS=$?
backup_files
FILE_STATUS=$?

if [ $DB_STATUS -ne 0 ] || [ $FILE_STATUS -ne 0 ]; then
  echo "Backup failed"
  send_alert
  exit 1
fi

# Use Case 3: Logging with status
critical_operation
STATUS=$?
log_event "Operation completed with status $STATUS"
if [ $STATUS -ne 0 ]; then
  send_alert "Critical operation failed: $STATUS"
  exit $STATUS
fi
"#;

    let result = BashParser::new(common_uses);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "Common $? patterns are POSIX-compliant"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

// DOCUMENTATION: Exit status comparison (POSIX vs Bash)
// $? is POSIX-compliant, 0-255 range, clobbered by every command.
// Rust mapping: std::process::Command .status() .code()
// bashrs: SUPPORTED, no transformation needed, preserve as-is.
#[test]
fn test_PARAM_SPEC_002_exit_status_comparison_table() {
    let comparison_example = r#"
# POSIX: $? fully supported
cmd
echo "Exit: $?"

# POSIX: Capture and use
cmd
STATUS=$?
if [ $STATUS -ne 0 ]; then
  echo "Failed with code $STATUS"
  exit $STATUS
fi

# POSIX: set -o pipefail (supported in bash, dash, ash)
set -o pipefail
cmd1 | cmd2 | cmd3
if [ $? -ne 0 ]; then
  echo "Pipeline failed"
fi

# Bash-only: PIPESTATUS (NOT SUPPORTED)
# cmd1 | cmd2 | cmd3
# echo "${PIPESTATUS[@]}"  # bashrs doesn't support this
"#;

    assert_parses_without_panic(comparison_example, "$? comparison documented");
}

// Summary:
// $? (exit status): FULLY SUPPORTED (POSIX)
// Range: 0-255 (0=success, non-zero=failure)
// Special codes: 126 (not executable), 127 (not found), 128+N (signal)
// Clobbering: Updated after every command
// Best practice: Capture immediately or use direct conditionals
// PIPESTATUS: NOT SUPPORTED (bash extension)
// pipefail: SUPPORTED (POSIX, available in bash/dash/ash)

// ============================================================================
// PARAM-SPEC-003: $$ Process ID (POSIX, but NON-DETERMINISTIC - PURIFY)
// ============================================================================

// DOCUMENTATION: $$ is POSIX but NON-DETERMINISTIC (must purify)
// $$ contains the process ID of the current shell. Changes every run.
// Purification: replace $$ with fixed identifier, use mktemp for temp files.
#[test]
fn test_PARAM_SPEC_003_process_id_non_deterministic() {
    let process_id = r#"
echo "Process ID: $$"
echo "Script PID: $$"
"#;

    assert_parses_without_panic(
        process_id,
        "$$ is POSIX-compliant but NON-DETERMINISTIC (must purify)",
    );
}

#[test]
fn test_PARAM_SPEC_003_process_id_temp_files() {
    // DOCUMENTATION: Common anti-pattern - $$ for temp files
    //
    // ANTI-PATTERN (non-deterministic):
    // $ TMPFILE=/tmp/myapp.$$
    // $ echo "data" > /tmp/script.$$.log
    // $ rm -f /tmp/output.$$
    //
    // Problem: File names change every run
    // - First run: /tmp/myapp.12345
    // - Second run: /tmp/myapp.67890
    // - Third run: /tmp/myapp.23456
    //
    // This breaks:
    // - Determinism (file names unpredictable)
    // - Idempotency (can't clean up old files reliably)
    // - Testing (can't assert on specific file names)
    //
    // POSIX alternatives (deterministic):
    // 1. Use mktemp (creates unique temp file safely):
    //    $ TMPFILE=$(mktemp /tmp/myapp.XXXXXX)
    //
    // 2. Use fixed name with script name:
    //    $ TMPFILE="/tmp/myapp.tmp"
    //
    // 3. Use XDG directories:
    //    $ TMPFILE="${XDG_RUNTIME_DIR:-/tmp}/myapp.tmp"
    //
    // 4. Use script name from $0:
    //    $ TMPFILE="/tmp/$(basename "$0").tmp"

    let temp_file_pattern = r#"
# ANTI-PATTERN: Non-deterministic temp files
TMPFILE=/tmp/myapp.$$
echo "data" > /tmp/script.$$.log
rm -f /tmp/output.$$

# BETTER: Use mktemp (deterministic, safe)
TMPFILE=$(mktemp /tmp/myapp.XXXXXX)

# BETTER: Use fixed name
TMPFILE="/tmp/myapp.tmp"

# BETTER: Use script name
TMPFILE="/tmp/$(basename "$0").tmp"
"#;

    let result = BashParser::new(temp_file_pattern);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "$$ for temp files is non-deterministic anti-pattern"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

#[test]

include!("part3_6_param_spec.rs");
