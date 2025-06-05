# rash

rash transpiles a subset of Rust to POSIX shell, with automatic safety validation.

[![CI](https://github.com/paiml/rash/workflows/CI/badge.svg)](https://github.com/paiml/rash/actions)
[![codecov](https://codecov.io/gh/paiml/rash/branch/main/graph/badge.svg)](https://codecov.io/gh/paiml/rash)
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
cargo install --git https://github.com/paiml/rash
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
rash init <project>                    # Initialize new project (planned)
```

### Options

```
-O, --optimize           Enable optimizations
-d, --dialect <DIALECT>  Target shell dialect [default: posix]
-v, --verbose           Verbose output
--verify <LEVEL>        Verification level [none, basic, strict, paranoid]
```

## Performance

Transpilation is near-instant for typical scripts:

```bash
$ time rash build installer.rs -o installer.sh
real    0m0.024s
```

Binary sizes:
- Linux x86_64: ~4.2MB (static, musl)
- macOS: ~4.4MB
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

## License

MIT