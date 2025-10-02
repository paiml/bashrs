# Rash MCP Server

MCP (Model Context Protocol) server for Rash - the Rust-to-Shell transpiler.

## Features

- **Transpile Tool**: Convert Rust code to POSIX-compliant shell scripts
- **Type-safe**: Strongly typed inputs and outputs using JSON Schema
- **Async execution**: Built on tokio for efficient async processing
- **Zero-boilerplate**: Powered by pforge framework

## Installation

```bash
cargo build --release -p rash-mcp
```

## Usage

### Standalone Demo

```bash
cargo run -p rash-mcp
```

### As MCP Server

The server exposes one tool:

#### `transpile`

Transpiles Rust source code to POSIX shell script.

**Input**:
```json
{
  "source": "fn main() { println!(\"Hello, World!\"); }",
  "optimize": false,
  "strict": false
}
```

**Output**:
```json
{
  "shell_script": "#!/bin/sh\n...",
  "warnings": []
}
```

**Parameters**:
- `source` (string, required): Rust source code to transpile
- `optimize` (boolean, optional): Enable optimizations (default: false)
- `strict` (boolean, optional): Enable strict mode (default: false)

## Examples

### Basic Transpilation

```rust
fn main() {
    let x = 42;
}
```

Generates:
```sh
#!/bin/sh
main() {
    x=42
}
main "$@"
```

### With println! Macro

```rust
fn main() {
    println!("Hello, MCP!");
}
```

Generates:
```sh
#!/bin/sh
rash_println() {
    printf '%s\n' "$1"
}
main() {
    rash_println 'Hello, MCP!'
}
main "$@"
```

## Testing

```bash
cargo test -p rash-mcp
```

## Architecture

- **Handler**: `TranspileHandler` implements pforge `Handler` trait
- **Runtime**: Uses `pforge-runtime` for MCP protocol
- **Transpiler**: Delegates to `bashrs` crate

## Development

Built using EXTREME TDD:
1. RED: Write failing test
2. GREEN: Make it pass
3. REFACTOR: Clean up

All 3 handler tests passing (100% coverage of critical paths).
