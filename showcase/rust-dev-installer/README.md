# Rust Development Environment Installer

A complete Rust development environment setup using the bashrs installer framework.

## What's Installed

This installer sets up:

### Toolchains
- **Rustup** - Rust toolchain manager
- **Rust Stable** - Default stable toolchain with clippy and rustfmt
- **Rust Nightly** - Nightly toolchain with miri for testing

### Cargo Tools
- **cargo-watch** - Watch for file changes and run commands
- **cargo-nextest** - Next-generation test runner
- **cargo-audit** - Security vulnerability scanner
- **cargo-deny** - Dependency policy enforcement
- **cargo-llvm-cov** - Code coverage using LLVM
- **cargo-mutants** - Mutation testing
- **sccache** - Build caching for faster compilation

### System Dependencies
- build-essential (gcc, make, etc.)
- pkg-config
- libssl-dev
- curl

## Usage

### Dry Run (Preview)

```bash
bashrs installer run ./showcase/rust-dev-installer --dry-run
```

### Full Installation

```bash
bashrs installer run ./showcase/rust-dev-installer
```

### With Signature Verification

```bash
# Initialize keyring first
bashrs installer keyring init

# Run with verification
bashrs installer run ./showcase/rust-dev-installer --verify-signatures
```

### Audit the Installer

```bash
bashrs installer audit ./showcase/rust-dev-installer
```

## Features Demonstrated

- **Checkpointing**: Each step saves progress, allowing resume on failure
- **Postconditions**: Verify each step completed successfully
- **Verification Commands**: Multiple verification checks per step
- **Dependencies**: Step ordering via depends_on
- **Idempotency**: Safe to run multiple times
- **Hermetic Mode**: Reproducible builds with locked versions

## Step Graph

```
install-deps
     │
     ▼
install-rustup
     │
     ├──────────────────┬────────────────┐
     ▼                  ▼                ▼
install-stable    configure-shell   (parallel)
     │
     ├──────────────────┬────────────────┐
     ▼                  ▼                ▼
install-nightly   install-cargo-tools   install-sccache
     │                  │                │
     └──────────────────┴────────────────┘
                        │
                        ▼
              verify-installation
```

## Customization

Edit `installer.toml` to customize:

- Add/remove cargo tools in step `install-cargo-tools`
- Modify shell configuration in step `configure-shell`
- Add project-specific dependencies
