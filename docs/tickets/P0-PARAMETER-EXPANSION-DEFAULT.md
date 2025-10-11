# P0: Parameter Expansion with Default Values Not Supported

**Severity**: P0 - STOP THE LINE
**Category**: Transpiler - Expression Conversion
**Found During**: GNU Bash Manual validation (Task ID: EXP-PARAM-001)
**Date**: 2025-10-11
**Status**: 🔴 OPEN

---

## Bug Description

The transpiler does not convert Rust's `Option::unwrap_or()` or `std::env::var().unwrap_or()` patterns to shell's `${VAR:-default}` parameter expansion syntax. This is a critical feature for:
- Environment variable configuration
- Optional parameters with sensible defaults
- Idempotent script design

## Expected Behavior

### Example 1: Simple Option::unwrap_or()

**Input (Rust)**:
```rust
fn main() {
    let value = Some("configured");
    let result = value.unwrap_or("default");
    echo(&result);
}
```

**Expected Shell**:
```bash
#!/bin/sh

main() {
    value="configured"
    result="${value:-default}"
    printf '%s\n' "$result"
}

main "$@"
```

### Example 2: Environment Variables with Defaults

**Input (Rust)**:
```rust
fn main() {
    let config_path = std::env::var("CONFIG_PATH")
        .unwrap_or("/etc/default/config".to_string());
    echo(&config_path);
}
```

**Expected Shell**:
```bash
#!/bin/sh

main() {
    config_path="${CONFIG_PATH:-/etc/default/config}"
    printf '%s\n' "$config_path"
}

main "$@"
```

### Example 3: Multiple Variables with Defaults

**Input (Rust)**:
```rust
fn main() {
    let host = std::env::var("HOST").unwrap_or("localhost".to_string());
    let port = std::env::var("PORT").unwrap_or("8080".to_string());
    let proto = std::env::var("PROTO").unwrap_or("http".to_string());
}
```

**Expected Shell**:
```bash
#!/bin/sh

main() {
    host="${HOST:-localhost}"
    port="${PORT:-8080}"
    proto="${PROTO:-http}"
}

main "$@"
```

## Actual Behavior

**Input (Rust)**:
```rust
fn main() {
    let value = Some("configured");
    let result = value.unwrap_or("default");
    echo(&result);
}
```

**Actual Output**:
```bash
#!/bin/sh

main() {
        value="$(Some configured)"  # ❌ Wrong: tries to execute "Some" as command
        result=unknown               # ❌ Wrong: hardcoded "unknown" instead of expansion
        echo "$result"
}
```

**Problems**:
1. `Some("configured")` is incorrectly treated as a command substitution
2. `.unwrap_or("default")` is completely ignored
3. Variable gets hardcoded to `unknown` instead of using parameter expansion
4. No `${VAR:-default}` syntax is generated

## Reproduction

### Minimal Test Case

```rust
// test-default-values.rs
fn main() {
    let result = Some("value").unwrap_or("default");
    println!("{}", result);
}
```

### Steps
```bash
$ cargo run --bin bashrs -- build test-default-values.rs
# Generates incorrect shell script (see Actual Output above)
```

### Files Affected
- All scripts using environment variable defaults
- Configuration management scripts
- Scripts requiring fallback values

## Impact

### Bash Manual Coverage
- **EXP-PARAM-001**: ${parameter:-word} (default value) ❌ BLOCKED
- **EXP-PARAM-002**: ${parameter:=word} (assign default) ❌ BLOCKED
- **EXP-PARAM-003**: ${parameter:?word} (error if unset) ❌ BLOCKED
- **EXP-PARAM-004**: ${parameter:+word} (alternative value) ❌ BLOCKED
- ~10% of Bash manual parameter expansion features blocked

### Workflow 1 (Rust → Shell)
- Cannot handle environment variable configuration
- No sensible defaults for optional parameters
- Configuration management broken
- Deployment scripts requiring ENV vars blocked

### Workflow 2 (Bash → Rust → Purified Bash)
- Cannot preserve `${VAR:-default}` patterns
- Loss of idempotency guarantees
- Configuration flexibility reduced

## Root Cause Analysis

The transpiler's expression converter (`rash/src/services/parser.rs` or `rash/src/transpiler/`) does not:

1. Recognize `Option::unwrap_or(default)` method call pattern
2. Recognize `Result::unwrap_or(default)` pattern
3. Map `std::env::var("VAR").unwrap_or(default)` to `${VAR:-default}`
4. Generate shell parameter expansion syntax

This is a **parser/transpiler limitation** affecting configuration management and environment variable handling.

## Priority Justification

**P0 (STOP THE LINE)** because:
1. Blocks 10% of Bash manual validation (4 parameter expansion tasks)
2. Critical for real-world deployment scripts (ENV configuration)
3. Affects idempotency and configuration management
4. Fundamental pattern for shell scripting
5. Required for both Workflow 1 and Workflow 2

---

## Fix Plan

### Step 1: RED Phase - Write Failing Tests ✅ COMPLETE

```rust
// rash/tests/integration_tests.rs (lines 494-577)

#[test]
#[ignore]
fn test_string_parameter_expansion_default() {
    let source = r#"
fn main() {
    let value = Some("configured");
    let result = value.unwrap_or("default");
    echo(&result);
}
"#;

    let config = Config::default();
    let shell = transpile(source, config).unwrap();

    assert!(shell.contains("${value:-default}"));
}

#[test]
#[ignore]
fn test_env_var_with_default() {
    let source = r#"
fn main() {
    let config_path = std::env::var("CONFIG_PATH")
        .unwrap_or("/etc/default/config".to_string());
    echo(&config_path);
}
"#;

    let shell = transpile(source, config).unwrap();
    assert!(shell.contains("${CONFIG_PATH:-/etc/default/config}"));
}

#[test]
#[ignore]
fn test_multiple_defaults() {
    // Tests multiple env vars with defaults
    // (see integration_tests.rs:554-577)
}
```

**Run**: `cargo test --test integration_tests -- --ignored`
**Status**: ✅ All 3 tests FAIL as expected (RED phase confirmed)

### Step 2: GREEN Phase - Implementation

**Phase 2a: Recognize unwrap_or() Pattern**

Location: `rash/src/services/parser.rs` (method call handling)

```rust
fn convert_method_call(expr: &syn::ExprMethodCall) -> Result<Expr> {
    match expr.method.to_string().as_str() {
        "unwrap_or" => {
            // Convert to ParameterExpansion AST node
            let receiver = convert_expr(&expr.receiver)?;
            let default = convert_expr(&expr.args[0])?;

            Ok(Expr::ParameterExpansion {
                variable: receiver,
                expansion_type: ExpansionType::DefaultValue,
                value: Some(Box::new(default)),
            })
        }
        // ... existing method calls
    }
}
```

**Phase 2b: Update AST**

Location: `rash/src/ast/restricted.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expr {
    // ... existing variants

    /// ${var:-default} parameter expansion
    ParameterExpansion {
        variable: Box<Expr>,
        expansion_type: ExpansionType,
        value: Option<Box<Expr>>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExpansionType {
    DefaultValue,      // ${var:-word}
    AssignDefault,     // ${var:=word}
    ErrorIfUnset,      // ${var:?word}
    AlternativeValue,  // ${var:+word}
}
```

**Phase 2c: Update IR Conversion**

Location: `rash/src/ir/mod.rs`

```rust
fn convert_parameter_expansion(
    variable: &Expr,
    expansion_type: &ExpansionType,
    value: &Option<Box<Expr>>,
) -> IrNode {
    // Convert to IR representation
    IrNode::ParameterExpansion {
        var_name: extract_variable_name(variable),
        operator: match expansion_type {
            ExpansionType::DefaultValue => ":-",
            ExpansionType::AssignDefault => ":=",
            ExpansionType::ErrorIfUnset => ":?",
            ExpansionType::AlternativeValue => ":+",
        },
        default_value: value.as_ref().map(|v| convert_expr(v)),
    }
}
```

**Phase 2d: Update Shell Emitter**

Location: `rash/src/emitter/mod.rs`

```rust
fn emit_parameter_expansion(
    var_name: &str,
    operator: &str,
    default_value: &Option<String>,
) -> String {
    match default_value {
        Some(default) => {
            // Emit: variable="${VAR:-default}"
            format!("\"${{{}{}{}}}\"", var_name, operator, default)
        }
        None => {
            // Emit: variable="$VAR"
            format!("\"${}\"", var_name)
        }
    }
}
```

**Phase 2e: Handle std::env::var() Pattern**

```rust
fn convert_call_expr(expr: &syn::ExprCall) -> Result<Expr> {
    if is_env_var_call(expr) {
        let env_var_name = extract_env_var_name(expr)?;

        // Check if followed by .unwrap_or()
        // Convert to ParameterExpansion with ENV variable name
        return Ok(Expr::ParameterExpansion {
            variable: Box::new(Expr::Variable(env_var_name)),
            expansion_type: ExpansionType::DefaultValue,
            value: None, // Will be filled by subsequent unwrap_or()
        });
    }

    // ... existing code
}
```

### Step 3: REFACTOR Phase

- Extract `convert_unwrap_or_pattern()` helper
- Extract `emit_parameter_expansion()` helper
- Ensure cognitive complexity <10
- Add comprehensive inline documentation

### Step 4: Property Testing

```rust
#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_parameter_expansion_always_quoted(
            var_name in "[A-Z_][A-Z0-9_]{0,20}",
            default in "[a-z0-9/_.-]{1,50}"
        ) {
            let rust = format!(r#"
                fn main() {{
                    let val = std::env::var("{}").unwrap_or("{}".to_string());
                }}
            "#, var_name, default);

            let shell = transpile(&rust).unwrap();

            // Verify proper quoting
            assert!(shell.contains(&format!("\"${{{}:-{}}}", var_name, default)));
        }

        #[test]
        fn prop_determinism(code in any::<String>()) {
            // Verify transpilation is deterministic
            if let Ok(shell1) = transpile(&code) {
                let shell2 = transpile(&code).unwrap();
                assert_eq!(shell1, shell2);
            }
        }
    }
}
```

### Step 5: Mutation Testing

```bash
cargo mutants --file rash/src/services/parser.rs -F unwrap_or
cargo mutants --file rash/src/emitter/mod.rs -F parameter_expansion
# Target: ≥90% kill rate
```

### Step 6: Integration Testing

```rust
#[test]
fn test_integration_parameter_expansion() {
    let rust = r#"
fn main() {
    let host = std::env::var("HOST").unwrap_or("localhost".to_string());
    let port = std::env::var("PORT").unwrap_or("8080".to_string());

    // Use the variables
    println!("Connecting to {}:{}", host, port);
}
"#;

    let shell = transpile(rust).unwrap();

    // Verify shellcheck passes
    assert!(run_shellcheck(&shell).success());

    // Verify determinism
    let shell2 = transpile(rust).unwrap();
    assert_eq!(shell, shell2);

    // Verify execution with ENV vars set
    let output = run_shell_with_env(
        &shell,
        &[("HOST", "example.com"), ("PORT", "3000")]
    ).unwrap();
    assert!(output.contains("example.com:3000"));

    // Verify execution with defaults (no ENV vars)
    let output = run_shell(&shell, &[]).unwrap();
    assert!(output.contains("localhost:8080"));
}
```

### Step 7: Regression Prevention

- Add tests to permanent suite (remove `#[ignore]`)
- Update `BASH-INGESTION-ROADMAP.yaml` → EXP-PARAM-001: completed
- Update `CHANGELOG.md` with fix
- Close this P0 ticket

---

## Verification Checklist

Before resuming Bash manual validation:

- [x] ✅ **RED**: 3 failing tests written and verified (integration_tests.rs:494-577)
- [ ] ✅ **GREEN**: Implementation complete, all tests pass
- [ ] ✅ **REFACTOR**: Code cleaned up, complexity <10
- [ ] ✅ **All tests pass**: 811+ tests (808 + 3 new), 100% pass rate
- [ ] ✅ **Property tests**: Quoting/determinism verified
- [ ] ✅ **Mutation test**: ≥90% kill rate on new code
- [ ] ✅ **Integration test**: End-to-end ENV var handling verified
- [ ] ✅ **Shellcheck**: Generated output passes POSIX compliance
- [ ] ✅ **Documentation**: CHANGELOG, roadmap updated
- [ ] ✅ **Ticket closed**: P0 marked as RESOLVED

---

**Current Status**: RED Phase complete, awaiting GREEN phase implementation

**Estimated Implementation Time**: 6-8 hours

**Impact**: CRITICAL - blocks environment variable configuration and parameter expansion validation

🚨 **STATUS**: STOP THE LINE - Awaiting implementation decision
