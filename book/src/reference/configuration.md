# Configuration

This chapter provides a complete reference for configuring bashrs v6.32.1 using configuration files, environment variables, and CLI options.

## Table of Contents

- [Configuration File Format](#configuration-file-format)
- [Configuration Options](#configuration-options)
- [Configuration Locations](#configuration-locations)
- [Environment Variables](#environment-variables)
- [Configuration Precedence](#configuration-precedence)
- [Per-Project Configuration](#per-project-configuration)
- [Global Configuration](#global-configuration)
- [Examples](#examples)
- [Best Practices](#best-practices)

## Configuration File Format

bashrs uses TOML format for configuration files. The default configuration file name is `bashrs.toml`.

### Basic Structure

```toml
[bashrs]
target = "posix"
verify = "strict"
optimize = true
emit_proof = false
strict_mode = false
validation_level = "minimal"
```

### Complete Schema

```toml
[bashrs]
# Target shell dialect for generated scripts
# Options: "posix", "bash", "dash", "ash"
# Default: "posix"
target = "posix"

# Verification level for transpilation and purification
# Options: "none", "basic", "strict", "paranoid"
# Default: "strict"
verify = "strict"

# Enable IR optimization passes
# Default: true
optimize = true

# Emit formal verification proofs
# Default: false
emit_proof = false

# Enable strict POSIX mode (no extensions)
# Default: false
strict_mode = false

# ShellCheck validation level
# Options: "none", "minimal", "strict", "paranoid"
# Default: "minimal"
validation_level = "minimal"

[linter]
# Enable/disable specific rule categories
security = true
determinism = true
idempotency = true
shellcheck = true
makefile = true
config = true

# Disable specific rules
disabled_rules = ["SC2119", "SC2120"]

# Enable auto-fix for safe rules
auto_fix = true

# Only apply auto-fixes marked as safe
safe_auto_fix_only = true

[formatter]
# Enable code formatting
enabled = true

# Indent size (spaces)
indent = 4

# Maximum line length
max_line_length = 100

# Use tabs instead of spaces
use_tabs = false

[output]
# Output format for diagnostics
# Options: "human", "json", "sarif", "checkstyle"
format = "human"

# Show rule documentation URLs in diagnostics
show_docs = true

# Colorize output
color = "auto"  # Options: "auto", "always", "never"
```

## Configuration Options

### Target Shell Dialect (`target`)

Determines which shell-specific features are available and how output is optimized.

- **`posix`** (default): Maximum compatibility, POSIX-only features
- **`bash`**: Bash 3.2+ features (arrays, `[[`, etc.)
- **`dash`**: Debian Almquist Shell optimizations
- **`ash`**: BusyBox Almquist Shell optimizations

**Example:**
```toml
[bashrs]
target = "bash"  # Enable bash-specific optimizations
```

**CLI Override:**
```bash
bashrs purify --target bash script.sh
```

### Verification Level (`verify`)

Controls the strictness of safety checks during transpilation and purification.

- **`none`**: No verification (not recommended)
- **`basic`**: Essential safety checks only (fast)
- **`strict`**: Recommended for production (balanced)
- **`paranoid`**: Maximum verification (slowest, most thorough)

**Example:**
```toml
[bashrs]
verify = "paranoid"  # Maximum safety checks
```

**CLI Override:**
```bash
bashrs purify --verify paranoid script.sh
```

**Verification Levels Comparison:**

| Level | Speed | Checks | Use Case |
|-------|-------|--------|----------|
| none | Fastest | None | Development only |
| basic | Fast | Essential | Quick iterations |
| strict | Medium | Recommended | Production default |
| paranoid | Slow | Maximum | Critical systems |

### Optimization (`optimize`)

Enables or disables IR (Intermediate Representation) optimization passes.

- **`true`** (default): Enable optimizations
- **`false`**: Disable optimizations (preserve exact structure)

**Example:**
```toml
[bashrs]
optimize = false  # Preserve exact script structure
```

**Optimization Passes:**
- Dead code elimination
- Constant folding
- Loop unrolling (when safe)
- Variable inlining

### Emit Proof (`emit_proof`)

Controls whether formal verification proofs are emitted.

- **`false`** (default): No proofs
- **`true`**: Emit verification proofs

**Example:**
```toml
[bashrs]
emit_proof = true  # Emit formal proofs for critical scripts
```

**Proof Output:**
```text
$ bashrs purify --emit-proof deploy.sh
Verification Proof:
  Determinism: PROVEN
  Idempotency: PROVEN
  POSIX Compliance: VERIFIED
```

### Strict Mode (`strict_mode`)

Enforces strict POSIX compliance with no shell extensions.

- **`false`** (default): Allow common extensions
- **`true`**: Pure POSIX only

**Example:**
```toml
[bashrs]
strict_mode = true  # Pure POSIX, no extensions
```

**Impact:**
- Rejects bash arrays
- Rejects `[[` test syntax
- Rejects `function` keyword
- Enforces `#!/bin/sh` shebang

### Validation Level (`validation_level`)

Controls ShellCheck validation strictness.

- **`none`**: Skip validation
- **`minimal`** (default): Basic validation
- **`strict`**: Comprehensive validation
- **`paranoid`**: Maximum validation

**Example:**
```toml
[bashrs]
validation_level = "strict"  # Comprehensive ShellCheck validation
```

## Configuration Locations

bashrs searches for configuration files in the following order:

### 1. Per-Project Configuration

**Location:** `./bashrs.toml` (current directory)

**Use Case:** Project-specific settings

**Example:**
```bash
$ cd /path/to/project
$ cat bashrs.toml
[bashrs]
target = "bash"
verify = "strict"
```

### 2. Parent Directory Configuration

**Location:** `..bashrs.toml` (parent directories, up to root)

**Use Case:** Repository-wide settings

**Example:**
```bash
# In /home/user/project/src/
$ bashrs purify script.sh
# Searches: ./bashrs.toml, ../bashrs.toml, ../../bashrs.toml, etc.
```

### 3. Global User Configuration

**Location:** `~/.config/bashrs/config.toml`

**Use Case:** User-wide preferences

**Example:**
```bash
$ cat ~/.config/bashrs/config.toml
[bashrs]
verify = "paranoid"
validation_level = "strict"

[output]
color = "always"
```

### 4. System-Wide Configuration

**Location:** `/etc/bashrs/config.toml`

**Use Case:** System administrator defaults

**Example:**
```bash
$ sudo cat /etc/bashrs/config.toml
[bashrs]
target = "posix"
strict_mode = true
```

## Environment Variables

Environment variables provide runtime configuration overrides.

### Core Environment Variables

#### `BASHRS_CONFIG`

Override the configuration file location.

```bash
export BASHRS_CONFIG=/path/to/custom.toml
bashrs purify script.sh
```

#### `BASHRS_VERIFICATION_LEVEL`

Override verification level at runtime.

```bash
export BASHRS_VERIFICATION_LEVEL=paranoid
bashrs purify deploy.sh
```

**Values:** `none`, `basic`, `strict`, `paranoid`

#### `BASHRS_TARGET`

Override target shell dialect.

```bash
export BASHRS_TARGET=bash
bashrs purify script.sh
```

**Values:** `posix`, `bash`, `dash`, `ash`

#### `BASHRS_NO_COLOR`

Disable colored output.

```bash
export BASHRS_NO_COLOR=1
bashrs lint script.sh
```

#### `BASHRS_DEBUG`

Enable debug output and error traces.

```bash
export BASHRS_DEBUG=1
bashrs purify script.sh
```

### Validation Environment Variables

#### `BASHRS_VALIDATION_LEVEL`

Override ShellCheck validation level.

```bash
export BASHRS_VALIDATION_LEVEL=strict
bashrs lint script.sh
```

#### `BASHRS_DISABLE_RULES`

Disable specific linter rules.

```bash
export BASHRS_DISABLE_RULES="SC2119,SC2120,DET002"
bashrs lint script.sh
```

#### `BASHRS_AUTO_FIX`

Enable automatic fixes.

```bash
export BASHRS_AUTO_FIX=1
bashrs lint --fix script.sh
```

## Configuration Precedence

Configuration sources are applied in this order (later sources override earlier ones):

1. **System configuration** (`/etc/bashrs/config.toml`)
2. **Global user configuration** (`~/.config/bashrs/config.toml`)
3. **Parent directory configuration** (`../bashrs.toml`, up to root)
4. **Per-project configuration** (`./bashrs.toml`)
5. **Environment variables** (`BASHRS_*`)
6. **CLI arguments** (`--target`, `--verify`, etc.)

### Example Precedence

```bash
# /etc/bashrs/config.toml
[bashrs]
target = "posix"
verify = "basic"

# ~/.config/bashrs/config.toml
[bashrs]
verify = "strict"  # Overrides system 'basic'

# ./bashrs.toml
[bashrs]
target = "bash"  # Overrides system 'posix'

# Environment
export BASHRS_VERIFICATION_LEVEL=paranoid  # Overrides user 'strict'

# CLI
bashrs purify --target dash script.sh  # Overrides project 'bash'

# Final configuration:
# target = "dash" (from CLI)
# verify = "paranoid" (from environment)
```

## Per-Project Configuration

Per-project configuration allows team-wide consistency.

### Example: Web Application Project

```toml
# bashrs.toml
[bashrs]
target = "bash"
verify = "strict"
optimize = true

[linter]
security = true
determinism = true
idempotency = true

# Disable noisy rules for this project
disabled_rules = ["SC2034"]  # Allow unused variables

[output]
format = "json"  # For CI/CD integration
```

### Example: Embedded System Project

```toml
# bashrs.toml
[bashrs]
target = "ash"  # BusyBox Almquist Shell
verify = "paranoid"
strict_mode = true  # Pure POSIX only

[linter]
security = true
determinism = true
idempotency = true

[output]
format = "checkstyle"  # For Jenkins integration
```

### Example: DevOps Scripts

```toml
# bashrs.toml
[bashrs]
target = "posix"
verify = "strict"
optimize = true

[linter]
security = true
determinism = true
idempotency = true
auto_fix = true
safe_auto_fix_only = true

[formatter]
enabled = true
indent = 2
max_line_length = 120
```

## Global Configuration

Global configuration provides user-wide defaults.

### Example: Developer Preferences

```toml
# ~/.config/bashrs/config.toml
[bashrs]
verify = "strict"
validation_level = "strict"

[output]
color = "always"
show_docs = true

[linter]
auto_fix = true
safe_auto_fix_only = true
```

### Example: CI/CD Environment

```toml
# ~/.config/bashrs/config.toml
[bashrs]
verify = "paranoid"
validation_level = "paranoid"

[output]
format = "json"
color = "never"

[linter]
security = true
determinism = true
idempotency = true
auto_fix = false  # Never auto-fix in CI
```

## Examples

### Example 1: Maximum Safety Configuration

For critical production scripts:

```toml
# bashrs.toml
[bashrs]
target = "posix"
verify = "paranoid"
strict_mode = true
validation_level = "paranoid"
emit_proof = true

[linter]
security = true
determinism = true
idempotency = true
auto_fix = false  # Manual review required

[output]
format = "human"
show_docs = true
```

Usage:
```bash
$ bashrs purify deploy.sh
Verification Proof:
  Determinism: PROVEN
  Idempotency: PROVEN
  POSIX Compliance: VERIFIED
  Security: VALIDATED
```

### Example 2: Fast Development Configuration

For rapid iteration:

```toml
# bashrs.toml
[bashrs]
target = "bash"
verify = "basic"
optimize = false
validation_level = "minimal"

[linter]
security = true
auto_fix = true

[output]
format = "human"
color = "always"
```

### Example 3: Team-Wide Consistency

For repository-wide standards:

```toml
# bashrs.toml (at repository root)
[bashrs]
target = "bash"
verify = "strict"
optimize = true

[linter]
security = true
determinism = true
idempotency = true
disabled_rules = ["SC2034", "SC2154"]

[formatter]
enabled = true
indent = 4
max_line_length = 100
use_tabs = false

[output]
format = "human"
show_docs = true
```

### Example 4: CI/CD Integration

For automated quality gates:

```toml
# bashrs.toml
[bashrs]
target = "posix"
verify = "strict"
validation_level = "strict"

[linter]
security = true
determinism = true
idempotency = true

[output]
format = "json"  # Machine-readable for CI
color = "never"
```

CI Script:
```bash
#!/bin/bash
bashrs lint --config bashrs.toml scripts/*.sh > lint-results.json
if [ $? -ne 0 ]; then
    echo "Linting failed"
    exit 1
fi
```

## Best Practices

### 1. Use Per-Project Configuration

Always include `bashrs.toml` in your repository:

```toml
[bashrs]
target = "bash"  # Or "posix" for maximum portability
verify = "strict"

[linter]
security = true
determinism = true
idempotency = true
```

**Benefits:**
- Team-wide consistency
- Reproducible builds
- Clear project standards

### 2. Set Appropriate Verification Levels

**Development:**
```toml
[bashrs]
verify = "basic"  # Fast iteration
```

**Production:**
```toml
[bashrs]
verify = "strict"  # Balanced safety
```

**Critical Systems:**
```toml
[bashrs]
verify = "paranoid"  # Maximum safety
```

### 3. Enable Security Rules

Always enable security, determinism, and idempotency:

```toml
[linter]
security = true
determinism = true
idempotency = true
```

### 4. Use Auto-Fix Safely

Enable auto-fix with safety checks:

```toml
[linter]
auto_fix = true
safe_auto_fix_only = true  # Only apply safe fixes
```

### 5. Configure Output for CI/CD

Use machine-readable formats in automation:

```toml
[output]
format = "json"  # For programmatic parsing
color = "never"  # Disable colors in CI logs
```

### 6. Version Control Configuration

**Always commit:**
- `bashrs.toml` (project config)

**Never commit:**
- `~/.config/bashrs/config.toml` (user preferences)

### 7. Document Project-Specific Rules

If disabling rules, document why:

```toml
[linter]
# Disable SC2034 because we use variables in sourced files
disabled_rules = ["SC2034"]
```

### 8. Use Environment Variables for Runtime Overrides

Avoid modifying config files for temporary changes:

```bash
# Good: Temporary override
BASHRS_VERIFICATION_LEVEL=paranoid bashrs purify critical.sh

# Bad: Editing config file for one-time use
```

### 9. Separate Development and Production Configs

**Development** (`bashrs.dev.toml`):
```toml
[bashrs]
verify = "basic"
validation_level = "minimal"
```

**Production** (`bashrs.prod.toml`):
```toml
[bashrs]
verify = "paranoid"
validation_level = "paranoid"
```

Usage:
```bash
# Development
bashrs purify --config bashrs.dev.toml script.sh

# Production
bashrs purify --config bashrs.prod.toml script.sh
```

### 10. Test Configuration Changes

After modifying configuration, verify it works:

```bash
# Validate configuration
bashrs config validate bashrs.toml

# Test on sample script
bashrs purify --config bashrs.toml test-script.sh
```

## Troubleshooting Configuration

### Configuration Not Loading

**Problem:** Changes to `bashrs.toml` have no effect.

**Solutions:**
1. Check file location (must be in current directory)
2. Verify TOML syntax with a validator
3. Use `BASHRS_DEBUG=1` to see configuration loading

```bash
BASHRS_DEBUG=1 bashrs purify script.sh
# Debug output will show which config files are loaded
```

### Conflicting Settings

**Problem:** Unexpected configuration behavior.

**Solution:** Check configuration precedence (CLI > ENV > Project > User > System)

```bash
# See effective configuration
bashrs config show
```

### Invalid Configuration Values

**Problem:** Error messages about invalid config values.

**Solution:** Verify against schema (see [Complete Schema](#complete-schema))

```bash
# Validate configuration file
bashrs config validate bashrs.toml
```

## Summary

bashrs provides flexible configuration through:

- **Configuration files** (`bashrs.toml`) for persistent settings
- **Environment variables** (`BASHRS_*`) for runtime overrides
- **CLI arguments** for command-specific options

**Key Points:**
1. Use per-project `bashrs.toml` for team consistency
2. Set appropriate verification levels for your use case
3. Always enable security, determinism, and idempotency checks
4. Use machine-readable formats in CI/CD
5. Document project-specific rule exceptions

For more information, see:
- [Exit Codes Reference](./exit-codes.md)
- [Linter Rules Reference](./rules.md)
- [CLI Commands Reference](./cli.md)
