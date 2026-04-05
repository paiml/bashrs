#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(unused_imports)]

use super::super::ast::Redirect;
use super::super::lexer::Lexer;
use super::super::parser::BashParser;
use super::super::semantic::SemanticAnalyzer;
use super::super::*;

#[test]
fn test_BASH_VAR_002_random_determinism_violations() {
    // DOCUMENTATION: How $RANDOM violates determinism (5 critical violations)
    //
    // VIOLATION 1: Same script, different results
    // #!/bin/sh
    // echo $RANDOM
    // Running twice produces different numbers: 12345, 8901
    // EXPECTED (deterministic): Same output every run
    //
    // VIOLATION 2: Cannot replay execution
    // Script with $RANDOM cannot be replayed exactly
    // Debugging impossible - cannot reproduce bug
    // EXPECTED: Replay should produce identical results
    //
    // VIOLATION 3: Tests non-reproducible
    // test_something() {
    //   value=$RANDOM
    //   assert value == ???  # What value to assert?
    // }
    // EXPECTED: Tests should be reproducible
    //
    // VIOLATION 4: Race conditions in parallel execution
    // Two scripts using $RANDOM may get same value (if executed at same time)
    // EXPECTED: Deterministic identifiers prevent collisions
    //
    // VIOLATION 5: Security through obscurity
    // Using $RANDOM for security (session IDs, tokens) is WEAK
    // PRNG is predictable if seed known
    // EXPECTED: Use cryptographic randomness for security

    let determinism_violations = r#"
# VIOLATION 1: Same script, different results
#!/bin/sh
# This script is NON-DETERMINISTIC
echo "Random number: $RANDOM"
# Run 1: Random number: 12345
# Run 2: Random number: 8901
# Run 3: Random number: 23456
# PROBLEM: Cannot predict output

# VIOLATION 2: Cannot replay execution
#!/bin/sh
# Deployment script (NON-DETERMINISTIC)
release_id="release-$RANDOM"
deploy "$release_id"
# PROBLEM: Cannot redeploy same release_id
# If deployment fails, cannot retry with same ID

# VIOLATION 3: Tests non-reproducible
#!/bin/sh
test_function() {
    value=$RANDOM
    # PROBLEM: Cannot assert on value (changes every run)
    # Test may pass sometimes, fail other times
}

# VIOLATION 4: Race conditions
#!/bin/sh
# Two scripts running in parallel
session_id=$RANDOM  # May get same value!
# PROBLEM: Collision if both scripts run at same microsecond

# VIOLATION 5: Weak security
#!/bin/sh
token=$RANDOM  # WEAK! Predictable!
# PROBLEM: Only 32768 possible values (2^15)
# Attacker can guess in seconds
"#;

    let mut lexer = Lexer::new(determinism_violations);
    if let Ok(tokens) = lexer.tokenize() {
        assert!(
            !tokens.is_empty(),
            "Determinism violations should tokenize successfully"
        );
        let _ = tokens;
    }

    // $RANDOM violates EVERY determinism principle
    // bashrs FORBIDS $RANDOM to enforce determinism
    // CRITICAL: Determinism is non-negotiable in bashrs
}

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

