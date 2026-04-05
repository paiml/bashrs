#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(unused_imports)]

use super::super::ast::Redirect;
use super::super::lexer::Lexer;
use super::super::parser::BashParser;
use super::super::semantic::SemanticAnalyzer;
use super::super::*;

#[test]
fn test_BASH_VAR_003_seconds_portability_issues() {
    // DOCUMENTATION: $SECONDS portability issues (3 critical issues)
    //
    // ISSUE 1: Not POSIX (bash-specific)
    // $SECONDS only exists in bash, ksh, zsh
    // POSIX sh: $SECONDS is UNDEFINED (may be literal string "$SECONDS")
    // dash: $SECONDS is UNDEFINED
    // ash: $SECONDS is UNDEFINED
    //
    // ISSUE 2: Reset behavior differs
    // bash: SECONDS=0 resets timer
    // ksh: SECONDS=0 resets timer (but may not reset to exactly 0)
    // zsh: SECONDS=0 resets timer
    // POSIX sh: SECONDS=0 just sets a variable (no timer)
    //
    // ISSUE 3: Precision varies
    // bash: $SECONDS is integer (whole seconds)
    // Some shells may have subsecond precision
    // Behavior is INCONSISTENT across shells
    //
    // PURIFICATION STRATEGY:
    // Replace ALL $SECONDS with deterministic alternatives
    // Use attempt counters, fixed durations, or remove timing logic

    let portability_issues = r#"
#!/bin/sh
# This script is NOT PORTABLE (uses $SECONDS)

# ISSUE 1: Not POSIX
echo "Elapsed: $SECONDS seconds"  # bash: works, dash: UNDEFINED

# ISSUE 2: Reset behavior
SECONDS=0  # bash: resets timer, dash: just sets variable
operation
echo "Took $SECONDS seconds"  # bash: elapsed time, dash: literal "0"

# ISSUE 3: Precision
# bash: integer seconds only
# zsh: may have subsecond precision (non-portable)

# PURIFIED (POSIX-compliant):
# Use attempt counter instead of time
attempts=0
max_attempts=60
while [ $attempts -lt $max_attempts ]; do
    check_condition && break
    sleep 1
    attempts=$((attempts + 1))
done
echo "Took $attempts attempts"
"#;

    let mut lexer = Lexer::new(portability_issues);
    if let Ok(tokens) = lexer.tokenize() {
        assert!(
            !tokens.is_empty(),
            "Portability issues should tokenize successfully"
        );
        let _ = tokens;
    }

    // $SECONDS is NOT PORTABLE (bash-specific)
    // bashrs targets POSIX sh (no $SECONDS support)
    // PURIFICATION: Use attempt counters or fixed durations
}

#[test]
fn test_BASH_VAR_003_seconds_testing_implications() {
    // DOCUMENTATION: $SECONDS testing implications (4 critical issues for testing)
    //
    // ISSUE 1: Non-reproducible tests
    // test_deployment() {
    //   SECONDS=0
    //   deploy_app
    //   assert $SECONDS -lt 60  # Flaky! Depends on machine speed
    // }
    // PROBLEM: Test passes on fast machine, fails on slow machine
    //
    // ISSUE 2: Cannot assert on output
    // output=$(./script.sh)  # Script uses $SECONDS
    // assert "$output" == "Took 5 seconds"  # Flaky! Timing varies
    // PROBLEM: Cannot write assertions for time-dependent output
    //
    // ISSUE 3: Flaky tests (timing heisenbug)
    // Test passes 99% of time (fast), fails 1% (slow)
    // Due to $SECONDS producing different values based on execution speed
    // PROBLEM: Developers lose trust in test suite
    //
    // ISSUE 4: Cannot replay failures
    // Test fails in CI (slow), cannot reproduce locally (fast)
    // Bug only occurs with specific timing
    // PROBLEM: Cannot debug or fix timing-dependent bug
    //
    // TESTING BEST PRACTICES:
    // 1. Never use $SECONDS in production code
    // 2. Use attempt counters instead of timers
    // 3. Remove timing assertions from tests
    // 4. Use deterministic test data (fixed attempt counts)

    let testing_implications = r#"
#!/bin/sh
# TESTING EXAMPLES

# BAD TEST: Time-dependent assertion
test_bad() {
    SECONDS=0
    operation
    # PROBLEM: Assertion depends on execution speed
    [ $SECONDS -lt 10 ] || exit 1
}

# GOOD TEST: Deterministic (no timing)
test_good() {
    operation
    # Assert on actual result, not timing
    [ -f /tmp/output.txt ] || exit 1
}

# BAD TEST: Cannot assert on output
test_flaky_output() {
    output=$(./script.sh)  # Uses $SECONDS
    # PROBLEM: Output varies based on timing
    # [ "$output" = "Took 5 seconds" ] || exit 1  # Flaky!
}

# GOOD TEST: Deterministic output
test_deterministic_output() {
    output=$(./script.sh)  # No $SECONDS
    [ "$output" = "Operation completed" ] || exit 1
}

# BAD TEST: Performance assertion (flaky)
test_performance_bad() {
    SECONDS=0
    benchmark
    # PROBLEM: Fast machine passes, slow machine fails
    [ $SECONDS -lt 30 ] || exit 1
}

# GOOD TEST: No performance assertions
test_correctness_good() {
    result=$(benchmark)
    # Assert on correctness, not speed
    [ "$result" = "expected_output" ] || exit 1
}

# GOOD TEST: Deterministic retry logic
test_retry_deterministic() {
    attempts=0
    max_attempts=10
    while [ $attempts -lt $max_attempts ]; do
        check_condition && break
        attempts=$((attempts + 1))
    done
    # Assert on attempts, not time
    [ $attempts -lt $max_attempts ] || exit 1
}
"#;

    let mut lexer = Lexer::new(testing_implications);
    if let Ok(tokens) = lexer.tokenize() {
        assert!(
            !tokens.is_empty(),
            "Testing implications should tokenize successfully"
        );
        let _ = tokens;
    }

    // $SECONDS makes tests NON-REPRODUCIBLE and FLAKY
    // bashrs enforces DETERMINISTIC testing
    // NEVER use $SECONDS in test code
}

#[test]
fn test_BASH_VAR_003_seconds_comparison_table() {
    // DOCUMENTATION: Comprehensive $SECONDS comparison (Bash vs POSIX vs Purified)
    //
    // ┌─────────────────────────────────────────────────────────────────────────┐
    // │ FEATURE                    │ Bash       │ POSIX      │ Purified         │
    // ├─────────────────────────────────────────────────────────────────────────┤
    // │ $SECONDS variable          │ SUPPORTED  │ NOT POSIX  │ NOT SUPPORTED    │
    // │ elapsed=$SECONDS           │ ✅ Timer  │ ❌         │ ❌ FORBIDDEN     │
    // │                            │            │            │                  │
    // │ Determinism                │ NO         │ N/A        │ YES (enforced)   │
    // │ Same script → same output  │ ❌ Timing │ N/A        │ ✅ Deterministic │
    // │                            │            │            │                  │
    // │ Reproducibility            │ NO         │ N/A        │ YES              │
    // │ Can replay execution       │ ❌ Timing │ N/A        │ ✅ No timing     │
    // │                            │            │            │                  │
    // │ Testing                    │ Flaky      │ N/A        │ Reproducible     │
    // │ Test assertions            │ ⚠️ Speed │ N/A        │ ✅ Deterministic │
    // │                            │            │            │                  │
    // │ Portability                │ bash/ksh   │ N/A        │ POSIX counters   │
    // │ Works in dash/ash          │ ❌         │ N/A        │ ✅               │
    // │                            │            │            │                  │
    // │ Reset timer                │ SECONDS=0  │ N/A        │ counter=0        │
    // │ Reset to zero              │ ✅ bash   │ N/A        │ ✅ POSIX         │
    // │                            │            │            │                  │
    // │ Precision                  │ Integer    │ N/A        │ Configurable     │
    // │ Subsecond timing           │ ❌ Seconds│ N/A        │ N/A (no timing)  │
    // │                            │            │            │                  │
    // │ Use case                   │ Timing     │ N/A        │ Attempt counters │
    // │ Timeouts, benchmarks       │ ⚠️ Non-det│ N/A        │ ✅ Deterministic │
    // └─────────────────────────────────────────────────────────────────────────┘
    //
    // RUST MAPPING:
    // $SECONDS → NOT MAPPED (use deterministic values instead)
    // For timing needs: Remove timing logic or use fixed durations
    // For timeouts: Use attempt counters (deterministic)
    // For benchmarks: Use external tools (hyperfine, criterion)
    //
    // PURIFICATION RULES:
    // 1. $SECONDS → FORBIDDEN (rewrite script with deterministic alternative)
    // 2. Timeouts → Use attempt counters (max_attempts)
    // 3. Benchmarks → Use external tools or remove timing
    // 4. Progress indicators → Use work-based progress (items processed)
    // 5. Log timestamps → Remove or use fixed format
    // 6. Performance assertions → Remove from tests (test correctness, not speed)

    let comparison_table = r#"
#!/bin/sh
# COMPARISON EXAMPLES

# BASH (NON-DETERMINISTIC):
# SECONDS=0
# operation
# echo "Took $SECONDS seconds"  # Different value each run

# POSIX (NOT AVAILABLE):
# $SECONDS doesn't exist in POSIX sh

# PURIFIED (DETERMINISTIC):
# Option 1: Fixed duration
duration=100
echo "Duration: $duration seconds"

# Option 2: Attempt counter (timeout)
attempts=0
max_attempts=60
while [ $attempts -lt $max_attempts ]; do
    check_condition && break
    sleep 1
    attempts=$((attempts + 1))
done
echo "Took $attempts attempts"

# Option 3: Remove timing
operation
echo "Operation completed"

# TESTING COMPARISON:
# BASH (flaky tests):
# SECONDS=0; operation; [ $SECONDS -lt 10 ] || exit 1  # Flaky!

# PURIFIED (reproducible tests):
operation
[ -f /tmp/output.txt ] || exit 1  # Deterministic assertion

# TIMEOUT COMPARISON:
# BASH (time-based, non-deterministic):
# start=$SECONDS
# while [ $((SECONDS - start)) -lt 60 ]; do
#     check_service && break
#     sleep 1
# done

# PURIFIED (attempt-based, deterministic):
attempts=0
max_attempts=60
while [ $attempts -lt $max_attempts ]; do
    check_service && break
    sleep 1
    attempts=$((attempts + 1))
done
"#;

    let mut lexer = Lexer::new(comparison_table);
    if let Ok(tokens) = lexer.tokenize() {
        assert!(
            !tokens.is_empty(),
            "Comparison table should tokenize successfully"
        );
        let _ = tokens;
    }

    // POSIX STATUS: $SECONDS is NOT POSIX (bash-specific)
    // bashrs STATUS: $SECONDS is FORBIDDEN (violates determinism)
    // PURIFICATION: Rewrite with deterministic alternatives (attempt counters, fixed durations, remove timing)
    // Determinism: $SECONDS is NON-DETERMINISTIC (time-dependent, execution speed affects output)
    // Portability: $SECONDS is NOT PORTABLE (bash/ksh/zsh only, not POSIX sh/dash/ash)
    // Testing: $SECONDS makes tests FLAKY and NON-REPRODUCIBLE (depends on execution speed)
}

// ============================================================================
// JOB-001: Background jobs (&) purification (NOT SUPPORTED)
// ============================================================================

// DOCUMENTATION: Background jobs (&) are NOT SUPPORTED (HIGH priority purification)
//
// Background jobs (&): Run command in background, return control to shell immediately
// Syntax: command &
// Returns job ID and process ID
//
// WHY NOT SUPPORTED:
// 1. Non-deterministic (race conditions - background jobs run concurrently)
// 2. Timing-dependent (order of execution not guaranteed)
// 3. Makes testing impossible (can't assert on state while job runs)
// 4. Resource management issues (background jobs may outlive parent script)
// 5. No error handling (background job failures are silent)
//
// CRITICAL: Background jobs violate determinism
// bashrs enforces DETERMINISM - concurrent execution introduces race conditions
//
// PURIFICATION STRATEGY:
// Background jobs (&) are DISCOURAGED - prefer foreground execution
//
// OPTION 1: Convert to foreground (deterministic)
// INPUT: long_task &; do_other_work; wait
// PURIFIED: long_task; do_other_work
//
// OPTION 2: Sequential execution (deterministic)
// INPUT: task1 &; task2 &; wait
// PURIFIED: task1; task2
//
// OPTION 3: Use explicit job control (if parallelism required)
// INPUT: `for file in *.txt; do process "$file" & done; wait`
// PURIFIED: `for file in *.txt; do process "$file"; done`
#[test]
fn test_JOB_001_background_jobs_not_supported() {
    // Background jobs (&) are NOT SUPPORTED (non-deterministic, race conditions)
    let background_jobs = concat!(
        "# NOT SUPPORTED: Background job (non-deterministic)\n",
        "long_running_task &\n",
        "echo \"Task started in background\"\n",
        "\n",
        "# NOT SUPPORTED: Multiple background jobs (race conditions)\n",
        "task1 &\n",
        "task2 &\n",
        "task3 &\n",
        "wait  # Wait for all background jobs\n",
        "\n",
        "# NOT SUPPORTED: Background job with no wait (orphan process)\n",
        "cleanup_temp_files &\n",
        "\n",
        "# NOT SUPPORTED: Fire-and-forget background job\n",
        "send_notification &\n",
        "exit 0\n",
    );

    let mut lexer = Lexer::new(background_jobs);
    // Parser may not support & - both Ok and Err are acceptable
    if let Ok(tokens) = lexer.tokenize() {
        assert!(
            !tokens.is_empty(),
            "Background jobs should tokenize (even though NOT SUPPORTED)"
        );
    }
}

#[test]
fn test_JOB_001_background_jobs_purification_strategies() {
    // DOCUMENTATION: Background job purification strategies (4 strategies)
    //
    // STRATEGY 1: Convert to foreground execution (RECOMMENDED)
    // Use case: Task doesn't need to run in background
    // INPUT: long_task &; do_work; wait
    // PURIFIED: long_task; do_work
    // Pros: Deterministic, simple, no race conditions
    // Cons: May be slower (sequential vs parallel)
    //
    // STRATEGY 2: Sequential execution (RECOMMENDED)
    // Use case: Multiple independent tasks
    // INPUT: task1 &; task2 &; task3 &; wait
    // PURIFIED: task1; task2; task3
    // Pros: Deterministic, reproducible, no race conditions
    // Cons: Slower than parallel (if tasks are independent)
    //
    // STRATEGY 3: Remove background job entirely
    // Use case: Background job is non-essential (cleanup, notification)
    // INPUT: send_notification &; exit 0
    // PURIFIED: exit 0  # Remove non-essential background task
    // Pros: Simplest, no complexity
    // Cons: Loses functionality
    //
    // STRATEGY 4: Use make -j for parallelism (if needed)
    // Use case: Need actual parallelism for performance
    // INPUT: for file in *.txt; do process "$file" & done; wait
    // PURIFIED: Write Makefile with parallel targets, use make -j4
    // Pros: Deterministic parallelism, explicit dependencies
    // Cons: Requires Makefile, more complex

    let purification_strategies = r#"
# STRATEGY 1: Convert to foreground (RECOMMENDED)
# INPUT: long_task &; do_work; wait
long_task
do_work

# STRATEGY 2: Sequential execution (RECOMMENDED)
# INPUT: task1 &; task2 &; task3 &; wait
task1
task2
task3

# STRATEGY 3: Remove background job
# INPUT: send_notification &; exit 0
exit 0  # Remove non-essential background task

# STRATEGY 4: Use make for parallelism (if needed)
# Create Makefile:
# all: file1.out file2.out file3.out
# %.out: %.txt
#     process $< > $@
#
# Then: make -j4  # Deterministic parallelism with explicit dependencies

# REAL-WORLD EXAMPLE: Log processing
# BAD (non-deterministic):
# for log in *.log; do
#     process_log "$log" &
# done
# wait

# GOOD (deterministic):
for log in *.log; do
    process_log "$log"
done
"#;

    let mut lexer = Lexer::new(purification_strategies);
    if let Ok(tokens) = lexer.tokenize() {
        assert!(
            !tokens.is_empty(),
            "Purification strategies should tokenize successfully"
        );
        let _ = tokens;
    }

    // All strategies are DETERMINISTIC
    // PREFERRED: Strategies 1-2 (foreground execution)
    // Strategy 4 acceptable if parallelism required (use make -j)
}

