# Configuration File Management

**Rash** provides specialized support for analyzing and purifying shell configuration files like `.bashrc`, `.zshrc`, `.bash_profile`, and `.profile`. These files are critical infrastructure - they set up your shell environment for every new session.

## Why Config File Management Matters

Shell configuration files are:
- **Long-lived**: Used for years, accumulating cruft
- **Critical**: Errors break your shell sessions
- **Complex**: Mix environment setup, PATH management, aliases, functions
- **Duplicated**: Across multiple machines with inconsistencies

Common problems in config files:
- PATH duplicates slowing down command lookup
- Non-deterministic environment variables
- Conflicting settings across machines
- Security vulnerabilities (unsafe eval, command injection)
- Broken symlinks and missing directories

## What Rash Detects

Rash analyzes config files for:

### 1. PATH Issues (CONFIG-001, CONFIG-002)
- **Duplicate PATH entries**: Same directory added multiple times
- **Non-existent directories**: PATH entries that don't exist
- **Order problems**: Important paths shadowed by others

### 2. Environment Variable Issues (CONFIG-003, CONFIG-004)
- **Non-deterministic values**: Using `$RANDOM`, timestamps, etc.
- **Conflicting definitions**: Same variable set multiple times
- **Missing quotes**: Variables with spaces unquoted

### 3. Security Issues (SEC001-SEC008)
- **Command injection**: `eval` with user input
- **Insecure SSL**: `curl -k`, `wget --no-check-certificate`
- **Printf injection**: Unquoted format strings
- **Unsafe symlinks**: `ln -s` without cleanup

### 4. Idempotency Issues (IDEM001-IDEM006)
- **Non-idempotent operations**: Commands that fail on re-source
- **Append-only operations**: Growing arrays/PATH on each source
- **Missing guards**: No checks for existing values

## Supported Config Files

| File | Shell | When Loaded | Purpose |
|------|-------|-------------|---------|
| `.bashrc` | bash | Interactive non-login | Aliases, functions, prompt |
| `.bash_profile` | bash | Login shell | Environment, PATH, startup |
| `.profile` | sh, bash | Login shell (POSIX) | Universal environment setup |
| `.zshrc` | zsh | Interactive | Zsh-specific configuration |
| `.zshenv` | zsh | All sessions | Zsh environment variables |

## Quick Start: Analyzing Your Config

### Step 1: Lint Your .bashrc

```bash
bashrs lint ~/.bashrc
```

**Example Output**:
```text
/home/user/.bashrc:15:1: CONFIG-001 [Warning] Duplicate PATH entry
  export PATH="/usr/local/bin:$PATH"
  Note: /usr/local/bin already in PATH

/home/user/.bashrc:42:1: CONFIG-002 [Warning] Non-existent PATH entry
  export PATH="/opt/custom/bin:$PATH"
  Note: /opt/custom/bin does not exist

/home/user/.bashrc:58:1: SEC001 [Error] Command injection via eval
  eval $(some_command)
  Fix: Use source or direct execution instead

3 issues found (1 error, 2 warnings)
```

### Step 2: Review Issues

Each issue shows:
- **Location**: Line number and column
- **Rule ID**: CONFIG-001, SEC001, etc.
- **Severity**: Error or Warning
- **Description**: What the problem is
- **Fix suggestion**: How to resolve it

### Step 3: Apply Fixes

For manual fixes:
```bash
# Edit your config file
vim ~/.bashrc

# Test the changes
source ~/.bashrc

# Verify issues are resolved
bashrs lint ~/.bashrc
```

For automatic fixes (when available):
```bash
bashrs purify ~/.bashrc -o ~/.bashrc.purified
diff ~/.bashrc ~/.bashrc.purified
```

## Common Patterns and Solutions

### Pattern 1: Duplicate PATH Entries

**Problem**:
```bash
# .bashrc sourced multiple times adds duplicates
export PATH="/usr/local/bin:$PATH"
export PATH="/usr/local/bin:$PATH"  # Added again
```

**Solution**:
```bash
# Guard against duplicates
if [[ ":$PATH:" != *":/usr/local/bin:"* ]]; then
    export PATH="/usr/local/bin:$PATH"
fi
```

### Pattern 2: Non-Existent Directories

**Problem**:
```bash
# Adds directory that doesn't exist
export PATH="/opt/custom/bin:$PATH"
```

**Solution**:
```bash
# Check existence before adding
if [ -d "/opt/custom/bin" ]; then
    export PATH="/opt/custom/bin:$PATH"
fi
```

### Pattern 3: Non-Idempotent Sourcing

**Problem**:
```bash
# Appends every time .bashrc is sourced
PATH="$PATH:/new/dir"
```

**Solution**:
```bash
# Idempotent: only add if not present
case ":$PATH:" in
    *":/new/dir:"*) ;;
    *) PATH="$PATH:/new/dir" ;;
esac
export PATH
```

### Pattern 4: Secure Environment Setup

**Problem**:
```bash
# Dangerous: executes untrusted code
eval $(ssh-agent)
```

**Solution**:
```bash
# Safer: capture specific variables
if command -v ssh-agent >/dev/null; then
    SSH_AUTH_SOCK=$(ssh-agent | grep SSH_AUTH_SOCK | cut -d';' -f1 | cut -d'=' -f2)
    export SSH_AUTH_SOCK
fi
```

## Advanced: Multi-Machine Config Management

### Strategy 1: Host-Specific Sections

```bash
# .bashrc - universal settings
export EDITOR=vim

# Host-specific configuration
case "$(hostname)" in
    dev-laptop)
        export PATH="/home/user/local/bin:$PATH"
        ;;
    prod-server)
        export PATH="/opt/production/bin:$PATH"
        ;;
esac
```

### Strategy 2: Modular Configuration

```bash
# .bashrc - main file
for config in ~/.bashrc.d/*.sh; do
    if [ -r "$config" ]; then
        source "$config"
    fi
done
```

### Strategy 3: Version Control

```bash
# Keep configs in git
cd ~
git init
git add .bashrc .bash_profile .profile
git commit -m "Initial config"

# Lint before committing
bashrs lint .bashrc && git commit -m "Update config"
```

## Integration with Shell Workflow

### Pre-Commit Hook

```bash
#!/bin/sh
# .git/hooks/pre-commit

# Lint shell configs before committing
for file in .bashrc .bash_profile .profile .zshrc; do
    if git diff --cached --name-only | grep -q "^$file$"; then
        echo "Linting $file..."
        if ! bashrs lint "$file"; then
            echo "ERROR: $file has linting issues"
            exit 1
        fi
    fi
done
```

### CI/CD Validation

```yaml
# .github/workflows/config-lint.yml
name: Config Lint
on: [push, pull_request]
jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install Rash
        run: cargo install bashrs
      - name: Lint Configs
        run: |
          bashrs lint .bashrc
          bashrs lint .bash_profile
```

## Comparison with Other Tools

| Feature | Rash | ShellCheck | Bash-it | Oh-My-Zsh |
|---------|------|------------|---------|-----------|
| **PATH Analysis** | ✅ Duplicates, missing dirs | ❌ No | ❌ No | ❌ No |
| **Security Linting** | ✅ 8 SEC rules | ⚠️ Basic | ❌ No | ❌ No |
| **Idempotency** | ✅ Full support | ❌ No | ❌ No | ❌ No |
| **Multi-shell** | ✅ bash, zsh, sh | ✅ Yes | ❌ Bash only | ❌ Zsh only |
| **Auto-fix** | ✅ Purification | ❌ No | ❌ No | ❌ No |

## Best Practices

### 1. Regular Linting

```bash
# Add to weekly cron
0 9 * * 1 bashrs lint ~/.bashrc | mail -s "Config Lint Report" you@example.com
```

### 2. Test Changes Safely

```bash
# Test in new shell before sourcing
bash --noprofile --norc
source ~/.bashrc.new
# Verify everything works
exit

# Only then replace original
mv ~/.bashrc.new ~/.bashrc
```

### 3. Backup Before Changes

```bash
# Always backup
cp ~/.bashrc ~/.bashrc.backup.$(date +%Y%m%d)

# Apply changes
bashrs purify ~/.bashrc -o ~/.bashrc.new

# Test and swap
bash -c "source ~/.bashrc.new" && mv ~/.bashrc.new ~/.bashrc
```

### 4. Version Control

```bash
# Keep history
cd ~
git init
git add .bashrc .bash_profile .zshrc
git commit -m "Baseline config"

# Track changes
git diff .bashrc  # See what changed
git log .bashrc   # See history
```

### 5. Document Non-Standard Paths

```bash
# .bashrc - document why paths are added
# pyenv: Python version management
export PATH="$HOME/.pyenv/bin:$PATH"

# Homebrew: macOS package manager
export PATH="/opt/homebrew/bin:$PATH"

# Local binaries: custom scripts
export PATH="$HOME/bin:$PATH"
```

## Troubleshooting

### Issue: "Command not found" after linting

**Cause**: PATH was incorrectly modified

**Solution**:
```bash
# Restore backup
cp ~/.bashrc.backup.YYYYMMDD ~/.bashrc
source ~/.bashrc

# Re-apply changes carefully
bashrs lint ~/.bashrc --fix-one-by-one
```

### Issue: Slow shell startup

**Cause**: Too many PATH entries, slow commands in config

**Solution**:
```bash
# Profile your config
bash -x -i -c exit 2>&1 | less

# Remove duplicate PATH entries
bashrs lint ~/.bashrc | grep CONFIG-001

# Move slow commands to background or login-only
```

### Issue: Config works on one machine, breaks on another

**Cause**: Host-specific paths or commands

**Solution**:
```bash
# Add guards for host-specific sections
if [ "$(hostname)" = "dev-laptop" ]; then
    export PATH="/home/user/custom:$PATH"
fi

# Check command existence before using
if command -v pyenv >/dev/null; then
    eval "$(pyenv init -)"
fi
```

## Examples

See detailed examples:
- [.bashrc Purification](./examples/bashrc.md)
- [.zshrc Analysis](./examples/zshrc.md)
- [Multi-Machine Setup](./examples/multi-machine.md)

## Rules Reference

See complete rule documentation:
- [CONFIG-001: Duplicate PATH Entry](./rules/config-001.md)
- [CONFIG-002: Non-Existent PATH Entry](./rules/config-002.md)
- [CONFIG-003: Non-Deterministic Environment Variable](./rules/config-003.md)
- [CONFIG-004: Conflicting Environment Variable](./rules/config-004.md)

## Next Steps

- **Analyze**: Run `bashrs lint` on your config files
- **Learn**: Read about [CONFIG rules](./rules/config-001.md)
- **Practice**: Try [purifying](./purifying.md) a backup of your config
- **Integrate**: Set up [CI/CD validation](./analyzing.md#cicd-integration)

---

**Remember**: Your shell config files are critical infrastructure. Treat them with the same care as production code - version control, testing, and linting are essential for reliability.
