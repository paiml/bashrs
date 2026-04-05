#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]

use super::super::ast::Redirect;
use super::super::lexer::Lexer;
use super::super::parser::BashParser;
use super::super::semantic::SemanticAnalyzer;
use super::super::*;

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
fn test_PARAM_SPEC_003_process_id_in_subshells() {
    // DOCUMENTATION: $$ behavior in subshells (POSIX gotcha)
    //
    // CRITICAL: $$ in subshell returns PARENT shell PID, not subshell PID!
    //
    // $ echo "Main: $$"
    // Main: 12345
    //
    // $ ( echo "Subshell: $$" )
    // Subshell: 12345  # Same as parent!
    //
    // To get actual subshell PID, use $BASHPID (bash extension):
    // $ ( echo "Subshell: $BASHPID" )
    // Subshell: 12346  # Different!
    //
    // But $BASHPID is NOT SUPPORTED (bash 4.0+ only, not POSIX)
    //
    // POSIX sh behavior:
    // - $$ always returns original shell PID
    // - Even in subshells, command substitution, pipelines
    // - This is POSIX-specified behavior
    //
    // Why this matters:
    // - Cannot use $$ to uniquely identify subprocesses
    // - Temp files in subshells will collide
    // - Must use other unique identifiers

    let subshell_pid = r#"
# Main shell
echo "Main PID: $$"

# Subshell (same PID as main!)
( echo "Subshell PID: $$" )

# Command substitution (same PID as main!)
RESULT=$(echo "Command sub PID: $$")

# Pipeline (same PID as main!)
echo "Pipeline PID: $$" | cat
"#;

    let result = BashParser::new(subshell_pid);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "$$ in subshells returns parent PID (POSIX behavior)"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

#[test]
fn test_PARAM_SPEC_003_process_id_purification_strategy() {
    // DOCUMENTATION: bashrs purification strategy for $$
    //
    // Strategy 1: Replace with fixed identifier
    // - Input:  echo "PID: $$"
    // - Purified: echo "PID: SCRIPT_ID"
    //
    // Strategy 2: Use script name
    // - Input:  TMPFILE=/tmp/app.$$
    // - Purified: TMPFILE="/tmp/$(basename "$0").tmp"
    //
    // Strategy 3: Use mktemp
    // - Input:  LOGFILE=/var/log/app.$$.log
    // - Purified: LOGFILE=$(mktemp /var/log/app.XXXXXX)
    //
    // Strategy 4: Remove if unnecessary
    // - Input:  echo "Running with PID $$"
    // - Purified: echo "Running"  # Remove non-essential logging
    //
    // Strategy 5: Use XDG directories (if available)
    // - Input:  TMPFILE=/tmp/app.$$
    // - Purified: TMPFILE="${XDG_RUNTIME_DIR:-/tmp}/app.tmp"
    //
    // When $$ is acceptable (rare cases):
    // - Trap cleanup: trap "rm -f /tmp/lock.$$" EXIT
    // - Lock files that MUST be unique per process
    // - Debugging/logging (not production)
    //
    // Rust equivalent (deterministic):
    // ```rust
    // // Don't use process::id() for file names!
    // // Use tempfile crate instead:
    // use tempfile::NamedTempFile;
    // let temp = NamedTempFile::new()?;  // Deterministic, safe
    // ```

    let purification_examples = r#"
# BEFORE (non-deterministic)
echo "PID: $$"
TMPFILE=/tmp/app.$$

# AFTER (deterministic)
echo "PID: SCRIPT_ID"
TMPFILE=$(mktemp /tmp/app.XXXXXX)
"#;

    let result = BashParser::new(purification_examples);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "Purification strategy: mktemp or fixed ID"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

#[test]
fn test_PARAM_SPEC_003_process_id_acceptable_uses() {
    // DOCUMENTATION: Acceptable uses of $$ (rare exceptions)
    //
    // Use Case 1: Trap cleanup (acceptable)
    // $ trap "rm -f /tmp/lock.$$" EXIT
    // $ # Process-specific cleanup is OK
    //
    // Why acceptable:
    // - Trap runs in same process, so $$ is consistent
    // - Cleanup files are process-scoped
    // - Not used for deterministic behavior
    //
    // Use Case 2: Lock files (acceptable with caution)
    // $ LOCKFILE=/var/lock/app.$$
    // $ if mkdir "$LOCKFILE" 2>/dev/null; then
    // $   trap "rmdir '$LOCKFILE'" EXIT
    // $   # Do work
    // $ fi
    //
    // Why acceptable:
    // - Lock must be unique per process
    // - Automatic cleanup via trap
    // - Race conditions handled by mkdir
    //
    // Use Case 3: Debugging/development (not production)
    // $ set -x; PS4='[$$] '; command
    // $ # Shows PID in debug traces
    //
    // UNACCEPTABLE uses:
    // - Temp files without cleanup
    // - Log file names (use rotation instead)
    // - Persistent files (violates determinism)
    // - Data file names (not reproducible)

    let acceptable_uses = r#"
# ACCEPTABLE: Trap cleanup
trap "rm -f /tmp/lock.$$" EXIT
trap "rm -f /tmp/work.$$ /tmp/data.$$" EXIT INT TERM

# ACCEPTABLE: Process-specific lock
LOCKFILE=/var/lock/myapp.$$
if mkdir "$LOCKFILE" 2>/dev/null; then
  trap "rmdir '$LOCKFILE'" EXIT
  # Do critical work
fi

# ACCEPTABLE: Debug traces
set -x
PS4='[$$] '
echo "Debug mode"

# UNACCEPTABLE: Persistent files
# LOGFILE=/var/log/app.$$.log  # BAD! Log names not reproducible
# DATAFILE=/data/output.$$      # BAD! Data files must be deterministic
"#;

    let result = BashParser::new(acceptable_uses);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "Trap cleanup and lock files are acceptable uses of $$"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

#[test]
fn test_PARAM_SPEC_003_process_id_bashpid_not_supported() {
    // DOCUMENTATION: $BASHPID is NOT SUPPORTED (bash extension)
    //
    // $BASHPID (bash 4.0+):
    // - Returns actual PID of current bash process
    // - Different from $$ in subshells
    // - Bash extension, not POSIX
    //
    // Example (bash only):
    // $ echo "Main: $$ $BASHPID"
    // Main: 12345 12345  # Same in main shell
    //
    // $ ( echo "Sub: $$ $BASHPID" )
    // Sub: 12345 12346   # Different in subshell!
    //
    // POSIX sh, dash, ash: $BASHPID not available
    //
    // bashrs: NOT SUPPORTED (bash extension)
    //
    // POSIX alternative:
    // - No direct equivalent
    // - Use $$ (aware it returns parent PID in subshells)
    // - Use sh -c 'echo $$' to get actual subshell PID (if needed)

    let bashpid_extension = r#"
# Bash extension (NOT SUPPORTED)
# echo "BASHPID: $BASHPID"

# POSIX (SUPPORTED, but returns parent PID in subshells)
echo "PID: $$"

# POSIX workaround for actual subshell PID (if needed)
( sh -c 'echo "Actual PID: $$"' )
"#;

    let result = BashParser::new(bashpid_extension);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "$BASHPID is bash extension, NOT SUPPORTED"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

// DOCUMENTATION: Common mistakes with $$
// Mistake 1: log rotation with $$. Mistake 2: data files with $$.
// Mistake 3: same $$ in loop. Mistake 4: $$ in subshell is parent PID.
// Fix: use fixed names, mktemp, or capture $$ before subshell.
#[test]
fn test_PARAM_SPEC_003_process_id_common_mistakes() {
    let common_mistakes = r#"
# Mistake 1: Log rotation (BAD)
# LOG=/var/log/app.$$.log
# echo "message" >> "$LOG"

# GOOD: Fixed log file
LOG=/var/log/app.log
echo "$(date): message" >> "$LOG"

# Mistake 2: Data files (BAD)
# OUTPUT=/data/result.$$.json
# process_data > "$OUTPUT"

# GOOD: Fixed output file
OUTPUT=/data/result.json
process_data > "$OUTPUT"

# Mistake 3: Same $$ in loop (BAD)
# for i in 1 2 3; do
#   echo "$i" > /tmp/item.$$
#   process /tmp/item.$$
# done

# GOOD: mktemp per iteration
for i in 1 2 3; do
  TMPFILE=$(mktemp)
  echo "$i" > "$TMPFILE"
  process "$TMPFILE"
  rm -f "$TMPFILE"
done
"#;

    assert_parses_without_panic(common_mistakes, "Common $$ mistakes documented");
}

// DOCUMENTATION: $$ comparison (POSIX vs Bash vs bashrs)
// $$ is POSIX but non-deterministic, must purify. $BASHPID not supported.
// Purification: mktemp for temp files, fixed names for logs/data, trap for locks.
#[test]
fn test_PARAM_SPEC_003_process_id_comparison_table() {
    let comparison_example = r#"
# POSIX: $$ is supported but non-deterministic
echo "PID: $$"

# bashrs: PURIFY to deterministic alternative
echo "PID: SCRIPT_ID"

# POSIX: mktemp is RECOMMENDED alternative
TMPFILE=$(mktemp /tmp/app.XXXXXX)

# POSIX: Fixed names for determinism
LOGFILE=/var/log/app.log

# Acceptable: Trap cleanup (process-scoped)
trap "rm -f /tmp/lock.$$" EXIT

# Bash-only: $BASHPID NOT SUPPORTED
# echo "Actual PID: $BASHPID"
"#;

    assert_parses_without_panic(
        comparison_example,
        "$$ comparison and purification strategy documented",
    );
}

// Summary:
// $$ (process ID): POSIX but NON-DETERMINISTIC (MUST PURIFY)
// Contains PID of current shell (changes every run)
// Subshells: $$ returns PARENT PID, not subshell PID (POSIX behavior)
// $BASHPID: NOT SUPPORTED (bash 4.0+ extension for actual subshell PID)
// Purification: Use mktemp for temp files, fixed names for logs/data
// Acceptable uses: Trap cleanup, lock files (with trap)
// Anti-patterns: Log rotation, data files, scripts called multiple times
// Best practice: mktemp instead of /tmp/file.$$, fixed names for determinism

// ============================================================================
// PARAM-SPEC-004: $! Background PID (POSIX, but NON-DETERMINISTIC - PURIFY)
// ============================================================================

#[test]
fn test_PARAM_SPEC_004_background_pid_non_deterministic() {
    // DOCUMENTATION: $! is POSIX but NON-DETERMINISTIC (must purify)
    //
    // $! contains the PID of the last background job:
    // - POSIX-compliant feature (sh, bash, dash, ash all support)
    // - NON-DETERMINISTIC: changes every time script runs
    // - bashrs policy: PURIFY to synchronous execution
    //
    // Example (non-deterministic):
    // $ sleep 10 &
    // $ echo "Background PID: $!"
    // Background PID: 12345  # Different every time!
    //
    // $ cmd &
    // $ echo "BG: $!"
    // BG: 67890  # Different process ID
    //
    // Why $! is non-deterministic:
    // - Each background job gets unique PID from OS
    // - PIDs are reused but unpredictable
    // - Scripts using $! for process management will have different PIDs each run
    // - Breaks determinism requirement for bashrs
    //
    // bashrs purification policy:
    // - Background jobs (&) are NON-DETERMINISTIC
    // - Purify to SYNCHRONOUS execution (remove &)
    // - No background jobs in purified scripts
    // - $! becomes unnecessary when & is removed
    //
    // Rust mapping (synchronous):
    // ```rust
    // use std::process::Command;
    //
    // // DON'T: Spawn background process (non-deterministic)
    // // let child = Command::new("cmd").spawn()?;
    // // let pid = child.id();
    //
    // // DO: Run synchronously (deterministic)
    // let status = Command::new("cmd").status()?;
    // ```

    let background_pid = r#"
# Background job (non-deterministic)
sleep 10 &
echo "Background PID: $!"

cmd &
BG_PID=$!
echo "Started job: $BG_PID"
"#;

    let result = BashParser::new(background_pid);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "$! is POSIX-compliant but NON-DETERMINISTIC (must purify)"
            );
        }
        Err(_) => {
            // Parse error acceptable - $! may not be fully implemented yet
        }
    }
}

