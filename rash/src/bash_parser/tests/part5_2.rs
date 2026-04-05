#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(unused_imports)]

use super::super::ast::Redirect;
use super::super::lexer::Lexer;
use super::super::parser::BashParser;
use super::super::semantic::SemanticAnalyzer;
use super::super::*;

#[test]
fn test_BASH_VAR_002_random_portability_issues() {
    // DOCUMENTATION: $RANDOM portability issues (4 critical issues)
    //
    // ISSUE 1: Not POSIX (bash-specific)
    // $RANDOM only exists in bash, ksh, zsh
    // POSIX sh: $RANDOM is UNDEFINED (may be literal string "$RANDOM")
    // dash: $RANDOM is UNDEFINED
    // ash: $RANDOM is UNDEFINED
    //
    // ISSUE 2: Different ranges in different shells
    // bash: $RANDOM is 0-32767 (2^15 - 1)
    // ksh: $RANDOM is 0-32767 (same)
    // zsh: $RANDOM is 0-32767 (same)
    // BUT: Implementation details differ (seed behavior, PRNG algorithm)
    //
    // ISSUE 3: Seed behavior differs
    // bash: RANDOM seed can be set with RANDOM=seed
    // ksh: Different seeding mechanism
    // zsh: Different seeding mechanism
    // POSIX sh: N/A (no $RANDOM)
    //
    // ISSUE 4: Subprocess behavior undefined
    // Some shells re-seed $RANDOM in subshells
    // Others inherit parent's PRNG state
    // Behavior is INCONSISTENT across shells
    //
    // PURIFICATION STRATEGY:
    // Replace ALL $RANDOM with POSIX-compliant alternatives
    // Use awk for PRNG (POSIX), or deterministic values

    let portability_issues = r#"
#!/bin/sh
# This script is NOT PORTABLE (uses $RANDOM)

# ISSUE 1: Not POSIX
echo $RANDOM  # bash: works, dash: UNDEFINED

# ISSUE 2: Range assumption
if [ $RANDOM -lt 16384 ]; then  # Assumes 0-32767 range
    echo "First half"
fi

# ISSUE 3: Seeding
RANDOM=42  # bash: sets seed, dash: just sets variable
echo $RANDOM  # bash: deterministic from seed, dash: literal "$RANDOM"

# ISSUE 4: Subshell behavior
echo $RANDOM  # Parent shell
(echo $RANDOM)  # Subshell (may be re-seeded or inherit)

# PURIFIED (POSIX-compliant):
# Use awk for portable PRNG
awk 'BEGIN { srand(42); print int(rand() * 32768) }'
"#;

    let mut lexer = Lexer::new(portability_issues);
    if let Ok(tokens) = lexer.tokenize() {
        assert!(
            !tokens.is_empty(),
            "Portability issues should tokenize successfully"
        );
        let _ = tokens;
    }

    // $RANDOM is NOT PORTABLE (bash-specific)
    // bashrs targets POSIX sh (no $RANDOM support)
    // PURIFICATION: Use awk PRNG or deterministic values
}

#[test]
fn test_BASH_VAR_002_random_security_implications() {
    // DOCUMENTATION: $RANDOM security implications (5 critical risks)
    //
    // RISK 1: Weak PRNG (Linear Congruential Generator)
    // $RANDOM uses simple LCG: next = (a * prev + c) % m
    // Predictable if seed known or can be guessed
    // NOT cryptographically secure
    //
    // RISK 2: Small range (0-32767)
    // Only 2^15 possible values (32,768)
    // Attacker can brute-force in milliseconds
    // For comparison: Cryptographic tokens need 2^128+ bits
    //
    // RISK 3: Predictable seed
    // Default seed often based on PID or timestamp
    // Attacker can guess seed from process list or system time
    // Once seed known, entire sequence predictable
    //
    // RISK 4: Collision probability high
    // Birthday paradox: 50% collision probability after ~215 samples
    // Session IDs using $RANDOM will collide frequently
    //
    // RISK 5: Observable output leaks state
    // If attacker observes few $RANDOM values, can reconstruct PRNG state
    // Future values become predictable
    //
    // NEVER USE $RANDOM FOR:
    // - Passwords, tokens, API keys
    // - Session IDs (unless collision acceptable)
    // - Cryptographic nonces
    // - Security-critical randomness
    //
    // SECURE ALTERNATIVES:
    // - /dev/urandom (cryptographically secure)
    // - openssl rand (cryptographic PRNG)
    // - /dev/random (blocks until enough entropy)

    let security_implications = r#"
#!/bin/sh
# SECURITY EXAMPLES

# INSECURE: Password generation
# BAD: password=$RANDOM
# Only 32,768 possible passwords!
# Attacker brute-forces in seconds

# SECURE: Use cryptographic randomness
password=$(openssl rand -base64 32)

# INSECURE: Session token
# BAD: token=$RANDOM
# Predictable, collidable

# SECURE: Use /dev/urandom
token=$(od -An -N16 -tx1 /dev/urandom | tr -d ' ')

# INSECURE: API key
# BAD: api_key=$RANDOM
# Only 15 bits of entropy (WEAK!)

# SECURE: Use openssl
api_key=$(openssl rand -hex 32)  # 256 bits of entropy

# INSECURE: Cryptographic nonce
# BAD: nonce=$RANDOM
# Predictable, violates nonce security requirements

# SECURE: Use /dev/urandom
nonce=$(od -An -N16 -tx1 /dev/urandom | tr -d ' ')

# INSECURE: Salt for password hashing
# BAD: salt=$RANDOM
# Weak salt enables rainbow table attacks

# SECURE: Use cryptographic randomness
salt=$(openssl rand -base64 16)
"#;

    let mut lexer = Lexer::new(security_implications);
    if let Ok(tokens) = lexer.tokenize() {
        assert!(
            !tokens.is_empty(),
            "Security implications should tokenize successfully"
        );
        let _ = tokens;
    }

    // $RANDOM is CRYPTOGRAPHICALLY WEAK
    // NEVER use for security purposes
    // ALWAYS use /dev/urandom or openssl rand for security
}

#[test]
fn test_BASH_VAR_002_random_testing_implications() {
    // DOCUMENTATION: $RANDOM testing implications (4 critical issues for testing)
    //
    // ISSUE 1: Non-reproducible tests
    // test_deployment() {
    //   release_id="release-$RANDOM"
    //   deploy "$release_id"
    //   assert deployed "$release_id"  # Which release_id?
    // }
    // PROBLEM: Test fails intermittently (different release_id each run)
    //
    // ISSUE 2: Cannot assert on output
    // output=$(./script.sh)  # Script uses $RANDOM
    // assert "$output" == "???"  # What value to assert?
    // PROBLEM: Cannot write assertions for non-deterministic output
    //
    // ISSUE 3: Flaky tests (heisenbug)
    // Test passes 99% of time, fails 1%
    // Due to $RANDOM producing edge case value
    // PROBLEM: Developers lose trust in test suite
    //
    // ISSUE 4: Cannot replay failures
    // Test fails in CI, cannot reproduce locally
    // Bug only occurs with specific $RANDOM value
    // PROBLEM: Cannot debug or fix bug
    //
    // TESTING BEST PRACTICES:
    // 1. Never use $RANDOM in production code
    // 2. If testing code that uses $RANDOM, mock it with fixed seed
    // 3. Use deterministic test data (fixed values, sequences)
    // 4. For testing randomness behavior, use property-based testing with seeds

    let testing_implications = r#"
#!/bin/sh
# TESTING EXAMPLES

# BAD TEST: Non-reproducible
test_bad() {
    value=$RANDOM
    process "$value"
    # PROBLEM: Cannot assert on result (value changes each run)
}

# GOOD TEST: Deterministic
test_good() {
    value=42  # Fixed test value
    result=$(process "$value")
    [ "$result" = "processed-42" ] || exit 1
}

# BAD TEST: Flaky (heisenbug)
test_flaky() {
    value=$RANDOM
    # Test passes for value < 16384, fails otherwise
    [ "$value" -lt 16384 ] || exit 1
}

# GOOD TEST: Deterministic edge cases
test_edge_cases() {
    # Test explicit edge cases
    process 0      || exit 1
    process 16383  || exit 1
    process 32767  || exit 1
}

# BAD TEST: Cannot replay failure
test_cannot_replay() {
    session_id="session-$RANDOM"
    deploy "$session_id"
    # Fails in CI with specific $RANDOM value
    # Cannot reproduce locally
}

# GOOD TEST: Deterministic, replayable
test_replayable() {
    session_id="session-test-1"
    deploy "$session_id"
    # Always same session_id, always reproducible
}

# GOOD TEST: Property-based with seed
test_property_based() {
    seed=42
    for i in $(seq 1 100); do
        value=$(awk -v seed="$seed" -v i="$i" 'BEGIN { srand(seed + i); print int(rand() * 32768) }')
        process "$value" || exit 1
    done
    # Deterministic (same seed), tests 100 values
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

    // $RANDOM makes tests NON-REPRODUCIBLE
    // bashrs enforces DETERMINISTIC testing
    // NEVER use $RANDOM in test code
}

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

include!("part5_2_bash_var.rs");
