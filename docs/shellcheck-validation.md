# ShellCheck Validation in Rash

Rash includes built-in ShellCheck-compatible validation that enforces shell safety at compile time. This ensures that generated shell scripts are free from common security vulnerabilities and portability issues.

## Overview

The validation system implements 20 critical ShellCheck rules directly in the transpiler, providing:

- **Zero-cost validation**: <1% overhead during transpilation
- **Compile-time safety**: Errors caught before shell generation
- **Auto-fix support**: Common issues automatically corrected
- **Configurable strictness**: From minimal to paranoid validation levels

## Validation Levels

```bash
# Minimal validation (default) - 20 core security rules
rash build script.rs --validation minimal

# Strict validation - Includes style rules
rash build script.rs --validation strict --strict

# No validation (dangerous, for debugging only)
rash build script.rs --validation none

# Paranoid validation - All rules + external shellcheck
rash build script.rs --validation paranoid
```

## Implemented Rules

### Critical Security Rules

| Rule | Description | Auto-Fix |
|------|-------------|----------|
| SC2086 | Double quote variables to prevent globbing/splitting | ✓ |
| SC2046 | Quote command substitutions | ✓ |
| SC2035 | Protect globs starting with `-` | ✓ |
| SC2115 | Use `${var:?}` for safer rm operations | ✓ |
| SC2068 | Double quote array expansions | ✓ |

### Best Practices

| Rule | Description | Auto-Fix |
|------|-------------|----------|
| SC2006 | Use `$()` instead of backticks | ✓ |
| SC2164 | Use `cd ... || exit` | ✓ |
| SC2162 | read without -r mangles backslashes | ✓ |
| SC2220 | Replace Unicode quotes with ASCII | ✓ |
| SC2181 | Check exit codes directly | ✗ |

## Examples

### Variable Quoting (SC2086)

```rust
// Rust input
fn main() {
    let user = std::env::var("USER").unwrap();
    println!("Hello {}", user);
}
```

```bash
# Generated shell (safe)
main() {
    local user="$USER"  # Automatically quoted
    echo "Hello $user"  # Safe in this context
}
```

### Command Substitution (SC2046)

```rust
// Rust input
let files = std::process::Command::new("find")
    .arg(".")
    .arg("-name")
    .arg("*.txt")
    .output()?;
```

```bash
# Generated shell (safe)
files="$(find . -name '*.txt')"  # Properly quoted
```

### Glob Protection (SC2035)

```rust
// Rust input
remove_files("-rf");  // Dangerous pattern
```

```bash
# Generated shell (safe)
remove_files './-rf'  # Automatically prefixed with ./
```

## Integration with CI/CD

### GitHub Actions

Rash includes comprehensive ShellCheck validation in CI/CD pipelines:

```yaml
# Complete ShellCheck validation job
shellcheck-validation:
  name: ShellCheck Validation
  runs-on: ubuntu-latest
  steps:
  - uses: actions/checkout@v4
  
  - name: Install ShellCheck
    run: |
      sudo apt-get update
      sudo apt-get install -y shellcheck
  
  - name: Install Rust
    uses: dtolnay/rust-toolchain@stable
  
  - name: Build rash
    run: cargo build --release --workspace
  
  - name: Run ShellCheck validation
    run: make shellcheck-validate
  
  - name: Run ShellCheck integration tests
    run: cargo test --test shellcheck_validation
```

### Local Development

```bash
# Install ShellCheck locally
make shellcheck-install

# Run validation on generated scripts
make shellcheck-validate

# Run comprehensive test suite
make shellcheck-test-all
```

### Pre-commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit
for file in $(git diff --cached --name-only | grep '\.rs$'); do
    if rash check "$file" 2>/dev/null; then
        rash build "$file" --validation strict --strict || exit 1
    fi
done
```

## Performance

Validation adds minimal overhead:

```
Benchmark results (1000 line script):
- Parsing:      12.3ms
- IR Transform:  8.7ms
- Validation:    0.2ms (0.97% overhead)
- Emission:      3.4ms
- Total:        24.6ms
```

## Custom Validation Rules

While the core 20 rules are built-in, you can add custom rules:

```toml
# .rash.toml
[validation]
level = "strict"
fail_on_warnings = true
custom_rules = ["no-curl-pipe-sh", "require-set-euo"]

[[validation.custom_rules.no-curl-pipe-sh]]
pattern = 'curl.*\|.*sh'
message = "Never pipe curl directly to sh"
severity = "error"

[[validation.custom_rules.require-set-euo]]
pattern = '^(?!.*set -euo pipefail)'
message = "Scripts must start with 'set -euo pipefail'"
severity = "warning"
```

## Troubleshooting

### Common Issues

1. **"Validation overhead exceeds 1%"**
   - This should never happen in production builds
   - File a bug report with reproduction steps

2. **"Auto-fix changed script behavior"**
   - Auto-fixes are designed to be behavior-preserving
   - Report any semantic changes as bugs

3. **"False positive validation errors"**
   - Use `#[allow(shellcheck::SC2086)]` to suppress specific rules
   - Or disable validation for a function:
   ```rust
   #[rash::no_validate]
   fn legacy_function() {
       // Unvalidated code
   }
   ```

## Test Suite

Rash includes comprehensive ShellCheck test coverage:

### Test Categories

1. **SC2086 - Variable Quoting**: Ensures all variables are properly quoted
2. **SC2046 - Command Substitution**: Tests modern `$()` syntax usage
3. **SC2035 - Glob Protection**: Validates protection against dangerous patterns
4. **SC2164 - CD Safety**: Ensures `cd` operations include error handling
5. **SC2068 - Array Expansion**: Tests proper array handling
6. **SC2006 - Modern Substitution**: Validates command substitution syntax
7. **SC2115 - Safe RM**: Tests variable validation before destructive operations
8. **Complex Installers**: Real-world installer script validation
9. **Error Handling**: Comprehensive error handling patterns

### Running Tests

```bash
# Run all ShellCheck tests
cargo test --test shellcheck_validation

# Run specific test category
cargo test shellcheck_validation::test_variable_quoting_sc2086

# Generate and validate specific test
make shellcheck-test-all
```

### Test Files Location

- Test fixtures: `tests/fixtures/shellcheck/*.rs`
- Generated output: `tests/shellcheck-output/*.sh` (gitignored)
- Integration tests: `tests/integration/shellcheck_validation.rs`

## Future Roadmap

- **v0.2**: 50 additional POSIX rules with control flow analysis
- **v0.3**: Full ShellCheck compatibility via FFI
- **v0.4**: Custom rule API with regex and AST patterns
- **v0.5**: IDE integration with real-time validation

## See Also

- [ShellCheck Wiki](https://www.shellcheck.net/wiki/)
- [POSIX Shell Specification](https://pubs.opengroup.org/onlinepubs/9699919799/)
- [Rash Security Model](./security.md)