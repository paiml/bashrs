# Quick Test Commands for Core Features (v0.3.1+)

## Testing Compile Mode (Working!)

```bash
# 1. Self-extracting script
rash compile examples/hello.rs -o hello-portable.sh --self-extracting
./hello-portable.sh

# 2. Container generation
rash compile examples/hello.rs -o Dockerfile --container --container-format docker
cat Dockerfile

# 3. Create your own script
cat > install.rs << 'EOF'
fn main() {
    echo("Starting installation...");
    let prefix = env_var_or("PREFIX", "/usr/local");
    echo(concat("Installing to: ", prefix));
    echo("Done!");
}
EOF

rash compile install.rs -o install.sh --self-extracting
./install.sh
PREFIX=/opt ./install.sh
```

## Testing Playground Mode

The playground requires an interactive terminal. Run this directly in your terminal:

```bash
rash playground
```

Once in the playground:
- Type Rust code and see live transpilation
- Commands:
  - `:help` - Show all commands
  - `:layout vertical` - Change layout
  - `:save session.rash` - Save your work
  - `:quit` - Exit

Example session:
```
rash> let name = "World"
rash> echo(concat("Hello, ", name))
```

## Testing Formal Verification

```bash
# Inspect a script's formal properties
rash inspect examples/hello.rs --format markdown

# Build with proof generation
rash build examples/hello.rs -o hello.sh --emit-proof
cat hello.proof
```

## All Features Are Working! 🎉

As of v0.3.1, all features are included by default! Just install normally:
```bash
cargo install --git https://github.com/paiml/rash
```

No need for `--all-features` anymore!