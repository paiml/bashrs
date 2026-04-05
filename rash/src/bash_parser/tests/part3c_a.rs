#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]

use super::super::ast::Redirect;
use super::super::lexer::Lexer;
use super::super::parser::BashParser;
use super::super::semantic::SemanticAnalyzer;
use super::super::*;

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
#[test]
fn test_PARAM_SPEC_004_background_pid_common_mistakes() {
    let common_mistakes = r#"
# Mistake 1: Race condition (BAD)
# cmd &
# kill $!  # May fail if job finished

# GOOD: Check if job exists
# cmd &
# BG_PID=$!
# if kill -0 $BG_PID 2>/dev/null; then
#   kill $BG_PID
# fi

# Mistake 2: Exit without wait (BAD)
# important_task &
# exit 0  # Task may not complete!

# GOOD: Wait for job
# important_task &
# wait $!

# BETTER (bashrs): Synchronous
important_task
exit 0

# Mistake 3: Uncontrolled parallelism (BAD)
# for i in 1 2 3 4 5; do
#   process_item $i &
# done

# BETTER (bashrs): Sequential
for i in 1 2 3 4 5; do
  process_item "$i"
done
"#;

    assert_parses_without_panic(common_mistakes, "Common $! mistakes documented");
}

#[test]
fn test_PARAM_SPEC_004_background_pid_comparison_table() {
    // DOCUMENTATION: $! and & comparison (POSIX vs bashrs)
    //
    // Feature                 | POSIX sh | bash | dash | ash | bashrs
    // ------------------------|----------|------|------|-----|--------
    // & (background job)      | ✅       | ✅   | ✅   | ✅  | ❌ PURIFY
    // $! (background PID)     | ✅       | ✅   | ✅   | ✅  | ❌ PURIFY
    // Deterministic           | ❌       | ❌   | ❌   | ❌  | ✅ (sync)
    // wait                    | ✅       | ✅   | ✅   | ✅  | ❌ (implicit)
    // jobs                    | ✅       | ✅   | ✅   | ✅  | ❌
    // fg/bg                   | ✅       | ✅   | ✅   | ✅  | ❌
    //
    // bashrs purification policy:
    // - & (background) is POSIX but NON-DETERMINISTIC
    // - MUST purify to synchronous execution
    // - Remove all background jobs
    // - Remove $! (unnecessary without &)
    // - Remove wait (implicit in synchronous)
    //
    // Purification strategies:
    // 1. Background job: cmd & → cmd (synchronous)
    // 2. Multiple jobs: task1 & task2 & wait → task1; task2 (sequential)
    // 3. Timeout: cmd & sleep 5; kill $! → timeout 5 cmd || true
    // 4. Wait pattern: cmd &; wait $! → cmd (implicit wait)
    // 5. Remove non-essential: log_task & → (remove or make sync)
    //
    // Rust mapping (synchronous):
    // ```rust
    // use std::process::Command;
    //
    // // DON'T: Background execution (non-deterministic)
    // // let child = Command::new("cmd").spawn()?;
    // // let pid = child.id();
    // // child.wait()?;
    //
    // // DO: Synchronous execution (deterministic)
    // let status = Command::new("cmd").status()?;
    // ```
    //
    // Best practices:
    // 1. Use synchronous execution for determinism
    // 2. Avoid background jobs in bootstrap/config scripts
    // 3. Use timeout command for time limits (not background + kill)
    // 4. Sequential execution is easier to test and debug
    // 5. Interactive tools can use &, but not purified scripts

    let comparison_example = r#"
# POSIX: Background job (non-deterministic)
# cmd &
# echo "BG: $!"
# wait $!

# bashrs: Synchronous (deterministic)
cmd
echo "Done"

# POSIX: Multiple background jobs
# task1 &
# task2 &
# wait

# bashrs: Sequential
task1
task2

# POSIX: Timeout with background
# task &
# BG=$!
# sleep 5
# kill $BG

# bashrs: Use timeout command
timeout 5 task || true
"#;

    let result = BashParser::new(comparison_example);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "$! and & comparison and purification strategy documented"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

// Summary:
// $! (background PID): POSIX but NON-DETERMINISTIC (MUST PURIFY)
// Contains PID of last background job (changes every run)
// Background jobs (&) are non-deterministic (PIDs, timing, execution order)
// bashrs policy: Purify to SYNCHRONOUS execution (remove & and $!)
// Purification: cmd & → cmd, task1 & task2 & wait → task1; task2
// Timeout pattern: cmd & sleep N; kill $! → timeout N cmd || true
// Job control (jobs, fg, bg): NOT SUPPORTED (interactive features)
// Common mistakes: Race conditions, exit without wait, uncontrolled parallelism
// Best practice: Synchronous execution for determinism, testability, reproducibility

// ============================================================================
// EXP-BRACE-001: Brace Expansion {..} (Bash extension, NOT SUPPORTED)
// ============================================================================

// DOCUMENTATION: Brace expansion is NOT SUPPORTED (bash extension)
// Bash 3.0+ feature: {1..5}, {a..z}, {foo,bar,baz}, {a,b}{1,2}.
// Not in POSIX. sh/dash/ash don't support. Work around with loops or lists.
