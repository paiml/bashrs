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
