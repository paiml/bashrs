# rash

rash transpiles a subset of Rust to POSIX shell, with automatic safety validation.

[![CI](https://github.com/paiml/rash/workflows/CI/badge.svg)](https://github.com/paiml/rash/actions)
[![Coverage](https://paiml.github.io/rash/badges/coverage.svg)](https://paiml.github.io/rash/coverage/)
[![Tests](https://img.shields.io/badge/tests-400%2B%20passing-brightgreen)](https://github.com/paiml/rash/actions)
[![QuickCheck](https://img.shields.io/badge/QuickCheck-1000%2B%20properties-blue)](https://github.com/paiml/rash/blob/main/rash/src/testing/quickcheck_tests.rs)

## Usage

Write Rust:

```rust
// install.rs
fn main() {
    let version = "1.0.0";
    let prefix = "/usr/local";
    
    // Variables are automatically quoted in output
    let install_dir = concat(prefix, "/bin");
    mkdir_p(install_dir);
}
```

Get POSIX shell:

```bash
$ rash build install.rs -o install.sh
$ cat install.sh
#!/bin/sh
set -euf
version="1.0.0"
prefix="/usr/local"
install_dir="$prefix/bin"
mkdir -p "$install_dir"
```

## Installation

### Binary releases

Pre-built binaries include all features (compile mode, playground, verification) by default:

```bash
# Linux x86_64
curl -L https://github.com/paiml/rash/releases/latest/download/rash-x86_64-unknown-linux-gnu.tar.gz | tar xz

# macOS x86_64  
curl -L https://github.com/paiml/rash/releases/latest/download/rash-x86_64-apple-darwin.tar.gz | tar xz

# macOS ARM64
curl -L https://github.com/paiml/rash/releases/latest/download/rash-aarch64-apple-darwin.tar.gz | tar xz
```

### From source

```bash
# Install with default features (includes compile mode and playground)
cargo install --git https://github.com/paiml/rash

# Install minimal version (smaller binary, core features only)
cargo install --git https://github.com/paiml/rash --no-default-features --features minimal
```

## Supported Rust subset

rash supports a minimal subset of Rust that maps cleanly to shell:

- **Variables**: `let x = "value";` → `x="value"`
- **Functions**: Limited built-ins only (echo, mkdir_p, concat, etc.)
- **Control flow**: `if`/`else` (basic conditions)
- **Types**: Strings and integers only

Not supported:
- Heap allocation, Vec, HashMap
- Loops (for, while)
- Pattern matching
- Error handling (`Result`, `?`)
- External crates
- Most of Rust's standard library

## Safety features

All output passes these ShellCheck rules:

- **SC2086**: Variables are quoted
- **SC2046**: Command substitutions are quoted
- **SC2035**: Glob patterns are protected
- **SC2164**: `cd` failures are handled
- **SC2006**: Backticks avoided for command substitution

Example:

```rust
fn main() {
    let user_input = env_var("USER_INPUT");
    echo(user_input);  // Transpiles to: echo "$user_input"
}
```

## Commands

```bash
rash build <input.rs> -o <output.sh>   # Transpile Rust to shell
rash check <input.rs>                  # Validate without output
rash init <project>                    # Initialize new project  
rash verify <input.rs> <output.sh>     # Verify transpilation correctness
rash compile <input.rs> -o <binary>    # Compile to standalone binary
rash inspect <input>                   # Inspect AST and formal properties
rash playground                        # Interactive REPL (experimental)
```

### Options

```
-O, --optimize           Enable optimizations
-d, --dialect <DIALECT>  Target shell dialect [default: posix]
-v, --verbose           Verbose output
--verify <LEVEL>        Verification level [none, basic, strict, paranoid]
--emit-proof            Emit formal verification proof
--strict                Enable strict mode
--validation <LEVEL>    Validation level [minimal, standard, comprehensive]
```

## Performance

Transpilation is near-instant for typical scripts:

```bash
$ time rash build installer.rs -o installer.sh
real    0m0.024s
```

Binary sizes:
- Linux x86_64: ~4.6MB (static, with all features)
- macOS: ~4.8MB (with all features)
- Minimal build: ~3.2MB (core transpilation only)
- No runtime dependencies

## Examples

### Basic script

```rust
// hello.rs
fn main() {
    let name = "World";
    echo(concat("Hello, ", name));
}
```

```bash
$ rash build hello.rs -o hello.sh
$ sh hello.sh
Hello, World
```

### Installation script

```rust
// install.rs
fn main() {
    let prefix = env_var_or("PREFIX", "/usr/local");
    let bin_dir = concat(prefix, "/bin");
    
    if !path_exists(bin_dir) {
        mkdir_p(bin_dir);
    }
    
    // Copy binary
    let binary = "myapp";
    let dest = concat(bin_dir, "/", binary);
    cp(binary, dest);
    chmod("755", dest);
}
```

### Available built-ins

```rust
// I/O
echo(msg: &str)
cat(file: &str)

// Environment  
env_var(name: &str) -> String
env_var_or(name: &str, default: &str) -> String

// Filesystem
mkdir_p(path: &str)
rm_f(path: &str)
cp(src: &str, dest: &str)
mv(src: &str, dest: &str)
chmod(mode: &str, path: &str)
path_exists(path: &str) -> bool
file_exists(path: &str) -> bool

// String operations
concat(a: &str, b: &str) -> String

// Process control
exit(code: i32)
command_exists(cmd: &str) -> bool
```

## Limitations

rash is intentionally limited:

1. **No stdlib**: Only built-in functions listed above
2. **No external commands**: Can't call arbitrary programs
3. **Simple types only**: Strings and integers
4. **No collections**: No arrays, vectors, or hashmaps
5. **Basic control flow**: Only if/else, no loops

These constraints ensure predictable, safe shell output.

## How it works

```
Rust source → Parse → Restricted AST → Validate → Shell IR → Emit → POSIX shell
                         ↓
                   Safety checks
```

1. **Parse**: Uses syn to parse Rust syntax
2. **Restrict**: Converts to limited AST supporting only shell-mappable constructs
3. **Validate**: Applies ShellCheck rules at AST level
4. **Emit**: Generates shell with proper quoting and escaping

## Development

```bash
git clone https://github.com/paiml/rash
cd rash
cargo build --release
cargo test
```

Run specific test suites:
```bash
cargo test --test integration_tests
cargo test --test shellcheck_validation
cargo bench
```

## Why rash?

Shell scripts are powerful but error-prone. Common issues:

- Missing quotes → word splitting bugs
- Unhandled command failures → corrupted state
- Command injection → security vulnerabilities

rash prevents these at compile time by enforcing safe patterns in a familiar Rust syntax.

## Similar projects

- [Oil Shell](https://www.oilshell.org/): New shell language with better syntax
- [Batsh](https://github.com/batsh-dev/batsh): Transpiles a C-like syntax to Bash/Batch

rash differs by using Rust syntax and targeting POSIX compatibility.

## Core Features (Included by Default)

### Binary Compilation (Compile Mode)

Create standalone executables from your Rust scripts:

```bash
# Create a self-extracting shell script
$ rash compile install.rs -o install-standalone.sh --self-extracting
$ ./install-standalone.sh  # No rash runtime needed!

# Create a static binary with embedded dash runtime
$ rash compile install.rs -o install-bin --runtime dash

# Create a minimal Docker container
$ rash compile install.rs -o Dockerfile --container --container-format docker
```

#### Self-Extracting Scripts

Self-extracting scripts embed your transpiled shell code in a compressed format:

```bash
$ rash compile hello.rs -o hello-portable.sh --self-extracting
$ ls -lh hello-portable.sh
-rwxr-xr-x 1 user user 1.2K hello-portable.sh

$ head -5 hello-portable.sh
#!/bin/sh
# Self-extracting RASH script
set -euf
# Embedded compressed script
PAYLOAD='H4sIAAAAAAAA...'
```

Features:
- Zstandard compression (falls back to gzip if unavailable)
- Base64 encoded for portability
- Works on any POSIX system with base64 and zstd/gzip
- Typically 60-80% smaller than original

### Interactive Playground (Experimental)

A TypeScript-style REPL for interactive development:

```bash
$ rash playground

RASH Playground v0.3.0
Type :help for commands, :quit to exit

rash> let name = "world"
rash> echo(concat("Hello, ", name))
#!/bin/sh
set -euf
name="world"
echo "Hello, $name"

rash> :layout vertical
Layout changed to vertical

rash> :save session.rash
Session saved to session.rash
```

Features:
- **Live transpilation**: See shell output as you type
- **Incremental parsing**: Fast updates using tree-sitter
- **Session management**: Save/load your work
- **Multiple layouts**: Horizontal, vertical, or focused views
- **VI/Emacs keybindings**: Familiar editor controls
- **Syntax highlighting**: SIMD-accelerated for performance
- **URL sharing**: Share sessions via compressed URLs

### Formal Verification

Inspect and verify the correctness of transpilation:

```bash
# Inspect formal properties
$ rash inspect echo-example --format markdown
# Formal Verification Report

## AST Structure
- Complexity: 3
- Depth: 2
- Node count: 5

## Safety Properties
✓ No injection vulnerabilities
✓ All variables properly quoted
✓ No glob expansion risks

## Verification Proofs
- Shell injection safety: PROVEN
- Quote correctness: PROVEN
- Determinism: PROVEN

# Generate machine-readable proof
$ rash build complex.rs -o complex.sh --emit-proof
$ cat complex.proof
{
  "version": "1.0",
  "properties": {
    "injection_safety": "proven",
    "quote_correctness": "proven",
    "determinism": "proven"
  }
}
```

### Kaizen Mode

Continuous improvement tooling for maintaining code quality:

```bash
$ make kaizen
=== KAIZEN: Continuous Improvement Protocol ===

Step 1: Static Analysis
✅ Baseline metrics collected

Step 2: Performance Regression Detection
✅ No regression detected

Step 3: Complexity Evolution
Files with complexity > 10: 0
Average complexity: 4.2

Step 4: Test Coverage
Coverage: 88.70%

Step 5: Binary Size
Binary size: 4.6M

✅ Kaizen cycle complete
```

### Enhanced Testing Infrastructure

#### Property-Based Testing
```rust
// Over 1000 property tests for correctness
proptest! {
    #[test]
    fn prop_quoting_safety(input in ".*") {
        let transpiled = transpile_string(&input);
        assert_no_injection_vulnerability(&transpiled);
    }
}
```

#### Fuzzing Support
```bash
# Differential fuzzing between optimization levels
$ cargo +nightly fuzz run differential_optimization

# Coverage-guided fuzzing for parser
$ cargo +nightly fuzz run ast_parser
```

#### Cross-Shell Validation
```bash
$ make test-shells
Testing POSIX compliance across shells...
✅ sh: Compatible
✅ bash: Compatible  
✅ dash: Compatible
✅ busybox: Compatible
✅ ksh: Compatible
```

### Container Support

Build minimal containers for your scripts:

```bash
# Generate distroless container
$ rash compile app.rs -o app.tar --container --container-format oci

# Generate Dockerfile
$ rash compile app.rs -o Dockerfile --container --container-format docker
$ cat Dockerfile
FROM scratch
COPY rash /rash
USER 65534:65534
ENTRYPOINT ["/rash"]
```

### Advanced Validation Pipeline

Multi-stage validation ensures correctness:

```rust
// Comprehensive validation with custom rules
let config = Config {
    validation_level: ValidationLevel::Comprehensive,
    strict_mode: true,
    verify: VerificationLevel::Paranoid,
};

// Validates against:
// - ShellCheck rules (SC2086, SC2046, etc.)
// - Custom RASH rules (no-unquoted-vars, etc.)
// - Formal properties (determinism, idempotency)
```

## Architecture Improvements

### Modular Design
```
rash/
├── ast/           # Restricted AST with visitor pattern
├── emitter/       # POSIX-compliant shell generation
├── formal/        # Formal verification engine
├── validation/    # Multi-stage validation pipeline
├── verifier/      # Property-based correctness proofs
├── playground/    # Interactive REPL components
├── compiler/      # Binary compilation subsystem
└── container/     # Container generation
```

### Performance Optimizations
- SIMD-accelerated syntax highlighting
- Lock-free incremental computation
- Zero-copy differential rendering
- Parallel validation pipeline

## Quality Metrics

- **Test Coverage**: 88.70% (target: 85-90%)
- **Tests**: 400+ unit tests, 19 integration tests
- **Property Tests**: 1000+ QuickCheck properties
- **Complexity**: Average 4.2 per function (threshold: 10)
- **Binary Size**: 4.6MB static Linux binary
- **Dependencies**: Minimal, security-audited
- **Cross-Shell**: 100% POSIX compliance

## License

MIT