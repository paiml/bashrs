# Development Setup

This guide covers setting up your development environment for contributing to Rash (bashrs). Following these steps ensures you have all tools needed for EXTREME TDD development.

## Prerequisites

### Required Software

**Rust Toolchain** (version 1.70+):
```bash
# Install Rust via rustup (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verify installation
rustc --version  # Should be 1.70.0 or higher
cargo --version
```

**Git**:
```bash
# Check if git is installed
git --version

# If not installed:
# Ubuntu/Debian
sudo apt-get install git

# macOS
brew install git
```

### Optional (Recommended) Software

**shellcheck** - For POSIX compliance verification:
```bash
# macOS
brew install shellcheck

# Ubuntu/Debian
sudo apt-get install shellcheck

# Arch Linux
sudo pacman -S shellcheck

# Verify installation
shellcheck --version
```

**mdbook** - For building documentation:
```bash
# Install from crates.io
cargo install mdbook

# Verify installation
mdbook --version
```

**cargo-mutants** - For mutation testing (NASA-level quality):
```bash
# Install from crates.io
cargo install cargo-mutants

# Verify installation
cargo mutants --version
```

**cargo-llvm-cov** - For code coverage measurement:
```bash
# Install from crates.io
cargo install cargo-llvm-cov

# Verify installation
cargo llvm-cov --version
```

**wasm-pack** - For WebAssembly development (if working on WASM features):
```bash
# Install from crates.io
cargo install wasm-pack

# Verify installation
wasm-pack --version
```

## Clone the Repository

```bash
# Clone from GitHub
git clone https://github.com/paiml/bashrs.git
cd bashrs

# Verify you're on main branch
git status
# Should show: On branch main
```

## Project Structure

Rash uses a Cargo workspace with multiple crates:

```text
bashrs/
├── rash/              # Core library (parser, linter, transpiler)
├── rash-runtime/      # Runtime library for generated scripts
├── rash-mcp/          # Model Context Protocol server
├── book/              # mdbook documentation
├── examples/          # Example scripts and usage
├── scripts/           # Development scripts
│   └── hooks/         # Git pre-commit hooks
└── Cargo.toml         # Workspace configuration
```

### Workspace Members

- **rash** - Main crate containing:
  - Bash parser
  - Makefile parser
  - Security linter (SEC001-SEC008)
  - Transpilation engine
  - CLI tool (`bashrs` binary)

- **rash-runtime** - Runtime support library:
  - POSIX-compliant shell functions
  - Helper utilities for generated scripts

- **rash-mcp** - MCP server for AI integration:
  - Model Context Protocol implementation
  - AI-assisted shell script generation

## Initial Build

### Build the Project

```bash
# Build all workspace members
cargo build

# Or build in release mode (optimized)
cargo build --release
```

### Run Tests

```bash
# Run all library tests (6321+ tests)
cargo test --lib

# Expected output:
# test result: ok. 6321 passed; 0 failed; 0 ignored

# Run tests with output
cargo test --lib -- --nocapture

# Run specific test
cargo test --lib test_sec001
```

### Install Development Version

```bash
# Install from local source
cargo install --path rash

# Verify installation
bashrs --version
# Should output: bashrs 6.30.1

# Test CLI
bashrs lint examples/security/sec001_eval.sh
```

## Development Workflow

### EXTREME TDD Cycle

Rash follows **EXTREME TDD** methodology:

**Formula**: EXTREME TDD = TDD + Property Testing + Mutation Testing + Fuzz Testing + PMAT + Examples

#### Phase 1: RED - Write Failing Test

```bash
# 1. Create test file or add test to existing file
# Example: rash/src/linter/rules/tests.rs

# 2. Run test (should FAIL)
cargo test --lib test_new_feature

# Expected: Test FAILS (RED) ✅
```

#### Phase 2: GREEN - Implement Feature

```bash
# 1. Implement the feature
# 2. Run test again (should PASS)
cargo test --lib test_new_feature

# Expected: Test PASSES (GREEN) ✅
```

#### Phase 3: REFACTOR - Clean Up Code

```bash
# 1. Refactor code (extract helpers, improve readability)
# 2. Verify all tests still pass
cargo test --lib

# 3. Check code formatting
cargo fmt

# 4. Run clippy for lint warnings
cargo clippy --all-targets -- -D warnings

# Expected: Zero warnings ✅
```

#### Phase 4: QUALITY - Comprehensive Validation

```bash
# 1. Run property-based tests
cargo test --lib prop_

# 2. Run mutation testing (for critical code)
cargo mutants --file rash/src/linter/rules/sec001.rs --timeout 300 -- --lib

# Expected: 90%+ mutation kill rate ✅

# 3. Measure code coverage
cargo llvm-cov --lib

# Expected: >85% coverage ✅

# 4. Verify examples work
cargo run --example quality_tools_demo
```

## Common Development Tasks

### Running Tests

```bash
# Run all tests
cargo test --lib

# Run tests for specific module
cargo test --lib linter::

# Run tests matching pattern
cargo test --lib sec00

# Run property tests with more cases
env PROPTEST_CASES=10000 cargo test --lib prop_

# Run tests with timing info
cargo test --lib -- --test-threads=1
```

### Code Quality Checks

```bash
# Format code
cargo fmt

# Check formatting without modifying files
cargo fmt -- --check

# Run clippy (Rust linter)
cargo clippy --all-targets

# Run clippy with strict warnings
cargo clippy --all-targets -- -D warnings
```

### Measuring Coverage

```bash
# Generate coverage report
cargo llvm-cov --lib

# Generate HTML coverage report
cargo llvm-cov --lib --html
# Opens report in browser

# Generate JSON coverage report
cargo llvm-cov --lib --json --output-path coverage.json
```

### Mutation Testing

```bash
# Test specific file
cargo mutants --file rash/src/linter/rules/sec001.rs --timeout 300 -- --lib

# Test with longer timeout (for complex files)
cargo mutants --file rash/src/bash_parser/parser.rs --timeout 600 -- --lib

# Run in background and monitor
cargo mutants --file rash/src/linter/rules/sec002.rs --timeout 300 -- --lib 2>&1 | tee mutation.log &
tail -f mutation.log
```

### Building Documentation

```bash
# Build the book
cd book
mdbook build

# Serve book locally (with live reload)
mdbook serve
# Opens at http://localhost:3000

# Test code examples in book
mdbook test
```

### Running Examples

```bash
# List available examples
ls examples/*.rs

# Run specific example
cargo run --example quality_tools_demo

# Run example with arguments
cargo run --example database_migration -- --dry-run
```

## Git Pre-Commit Hooks

Rash uses pre-commit hooks to enforce quality standards.

### Install Hooks

```bash
# Run installation script
./scripts/hooks/install-hooks.sh

# Verify installation
ls -la .git/hooks/pre-commit
```

### What Hooks Check

Pre-commit hooks verify:

1. **Tests pass**: `cargo test --lib`
2. **No clippy warnings**: `cargo clippy --all-targets -- -D warnings`
3. **Code formatted**: `cargo fmt -- --check`
4. **Complexity <10**: Checks function complexity

If any check fails, the commit is **rejected**. Fix the issues before committing.

### Skipping Hooks (Emergency Only)

```bash
# Skip hooks (NOT RECOMMENDED)
git commit --no-verify -m "Emergency fix"

# Better: Fix the issues properly
cargo fmt
cargo clippy --all-targets --fix
cargo test --lib
git commit -m "Fix: Proper fix with all checks passing"
```

## Environment Variables

### Development Configuration

```bash
# Increase property test cases for thorough testing
export PROPTEST_CASES=10000

# Enable detailed test output
export RUST_TEST_THREADS=1

# Enable backtrace on panic
export RUST_BACKTRACE=1
export RUST_BACKTRACE=full  # Even more detail

# Set log level for tracing
export RUST_LOG=debug
export RUST_LOG=bashrs=trace  # Only bashrs crate
```

### Performance Profiling

```bash
# Build with profiling symbols
cargo build --profile profiling

# Run with profiler
cargo flamegraph --bin bashrs -- lint examples/security/sec001_eval.sh
```

## Troubleshooting

### "cargo: command not found"

Rust toolchain not installed or not in PATH.

**Solution**:
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add to PATH (usually automatic, but manual if needed)
source $HOME/.cargo/env
```

### Tests Failing After Pull

Dependency changes or API breaking changes.

**Solution**:
```bash
# Clean build artifacts
cargo clean

# Update dependencies
cargo update

# Rebuild
cargo build

# Run tests
cargo test --lib
```

### Clippy Warnings Won't Fix

Old clippy version or caching issues.

**Solution**:
```bash
# Update Rust toolchain
rustup update

# Clean clippy cache
cargo clean
cargo clippy --all-targets -- -D warnings
```

### Slow Test Execution

Too many tests running in parallel.

**Solution**:
```bash
# Run tests single-threaded
cargo test --lib -- --test-threads=1

# Or limit parallel tests
cargo test --lib -- --test-threads=4
```

### "shellcheck: command not found"

shellcheck not installed.

**Solution**:
```bash
# macOS
brew install shellcheck

# Ubuntu/Debian
sudo apt-get install shellcheck

# Verify
shellcheck --version
```

### Mutation Testing Takes Too Long

Default timeout may be insufficient for complex files.

**Solution**:
```bash
# Increase timeout
cargo mutants --file rash/src/module.rs --timeout 600 -- --lib

# Run overnight for comprehensive testing
cargo mutants --timeout 600 -- --lib 2>&1 | tee mutation_full.log &
```

## Development Best Practices

### Before Making Changes

1. **Pull latest changes**:
   ```bash
   git pull origin main
   ```

2. **Verify tests pass**:
   ```bash
   cargo test --lib
   ```

3. **Check clean state**:
   ```bash
   git status  # Should be clean
   ```

### While Developing

1. **Run tests frequently**:
   ```bash
   cargo test --lib test_your_feature
   ```

2. **Keep tests passing**: Never commit broken tests

3. **Format code regularly**:
   ```bash
   cargo fmt
   ```

### Before Committing

1. **Run all tests**:
   ```bash
   cargo test --lib
   ```

2. **Format code**:
   ```bash
   cargo fmt
   ```

3. **Check clippy**:
   ```bash
   cargo clippy --all-targets -- -D warnings
   ```

4. **Verify hooks will pass**:
   ```bash
   ./scripts/hooks/pre-commit
   ```

## Editor Setup

### VS Code

Recommended extensions:

```json
{
  "recommendations": [
    "rust-lang.rust-analyzer",
    "tamasfe.even-better-toml",
    "serayuzgur.crates",
    "vadimcn.vscode-lldb"
  ]
}
```

Settings (`.vscode/settings.json`):

```json
{
  "rust-analyzer.check.command": "clippy",
  "rust-analyzer.check.allTargets": true,
  "editor.formatOnSave": true,
  "editor.defaultFormatter": "rust-lang.rust-analyzer"
}
```

### Vim/Neovim

Install rust-analyzer and configure your plugin manager:

```lua
-- For nvim-lspconfig
require('lspconfig').rust_analyzer.setup({
  settings = {
    ['rust-analyzer'] = {
      checkOnSave = {
        command = 'clippy',
      },
    },
  },
})
```

### IntelliJ IDEA / CLion

1. Install "Rust" plugin
2. Open project root
3. IntelliJ will auto-detect Cargo workspace
4. Configure "On Save" actions:
   - Format with rustfmt
   - Run clippy

## Performance Tips

### Fast Incremental Builds

```bash
# Use dev-fast profile for faster compilation
cargo build --profile dev-fast

# Enable shared target directory (across projects)
export CARGO_TARGET_DIR=~/.cargo-target
```

### Parallel Testing

```bash
# Let cargo use optimal thread count
cargo test --lib

# Or specify explicitly
cargo test --lib -- --test-threads=8
```

### Caching Dependencies

```bash
# Use sccache for faster rebuilds
cargo install sccache
export RUSTC_WRAPPER=sccache
```

## Next Steps

Now that your environment is set up:

1. Read [EXTREME TDD](./extreme-tdd.md) methodology
2. Check [Release Process](./release.md) for releasing
3. Review [Toyota Way](./toyota-way.md) principles
4. Browse [Examples](../examples/) for practical usage

## Getting Help

If you encounter issues:

1. Check [Troubleshooting](#troubleshooting) section above
2. Search existing GitHub issues: https://github.com/paiml/bashrs/issues
3. Ask in discussions: https://github.com/paiml/bashrs/discussions
4. Read the [Book](https://docs.claude.com/en/docs/claude-code/)

---

**Quality Reminder**: Rash follows **zero-defect policy**. All tests must pass, clippy must be clean, and code must be formatted before committing. The pre-commit hooks enforce this automatically.
