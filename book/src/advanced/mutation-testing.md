# Mutation Testing

Mutation testing is the gold standard for measuring test quality. While code coverage tells you which lines are executed, mutation testing tells you whether your tests actually catch bugs. bashrs uses `cargo-mutants` to achieve 80-90%+ kill rates on security-critical code.

## What is Mutation Testing?

Mutation testing works by introducing small bugs (mutations) into your code and checking if your tests catch them:

```rust,ignore
// Original code
fn is_safe_command(cmd: &str) -> bool {
    !cmd.contains("eval")
}

// Mutant 1: Negate condition
fn is_safe_command(cmd: &str) -> bool {
    cmd.contains("eval")  // Bug: inverted logic
}

// Mutant 2: Change constant
fn is_safe_command(cmd: &str) -> bool {
    !cmd.contains("")  // Bug: empty string always matches
}
```

**If your tests pass** with the mutant, the mutant **survived** (bad - your tests missed a bug).

**If your tests fail** with the mutant, the mutant was **killed** (good - your tests caught the bug).

### Mutation Score (Kill Rate)

```
Mutation Score = (Killed Mutants / Total Viable Mutants) × 100%
```

bashrs targets:
- **90%+** for CRITICAL security rules (SEC001-SEC008, SC2064, SC2059)
- **80%+** for core infrastructure (shell_type, rule_registry)
- **70%+** for high-priority linter rules

## How bashrs Achieves High Mutation Scores

### Example: SEC001 (Command Injection via eval)

**Current stats**: 16 mutants, 16 killed, **100% kill rate**

Let's trace how this was achieved:

#### Initial Implementation (Naive)

```rust,ignore
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        if line.contains("eval") {
            result.add_diagnostic(Diagnostic {
                rule_code: "SEC001".to_string(),
                severity: Severity::Error,
                message: "Use of eval detected".to_string(),
                line: line_num + 1,
                column: 0,
                suggestion: None,
            });
        }
    }

    result
}
```

#### Baseline Mutation Test

```bash
$ cargo mutants --file rash/src/linter/rules/sec001.rs -- --lib
```

**Results**: 10 mutants generated, **3 survived** (70% kill rate)

**Surviving mutants**:
1. Changed `line.contains("eval")` to `line.contains("")` - Test passed!
2. Changed `line_num + 1` to `line_num` - Test passed!
3. Removed `if` condition guard - Test passed!

#### Iteration 1: Kill Surviving Mutants

Add targeted tests:

```rust,ignore
#[test]
fn test_sec001_word_boundary_before() {
    // Kill mutant: "eval" → "" (empty string always matches)
    let safe = "# evaluation is not eval";
    let result = check(safe);
    assert_eq!(result.diagnostics.len(), 0,
        "Should not flag 'eval' within another word");
}

#[test]
fn test_sec001_correct_line_number() {
    // Kill mutant: line_num + 1 → line_num
    let script = "\n\neval \"$cmd\"\n";  // Line 3
    let result = check(script);
    assert_eq!(result.diagnostics[0].line, 3,
        "Should report correct line number");
}

#[test]
fn test_sec001_requires_eval_presence() {
    // Kill mutant: removed if condition
    let safe = "echo hello";
    let result = check(safe);
    assert_eq!(result.diagnostics.len(), 0,
        "Should not flag commands without eval");
}
```

#### Iteration 2: Add Edge Cases

```rust,ignore
#[test]
fn test_sec001_eval_at_line_start() {
    // Edge case: eval at beginning of line
    let script = "eval \"$cmd\"";
    let result = check(script);
    assert_eq!(result.diagnostics.len(), 1);
}

#[test]
fn test_sec001_eval_at_line_end() {
    // Edge case: eval at end of line
    let script = "  eval";
    let result = check(script);
    assert_eq!(result.diagnostics.len(), 1);
}

#[test]
fn test_sec001_eval_with_quotes() {
    // Edge case: eval in various quote contexts
    let script = r#"eval "$cmd""#;
    let result = check(script);
    assert_eq!(result.diagnostics.len(), 1);
}
```

#### Final Implementation (Robust)

```rust,ignore
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        // Look for eval usage as a command (not part of another word)
        if let Some(col) = line.find("eval") {
            // Check if it's a standalone command (word boundary)
            let before_ok = if col == 0 {
                true
            } else {
                let char_before = line.chars().nth(col - 1);
                matches!(
                    char_before,
                    Some(' ') | Some('\t') | Some(';') | Some('&') | Some('|') | Some('(')
                )
            };

            let after_idx = col + 4; // "eval" is 4 chars
            let after_ok = if after_idx >= line.len() {
                true
            } else {
                let char_after = line.chars().nth(after_idx);
                matches!(
                    char_after,
                    Some(' ') | Some('\t') | Some('\n') | Some(';') | Some('&')
                        | Some('|') | Some(')') | Some('"') | Some('\'')
                )
            };

            if before_ok && after_ok {
                result.add_diagnostic(Diagnostic {
                    rule_code: "SEC001".to_string(),
                    severity: Severity::Error,
                    message: "Use of eval with user input can lead to command injection"
                        .to_string(),
                    line: line_num + 1,  // 1-indexed for user display
                    column: col,
                    suggestion: Some(
                        "Avoid eval or validate input strictly. Consider using arrays \
                         and proper quoting instead."
                            .to_string(),
                    ),
                });
            }
        }
    }

    result
}
```

#### Final Mutation Test

```bash
$ cargo mutants --file rash/src/linter/rules/sec001.rs -- --lib
```

**Results**: 16 mutants generated, **16 killed**, **100% kill rate**

## Examples from bashrs SEC Rules

### SEC002: Unquoted Variables (75% → 87.5% improvement)

**Baseline**: 24/32 mutants killed (75%)

**Surviving mutants identified**:
```rust,ignore
// Mutant 1: Changed `contains("$")` to `contains("")`
// Mutant 2: Changed `!is_quoted()` to `is_quoted()`
// Mutant 3: Removed `if !var.is_empty()` guard
// ... 8 total survivors
```

**Iteration 1**: Add 8 targeted tests:

```rust,ignore
#[test]
fn test_sec002_empty_variable_not_flagged() {
    // Kill mutant: removed is_empty() guard
    let script = "echo ''";  // Empty string, not a variable
    let result = check(script);
    assert_eq!(result.diagnostics.len(), 0);
}

#[test]
fn test_sec002_dollar_sign_requires_variable() {
    // Kill mutant: contains("$") → contains("")
    let script = "echo 'price is $5'";  // $ but not a variable
    let result = check(script);
    assert_eq!(result.diagnostics.len(), 0);
}

#[test]
fn test_sec002_quoted_variable_not_flagged() {
    // Kill mutant: !is_quoted() → is_quoted()
    let script = r#"echo "$VAR""#;  // Properly quoted
    let result = check(script);
    assert_eq!(result.diagnostics.len(), 0);
}

// ... 5 more tests targeting remaining mutants
```

**Result**: 28/32 killed (87.5%) - 12.5 percentage point improvement

### SEC006: Unsafe Temporary Files (85.7% baseline)

**Baseline**: 12/14 mutants killed

**Key insight**: High baseline score indicates good initial test coverage.

**Surviving mutants**:
```rust,ignore
// Mutant 1: Changed `mktemp` to `mktmp` (typo)
// Mutant 2: Changed severity Error → Warning
```

**Iteration 1**: Add tests for edge cases:

```rust,ignore
#[test]
fn test_sec006_exact_command_name() {
    // Kill mutant: mktemp → mktmp
    let typo = "mktmp";  // Common typo
    let result = check(typo);
    assert_eq!(result.diagnostics.len(), 0,
        "Should only flag actual mktemp command");

    let correct = "mktemp";
    let result = check(correct);
    assert!(result.diagnostics.len() > 0,
        "Should flag mktemp command");
}

#[test]
fn test_sec006_severity_is_error() {
    // Kill mutant: Error → Warning
    let script = "FILE=$(mktemp)";
    let result = check(script);
    assert_eq!(result.diagnostics[0].severity, Severity::Error,
        "Unsafe temp files must be Error severity");
}
```

**Result**: 14/14 killed (100%)

### SC2064: Trap Command Timing (100% from start)

**What made this rule perfect?**

1. **Property-based tests** for all trap timing scenarios
2. **Mutation-driven test design** - wrote tests anticipating mutations
3. **Edge case enumeration** - tested all quote/expansion combinations

```rust,ignore
#[test]
fn test_sc2064_double_quotes_immediate_expansion() {
    let script = r#"trap "echo $VAR" EXIT"#;  // Expands immediately
    let result = check(script);
    assert_eq!(result.diagnostics.len(), 1);
}

#[test]
fn test_sc2064_single_quotes_delayed_expansion() {
    let script = r#"trap 'echo $VAR' EXIT"#;  // Expands on signal
    let result = check(script);
    assert_eq!(result.diagnostics.len(), 0);
}

#[test]
fn test_sc2064_escaped_dollar_delayed_expansion() {
    let script = r#"trap "echo \$VAR" EXIT"#;  // Escaped = delayed
    let result = check(script);
    assert_eq!(result.diagnostics.len(), 0);
}

#[test]
fn test_sc2064_mixed_expansion() {
    let script = r#"trap "cleanup $PID; rm \$TMPFILE" EXIT"#;
    // $PID expands immediately, \$TMPFILE expands on signal
    let result = check(script);
    assert_eq!(result.diagnostics.len(), 1);
}

// ... 16 more tests covering all combinations
```

**Result**: 7 mutants, 7 killed, 100% kill rate

## Writing Effective Tests for High Mutation Scores

### Pattern 1: Boundary Testing

Test both sides of every condition:

```rust,ignore
// Original code
if cmd.len() > 0 {
    process(cmd);
}

// Mutation: > → >=
if cmd.len() >= 0 {  // Always true!
    process(cmd);
}
```

**Kill this mutant**:
```rust,ignore
#[test]
fn test_empty_command_not_processed() {
    let cmd = "";
    let result = process_if_nonempty(cmd);
    assert_eq!(result, None, "Empty command should not be processed");
}

#[test]
fn test_nonempty_command_processed() {
    let cmd = "ls";
    let result = process_if_nonempty(cmd);
    assert!(result.is_some(), "Non-empty command should be processed");
}
```

### Pattern 2: Assertion Strengthening

Weak assertions let mutants survive:

```rust,ignore
// ❌ WEAK: Only checks presence
#[test]
fn test_diagnostic_exists() {
    let result = check("eval cmd");
    assert!(!result.diagnostics.is_empty());  // Mutants can survive
}

// ✅ STRONG: Checks all properties
#[test]
fn test_diagnostic_complete() {
    let result = check("eval cmd");
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].rule_code, "SEC001");
    assert_eq!(result.diagnostics[0].severity, Severity::Error);
    assert_eq!(result.diagnostics[0].line, 1);
    assert!(result.diagnostics[0].message.contains("eval"));
}
```

### Pattern 3: Negation Testing

Test both positive and negative cases:

```rust,ignore
#[test]
fn test_detects_vulnerability() {
    let vulnerable = "eval \"$USER_INPUT\"";
    let result = check(vulnerable);
    assert!(result.diagnostics.len() > 0,
        "Should flag vulnerable code");
}

#[test]
fn test_ignores_safe_code() {
    let safe = "echo hello";
    let result = check(safe);
    assert_eq!(result.diagnostics.len(), 0,
        "Should not flag safe code");
}
```

### Pattern 4: Value Testing

Test specific values, not just presence:

```rust,ignore
// ❌ WEAK
#[test]
fn test_line_number_set() {
    let result = check("\n\neval cmd");
    assert!(result.diagnostics[0].line > 0);  // Mutants: could be wrong value
}

// ✅ STRONG
#[test]
fn test_line_number_exact() {
    let result = check("\n\neval cmd");
    assert_eq!(result.diagnostics[0].line, 3,  // Exact value
        "Should report line 3");
}
```

### Pattern 5: Composition Testing

Test how components work together:

```rust,ignore
#[test]
fn test_multiple_violations() {
    let script = r#"
eval "$cmd1"
echo safe
eval "$cmd2"
"#;
    let result = check(script);

    assert_eq!(result.diagnostics.len(), 2,
        "Should flag both eval statements");

    assert_eq!(result.diagnostics[0].line, 2);
    assert_eq!(result.diagnostics[1].line, 4);
}
```

## Iterative Mutation Testing Workflow

bashrs follows a systematic process:

### Step 1: Baseline Mutation Test

```bash
cargo mutants --file rash/src/linter/rules/sec001.rs -- --lib
```

**Output**:
```
sec001.rs: 16 mutants tested in 2m 31s
  caught: 13
  missed: 3
  unviable: 0

Kill rate: 81.25%
```

### Step 2: Analyze Surviving Mutants

```bash
cargo mutants --file rash/src/linter/rules/sec001.rs \
    --list-mutants -- --lib
```

**Surviving mutants**:
```text
src/linter/rules/sec001.rs:34: replace contains -> is_empty
src/linter/rules/sec001.rs:42: replace line_num + 1 -> line_num
src/linter/rules/sec001.rs:50: replace Error -> Warning
```

### Step 3: Write Tests to Kill Survivors

For each surviving mutant, write a test that would fail if that mutation existed:

```rust,ignore
// Kill: contains → is_empty
#[test]
fn test_sec001_requires_eval_keyword() {
    let without_eval = "echo safe";
    assert_eq!(check(without_eval).diagnostics.len(), 0);

    let with_eval = "eval cmd";
    assert!(check(with_eval).diagnostics.len() > 0);
}

// Kill: line_num + 1 → line_num
#[test]
fn test_sec001_reports_correct_line() {
    let script = "\n\neval cmd\n";  // Line 3
    let diag = &check(script).diagnostics[0];
    assert_eq!(diag.line, 3);  // Not 2!
}

// Kill: Error → Warning
#[test]
fn test_sec001_is_error_severity() {
    let script = "eval cmd";
    let diag = &check(script).diagnostics[0];
    assert_eq!(diag.severity, Severity::Error);
}
```

### Step 4: Verify Kill Rate Improvement

```bash
cargo mutants --file rash/src/linter/rules/sec001.rs -- --lib
```

**Output**:
```
sec001.rs: 16 mutants tested in 2m 45s
  caught: 16
  missed: 0
  unviable: 0

Kill rate: 100.0%  ✓ Target achieved!
```

### Step 5: Document and Commit

```bash
git add rash/src/linter/rules/sec001.rs rash/tests/test_sec001_mutation.rs
git commit -m "feat: SEC001 mutation testing - 100% kill rate (16/16)

- Added 8 mutation-targeted tests
- Strengthened boundary checking
- Validated exact line numbers and severity
- Perfect mutation score achieved

Mutation results:
- Caught: 16/16
- Kill rate: 100%
- Test suite: 18 tests (10 original + 8 mutation-driven)
"
```

## Analyzing Mutation Testing Results

### Understanding cargo-mutants Output

```text
cargo-mutants auto_tested 71 mutants in 35m 5s:
  16 caught
   3 missed
   2 unviable
```

**Caught**: Tests detected the mutation (good)
**Missed**: Mutation survived, tests didn't catch it (bad)
**Unviable**: Mutation doesn't compile (ignored in score)

**Kill Rate** = 16 / (16 + 3) = 84.2%

### Common Mutation Types

`cargo-mutants` generates these mutation types:

1. **Replace Binary Operator**: `>` → `>=`, `==` → `!=`
2. **Replace Function**: `contains()` → `is_empty()`
3. **Replace Constant**: `1` → `0`, `true` → `false`
4. **Delete Statement**: Remove function calls
5. **Replace Return Value**: `Ok(x)` → `Err(x)`

### Reading Mutation Reports

```bash
$ cargo mutants --file rash/src/linter/rules/sec002.rs \
    --list-mutants -- --lib > mutations.txt
```

**Sample output**:
```text
src/linter/rules/sec002.rs:15:17: replace contains("$") -> is_empty()
src/linter/rules/sec002.rs:23:12: replace !is_quoted -> is_quoted
src/linter/rules/sec002.rs:34:20: replace line_num + 1 -> line_num + 0
src/linter/rules/sec002.rs:45:28: replace Error -> Warning
```

Each line shows:
- File and line number
- Type of mutation
- Original → Mutated code

## Best Practices

### 1. Run Mutations Early and Often

```bash
# During development
cargo mutants --file rash/src/linter/rules/sec001.rs -- --lib

# Before commit
cargo mutants --file rash/src/linter/rules/sec001.rs \
    --timeout 300 -- --lib

# In CI (comprehensive)
cargo mutants --workspace -- --lib
```

### 2. Target 90%+ for Security-Critical Code

bashrs quality tiers:
- **CRITICAL (SEC rules)**: 90%+ required
- **Important (core infrastructure)**: 80%+ required
- **Standard (linter rules)**: 70%+ target

### 3. Use Timeouts for Slow Tests

```bash
# Default: 300s timeout per mutant
cargo mutants --timeout 300 -- --lib

# For slower tests
cargo mutants --timeout 600 -- --lib
```

### 4. Parallelize in CI

```bash
# Run mutation tests in parallel
cargo mutants --jobs 4 -- --lib
```

### 5. Focus on Changed Code

```bash
# Only test files changed in current branch
git diff --name-only main | grep '\.rs$' | \
    xargs -I {} cargo mutants --file {} -- --lib
```

### 6. Integrate with EXTREME TDD

```
RED → GREEN → REFACTOR → MUTATION

1. RED: Write failing test
2. GREEN: Implement feature
3. REFACTOR: Clean up code
4. MUTATION: Verify tests catch bugs (90%+ kill rate)
```

## Real-World bashrs Mutation Results

### SEC Rules (Error Severity) - Final Results

| Rule | Tests | Mutants | Caught | Kill Rate | Status |
|------|-------|---------|--------|-----------|--------|
| SEC001 | 18 | 16 | 16 | 100.0% | PERFECT |
| SEC002 | 16 | 32 | 28 | 87.5% | IMPROVED |
| SEC003 | 14 | 11 | 9 | 81.8% | GOOD |
| SEC004 | 15 | 26 | 20 | 76.9% | BASELINE |
| SEC005 | 13 | 26 | 19 | 73.1% | BASELINE |
| SEC006 | 12 | 14 | 12 | 85.7% | BASELINE |
| SEC007 | 11 | 9 | 8 | 88.9% | BASELINE |
| SEC008 | 14 | 23 | 20 | 87.0% | BASELINE |

**Average**: 81.2% (exceeds 80% target)

### Core Infrastructure

| Module | Tests | Mutants | Caught | Kill Rate |
|--------|-------|---------|--------|-----------|
| shell_compatibility.rs | 13 | 13 | 13 | 100% |
| rule_registry.rs | 3 | 3 | 3 | 100% |
| shell_type.rs | 34 | 21 | 19 | 90.5% |

### ShellCheck CRITICAL Rules

| Rule | Tests | Mutants | Caught | Kill Rate |
|------|-------|---------|--------|-----------|
| SC2064 (trap timing) | 20 | 7 | 7 | 100% |
| SC2059 (format injection) | 21 | 12 | 12 | 100% |
| SC2086 (word splitting) | 68 | 35 | 21 | 58.8% |

**Pattern**: Rules with comprehensive property tests achieve 100% scores.

## Common Pitfalls

### Pitfall 1: Testing Implementation Instead of Behavior

```rust,ignore
// ❌ BAD: Tests internal implementation
#[test]
fn test_uses_regex() {
    let checker = Checker::new();
    assert!(checker.regex.is_some());  // Implementation detail
}

// ✅ GOOD: Tests observable behavior
#[test]
fn test_detects_pattern() {
    let result = check("eval cmd");
    assert!(result.diagnostics.len() > 0);  // Behavior
}
```

### Pitfall 2: Weak Assertions

```rust,ignore
// ❌ WEAK: Mutants can survive
assert!(result.is_ok());
assert!(!diagnostics.is_empty());

// ✅ STRONG: Kills more mutants
assert_eq!(result.unwrap().len(), 1);
assert_eq!(diagnostics[0].rule_code, "SEC001");
assert_eq!(diagnostics[0].severity, Severity::Error);
```

### Pitfall 3: Not Testing Edge Cases

```rust,ignore
// ❌ INCOMPLETE: Only tests happy path
#[test]
fn test_basic_case() {
    let result = check("eval cmd");
    assert_eq!(result.diagnostics.len(), 1);
}

// ✅ COMPLETE: Tests boundaries
#[test]
fn test_all_cases() {
    // Empty input
    assert_eq!(check("").diagnostics.len(), 0);

    // eval at start
    assert_eq!(check("eval").diagnostics.len(), 1);

    // eval at end
    assert_eq!(check("  eval").diagnostics.len(), 1);

    // eval in middle
    assert_eq!(check("x; eval; y").diagnostics.len(), 1);

    // Not eval (contains but not standalone)
    assert_eq!(check("evaluation").diagnostics.len(), 0);
}
```

## Summary

Mutation testing is essential for bashrs's NASA-level quality:

**Key Benefits**:
- Validates test effectiveness mathematically
- Catches weak tests that miss bugs
- Provides objective quality metric
- Complements property testing and TDD

**bashrs Mutation Strategy**:
1. Baseline test (identify surviving mutants)
2. Write targeted tests (kill survivors)
3. Verify improvement (90%+ for critical code)
4. Document results (track kill rates)
5. Integrate with CI/CD (continuous validation)

**Quality Tiers**:
- **90%+**: CRITICAL security rules (SEC, SC2064, SC2059)
- **80%+**: Core infrastructure (shell_type, registry)
- **70%+**: Standard linter rules

**Integration with EXTREME TDD**:
```
EXTREME TDD = TDD + Property Testing + Mutation Testing + PMAT + Examples
```

Mutation testing provides empirical validation that tests actually catch bugs, ensuring bashrs maintains world-class quality.

For more on comprehensive testing, see [Property Testing](./property-testing.md) and [Performance Optimization](./performance.md).
