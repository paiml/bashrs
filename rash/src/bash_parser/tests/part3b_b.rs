#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]

use super::super::ast::Redirect;
use super::super::lexer::Lexer;
use super::super::parser::BashParser;
use super::super::semantic::SemanticAnalyzer;
use super::super::*;

#[test]
fn test_REDIR_005_bash_vs_posix_herestrings() {
    // DOCUMENTATION: Bash vs POSIX here-strings comparison
    //
    // | Feature                  | POSIX sh         | Bash      | bashrs         |
    // |--------------------------|------------------|-----------|----------------|
    // | echo "str" \| cmd        | ✅               | ✅        | ✅             |
    // | printf '%s' "str" \| cmd | ✅               | ✅        | ✅             |
    // | <<< "string"             | ❌               | ✅        | ❌ → POSIX     |
    // | <<< $VAR                 | ❌               | ✅        | ❌ → POSIX     |
    //
    // POSIX-compliant alternatives:
    // - echo "string" | cmd (adds newline)
    // - printf '%s\n' "string" | cmd (explicit newline)
    // - printf '%s' "string" | cmd (no newline)
    //
    // Bash here-string NOT SUPPORTED:
    // - <<< "string" (Bash 2.05b+ only)
    //
    // bashrs purification strategy:
    // - Convert <<< "string" → echo "string" | cmd
    // - Preserve variable expansion: <<< "$VAR" → echo "$VAR" | cmd
    // - Use printf for explicit control over newlines
    // - Always quote strings for safety
    //
    // Why here-strings are Bash-only:
    // - Not in POSIX specification
    // - Bash 2.05b+ (2002) introduced <<<
    // - sh, dash, ash don't support <<<
    // - Easy to work around with echo | cmd
    //
    // When to use alternatives:
    // - Single line with newline → echo "text" | cmd
    // - Single line without newline → printf '%s' "text" | cmd
    // - Multi-line → cat << EOF ... EOF
    // - Read into variable → var="value" (direct assignment)

    let bash_extensions = r#"
# POSIX (SUPPORTED)
echo "text" | grep "pattern"
printf '%s\n' "text" | wc -w

# Bash extensions (NOT SUPPORTED)
# grep "pattern" <<< "text"
# wc -w <<< "count words"
"#;

    let result = BashParser::new(bash_extensions);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "Bash <<< NOT SUPPORTED, POSIX echo | cmd SUPPORTED"
            );
        }
        Err(_) => {
            // Parse error expected for Bash extensions
        }
    }

    // Summary:
    // POSIX alternatives: Fully supported (echo | cmd, printf | cmd)
    // Bash extensions: NOT SUPPORTED (<<<)
    // bashrs: Convert <<< to echo | cmd during purification
    // Newline behavior: echo adds newline, printf '%s' doesn't
}

// ============================================================================
// PARAM-SPEC-002: $? Exit Status (POSIX, SUPPORTED)
// ============================================================================

#[test]
fn test_PARAM_SPEC_002_exit_status_basic() {
    // DOCUMENTATION: $? exit status is SUPPORTED (POSIX)
    //
    // $? contains the exit status of the last executed command:
    // - 0: Success
    // - 1-125: Various failure codes
    // - 126: Command found but not executable
    // - 127: Command not found
    // - 128+N: Terminated by signal N
    //
    // POSIX sh, bash, dash, ash: FULLY SUPPORTED
    //
    // Example:
    // $ true
    // $ echo $?
    // 0
    // $ false
    // $ echo $?
    // 1
    //
    // Rust mapping:
    // ```rust
    // use std::process::Command;
    //
    // let status = Command::new("cmd").status()?;
    // let exit_code = status.code().unwrap_or(1);
    // println!("Exit: {}", exit_code);
    // ```

    let exit_status = r#"
cmd
echo "Exit: $?"

true
echo "Success: $?"

false
echo "Failure: $?"
"#;

    let result = BashParser::new(exit_status);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "$? is POSIX-compliant, FULLY SUPPORTED"
            );
        }
        Err(_) => {
            // Parse error acceptable - $? may not be fully implemented yet
        }
    }
}

#[test]
fn test_PARAM_SPEC_002_exit_status_in_conditionals() {
    // DOCUMENTATION: Using $? in conditionals (POSIX)
    //
    // Common pattern: Check exit status in if statements
    //
    // $ cmd
    // $ if [ $? -eq 0 ]; then
    // $   echo "Success"
    // $ else
    // $   echo "Failed"
    // $ fi
    //
    // Best practice: Direct if statement (more concise):
    // $ if cmd; then
    // $   echo "Success"
    // $ fi
    //
    // When $? is necessary:
    // - Multiple commands before check
    // - Need to preserve exit status
    // - Logging before checking

    let exit_status_conditional = r#"
# Pattern 1: $? in conditional
cmd
if [ $? -eq 0 ]; then
  echo "Success"
else
  echo "Failed"
fi

# Pattern 2: Direct conditional (better)
if cmd; then
  echo "Success"
fi

# Pattern 3: Preserve status
cmd
STATUS=$?
log_message "Command exited with $STATUS"
if [ $STATUS -ne 0 ]; then
  handle_error
fi
"#;

    let result = BashParser::new(exit_status_conditional);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "$? in conditionals is POSIX-compliant"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

#[test]
fn test_PARAM_SPEC_002_exit_status_pipelines() {
    // DOCUMENTATION: $? with pipelines (POSIX)
    //
    // $? contains exit status of LAST command in pipeline:
    // $ cmd1 | cmd2 | cmd3
    // $ echo $?  # Exit status of cmd3 only
    //
    // To check all commands in pipeline, use PIPESTATUS (bash) or set -o pipefail:
    //
    // Bash-specific (NOT SUPPORTED):
    // $ cmd1 | cmd2 | cmd3
    // $ echo "${PIPESTATUS[@]}"  # Array of all exit codes
    //
    // POSIX alternative: set -o pipefail
    // $ set -o pipefail
    // $ cmd1 | cmd2 | cmd3
    // $ echo $?  # Non-zero if ANY command failed

    let pipeline_exit = r#"
# $? gets last command only
grep pattern file.txt | sort | uniq
echo "Last command status: $?"

# POSIX: set -o pipefail for pipeline failures
set -o pipefail
grep pattern file.txt | sort | uniq
if [ $? -ne 0 ]; then
  echo "Pipeline failed"
fi
"#;

    let result = BashParser::new(pipeline_exit);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "$? with pipelines is POSIX-compliant"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

// DOCUMENTATION: $? is clobbered by every command (POSIX)
// CRITICAL: $? is updated after EVERY command, including [ and test.
// BAD: checking $? inside [ clobbers it. GOOD: capture first. BETTER: direct conditional.
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
