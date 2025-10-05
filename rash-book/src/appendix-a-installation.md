# Appendix A: Installation Guide

This appendix provides complete installation instructions for bashrs on all supported platforms.

---

## Prerequisites

### Required

- **Rust 1.70.0 or later**: bashrs requires Rust toolchain for installation from source or crates.io
- **Operating System**: Linux, macOS, or WSL2 (Windows Subsystem for Linux)
- **Shell**: POSIX-compliant shell (sh, dash, bash, ash)

### Optional

- **ShellCheck**: For validation of generated scripts (highly recommended)
- **Git**: For installing from source
- **make**: For using the provided Makefile (optional convenience)

### System Requirements

- **Disk Space**: ~50MB for binary + dependencies
- **RAM**: Minimal (transpilation uses <100MB typically)
- **CPU**: Any modern x86_64 or ARM64 processor

---

## Installation Methods

### Method 1: Install from crates.io (Recommended)

The easiest way to install bashrs:

```bash
cargo install bashrs
```

This installs the `bashrs` binary to `~/.cargo/bin/`. Ensure this directory is in your `PATH`:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

Add this to your shell profile (`~/.bashrc`, `~/.zshrc`, etc.) to make it permanent.

**Verify installation:**
```bash
bashrs --version
```

Expected output:
```
bashrs 0.9.3
```

---

### Method 2: Install from Source

For latest development features or custom builds:

#### Clone the Repository

```bash
git clone https://github.com/paiml/bashrs
cd bashrs
```

#### Build from Source

```bash
# Development build (faster compilation, slower runtime)
cargo build

# Release build (slower compilation, optimized binary)
cargo build --release
```

The binary will be in `target/release/bashrs`.

#### Install Locally

```bash
# Install to ~/.cargo/bin
cargo install --path rash

# Or copy manually
cp target/release/bashrs ~/.local/bin/  # Or any directory in $PATH
```

**Verify installation:**
```bash
bashrs --version
```

---

### Method 3: Pre-built Binaries

Download pre-built binaries from GitHub Releases (when v1.0 is released):

```bash
# Linux x86_64
wget https://github.com/paiml/bashrs/releases/download/v1.0.0/bashrs-linux-x86_64.tar.gz
tar -xzf bashrs-linux-x86_64.tar.gz
sudo mv bashrs /usr/local/bin/

# macOS (ARM64)
wget https://github.com/paiml/bashrs/releases/download/v1.0.0/bashrs-macos-arm64.tar.gz
tar -xzf bashrs-macos-arm64.tar.gz
sudo mv bashrs /usr/local/bin/

# macOS (x86_64)
wget https://github.com/paiml/bashrs/releases/download/v1.0.0/bashrs-macos-x86_64.tar.gz
tar -xzf bashrs-macos-x86_64.tar.gz
sudo mv bashrs /usr/local/bin/
```

**Verify installation:**
```bash
bashrs --version
```

---

## Installing ShellCheck (Recommended)

ShellCheck validates generated shell scripts. bashrs can optionally use it for verification.

### Debian/Ubuntu
```bash
sudo apt-get install shellcheck
```

### macOS
```bash
brew install shellcheck
```

### From Source
```bash
cabal install ShellCheck
```

**Verify ShellCheck:**
```bash
shellcheck --version
```

---

## Shell Completion

bashrs supports shell completion for bash, zsh, and fish.

### Generate Completion Scripts

#### Bash
```bash
bashrs completions bash > ~/.local/share/bash-completion/completions/bashrs
source ~/.local/share/bash-completion/completions/bashrs
```

Add to `~/.bashrc`:
```bash
source ~/.local/share/bash-completion/completions/bashrs
```

#### Zsh
```bash
bashrs completions zsh > ~/.zsh/completions/_bashrs
```

Add to `~/.zshrc`:
```bash
fpath=(~/.zsh/completions $fpath)
autoload -Uz compinit && compinit
```

#### Fish
```bash
bashrs completions fish > ~/.config/fish/completions/bashrs.fish
```

---

## Testing Installation

### Quick Test

Create a test file:

```bash
cat > test.rs <<'EOF'
fn main() {
    let greeting = "Hello, bashrs!";
    println(greeting);
}

fn println(msg: &str) {}
EOF
```

Transpile it:

```bash
bashrs build test.rs -o test.sh
```

Run the generated script:

```bash
sh test.sh
```

Expected output:
```
Hello, bashrs!
```

### Run Test Suite

From source repository:

```bash
cd bashrs
cargo test
```

Expected: All tests pass
```
test result: ok. 237 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

---

## Configuration

### Default Configuration File

bashrs looks for configuration in:
- `./rash.toml` (project-specific)
- `~/.config/rash/config.toml` (user-wide)
- `/etc/rash/config.toml` (system-wide)

### Example Configuration

Create `~/.config/rash/config.toml`:

```toml
[transpile]
# Default target shell dialect
target = "posix"  # Options: posix, bash, dash, ash

# Verification level
verify = "strict"  # Options: none, basic, strict, paranoid

# Enable optimizations
optimize = true

# Emit formal verification proofs
emit_proof = false

[validation]
# ShellCheck validation level
shellcheck_level = "minimal"  # Options: none, minimal, standard, strict

# Strict POSIX mode (no extensions)
strict_mode = false
```

---

## Environment Variables

bashrs respects these environment variables:

| Variable | Purpose | Default |
|----------|---------|---------|
| `RASH_TARGET` | Default shell dialect | `posix` |
| `RASH_VERIFY` | Verification level | `strict` |
| `RASH_OPTIMIZE` | Enable optimizations | `true` |
| `SHELLCHECK_OPTS` | ShellCheck options | `-s sh` |

Example:
```bash
export RASH_TARGET=bash
export RASH_VERIFY=paranoid
bashrs build script.rs
```

---

## Uninstalling

### If Installed via Cargo

```bash
cargo uninstall bashrs
```

### If Installed Manually

```bash
# Remove binary
rm ~/.local/bin/bashrs  # Or wherever you installed it

# Remove configuration
rm -rf ~/.config/rash

# Remove completion scripts
rm ~/.local/share/bash-completion/completions/bashrs  # Bash
rm ~/.zsh/completions/_bashrs                         # Zsh
rm ~/.config/fish/completions/bashrs.fish             # Fish
```

---

## Troubleshooting

### Issue: `bashrs: command not found`

**Solution**: Ensure `~/.cargo/bin` is in your `PATH`:

```bash
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### Issue: `error: failed to compile bashrs`

**Solution**: Update Rust toolchain:

```bash
rustup update stable
cargo install bashrs
```

### Issue: Generated script fails shellcheck

**Solution**: Check ShellCheck version (bashrs requires 0.8.0+):

```bash
shellcheck --version
```

Update if needed:
```bash
# Debian/Ubuntu
sudo apt-get update && sudo apt-get install shellcheck

# macOS
brew upgrade shellcheck
```

### Issue: `Unsupported feature` error

**Solution**: Check that your bashrs code uses only supported features (see [Chapter 18: Limitations](ch18-limitations.md)).

Common unsupported features in v1.0:
- `for` loops (use function iteration pattern)
- `match` expressions (use if/else-if chains)
- `while` loops (planned for v1.1)

### Issue: Permission denied when running generated script

**Solution**: Make the script executable:

```bash
chmod +x generated.sh
./generated.sh
```

Or run with explicit shell:
```bash
sh generated.sh
```

### Issue: Slow transpilation

**Solution**: Use release build for better performance:

```bash
cargo build --release
./target/release/bashrs build input.rs
```

Or install optimized version:
```bash
cargo install bashrs --release
```

---

## Platform-Specific Notes

### Linux

- **Recommended shells**: dash (fastest), ash (minimal), bash (feature-rich)
- **Default sh**: Usually `/bin/dash` on Debian/Ubuntu, `/bin/bash` on RHEL/CentOS
- **Binary location**: `~/.cargo/bin/bashrs` or `/usr/local/bin/bashrs`

### macOS

- **Recommended shell**: bash 3.2+ (built-in) or zsh
- **Default sh**: `/bin/sh` (bash 3.2 compatibility mode)
- **Binary location**: `~/.cargo/bin/bashrs` or `/usr/local/bin/bashrs`
- **Note**: macOS bash is old (3.2); generated scripts are still compatible

### Windows (WSL2)

- **Use WSL2**: bashrs is not supported on native Windows (no POSIX shell)
- **Recommended**: Ubuntu or Debian WSL2 image
- **Installation**: Same as Linux (use Ubuntu/Debian WSL2 terminal)

### FreeBSD/OpenBSD

- **Status**: Should work (POSIX-compliant shells available)
- **Testing**: Limited testing, report issues at https://github.com/paiml/bashrs/issues
- **Shell**: `/bin/sh` is typically ash or dash

---

## Upgrading

### From crates.io

```bash
cargo install bashrs --force
```

### From Source

```bash
cd bashrs
git pull origin main
cargo build --release
cargo install --path rash --force
```

### Check Current Version

```bash
bashrs --version
```

---

## Development Setup

For contributors and advanced users:

### Clone and Setup

```bash
git clone https://github.com/paiml/bashrs
cd bashrs
```

### Install Development Dependencies

```bash
# Rust toolchain (nightly for coverage)
rustup install nightly

# Development tools
cargo install cargo-watch  # Auto-rebuild on changes
cargo install cargo-tarpaulin  # Code coverage
cargo install cargo-mutants  # Mutation testing
cargo install mdbook  # For building documentation
```

### Run Development Commands

```bash
# Run tests on file change
cargo watch -x test

# Generate code coverage
cargo tarpaulin --out Html

# Build documentation
cd rash-book && mdbook build

# Run mutation tests
cargo mutants
```

### Run Quality Gates (as in CI)

```bash
make test          # Run all tests
make lint          # Run clippy
make coverage      # Generate coverage report
make integration   # Run integration tests
make quality       # Run all quality checks
```

---

## Docker Usage

Run bashrs in Docker for isolated environments:

```bash
# Build Docker image
docker build -t bashrs .

# Run transpilation
docker run --rm -v $(pwd):/workspace bashrs build /workspace/input.rs -o /workspace/output.sh

# Interactive shell
docker run --rm -it -v $(pwd):/workspace bashrs bash
```

---

## Continuous Integration

Example GitHub Actions workflow:

```yaml
name: Transpile
on: [push]
jobs:
  transpile:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo install bashrs
      - run: bashrs build installer.rs -o install.sh
      - run: shellcheck install.sh
      - uses: actions/upload-artifact@v3
        with:
          name: installer
          path: install.sh
```

---

## Getting Help

- **Documentation**: https://docs.bashrs.dev (or local: `mdbook serve rash-book`)
- **GitHub Issues**: https://github.com/paiml/bashrs/issues
- **Discord**: https://discord.gg/bashrs
- **Email**: support@pragmaticai.com

---

## Quick Reference

### Common Commands

```bash
# Transpile to shell script
bashrs build input.rs -o output.sh

# Check syntax only (no output)
bashrs check input.rs

# Verify with ShellCheck
bashrs build input.rs --verify strict

# Specify target shell
bashrs build input.rs --target bash

# Enable optimizations
bashrs build input.rs --optimize

# Compile to self-extracting binary
bashrs compile input.rs -o installer

# Show IR (intermediate representation)
bashrs inspect ir input.rs

# Show AST
bashrs inspect ast input.rs

# Initialize new project
bashrs init my-installer
```

### File Locations

- **Binary**: `~/.cargo/bin/bashrs`
- **Config**: `~/.config/rash/config.toml`
- **Completions**: `~/.local/share/bash-completion/completions/bashrs`
- **Cache**: `~/.cache/rash/`

---

**Next Steps**: Now that bashrs is installed, start with [Chapter 1: Hello Shell](ch01-hello-shell-tdd.md) to learn the basics!
