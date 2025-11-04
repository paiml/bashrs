# Property Testing

Property-based testing is a powerful technique that tests code against mathematical properties rather than specific examples. bashrs uses the `proptest` crate to generate hundreds of test cases automatically, catching edge cases that manual tests miss.

## What is Property-Based Testing?

Traditional unit tests use specific examples:

```rust
#[test]
fn test_addition() {
    assert_eq!(add(2, 3), 5);
    assert_eq!(add(0, 0), 0);
    assert_eq!(add(-1, 1), 0);
}
```

Property-based tests specify **properties** that should hold for **all** inputs:

```rust
proptest! {
    #[test]
    fn prop_addition_is_commutative(a: i32, b: i32) {
        assert_eq!(add(a, b), add(b, a));  // Property: a + b == b + a
    }
}
```

The framework generates 100-1000+ test cases automatically, including edge cases like:
- Maximum/minimum values
- Zero and negative numbers
- Random combinations
- Boundary conditions

### Why Property Testing Matters for Shell Scripts

Shell scripts have **complex input spaces**:
- Variable names: `[a-zA-Z_][a-zA-Z0-9_]*`
- Strings: arbitrary Unicode with quotes, escapes, newlines
- Commands: any valid command name + arguments
- Expansions: `$VAR`, `${VAR:-default}`, `$(cmd)`, etc.

Manual testing can't cover all combinations. Property testing generates thousands of valid inputs automatically.

## How bashrs Uses Property Tests

bashrs property tests validate three critical properties:

### Property 1: Determinism

**Property**: Purification is deterministic - same input always produces same output.

```rust
proptest! {
    #[test]
    fn prop_purification_is_deterministic(script in bash_script_strategy()) {
        let purified1 = purify(&script).unwrap();
        let purified2 = purify(&script).unwrap();

        // Property: Multiple runs produce identical output
        assert_eq!(purified1, purified2);
    }
}
```

**Why this matters**: Build systems and CI/CD pipelines depend on reproducible outputs. Non-determinism breaks caching and verification.

### Property 2: Idempotency

**Property**: Purification is idempotent - purifying already-purified code changes nothing.

```rust
proptest! {
    #[test]
    fn prop_purification_is_idempotent(script in bash_script_strategy()) {
        let purified1 = purify(&script).unwrap();
        let purified2 = purify(&purified1).unwrap();

        // Property: Purify(Purify(x)) == Purify(x)
        assert_eq!(purified1, purified2);
    }
}
```

**Why this matters**: Users should be able to run bashrs multiple times without changing the output. This is essential for version control and diffing.

### Property 3: Semantic Preservation

**Property**: Purification preserves behavior - purified scripts behave identically to originals.

```rust
proptest! {
    #[test]
    fn prop_purification_preserves_semantics(script in bash_script_strategy()) {
        let original_output = execute_bash(&script);
        let purified = purify(&script).unwrap();
        let purified_output = execute_sh(&purified);

        // Property: Same behavior (modulo determinism)
        assert_eq!(original_output, purified_output);
    }
}
```

**Why this matters**: Purification must not break existing scripts. Users need confidence that bashrs won't introduce bugs.

## Writing Property Tests for Shell Transformations

### Step 1: Define Input Strategies

Strategies generate random valid inputs. bashrs uses domain-specific strategies for shell constructs:

```rust
use proptest::prelude::*;

/// Generate valid bash identifiers: [a-zA-Z_][a-zA-Z0-9_]{0,15}
fn bash_identifier() -> impl Strategy<Value = String> {
    "[a-zA-Z_][a-zA-Z0-9_]{0,15}"
}

/// Generate safe strings (no shell metacharacters)
fn bash_string() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-zA-Z0-9_ ]{0,50}").unwrap()
}

/// Generate common variable names
fn bash_variable_name() -> impl Strategy<Value = String> {
    prop::sample::select(vec![
        "PATH".to_string(),
        "HOME".to_string(),
        "USER".to_string(),
        "foo".to_string(),
        "result".to_string(),
    ])
}

/// Generate integers in reasonable range
fn bash_integer() -> impl Strategy<Value = i64> {
    -1000i64..1000i64
}
```

### Step 2: Compose Strategies for Complex Structures

Build AST nodes from primitive strategies:

```rust
use bashrs::bash_parser::ast::*;

/// Generate variable assignments
fn bash_assignment() -> impl Strategy<Value = BashStmt> {
    (bash_identifier(), bash_string()).prop_map(|(name, value)| {
        BashStmt::Assignment {
            name,
            value: BashExpr::Literal(value),
            exported: false,
            span: Span::dummy(),
        }
    })
}

/// Generate commands
fn bash_command() -> impl Strategy<Value = BashStmt> {
    (
        bash_identifier(),
        prop::collection::vec(bash_string(), 0..4)
    ).prop_map(|(name, args)| {
        BashStmt::Command {
            name,
            args: args.into_iter().map(BashExpr::Literal).collect(),
            span: Span::dummy(),
        }
    })
}

/// Generate complete bash scripts
fn bash_script() -> impl Strategy<Value = BashAst> {
    prop::collection::vec(
        prop_oneof![
            bash_assignment(),
            bash_command(),
        ],
        1..10
    ).prop_map(|statements| {
        BashAst {
            statements,
            metadata: AstMetadata {
                source_file: None,
                line_count: statements.len(),
                parse_time_ms: 0,
            },
        }
    })
}
```

### Step 3: Write Property Tests

Test properties using generated inputs:

```rust
proptest! {
    #![proptest_config(ProptestConfig {
        cases: 1000,  // Generate 1000 test cases
        max_shrink_iters: 1000,
        .. ProptestConfig::default()
    })]

    /// Property: All valid assignments can be purified
    #[test]
    fn prop_assignments_can_be_purified(stmt in bash_assignment()) {
        let ast = BashAst {
            statements: vec![stmt],
            metadata: AstMetadata::default(),
        };

        // Should not panic
        let result = purify(ast);
        prop_assert!(result.is_ok());
    }

    /// Property: Commands with safe arguments are preserved
    #[test]
    fn prop_safe_commands_preserved(stmt in bash_command()) {
        let ast = BashAst {
            statements: vec![stmt.clone()],
            metadata: AstMetadata::default(),
        };

        let purified = purify(ast).unwrap();

        // Command name should be preserved
        match (&stmt, &purified.statements[0]) {
            (
                BashStmt::Command { name: orig_name, .. },
                BashStmt::Command { name: purified_name, .. }
            ) => {
                prop_assert_eq!(orig_name, purified_name);
            }
            _ => prop_assert!(false, "Expected commands"),
        }
    }
}
```

## Examples from bashrs

### Example 1: Variable Quoting Property

**Property**: All variable references in purified output should be quoted.

```rust
proptest! {
    #[test]
    fn prop_variables_are_quoted(
        var_name in bash_identifier(),
        value in bash_string()
    ) {
        let script = format!(r#"
#!/bin/bash
{}="{}"
echo ${}
"#, var_name, value, var_name);

        let purified = purify_bash(&script).unwrap();

        // Property: Variable usage should be quoted
        let expected = format!(r#"echo "${{{}}}"#, var_name);
        prop_assert!(purified.contains(&expected),
            "Expected quoted variable ${{{}}}, got:\n{}",
            var_name, purified);
    }
}
```

**Real-world bug caught**: This test discovered that variables in command substitutions weren't being quoted:

```bash
# Original (vulnerable)
RESULT=$(command $UNQUOTED)

# After fix (safe)
RESULT=$(command "$UNQUOTED")
```

### Example 2: Idempotency of mkdir -p

**Property**: Adding `-p` to `mkdir` is idempotent - doing it twice doesn't add it again.

```rust
proptest! {
    #[test]
    fn prop_mkdir_p_idempotent(dir in "[/a-z]{1,20}") {
        let script = format!("mkdir {}", dir);

        let purified1 = purify_bash(&script).unwrap();
        let purified2 = purify_bash(&purified1).unwrap();

        // Property: Second purification doesn't add another -p
        prop_assert_eq!(purified1, purified2);

        // Verify -p appears exactly once
        let p_count = purified1.matches("-p").count();
        prop_assert_eq!(p_count, 1, "Expected exactly one -p, got {}", p_count);
    }
}
```

### Example 3: POSIX Compatibility

**Property**: All purified scripts pass shellcheck in POSIX mode.

```rust
proptest! {
    #[test]
    fn prop_purified_is_posix_compliant(script in bash_script()) {
        let purified = purify(script).unwrap();
        let shell_output = generate_shell(&purified).unwrap();

        // Property: Passes shellcheck -s sh
        let result = std::process::Command::new("shellcheck")
            .arg("-s").arg("sh")
            .arg("-")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();

        let mut stdin = result.stdin.unwrap();
        stdin.write_all(shell_output.as_bytes()).unwrap();
        drop(stdin);

        let output = result.wait_with_output().unwrap();
        prop_assert!(output.status.success(),
            "Shellcheck failed:\n{}",
            String::from_utf8_lossy(&output.stderr));
    }
}
```

### Example 4: Parameter Expansion Preservation

**Property**: Valid parameter expansions are preserved (not broken).

```rust
proptest! {
    #[test]
    fn prop_parameter_expansion_preserved(
        var in bash_identifier(),
        default in bash_string()
    ) {
        let script = format!(r#"echo "${{{var}:-{default}}}"#,
            var = var, default = default);

        let purified = purify_bash(&script).unwrap();

        // Property: Parameter expansion syntax is preserved
        prop_assert!(
            purified.contains(&format!("${{{}:-", var)),
            "Expected parameter expansion preserved, got:\n{}",
            purified
        );
    }
}
```

**Real bug caught**: Initial implementation would incorrectly transform:
```bash
# Before: ${VAR:-default}
# After:  $VARdefault  # BROKEN!
```

Property test caught this immediately with 100+ generated test cases.

## Shrinking and Edge Case Discovery

When a property test fails, `proptest` **shrinks** the input to find the minimal failing case.

### Example: Shrinking in Action

```rust
proptest! {
    #[test]
    fn prop_commands_dont_panic(cmd in bash_command()) {
        // Bug: panics on empty command name
        process_command(&cmd);
    }
}
```

**Initial failure** (random):
```
thread 'prop_commands_dont_panic' panicked at 'assertion failed'
  cmd = BashStmt::Command {
      name: "",
      args: ["foo", "bar", "baz", "qux"],
      span: Span { ... }
  }
```

**After shrinking**:
```
Minimal failing case:
  cmd = BashStmt::Command {
      name: "",        // Empty name causes panic
      args: [],        // Irrelevant args removed
      span: Span::dummy()
  }
```

Shrinking makes debugging trivial - you immediately see the root cause.

### Configuring Shrinking

```rust
proptest! {
    #![proptest_config(ProptestConfig {
        cases: 1000,              // Try 1000 random inputs
        max_shrink_iters: 10000,  // Spend up to 10k iterations shrinking
        max_shrink_time: 60000,   // Or 60 seconds
        .. ProptestConfig::default()
    })]

    #[test]
    fn prop_complex_test(input in complex_strategy()) {
        // Test code
    }
}
```

## Integration with EXTREME TDD

Property tests are a key component of bashrs's EXTREME TDD methodology:

```
EXTREME TDD = TDD + Property Testing + Mutation Testing + PMAT + Examples
```

### RED → GREEN → REFACTOR → PROPERTY

1. **RED**: Write failing unit test
2. **GREEN**: Implement minimal fix
3. **REFACTOR**: Clean up implementation
4. **PROPERTY**: Add property test to prevent regressions

Example workflow:

```rust
// Step 1: RED - Failing unit test
#[test]
fn test_mkdir_adds_dash_p() {
    let input = "mkdir /tmp/foo";
    let output = purify_bash(input).unwrap();
    assert!(output.contains("mkdir -p"));
}

// Step 2: GREEN - Implement
fn make_mkdir_idempotent(stmt: BashStmt) -> BashStmt {
    match stmt {
        BashStmt::Command { name, mut args, span } if name == "mkdir" => {
            args.insert(0, BashExpr::Literal("-p".to_string()));
            BashStmt::Command { name, args, span }
        }
        _ => stmt,
    }
}

// Step 3: REFACTOR - Clean up
fn make_mkdir_idempotent(stmt: BashStmt) -> BashStmt {
    match stmt {
        BashStmt::Command { name, mut args, span } if name == "mkdir" => {
            if !has_flag(&args, "-p") {
                args.insert(0, BashExpr::Literal("-p".to_string()));
            }
            BashStmt::Command { name, args, span }
        }
        _ => stmt,
    }
}

// Step 4: PROPERTY - Prevent regressions
proptest! {
    #[test]
    fn prop_mkdir_always_gets_dash_p(dir in "[/a-z]{1,20}") {
        let script = format!("mkdir {}", dir);
        let purified = purify_bash(&script).unwrap();

        // Property: All mkdir commands get -p
        prop_assert!(purified.contains("mkdir -p"),
            "Expected 'mkdir -p', got: {}", purified);
    }

    #[test]
    fn prop_mkdir_dash_p_idempotent(dir in "[/a-z]{1,20}") {
        let script = format!("mkdir {}", dir);
        let purified1 = purify_bash(&script).unwrap();
        let purified2 = purify_bash(&purified1).unwrap();

        // Property: Idempotent
        prop_assert_eq!(purified1, purified2);
    }
}
```

### Property Tests Complement Mutation Testing

Property tests catch bugs mutation tests miss:

**Mutation test**: Changes `if !has_flag` to `if has_flag`
- Unit tests: May pass if they don't cover all flag combinations
- Property tests: **Fail immediately** across 1000+ generated cases

**Property test**: Catches missing edge case
- Mutation tests: Only test what you wrote
- Property tests: Test what you **didn't think of**

## Best Practices

### 1. Start with Simple Properties

Don't try to test everything at once:

```rust
// ✅ GOOD: Simple, focused property
proptest! {
    #[test]
    fn prop_parse_never_panics(input in ".*{0,1000}") {
        // Should handle any input without crashing
        let _ = parse_bash(&input);
    }
}

// ❌ TOO COMPLEX: Testing too much
proptest! {
    #[test]
    fn prop_everything_works(input in ".*{0,1000}") {
        let ast = parse_bash(&input).unwrap();  // Assumes parse succeeds
        let purified = purify(ast).unwrap();    // Assumes purify succeeds
        let output = generate(purified).unwrap();
        assert!(shellcheck_passes(&output));    // Too many assumptions
    }
}
```

### 2. Use Domain-Specific Strategies

Generate **valid** inputs, not random garbage:

```rust
// ❌ BAD: Random strings aren't valid bash
proptest! {
    #[test]
    fn prop_parse_succeeds(input in ".*") {
        parse_bash(&input).unwrap();  // Will fail on invalid syntax
    }
}

// ✅ GOOD: Generate valid bash constructs
fn valid_bash_script() -> impl Strategy<Value = String> {
    prop::collection::vec(
        prop_oneof![
            bash_assignment_string(),
            bash_command_string(),
            bash_if_statement_string(),
        ],
        1..20
    ).prop_map(|lines| lines.join("\n"))
}

proptest! {
    #[test]
    fn prop_valid_bash_parses(script in valid_bash_script()) {
        parse_bash(&script).unwrap();  // Should always succeed
    }
}
```

### 3. Test Properties, Not Implementation

Focus on **what** should be true, not **how** it's implemented:

```rust
// ❌ BAD: Tests implementation details
proptest! {
    #[test]
    fn prop_uses_regex_to_find_variables(input in ".*") {
        let result = purify(&input);
        assert!(result.internal_regex.is_some());  // Implementation detail
    }
}

// ✅ GOOD: Tests observable behavior
proptest! {
    #[test]
    fn prop_all_variables_are_quoted(script in bash_script()) {
        let purified = purify(&script).unwrap();

        // Observable: No unquoted variables in output
        let unquoted_vars = find_unquoted_variables(&purified);
        prop_assert!(unquoted_vars.is_empty(),
            "Found unquoted variables: {:?}", unquoted_vars);
    }
}
```

### 4. Use Preconditions with `prop_assume`

Filter out invalid cases instead of failing:

```rust
proptest! {
    #[test]
    fn prop_division_works(a: i32, b: i32) {
        prop_assume!(b != 0);  // Skip division by zero

        let result = divide(a, b);
        prop_assert_eq!(result * b, a);
    }
}
```

For bashrs:

```rust
proptest! {
    #[test]
    fn prop_safe_eval_works(cmd in bash_command_string()) {
        // Only test safe commands (no eval)
        prop_assume!(!cmd.contains("eval"));

        let result = execute_safely(&cmd);
        prop_assert!(result.is_ok());
    }
}
```

### 5. Balance Test Cases vs Runtime

More cases = better coverage, but slower tests:

```rust
proptest! {
    #![proptest_config(ProptestConfig {
        cases: 100,  // Quick smoke test (CI)
        .. ProptestConfig::default()
    })]

    #[test]
    fn prop_fast_smoke_test(input in bash_script()) {
        // Runs 100 times, finishes in seconds
    }
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 10000,  // Thorough test (nightly)
        .. ProptestConfig::default()
    })]

    #[test]
    #[ignore]  // Only run with --ignored
    fn prop_exhaustive_test(input in bash_script()) {
        // Runs 10k times, may take minutes
    }
}
```

### 6. Document Expected Failures

Some properties have known limitations:

```rust
proptest! {
    #[test]
    fn prop_parse_all_bash(input in ".*") {
        match parse_bash(&input) {
            Ok(_) => {},
            Err(e) => {
                // Document known limitations
                if input.contains("$($(nested))") {
                    // Known: Nested command substitution not supported
                    return Ok(());
                }
                prop_assert!(false, "Unexpected parse error: {}", e);
            }
        }
    }
}
```

## Advanced Techniques

### Regression Testing with `proptest-regressions`

Save failing cases for permanent regression tests:

```toml
# proptest-regressions/prop_test_name.txt
cc 0123456789abcdef  # Hex seed for failing case
```

```rust
proptest! {
    #[test]
    fn prop_no_regressions(input in bash_script()) {
        // Failed cases automatically become permanent tests
        purify(input).unwrap();
    }
}
```

### Stateful Property Testing

Test sequences of operations:

```rust
#[derive(Debug, Clone)]
enum Operation {
    AddVariable(String, String),
    UseVariable(String),
    DefineFunction(String),
    CallFunction(String),
}

fn operation_strategy() -> impl Strategy<Value = Operation> {
    prop_oneof![
        (bash_identifier(), bash_string())
            .prop_map(|(k, v)| Operation::AddVariable(k, v)),
        bash_identifier()
            .prop_map(Operation::UseVariable),
        // ... other operations
    ]
}

proptest! {
    #[test]
    fn prop_stateful_execution(ops in prop::collection::vec(operation_strategy(), 1..20)) {
        let mut state = BashState::new();

        for op in ops {
            match op {
                Operation::AddVariable(k, v) => state.set_var(&k, &v),
                Operation::UseVariable(k) => {
                    // Should never panic
                    let _ = state.get_var(&k);
                }
                // ... handle other operations
            }
        }

        // Property: State should always be consistent
        prop_assert!(state.is_consistent());
    }
}
```

## Summary

Property-based testing is essential for bashrs quality:

**Benefits**:
- Catches edge cases manual tests miss
- Tests thousands of cases automatically
- Shrinks failures to minimal examples
- Validates mathematical properties (determinism, idempotency)
- Integrates with EXTREME TDD workflow

**When to use**:
- Functions with large input spaces (parsers, transformations)
- Properties that should hold universally (idempotency, commutativity)
- Complex algorithms with many edge cases
- Complementing mutation testing

**bashrs uses property tests for**:
1. Parser robustness (never panics)
2. Transformation determinism (same input → same output)
3. Purification idempotency (purify twice = purify once)
4. POSIX compliance (shellcheck always passes)
5. Semantic preservation (behavior unchanged)

For more on testing quality, see [Mutation Testing](./mutation-testing.md) and [Performance Optimization](./performance.md).
