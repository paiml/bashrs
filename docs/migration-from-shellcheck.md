# Migrating from ShellCheck to bashrs

This guide helps you migrate from ShellCheck to bashrs. bashrs is a drop-in
replacement that includes all ShellCheck rules plus additional capabilities.

## Quick Start

```bash
# Install bashrs
cargo install bashrs

# Replace shellcheck with bashrs
# Before: shellcheck script.sh
# After:
bashrs lint script.sh
```

## Rule Compatibility

bashrs implements **388 ShellCheck rules** (SC1000–SC2300+), covering 100% of
ShellCheck's rule set. Rules use the same SC identifiers, so existing
`# shellcheck disable=SC2034` comments work unchanged.

bashrs adds **90+ additional rules** that ShellCheck does not have:

| Rule Prefix | Category | Count | Example |
|-------------|----------|-------|---------|
| `SEC001–008` | Security | 8 | Command injection, path traversal |
| `DET001–006` | Determinism | 6 | `$RANDOM`, timestamps, `$$` |
| `IDEM001–006` | Idempotency | 6 | `mkdir` without `-p`, `rm` without `-f` |
| `REL001–003` | Reliability | 3 | Missing error handling |
| `PERF001–005` | Performance | 5 | Useless use of cat |
| `MAKE001–020` | Makefile | 20 | Makefile-specific linting |
| `DOCKER001–020` | Dockerfile | 20 | Dockerfile best practices |
| `BASH001–010` | Bash-specific | 10 | Bash portability issues |

## Configuration Migration

### ShellCheck Configuration

ShellCheck uses `.shellcheckrc` or command-line flags:

```bash
# .shellcheckrc
disable=SC2034,SC2086
shell=bash
```

### bashrs Configuration

bashrs uses `.bashrsrc.toml` or command-line flags:

```toml
# .bashrsrc.toml
[lint]
disable = ["SC2034", "SC2086"]
shell = "bash"
```

**Inline directives work the same way:**

```bash
# ShellCheck:
# shellcheck disable=SC2034
unused_var="hello"

# bashrs (same syntax — fully compatible):
# shellcheck disable=SC2034
unused_var="hello"
```

### Ignore Files

ShellCheck has no built-in ignore file. bashrs supports `.bashrsignore`:

```
# .bashrsignore
vendor/
node_modules/
*.generated.sh
```

## CLI Flag Mapping

| ShellCheck | bashrs | Notes |
|------------|--------|-------|
| `shellcheck script.sh` | `bashrs lint script.sh` | |
| `shellcheck -f json` | `bashrs lint --format json` | Also supports `sarif` |
| `shellcheck -s bash` | `bashrs lint --shell bash` | Auto-detected from shebang |
| `shellcheck -e SC2034` | `bashrs lint --disable SC2034` | |
| `shellcheck -x` | (default) | External sources followed by default |
| `shellcheck --severity=warning` | `bashrs lint --level warning` | |
| N/A | `bashrs lint --fix` | Auto-fix (bashrs exclusive) |
| N/A | `bashrs lint --changed` | Git-aware incremental lint |
| N/A | `bashrs lint --ci` | GitHub Actions annotations |

## CI Migration

### GitHub Actions

**Before (ShellCheck):**
```yaml
- name: ShellCheck
  uses: ludeeus/action-shellcheck@master
  with:
    scandir: scripts/
```

**After (bashrs):**
```yaml
- name: bashrs lint
  uses: paiml/bashrs@main
  with:
    command: lint
    files: scripts/
    upload-sarif: 'true'  # GitHub Security tab integration
```

### GitLab CI

**Before:**
```yaml
shellcheck:
  image: koalaman/shellcheck-alpine
  script:
    - shellcheck scripts/*.sh
```

**After:**
```yaml
bashrs:
  image: rust:latest
  before_script:
    - cargo install bashrs
  script:
    - bashrs lint scripts/*.sh --ci
```

## Feature Comparison

| Feature | ShellCheck | bashrs |
|---------|-----------|--------|
| Shell script linting | ✅ ~400 rules | ✅ 388 SC + 90 custom |
| Makefile linting | ❌ | ✅ 20 rules |
| Dockerfile linting | ❌ | ✅ 20 rules |
| Auto-fix | ❌ | ✅ |
| Test generation | ❌ | ✅ `--generate` BATS stubs |
| Test runner | ❌ | ✅ `bashrs test` |
| Coverage analysis | ❌ | ✅ Line + branch |
| Property testing | ❌ | ✅ 4 built-in properties |
| Mutation testing | ❌ | ✅ Bash-specific operators |
| Quality scoring | ❌ | ✅ A+ to F grading |
| Quality gates | ❌ | ✅ 3 tiers |
| SARIF output | ❌ | ✅ GitHub Security tab |
| Incremental lint | ❌ | ✅ Git-aware `--changed` |
| Watch mode | ❌ | ✅ `bashrs watch` |
| REPL | ❌ | ✅ Interactive debugger |
| LSP | ❌ | ✅ VS Code integration |
| Formatting | ❌ | ✅ 4 presets |
| Profiles | ❌ | ✅ standard, coursera, devcontainer |

## Common Workflows

### Basic Linting (Drop-in Replacement)

```bash
# Identical behavior to shellcheck
bashrs lint *.sh
```

### Full Quality Check (bashrs Exclusive)

```bash
# Lint + score + property test in one pipeline
bashrs lint scripts/ && \
bashrs score scripts/deploy.sh --grade B && \
bashrs property scripts/deploy.sh
```

### Pre-commit Hook

```bash
#!/bin/sh
# .git/hooks/pre-commit
bashrs lint --changed --fail-on warning
```

### Generate Tests for Existing Scripts

```bash
# Auto-generate BATS test stubs
bashrs test --generate --output tests/ scripts/deploy.sh

# Run generated tests
bats tests/deploy_test.bats
```

## Troubleshooting

### "Rule SCxxxx not recognized"

bashrs implements SC1000–SC2300+. If you're using a very new ShellCheck rule,
check `bashrs lint --list-rules` for availability.

### Different output format

bashrs uses a different default output format than ShellCheck. Use `--format human`
for the most readable output, or `--format json` for machine parsing.

### Performance

bashrs is typically faster than ShellCheck due to parallel rule execution and
file-level caching. For large codebases, use `bashrs lint --changed` to only
lint modified files.
