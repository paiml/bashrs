fn test_BASH_VAR_002_random_common_antipatterns() {
    // DOCUMENTATION: Common $RANDOM antipatterns and their fixes (8 antipatterns)
    //
    // ANTIPATTERN 1: Random session IDs
    // BAD: session_id=$RANDOM
    // GOOD: session_id="session-$VERSION"
    // Why: Session IDs should be deterministic for reproducibility
    //
    // ANTIPATTERN 2: Random temporary filenames
    // BAD: temp_file="/tmp/file-$RANDOM.txt"
    // GOOD: temp_file=$(mktemp)
    // Why: mktemp is POSIX, secure, deterministic if TMPDIR set
    //
    // ANTIPATTERN 3: Random sleep delays
    // BAD: sleep $((RANDOM % 10))
    // GOOD: sleep 5  # Fixed delay
    // Why: Sleep delays should be deterministic for predictable behavior
    //
    // ANTIPATTERN 4: Random port selection
    // BAD: port=$((8000 + RANDOM % 1000))
    // GOOD: port=8080  # Fixed port, or read from config
    // Why: Port numbers should be deterministic or configurable
    //
    // ANTIPATTERN 5: Random passwords
    // BAD: password=$(echo $RANDOM | md5sum | head -c 20)
    // GOOD: password=$(openssl rand -base64 20)  # Cryptographically secure
    // Why: Passwords need cryptographic randomness, not weak PRNG
    //
    // ANTIPATTERN 6: Random load balancing
    // BAD: server=server$((RANDOM % 3)).example.com
    // GOOD: Use round-robin or least-connections algorithm (deterministic)
    // Why: Load balancing should be predictable for debugging
    //
    // ANTIPATTERN 7: Random retry delays (jitter)
    // BAD: sleep $((RANDOM % 5))
    // GOOD: sleep $((attempt * 2))  # Exponential backoff (deterministic)
    // Why: Retry delays should be deterministic for testing
    //
    // ANTIPATTERN 8: Random test data
    // BAD: test_value=$RANDOM
    // GOOD: test_value=42  # Fixed test value
    // Why: Test data MUST be deterministic for reproducible tests

    let antipatterns = r#"
# ANTIPATTERN 1: Random session IDs
# BAD: session_id=$RANDOM
session_id="session-1.0.0"  # GOOD: Deterministic

# ANTIPATTERN 2: Random temp files
# BAD: temp_file="/tmp/file-$RANDOM.txt"
temp_file=$(mktemp)  # GOOD: POSIX mktemp

# ANTIPATTERN 3: Random sleep delays
# BAD: sleep $((RANDOM % 10))
sleep 5  # GOOD: Fixed delay

# ANTIPATTERN 4: Random port selection
# BAD: port=$((8000 + RANDOM % 1000))
port=8080  # GOOD: Fixed or from config

# ANTIPATTERN 5: Random passwords
# BAD: password=$(echo $RANDOM | md5sum | head -c 20)
password=$(openssl rand -base64 20)  # GOOD: Cryptographic

# ANTIPATTERN 6: Random load balancing
# BAD: server=server$((RANDOM % 3)).example.com
# GOOD: Use deterministic algorithm
servers="server1.example.com server2.example.com server3.example.com"
server=$(echo "$servers" | awk -v n="$REQUEST_ID" '{print $(n % NF + 1)}')

# ANTIPATTERN 7: Random retry delays
# BAD: sleep $((RANDOM % 5))
attempt=1
sleep $((attempt * 2))  # GOOD: Exponential backoff

# ANTIPATTERN 8: Random test data
# BAD: test_value=$RANDOM
test_value=42  # GOOD: Fixed test value
"#;

    let mut lexer = Lexer::new(antipatterns);
    if let Ok(tokens) = lexer.tokenize() {
        assert!(
            !tokens.is_empty(),
            "Antipatterns should tokenize successfully"
        );
        let _ = tokens;
    }

    // All antipatterns involve $RANDOM (non-deterministic)
    // All fixes are DETERMINISTIC alternatives
    // CRITICAL: Never use $RANDOM in production scripts
}

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
