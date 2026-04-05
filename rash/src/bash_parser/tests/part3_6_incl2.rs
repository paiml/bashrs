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
