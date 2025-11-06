# Writing Custom Lint Rules

This guide explains how to implement custom lint rules in bashrs using EXTREME TDD methodology. Custom rules extend bashrs's linting capabilities for project-specific requirements.

## Overview

bashrs's linting architecture supports:
- **Pattern-based rules**: Regex and string matching (most common)
- **AST-based rules**: Deep semantic analysis (advanced)
- **Auto-fix support**: Safe, safe-with-assumptions, or unsafe fixes
- **Shell compatibility**: Rules can be shell-specific (sh, bash, zsh)
- **Comprehensive testing**: Unit, property, mutation, and integration tests

## Rule Architecture

### Rule Structure

Every lint rule is a Rust module implementing a `check()` function:

```rust,ignore
//! RULEID: Short description
//!
//! **Rule**: What pattern this detects
//!
//! **Why this matters**: Impact and reasoning
//!
//! **Auto-fix**: Fix strategy (if applicable)

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};

/// Check function - entry point for the rule
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Rule implementation here

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests here
}
```

### Core Types

**Diagnostic**: Represents a lint violation

```rust,ignore
pub struct Diagnostic {
    pub code: String,        // "DET001", "SEC002", etc.
    pub severity: Severity,  // Error, Warning, Info, etc.
    pub message: String,     // Human-readable message
    pub span: Span,          // Source location
    pub fix: Option<Fix>,    // Suggested fix (optional)
}
```

**Span**: Source code location (1-indexed)

```rust,ignore
pub struct Span {
    pub start_line: usize,  // 1-indexed line number
    pub start_col: usize,   // 1-indexed column
    pub end_line: usize,
    pub end_col: usize,
}
```

**Fix**: Auto-fix suggestion

```rust,ignore
pub struct Fix {
    pub replacement: String,             // Replacement text
    pub safety_level: FixSafetyLevel,    // Safe, SafeWithAssumptions, Unsafe
    pub assumptions: Vec<String>,        // For SafeWithAssumptions
    pub suggested_alternatives: Vec<String>,  // For Unsafe
}
```

**Severity Levels**:
- `Info`: Style suggestions
- `Note`: Informational
- `Perf`: Performance anti-patterns
- `Risk`: Potential runtime failure
- `Warning`: Likely bug
- `Error`: Definite error (must fix)

## EXTREME TDD Workflow for Rules

### Phase 1: RED - Write Failing Test

Start with a test that defines the desired behavior:

```rust,ignore
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_CUSTOM001_detects_pattern() {
        let script = "#!/bin/bash\ndangerous_pattern";
        let result = check(script);

        // Verify detection
        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "CUSTOM001");
        assert_eq!(diag.severity, Severity::Error);
        assert!(diag.message.contains("dangerous"));
    }
}
```

Run the test - it should FAIL:
```bash
cargo test test_CUSTOM001_detects_pattern
```

### Phase 2: GREEN - Implement Rule

Implement the minimal code to make the test pass:

```rust,ignore
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        if line.contains("dangerous_pattern") {
            let col = line.find("dangerous_pattern").unwrap();

            let span = Span::new(
                line_num + 1,
                col + 1,
                line_num + 1,
                col + 17,  // "dangerous_pattern" length
            );

            let diag = Diagnostic::new(
                "CUSTOM001",
                Severity::Error,
                "Dangerous pattern detected",
                span,
            );

            result.add(diag);
        }
    }

    result
}
```

Run test again - should PASS:
```bash
cargo test test_CUSTOM001_detects_pattern
```

### Phase 3: REFACTOR - Clean Up

Extract helpers, improve readability, ensure complexity <10:

```rust,ignore
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        if let Some(violation) = detect_pattern(line, line_num) {
            result.add(violation);
        }
    }

    result
}

fn detect_pattern(line: &str, line_num: usize) -> Option<Diagnostic> {
    if !line.contains("dangerous_pattern") {
        return None;
    }

    let col = line.find("dangerous_pattern")?;
    let span = Span::new(line_num + 1, col + 1, line_num + 1, col + 17);

    Some(Diagnostic::new(
        "CUSTOM001",
        Severity::Error,
        "Dangerous pattern detected",
        span,
    ))
}
```

### Phase 4: Property Testing

Add generative tests to verify properties:

```rust,ignore
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_no_false_positives(
            safe_code in "[a-z]{1,100}"
                .prop_filter("Must not contain pattern", |s| !s.contains("dangerous"))
        ) {
            let result = check(&safe_code);
            // Property: Safe code produces no diagnostics
            prop_assert_eq!(result.diagnostics.len(), 0);
        }

        #[test]
        fn prop_always_detects_pattern(
            prefix in "[a-z]{0,50}",
            suffix in "[a-z]{0,50}"
        ) {
            let code = format!("{}dangerous_pattern{}", prefix, suffix);
            let result = check(&code);
            // Property: Pattern is always detected
            prop_assert!(result.diagnostics.len() >= 1);
        }
    }
}
```

### Phase 5: Mutation Testing

Verify test quality with cargo-mutants:

```bash
cargo mutants --file rash/src/linter/rules/custom001.rs --timeout 300
```

**Target**: ≥90% kill rate

If mutations survive, add tests to kill them:

```rust,ignore
#[test]
fn test_mutation_exact_column() {
    // Kills mutation: col + 1 → col * 1
    let script = "  dangerous_pattern";  // 2 spaces before
    let result = check(script);
    let span = result.diagnostics[0].span;
    assert_eq!(span.start_col, 3);  // Must be 3, not 0 or 2
}

#[test]
fn test_mutation_line_number() {
    // Kills mutation: line_num + 1 → line_num * 1
    let script = "safe\ndangerous_pattern";
    let result = check(script);
    let span = result.diagnostics[0].span;
    assert_eq!(span.start_line, 2);  // Must be 2, not 1
}
```

### Phase 6: Integration Testing

Test end-to-end with realistic scripts:

```rust,ignore
#[test]
fn test_integration_full_script() {
    let script = r#"
#!/bin/bash
set -e

function deploy() {
    dangerous_pattern  # Should detect
    safe_code
}

deploy
"#;

    let result = check(script);
    assert_eq!(result.diagnostics.len(), 1);

    // Verify correct line number
    assert_eq!(result.diagnostics[0].span.start_line, 6);
}
```

### Phase 7: pmat Verification

Verify code quality:

```bash
# Complexity check
pmat analyze complexity --file rash/src/linter/rules/custom001.rs --max 10

# Quality score
pmat quality-score --min 9.0
```

### Phase 8: Example Verification

Create example that demonstrates the rule:

```bash
# examples/custom_rule_demo.sh
#!/bin/bash
# Demonstrates CUSTOM001 rule

dangerous_pattern  # Will be caught by linter
```

Run linter on example:
```bash
cargo run -- lint examples/custom_rule_demo.sh
```

## Example: Implementing a Security Rule

Let's implement SEC009: Detect unquoted command substitution in eval.

### Step 1: RED Phase

```rust,ignore
// rash/src/linter/rules/sec009.rs
//! SEC009: Unquoted command substitution in eval
//!
//! **Rule**: Detect eval with unquoted $(...)
//!
//! **Why this matters**: eval "$(cmd)" is vulnerable to injection

use crate::linter::{Diagnostic, LintResult, Severity, Span};

pub fn check(source: &str) -> LintResult {
    LintResult::new()  // Empty - will fail tests
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_SEC009_detects_unquoted_command_sub() {
        let script = r#"eval $(get_command)"#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "SEC009");
        assert_eq!(diag.severity, Severity::Error);
    }

    #[test]
    fn test_SEC009_no_warning_for_quoted() {
        let script = r#"eval "$(get_command)""#;
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0);
    }
}
```

Run test:
```bash
cargo test test_SEC009_detects_unquoted_command_sub
# FAILS - as expected (RED)
```

### Step 2: GREEN Phase

```rust,ignore
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        // Check for eval $(...) pattern
        if line.contains("eval") && line.contains("$(") {
            // Verify not quoted
            if !is_quoted(line, "eval") {
                if let Some(col) = line.find("eval") {
                    let span = Span::new(
                        line_num + 1,
                        col + 1,
                        line_num + 1,
                        col + 5,
                    );

                    let diag = Diagnostic::new(
                        "SEC009",
                        Severity::Error,
                        "Unquoted command substitution in eval - command injection risk",
                        span,
                    );

                    result.add(diag);
                }
            }
        }
    }

    result
}

fn is_quoted(line: &str, pattern: &str) -> bool {
    if let Some(pos) = line.find(pattern) {
        // Simple heuristic: check if followed by quote
        let after = &line[pos + pattern.len()..];
        after.trim_start().starts_with('"')
    } else {
        false
    }
}
```

Run tests:
```bash
cargo test test_SEC009
# PASSES - GREEN achieved!
```

### Step 3: REFACTOR Phase

```rust,ignore
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        if let Some(violation) = detect_unquoted_eval(line, line_num) {
            result.add(violation);
        }
    }

    result
}

fn detect_unquoted_eval(line: &str, line_num: usize) -> Option<Diagnostic> {
    // Must have both eval and command substitution
    if !line.contains("eval") || !line.contains("$(") {
        return None;
    }

    // Check if quoted
    if is_command_sub_quoted(line) {
        return None;
    }

    // Find eval position
    let col = line.find("eval")?;

    let span = Span::new(line_num + 1, col + 1, line_num + 1, col + 5);

    Some(Diagnostic::new(
        "SEC009",
        Severity::Error,
        "Unquoted command substitution in eval - command injection risk",
        span,
    ))
}

fn is_command_sub_quoted(line: &str) -> bool {
    // Check for eval "$(...)" pattern
    line.contains(r#"eval "$"#) || line.contains(r#"eval '$"#)
}
```

### Step 4: Property Testing

```rust,ignore
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_safe_code_no_warnings(
            safe_code in "[a-z ]{1,50}"
                .prop_filter("No eval", |s| !s.contains("eval"))
        ) {
            let result = check(&safe_code);
            prop_assert_eq!(result.diagnostics.len(), 0);
        }

        #[test]
        fn prop_quoted_eval_safe(
            cmd in "[a-z_]{1,20}"
        ) {
            let code = format!(r#"eval "$({})""#, cmd);
            let result = check(&code);
            prop_assert_eq!(result.diagnostics.len(), 0);
        }

        #[test]
        fn prop_unquoted_eval_detected(
            cmd in "[a-z_]{1,20}"
        ) {
            let code = format!("eval $({})", cmd);
            let result = check(&code);
            prop_assert!(result.diagnostics.len() >= 1);
        }
    }
}
```

### Step 5: Mutation Testing

```bash
cargo mutants --file rash/src/linter/rules/sec009.rs --timeout 300
```

Add tests to kill survivors:

```rust,ignore
#[test]
fn test_mutation_column_calculation() {
    let script = "  eval $(cmd)";  // 2-space indent
    let result = check(script);
    assert_eq!(result.diagnostics[0].span.start_col, 3);
}

#[test]
fn test_mutation_line_number() {
    let script = "safe\neval $(cmd)";
    let result = check(script);
    assert_eq!(result.diagnostics[0].span.start_line, 2);
}
```

### Step 6: Register Rule

Add to `rash/src/linter/rules/mod.rs`:

```rust,ignore
pub mod sec009;

// In lint_shell() function:
result.merge(sec009::check(source));
```

### Step 7: Documentation

Add rule to security documentation:

```markdown
## SEC009: Unquoted Command Substitution in eval

**Severity**: Error (Critical)

### Examples

❌ **VULNERABILITY**:
```bash
eval $(get_command)
```

✅ **SAFE**:
```bash
eval "$(get_command)"
```

## Adding Auto-fix Support

### Safe Fix Example

For deterministic fixes (quoting variables):

```rust,ignore
let fix = Fix::new("\"${VAR}\"");  // Safe replacement

let diag = Diagnostic::new(
    "SC2086",
    Severity::Error,
    "Quote variable to prevent word splitting",
    span,
).with_fix(fix);
```

### Safe-with-Assumptions Fix Example

For fixes that work in most cases:

```rust,ignore
let fix = Fix::new_with_assumptions(
    "mkdir -p",
    vec!["Directory creation failure is not critical".to_string()],
);

let diag = Diagnostic::new(
    "IDEM001",
    Severity::Warning,
    "Non-idempotent mkdir - add -p flag",
    span,
).with_fix(fix);
```

### Unsafe Fix Example

For fixes requiring human judgment:

```rust,ignore
let fix = Fix::new_unsafe(vec![
    "Option 1: Use version: ID=\"${VERSION}\"".to_string(),
    "Option 2: Use git commit: ID=\"$(git rev-parse HEAD)\"".to_string(),
    "Option 3: Pass as argument: ID=\"$1\"".to_string(),
]);

let diag = Diagnostic::new(
    "DET001",
    Severity::Error,
    "Non-deterministic $RANDOM - requires manual fix",
    span,
).with_fix(fix);
```

## Shell Compatibility

### Marking Rules as Shell-Specific

Register rule compatibility in `rule_registry.rs`:

```rust,ignore
pub fn get_rule_compatibility(rule_id: &str) -> ShellCompatibility {
    match rule_id {
        // Bash-only features
        "SC2198" => ShellCompatibility::NotSh,  // Arrays
        "SC2199" => ShellCompatibility::NotSh,
        "SC2200" => ShellCompatibility::NotSh,

        // Universal (all shells)
        "SEC001" => ShellCompatibility::Universal,
        "DET001" => ShellCompatibility::Universal,
        "IDEM001" => ShellCompatibility::Universal,

        // Default: assume universal
        _ => ShellCompatibility::Universal,
    }
}
```

### Shell Types

- `ShellType::Sh`: POSIX sh
- `ShellType::Bash`: GNU Bash
- `ShellType::Zsh`: Z shell
- `ShellType::Dash`: Debian Almquist shell
- `ShellType::Ksh`: Korn shell
- `ShellType::Ash`: Almquist shell
- `ShellType::BusyBox`: BusyBox sh

## Pattern-Based vs AST-Based Rules

### Pattern-Based Rules (Recommended)

Most rules use regex or string matching:

**Pros**:
- Simple to implement
- Fast execution
- Easy to test
- Good for 90% of use cases

**Example**:
```rust,ignore
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        if line.contains("dangerous_pattern") {
            // Create diagnostic
        }
    }

    result
}
```

### AST-Based Rules (Advanced)

For semantic analysis:

**Pros**:
- Semantic understanding
- Context-aware
- Fewer false positives

**Cons**:
- Complex implementation
- Slower execution
- Requires parser

**Example**:
```rust,ignore
use crate::parser::bash_parser;

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Parse to AST
    let ast = bash_parser::parse(source)?;

    // Traverse AST
    for node in ast.commands() {
        if let Command::Function(func) = node {
            // Analyze function semantics
        }
    }

    result
}
```

## Testing Best Practices

### Comprehensive Test Coverage

Every rule needs:

1. **Basic detection tests**:
   ```rust
   #[test]
   fn test_detects_violation() { }
   ```

2. **No false positive tests**:
   ```rust
   #[test]
   fn test_no_false_positive() { }
   ```

3. **Edge case tests**:
   ```rust
   #[test]
   fn test_edge_case_empty_line() { }
   ```

4. **Property tests**:
   ```rust,ignore
   proptest! { fn prop_no_false_positives() { } }
   ```

5. **Mutation tests**:
   ```bash
   cargo mutants --file rash/src/linter/rules/custom001.rs
   ```

6. **Integration tests**:
   ```rust
   #[test]
   fn test_integration_real_script() { }
   ```

### Test Naming Convention

Format: `test_<RULE_ID>_<feature>_<scenario>`

Examples:
```rust,ignore
#[test]
fn test_SEC009_detects_unquoted_eval() { }

#[test]
fn test_SEC009_no_warning_for_quoted() { }

#[test]
fn test_SEC009_handles_multiline() { }
```

## Common Patterns

### Pattern 1: Simple String Matching

```rust,ignore
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        if let Some(col) = line.find("pattern") {
            let span = Span::new(line_num + 1, col + 1, line_num + 1, col + 8);
            result.add(Diagnostic::new("CODE", Severity::Warning, "Message", span));
        }
    }

    result
}
```

### Pattern 2: Regex Matching

```rust,ignore
use regex::Regex;

lazy_static::lazy_static! {
    static ref PATTERN: Regex = Regex::new(r"\$RANDOM").unwrap();
}

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        if let Some(m) = PATTERN.find(line) {
            let span = Span::new(
                line_num + 1,
                m.start() + 1,
                line_num + 1,
                m.end() + 1,
            );
            result.add(Diagnostic::new("CODE", Severity::Error, "Message", span));
        }
    }

    result
}
```

### Pattern 3: Context-Aware Detection

```rust,ignore
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        // Only check in specific context
        if line.trim_start().starts_with("eval") {
            if line.contains("$(") && !line.contains(r#""$""#) {
                // Detect violation
            }
        }
    }

    result
}
```

### Pattern 4: Multi-line Pattern

```rust,ignore
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let lines: Vec<&str> = source.lines().collect();

    for i in 0..lines.len() {
        // Check current + next line
        if i + 1 < lines.len() {
            if lines[i].contains("pattern_part1") &&
               lines[i + 1].contains("pattern_part2") {
                // Detect violation spanning lines
            }
        }
    }

    result
}
```

## CI/CD Integration

### Test Rules in CI

```yaml
# .github/workflows/lint-rules.yml
name: Test Lint Rules
on: [push, pull_request]
jobs:
  test-rules:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Run unit tests
        run: cargo test --lib sec009
      - name: Run property tests
        run: cargo test --lib prop_ --release
      - name: Run mutation tests
        run: |
          cargo install cargo-mutants
          cargo mutants --file rash/src/linter/rules/sec009.rs --timeout 300
```

## Troubleshooting

### Rule Not Triggering

1. Check pattern matching logic
2. Verify span calculation (1-indexed!)
3. Test with minimal example
4. Add debug prints:
   ```rust,ignore
   eprintln!("Line {}: {}", line_num, line);
   eprintln!("Pattern match: {:?}", line.find("pattern"));
   ```

### False Positives

1. Add context checks
2. Use more specific patterns
3. Check for quoted strings
4. Ignore comments
5. Add exclusion tests

### Mutation Tests Failing

1. Review survived mutants:
   ```bash
   cargo mutants --file rash/src/linter/rules/sec009.rs --list
   ```
2. Add tests targeting specific mutations
3. Verify edge cases covered

## Further Reading

- [bashrs Rule Registry](/linting/security.md)
- [EXTREME TDD Guide](/CLAUDE.md#extreme-tdd-definition)
- [Mutation Testing](https://pitest.org/)
- [Property Testing with Proptest](https://proptest-rs.github.io/proptest/)

---

**Quality Standard**: All custom rules must achieve ≥90% mutation kill rate and pass comprehensive property tests before merging.
