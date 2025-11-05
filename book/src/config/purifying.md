# Purifying .bashrc and .zshrc

Shell configuration files like `.bashrc` and `.zshrc` accumulate cruft over time. Duplicate PATH entries, redundant exports, non-idempotent operations, and unquoted variables create fragile, unpredictable environments. The `bashrs config purify` command transforms messy configuration files into clean, safe, deterministic shell scripts.

This chapter covers how to use `bashrs` to purify your shell configuration files, with comprehensive examples, best practices, and troubleshooting guidance.

## What Purification Does

The `bashrs config purify` command applies four critical transformations:

### 1. Deduplication

Removes duplicate entries that accumulate from repeatedly sourcing configuration files or copy-pasting snippets.

**Before**:
```bash
export PATH="/usr/local/bin:$PATH"
export PATH="/usr/local/bin:$PATH"
export PATH="/opt/bin:$PATH"
export PATH="/opt/bin:$PATH"
```

**After**:
```bash
export PATH="/usr/local/bin:/opt/bin:$PATH"
```

### 2. Idempotency

Ensures operations can be safely re-run without side effects. Critical for configuration files that may be sourced multiple times.

**Before**:
```bash
export PATH="/usr/local/bin:$PATH"  # Grows every time .bashrc is sourced
alias ll='ls -la'
alias ll='ls -lah'  # Duplicate alias
```

**After**:
```bash
# Idempotent PATH management
add_to_path() {
    case ":$PATH:" in
        *":$1:"*) ;;
        *) export PATH="$1:$PATH" ;;
    esac
}

add_to_path "/usr/local/bin"

# Single alias definition
alias ll='ls -lah'
```

### 3. Determinism

Eliminates non-deterministic constructs like `$RANDOM`, timestamps, and process IDs that cause inconsistent behavior.

**Before**:
```bash
export SESSION_ID=$RANDOM
export LOG_FILE="/tmp/session-$(date +%s).log"
export PROMPT_PID=$$
```

**After**:
```bash
# Deterministic session identifier based on user and hostname
export SESSION_ID="${USER}-${HOSTNAME}"
export LOG_FILE="${HOME}/.logs/session.log"
export PROMPT_PID="${USER}"
```

### 4. Safety (Variable Quoting)

Quotes all variable expansions to prevent word splitting and glob expansion vulnerabilities.

**Before**:
```bash
export JAVA_HOME=/usr/lib/jvm/java-11
export PATH=$JAVA_HOME/bin:$PATH
if [ -d $HOME/.cargo/bin ]; then
    export PATH=$HOME/.cargo/bin:$PATH
fi
```

**After**:
```bash
export JAVA_HOME="/usr/lib/jvm/java-11"
export PATH="${JAVA_HOME}/bin:${PATH}"
if [ -d "${HOME}/.cargo/bin" ]; then
    export PATH="${HOME}/.cargo/bin:${PATH}"
fi
```

## Command Usage

### Basic Syntax

```bash
bashrs config purify <input-file> [options]
```

### Options

- `--output <file>` - Write purified output to specified file (default: stdout)
- `--backup` - Create backup of original file (`.bak` extension)
- `--check` - Dry-run mode, report issues without modifying
- `--shellcheck` - Validate output with shellcheck
- `--shell <sh|bash|zsh>` - Target shell (default: auto-detect)

### Examples

**Purify and print to stdout**:
```bash
bashrs config purify ~/.bashrc
```

**Purify to new file**:
```bash
bashrs config purify ~/.bashrc --output ~/.bashrc.purified
```

**Purify with automatic backup**:
```bash
bashrs config purify ~/.bashrc --output ~/.bashrc --backup
# Creates ~/.bashrc.bak before overwriting
```

**Check what would be purified**:
```bash
bashrs config purify ~/.bashrc --check
```

**Purify and validate**:
```bash
bashrs config purify ~/.bashrc --output ~/.bashrc.purified --shellcheck
```

## Complete Example: Purifying a Messy .bashrc

### Before: Messy .bashrc

This configuration file has accumulated common problems over years of use:

```bash
#!/bin/bash
# .bashrc - Accumulated over 5 years

# PATH modifications (duplicates and non-idempotent)
export PATH="/usr/local/bin:$PATH"
export PATH="/usr/local/bin:$PATH"
export PATH="$HOME/.cargo/bin:$PATH"
export PATH="/opt/homebrew/bin:$PATH"
export PATH="$HOME/.cargo/bin:$PATH"
export PATH="/opt/homebrew/bin:$PATH"

# Environment variables (unquoted, some duplicated)
export EDITOR=vim
export EDITOR=nvim
export VISUAL=$EDITOR
export GOPATH=$HOME/go
export GOROOT=/usr/local/go
export PATH=$GOPATH/bin:$PATH
export PATH=$GOROOT/bin:$PATH

# Non-deterministic values
export SESSION_ID=$RANDOM
export BUILD_TIMESTAMP=$(date +%s)
export TEMP_DIR="/tmp/build-$$"

# Aliases (duplicates)
alias ll='ls -l'
alias ll='ls -la'
alias ll='ls -lah'
alias gs='git status'
alias gs='git status -sb'

# Functions (non-idempotent)
mkcd() {
    mkdir $1
    cd $1
}

# Conditional PATH additions (unquoted)
if [ -d $HOME/.local/bin ]; then
    export PATH=$HOME/.local/bin:$PATH
fi

if [ -d /usr/local/opt/ruby/bin ]; then
    export PATH=/usr/local/opt/ruby/bin:$PATH
fi

# Non-idempotent operations
mkdir ~/.config/myapp
touch ~/.config/myapp/config.ini
```

### Purification Command

```bash
bashrs config purify ~/.bashrc --output ~/.bashrc.purified --backup --shellcheck
```

### After: Purified .bashrc

The purified version is clean, safe, and idempotent:

```bash
#!/bin/sh
# .bashrc - Purified by bashrs v6.31.0
# Original backed up to ~/.bashrc.bak

# Idempotent PATH management helper
add_to_path() {
    case ":${PATH}:" in
        *":$1:"*) ;;
        *) export PATH="$1:${PATH}" ;;
    esac
}

# Deduplicated and idempotent PATH modifications
add_to_path "/usr/local/bin"
add_to_path "${HOME}/.cargo/bin"
add_to_path "/opt/homebrew/bin"
add_to_path "${GOPATH}/bin"
add_to_path "${GOROOT}/bin"

# Environment variables (deduplicated, properly quoted)
export EDITOR="nvim"
export VISUAL="${EDITOR}"
export GOPATH="${HOME}/go"
export GOROOT="/usr/local/go"

# Deterministic values (replaced non-deterministic constructs)
export SESSION_ID="${USER}-${HOSTNAME}"
export BUILD_TIMESTAMP="static"
export TEMP_DIR="${HOME}/.cache/build"

# Aliases (deduplicated, kept most recent)
alias ll='ls -lah'
alias gs='git status -sb'

# Functions (idempotent, properly quoted)
mkcd() {
    mkdir -p "$1" || return 1
    cd "$1" || return 1
}

# Conditional PATH additions (properly quoted, idempotent)
if [ -d "${HOME}/.local/bin" ]; then
    add_to_path "${HOME}/.local/bin"
fi

if [ -d "/usr/local/opt/ruby/bin" ]; then
    add_to_path "/usr/local/opt/ruby/bin"
fi

# Idempotent directory creation
mkdir -p "${HOME}/.config/myapp"
touch "${HOME}/.config/myapp/config.ini"
```

### Purification Report

```
bashrs config purify v6.31.0

Input:  /home/user/.bashrc (42 lines)
Output: /home/user/.bashrc.purified (45 lines)
Backup: /home/user/.bashrc.bak

Transformations Applied:
  - Deduplicated 6 PATH entries → 5 unique entries
  - Removed 2 duplicate aliases
  - Removed 1 duplicate export
  - Added idempotent add_to_path() helper
  - Replaced 3 non-deterministic values
  - Quoted 12 unquoted variable expansions
  - Made 3 operations idempotent (mkdir, cd)

Shellcheck: PASSED (0 issues)

Safety: 100% (all variables quoted)
Idempotency: 100% (safe to re-source)
Determinism: 100% (no random/timestamp values)
```

## Idempotent PATH Management

The `add_to_path()` helper function is the cornerstone of idempotent configuration. It prevents duplicate PATH entries even when `.bashrc` is sourced multiple times.

### The Helper Function

```bash
add_to_path() {
    case ":${PATH}:" in
        *":$1:"*) ;;  # Already in PATH, do nothing
        *) export PATH="$1:${PATH}" ;;  # Not in PATH, prepend it
    esac
}
```

### How It Works

The function uses shell pattern matching to check if the directory is already in `$PATH`:

1. Wraps `$PATH` in colons: `:${PATH}:`
2. Checks if `":$1:"` exists in the wrapped path
3. If found, does nothing (already present)
4. If not found, prepends to `$PATH`

### Usage Examples

```bash
# Add single directory
add_to_path "/usr/local/bin"

# Add multiple directories
add_to_path "${HOME}/.cargo/bin"
add_to_path "${HOME}/.local/bin"
add_to_path "/opt/homebrew/bin"

# Conditional additions
if [ -d "${HOME}/.rbenv/bin" ]; then
    add_to_path "${HOME}/.rbenv/bin"
fi
```

### Testing Idempotency

```bash
# Source .bashrc multiple times
$ echo "$PATH"
/home/user/.cargo/bin:/usr/local/bin:/usr/bin:/bin

$ source ~/.bashrc
$ echo "$PATH"
/home/user/.cargo/bin:/usr/local/bin:/usr/bin:/bin

$ source ~/.bashrc
$ echo "$PATH"
/home/user/.cargo/bin:/usr/local/bin:/usr/bin:/bin
```

The PATH remains identical after multiple sourcing operations.

### Variant: Append Instead of Prepend

```bash
add_to_path_append() {
    case ":${PATH}:" in
        *":$1:"*) ;;
        *) export PATH="${PATH}:$1" ;;
    esac
}
```

Use this variant when you want to add directories to the end of PATH (lower priority).

## Shell-Specific Considerations

### Bash vs Zsh Differences

While `bashrs` generates POSIX-compliant output that works in both shells, there are considerations:

#### Bash-Specific Features

**Arrays** (not POSIX):
```bash
# Before (.bashrc)
declare -a my_array=(one two three)

# After (purified, POSIX-compliant)
my_array="one two three"
```

**Bash completion**:
```bash
# Bash-specific completion files
if [ -f /etc/bash_completion ]; then
    . /etc/bash_completion
fi
```

Purified output preserves bash-specific features but adds shell detection:

```bash
# Purified with shell detection
if [ -n "${BASH_VERSION}" ] && [ -f /etc/bash_completion ]; then
    . /etc/bash_completion
fi
```

#### Zsh-Specific Features

**oh-my-zsh** integration:
```bash
# Before (.zshrc)
export ZSH="$HOME/.oh-my-zsh"
ZSH_THEME="robbyrussell"
plugins=(git docker kubectl)
source $ZSH/oh-my-zsh.sh

# After (purified)
export ZSH="${HOME}/.oh-my-zsh"
ZSH_THEME="robbyrussell"
plugins=(git docker kubectl)
# shellcheck source=/dev/null
. "${ZSH}/oh-my-zsh.sh"
```

**Zsh arrays**:
```bash
# Zsh uses different array syntax
typeset -U path  # Zsh-specific: unique PATH entries
path=(/usr/local/bin $path)
```

Purified output converts to POSIX-compatible syntax or adds shell detection.

### Shell Detection Pattern

For features that only work in specific shells:

```bash
# Detect bash
if [ -n "${BASH_VERSION}" ]; then
    # Bash-specific configuration
    shopt -s histappend
fi

# Detect zsh
if [ -n "${ZSH_VERSION}" ]; then
    # Zsh-specific configuration
    setopt HIST_IGNORE_DUPS
fi
```

## Verification Steps

After purifying your configuration, follow these steps to verify correctness:

### Step 1: Syntax Validation

```bash
# Validate with shellcheck
shellcheck -s sh ~/.bashrc.purified

# Check syntax with shell parser
sh -n ~/.bashrc.purified
bash -n ~/.bashrc.purified
```

Expected output:
```text
# No output = success
```

### Step 2: Source Multiple Times

Test idempotency by sourcing multiple times:

```bash
# Start fresh shell
bash --norc --noprofile

# Source purified config
source ~/.bashrc.purified
echo "PATH after 1st source: $PATH"

# Source again
source ~/.bashrc.purified
echo "PATH after 2nd source: $PATH"

# Source third time
source ~/.bashrc.purified
echo "PATH after 3rd source: $PATH"
```

**Expected**: PATH should be identical after each sourcing.

### Step 3: Environment Comparison

Compare environment before and after:

```bash
# Capture original environment
env > /tmp/env-before.txt

# Source purified config in new shell
bash --norc --noprofile -c 'source ~/.bashrc.purified && env' > /tmp/env-after.txt

# Compare
diff /tmp/env-before.txt /tmp/env-after.txt
```

Review differences to ensure expected variables are set.

### Step 4: Function Testing

Test all functions defined in config:

```bash
# Source config
source ~/.bashrc.purified

# Test mkcd function
mkcd /tmp/test-dir
pwd  # Should be /tmp/test-dir

# Test again (idempotency)
mkcd /tmp/test-dir
pwd  # Should still work
```

### Step 5: Alias Verification

```bash
# Check aliases are defined
alias ll
alias gs

# Test aliases work
ll /tmp
gs  # If in git repo
```

### Step 6: PATH Verification

```bash
# Check PATH entries are unique
echo "$PATH" | tr ':' '\n' | sort | uniq -d
# No output = no duplicates
```

### Step 7: Integration Testing

Test with real tools:

```bash
# Test language tooling
which python
which ruby
which go

# Test custom binaries
which custom-tool

# Test completions (if any)
kubectl <TAB>
git <TAB>
```

## Rollback Strategy

Always have a rollback plan when modifying critical configuration files.

### 1. Create Backup

```bash
# Manual backup
cp ~/.bashrc ~/.bashrc.backup-$(date +%Y%m%d)

# Automatic backup with bashrs
bashrs config purify ~/.bashrc --output ~/.bashrc --backup
# Creates ~/.bashrc.bak
```

### 2. Test in Isolated Environment

```bash
# Test in new shell session (doesn't affect current shell)
bash --rcfile ~/.bashrc.purified

# Test in Docker container
docker run -it --rm -v ~/.bashrc.purified:/root/.bashrc ubuntu bash

# Test in subshell
(source ~/.bashrc.purified; env; alias)
```

### 3. Gradual Deployment

**Phase 1**: Test for one session
```bash
# Use purified config for current session only
source ~/.bashrc.purified
# Test thoroughly
# If issues arise, close terminal
```

**Phase 2**: Deploy for one day
```bash
# Replace config
mv ~/.bashrc ~/.bashrc.old
mv ~/.bashrc.purified ~/.bashrc

# Use for a day, monitor for issues
```

**Phase 3**: Full deployment
```bash
# After successful testing period
rm ~/.bashrc.old
# Purified config is now the primary
```

### 4. Quick Rollback

If issues arise:

```bash
# Restore from backup
cp ~/.bashrc.bak ~/.bashrc
source ~/.bashrc

# Or restore from timestamped backup
cp ~/.bashrc.backup-20250104 ~/.bashrc
source ~/.bashrc
```

### 5. Emergency Recovery

If you're locked out (e.g., broken PATH):

```bash
# Start shell without config
bash --norc --noprofile

# Fix PATH manually
export PATH="/usr/local/bin:/usr/bin:/bin"

# Restore backup
cp ~/.bashrc.bak ~/.bashrc

# Restart shell
exec bash
```

## Common Purification Patterns

### Pattern 1: Deduplicating Exports

**Before**:
```bash
export LANG=en_US.UTF-8
export LC_ALL=en_US.UTF-8
export LANG=en_US.UTF-8  # Duplicate
```

**After**:
```bash
export LANG="en_US.UTF-8"
export LC_ALL="en_US.UTF-8"
```

### Pattern 2: Consolidating Conditionals

**Before**:
```bash
if [ -f ~/.bash_aliases ]; then
    source ~/.bash_aliases
fi

if [ -f ~/.bash_functions ]; then
    source ~/.bash_functions
fi

if [ -f ~/.bash_local ]; then
    source ~/.bash_local
fi
```

**After**:
```bash
# Source additional config files if they exist
for config_file in "${HOME}/.bash_aliases" \
                   "${HOME}/.bash_functions" \
                   "${HOME}/.bash_local"; do
    if [ -f "${config_file}" ]; then
        # shellcheck source=/dev/null
        . "${config_file}"
    fi
done
```

### Pattern 3: Idempotent Sourcing

**Before**:
```bash
source ~/.nvm/nvm.sh
source ~/.nvm/nvm.sh  # Sourced twice
```

**After**:
```bash
# Source only if not already loaded
if [ -z "${NVM_DIR}" ] && [ -f "${HOME}/.nvm/nvm.sh" ]; then
    # shellcheck source=/dev/null
    . "${HOME}/.nvm/nvm.sh"
fi
```

### Pattern 4: Safe Command Availability Checks

**Before**:
```bash
eval "$(rbenv init -)"
eval "$(pyenv init -)"
```

**After**:
```bash
# Initialize rbenv if available
if command -v rbenv >/dev/null 2>&1; then
    eval "$(rbenv init -)"
fi

# Initialize pyenv if available
if command -v pyenv >/dev/null 2>&1; then
    eval "$(pyenv init -)"
fi
```

### Pattern 5: History Management

**Before**:
```bash
export HISTSIZE=10000
export HISTSIZE=50000
export HISTFILESIZE=20000
export HISTCONTROL=ignoreboth
export HISTCONTROL=ignoredups
```

**After**:
```bash
export HISTSIZE="50000"
export HISTFILESIZE="50000"
export HISTCONTROL="ignoreboth"
```

### Pattern 6: Prompt Customization

**Before**:
```bash
export PS1='\u@\h:\w\$ '
export PS1='[\u@\h \W]\$ '  # Overrides previous
```

**After**:
```bash
# Customized prompt (last definition wins)
export PS1='[\u@\h \W]\$ '
```

## Best Practices

### 1. Always Create Backups

```bash
# Before purification
cp ~/.bashrc ~/.bashrc.backup-$(date +%Y%m%d-%H%M%S)

# Or use --backup flag
bashrs config purify ~/.bashrc --output ~/.bashrc --backup
```

### 2. Test in Isolated Environment

```bash
# Test in subshell first
bash --rcfile ~/.bashrc.purified -i

# Or test specific sections
(source ~/.bashrc.purified; which python; echo "$PATH")
```

### 3. Use Version Control

```bash
# Initialize git repo for dotfiles
cd ~
git init
git add .bashrc .zshrc
git commit -m "Initial commit before purification"

# After purification
git add .bashrc.purified
git commit -m "Purified .bashrc with bashrs v6.31.0"
```

### 4. Separate Concerns

Organize configuration into modular files:

```bash
# ~/.bashrc (main config)
# Source modular configs
for config in "${HOME}/.config/bash"/*.sh; do
    [ -f "${config}" ] && . "${config}"
done

# ~/.config/bash/path.sh (PATH management)
add_to_path "/usr/local/bin"
add_to_path "${HOME}/.cargo/bin"

# ~/.config/bash/aliases.sh (aliases)
alias ll='ls -lah'
alias gs='git status -sb'

# ~/.config/bash/functions.sh (functions)
mkcd() { mkdir -p "$1" && cd "$1"; }
```

Purify each file separately:

```bash
bashrs config purify ~/.config/bash/path.sh --output ~/.config/bash/path.sh --backup
bashrs config purify ~/.config/bash/aliases.sh --output ~/.config/bash/aliases.sh --backup
bashrs config purify ~/.config/bash/functions.sh --output ~/.config/bash/functions.sh --backup
```

### 5. Document Customizations

Add comments to explain non-obvious configurations:

```bash
# Custom PATH for local development
# Prepend local bin directories (higher priority)
add_to_path "${HOME}/.local/bin"
add_to_path "${HOME}/bin"

# Language-specific tooling
add_to_path "${HOME}/.cargo/bin"     # Rust
add_to_path "${GOPATH}/bin"          # Go
add_to_path "${HOME}/.rbenv/bin"     # Ruby
```

### 6. Regular Purification

Schedule periodic purification to prevent cruft accumulation:

```bash
# Monthly purification check
0 0 1 * * /usr/local/bin/bashrs config purify ~/.bashrc --check | mail -s "bashrc purification report" user@example.com
```

### 7. Validate After Changes

Always validate after manual edits:

```bash
# After editing .bashrc
bashrs config purify ~/.bashrc --check --shellcheck
```

## Troubleshooting

### Issue 1: PATH Still Has Duplicates

**Symptom**:
```bash
$ echo "$PATH" | tr ':' '\n' | sort | uniq -d
/usr/local/bin
/usr/local/bin
```

**Cause**: Sourcing other scripts that modify PATH.

**Solution**: Audit all sourced files:
```bash
# Find all sourced files
grep -E '^\s*(source|\.)' ~/.bashrc

# Purify each one
bashrs config purify ~/.bash_aliases --output ~/.bash_aliases --backup
bashrs config purify ~/.bash_functions --output ~/.bash_functions --backup
```

### Issue 2: Aliases Not Working

**Symptom**:
```bash
$ ll
bash: ll: command not found
```

**Cause**: Aliases defined in non-interactive shell.

**Solution**: Check if running in interactive mode:
```bash
# Add to .bashrc
case $- in
    *i*)
        # Interactive shell, define aliases
        alias ll='ls -lah'
        ;;
esac
```

### Issue 3: Functions Lost After Purification

**Symptom**: Functions work before purification but not after.

**Cause**: bashrs may have converted bash-specific functions to POSIX.

**Solution**: Check purified function syntax:
```bash
# Before (bash-specific)
function my_func() {
    local var=$1
    echo $var
}

# After (POSIX-compliant)
my_func() {
    _var="$1"
    echo "${_var}"
}
```

### Issue 4: Environment Variables Not Set

**Symptom**: `$GOPATH` is empty after sourcing purified config.

**Cause**: Variable depends on another variable that's not set.

**Solution**: Check dependency order:
```bash
# Wrong order
export PATH="${GOPATH}/bin:${PATH}"
export GOPATH="${HOME}/go"

# Correct order (purified)
export GOPATH="${HOME}/go"
add_to_path "${GOPATH}/bin"
```

### Issue 5: Slow Shell Startup

**Symptom**: Shell takes 5+ seconds to start after purification.

**Cause**: Purified config may have added expensive operations.

**Solution**: Profile the config:
```bash
# Add to top of .bashrc
PS4='+ $(date "+%s.%N")\011 '
set -x

# Add to bottom
set +x
```

Check timestamps to identify slow operations, then optimize or lazy-load them.

### Issue 6: Shellcheck Warnings

**Symptom**:
```bash
$ bashrs config purify ~/.bashrc --shellcheck
SC2034: UNUSED_VAR appears unused. Verify use (or export if used externally).
```

**Solution**: Export used variables or remove unused ones:
```bash
# If used by external programs
export UNUSED_VAR="value"

# If truly unused
# Remove it
```

### Issue 7: Non-POSIX Constructs

**Symptom**: Purified config doesn't work in `sh`.

**Cause**: bashrs detected shell-specific features.

**Solution**: Use shell detection:
```bash
# Bash-specific features
if [ -n "${BASH_VERSION}" ]; then
    shopt -s histappend
    shopt -s checkwinsize
fi

# Zsh-specific features
if [ -n "${ZSH_VERSION}" ]; then
    setopt HIST_IGNORE_DUPS
fi
```

### Issue 8: Broken Sourcing Chain

**Symptom**: Scripts that source other scripts fail.

**Cause**: Relative paths broken after purification.

**Solution**: Use absolute paths:
```bash
# Before
source ../lib/helpers.sh

# After (purified)
# shellcheck source=/dev/null
. "${HOME}/.config/bash/lib/helpers.sh"
```

## Real-World Example: Full Workflow

Here's a complete workflow for purifying a production `.bashrc`:

### Step 1: Backup

```bash
# Create timestamped backup
cp ~/.bashrc ~/.bashrc.backup-$(date +%Y%m%d-%H%M%S)

# Verify backup
diff ~/.bashrc ~/.bashrc.backup-*
```

### Step 2: Analyze Current State

```bash
# Check current config
wc -l ~/.bashrc
# 234 lines

# Count PATH modifications
grep -c 'export PATH' ~/.bashrc
# 18 (likely duplicates)

# Check for non-deterministic constructs
grep -E '\$RANDOM|\$\$|date \+' ~/.bashrc
# 3 matches (need fixing)
```

### Step 3: Purify

```bash
bashrs config purify ~/.bashrc \
    --output ~/.bashrc.purified \
    --shellcheck
```

Output:
```text
bashrs config purify v6.31.0

Transformations Applied:
  - Deduplicated 18 PATH entries → 9 unique
  - Added add_to_path() helper
  - Replaced 3 non-deterministic values
  - Quoted 47 variable expansions
  - Made 8 operations idempotent

Shellcheck: PASSED
```

### Step 4: Test in Subshell

```bash
# Test in isolated environment
bash --rcfile ~/.bashrc.purified -i

# Verify PATH
echo "$PATH"

# Test aliases
ll
gs

# Test functions
mkcd /tmp/test
pwd

# Exit test shell
exit
```

### Step 5: Deploy Gradually

```bash
# Day 1: Use in current session only
source ~/.bashrc.purified

# Day 2: Use as default for new shells
mv ~/.bashrc ~/.bashrc.old
ln -s ~/.bashrc.purified ~/.bashrc

# Day 7: Commit to version control
git add ~/.bashrc.purified
git commit -m "Purified .bashrc with bashrs v6.31.0"
git push

# Day 30: Remove old backup
rm ~/.bashrc.old
```

### Step 6: Verify Production

```bash
# Source multiple times
for i in 1 2 3; do
    bash -c 'source ~/.bashrc && echo "PATH: $PATH"'
done

# All outputs should be identical
```

## Summary

The `bashrs config purify` command transforms messy shell configuration files into clean, safe, deterministic scripts by:

1. **Deduplicating** repeated exports, aliases, and PATH entries
2. **Enforcing idempotency** with helper functions like `add_to_path()`
3. **Eliminating non-determinism** by replacing `$RANDOM`, timestamps, and process IDs
4. **Ensuring safety** by quoting all variable expansions

**Key takeaways**:

- Always backup before purifying
- Test in isolated environments before deploying
- Use the `add_to_path()` helper for idempotent PATH management
- Validate with shellcheck and manual testing
- Deploy gradually with rollback plan
- Organize configs into modular files
- Purify regularly to prevent cruft accumulation

With purified configuration files, you can confidently source your `.bashrc` or `.zshrc` multiple times without side effects, ensuring consistent, predictable shell environments across all your systems.
