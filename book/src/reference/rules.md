# Linter Rules Reference

This chapter provides a complete reference for all linter rules in bashrs v6.31.0, including security rules, determinism rules, idempotency rules, config rules, Makefile rules, and ShellCheck integration.

## Table of Contents

- [Rule Categories](#rule-categories)
- [Security Rules (SEC001-SEC008)](#security-rules-sec001-sec008)
- [Determinism Rules (DET001-DET003)](#determinism-rules-det001-det003)
- [Idempotency Rules (IDEM001-IDEM003)](#idempotency-rules-idem001-idem003)
- [Config Rules (CONFIG-001 to CONFIG-003)](#config-rules-config-001-to-config-003)
- [Makefile Rules (MAKE001-MAKE020)](#makefile-rules-make001-make020)
- [ShellCheck Integration](#shellcheck-integration)
- [Rule Severity Levels](#rule-severity-levels)
- [Auto-Fix Capabilities](#auto-fix-capabilities)
- [Disabling Rules](#disabling-rules)
- [Custom Rule Development](#custom-rule-development)

## Rule Categories

bashrs organizes linter rules into several categories:

| Category | Rule Prefix | Count | Purpose |
|----------|-------------|-------|---------|
| Security | SEC | 8 | Detect security vulnerabilities |
| Determinism | DET | 3 | Ensure predictable output |
| Idempotency | IDEM | 3 | Ensure safe re-execution |
| Config | CONFIG | 3 | Shell configuration analysis |
| Makefile | MAKE | 20 | Makefile-specific issues |
| ShellCheck | SC | 324+ | Shell script best practices |

## Security Rules (SEC001-SEC008)

Security rules detect critical vulnerabilities that could lead to command injection, information disclosure, or other security issues.

### SEC001: Command Injection via eval

**Severity:** Error
**Auto-fix:** No (manual review required)

Detects `eval` usage with potentially user-controlled input, the #1 command injection vector.

**Bad:**
```bash
eval "rm -rf $USER_INPUT"  # DANGEROUS
eval "$CMD"                # DANGEROUS
```

**Good:**
```bash
# Use arrays and proper quoting
cmd_array=("rm" "-rf" "$USER_INPUT")
"${cmd_array[@]}"

# Or explicit validation
if [[ "$CMD" =~ ^[a-zA-Z0-9_-]+$ ]]; then
    $CMD
fi
```

**Why it matters:** Attackers can inject arbitrary commands through shell metacharacters (`;`, `|`, `&`, etc.).

**Detection pattern:** Searches for `eval` as a standalone command

### SEC002: Unquoted Variable in Command

**Severity:** Error
**Auto-fix:** Yes (safe)

Detects unquoted variables in dangerous commands that could lead to command injection.

**Dangerous commands checked:**
- `curl`, `wget` (network)
- `ssh`, `scp`, `rsync` (remote)
- `git` (version control)
- `docker`, `kubectl` (containers)

**Bad:**
```bash
curl $URL           # Word splitting risk
wget $FILE_PATH     # Injection risk
ssh $HOST           # Command injection
git clone $REPO     # Path traversal
```

**Good:**
```bash
curl "${URL}"
wget "${FILE_PATH}"
ssh "${HOST}"
git clone "${REPO}"
```

**Auto-fix:** Wraps variable in double quotes: `"${VAR}"`

### SEC003: Unquoted {} in find -exec

**Severity:** Error
**Auto-fix:** Yes (safe)

Detects unquoted `{}` placeholder in `find -exec` commands.

**Bad:**
```bash
find . -name "*.sh" -exec chmod +x {} \;
find /tmp -type f -exec rm {} \;
```

**Good:**
```bash
find . -name "*.sh" -exec chmod +x "{}" \;
find /tmp -type f -exec rm "{}" \;
```

**Why it matters:** Filenames with spaces or special characters will break without quotes.

**Auto-fix:** Changes `{}` to `"{}"`

### SEC004: Hardcoded Credentials

**Severity:** Error
**Auto-fix:** No (manual review required)

Detects potential hardcoded passwords, API keys, or tokens in scripts.

**Bad:**
```bash
PASSWORD="MySecretPass123"
API_KEY="sk-1234567890abcdef"
TOKEN="ghp_xxxxxxxxxxxx"
```

**Good:**
```bash
# Read from environment
PASSWORD="${DB_PASSWORD:?}"

# Read from secure file
PASSWORD=$(cat /run/secrets/db_password)

# Use credential manager
PASSWORD=$(vault kv get -field=password secret/db)
```

**Detection patterns:**
- Variables named `PASSWORD`, `SECRET`, `TOKEN`, `API_KEY`
- Obvious credential assignment patterns

### SEC005: Command Substitution in Variables

**Severity:** Warning
**Auto-fix:** No (context-dependent)

Detects potentially dangerous command substitution that could execute unintended commands.

**Bad:**
```bash
FILE=$USER_INPUT
cat $(echo $FILE)  # Command injection if FILE contains $(...)
```

**Good:**
```bash
# Quote and validate
FILE="$USER_INPUT"
if [[ -f "$FILE" ]]; then
    cat "$FILE"
fi
```

### SEC006: Predictable Temporary Files

**Severity:** Warning
**Auto-fix:** Yes (suggests safer alternatives)

Detects use of predictable temporary file names (race condition vulnerability).

**Bad:**
```bash
TMP=/tmp/myapp.tmp        # Predictable
TMP=/tmp/app-$$           # Process ID predictable
TMP=/tmp/file-$RANDOM     # Not secure randomness
```

**Good:**
```bash
TMP=$(mktemp)                    # Secure
TMP=$(mktemp -d)                # Secure directory
TMP=$(mktemp /tmp/myapp.XXXXXX) # Template-based
```

**Auto-fix:** Suggests using `mktemp` or `mktemp -d`

### SEC007: World-Writable File Creation

**Severity:** Error
**Auto-fix:** No (must set appropriate permissions)

Detects creation of world-writable files or directories (permission 777, 666).

**Bad:**
```bash
chmod 777 /var/log/app.log  # Everyone can write
mkdir -m 777 /tmp/shared    # Insecure directory
```

**Good:**
```bash
chmod 644 /var/log/app.log  # Owner write, others read
chmod 755 /var/app          # Owner write, others execute
mkdir -m 700 /tmp/private   # Owner only
```

### SEC008: Piping curl/wget to Shell

**Severity:** Error
**Auto-fix:** No (manual review required)

Detects EXTREMELY DANGEROUS pattern of piping curl/wget directly to shell execution.

**Bad:**
```bash
curl https://install.sh | sh          # NEVER DO THIS
wget -qO- https://get.sh | bash       # EXTREMELY DANGEROUS
curl -sSL https://install.sh | sudo sh  # CRITICAL RISK
```

**Good:**
```bash
# Download first, inspect, then execute
curl -o install.sh https://install.sh
# INSPECT install.sh for malicious code
cat install.sh  # Review the script
chmod +x install.sh
./install.sh
```

**Why it matters:**
- MITM attacks can inject malicious code
- No opportunity to review what's being executed
- Server compromise = instant system compromise
- Sudo escalation compounds the risk

## Determinism Rules (DET001-DET003)

Determinism rules ensure scripts produce predictable, reproducible output.

### DET001: Non-deterministic $RANDOM Usage

**Severity:** Error
**Auto-fix:** Suggests alternatives (unsafe)

Detects `$RANDOM` which produces different output on each run.

**Bad:**
```bash
SESSION_ID=$RANDOM
FILE=output-$RANDOM.log
```

**Good:**
```bash
# Use version/build identifier
SESSION_ID="session-${VERSION}"

# Use timestamp as argument
SESSION_ID="$1"

# Use hash of input
SESSION_ID=$(echo "$INPUT" | sha256sum | cut -c1-8)
```

**Auto-fix suggestions:**
1. Use version/build ID
2. Pass value as argument
3. Use deterministic hash function

### DET002: Non-deterministic Timestamp Usage

**Severity:** Error
**Auto-fix:** Suggests alternatives (unsafe)

Detects timestamp generation that varies between runs.

**Bad:**
```bash
RELEASE="release-$(date +%s)"
BACKUP="backup-$(date +%Y%m%d-%H%M%S)"
```

**Good:**
```bash
# Use explicit version
RELEASE="release-${VERSION}"

# Pass timestamp as argument
RELEASE="release-$1"

# Use git commit hash
RELEASE="release-$(git rev-parse --short HEAD)"
```

**Detected patterns:**
- `$(date +%s)` (Unix timestamp)
- `$(date +%Y%m%d)` (date formatting)
- `$EPOCHSECONDS` (bash 5.0+)

### DET003: Non-deterministic Process ID

**Severity:** Error
**Auto-fix:** Suggests alternatives (unsafe)

Detects use of `$$` (process ID) which changes on every execution.

**Bad:**
```bash
LOCKFILE=/tmp/app-$$.lock
TMPDIR=/tmp/work-$$
```

**Good:**
```bash
# Use mktemp for temporary files
LOCKFILE=$(mktemp /tmp/app.lock.XXXXXX)

# Use application-specific identifier
LOCKFILE=/var/run/app-${APP_NAME}.lock
```

## Idempotency Rules (IDEM001-IDEM003)

Idempotency rules ensure scripts can be safely re-run without side effects.

### IDEM001: Non-idempotent mkdir

**Severity:** Warning
**Auto-fix:** Yes (safe with assumptions)

Detects `mkdir` without `-p` flag (fails if directory exists).

**Bad:**
```bash
mkdir /app/releases      # Fails on second run
mkdir /var/log/myapp     # Non-idempotent
```

**Good:**
```bash
mkdir -p /app/releases   # Succeeds if exists
mkdir -p /var/log/myapp  # Idempotent
```

**Auto-fix:** Adds `-p` flag: `mkdir -p`

**Assumption:** Directory creation failure is not critical

### IDEM002: Non-idempotent ln

**Severity:** Warning
**Auto-fix:** Yes (safe with assumptions)

Detects `ln -s` without force flag (fails if symlink exists).

**Bad:**
```bash
ln -s /app/releases/v1.0 /app/current  # Fails if exists
```

**Good:**
```bash
ln -sf /app/releases/v1.0 /app/current  # Overwrites if exists
# Or more explicit:
rm -f /app/current
ln -s /app/releases/v1.0 /app/current
```

**Auto-fix:** Adds `-f` flag: `ln -sf`

### IDEM003: Non-idempotent rm

**Severity:** Warning
**Auto-fix:** Yes (safe)

Detects `rm` without `-f` flag (may fail if file doesn't exist).

**Bad:**
```bash
rm /tmp/lockfile          # Fails if not exists
rm /var/run/app.pid       # Non-idempotent
```

**Good:**
```bash
rm -f /tmp/lockfile       # Succeeds if not exists
rm -f /var/run/app.pid    # Idempotent
```

**Auto-fix:** Adds `-f` flag: `rm -f`

## Config Rules (CONFIG-001 to CONFIG-003)

Config rules analyze shell configuration files (.bashrc, .zshrc, etc.).

### CONFIG-001: PATH Deduplication

**Severity:** Warning
**Auto-fix:** Yes (safe)

Detects duplicate entries in PATH variable.

**Bad:**
```bash
export PATH="/usr/local/bin:$PATH"
export PATH="/usr/local/bin:$PATH"  # Duplicate
export PATH="$HOME/.local/bin:$PATH"
export PATH="$HOME/.local/bin:$PATH"  # Duplicate
```

**Good:**
```bash
# Use function to deduplicate
dedupe_path() {
    echo "$PATH" | tr ':' '\n' | awk '!seen[$0]++' | tr '\n' ':'
}
export PATH=$(dedupe_path)

# Or add only if not present
case ":$PATH:" in
    *:/usr/local/bin:*) ;;
    *) export PATH="/usr/local/bin:$PATH" ;;
esac
```

**Auto-fix:** Removes duplicate PATH entries

### CONFIG-002: Quote Variables

**Severity:** Warning
**Auto-fix:** Yes (safe)

Detects unquoted variables in config files.

**Bad:**
```bash
export EDITOR=$HOME/bin/editor     # Word splitting
alias ll=ls -la $HOME              # Unquoted
```

**Good:**
```bash
export EDITOR="$HOME/bin/editor"
alias ll="ls -la $HOME"
```

**Auto-fix:** Adds double quotes around variables

### CONFIG-003: Consolidate Aliases

**Severity:** Style
**Auto-fix:** Yes (safe)

Detects duplicate alias definitions.

**Bad:**
```bash
alias ll="ls -l"
alias ll="ls -la"   # Overwrites previous
alias gs="git status"
alias gs="git status --short"  # Duplicate
```

**Good:**
```bash
# Keep only the final definition
alias ll="ls -la"
alias gs="git status --short"
```

**Auto-fix:** Removes duplicate aliases, keeps last definition

## Makefile Rules (MAKE001-MAKE020)

Makefile-specific rules for build system issues.

### MAKE001: Non-deterministic Wildcard

**Severity:** Warning
**Auto-fix:** Yes (safe)

Detects `$(wildcard ...)` without `$(sort ...)` (filesystem ordering varies).

**Bad:**
```makefile
SOURCES = $(wildcard *.c)        # Non-deterministic order
HEADERS = $(wildcard include/*.h)
```

**Good:**
```makefile
SOURCES = $(sort $(wildcard *.c))        # Deterministic
HEADERS = $(sort $(wildcard include/*.h))
```

**Auto-fix:** Wraps with `$(sort ...)`

### MAKE002: Unsafe Shell Variable

**Severity:** Warning
**Auto-fix:** Yes (safe)

Detects unquoted shell variables in Makefile recipes.

**Bad:**
```makefile
build:
\trm -rf $(OUTPUT)  # Make variable - OK
\trm -rf $OUTPUT    # Shell variable - unquoted
```

**Good:**
```makefile
build:
\trm -rf "$$OUTPUT"  # Quoted shell variable
```

### MAKE008: Tab vs Spaces

**Severity:** Error
**Auto-fix:** Yes (CRITICAL)

Detects spaces instead of tabs in recipe lines (causes Make errors).

**Bad:**
```makefile
build:
    echo "Building"  # Spaces instead of tab
```

**Good:**
```makefile
build:
\techo "Building"  # Tab character
```

**Why it matters:** Make REQUIRES tabs, not spaces. This is a syntax error.

**Auto-fix:** Converts leading spaces to tabs in recipe lines

### Additional Makefile Rules

bashrs implements 20 Makefile rules (MAKE001-MAKE020) covering:
- Determinism issues (wildcards, timestamps)
- Shell safety (quoting, escaping)
- Build correctness (tabs, dependencies)
- POSIX compliance
- Best practices (.PHONY targets, etc.)

See [Makefile Best Practices](../makefile/best-practices.md) for details.

## ShellCheck Integration

bashrs integrates 324+ ShellCheck rules for comprehensive shell script analysis.

### Critical ShellCheck Rules

#### SC2086: Quote to Prevent Word Splitting

**Severity:** Error
**Auto-fix:** Yes (safe)

The MOST IMPORTANT rule - prevents word splitting and globbing.

**Bad:**
```bash
rm $FILE            # If FILE="a b", removes "a" and "b"
cp $SRC $DST        # Word splitting risk
echo $PATH          # Glob expansion risk
```

**Good:**
```bash
rm "$FILE"          # Treats as single argument
cp "$SRC" "$DST"    # Safe
echo "$PATH"        # Quoted
```

**Impact:** This single rule prevents the majority of shell scripting bugs.

#### SC2046: Quote to Prevent Word Splitting in $()

**Severity:** Error
**Auto-fix:** Yes (safe)

Similar to SC2086 but for command substitution.

**Bad:**
```bash
rm $(find . -name "*.tmp")  # Breaks with spaces
```

**Good:**
```bash
find . -name "*.tmp" -delete  # Native find solution
# Or:
while IFS= read -r file; do
    rm "$file"
done < <(find . -name "*.tmp")
```

#### SC2059: Printf Format Injection

**Severity:** Error
**Auto-fix:** Yes (CRITICAL security)

Prevents format string injection in printf.

**Bad:**
```bash
printf "$USER_INPUT"    # Format injection
printf "Error: $MSG\n"  # MSG could contain %s
```

**Good:**
```bash
printf '%s\n' "$USER_INPUT"  # Safe
printf 'Error: %s\n' "$MSG"  # Explicit format
```

#### SC2064: Trap Quote Timing

**Severity:** Error
**Auto-fix:** Yes (CRITICAL bug)

Ensures trap commands quote correctly to expand at trap time, not definition time.

**Bad:**
```bash
trap "rm $TMPFILE" EXIT  # Expands NOW, not at exit
```

**Good:**
```bash
trap 'rm "$TMPFILE"' EXIT  # Expands at exit time
```

### ShellCheck Rule Categories

bashrs implements ShellCheck rules across categories:

| Category | Example Rules | Count |
|----------|---------------|-------|
| Quoting | SC2086, SC2046, SC2068 | 30+ |
| Variables | SC2034, SC2154, SC2155 | 25+ |
| Arrays | SC2198, SC2199, SC2200 | 15+ |
| Conditionals | SC2166, SC2181, SC2244 | 20+ |
| Loops | SC2044, SC2045, SC2162 | 15+ |
| Functions | SC2119, SC2120, SC2128 | 10+ |
| Redirects | SC2094, SC2095, SC2069 | 10+ |
| Security | SC2115, SC2164, SC2230 | 15+ |
| POSIX | SC2039, SC2169, SC2295 | 20+ |
| Deprecations | SC2006, SC2016, SC2027 | 10+ |

**Total:** 324+ rules implemented (and growing)

### Shell Type Detection

bashrs automatically detects shell type and applies appropriate rules:

**POSIX sh:**
- Skips bash-only rules (arrays, `[[`, etc.)
- Enforces strict POSIX compliance
- Warns about bashisms

**Bash:**
- Enables bash-specific rules
- Checks array usage
- Validates bash 3.2+ features

**Zsh:**
- Zsh-specific rules
- Array syntax differences
- Extended features

**Detection methods:**
1. Shebang (`#!/bin/bash`, `#!/bin/sh`)
2. File extension (`.bash`, `.sh`)
3. Filename pattern (`.bashrc`, `.zshrc`)
4. Content analysis (bash-specific syntax)

## Rule Severity Levels

bashrs uses three severity levels:

### Error

**Impact:** Blocks CI/CD, prevents deployment

**Rules:**
- All security rules (SEC001-SEC008)
- All determinism rules (DET001-DET003)
- Critical ShellCheck rules (SC2086, SC2046, SC2059, SC2064)

**Example:**
```bash
$ bashrs lint insecure.sh
error[SEC001]: Command injection risk via eval
  --> insecure.sh:5:1
```

**Exit code:** 3 (validation error)

### Warning

**Impact:** Should be fixed, but not blocking

**Rules:**
- Idempotency rules (IDEM001-IDEM003)
- Config rules (CONFIG-001 to CONFIG-003)
- Non-critical ShellCheck rules

**Example:**
```bash
$ bashrs lint script.sh
warning[IDEM001]: Non-idempotent mkdir - add -p flag
  --> script.sh:10:1
```

**Exit code:** 0 (warnings don't fail by default)

### Style

**Impact:** Cosmetic, best practices

**Rules:**
- Code formatting
- Alias consolidation (CONFIG-003)
- Stylistic preferences

**Example:**
```bash
$ bashrs lint config.sh
style[CONFIG-003]: Consolidate duplicate aliases
  --> .bashrc:45:1
```

## Auto-Fix Capabilities

bashrs provides three types of auto-fixes:

### Safe Auto-Fix

**Guaranteed safe** - no semantic changes

**Examples:**
- Adding quotes: `$VAR` → `"$VAR"`
- Adding flags: `mkdir` → `mkdir -p`
- Format strings: `printf "$msg"` → `printf '%s' "$msg"`

**Enable:**
```bash
bashrs lint --fix script.sh
```

**Config:**
```toml
[linter]
auto_fix = true
safe_auto_fix_only = true
```

### Safe With Assumptions

**Safe if assumptions hold** - documented assumptions

**Examples:**
- `mkdir -p` (assumes dir creation failure not critical)
- `ln -sf` (assumes overwriting symlink is safe)

**Assumptions documented** in fix output

### Unsafe (Manual Review Required)

**Requires human judgment** - provides suggestions only

**Examples:**
- `eval` removal (context-dependent)
- `$RANDOM` replacement (depends on use case)
- Credential handling (requires architecture change)

**Output:**
```bash
error[DET001]: Non-deterministic $RANDOM usage
  Suggestions:
    1. Use version ID: SESSION_ID="session-${VERSION}"
    2. Pass as argument: SESSION_ID="$1"
    3. Use hash: SESSION_ID=$(echo "$INPUT" | sha256sum)
```

## Disabling Rules

### Inline Comments

Disable specific rules on specific lines:

```bash
# shellcheck disable=SC2086
rm $FILES  # Intentional word splitting

# bashrs-disable-next-line DET002
RELEASE="release-$(date +%s)"  # Timestamp needed here
```

### Configuration File

Disable rules project-wide:

```toml
# bashrs.toml
[linter]
disabled_rules = [
    "SC2034",   # Allow unused variables
    "SC2154",   # Variables from sourced files
    "DET002",   # Timestamps allowed in this project
]
```

### Environment Variable

Disable rules at runtime:

```bash
export BASHRS_DISABLE_RULES="SC2119,SC2120,DET002"
bashrs lint script.sh
```

### CLI Argument

Disable rules per invocation:

```bash
bashrs lint --disable SC2034,SC2154 script.sh
```

## Custom Rule Development

bashrs supports custom rules through plugins (future feature).

### Rule Interface

```rust,ignore
pub trait LintRule {
    fn check(&self, source: &str) -> LintResult;
    fn code(&self) -> &str;
    fn severity(&self) -> Severity;
    fn auto_fix(&self) -> Option<Fix>;
}
```

### Example Custom Rule

```rust,ignore
pub struct CustomRule001;

impl LintRule for CustomRule001 {
    fn check(&self, source: &str) -> LintResult {
        let mut result = LintResult::new();

        for (line_num, line) in source.lines().enumerate() {
            if line.contains("forbidden_pattern") {
                let diag = Diagnostic::new(
                    "CUSTOM001",
                    Severity::Error,
                    "Forbidden pattern detected",
                    Span::new(line_num + 1, 1, line_num + 1, line.len()),
                );
                result.add(diag);
            }
        }

        result
    }

    fn code(&self) -> &str { "CUSTOM001" }
    fn severity(&self) -> Severity { Severity::Error }
    fn auto_fix(&self) -> Option<Fix> { None }
}
```

**Plugin location:**
```text
~/.config/bashrs/plugins/custom_rules.so
```

**Load in config:**
```toml
[linter]
plugins = ["custom_rules"]
```

## Summary

bashrs provides comprehensive linting across 350+ rules:

**Security (8 rules):**
- Command injection prevention
- Credential security
- File permission safety

**Determinism (3 rules):**
- Reproducible output
- Predictable behavior

**Idempotency (3 rules):**
- Safe re-execution
- No side effects

**Config (3 rules):**
- Shell configuration best practices

**Makefile (20 rules):**
- Build system correctness

**ShellCheck (324+ rules):**
- Comprehensive shell script analysis

**Key Features:**
1. Auto-fix for 200+ rules
2. Shell type detection
3. Severity levels (Error, Warning, Style)
4. Flexible rule disabling
5. CI/CD integration
6. Custom rule support (coming soon)

For more information, see:
- [Security Rules Deep Dive](../linting/security.md)
- [Determinism Rules](../linting/determinism.md)
- [Idempotency Rules](../linting/idempotency.md)
- [Configuration Reference](./configuration.md)
- [Exit Codes Reference](./exit-codes.md)
