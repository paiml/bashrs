# Chapter 12: Shell Configuration Management

Shell configuration files (`.bashrc`, `.zshrc`, `.bash_profile`, etc.) are the foundation of your command-line environment. Over time, these files accumulate complexity: duplicate PATH entries, unquoted variables, conflicting aliases, and non-deterministic constructs that make your environment unpredictable.

Rash provides **automatic analysis, linting, and purification** for shell config files through four specialized rules: CONFIG-001 through CONFIG-004.

## Why Configuration Management Matters

**Problem**: Configuration files grow organically over years:
- Copy-pasting snippets from the internet
- Adding new tools without cleaning up old entries
- Accumulating duplicate PATH entries
- Leaving behind non-deterministic experiments

**Result**: Slow shell startup, unpredictable behavior, security vulnerabilities.

**Solution**: Rash automatically detects and fixes common configuration issues.

## Supported File Types

Rash recognizes these shell configuration files:
- `.bashrc` - Bash interactive shell configuration
- `.bash_profile` - Bash login shell configuration
- `.zshrc` - Zsh interactive shell configuration
- `.zprofile` - Zsh login shell configuration
- `.profile` - POSIX shell configuration

## Quick Start

```bash
# Analyze your .bashrc for issues
bashrs config analyze ~/.bashrc

# Lint with exit code 1 if issues found
bashrs config lint ~/.bashrc

# Auto-fix all issues (creates .bashrc.bak backup)
bashrs config purify ~/.bashrc
```

---

## CONFIG-001: PATH Deduplication

### The Problem

Over time, configuration files accumulate duplicate PATH entries:

```bash
# .bashrc - accumulated over time
export PATH="/usr/local/bin:$PATH"
export PATH="/usr/local/bin:$PATH"  # Duplicate!
export PATH="/opt/homebrew/bin:$PATH"
export PATH="/usr/local/bin:$PATH"  # Duplicate again!
```

**Impact**:
- **Performance**: Shell searches each PATH entry on every command lookup
- **Confusion**: Multiple copies make debugging harder
- **Maintenance**: Hard to know which line is actually used

### What CONFIG-001 Detects

Rash identifies duplicate PATH additions and reports which lines are redundant:

```bash
$ bashrs config analyze .bashrc
```

Output:
```
CONFIG-001 [Warning] Line 2: Duplicate PATH entry: '/usr/local/bin' (already added earlier)
CONFIG-001 [Warning] Line 4: Duplicate PATH entry: '/usr/local/bin' (already added earlier)
```

### Auto-Fix

```bash
$ bashrs config purify .bashrc
```

Result - clean, deduplicated PATH:
```bash
# .bashrc - after purification
export PATH="/usr/local/bin:$PATH"
# Removed duplicate: export PATH="/usr/local/bin:$PATH"
export PATH="/opt/homebrew/bin:$PATH"
# Removed duplicate: export PATH="/usr/local/bin:$PATH"
```

### How It Works

CONFIG-001 tracks first occurrence of each PATH entry and marks subsequent additions as duplicates. The purifier comments out duplicates while preserving the original file structure.

---

## CONFIG-002: Quote Variable Expansions

### The Problem

Unquoted variable expansions can lead to **word splitting**, **glob expansion**, and **injection vulnerabilities**:

```bash
# .bashrc - DANGEROUS unquoted variables
export PROJECT_DIR=$HOME/my projects      # ❌ Breaks on spaces
cd $PROJECT_DIR                            # ❌ Expands to 2 arguments
FILES=$(ls *.txt)                          # ❌ Loses whitespace info
```

**What goes wrong**:
```bash
$ cd $PROJECT_DIR
cd: too many arguments  # Shell splits at space!
```

### What CONFIG-002 Detects

Rash identifies all unquoted variable references that should be quoted:

```bash
$ bashrs config analyze .bashrc
```

Output:
```
CONFIG-002 [Warning] Line 1: Unquoted variable: $HOME (should be quoted)
CONFIG-002 [Warning] Line 2: Unquoted variable: $PROJECT_DIR (should be quoted)
CONFIG-002 [Warning] Line 3: Unquoted command substitution: $(ls *.txt)
```

### Auto-Fix

```bash
$ bashrs config purify .bashrc
```

Result - safely quoted variables:
```bash
# .bashrc - after purification
export PROJECT_DIR="${HOME}/my projects"   # ✅ Safe from splitting
cd "${PROJECT_DIR}"                         # ✅ Single argument
FILES="$(ls *.txt)"                         # ✅ Preserves whitespace
```

### When Quoting Is Skipped

CONFIG-002 is smart - it **doesn't quote** in these contexts:
- **Arithmetic**: `(( x = $var + 1 ))` - quoting not needed
- **[[ ]]**: Variable expansion is safe in double brackets
- **Assignments**: `VAR=$OTHER` - right-hand side is safe

### Why This Matters

**Security**: Unquoted variables are a common injection vector
**Correctness**: Spaces in paths cause hard-to-debug failures
**Reliability**: Glob patterns activate unexpectedly without quotes

---

## CONFIG-003: Consolidate Duplicate Aliases

### The Problem

Aliases get redefined multiple times as configuration files evolve:

```bash
# .bashrc - multiple alias definitions
alias ls='ls --color=auto'      # From Ubuntu default
alias ls='ls -G'                # From macOS snippet
alias ll='ls -la'
alias ll='ls -alh'              # Updated version
```

**What happens**:
- **Last definition wins** - earlier definitions are silently ignored
- **Confusion** - hard to know which alias is actually active
- **Bloat** - unnecessary lines slow shell startup

### What CONFIG-003 Detects

Rash finds all duplicate alias definitions:

```bash
$ bashrs config analyze .bashrc
```

Output:
```
CONFIG-003 [Warning] Line 2: Duplicate alias definition: 'ls' (first defined at line 1)
CONFIG-003 [Warning] Line 4: Duplicate alias definition: 'll' (first defined at line 3)
```

### Auto-Fix

```bash
$ bashrs config purify .bashrc
```

Result - only last definition kept:
```bash
# .bashrc - after purification
# Removed duplicate: alias ls='ls --color=auto'
alias ls='ls -G'                # ✅ Last definition wins
# Removed duplicate: alias ll='ls -la'
alias ll='ls -alh'              # ✅ Latest version kept
```

### Shell Behavior

The purifier matches **actual shell behavior**: when you define an alias multiple times, the last definition is what the shell uses. CONFIG-003 makes this explicit by removing earlier definitions.

---

## CONFIG-004: Remove Non-Deterministic Constructs

### The Problem

Non-deterministic constructs make your shell environment **unpredictable** and **unreproducible**:

```bash
# .bashrc - NON-DETERMINISTIC constructs
export SESSION_ID=$RANDOM                # ❌ Changes every time
export BUILD_TAG="v1.0.$(date +%s)"     # ❌ Timestamp changes
export LOG_FILE="/tmp/shell.$$.log"     # ❌ Process ID changes
export PS1="[$(hostname)] $ "            # ❌ Hostname-dependent
UPTIME=$(uptime)                         # ❌ Always different
```

**Why this is bad**:
- **Debugging nightmare**: Can't reproduce issues across sessions
- **Testing impossible**: Config behaves differently every time
- **Deployment chaos**: Same config produces different results

### What CONFIG-004 Detects

Rash identifies five types of non-deterministic constructs:

1. **$RANDOM** - generates unpredictable values
2. **$(date ...)** - timestamps change constantly
3. **$$** - process ID different each session
4. **$(hostname)** - varies across machines
5. **$(uptime)** - always changing

```bash
$ bashrs config analyze .bashrc
```

Output:
```
CONFIG-004 [Warning] Line 1: $RANDOM generates unpredictable values
CONFIG-004 [Warning] Line 2: Timestamp generation is non-deterministic
CONFIG-004 [Warning] Line 3: $$ (process ID) changes between sessions
CONFIG-004 [Warning] Line 4: $(hostname) may vary across environments
CONFIG-004 [Warning] Line 5: $(uptime) changes constantly
```

### Auto-Fix

```bash
$ bashrs config purify .bashrc
```

Result - non-deterministic constructs commented out:
```bash
# .bashrc - after purification
# CONFIG-004: Removed $RANDOM (non-deterministic)
# export SESSION_ID=$RANDOM

# CONFIG-004: Removed timestamp (non-deterministic)
# export BUILD_TAG="v1.0.$(date +%s)"

# CONFIG-004: Removed process ID (non-deterministic)
# export LOG_FILE="/tmp/shell.$$.log"

# CONFIG-004: Removed hostname check (environment-dependent)
# export PS1="[$(hostname)] $ "

# CONFIG-004: Removed uptime (non-deterministic)
# UPTIME=$(uptime)
```

### Replacements

Instead of non-deterministic constructs, use:
- **Fixed seeds**: `SESSION_ID="stable-session-001"` instead of `$RANDOM`
- **Version strings**: `BUILD_TAG="v1.0.0"` instead of timestamps
- **Named logs**: `LOG_FILE="/tmp/shell.log"` instead of PID-based names
- **Environment configs**: Separate config files per environment instead of `$(hostname)` checks

### Philosophy

Configuration files should be **deterministic** and **reproducible**. If you need environment-specific behavior, use separate config files (e.g., `.bashrc.production`, `.bashrc.development`) rather than runtime checks.

---

## Complete Workflow Example

### Before Purification

Your messy `.bashrc`:
```bash
# Accumulated over 5 years
export PATH="/usr/local/bin:$PATH"
export PATH="/usr/local/bin:$PATH"  # Duplicate
export PROJECT_DIR=$HOME/my projects  # Unquoted
alias ls='ls --color=auto'
alias ls='ls -G'  # Duplicate
export SESSION_ID=$RANDOM  # Non-deterministic
```

### Step 1: Analyze

```bash
$ bashrs config analyze ~/.bashrc
```

Output:
```
CONFIG-001 [Warning] Line 2: Duplicate PATH entry: '/usr/local/bin'
CONFIG-002 [Warning] Line 3: Unquoted variable: $HOME
CONFIG-003 [Warning] Line 5: Duplicate alias definition: 'ls' (first defined at line 4)
CONFIG-004 [Warning] Line 6: $RANDOM generates unpredictable values

Found 4 issues
```

### Step 2: Purify

```bash
$ bashrs config purify ~/.bashrc
Created backup: ~/.bashrc.bak
Purified: ~/.bashrc
```

### After Purification

Your clean `.bashrc`:
```bash
# Accumulated over 5 years
export PATH="/usr/local/bin:$PATH"
# Removed duplicate: export PATH="/usr/local/bin:$PATH"
export PROJECT_DIR="${HOME}/my projects"  # ✅ Quoted
# Removed duplicate: alias ls='ls --color=auto'
alias ls='ls -G'  # ✅ Last definition kept
# CONFIG-004: Removed $RANDOM (non-deterministic)
# export SESSION_ID=$RANDOM
```

## Best Practices

1. **Run purification regularly**: Add `bashrs config lint ~/.bashrc` to your CI/CD
2. **Review changes**: Check the `.bak` backup before committing purified files
3. **Idempotent**: Safe to run `purify` multiple times - it won't double-fix
4. **Source control**: Commit your clean config files to version control

## Summary

Rash's CONFIG rules provide **automatic cleanup** for shell configuration files:

| Rule | What It Fixes | Impact |
|------|---------------|--------|
| **CONFIG-001** | Duplicate PATH entries | ⚡ Faster command lookup |
| **CONFIG-002** | Unquoted variables | 🔒 Security + correctness |
| **CONFIG-003** | Duplicate aliases | 🧹 Cleaner config |
| **CONFIG-004** | Non-deterministic constructs | 🎯 Reproducible environment |

**Next chapter**: [Chapter 13: Verification and Testing →](ch13-verification-tdd.html)
