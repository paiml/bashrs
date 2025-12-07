<div align="center">

# Rash - Bidirectional Shell Safety Tool

[![Crates.io](https://img.shields.io/crates/v/bashrs.svg)](https://crates.io/crates/bashrs)
[![Documentation](https://docs.rs/bashrs/badge.svg)](https://docs.rs/bashrs)
[![Book](https://img.shields.io/badge/book-The%20Rash%20Book-blue)](https://paiml.github.io/bashrs/)
[![License](https://img.shields.io/crates/l/bashrs.svg)](LICENSE)
[![CI](https://github.com/paiml/bashrs/workflows/CI/badge.svg)](https://github.com/paiml/bashrs/actions)
[![Tests](https://img.shields.io/badge/tests-6583%20passing-brightgreen)](https://github.com/paiml/bashrs/actions)
[![Coverage](https://img.shields.io/badge/coverage-88.71%25-green)](https://github.com/paiml/bashrs/actions)

**Rash** (v6.34.0) is a bidirectional shell safety tool that purifies legacy bash scripts and lets you write shell scripts in REAL Rust with automatic safety guarantees.

</div>

## Table of Contents

- [What's New](#-whats-new-in-v6340)
- [Why Rash?](#why-rash)
- [Quick Start](#quick-start)
- [Features](#features)
- [Core Commands](#core-commands)
- [Documentation](#-documentation)
- [Quality Metrics](#quality-metrics-v6360)
- [Shell Compatibility](#shell-compatibility)
- [Performance](#performance)
- [MCP Server](#mcp-server)
- [Contributing](#contributing)
- [License](#license)
- [Acknowledgments](#acknowledgments)

## 🚀 What's New in v6.34.0+

**Latest Updates** - 2025-11-12

- **Issue #21 FIXED**: SC2171 false positive with JSON brackets in heredocs (now correctly handles heredoc contexts)
- **Issue #22 FIXED**: SC2247 false positive with math operations in awk/bc (context-aware math detection)
- **Test Suite**: 6,583 tests, 100% pass rate, zero regressions
- **Quality**: All fixes implemented using EXTREME TDD (unit tests, property tests, mutation tests, integration tests)

**v6.34.0 Feature Completions** - Released 2025-11-12

- **Issue #2 RESOLVED**: Makefile multi-line format preservation with `--preserve-formatting` and `--skip-consolidation` flags
- **Issue #4 RESOLVED**: Complete bash parser - all 9 phases including redirection operators, heredocs, pipelines, special variables
- **Dockerfile Purification**: 6 comprehensive transformations (DOCKER001-006) for production-ready Docker images
- **Dogfooding Complete**: Fixed all P0 errors in bashrs's own infrastructure (0 errors found by self-analysis)

See [CHANGELOG.md](CHANGELOG.md) for complete release notes.

## Why Rash?

## Features

- 🛡️ **Automatic Safety**: Protection against shell injection, word splitting, glob expansion
- 🔍 **Beyond Linting**: Full AST semantic understanding - **transforms** code, doesn't just warn
- 📦 **Zero Runtime Dependencies**: Generated scripts work on any POSIX shell
- 🎯 **Deterministic Output**: Same input always produces identical scripts
- ✅ **ShellCheck Compliant**: All output passes strict linting

### How Rash Exceeds ShellCheck

| What ShellCheck Does | What Rash Does |
|---------------------|----------------|
| ⚠️ **Warns**: "$RANDOM is non-deterministic" | ✅ **Rewrites** to version-based deterministic IDs |
| ⚠️ **Warns**: "mkdir may fail if exists" | ✅ **Transforms** to `mkdir -p` (idempotent) |
| ⚠️ **Warns**: "Unquoted variable expansion" | ✅ **Quotes** all variables automatically |
| Static pattern matching | **Full AST semantic understanding** |
| Detects issues (read-only) | **Fixes issues (read-write transformation)** |

**Key Difference**: ShellCheck tells you what's wrong. Rash **understands your code's intent** and rewrites it to be safe, deterministic, and idempotent — automatically.

## Quick Start

### Installation

```bash
# From crates.io (recommended)
cargo install bashrs

# Or from source
git clone https://github.com/paiml/bashrs
cd bashrs
cargo install --path rash
```

### Write Rust, Get Safe Shell

```rust
// install.rs
#[rash::main]
fn main() {
    let version = env_var_or("VERSION", "1.0.0");
    let prefix = env_var_or("PREFIX", "/usr/local");

    echo("Installing MyApp {version} to {prefix}");

    mkdir_p("{prefix}/bin");
    mkdir_p("{prefix}/share/myapp");

    if exec("cp myapp {prefix}/bin/") {
        echo("✓ Binary installed");
    } else {
        eprint("✗ Failed to install binary");
        exit(1);
    }
}
```

**Transpile to safe POSIX shell**:

```bash
$ bashrs build install.rs -o install.sh
```

### Or Purify Existing Bash

**Before** (messy bash):
```bash
#!/bin/bash
SESSION_ID=$RANDOM                      # Non-deterministic
mkdir /app/releases/$RELEASE            # Non-idempotent
rm /app/current                         # Fails if doesn't exist
```

**After** (purified by Rash):
```bash
#!/bin/sh
session_id="session-${version}"         # ✅ Deterministic
mkdir -p "/app/releases/${release}"     # ✅ Idempotent
rm -f "/app/current"                    # ✅ Safe removal
```

## Core Commands

```bash
# Transpile Rust to shell
bashrs build input.rs -o output.sh

# Purify legacy bash scripts
bashrs purify messy.sh -o clean.sh

# Interactive REPL with debugging
bashrs repl

# Lint shell scripts
bashrs lint script.sh

# Test bash scripts
bashrs test script.sh

# Quality scoring
bashrs score script.sh

# Comprehensive audit
bashrs audit script.sh
```

## 📚 Documentation

**The Rash Book** is the canonical source for all documentation:

### [**→ Read The Rash Book**](https://paiml.github.io/bashrs/)

**Quick links**:
- [Getting Started](https://paiml.github.io/bashrs/getting-started/installation.html)
- [Quick Start Tutorial](https://paiml.github.io/bashrs/getting-started/quick-start.html)
- [Linting Rules](https://paiml.github.io/bashrs/linting/security.html)
- [Configuration Management](https://paiml.github.io/bashrs/config/overview.html)
- [API Reference](https://docs.rs/bashrs)

**Why the book?**
- ✅ All examples automatically tested
- ✅ Always up-to-date with latest release
- ✅ Comprehensive coverage of all features
- ✅ Real-world examples and tutorials

## Quality Metrics (v6.36.0+)

| Metric | Status |
|--------|--------|
| **Quality Grade** | **A+ (Near Perfect)** ✅ |
| **Tests** | **6,583 passing** (0 failures) ✅ |
| **Coverage** | **88.71%** (exceeds 85% target) ✅ |
| **Mutation Testing** | **92% kill rate** ✅ |
| **Property Tests** | **52+ properties** (~26k+ cases) ✅ |
| **ShellCheck** | **100% compliant** ✅ |
| **Shell Compatibility** | **sh, dash, bash, ash, zsh, mksh** ✅ |
| **Golden Traces** | **Renacer integration for regression detection** ✅ |

### Golden Trace Regression Detection (v6.36.0+)

Rash integrates with [renacer](https://github.com/paiml/renacer) to capture and compare syscall patterns for regression detection:

```bash
# Capture reference trace
make golden-capture TRACE=version CMD='./target/release/bashrs --version'

# Compare against golden (detect regressions)
make golden-compare TRACE=version CMD='./target/release/bashrs --version'
```

**Use cases**:
- Detect unexpected file access patterns
- Prevent security regressions
- Verify performance optimizations reduce syscalls
- Ensure deterministic behavior across builds

See [Golden Trace Documentation](docs/GOLDEN_TRACE.md) for complete guide.

## Shell Compatibility

Generated scripts are tested on:

| Shell | Version | Status |
|-------|---------|--------|
| POSIX sh | - | ✅ Full support |
| dash | 0.5.11+ | ✅ Full support |
| bash | 3.2+ | ✅ Full support |
| ash (BusyBox) | 1.30+ | ✅ Full support |
| zsh | 5.0+ | ✅ Full support |
| mksh | R59+ | ✅ Full support |

## Performance

Rash is designed for fast transpilation:

- **Rust-to-Shell**: 21.1µs transpile time
- **Makefile Parsing**: 0.034-1.43ms (70-320x faster than targets)
- **Memory Usage**: <10MB for most scripts

## MCP Server

Rash provides a Model Context Protocol (MCP) server for AI-assisted shell script generation:

```bash
# Install MCP server
cargo install rash-mcp

# Run server
rash-mcp
```

Available in the official MCP registry as `io.github.paiml/rash`.

## Contributing

We welcome contributions! See our [Contributing Guide](CONTRIBUTING.md) for details.

```bash
# Clone and test
git clone https://github.com/paiml/bashrs.git
cd bashrs
make test

# Run all quality checks
make validate
```

## License

Rash is licensed under the MIT License. See [LICENSE](LICENSE) for details.

## Acknowledgments

Rash is built with safety principles inspired by:
- [ShellCheck](https://www.shellcheck.net/) for shell script analysis
- [Oil Shell](https://www.oilshell.org/) for shell language design
- The Rust community for memory safety practices

---

**For comprehensive documentation, tutorials, and examples, visit [The Rash Book](https://paiml.github.io/bashrs/).**
