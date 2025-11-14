# Appendix D: Complete API Reference

This appendix provides comprehensive API documentation for all bashrs commands, flags, and configuration options.

---

## Command-Line Interface

### bashrs build

Transpile Rust code to shell scripts with validation.

```sh
bashrs build [OPTIONS] <FILES...>
```

**Arguments:**
- `<FILES...>` - Rust source files to transpile (`.rs` extension)

**Options:**
- `--output-dir <DIR>` - Output directory for generated scripts (default: `./dist`)
- `--validation <LEVEL>` - Validation level: `none`, `minimal`, `strict`, `paranoid` (default: `minimal`)
- `--strict` - Treat warnings as errors
- `--no-verify` - Skip shellcheck verification
- `--format <FORMAT>` - Output format: `shell`, `json`, `yaml` (default: `shell`)
- `--target <SHELL>` - Target shell: `sh`, `bash`, `dash`, `zsh` (default: `sh`)
- `--help` - Display help information
- `--version` - Display version information

**Examples:**
```sh
# Basic transpilation
bashrs build src/main.rs

# Strict validation with custom output
bashrs build src/*.rs --validation strict --strict --output-dir build/

# Transpile for specific shell
bashrs build installer.rs --target bash --output-dir dist/
```

**Exit Codes:**
- `0` - Success
- `1` - Compilation error
- `2` - Validation error
- `3` - File I/O error

---

### bashrs parse

Parse bash or Makefile to AST (Abstract Syntax Tree).

```sh
bashrs parse [OPTIONS] <FILE>
```

**Arguments:**
- `<FILE>` - Bash script (`.sh`) or Makefile to parse

**Options:**
- `--format <FORMAT>` - Output format: `json`, `yaml`, `tree` (default: `tree`)
- `--output <FILE>` - Write output to file instead of stdout
- `--pretty` - Pretty-print JSON/YAML output
- `--help` - Display help information

**Examples:**
```sh
# Parse bash script to tree view
bashrs parse deploy.sh

# Parse Makefile to JSON
bashrs parse Makefile --format json --pretty

# Save AST to file
bashrs parse script.sh --format yaml --output ast.yaml
```

**Exit Codes:**
- `0` - Success
- `1` - Parse error
- `3` - File I/O error

---

### bashrs purify

Transform non-deterministic bash to safe POSIX sh.

```sh
bashrs purify [OPTIONS] <FILE>
```

**Arguments:**
- `<FILE>` - Bash script to purify

**Options:**
- `--output <FILE>` - Output file (default: stdout)
- `--in-place` - Overwrite input file
- `--validation <LEVEL>` - Validation level (default: `strict`)
- `--strict` - Treat warnings as errors
- `--diff` - Show diff of changes
- `--dry-run` - Preview changes without writing
- `--help` - Display help information

**Examples:**
```sh
# Purify to stdout
bashrs purify messy.sh

# Purify in-place
bashrs purify legacy.sh --in-place

# Preview changes
bashrs purify old-script.sh --dry-run --diff

# Strict purification
bashrs purify build.sh --validation paranoid --strict --output clean-build.sh
```

**Exit Codes:**
- `0` - Success
- `1` - Purification error
- `2` - Validation error
- `3` - File I/O error

---

### bashrs lint

Lint bash scripts or Makefiles with shellcheck rules.

```sh
bashrs lint [OPTIONS] <FILES...>
```

**Arguments:**
- `<FILES...>` - Files to lint (bash scripts or Makefiles)

**Options:**
- `--validation <LEVEL>` - Validation level (default: `strict`)
- `--strict` - Treat warnings as errors
- `--format <FORMAT>` - Output format: `text`, `json`, `checkstyle`, `gcc` (default: `text`)
- `--output <FILE>` - Write output to file
- `--fix` - Auto-fix issues where possible
- `--fix-all` - Auto-fix all issues (no prompts)
- `--help` - Display help information

**Examples:**
```sh
# Lint single file
bashrs lint deploy.sh

# Lint multiple files
bashrs lint src/*.sh

# Auto-fix issues
bashrs lint build.sh --fix

# Strict linting with JSON output
bashrs lint scripts/ --strict --format json --output lint-report.json
```

**Exit Codes:**
- `0` - No issues found
- `1` - Warnings found
- `2` - Errors found
- `3` - File I/O error

---

### bashrs check

Type-check and validate Rust code before transpilation.

```sh
bashrs check [OPTIONS] <FILES...>
```

**Arguments:**
- `<FILES...>` - Rust files to check

**Options:**
- `--validation <LEVEL>` - Validation level (default: `strict`)
- `--strict` - Treat warnings as errors
- `--help` - Display help information

**Examples:**
```sh
# Check single file
bashrs check src/main.rs

# Check all files
bashrs check src/*.rs --strict
```

**Exit Codes:**
- `0` - All checks passed
- `1` - Type errors found
- `2` - Validation errors found

---

### bashrs bench

Benchmark performance and memory usage of generated scripts.

```sh
bashrs bench [OPTIONS] <FILES...>
```

**Arguments:**
- `<FILES...>` - Shell scripts to benchmark

**Options:**
- `--iterations <N>` - Number of iterations (default: 100)
- `--measure-memory` - Measure RSS memory usage
- `--format <FORMAT>` - Output format: `text`, `json`, `csv` (default: `text`)
- `--output <FILE>` - Write results to file
- `--help` - Display help information

**Examples:**
```sh
# Basic benchmark
bashrs bench dist/install.sh

# Benchmark with memory profiling
bashrs bench dist/*.sh --measure-memory --iterations 1000

# Export results to JSON
bashrs bench build.sh --format json --output bench-results.json
```

**Exit Codes:**
- `0` - Benchmark completed
- `1` - Benchmark error
- `3` - File I/O error

---

### bashrs mcp

Start MCP (Model Context Protocol) server for AI integration.

```sh
bashrs mcp serve [OPTIONS]
```

**Options:**
- `--port <PORT>` - Server port (default: 3000)
- `--host <HOST>` - Server host (default: 127.0.0.1)
- `--config <FILE>` - Configuration file (JSON)
- `--help` - Display help information
- `--version` - Display MCP server version

**Examples:**
```sh
# Start server on default port
bashrs mcp serve

# Start with custom port and config
bashrs mcp serve --port 8080 --config /etc/bashrs/mcp-config.json

# Check version
bashrs mcp --version
```

**Exit Codes:**
- `0` - Server stopped gracefully
- `1` - Server error
- `3` - Configuration error

---

## Configuration File

bashrs can be configured via `bashrs.toml` in the project root or `~/.config/bashrs/config.toml` for global settings.

### Configuration Schema

```toml
# bashrs.toml

[validation]
# Validation level: "none", "minimal", "strict", "paranoid"
level = "strict"

# Treat warnings as errors
strict = true

# Skip shellcheck verification
no_verify = false

[output]
# Output directory for generated scripts
dir = "dist/"

# Output format: "shell", "json", "yaml"
format = "shell"

# Target shell: "sh", "bash", "dash", "zsh"
target = "sh"

[linting]
# Enable auto-fix
auto_fix = false

# Auto-fix all issues without prompts
auto_fix_all = false

# Lint output format: "text", "json", "checkstyle", "gcc"
format = "text"

[benchmarks]
# Number of benchmark iterations
iterations = 100

# Measure memory usage
measure_memory = true

# Benchmark output format: "text", "json", "csv"
format = "text"

[mcp]
# MCP server port
port = 3000

# MCP server host
host = "127.0.0.1"

# Max script size for MCP requests (bytes)
max_script_size = 1048576

# Request timeout (seconds)
timeout = 30

[logging]
# Log level: "trace", "debug", "info", "warn", "error"
level = "info"

# Log file path
file = "/var/log/bashrs.log"
```

**Configuration Precedence** (highest to lowest):
1. Command-line flags
2. Project-local `bashrs.toml`
3. Global `~/.config/bashrs/config.toml`
4. Built-in defaults

---

## Validation Levels

### None (level = "none")

**Rules**: 0
**Speed**: Fastest
**Use Case**: Prototyping, testing, non-production

No validation checks. Generates scripts without any safety verification.

**Risks**:
- Injection vulnerabilities
- Non-deterministic behavior
- POSIX violations
- Non-idempotent operations

---

### Minimal (level = "minimal") [DEFAULT]

**Rules**: 8 critical rules
**Speed**: Fast (95% of "none" speed)
**Use Case**: Development, CI/CD pipelines

Essential safety checks for production scripts.

**Rules**:
- `SC2086` - Unquoted variables (injection risk)
- `SC2046` - Unquoted command substitution
- `SC2154` - Undefined variables
- `SEC001` - Dangerous rm -rf with unquoted vars
- `SEC002` - curl|sh pattern (insecure download+execute)
- `DET001` - Non-deterministic $RANDOM usage
- `IDEM001` - Non-idempotent mkdir (missing -p)
- `IDEM002` - Non-idempotent rm (missing -f)

---

### Strict (level = "strict") [RECOMMENDED FOR PRODUCTION]

**Rules**: 18 rules (Minimal + 10 additional)
**Speed**: Medium (85% of "none" speed)
**Use Case**: Production deployments, critical systems

Comprehensive production-ready validation.

**Additional Rules** (beyond Minimal):
- `SC2004` - $/${} unnecessary on arithmetic variables
- `SC2006` - Use $() instead of backticks
- `SC2034` - Unused variables
- `SC2045` - Iterating over ls output
- `SC2068` - Unquoted $@
- `SC2116` - Useless echo
- `SC2196` - egrep deprecated, use grep -E
- `SC2197` - fgrep deprecated, use grep -F
- `SEC003` - eval with user input
- `POSIX001` - Bash-specific [[ ]] syntax

---

### Paranoid (level = "paranoid")

**Rules**: 30+ rules (Strict + 12+ additional)
**Speed**: Slow (70% of "none" speed)
**Use Case**: Security-critical, compliance-required, finance/healthcare

Maximum safety with exhaustive checks.

**Additional Rules** (beyond Strict):
- `SC2015` - [[ && || ]] ordering issues
- `SC2053` - Quote right-hand side of =
- `SC2066` - For loop over a single item
- `SC2076` - Remove quotes from right-hand side of =~
- `SC2089` - Quotes/backslashes in assignments
- `SC2090` - Quotes/backslashes in expansions
- `SEC004` - SSH command injection
- `SEC005` - SQL injection patterns
- `SEC006` - Path traversal patterns
- `SEC007` - Command injection via $()
- `SEC008` - Unvalidated redirects
- `POSIX002` - Arrays (bash-only)

---

## Environment Variables

bashrs recognizes these environment variables:

### BASHRS_VALIDATION
Override default validation level.

```sh
export BASHRS_VALIDATION=strict
bashrs build src/main.rs
```

**Values**: `none`, `minimal`, `strict`, `paranoid`

---

### BASHRS_STRICT_MODE
Enable strict mode (warnings as errors).

```sh
export BASHRS_STRICT_MODE=true
bashrs lint scripts/*.sh
```

**Values**: `true`, `false` (default: `false`)

---

### BASHRS_OUTPUT_DIR
Override output directory.

```sh
export BASHRS_OUTPUT_DIR=/opt/scripts/dist
bashrs build src/*.rs
```

**Default**: `./dist`

---

### BASHRS_LOG_LEVEL
Set logging verbosity.

```sh
export BASHRS_LOG_LEVEL=debug
bashrs build src/main.rs
```

**Values**: `trace`, `debug`, `info`, `warn`, `error` (default: `info`)

---

### BASHRS_NO_COLOR
Disable colored output.

```sh
export BASHRS_NO_COLOR=1
bashrs lint script.sh
```

**Values**: `1` (disable), `0` (enable)

---

### BASHRS_CONFIG
Override configuration file path.

```sh
export BASHRS_CONFIG=/etc/bashrs/custom-config.toml
bashrs build src/*.rs
```

**Default**: `./bashrs.toml` or `~/.config/bashrs/config.toml`

---

## Exit Codes

bashrs uses standard Unix exit codes:

| Code | Meaning | Description |
|------|---------|-------------|
| `0` | Success | Operation completed successfully |
| `1` | General error | Compilation, parse, or lint errors |
| `2` | Validation error | Validation checks failed (strict mode) |
| `3` | I/O error | File read/write failed |
| `4` | Configuration error | Invalid config file or options |
| `64` | Usage error | Invalid command-line arguments |
| `130` | Interrupted | User cancelled with Ctrl+C |

**Examples:**
```sh
# Check exit code
bashrs lint script.sh
if [ $? -ne 0 ]; then
    echo "Linting failed"
    exit 1
fi

# Use in CI/CD
bashrs build src/*.rs --strict || exit 1
```

---

## Output Formats

### Text Format (default)

Human-readable output with colors and formatting.

```sh
$ bashrs lint script.sh

Found 2 issues:

1. SC2086 (error): Unquoted variable 'directory'
   Line 5: rm -rf $directory
   Fix: rm -rf "${directory}"

2. SEC001 (error): Dangerous rm -rf with unquoted variable
   Line 5: rm -rf $directory
   Fix: Add validation before destructive operations

Summary: 2 errors, 0 warnings
```

---

### JSON Format

Machine-readable JSON for automation.

```sh
$ bashrs lint script.sh --format json
```

```json
{
  "files": [
    {
      "path": "script.sh",
      "issues": [
        {
          "code": "SC2086",
          "severity": "error",
          "line": 5,
          "column": 8,
          "message": "Unquoted variable 'directory'",
          "fix": "rm -rf \"${directory}\""
        },
        {
          "code": "SEC001",
          "severity": "error",
          "line": 5,
          "column": 1,
          "message": "Dangerous rm -rf with unquoted variable",
          "fix": "Add validation before destructive operations"
        }
      ]
    }
  ],
  "summary": {
    "errors": 2,
    "warnings": 0,
    "fixed": 0
  }
}
```

---

### Checkstyle Format (XML)

Compatible with Checkstyle-based tools (Jenkins, SonarQube).

```sh
$ bashrs lint script.sh --format checkstyle
```

```xml
<?xml version="1.0" encoding="UTF-8"?>
<checkstyle version="4.3">
  <file name="script.sh">
    <error line="5" column="8" severity="error"
           message="Unquoted variable 'directory'"
           source="SC2086" />
    <error line="5" column="1" severity="error"
           message="Dangerous rm -rf with unquoted variable"
           source="SEC001" />
  </file>
</checkstyle>
```

---

### GCC Format

Compatible with GCC-style error messages (Emacs, Vim).

```sh
$ bashrs lint script.sh --format gcc
```

```text
script.sh:5:8: error: Unquoted variable 'directory' [SC2086]
script.sh:5:1: error: Dangerous rm -rf with unquoted variable [SEC001]
```

---

## Rule Reference

### SC2086 - Unquoted Variables

**Severity**: Error
**Category**: Security (Injection)

**Problem**: Variables without quotes can cause word splitting and injection.

**Example**:
```sh
# ❌ Bad
rm -rf $directory

# ✅ Good
rm -rf "${directory}"
```

**Auto-fix**: Adds quotes around variables

---

### SC2046 - Unquoted Command Substitution

**Severity**: Error
**Category**: Security (Injection)

**Problem**: Unquoted command substitution can cause word splitting.

**Example**:
```sh
# ❌ Bad
files=$(ls *.txt)
rm $files

# ✅ Good
files=$(ls *.txt)
rm "${files}"
```

**Auto-fix**: Adds quotes around command substitution

---

### SC2154 - Undefined Variables

**Severity**: Warning
**Category**: Correctness

**Problem**: Variable used but never assigned.

**Example**:
```sh
# ❌ Bad
echo "${undefined_var}"

# ✅ Good
undefined_var="value"
echo "${undefined_var}"
```

**Auto-fix**: Not available (manual fix required)

---

### SEC001 - Dangerous rm -rf

**Severity**: Error
**Category**: Security (Destructive)

**Problem**: `rm -rf` with unquoted variables can delete unexpected files.

**Example**:
```sh
# ❌ Bad
rm -rf $directory

# ✅ Good
if [ -d "${directory}" ]; then
    rm -rf "${directory}"
fi
```

**Auto-fix**: Adds quotes and validation

---

### DET001 - Non-deterministic $RANDOM

**Severity**: Warning
**Category**: Determinism

**Problem**: `$RANDOM` produces different values on each run.

**Example**:
```sh
# ❌ Bad
id=$RANDOM

# ✅ Good
id=$(od -An -N4 -tu4 /dev/urandom | tr -d ' ')
```

**Auto-fix**: Replaces with `/dev/urandom`

---

### IDEM001 - Non-idempotent mkdir

**Severity**: Warning
**Category**: Idempotency

**Problem**: `mkdir` without `-p` fails if directory exists.

**Example**:
```sh
# ❌ Bad
mkdir /tmp/build

# ✅ Good
mkdir -p /tmp/build
```

**Auto-fix**: Adds `-p` flag

---

## Performance Benchmarks

bashrs performance characteristics:

| Operation | Time | Memory | Notes |
|-----------|------|--------|-------|
| Parse 1KB bash | <10ms | <1MB | Typical script |
| Parse 100KB bash | <100ms | <5MB | Large script |
| Lint 1KB (minimal) | <20ms | <2MB | 8 rules |
| Lint 1KB (strict) | <50ms | <3MB | 18 rules |
| Lint 1KB (paranoid) | <100ms | <5MB | 30+ rules |
| Purify 1KB | <30ms | <2MB | Determinism + idempotency |
| Transpile 100 LOC Rust | <200ms | <10MB | Typical program |

**Scaling**:
- Linear scaling with file size
- Memory usage: O(n) where n = file size
- Parallelization: Processes multiple files concurrently

---

## Integration Examples

### GitHub Actions

```yaml
# .github/workflows/bashrs.yml
name: Shell Script Quality

on: [push, pull_request]

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install bashrs
        run: cargo install bashrs

      - name: Lint shell scripts
        run: bashrs lint scripts/*.sh --strict --format checkstyle --output checkstyle.xml

      - name: Upload results
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: lint-results
          path: checkstyle.xml
```

---

### GitLab CI

```yaml
# .gitlab-ci.yml
bashrs:lint:
  image: rust:latest
  stage: test
  script:
    - cargo install bashrs
    - bashrs lint scripts/*.sh --strict --format json --output lint-report.json
  artifacts:
    reports:
      codequality: lint-report.json
    paths:
      - lint-report.json
    expire_in: 1 week
```

---

### Pre-commit Hook

```yaml
# .pre-commit-config.yaml
repos:
  - repo: local
    hooks:
      - id: bashrs-lint
        name: bashrs lint
        entry: bashrs lint
        language: system
        types: [shell]
        args: [--strict]
```

---

### Makefile Integration

```makefile
# Makefile
.PHONY: lint
lint:
	bashrs lint scripts/*.sh --strict

.PHONY: build
build:
	bashrs build src/*.rs --validation strict --output-dir dist/

.PHONY: bench
bench:
	bashrs bench dist/*.sh --measure-memory --iterations 1000

.PHONY: ci
ci: lint build
	@echo "All checks passed"
```

---

## Troubleshooting

### Common Issues

#### Issue: "Command not found: bashrs"
**Solution**: Install bashrs with `cargo install bashrs` or add to PATH

#### Issue: "Parse error: unexpected token"
**Solution**: Check syntax with `bash -n script.sh`, fix syntax errors

#### Issue: "Validation failed: SC2086"
**Solution**: Quote variables: `"${var}"` instead of `$var`

#### Issue: "Permission denied: /etc/bashrs/config.toml"
**Solution**: Use user config: `~/.config/bashrs/config.toml` or run with sudo

#### Issue: "MCP server not starting"
**Solution**: Check port availability with `netstat -tlnp | grep 3000`

---

## Version Compatibility

### bashrs Versions

| Version | Release Date | Status | Notes |
|---------|--------------|--------|-------|
| v6.34.1 | 2025-11-14 | Current | False positive fixes |
| v6.34.0 | 2025-11-13 | Stable | Issue #1 auto-fix bug fix |
| v6.26.0 | 2025-10-15 | Stable | Memory profiling |
| v6.0.0 | 2025-09-01 | Stable | Validation levels |
| v1.4.0 | 2025-04-15 | Stable | Makefile purification |
| v1.0.0 | 2025-04-01 | Stable | Initial release |

### Shell Compatibility

bashrs generates scripts compatible with:

| Shell | Version | Status |
|-------|---------|--------|
| sh | POSIX | ✅ Full |
| dash | 0.5.12+ | ✅ Full |
| ash (BusyBox) | 1.35+ | ✅ Full |
| bash | 3.2+ | ✅ Full |
| zsh | 5.x | ✅ Full |
| ksh | 93u+ | ✅ Full |

---

## Support and Resources

### Documentation
- **Book**: https://bashrs.com/book
- **API Reference**: https://docs.rs/bashrs
- **Examples**: https://github.com/paiml/bashrs/tree/main/examples

### Community
- **GitHub Issues**: https://github.com/paiml/bashrs/issues
- **Discussions**: https://github.com/paiml/bashrs/discussions
- **Discord**: https://discord.gg/bashrs

### Contributing
- **Contributing Guide**: https://github.com/paiml/bashrs/blob/main/CONTRIBUTING.md
- **Development Setup**: See CLAUDE.md in repository
- **Code of Conduct**: https://github.com/paiml/bashrs/blob/main/CODE_OF_CONDUCT.md

---

## See Also

- **Chapter 1**: Getting started tutorial
- **Chapter 13**: Validation levels explained
- **Chapter 15**: CI/CD integration patterns
- **Chapter 16**: MCP server integration
- **Appendix B**: Glossary of terms
- **Appendix C**: Shell compatibility matrix

---

*API Reference last updated: 2025-11-14 for bashrs v6.34.1*
