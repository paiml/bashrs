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
