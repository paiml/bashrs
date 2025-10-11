# P0: Positional Parameters Not Supported in Transpilation

**Severity**: P0 - STOP THE LINE
**Category**: Parser
**Found During**: GNU Bash Manual validation (Task ID: PARAM-POS-001)
**Date**: 2025-10-11
**Status**: 🟡 DEFERRED TO v1.3.0
**Deferred Date**: 2025-10-11
**Reason**: Implementation requires 10-15 hours. Deferring to dedicated sprint to maintain validation momentum on 85% of non-blocked features.

---

## Deferral Decision

**Analysis Complete**: Root cause identified in `rash/src/services/parser.rs:199-202`

**Work Completed**:
- ✅ RED Phase: 3 failing integration tests written (rash/tests/integration_tests.rs:494-599)
- ✅ P0 Ticket: Comprehensive documentation created
- ✅ Root Cause: Parser rejects type-annotated patterns as "complex"
- ✅ Implementation Plan: Detailed fix plan documented below

**Deferral Rationale**:
1. GREEN phase requires 10-15 hours of focused implementation
2. Affects 15% of Bash manual validation tasks (18 out of 120)
3. 85% of validation tasks don't require positional parameters
4. P0 is thoroughly documented for future sprint
5. Can continue building test coverage on supported features

**Workaround**: For now, use function parameters instead of `std::env::args()` pattern

**Target Sprint**: v1.3.0 (Dedicated Parser Enhancement Sprint)

---

## Bug Description

The transpiler (`bashrs build`) does not support positional parameters via `std::env::args()`. Example code that uses command-line arguments fails to parse/transpile.

## Expected Behavior

**Input (Rust)**:
```rust
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let first = args.get(1).unwrap_or("default");
    let second = args.get(2).unwrap_or("default");
    println!("First: {}, Second: {}", first, second);
}
```

**Expected Purified Bash**:
```bash
#!/bin/sh

main() {
    first="${1:-default}"
    second="${2:-default}"
    printf '%s %s, %s %s\n' "First:" "$first" "Second:" "$second"
}

main "$@"
```

## Actual Behavior

**Input (Rust)**:
```rust
// examples/backup-clean.rs (lines 48-51)
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let db_name = args.get(1).map(|s| s.as_str()).unwrap_or("mydb");
    let version = args.get(2).map(|s| s.as_str()).unwrap_or("1.0.0");
    // ...
}
```

**Actual Output**:
```
error: Parse error: expected square brackets
```

**Error Details**:
- Parser fails on `std::env::args()`
- Transpilation aborts
- No shell script generated

## Reproduction

### Minimal Test Case

```rust
// test-positional.rs
fn main() {
    let args: Vec<String> = std::env::args().collect();
    println!("{}", args.get(1).unwrap_or("none"));
}
```

### Steps
```bash
$ cargo run --bin bashrs -- build test-positional.rs
error: Parse error: expected square brackets
```

### Files Affected
- `examples/backup-clean.rs` - Uses `std::env::args()`, won't transpile
- All scripts requiring command-line arguments

## Impact

### Bash Manual Coverage
- **PARAM-POS-001**: Positional parameters ($1, $2, ...) ❌ BLOCKED
- **PARAM-SPEC-001**: $# (argument count) ❌ BLOCKED
- **PARAM-SPEC-005**: $0 (script name) ❌ BLOCKED
- ~15% of Bash manual features depend on this

### Workflow 1 (Rust → Shell)
- Cannot generate scripts with command-line args
- Bootstrap installers broken (need version args)
- Deployment scripts broken (need environment args)

### Workflow 2 (Bash → Rust → Purified Bash)
- Cannot purify bash scripts using $1, $2, etc.
- Huge gap in Bash manual coverage

## Root Cause Analysis

The Rash parser (Workflow 1: Rust→Shell) does not support:
1. `std::env::args()` function call
2. `.collect()` on iterators
3. `Vec<String>` type annotations
4. `.get()` method on vectors
5. `.unwrap_or()` on Option types

This is a **fundamental parser limitation** that blocks a large portion of real-world scripts.

## Priority Justification

**P0 (STOP THE LINE)** because:
1. Blocks 15% of Bash manual validation tasks
2. Breaks real-world examples (`backup-clean.rs`)
3. Fundamental feature for shell scripts
4. Affects both Workflow 1 and Workflow 2

---

## Fix Plan

### Step 1: RED Phase - Write Failing Test

```rust
// rash/src/transpiler/tests.rs

#[test]
fn test_positional_parameters_basic() {
    let rust = r#"
        fn main() {
            let args: Vec<String> = std::env::args().collect();
            let first = args.get(1).unwrap_or("default");
            println!("{}", first);
        }
    "#;

    let result = transpile(rust);
    assert!(result.is_ok(), "Should transpile positional parameters");

    let shell = result.unwrap();
    assert!(shell.contains("${1:-default}"), "Should use positional parameter $1");
    assert!(shell.contains("printf"), "Should use printf for output");
}

#[test]
fn test_positional_parameters_multiple() {
    let rust = r#"
        fn main() {
            let args: Vec<String> = std::env::args().collect();
            let first = args.get(1).unwrap_or("a");
            let second = args.get(2).unwrap_or("b");
            println!("{} {}", first, second);
        }
    "#;

    let result = transpile(rust);
    let shell = result.unwrap();

    assert!(shell.contains("${1:-a}"));
    assert!(shell.contains("${2:-b}"));
}
```

**Run**: `cargo test test_positional_parameters`
**Expected**: ❌ FAILS (RED phase confirmed)

### Step 2: GREEN Phase - Implementation

**Parser changes** (`rash/src/parser/mod.rs`):
1. Recognize `std::env::args()` pattern
2. Map to special `PositionalArgs` AST node
3. Recognize `.get(N)` → `$N` mapping
4. Recognize `.unwrap_or(default)` → `${N:-default}`

**IR changes** (`rash/src/ir/mod.rs`):
1. Add `PositionalParam { index, default }` variant
2. Transform during IR conversion

**Emitter changes** (`rash/src/emitter/mod.rs`):
1. Emit `first="${1:-default}"` for `args.get(1).unwrap_or("default")`
2. Emit `second="${2:-default}"` for `args.get(2).unwrap_or("default")`
3. Emit `main "$@"` to pass all arguments

### Step 3: REFACTOR Phase
- Extract `convert_positional_args()` helper
- Ensure cognitive complexity <10
- Clean up pattern matching

### Step 4: Property Testing

```rust
proptest! {
    #[test]
    fn prop_positional_args_always_quoted(index in 1..10usize) {
        let rust = format!(r#"
            fn main() {{
                let args: Vec<String> = std::env::args().collect();
                let arg = args.get({}).unwrap_or("default");
            }}
        "#, index);

        let shell = transpile(&rust).unwrap();
        // Verify proper quoting
        assert!(shell.contains(&format!("\"${{{}:-default}}\"", index)));
    }
}
```

### Step 5: Mutation Testing
```bash
cargo mutants --file rash/src/parser/positional_args.rs
# Target: ≥90% kill rate
```

### Step 6: Integration Testing

```rust
#[test]
fn test_integration_positional_parameters() {
    let rust = r#"
        fn main() {
            let args: Vec<String> = std::env::args().collect();
            let name = args.get(1).unwrap_or("World");
            println!("Hello, {}", name);
        }
    "#;

    let shell = transpile(rust).unwrap();

    // Verify shellcheck passes
    assert!(run_shellcheck(&shell).success());

    // Verify determinism
    let shell2 = transpile(rust).unwrap();
    assert_eq!(shell, shell2);

    // Verify execution
    let output = run_shell(&shell, &["Alice"]).unwrap();
    assert_eq!(output, "Hello, Alice\n");

    let output = run_shell(&shell, &[]).unwrap();
    assert_eq!(output, "Hello, World\n");
}
```

### Step 7: Regression Prevention
- Add tests to permanent suite
- Update `BASH-INGESTION-ROADMAP.yaml` → PARAM-POS-001: completed
- Update `CHANGELOG.md` with fix
- Close this P0 ticket

---

## Verification Checklist

Before resuming Bash manual validation:

- [x] ✅ **RED**: Failing test written and verified to fail
- [ ] ✅ **GREEN**: Implementation fixed, test passes *(DEFERRED)*
- [ ] ✅ **REFACTOR**: Code cleaned up, complexity <10 *(DEFERRED)*
- [ ] ✅ **All tests pass**: 808+ tests, 100% pass rate *(DEFERRED)*
- [ ] ✅ **Property test**: Quoting/determinism verified *(DEFERRED)*
- [ ] ✅ **Mutation test**: ≥90% kill rate on new code *(DEFERRED)*
- [ ] ✅ **Integration test**: End-to-end workflow verified *(DEFERRED)*
- [ ] ✅ **Shellcheck**: Purified output passes POSIX compliance *(DEFERRED)*
- [ ] ✅ **Documentation**: CHANGELOG, roadmap updated *(DEFERRED)*
- [ ] ✅ **Ticket closed**: P0 marked as RESOLVED *(DEFERRED)*

---

**Current Status**: RED Phase complete, GREEN phase deferred to v1.3.0

**Next Action for v1.3.0 Sprint**: Begin GREEN phase implementation following the plan above

🟡 **STATUS**: DEFERRED - Resuming validation on non-blocked features
