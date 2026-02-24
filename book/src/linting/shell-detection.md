# Shell Type Detection

bashrs automatically detects the shell type from your file path and content, ensuring linting rules are appropriate for the target shell.

## Supported Shells

- **bash** - Bourne Again Shell (default)
- **zsh** - Z Shell
- **sh** - POSIX Shell
- **ksh** - Korn Shell
- **auto** - Let ShellCheck auto-detect

## Detection Priority

bashrs uses a priority-based detection system (highest to lowest):

1. **ShellCheck directive** - Explicit override
2. **Shebang line** - Script header
3. **File extension** - `.zsh`, `.bash`, etc.
4. **File name** - `.zshrc`, `.bashrc`, etc.
5. **Default** - Falls back to bash

### Priority Example

```bash
#!/bin/bash
# shellcheck shell=zsh
# This file will be treated as ZSH (directive wins)
```

## Detection Methods

### 1. ShellCheck Directive (Highest Priority)

Explicitly specify the shell type in a comment:

```bash
# shellcheck shell=zsh
echo "This is zsh"
```

```bash
# shellcheck shell=sh
echo "This is POSIX sh"
```

**Use case**: Override auto-detection when file markers conflict.

### 2. Shebang Line

The script's shebang determines the shell:

```bash
#!/usr/bin/env zsh
# Detected as: zsh
```

```bash
#!/bin/bash
# Detected as: bash
```

```bash
#!/bin/sh
# Detected as: sh (POSIX)
```

### 3. File Extension

File extensions trigger automatic detection:

| Extension | Detected As |
|-----------|-------------|
| `.zsh` | zsh |
| `.bash` | bash |
| `.ksh` | ksh |
| `.sh` | bash (default) |

### 4. File Name

Special configuration files are automatically detected:

| File Name | Detected As |
|-----------|-------------|
| `.zshrc` | zsh |
| `.zshenv` | zsh |
| `.zprofile` | zsh |
| `.bashrc` | bash |
| `.bash_profile` | bash |
| `.bash_login` | bash |

## Why Shell Type Detection Matters

### The Problem

Different shells have different syntax:

**Valid in zsh** (but bash might flag it):
```zsh
# zsh array splitting with nested parameter expansion
filtered=("${(@f)"$(echo -e "line1\nline2")"}")
```

**bash linting error** (false positive):
```text
❌ SC2296: Parameter expansions can't be nested
```

### The Solution

With shell type detection:

```bash
# .zshrc is automatically detected as zsh
filtered=("${(@f)"$(echo -e "line1\nline2")"}")
# ✅ No error - valid zsh syntax
```

## Using the API

For programmatic access, use `lint_shell_with_path()`:

```rust,ignore
use bashrs::linter::{lint_shell_with_path, LintResult};
use std::path::PathBuf;

// Automatically detects zsh from .zshrc
let path = PathBuf::from(".zshrc");
let content = r#"
#!/usr/bin/env zsh
echo "Hello from zsh"
"#;

let result = lint_shell_with_path(&path, content);
// Uses zsh-appropriate rules
```

For shell type detection only:

```rust,ignore
use bashrs::linter::{detect_shell_type, ShellType};
use std::path::PathBuf;

let path = PathBuf::from(".zshrc");
let content = "echo hello";
let shell = detect_shell_type(&path, content);

assert_eq!(shell, ShellType::Zsh);
```

## Real-World Examples

### Example 1: zsh Configuration

```zsh
# ~/.zshrc (automatically detected as zsh)

# zsh-specific array handling
setopt EXTENDED_GLOB
files=(*.txt(N))  # Null glob modifier

# zsh parameter expansion
result=${${param#prefix}%%suffix}
```

**Result**: ✅ No false positives on zsh-specific syntax

### Example 2: Multi-Shell Script

```bash
#!/bin/bash
# shellcheck shell=sh
# Force POSIX sh rules despite bash shebang

# Only POSIX-compliant code allowed
echo "Portable script"
```

**Result**: ✅ Linted with strict POSIX rules

### Example 3: Shebang Override

```zsh
#!/bin/bash
# File has .zsh extension but bash shebang

# Will be linted as bash (shebang wins)
echo "This is actually bash"
```

**Result**: ✅ Bash rules applied (shebang priority)

## Common Patterns

### Pattern 1: Force zsh Detection

```bash
# For files without clear markers
# shellcheck shell=zsh
# Rest of zsh code...
```

### Pattern 2: POSIX Compliance Check

```bash
#!/bin/bash
# shellcheck shell=sh
# Ensures code is POSIX-portable
```

### Pattern 3: Default Behavior

```bash
# No shebang, no extension → defaults to bash
echo "Assumed to be bash"
```

## Benefits

### For zsh Users (70%+ of developers)

- ✅ No false positives on valid zsh syntax
- ✅ Automatic detection from `.zshrc`
- ✅ Supports zsh-specific features

### For macOS Users

- ✅ zsh is default shell (since 2019)
- ✅ Configuration files work out-of-the-box
- ✅ Oh My Zsh compatible

### For Script Authors

- ✅ Write once, lint correctly
- ✅ No manual configuration needed
- ✅ Multi-shell project support

## Troubleshooting

### Issue: Wrong Shell Detected

**Solution**: Add ShellCheck directive

```bash
# shellcheck shell=zsh
# Forces zsh detection
```

### Issue: Want Default Behavior

**Solution**: Remove all shell indicators, defaults to bash

### Issue: Testing Detection

```bash
# Create test file
echo '#!/usr/bin/env zsh' > test.sh

# Check detection (programmatically)
# bashrs will auto-detect from shebang
```

## Shell-Specific Rule Filtering

bashrs filters linter rules based on detected shell type.

### How It Works

When you use `lint_shell_with_path()`, bashrs:
1. Detects shell type from path and content (as before)
2. **Filters rules** based on shell compatibility
3. Skips bash-only rules for POSIX sh files
4. Skips sh-specific rules for bash/zsh files

### Example: POSIX sh Protection

```bash
#!/bin/sh
# This is POSIX sh - no bash arrays

# bashrs will NOT warn about missing bash features
# because it knows this is POSIX sh
```

**Bash-specific rules skipped for sh**:
- SC2198-2201 (arrays -- bash/zsh only)
- SC2039 (bash features undefined in sh)
- SC2044 (process substitution suggestions)

### Example: Universal Rules Always Apply

```bash
#!/bin/zsh
# Even in zsh, bad practices are still bad

SESSION_ID=$RANDOM  # DET001: Non-deterministic
mkdir /tmp/build    # IDEM001: Non-idempotent
```

**Universal rules apply to ALL shells**:
- DET001-003 (Determinism)
- IDEM001-003 (Idempotency)
- SEC001-019 (Security)
- Most SC1xxx and SC2xxx quoting/syntax rules

### Current Status (v6.64.0)

- **396 rules classified** in the rule registry
- **Shell compatibility** specified for every rule (Universal, NotSh, ShOnly, BashOnly)
- **60 SC1xxx rules** for source code issues (syntax, encoding, shebang)
- **325 SC2xxx rules** for shell best practices
- **Filtering active** in `lint_shell_with_path()`

## Summary

- **Automatic**: No configuration needed
- **Priority-based**: Clear precedence rules
- **Compatible**: Works with all major shells
- **Accurate**: 100% detection accuracy on test suite

**Result**: Write shell scripts naturally, lint correctly automatically.
