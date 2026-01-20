# DSL Built-in Functions

This reference documents the built-in functions available in the bashrs Rust DSL for transpiling to shell scripts.

## Overview

When writing `.rs` files for bashrs transpilation, you can use these built-in functions without declaring them. bashrs recognizes these as DSL primitives and emits the appropriate shell code.

## Built-in Functions

### `echo(msg: &str)`

Prints a message to stdout with a trailing newline.

```rust,ignore
#[bashrs::main]
fn main() {
    echo("Hello, world!");
}
```

Transpiles to:

```sh
echo 'Hello, world!'
```

### `exec(cmd: &str)`

Executes a shell command string. This is the primary way to run arbitrary shell commands, including those with pipes, redirections, and logical operators.

```rust,ignore
#[bashrs::main]
fn main() {
    // Simple command
    exec("ls -la");

    // Commands with pipes
    exec("cat file.txt | grep pattern | head -10");

    // Commands with logical operators
    exec("mkdir -p /tmp/foo && cd /tmp/foo");

    // Commands with redirections
    exec("command 2>&1 | tee output.log");
}
```

Transpiles to:

```sh
eval 'ls -la'
eval 'cat file.txt | grep pattern | head -10'
eval 'mkdir -p /tmp/foo && cd /tmp/foo'
eval 'command 2>&1 | tee output.log'
```

> **Note (v6.56.2+):** The `exec()` function uses `eval` internally to properly handle shell operators like `|`, `&&`, `||`, and `;`. This was fixed in [Issue #95](https://github.com/paiml/bashrs/issues/95).

#### Why `eval`?

Shell operators like pipes and logical operators are interpreted by the shell, not by individual commands. When you pass a string like `"cmd1 | cmd2"` to a function, the shell sees it as a single argument. Using `eval` causes the shell to re-interpret the string, properly parsing the operators.

#### Security Considerations

The `exec()` function still validates against:
- **Shellshock attacks** (`() { :; }` patterns)
- **Command substitution** (`$(...)` and backticks)

These protections remain active even when shell operators are allowed.

## Example: Complete Script

```rust,ignore
//! Performance benchmark script
//!
//! Usage:
//! ```bash
//! bashrs build benchmark.rs -o benchmark.sh
//! ./benchmark.sh
//! ```

#[bashrs::main]
fn main() {
    print_header();
    run_benchmarks();
}

fn print_header() {
    echo("=================================");
    echo("  Performance Benchmark Suite    ");
    echo("=================================");
    echo("");
}

fn run_benchmarks() {
    echo("Checking system info...");
    exec("uname -a");

    echo("Checking CPU cores...");
    exec("nproc 2>/dev/null || sysctl -n hw.ncpu");

    echo("Running benchmark...");
    exec("time cargo build --release 2>&1 | tail -5");
}
```

## Version History

| Version | Change |
|---------|--------|
| 6.56.2  | Fixed `exec()` to use `eval` for proper shell operator handling |
| 6.56.1  | Added context-aware validation to allow shell operators in `exec()` |
| 6.56.0  | Initial DSL support |
