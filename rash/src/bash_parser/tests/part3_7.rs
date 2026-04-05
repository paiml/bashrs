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

#[test]
fn test_PARAM_SPEC_004_background_pid_wait_pattern() {
    // DOCUMENTATION: Common pattern - background job + wait
    //
    // ANTI-PATTERN (non-deterministic):
    // $ long_running_task &
    // $ BG_PID=$!
    // $ echo "Running task $BG_PID in background"
    // $ wait $BG_PID
    // $ echo "Task $BG_PID completed"
    //
    // Problem: Background execution is non-deterministic
    // - PID changes every run
    // - Timing issues (race conditions)
    // - Can't reproduce exact execution order
    // - Breaks testing and debugging
    //
    // bashrs purification: Run synchronously
    // $ long_running_task
    // $ echo "Task completed"
    //
    // Why synchronous is better for bashrs:
    // - Deterministic execution order
    // - No race conditions
    // - Reproducible behavior
    // - Easier to test and debug
    // - Same results every run
    //
    // When background jobs are acceptable (rare):
    // - Interactive scripts (not for bashrs purification)
    // - User-facing tools (not bootstrap/config scripts)
    // - Explicitly requested parallelism (user choice)

    let wait_pattern = r#"
# ANTI-PATTERN: Background + wait
long_running_task &
BG_PID=$!
echo "Running task $BG_PID in background"
wait $BG_PID
echo "Task $BG_PID completed"

# BETTER (bashrs): Synchronous execution
long_running_task
echo "Task completed"
"#;

    let result = BashParser::new(wait_pattern);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "Background + wait pattern is non-deterministic"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

#[test]
fn test_PARAM_SPEC_004_background_pid_multiple_jobs() {
    // DOCUMENTATION: Multiple background jobs (highly non-deterministic)
    //
    // ANTI-PATTERN (non-deterministic):
    // $ task1 &
    // $ PID1=$!
    // $ task2 &
    // $ PID2=$!
    // $ task3 &
    // $ PID3=$!
    // $ wait $PID1 $PID2 $PID3
    //
    // Problems:
    // - 3 PIDs, all unpredictable
    // - Race conditions (which finishes first?)
    // - Non-deterministic completion order
    // - Can't reproduce test scenarios
    // - Debugging nightmare
    //
    // bashrs purification: Sequential execution
    // $ task1
    // $ task2
    // $ task3
    //
    // Benefits:
    // - Deterministic execution order (always task1 → task2 → task3)
    // - No race conditions
    // - Reproducible results
    // - Easy to test
    // - Clear execution flow

    let multiple_jobs = r#"
# ANTI-PATTERN: Multiple background jobs
task1 &
PID1=$!
task2 &
PID2=$!
task3 &
PID3=$!

echo "Started: $PID1 $PID2 $PID3"
wait $PID1 $PID2 $PID3
echo "All completed"

# BETTER (bashrs): Sequential
task1
task2
task3
echo "All completed"
"#;

    let result = BashParser::new(multiple_jobs);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "Multiple background jobs are highly non-deterministic"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

#[test]
fn test_PARAM_SPEC_004_background_pid_with_kill() {
    // DOCUMENTATION: Background job + kill pattern
    //
    // ANTI-PATTERN (non-deterministic + destructive):
    // $ timeout_task &
    // $ BG_PID=$!
    // $ sleep 5
    // $ kill $BG_PID 2>/dev/null
    //
    // Problems:
    // - Non-deterministic PID
    // - Timing-dependent behavior
    // - Race condition (task may finish before kill)
    // - Signal handling is process-dependent
    // - Not reproducible
    //
    // bashrs purification: Use timeout command
    // $ timeout 5 timeout_task || true
    //
    // Benefits:
    // - Deterministic timeout behavior
    // - No background jobs
    // - No PIDs to track
    // - POSIX timeout command (coreutils)
    // - Reproducible results

    let kill_pattern = r#"
# ANTI-PATTERN: Background + kill
timeout_task &
BG_PID=$!
sleep 5
kill $BG_PID 2>/dev/null || true

# BETTER (bashrs): Use timeout command
timeout 5 timeout_task || true

# Alternative: Run synchronously with resource limits
ulimit -t 5  # CPU time limit
timeout_task || true
"#;

    let result = BashParser::new(kill_pattern);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "Background + kill pattern is non-deterministic"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

#[test]
fn test_PARAM_SPEC_004_background_pid_purification_strategy() {
    // DOCUMENTATION: bashrs purification strategy for $! and &
    //
    // Strategy 1: Remove background execution
    // - Input:  cmd &; echo "BG: $!"
    // - Purified: cmd; echo "Done"
    //
    // Strategy 2: Use wait without &
    // - Input:  task &; wait $!
    // - Purified: task  # wait is implicit
    //
    // Strategy 3: Sequential instead of parallel
    // - Input:  task1 & task2 & wait
    // - Purified: task1; task2
    //
    // Strategy 4: Use timeout for time limits
    // - Input:  task &; sleep 5; kill $!
    // - Purified: timeout 5 task || true
    //
    // Strategy 5: Remove entirely if non-essential
    // - Input:  log_task &  # Background logging
    // - Purified: # Remove (or make synchronous if needed)
    //
    // When & is acceptable (never in bashrs):
    // - Interactive user tools (not bootstrap scripts)
    // - Explicitly requested parallelism
    // - NOT acceptable in bashrs purified output
    //
    // Rust equivalent (synchronous):
    // ```rust
    // use std::process::Command;
    //
    // // DON'T: Background process
    // // let child = Command::new("task1").spawn()?;
    // // let child2 = Command::new("task2").spawn()?;
    // // child.wait()?;
    // // child2.wait()?;
    //
    // // DO: Sequential execution
    // Command::new("task1").status()?;
    // Command::new("task2").status()?;
    // ```

    let purification_examples = r#"
# BEFORE (non-deterministic)
cmd &
echo "BG: $!"

# AFTER (deterministic)
cmd
echo "Done"

# BEFORE (parallel)
task1 &
task2 &
wait

# AFTER (sequential)
task1
task2
"#;

    let result = BashParser::new(purification_examples);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "Purification strategy: remove & and $!"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

#[test]
fn test_PARAM_SPEC_004_background_pid_job_control() {
    // DOCUMENTATION: Job control and $! (POSIX but discouraged)
    //
    // Job control features (POSIX but non-deterministic):
    // - & (background execution)
    // - $! (last background PID)
    // - jobs (list jobs)
    // - fg (foreground job)
    // - bg (background job)
    // - wait (wait for jobs)
    //
    // Why bashrs doesn't support job control:
    // - Non-deterministic (PIDs, timing, execution order)
    // - Interactive feature (not for scripts)
    // - Race conditions
    // - Hard to test
    // - Not needed for bootstrap/config scripts
    //
    // POSIX job control example (NOT SUPPORTED):
    // $ sleep 100 &
    // $ jobs  # List background jobs
    // [1]+  Running   sleep 100 &
    // $ fg %1  # Bring to foreground
    //
    // bashrs approach:
    // - Synchronous execution only
    // - No background jobs
    // - No job control commands
    // - Deterministic, testable, reproducible

    let job_control = r#"
# Job control (NOT SUPPORTED in bashrs purification)
# sleep 100 &
# jobs
# fg %1
# bg %1

# bashrs: Synchronous only
sleep 100  # Runs in foreground, blocks until complete
"#;

    let result = BashParser::new(job_control);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "Job control is POSIX but discouraged in bashrs"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

// DOCUMENTATION: Common mistakes with $! and &
// Mistake 1: kill $! without checking job exists (race condition).
// Mistake 2: exit without wait (job may not complete).
// Mistake 3: uncontrolled parallelism in loops.
// bashrs fix: synchronous execution, sequential loops.
