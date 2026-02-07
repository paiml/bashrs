# Introduction

Welcome to **The Rash Book**! This guide will teach you how to write safe, deterministic, and idempotent shell scripts using Rash.

## What is Rash?

**Rash** (bashrs) is a shell safety and purification tool that goes beyond traditional linters like ShellCheck. While ShellCheck *detects* problems, Rash *transforms* your code to fix them automatically.

### Key Features

- **Shell Purification**: Automatically transform bash scripts to be deterministic and idempotent
- **Security Linting**: Detect and fix 8 critical security vulnerabilities (SEC001-SEC008)
- **Configuration Management**: Analyze and purify shell config files like .bashrc and .zshrc
- **Makefile Linting**: Security and best-practice linting for Makefiles
- **POSIX Compliance**: All generated shell code passes `shellcheck -s sh`

### How is Rash Different from ShellCheck?

| Feature | ShellCheck | Rash |
|---------|------------|------|
| **Mode** | Read-only (detect) | Read-write (transform) |
| **Non-determinism** | ⚠️ Warns about `$RANDOM` | ✅ Rewrites to deterministic IDs |
| **Idempotency** | ⚠️ Warns about `mkdir` | ✅ Transforms to `mkdir -p` |
| **Variable Quoting** | ⚠️ Suggests quoting | ✅ Automatically adds quotes |
| **PATH Duplicates** | ❌ Not detected | ✅ Detects and removes |
| **Config Files** | ❌ No support | ✅ Analyzes .bashrc, .zshrc |
| **Makefiles** | ❌ No support | ✅ Full linting support |

### Example: Before and After

**Before (messy, non-deterministic):**

```bash
#!/bin/bash
SESSION_ID=$RANDOM
mkdir /tmp/deploy-$SESSION_ID
cd /tmp/deploy-$SESSION_ID
```

**After (purified, deterministic):**

```bash
#!/bin/sh
# Purified by Rash v6.60.0
SESSION_ID="${VERSION:-1.0.0}"
mkdir -p "/tmp/deploy-${SESSION_ID}"
cd "/tmp/deploy-${SESSION_ID}" || exit 1
```

### Design Philosophy: Toyota Way

Rash follows Toyota Way principles:

- **自働化 (Jidoka)**: Build quality in from the start
- **現地現物 (Genchi Genbutsu)**: Test with real shells (dash, ash, busybox)
- **反省 (Hansei)**: Fix bugs before adding features
- **改善 (Kaizen)**: Continuous improvement through feedback

## Who Should Read This Book?

- DevOps engineers who write shell scripts
- Developers maintaining .bashrc/.zshrc files
- System administrators automating tasks
- Anyone who wants safer, more reliable shell scripts

## How to Use This Book

1. **Getting Started**: Install Rash and run your first purification
2. **Core Concepts**: Learn about determinism, idempotency, and POSIX compliance
3. **Linting**: Explore security, determinism, and idempotency rules
4. **Configuration Management**: Purify your shell config files
5. **Examples**: See real-world use cases
6. **Advanced Topics**: Deep dive into AST transformation and testing
7. **Reference**: Quick lookup for commands and rules

## Prerequisites

- Basic shell scripting knowledge
- Familiarity with command-line tools
- (Optional) Understanding of AST concepts for advanced topics

## Conventions

Throughout this book, we use the following conventions:

```bash
# Shell commands you can run
bashrs --version
```

```rust
// Rust code examples (for advanced topics)
fn main() {
    println!("Hello from Rash!");
}
```

> **Note**: Important information or tips

⚠️ **Warning**: Critical information to avoid common mistakes

✅ **Best Practice**: Recommended approaches

## Let's Get Started!

Ready to write safer shell scripts? Let's dive into [Installation](./getting-started/installation.md)!
