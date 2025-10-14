# Sprint 27a: Environment Variables Support - CHECKPOINT

```yaml
checkpoint:
  sprint: "Sprint 27a - Environment Variables Only"
  date: "2025-10-14"
  status: "RED_PHASE_IN_PROGRESS"
  completion: "16.7% (2/12 RED tests)"
  next_phase: "Complete RED phase (write 10 more tests)"

context:
  parent_sprint: "Sprint 27 - Core Shell Features Enhancement"
  methodology: "EXTREME TDD (RED-GREEN-REFACTOR)"
  duration_estimate: "2-3 hours total"
  time_elapsed: "~30 minutes"
  time_remaining: "~90-150 minutes"

current_state:
  tests_written: 2
  tests_passing: 0
  tests_failing: 2
  files_modified: 1
  commits: 1
  quality_status: "RED phase proceeding correctly ✅"
```

## Progress Summary

### ✅ Completed (RED Phase - 2/12 tests)

**File**: `rash/src/stdlib.rs` (lines 160-171)

Added 2 failing tests:

```rust
// Sprint 27a: Environment Variables Support - RED PHASE
#[test]
fn test_stdlib_env_function_recognized() {
    // RED: This test will fail until we add "env" to is_stdlib_function()
    assert!(is_stdlib_function("env"), "env() should be recognized as stdlib function");
}

#[test]
fn test_stdlib_env_var_or_function_recognized() {
    // RED: This test will fail until we add "env_var_or" to is_stdlib_function()
    assert!(is_stdlib_function("env_var_or"), "env_var_or() should be recognized as stdlib function");
}
```

**Test Results** (verified failing correctly):
```bash
cargo test --package bashrs --lib stdlib::tests::test_stdlib_env
# Output:
test stdlib::tests::test_stdlib_env_function_recognized ... FAILED
test stdlib::tests::test_stdlib_env_var_or_function_recognized ... FAILED
# ✅ Both tests fail as expected (RED phase)
```

### ⏳ Pending (RED Phase - 10 more tests)

#### IR Conversion Tests (3-4 tests)
**File**: `rash/src/ir/tests.rs` (to be added)

1. `test_env_call_converts_to_ir`
   - Parse `env("HOME")`
   - Verify IR has `ShellValue::EnvVar { name: "HOME", default: None }`

2. `test_env_var_or_call_converts_to_ir`
   - Parse `env_var_or("PREFIX", "/usr/local")`
   - Verify IR has `ShellValue::EnvVar { name: "PREFIX", default: Some("/usr/local") }`

3. `test_env_in_assignment`
   - Parse `let home = env("HOME");`
   - Verify variable assignment with EnvVar value

4. `test_env_in_string_interpolation`
   - Parse string containing env() call
   - Verify correct IR structure

#### Emitter Tests (3-4 tests)
**File**: `rash/src/emitter/tests.rs` (to be added)

1. `test_env_emits_dollar_brace_syntax`
   - Input: `ShellValue::EnvVar { name: "HOME", default: None }`
   - Expected output: `"${HOME}"`

2. `test_env_var_or_emits_with_default`
   - Input: `ShellValue::EnvVar { name: "PREFIX", default: Some("/usr") }`
   - Expected output: `"${PREFIX:-/usr}"`

3. `test_env_var_quoted_for_safety`
   - Verify all env var expansions are properly quoted
   - Test: `home="${HOME}"` not `home=${HOME}`

4. `test_env_complex_default_value`
   - Test default values with spaces, special chars
   - Verify proper escaping

#### Security Tests (2-3 tests)
**File**: `rash/src/stdlib.rs` or `rash/src/ir/mod.rs`

1. `test_env_rejects_invalid_var_names`
   - Input: `env("'; rm -rf /; #")`
   - Expected: Error or sanitization
   - Valid: alphanumeric + underscore only

2. `test_env_var_or_escapes_default`
   - Input: `env_var_or("VAR", "\"; rm -rf /; echo \"")`
   - Expected: Proper escaping in output
   - No injection possible

3. `prop_env_calls_are_safe` (property test)
   - Generate random var names and defaults
   - Verify no injection vectors

#### Integration Test (1 test)
**File**: `rash/tests/integration/environment.rs` (new file)

1. `test_env_integration_end_to_end`
   - Full Rust code with env() and env_var_or()
   - Transpile to shell
   - Verify shellcheck passes
   - Verify correct behavior

## Technical Design

### Layer 1: stdlib.rs (GREEN phase changes)
```rust
pub fn is_stdlib_function(name: &str) -> bool {
    matches!(
        name,
        // String module
        "string_trim" | "string_contains" | /* ... */
        // File system module
        "fs_exists" | "fs_read_file" | /* ... */
        // Array module
        "array_len" | "array_join"
        // Environment module - TO ADD:
        | "env"
        | "env_var_or"
    )
}

pub const STDLIB_FUNCTIONS: &[StdlibFunction] = &[
    // ... existing functions ...
    // TO ADD:
    StdlibFunction {
        name: "env",
        shell_name: "rash_env", // OR direct ${VAR} expansion
        module: "env",
        description: "Get environment variable value",
    },
    StdlibFunction {
        name: "env_var_or",
        shell_name: "rash_env_var_or", // OR direct ${VAR:-default}
        module: "env",
        description: "Get environment variable with default",
    },
];
```

### Layer 2: ir/shell_ir.rs (GREEN phase changes)
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShellValue {
    String(String),
    Bool(bool),
    Variable(String),
    // ... existing variants ...

    /// Environment variable expansion
    /// TO ADD:
    EnvVar {
        name: String,
        default: Option<String>,
    },
}
```

### Layer 3: ir/mod.rs converter (GREEN phase changes)
```rust
// In convert_function_call() or convert_expr()
fn convert_function_call(name: &str, args: &[Expr]) -> Result<ShellValue> {
    match name {
        "env" => {
            // Extract var name from args[0]
            let var_name = extract_string_literal(&args[0])?;
            Ok(ShellValue::EnvVar {
                name: var_name,
                default: None,
            })
        }
        "env_var_or" => {
            let var_name = extract_string_literal(&args[0])?;
            let default = extract_string_literal(&args[1])?;
            Ok(ShellValue::EnvVar {
                name: var_name,
                default: Some(default),
            })
        }
        // ... existing cases ...
    }
}
```

### Layer 4: emitter/mod.rs (GREEN phase changes)
```rust
// In emit_value()
fn emit_value(value: &ShellValue) -> Result<String> {
    match value {
        ShellValue::EnvVar { name, default } => {
            validate_var_name(name)?; // Security check

            match default {
                None => Ok(format!("\"${{{}}}\"", name)),
                Some(def) => {
                    let escaped = escape_default_value(def)?; // Security
                    Ok(format!("\"${{{}:-{}}}\"", name, escaped))
                }
            }
        }
        // ... existing cases ...
    }
}

fn validate_var_name(name: &str) -> Result<()> {
    if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err(format!("Invalid variable name: {}", name));
    }
    Ok(())
}

fn escape_default_value(value: &str) -> Result<String> {
    // Escape special characters for shell safety
    // TODO: Implement proper escaping
    Ok(value.replace("\"", "\\\""))
}
```

## Resumption Instructions

### Step 1: Continue RED Phase
```bash
cd /home/noahgift/src/bashrs

# Write next batch of RED tests (IR conversion)
# Add tests to: rash/src/ir/tests.rs
```

### Step 2: Run Tests to Verify RED
```bash
cargo test --lib
# Expect: 813 passing, 2+N failing (where N = new RED tests)
```

### Step 3: Start GREEN Phase (after all RED tests written)
```bash
# 1. Update stdlib.rs
# 2. Update ir/shell_ir.rs
# 3. Update ir/mod.rs
# 4. Update emitter/mod.rs
# 5. Run tests - aim for all passing
cargo test --lib
```

### Step 4: REFACTOR Phase
```bash
cargo clippy --fix
cargo fmt
# Extract helper functions
# Add documentation
```

### Step 5: Complete Sprint
```bash
# Run examples
cargo run --example environment-setup > /tmp/env-setup.sh
shellcheck /tmp/env-setup.sh

# Update docs
# Create completion report
# Commit and push
```

## Key Files Reference

### Currently Modified
- `rash/src/stdlib.rs` - 2 RED tests added (lines 160-171)

### Need Modification (GREEN phase)
- `rash/src/stdlib.rs` - Add env/env_var_or to is_stdlib_function()
- `rash/src/ir/shell_ir.rs` - Add EnvVar variant to ShellValue
- `rash/src/ir/mod.rs` - Add converter logic for env() calls
- `rash/src/emitter/mod.rs` - Add shell generation for ${VAR} syntax

### Need Creation (RED + GREEN phases)
- `rash/src/ir/tests.rs` - IR conversion tests (if not exists, add to mod.rs)
- `rash/src/emitter/tests.rs` - Emitter tests (if not exists, add to mod.rs)
- `rash/tests/integration/environment.rs` - Integration test

## Success Criteria

### RED Phase Complete
- [ ] 12 failing tests written
- [ ] All tests fail for correct reasons
- [ ] No false failures (implementation not present, not broken)

### GREEN Phase Complete
- [ ] All 813 existing tests pass
- [ ] All 12 new tests pass
- [ ] examples/environment-setup.rs transpiles successfully
- [ ] Generated shell passes shellcheck

### REFACTOR Phase Complete
- [ ] No clippy warnings
- [ ] Code formatted with cargo fmt
- [ ] Helper functions extracted
- [ ] Documentation added

### Sprint 27a Complete
- [ ] env() function works
- [ ] env_var_or() function works
- [ ] Security validated (no injection)
- [ ] ROADMAP.md updated
- [ ] Completion report written
- [ ] Changes committed and pushed

## References

- **Specification**: `/home/noahgift/src/bashrs/docs/specifications/SPRINT_27A.md`
- **Example Code**: `/home/noahgift/src/bashrs/examples/environment-setup.rs`
- **POSIX Spec**: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html#tag_18_06_02

---

**Next Session**: Write 10 remaining RED tests (IR, emitter, security, integration)
**Estimated Time**: 30-45 minutes for remaining RED tests
**Quality**: EXTREME TDD - test-first discipline maintained ✅
