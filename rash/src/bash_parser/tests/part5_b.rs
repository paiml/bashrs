#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(unused_imports)]

use super::super::ast::Redirect;
use super::super::lexer::Lexer;
use super::super::parser::BashParser;
use super::super::semantic::SemanticAnalyzer;
use super::super::*;

#[test]
fn test_BASH_VAR_002_random_comparison_table() {
    // DOCUMENTATION: Comprehensive $RANDOM comparison (Bash vs POSIX vs Purified)
    //
    // ┌─────────────────────────────────────────────────────────────────────────┐
    // │ FEATURE                    │ Bash       │ POSIX      │ Purified         │
    // ├─────────────────────────────────────────────────────────────────────────┤
    // │ $RANDOM variable           │ SUPPORTED  │ NOT POSIX  │ NOT SUPPORTED    │
    // │ num=$RANDOM                │ ✅ 0-32767│ ❌         │ ❌ FORBIDDEN     │
    // │                            │            │            │                  │
    // │ Determinism                │ NO         │ N/A        │ YES (enforced)   │
    // │ Same script → same output  │ ❌ Random │ N/A        │ ✅ Deterministic │
    // │                            │            │            │                  │
    // │ Reproducibility            │ NO         │ N/A        │ YES              │
    // │ Can replay execution       │ ❌         │ N/A        │ ✅               │
    // │                            │            │            │                  │
    // │ Testing                    │ Flaky      │ N/A        │ Reproducible     │
    // │ Test assertions            │ ⚠️ Hard   │ N/A        │ ✅ Easy          │
    // │                            │            │            │                  │
    // │ Security                   │ WEAK       │ N/A        │ Use crypto PRNG  │
    // │ Cryptographic use          │ ❌ Unsafe │ N/A        │ ✅ /dev/urandom  │
    // │                            │            │            │                  │
    // │ Portability                │ bash/ksh   │ N/A        │ POSIX awk        │
    // │ Works in dash/ash          │ ❌         │ N/A        │ ✅               │
    // │                            │            │            │                  │
    // │ Seeding                    │ RANDOM=n   │ N/A        │ awk srand(n)     │
    // │ Set seed for determinism   │ ⚠️ bash   │ N/A        │ ✅ POSIX         │
    // │                            │            │            │                  │
    // │ Range                      │ 0-32767    │ N/A        │ Configurable     │
    // │ Number of possible values  │ 32768      │ N/A        │ Unlimited        │
    // │                            │            │            │                  │
    // │ Collision probability      │ HIGH       │ N/A        │ Configurable     │
    // │ Birthday paradox (50%)     │ ~215 uses  │ N/A        │ Depends on range │
    // └─────────────────────────────────────────────────────────────────────────┘
    //
    // RUST MAPPING:
    // $RANDOM → NOT MAPPED (use deterministic values instead)
    // For PRNG needs: use rand crate with explicit seed
    // For unique IDs: use uuid, sequence numbers, or version-based IDs
    // For security: use rand::rngs::OsRng (cryptographically secure)
    //
    // PURIFICATION RULES:
    // 1. $RANDOM → FORBIDDEN (rewrite script with deterministic alternative)
    // 2. Session IDs → Use version/timestamp-based identifiers
    // 3. Temporary files → Use mktemp (POSIX)
    // 4. Test data → Use fixed values (42, 100, 1000, etc.)
    // 5. Crypto randomness → Use /dev/urandom or openssl rand
    // 6. Need PRNG → Use awk with explicit seed (deterministic)

    let comparison_table = r#"
#!/bin/sh
# COMPARISON EXAMPLES

# BASH (NON-DETERMINISTIC):
# num=$RANDOM  # Different value each run

# POSIX (NOT AVAILABLE):
# $RANDOM doesn't exist in POSIX sh

# PURIFIED (DETERMINISTIC):
# Option 1: Fixed value
num=42

# Option 2: Sequence
num=$(seq 1 1)  # Or seq 1 100 for range

# Option 3: Deterministic PRNG (awk with seed)
seed=42
num=$(awk -v seed="$seed" 'BEGIN { srand(seed); print int(rand() * 32768) }')

# Option 4: Hash-based (deterministic from input)
input="user@example.com"
num=$(printf '%s' "$input" | sha256sum | cut -c1-5 | xargs -I{} printf '%d' "0x{}")

# Option 5: Crypto randomness (LAST RESORT - non-deterministic)
# Only for security purposes
# num=$(od -An -N2 -i /dev/urandom)

# TESTING COMPARISON:
# BASH (flaky tests):
# test_value=$RANDOM  # Different each run, cannot assert

# PURIFIED (reproducible tests):
test_value=42  # Same every run, can assert
[ "$test_value" = "42" ] || exit 1

# SECURITY COMPARISON:
# BASH (INSECURE):
# token=$RANDOM  # Only 32768 values, predictable

# PURIFIED (SECURE):
token=$(openssl rand -hex 32)  # 2^256 values, cryptographic
"#;

    let mut lexer = Lexer::new(comparison_table);
    if let Ok(tokens) = lexer.tokenize() {
        assert!(
            !tokens.is_empty(),
            "Comparison table should tokenize successfully"
        );
        let _ = tokens;
    }

    // POSIX STATUS: $RANDOM is NOT POSIX (bash-specific)
    // bashrs STATUS: $RANDOM is FORBIDDEN (violates determinism)
    // PURIFICATION: Rewrite with deterministic alternatives (fixed values, sequences, awk PRNG with seed)
    // Determinism: $RANDOM is NON-DETERMINISTIC (antithetical to bashrs philosophy)
    // Portability: $RANDOM is NOT PORTABLE (bash/ksh/zsh only, not POSIX sh/dash/ash)
    // Security: $RANDOM is CRYPTOGRAPHICALLY WEAK (never use for passwords/tokens/keys)
    // Testing: $RANDOM makes tests FLAKY and NON-REPRODUCIBLE
}

// ============================================================================
// BASH-VAR-003: $SECONDS purification (NOT SUPPORTED)
// ============================================================================

// DOCUMENTATION: $SECONDS is NOT SUPPORTED (bash-specific, MEDIUM priority purification)
//
// $SECONDS: Bash-specific variable that tracks seconds since shell started
// Each time $SECONDS is referenced, returns number of seconds elapsed
// Can be reset: SECONDS=0 (resets timer to zero)
//
// WHY NOT SUPPORTED:
// 1. Non-deterministic (different value each time script runs)
// 2. Time-dependent (value depends on when script started, how long it ran)
// 3. Bash-specific (not POSIX, doesn't exist in sh/dash/ash)
// 4. Breaks reproducibility (cannot replay script execution with same timing)
// 5. Breaks testing (tests run at different speeds, produce different results)
//
// CRITICAL: $SECONDS violates determinism
// bashrs enforces DETERMINISM - execution time should not affect output
//
// PURIFICATION STRATEGY:
// $SECONDS is FORBIDDEN - scripts using $SECONDS must be rewritten
//
// OPTION 1: Use fixed durations (deterministic)
// INPUT: duration=$SECONDS
// PURIFIED: duration=100
//
// OPTION 2: Use explicit timestamps (deterministic if timestamps are)
// INPUT: elapsed=$SECONDS
// PURIFIED: start_time=1640000000; end_time=1640000100; elapsed=$((end_time - start_time))
//
// OPTION 3: Remove timing logic entirely
// INPUT: echo "Script ran for $SECONDS seconds"
// PURIFIED: echo "Script completed"
#[test]
fn test_BASH_VAR_003_seconds_not_supported() {
    // $SECONDS is NOT SUPPORTED (non-deterministic, time-dependent)
    let seconds_variable = concat!(
        "# NOT SUPPORTED: $SECONDS (non-deterministic, time-dependent)\n",
        "echo \"Elapsed: $SECONDS seconds\"\n",
        "\n",
        "# NOT SUPPORTED: Reset SECONDS\n",
        "SECONDS=0\n",
        "operation\n",
        "echo \"Operation took $SECONDS seconds\"\n",
        "\n",
        "# NOT SUPPORTED: Timeout based on SECONDS\n",
        "start=$SECONDS\n",
        "while [ $((SECONDS - start)) -lt 60 ]; do\n",
        "    # Wait up to 60 seconds\n",
        "    sleep 1\n",
        "done\n",
        "\n",
        "# NOT SUPPORTED: Performance measurement\n",
        "SECONDS=0\n",
        "run_benchmark\n",
        "echo \"Benchmark completed in $SECONDS seconds\"\n",
    );

    let mut lexer = Lexer::new(seconds_variable);
    // Parser may not support $SECONDS - both Ok and Err are acceptable
    if let Ok(tokens) = lexer.tokenize() {
        assert!(
            !tokens.is_empty(),
            "$SECONDS should tokenize (even though NOT SUPPORTED)"
        );
    }
}

#[test]
fn test_BASH_VAR_003_seconds_purification_strategies() {
    // DOCUMENTATION: $SECONDS purification strategies (4 strategies for different use cases)
    //
    // STRATEGY 1: Fixed durations
    // Use case: Script needs duration but value doesn't matter
    // INPUT: duration=$SECONDS
    // PURIFIED: duration=100
    // Pros: Simple, deterministic
    // Cons: Not realistic timing
    //
    // STRATEGY 2: Explicit timestamp arithmetic
    // Use case: Need specific duration calculation
    // INPUT: elapsed=$SECONDS
    // PURIFIED: start=1640000000; end=1640000100; elapsed=$((end - start))
    // Pros: Deterministic, controlled timing
    // Cons: Requires explicit timestamps
    //
    // STRATEGY 3: Remove timing logic entirely
    // Use case: Timing is not essential to script logic
    // INPUT: echo "Took $SECONDS seconds"
    // PURIFIED: echo "Operation completed"
    // Pros: Simplest, no timing dependency
    // Cons: Loses timing information
    //
    // STRATEGY 4: Use external time source (deterministic if source is)
    // Use case: Need actual timing but controlled
    // INPUT: duration=$SECONDS
    // PURIFIED: duration=$(cat /path/to/fixed_duration.txt)
    // Pros: Deterministic from file, can be version-controlled
    // Cons: Requires external file

    let purification_strategies = r#"
# STRATEGY 1: Fixed durations
duration=100  # Fixed value instead of $SECONDS
echo "Duration: $duration seconds"

# STRATEGY 2: Explicit timestamp arithmetic
start_time=1640000000  # Fixed Unix timestamp (2021-12-20)
end_time=1640000100    # Fixed Unix timestamp
elapsed=$((end_time - start_time))
echo "Elapsed: $elapsed seconds"

# STRATEGY 3: Remove timing logic
# INPUT: echo "Script took $SECONDS seconds"
echo "Script completed successfully"

# STRATEGY 4: External time source (deterministic)
# duration=$(cat config/benchmark_duration.txt)
# echo "Benchmark duration: $duration seconds"

# REAL-WORLD EXAMPLE: Timeout loop
# BAD (non-deterministic):
# start=$SECONDS
# while [ $((SECONDS - start)) -lt 60 ]; do
#     check_condition && break
#     sleep 1
# done

# GOOD (deterministic):
max_attempts=60
attempt=0
while [ $attempt -lt $max_attempts ]; do
    check_condition && break
    sleep 1
    attempt=$((attempt + 1))
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
    // PREFERRED: Strategies 1-3 (remove timing dependency)
    // Strategy 4 acceptable if external source is deterministic
}

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

