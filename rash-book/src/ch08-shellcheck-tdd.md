# Chapter 8: ShellCheck-Equivalent Linting

Shell scripts are notoriously error-prone. A missing quote, an unescaped variable, or a subtle portability issue can cause production failures that are hard to debug. **ShellCheck**, created by Vidar Holen, revolutionized shell script quality by detecting hundreds of common mistakes before they reach production.

Rash provides **ShellCheck-equivalent linting** with **100% coverage of the SC2xxx series** - all 300 rules implemented natively in Rust with zero external dependencies.

## Why Native Linting Matters

### The Problem with External Tools

Traditional approach:
```bash
# Write shell script
./deploy.sh

# Lint with external tool
shellcheck deploy.sh

# Fix issues, repeat
```

**Limitations**:
- **Separate tool**: Extra dependency to install and maintain
- **Slow feedback**: Run-parse-report cycle for every check
- **No integration**: Linting separate from transpilation/purification
- **Platform issues**: Different versions, installation challenges

### The Rash Approach

Rash integrates linting **directly into the workflow**:
```bash
# Lint as you transpile
bashrs transpile deploy.rs

# Lint as you purify
bashrs purify legacy.sh

# Or lint standalone
bashrs lint script.sh
```

**Benefits**:
- **Zero dependencies**: No ShellCheck installation required
- **Fast**: Native Rust implementation, no subprocess overhead
- **Integrated**: Linting during transpilation catches issues early
- **Consistent**: Same rules everywhere, same version

## 100% Coverage Milestone

**Historic Achievement**: Rash implements **all 300 ShellCheck SC2xxx rules** (100% coverage)

Journey to completion:
- **Sprint 116** (80%): 240 rules - Array safety, test expressions
- **Sprint 117** (85%): 255 rules - Functions, case statements
- **Sprint 118** (90%): 270 rules - Variable best practices
- **Sprint 119** (95%): 285 rules - Advanced shell patterns
- **Sprint 120** (100%): 300 rules - **Complete coverage! 🏆**

All rules implemented using **EXTREME TDD** methodology:
1. RED: Write failing test first
2. GREEN: Implement rule to pass test
3. REFACTOR: Clean up implementation
4. PROPERTY TEST: Verify with 100+ generated cases
5. MUTATION TEST: Achieve ≥90% kill rate

## Quick Start

### Install and Lint

```bash
# Install bashrs
cargo install bashrs

# Lint a script
bashrs lint script.sh
```

Output:
```text
script.sh:5:10: SC2086 [error] Double quote to prevent globbing and word splitting
script.sh:8:15: SC2046 [warning] Quote this to prevent word splitting
script.sh:12:5: SC2164 [error] Use 'cd ... || exit' in case cd fails

Found 3 issues (2 errors, 1 warning)
```

### Auto-Fix

Many rules include automatic fixes:
```bash
# Apply automatic fixes
bashrs lint --fix script.sh

# Creates backup: script.sh.bak
# Applies fixes to: script.sh
```

---

## Common Rules and Examples

### SC2086: Quote Variables to Prevent Word Splitting

**The Problem**:
```bash
# Unquoted variable - DANGEROUS
files=$FILE_LIST
rm $files  # ❌ Word splitting can cause disasters
```

If `$FILE_LIST` contains spaces, `rm` gets multiple arguments:
```bash
FILE_LIST="important.txt other.txt"
rm $files  # Removes TWO files, not one!
```

**The Fix**:
```bash
files="$FILE_LIST"
rm "$files"  # ✅ Safe - treated as single argument
```

**Rash detects**:
```text
SC2086 [error] Line 3: Double quote to prevent globbing and word splitting
  Suggestion: Use "$files" instead of $files
```

### SC2046: Quote Command Substitution

**The Problem**:
```bash
# Unquoted command substitution
files=$(find . -name "*.log")
rm $files  # ❌ Breaks on filenames with spaces
```

**The Fix**:
```bash
files="$(find . -name "*.log")"
rm "$files"  # ✅ Safe
```

### SC2164: Check cd Return Value

**The Problem**:
```bash
cd /critical/path
rm -rf *  # ❌ DISASTER if cd failed!
```

If `/critical/path` doesn't exist, `cd` fails silently and `rm -rf *` runs in the **current directory**!

**The Fix**:
```bash
cd /critical/path || exit 1  # ✅ Exit if cd fails
rm -rf *  # Safe - only runs if cd succeeded
```

**Rash detects**:
```text
SC2164 [error] Line 1: Use 'cd ... || exit' in case cd fails
  Suggestion: cd /critical/path || exit 1
```

### SC2115: Protect Dangerous rm -rf

**The Problem**:
```bash
# Empty variable = disaster
rm -rf "$PROJECT_DIR/"  # ❌ If $PROJECT_DIR is empty, becomes rm -rf /
```

**The Fix**:
```bash
# Fail if variable is unset
rm -rf "${PROJECT_DIR:?}/"  # ✅ Exits with error if unset

# Or check explicitly
if [ -n "$PROJECT_DIR" ]; then
    rm -rf "$PROJECT_DIR/"
fi
```

### SC2006: Use $() Instead of Backticks

**The Problem**:
```bash
# Old-style backticks - hard to read
result=`command arg1 arg2`  # ❌ Deprecated syntax
```

**The Fix**:
```bash
# Modern command substitution
result="$(command arg1 arg2)"  # ✅ Clearer, nestable
```

---

## Rule Categories

Rash's 300 linter rules cover these categories:

### Quoting and Safety (SC2046, SC2086, SC2116, etc.)
- Unquoted variables and command substitutions
- Word splitting prevention
- Glob expansion protection

### Command Execution (SC2006, SC2046, SC2116, etc.)
- Deprecated backtick syntax
- Useless command invocations
- Command substitution best practices

### File Operations (SC2115, SC2164, SC2181, etc.)
- Dangerous rm -rf patterns
- cd failure handling
- File test operators

### Arrays and Variables (SC2128, SC2178, SC2198-SC2201, etc.)
- Array expansion safety
- Variable type conflicts
- Array vs scalar usage

### Control Flow (SC2236-SC2250, SC2221-SC2235, etc.)
- Test expression syntax
- Loop control (break/continue)
- Conditional statement structure

### POSIX Compliance (SC2039, SC2040, SC2048, etc.)
- Bash-specific features in sh scripts
- Portability issues
- Standard vs non-standard syntax

## CI/CD Integration

### GitHub Actions

```yaml
name: Shell Script Quality

on: [push, pull_request]

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install bashrs
        run: cargo install bashrs

      - name: Lint shell scripts
        run: |
          find . -name "*.sh" -exec bashrs lint {} \;

      - name: Fail on errors
        run: bashrs lint --strict scripts/*.sh
```

### Pre-commit Hook

`.git/hooks/pre-commit`:
```bash
#!/bin/sh
# Lint all staged .sh files

# Find staged shell scripts
scripts=$(git diff --cached --name-only --diff-filter=ACM | grep '\.sh$')

if [ -n "$scripts" ]; then
    echo "Linting shell scripts..."
    for script in $scripts; do
        bashrs lint "$script" || exit 1
    done
fi
```

### Makefile Integration

```makefile
.PHONY: lint
lint:
	@echo "Linting shell scripts..."
	@find scripts -name "*.sh" -exec bashrs lint {} \;

.PHONY: lint-fix
lint-fix:
	@echo "Applying automatic fixes..."
	@find scripts -name "*.sh" -exec bashrs lint --fix {} \;
```

## Summary

**Rash provides ShellCheck-equivalent linting with zero dependencies**:

| Feature | Rash | ShellCheck |
|---------|------|------------|
| **Rules** | 300 SC2xxx (100%) | 300 SC2xxx |
| **Language** | Rust (native) | Haskell (external) |
| **Dependencies** | Zero | Requires installation |
| **Integration** | Built-in | Separate tool |
| **Speed** | Native (fast) | Subprocess overhead |
| **Auto-fix** | Yes | No |

**When to use Rash linting**:
- ✅ During transpilation (automatic)
- ✅ During purification (automatic)
- ✅ Standalone linting (`bashrs lint`)
- ✅ CI/CD pipelines
- ✅ Pre-commit hooks

**Key advantages**:
1. **No installation hassle** - ships with bashrs
2. **Consistent versions** - same rules everywhere
3. **Integrated workflow** - lint + transpile + purify
4. **Auto-fix support** - automatically apply fixes
5. **Fast execution** - native Rust, no subprocess
