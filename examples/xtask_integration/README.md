# xtask Integration Example

This example demonstrates how to integrate bashrs into your Cargo workspace without requiring global installation. It follows the **xtask pattern** for project-specific build automation.

## Overview

The xtask pattern provides three integration methods:

1. **build.rs**: Automatic transpilation during `cargo build`
2. **xtask commands**: Manual transpilation via `cargo xtask transpile`
3. **Programmatic API**: Full control in custom build scripts

All methods use bashrs as a library dependency, eliminating the need for `cargo install bashrs`.

## Project Structure

```
my-project/
├── Cargo.toml              # Workspace configuration
├── build.rs                # Auto-transpilation (Option 1)
├── hooks/                  # Git hooks written in Rust
│   ├── pre-commit.rs
│   └── pre-push.rs
├── xtask/                  # Project automation tasks
│   ├── Cargo.toml
│   └── src/
│       └── main.rs         # xtask commands (Option 2)
└── examples/
    └── custom_build.rs     # Programmatic API (Option 3)
```

## Method 1: build.rs Integration (Recommended)

**Best for**: Automatic transpilation with zero manual steps.

Add bashrs to your main `Cargo.toml`:

```toml
[build-dependencies]
bashrs = "7.1"
```

Create `build.rs` at your project root:

```rust
// build.rs
use bashrs::build_rs::auto_transpile;

fn main() {
    println!("cargo:rerun-if-changed=hooks");

    // Automatically transpile all hooks/*.rs to .git/hooks/*
    auto_transpile("hooks", ".git/hooks", 0o755)
        .expect("Failed to transpile git hooks");
}
```

Now `cargo build` automatically transpiles your hooks!

```bash
$ cargo build
   Compiling my-project v0.1.0
   cargo:rerun-if-changed=hooks/pre-commit.rs
   cargo:rerun-if-changed=hooks/pre-push.rs
   Finished dev [unoptimized + debuginfo] target(s) in 2.3s

$ ls -la .git/hooks/
-rwxr-xr-x  1 user  staff  pre-commit
-rwxr-xr-x  1 user  staff  pre-push
```

## Method 2: xtask Commands

**Best for**: Manual control and CI/CD pipelines.

### Setup

Create `xtask/Cargo.toml`:

```toml
[package]
name = "xtask"
version = "0.1.0"
edition = "2021"

[dependencies]
bashrs = "7.1"
```

Create `xtask/src/main.rs`:

```rust
use bashrs::Transpiler;
use std::env;

fn main() {
    let task = env::args().nth(1);
    match task.as_deref() {
        Some("transpile") => transpile_hooks().expect("Failed to transpile"),
        _ => print_help(),
    }
}

fn transpile_hooks() -> bashrs::Result<()> {
    println!("Transpiling git hooks...");

    Transpiler::new()
        .input("hooks/pre-commit.rs")
        .output(".git/hooks/pre-commit")
        .permissions(0o755)
        .transpile()?;

    Transpiler::new()
        .input("hooks/pre-push.rs")
        .output(".git/hooks/pre-push")
        .permissions(0o755)
        .transpile()?;

    println!("✓ Hooks transpiled successfully!");
    Ok(())
}

fn print_help() {
    eprintln!("Usage: cargo xtask <TASK>");
    eprintln!();
    eprintln!("Tasks:");
    eprintln!("  transpile    Transpile git hooks to shell scripts");
}
```

### Usage

Add to root `Cargo.toml`:

```toml
[workspace]
members = [".", "xtask"]
```

Create `.cargo/config.toml`:

```toml
[alias]
xtask = "run --package xtask --"
```

Now run:

```bash
$ cargo xtask transpile
Transpiling git hooks...
✓ Hooks transpiled successfully!
```

## Method 3: Programmatic API

**Best for**: Complex build workflows with custom logic.

```rust
// examples/custom_build.rs
use bashrs::{Transpiler, Config};
use bashrs::models::{ShellDialect, VerificationLevel};

fn main() -> bashrs::Result<()> {
    let config = Config {
        target: ShellDialect::Posix,
        verify: VerificationLevel::Strict,
        optimize: true,
        ..Default::default()
    };

    // Transpile with custom configuration
    Transpiler::new()
        .input("hooks/pre-commit.rs")
        .output(".git/hooks/pre-commit")
        .permissions(0o755)
        .config(config.clone())
        .transpile()?;

    // Batch process multiple files
    let hooks = vec![
        ("hooks/pre-commit.rs", ".git/hooks/pre-commit"),
        ("hooks/pre-push.rs", ".git/hooks/pre-push"),
        ("hooks/post-merge.rs", ".git/hooks/post-merge"),
    ];

    for (input, output) in hooks {
        println!("Transpiling {} -> {}", input, output);
        Transpiler::new()
            .input(input)
            .output(output)
            .permissions(0o755)
            .config(config.clone())
            .transpile()?;
    }

    println!("✓ All hooks transpiled!");
    Ok(())
}
```

## Comparison

| Feature | build.rs | xtask | Programmatic |
|---------|----------|-------|--------------|
| Setup complexity | Simple | Medium | Complex |
| Automatic | Yes | No | No |
| Control | Low | Medium | Full |
| CI/CD friendly | Yes | Yes | Yes |
| Custom logic | Limited | Good | Excellent |

## Real-World Example

Projects using this pattern can maintain git hooks in Rust:

```rust
// build.rs
use bashrs::build_rs::auto_transpile;

fn main() {
    auto_transpile("hooks", ".git/hooks", 0o755)
        .expect("Failed to transpile hooks");
}
```

Contributors simply run `cargo build` - hooks are automatically transpiled and installed.

## Benefits

✅ **No global installation**: bashrs is a workspace dependency
✅ **Version locked**: `Cargo.lock` ensures consistent behavior
✅ **Zero setup**: New contributors just run `cargo build`
✅ **Type-safe**: Write hooks in Rust with full tooling support
✅ **Testable**: Unit test your hooks before transpilation
✅ **CI/CD ready**: Works seamlessly in automated builds

## Migration from CLI

**Before** (manual CLI):
```bash
# Every developer must run
$ cargo install bashrs
$ bashrs build hooks/pre-commit.rs .git/hooks/pre-commit
$ chmod +x .git/hooks/pre-commit
```

**After** (automatic xtask):
```bash
# Just works™
$ cargo build
```

## Advanced Usage

### Conditional Transpilation

```rust
// build.rs
use bashrs::build_rs::auto_transpile;

fn main() {
    // Only transpile in dev mode
    if cfg!(debug_assertions) {
        auto_transpile("hooks", ".git/hooks", 0o755)
            .expect("Failed to transpile");
    }
}
```

### Custom Output Naming

```rust
use bashrs::Transpiler;

fn main() -> bashrs::Result<()> {
    // Transpile install.rs -> install.sh (not install)
    Transpiler::new()
        .input("src/install.rs")
        .output("dist/install.sh")
        .permissions(0o755)
        .transpile()?;
    Ok(())
}
```

### Watch Mode (Development)

For development, combine with [cargo-watch](https://github.com/watchexec/cargo-watch):

```bash
$ cargo watch -x "xtask transpile"
```

This automatically retranspiles hooks when source files change.

## Troubleshooting

### Issue: "Input path not set"

**Cause**: Missing `.input()` call.

**Fix**:
```rust
Transpiler::new()
    .input("hooks/pre-commit.rs")  // ← Add this
    .output(".git/hooks/pre-commit")
    .transpile()
```

### Issue: Permissions not set

**Cause**: Platform is not Unix.

**Note**: `permissions()` only works on Unix. On Windows, scripts won't be executable.

### Issue: Hooks not found after build

**Cause**: Output path incorrect.

**Fix**: Verify `.git/hooks/` directory exists:
```rust
std::fs::create_dir_all(".git/hooks")?;
```

## See Also

- [Cargo xtask pattern](https://github.com/matklad/cargo-xtask)
- [bashrs documentation](https://docs.rs/bashrs)
- [Git hooks in Rust](https://git-scm.com/book/en/v2/Customizing-Git-Git-Hooks)
