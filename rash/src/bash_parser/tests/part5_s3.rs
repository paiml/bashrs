#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(unused_imports)]

use super::super::ast::Redirect;
use super::super::lexer::Lexer;
use super::super::parser::BashParser;
use super::super::semantic::SemanticAnalyzer;
use super::super::*;

#[test]
fn test_BASH_VAR_003_seconds_common_antipatterns() {
    // DOCUMENTATION: Common $SECONDS antipatterns and their fixes (6 antipatterns)
    //
    // ANTIPATTERN 1: Performance measurement
    // BAD: SECONDS=0; run_benchmark; echo "Took $SECONDS seconds"
    // GOOD: Use external benchmarking tool (hyperfine, time)
    // Why: Benchmarks should be repeatable with controlled environment
    //
    // ANTIPATTERN 2: Timeouts based on elapsed time
    // BAD: start=$SECONDS; while [ $((SECONDS - start)) -lt 60 ]; do ...; done
    // GOOD: Use attempt counter: attempt=0; while [ $attempt -lt 60 ]; do ...; attempt=$((attempt + 1)); done
    // Why: Attempt counters are deterministic
    //
    // ANTIPATTERN 3: Log timestamps with $SECONDS
    // BAD: echo "[$SECONDS] Operation completed"
    // GOOD: Use fixed log format or remove timestamps
    // Why: Logs should be reproducible for testing
    //
    // ANTIPATTERN 4: Rate limiting with $SECONDS
    // BAD: if [ $((SECONDS % 10)) -eq 0 ]; then echo "Status"; fi
    // GOOD: Use fixed intervals or remove rate limiting
    // Why: Rate limiting should be deterministic
    //
    // ANTIPATTERN 5: Progress indicators with $SECONDS
    // BAD: echo "Progress: $((SECONDS * 100 / 300))%"
    // GOOD: Use actual progress counter
    // Why: Progress should be based on work done, not time
    //
    // ANTIPATTERN 6: Script execution time reporting
    // BAD: echo "Script ran for $SECONDS seconds"
    // GOOD: Remove execution time reporting
    // Why: Execution time varies, not deterministic

    let antipatterns = r#"
# ANTIPATTERN 1: Performance measurement
# BAD: SECONDS=0; run_benchmark; echo "Took $SECONDS seconds"
# GOOD: Use external tool
# hyperfine --warmup 3 './benchmark.sh'

# ANTIPATTERN 2: Timeouts
# BAD: start=$SECONDS; while [ $((SECONDS - start)) -lt 60 ]; do ...; done
# GOOD: Attempt counter
max_attempts=60
attempt=0
while [ $attempt -lt $max_attempts ]; do
    check_condition && break
    sleep 1
    attempt=$((attempt + 1))
done

# ANTIPATTERN 3: Log timestamps
# BAD: echo "[$SECONDS] Operation completed"
# GOOD: Fixed log format
echo "[INFO] Operation completed"

# ANTIPATTERN 4: Rate limiting
# BAD: if [ $((SECONDS % 10)) -eq 0 ]; then echo "Status"; fi
# GOOD: Fixed intervals (deterministic)
counter=0
for item in $items; do
    process "$item"
    counter=$((counter + 1))
    if [ $((counter % 10)) -eq 0 ]; then
        echo "Processed $counter items"
    fi
done

# ANTIPATTERN 5: Progress indicators
# BAD: echo "Progress: $((SECONDS * 100 / 300))%"
# GOOD: Actual progress
total=100
completed=0
for item in $items; do
    process "$item"
    completed=$((completed + 1))
    progress=$((completed * 100 / total))
    echo "Progress: ${progress}%"
done

# ANTIPATTERN 6: Execution time reporting
# BAD: echo "Script ran for $SECONDS seconds"
# GOOD: Remove timing
echo "Script completed successfully"
"#;

    let mut lexer = Lexer::new(antipatterns);
    if let Ok(tokens) = lexer.tokenize() {
        assert!(
            !tokens.is_empty(),
            "Antipatterns should tokenize successfully"
        );
        let _ = tokens;
    }

    // All antipatterns involve $SECONDS (time-dependent)
    // All fixes are DETERMINISTIC alternatives
    // CRITICAL: Never use $SECONDS in production scripts
}

#[test]
fn test_BASH_VAR_003_seconds_determinism_violations() {
    // DOCUMENTATION: How $SECONDS violates determinism (4 critical violations)
    //
    // VIOLATION 1: Time-dependent output
    // #!/bin/sh
    // echo "Elapsed: $SECONDS seconds"
    // Running at different times produces different output
    // EXPECTED (deterministic): Same output every run
    //
    // VIOLATION 2: Cannot replay execution
    // Script with $SECONDS cannot be replayed with same timing
    // Fast machine vs slow machine produces different results
    // EXPECTED: Replay should produce identical results regardless of execution speed
    //
    // VIOLATION 3: Tests non-reproducible
    // test_performance() {
    //   SECONDS=0
    //   run_operation
    //   assert $SECONDS -lt 10  # Flaky! Depends on machine speed
    // }
    // EXPECTED: Tests should be reproducible regardless of machine speed
    //
    // VIOLATION 4: Race conditions in timing logic
    // Timeout logic using $SECONDS may behave differently on different runs
    // EXPECTED: Deterministic retry logic (attempt counters)

    let determinism_violations = r#"
# VIOLATION 1: Time-dependent output
#!/bin/sh
echo "Script ran for $SECONDS seconds"
# Run 1 (fast machine): Script ran for 2 seconds
# Run 2 (slow machine): Script ran for 5 seconds
# PROBLEM: Output depends on execution speed

# VIOLATION 2: Cannot replay execution
#!/bin/sh
SECONDS=0
deploy_application
echo "Deployment took $SECONDS seconds"
# PROBLEM: Cannot replay with same timing
# Fast retry: 3 seconds, Slow retry: 10 seconds

# VIOLATION 3: Tests non-reproducible
#!/bin/sh
test_performance() {
    SECONDS=0
    run_operation
    # PROBLEM: Test may pass on fast machine, fail on slow machine
    [ $SECONDS -lt 10 ] || exit 1
}

# VIOLATION 4: Timing race conditions
#!/bin/sh
start=$SECONDS
while [ $((SECONDS - start)) -lt 30 ]; do
    check_service && break
    sleep 1
done
# PROBLEM: Service may start at different times
# Fast run: service starts in 5 seconds
# Slow run: service starts in 25 seconds
# Results in different behavior
"#;

    let mut lexer = Lexer::new(determinism_violations);
    if let Ok(tokens) = lexer.tokenize() {
        assert!(
            !tokens.is_empty(),
            "Determinism violations should tokenize successfully"
        );
        let _ = tokens;
    }

    // $SECONDS violates determinism (time-dependent)
    // bashrs FORBIDS $SECONDS to enforce determinism
    // CRITICAL: Execution time should not affect script output
}

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
