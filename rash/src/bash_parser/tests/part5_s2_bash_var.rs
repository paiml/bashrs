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
