# Known Limitations - Rash v1.0

This document outlines the current limitations of Rash v1.0 and provides guidance on workarounds, future plans, and best practices.

## Language Feature Limitations

### Not Yet Supported

The following Rust language features are **not currently supported** in Rash v1.0:

#### 1. Loop Constructs

**For Loops**:
```rust
// ❌ NOT SUPPORTED
for i in 0..10 {
    echo("Iteration: {i}");
}

// ✅ WORKAROUND: Use shell-native loops with exec()
exec("for i in $(seq 0 9); do echo \"Iteration: $i\"; done");
```

**Status**: Deferred to v1.1 (parser work required)

**While Loops**:
```rust
// ❌ NOT SUPPORTED
let mut count = 0;
while count < 10 {
    count = count + 1;
}

// ✅ WORKAROUND: Use shell-native while
exec("count=0; while [ $count -lt 10 ]; do count=$((count + 1)); done");
```

**Status**: Deferred to v1.1 (semantic model needed for mutable variables)

#### 2. Pattern Matching

```rust
// ❌ NOT SUPPORTED
match os {
    "linux" => echo("Linux detected"),
    "darwin" => echo("macOS detected"),
    _ => echo("Unknown OS"),
}

// ✅ WORKAROUND: Use if/else chains
if os == "linux" {
    echo("Linux detected");
} else if os == "darwin" {
    echo("macOS detected");
} else {
    echo("Unknown OS");
}
```

**Status**: Deferred to v1.1 (pattern matching infrastructure required)

#### 3. Collections and Arrays

```rust
// ❌ NOT SUPPORTED
let names = vec!["Alice", "Bob", "Charlie"];
for name in names {
    echo("Hello, {name}");
}

// ✅ WORKAROUND: Use shell arrays with exec()
exec(r#"
    names="Alice Bob Charlie"
    for name in $names; do
        echo "Hello, $name"
    done
"#);
```

**Status**: Deferred to v1.2+ (requires type system extension)

#### 4. Mutable Variables

```rust
// ❌ NOT SUPPORTED
let mut counter = 0;
counter += 1;

// ✅ WORKAROUND: Reassign with let
let counter = 0;
let counter = counter + 1;  // This works - creates new binding
```

**Status**: Limited support - reassignment works, but `mut` keyword not supported

#### 5. Closures and Higher-Order Functions

```rust
// ❌ NOT SUPPORTED
let mapper = |x| x * 2;
let result = map(numbers, mapper);

// ✅ WORKAROUND: Use regular functions
fn double(x: i32) -> i32 {
    x * 2
}
```

**Status**: Deferred to v1.2+ (significant semantic complexity)

### Partially Supported

#### 1. String Interpolation

```rust
// ✅ SUPPORTED: Basic variable interpolation
let name = "Alice";
echo("Hello, {name}");

// ❌ NOT SUPPORTED: Expression interpolation
echo("Sum: {1 + 2}");  // Error: expressions in strings not supported

// ✅ WORKAROUND: Compute first, then interpolate
let sum = 1 + 2;
echo("Sum: {sum}");
```

#### 2. Arithmetic Expressions

```rust
// ✅ SUPPORTED: Basic arithmetic
let x = 1 + 2;
let y = 10 - 3;
let z = 4 * 5;
let w = 20 / 4;

// ⚠️ LIMITED: Modulo operator
let r = 10 % 3;  // Works, generates $((10 % 3))

// ❌ NOT SUPPORTED: Bitwise operations
let b = 5 & 3;  // Error: bitwise ops not supported
```

#### 3. Function Signatures

```rust
// ✅ SUPPORTED: Simple parameter types
fn add(a: i32, b: i32) -> i32 {
    a + b
}

// ❌ NOT SUPPORTED: Generic parameters
fn generic<T>(value: T) -> T {  // Error: generics not supported
    value
}

// ❌ NOT SUPPORTED: Option/Result types
fn maybe(x: i32) -> Option<i32> {  // Error: Option not supported
    Some(x)
}
```

## Beta Features (Experimental)

The following features are available in v1.0 but marked as **experimental**:

### 1. Binary Compilation

**`bashrs compile` Command**:

```bash
# ✅ Self-extracting scripts (tested, production-ready)
bashrs compile install.rs -o install --self-extracting

# ⚠️ Container packaging (experimental, incomplete)
bashrs compile app.rs -o app --container --format oci
```

**Limitations**:
- Container formats (OCI, Docker) are partially implemented
- Binary optimization is stub implementation
- Limited runtime selection (dash, bash, busybox only)
- No cross-compilation support yet

**Recommendation**: Use `bashrs build` for production. Use `compile --self-extracting` only for quick testing or single-file installers.

### 2. Proof Generation

**`--emit-proof` Flag**:

```bash
bashrs build input.rs -o output.sh --emit-proof
# Generates output.proof with formal verification metadata
```

**Limitations**:
- Proof format is experimental and may change
- Limited verification properties included
- No SMT solver integration yet
- Proof validation tools not included

**Status**: Format subject to change in v1.1+

## Shell Compatibility Limitations

### Target Shells

**Fully Supported** ✅:
- POSIX sh
- dash (0.5.11+)
- bash (3.2+)
- ash/BusyBox (1.30+)

**Partially Supported** ⚠️:
- zsh (5.0+) - works but not actively tested
- mksh (R59+) - works but not actively tested

**Not Supported** ❌:
- fish - Different syntax model
- PowerShell - Different platform
- nushell - Different architecture

### Shell-Specific Features

Rash targets **POSIX compliance** by default, which means:

```rust
// ✅ WORKS: POSIX-compliant
let result = capture("command -v git");

// ❌ FAILS: bash-specific
let result = capture("type -p git");  // type -p is bash-only
```

**Workaround**: Use `--target bash` for bash-specific scripts:

```bash
bashrs build script.rs -o script.sh --target bash
```

## Performance Limitations

### Transpilation Speed

**Current Performance**:
- ~21µs for simple scripts (excellent)
- ~500µs for medium scripts (good)
- ~2ms for large scripts (acceptable)

**Known Bottlenecks**:
- Complex nested control flow
- Large number of user-defined functions
- Extensive validation passes

**Planned Improvements** (v1.1+):
- Incremental compilation
- Cached validation results
- Parallel IR generation

### Generated Code Size

**Typical Overhead**:
- ~20 lines of runtime boilerplate
- ~5-10 lines per stdlib function used
- ~2-3 lines per user function

**Large Script Example**:
- Input: 100 lines of Rust
- Output: ~250 lines of shell script
- Ratio: ~2.5x expansion

**Optimization Options**:

```bash
# Minimize generated code size
bashrs build input.rs -o output.sh --optimize --target posix

# Readable output (default)
bashrs build input.rs -o output.sh
```

## CLI Limitations

### 1. Interactive Features

**Not Available in v1.0**:
- ❌ Playground/REPL (removed, planned for v1.1)
- ❌ Watch mode for file changes
- ❌ Language server protocol (LSP)

**Workarounds**:
```bash
# Instead of watch mode, use a simple loop:
while true; do bashrs build input.rs -o output.sh && sh output.sh; sleep 1; done

# For LSP, use standard Rust tools (rust-analyzer) on source files
```

### 2. Project Management

**Limited Features**:
- `bashrs init` creates basic project structure
- No dependency management
- No package registry integration
- No build profiles

**Workaround**: Use standard Cargo for Rust dependencies:

```toml
# Cargo.toml
[dependencies]
# Regular Rust crates work for build-time code
serde = "1.0"
```

## Safety and Security Limitations

### What Rash Protects Against ✅

- **Command Injection**: All variables properly quoted
- **Path Traversal**: Path validation and escaping
- **Glob Expansion**: Automatic quoting
- **Word Splitting**: Safe IFS handling
- **Undefined Variables**: `set -u` enforcement

### What Rash CANNOT Protect Against ❌

#### 1. Unsafe exec() Calls

```rust
// ⚠️ DANGEROUS: User input in exec() is NOT automatically escaped
let user_input = env("UNTRUSTED");
exec("rm -rf {user_input}");  // BAD: exec() bypasses safety

// ✅ SAFE: Use stdlib functions instead
let user_input = env("UNTRUSTED");
fs_remove(&user_input);  // GOOD: Proper escaping
```

**Recommendation**: Avoid `exec()` with user input. Use stdlib functions when possible.

#### 2. Time-of-Check-Time-of-Use (TOCTOU) Races

```rust
// ⚠️ RACE CONDITION POSSIBLE
if path_exists("/tmp/lockfile") {
    // File could be deleted here
    let content = read_file("/tmp/lockfile");  // Might fail
}
```

**Recommendation**: Use atomic operations where possible. Rash inherits shell's TOCTOU limitations.

#### 3. Resource Exhaustion

```rust
// ⚠️ NO PROTECTION: Can cause resource exhaustion
exec("cat /dev/zero");  // Infinite output
exec(":(){ :|:& };:");   // Fork bomb
```

**Recommendation**: Implement your own resource limits and validation.

## Testing Limitations

### What's Tested

- ✅ 683 unit and integration tests
- ✅ 114K property test executions
- ✅ Multi-shell compatibility (sh, dash, bash, ash)
- ✅ ShellCheck validation (24/24 passing)
- ✅ 83.07% total code coverage
- ✅ 88.74% core transpiler coverage

### What's NOT Fully Tested

- ⚠️ Binary compilation (32% coverage - experimental)
- ⚠️ Container packaging (stub implementation)
- ⚠️ Binary entry points (0% coverage - process-level testing needed)
- ⚠️ Rare error paths (~300 lines uncovered)

## Documentation Limitations

### Available Documentation

- ✅ README.md with quick start
- ✅ STDLIB.md with function reference
- ✅ ERROR_GUIDE.md for troubleshooting
- ✅ CHANGELOG.md with release notes
- ✅ API documentation (docs.rs)

### Missing Documentation (v1.0)

- ❌ Comprehensive tutorial series
- ❌ Best practices guide
- ❌ Architecture deep-dive
- ❌ Video tutorials
- ❌ Interactive examples

**Planned for v1.1+**

## Platform Limitations

### Supported Platforms

- ✅ Linux (x86_64, aarch64)
- ✅ macOS (x86_64, Apple Silicon)
- ⚠️ Windows (via WSL only)

### Not Supported

- ❌ Windows native (PowerShell target planned for v1.2+)
- ❌ BSD variants (untested, may work)
- ❌ Embedded systems (busybox works, but untested on actual embedded)

## Migration and Breaking Changes

### Breaking Changes from v0.9.x → v1.0

**Removed Features**:
- `bashrs playground` command removed
  - **Migration**: Use `bashrs build` for now, or wait for v1.1 `rash-playground` crate

**Changed Behavior**:
- None - core transpilation API is stable

**Deprecated Features**:
- None currently

### Future Breaking Changes (Planned)

**v1.1 (Minor Release)**:
- May change proof generation format
- May adjust container compilation API
- CLI flags may be reorganized

**v2.0 (Major Release - Future)**:
- Possible changes to stdlib function signatures
- May require shell runtime v2.0
- Possible IR format changes

## Workarounds and Best Practices

### When to Use Rash

**Good Use Cases** ✅:
- Bootstrap installers for software
- System configuration scripts
- CI/CD deployment automation
- Simple CLI tools with limited logic
- POSIX-compliant shell script generation

**Poor Use Cases** ❌:
- Complex business logic with loops/conditionals
- Data processing with collections
- Scripts requiring mutable state
- Real-time interactive applications
- Scripts with heavy string processing

### Hybrid Approach

For complex scripts, consider a **hybrid approach**:

```rust
// Use Rash for the shell interface
#[rash::main]
fn main() {
    // Simple CLI logic in Rash
    let version = env_var_or("VERSION", "1.0.0");
    echo("Installing version {version}");

    // Delegate complex logic to native binaries
    exec("python3 complex_logic.py");
    exec("./rust-binary --process-data");
}
```

This leverages Rash's safety for shell integration while using appropriate languages for complex logic.

## Reporting Issues

If you encounter limitations not documented here:

1. **Check the issue tracker**: https://github.com/paiml/rash/issues
2. **Search existing discussions**: https://github.com/paiml/rash/discussions
3. **File a new issue** with:
   - Rust code that doesn't work
   - Expected behavior
   - Actual error message
   - Rash version (`bashrs --version`)

## Roadmap for Addressing Limitations

### v1.1 (Q1 2025)

- [ ] For loops support
- [ ] Match expressions
- [ ] Playground/REPL (separate crate)
- [ ] Improved error diagnostics
- [ ] Tutorial series

### v1.2 (Q2 2025)

- [ ] While loops
- [ ] Collections support
- [ ] LSP implementation
- [ ] PowerShell target (experimental)

### v2.0 (2025+)

- [ ] Full standard library
- [ ] Advanced optimizations
- [ ] SMT-based verification
- [ ] Incremental compilation

See [README.md](README.md#roadmap) for the complete roadmap.

---

**Last Updated**: 2025-10-04 (v1.0.0 release)
**Next Review**: v1.1.0 release
