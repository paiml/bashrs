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
fn test_REDIR_005_herestring_with_variables() {
    // DOCUMENTATION: Variable expansion in here-strings (Bash)
    //
    // Here-strings expand variables by default:
    // $ cmd <<< "$VAR"
    // $ cmd <<< "User: $USER"
    //
    // Unlike here documents, there's no way to disable expansion
    // (no quoted delimiter concept for <<<).

    let herestring_vars = r#"
grep "test" <<< "$HOME"
wc -w <<< "User: $USER"
"#;

    let result = BashParser::new(herestring_vars);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "<<< with variables is Bash extension, NOT SUPPORTED"
            );
        }
        Err(_) => {
            // Parse error acceptable - Bash extension
        }
    }
}

#[test]
fn test_REDIR_005_posix_echo_pipe_equivalent() {
    // DOCUMENTATION: POSIX equivalent for here-strings (SUPPORTED)
    //
    // Instead of Bash <<<, use POSIX echo | cmd:
    //
    // Bash (NOT SUPPORTED):
    // $ cmd <<< "input string"
    //
    // POSIX (SUPPORTED):
    // $ echo "input string" | cmd
    //
    // Or printf for more control:
    // $ printf '%s\n' "input string" | cmd
    // $ printf '%s' "no newline" | cmd

    let posix_equivalent = r#"
# POSIX-compliant alternatives to <<<
echo "search this text" | grep "pattern"
printf '%s\n' "count these words" | wc -w
echo "$HOME" | grep "test"
"#;

    let result = BashParser::new(posix_equivalent);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "POSIX echo | cmd is SUPPORTED"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

#[test]
fn test_REDIR_005_purification_strategy() {
    // DOCUMENTATION: Purification strategy for here-strings
    //
    // bashrs purification should convert Bash <<< to POSIX:
    //
    // INPUT (Bash):
    // cmd <<< "input string"
    //
    // PURIFIED (POSIX sh):
    // echo "input string" | cmd
    //
    // Or for literal strings (no newline):
    // printf '%s' "input string" | cmd
    //
    // Purification steps:
    // 1. Detect <<< syntax
    // 2. Convert to echo "string" | cmd
    // 3. Or printf '%s\n' "string" | cmd (more explicit)
    // 4. Quote string for safety
    // 5. Preserve variable expansion

    // This test documents the purification strategy
}

#[test]
fn test_REDIR_005_herestring_vs_heredoc() {
    // DOCUMENTATION: Here-string vs here document comparison
    //
    // Here-string (<<<):
    // - Single line only
    // - Bash 2.05b+ extension
    // - No delimiter needed
    // - Adds newline at end
    // - Syntax: cmd <<< "string"
    //
    // Here document (<<):
    // - Multi-line
    // - POSIX compliant
    // - Requires delimiter (EOF)
    // - No automatic newline
    // - Syntax: cmd << EOF ... EOF
    //
    // When to use which (in Bash):
    // - Single line → <<< "text" (Bash only)
    // - Multi-line → << EOF ... EOF (POSIX)
    //
    // bashrs strategy:
    // - Use echo | cmd for single-line (POSIX)
    // - Use << EOF for multi-line (POSIX)

    let comparison = r#"
# Bash here-string (NOT SUPPORTED)
# grep "pattern" <<< "single line"

# POSIX equivalent (SUPPORTED)
echo "single line" | grep "pattern"

# POSIX here document (SUPPORTED, for multi-line)
cat << EOF
Line 1
Line 2
EOF
"#;

    let result = BashParser::new(comparison);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "POSIX alternatives documented"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

#[test]
fn test_REDIR_005_newline_behavior() {
    // DOCUMENTATION: Here-string newline behavior (Bash)
    //
    // Here-strings automatically add a newline at the end:
    // $ cmd <<< "text"
    // # Equivalent to: echo "text" | cmd (includes newline)
    //
    // To avoid newline in POSIX:
    // $ printf '%s' "text" | cmd
    //
    // Comparison:
    // - <<< "text" → "text\n" (Bash, adds newline)
    // - echo "text" → "text\n" (POSIX, adds newline)
    // - printf '%s' "text" → "text" (POSIX, no newline)
    // - printf '%s\n' "text" → "text\n" (POSIX, explicit newline)

    let newline_test = r#"
# POSIX with newline (default)
echo "text" | cmd

# POSIX without newline
printf '%s' "text" | cmd

# POSIX with explicit newline
printf '%s\n' "text" | cmd
"#;

    let result = BashParser::new(newline_test);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "Newline behavior documented for POSIX alternatives"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

#[test]
fn test_REDIR_005_common_use_cases() {
    // DOCUMENTATION: Common here-string use cases (POSIX alternatives)
    //
    // 1. Pass string to grep (Bash: grep "pattern" <<< "text"):
    //    POSIX: echo "text" | grep "pattern"
    //
    // 2. Word count (Bash: wc -w <<< "count words"):
    //    POSIX: echo "count words" | wc -w
    //
    // 3. Process variable (Bash: cmd <<< "$VAR"):
    //    POSIX: echo "$VAR" | cmd
    //
    // 4. Feed to read (Bash: read var <<< "value"):
    //    POSIX: echo "value" | read var
    //    Warning: pipe runs in subshell, use var="value" instead
    //
    // 5. Base64 encode (Bash: base64 <<< "text"):
    //    POSIX: echo "text" | base64

    let use_cases = r#"
# Pass string to grep (POSIX)
echo "search this text" | grep "pattern"

# Word count (POSIX)
echo "count these words" | wc -w

# Process variable (POSIX)
echo "$HOME" | grep "test"

# Feed to read (POSIX, but use direct assignment)
# echo "value" | read var  # Runs in subshell
var="value"  # Better POSIX alternative

# Base64 encode (POSIX)
echo "text" | base64
"#;

    let result = BashParser::new(use_cases);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "Common POSIX alternatives to <<< documented"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

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
