# POSIX Compliance

**POSIX Compliance** means writing shell scripts that follow the Portable Operating System Interface (POSIX) standard, ensuring they work on any POSIX-compliant shell. These scripts are portable, predictable, and work everywhere from minimal Alpine containers to enterprise Unix systems.

## Definition

A shell script is **POSIX compliant** if and only if:

**Runs on any POSIX shell (sh, dash, ash, busybox sh, bash, ksh, zsh)**

**Formula**: `shellcheck -s sh script.sh` (passes without errors)

## Why POSIX Compliance Matters

### The Problem: Bash-Specific Scripts

Bash-specific scripts use non-standard features that break portability:

```bash
#!/bin/bash
# Bash-specific (NOT portable)

# Bash arrays (not POSIX)
declare -a servers=("web1" "web2" "web3")
for server in "${servers[@]}"; do
    echo "$server"
done

# [[ ]] test (not POSIX)
if [[ "$VAR" == "value" ]]; then
    echo "match"
fi

# String manipulation (not POSIX)
filename="report.txt"
echo "${filename%.txt}"  # report

# Process substitution (not POSIX)
diff <(ls dir1) <(ls dir2)
```

**Problems**:
- **Fails on Alpine Linux** (uses busybox sh, not bash)
- **Fails on minimal containers** (no bash installed)
- **Fails on BSD/Unix** (sh is not bash)
- **Fails on embedded systems** (dash or ash, not bash)

**Error Example**:
```
/bin/sh: line 3: declare: not found
/bin/sh: line 7: syntax error: unexpected "("
```

### The Solution: POSIX-Compliant Scripts

POSIX scripts work everywhere:

```bash
#!/bin/sh
# POSIX-compliant (portable)

# Space-separated lists (POSIX)
servers="web1 web2 web3"
for server in $servers; do
    echo "$server"
done

# [ ] test (POSIX)
if [ "$VAR" = "value" ]; then
    echo "match"
fi

# Parameter expansion (POSIX subset)
filename="report.txt"
basename "$filename" .txt  # report

# Named pipes (POSIX)
mkfifo /tmp/pipe1 /tmp/pipe2
ls dir1 > /tmp/pipe1 &
ls dir2 > /tmp/pipe2 &
diff /tmp/pipe1 /tmp/pipe2
```

**Benefits**:
- ✅ **Portable**: Runs on Alpine, Debian, Ubuntu, BSD, macOS, embedded
- ✅ **Minimal**: Works without bash installed
- ✅ **Standard**: Follows POSIX specification
- ✅ **Verified**: Passes `shellcheck -s sh`

## Common Bash-isms to Avoid

Rash detects and eliminates these non-POSIX patterns:

### 1. Bash Arrays

**Problem**: Arrays are bash-specific

```bash
# ❌ BAD: Bash arrays (not POSIX)
declare -a files=("a.txt" "b.txt" "c.txt")
for file in "${files[@]}"; do
    echo "$file"
done
```

**Solution**: Use space-separated lists

```bash
# ✅ GOOD: Space-separated (POSIX)
files="a.txt b.txt c.txt"
for file in $files; do
    echo "$file"
done

# Or line-separated with read
printf '%s\n' "a.txt" "b.txt" "c.txt" | while read -r file; do
    echo "$file"
done
```

### 2. [[ ]] Double Brackets

**Problem**: [[ ]] is bash-specific

```bash
# ❌ BAD: Double brackets (not POSIX)
if [[ "$VAR" == "value" ]]; then
    echo "match"
fi

if [[ -f "$FILE" && -r "$FILE" ]]; then
    echo "file is readable"
fi
```

**Solution**: Use [ ] single brackets

```bash
# ✅ GOOD: Single brackets (POSIX)
if [ "$VAR" = "value" ]; then
    echo "match"
fi

if [ -f "$FILE" ] && [ -r "$FILE" ]; then
    echo "file is readable"
fi
```

### 3. String Manipulation

**Problem**: ${var%.ext} and ${var#prefix} are bash-specific (beyond POSIX)

```bash
# ❌ BAD: Bash string ops (not POSIX)
filename="report.txt"
echo "${filename%.txt}"  # report
echo "${filename#/tmp/}" # removes /tmp/ prefix
```

**Solution**: Use POSIX commands

```bash
# ✅ GOOD: basename and dirname (POSIX)
filename="report.txt"
basename "$filename" .txt  # report

path="/tmp/file.txt"
dirname "$path"   # /tmp
basename "$path"  # file.txt
```

### 4. Process Substitution

**Problem**: <(...) is bash-specific

```bash
# ❌ BAD: Process substitution (not POSIX)
diff <(ls dir1) <(ls dir2)
```

**Solution**: Use temporary files or named pipes

```bash
# ✅ GOOD: Temporary files (POSIX)
ls dir1 > /tmp/ls1
ls dir2 > /tmp/ls2
diff /tmp/ls1 /tmp/ls2
rm -f /tmp/ls1 /tmp/ls2

# Or named pipes (POSIX)
mkfifo /tmp/pipe1 /tmp/pipe2
ls dir1 > /tmp/pipe1 &
ls dir2 > /tmp/pipe2 &
diff /tmp/pipe1 /tmp/pipe2
rm -f /tmp/pipe1 /tmp/pipe2
```

### 5. == Equality Operator

**Problem**: == is bash-specific

```bash
# ❌ BAD: == operator (not POSIX)
if [ "$VAR" == "value" ]; then
    echo "match"
fi
```

**Solution**: Use = operator

```bash
# ✅ GOOD: = operator (POSIX)
if [ "$VAR" = "value" ]; then
    echo "match"
fi
```

### 6. Local Variables

**Problem**: `local` keyword is not POSIX (though widely supported)

```bash
# ❌ BAD: local keyword (not POSIX)
my_function() {
    local temp="value"
    echo "$temp"
}
```

**Solution**: Use naming conventions or accept it as widely-supported

```bash
# ✅ GOOD: Naming convention (POSIX)
my_function() {
    _my_function_temp="value"
    echo "$_my_function_temp"
}

# Or accept `local` as de-facto standard
# (Supported by dash, bash, ksh, zsh - just not in POSIX spec)
my_function() {
    local temp="value"  # Widely supported
    echo "$temp"
}
```

## POSIX Shell Features

What you CAN use safely in POSIX sh:

### Core Commands

```bash
# File operations (POSIX)
cat file.txt
cp source dest
mv old new
rm file
mkdir -p dir
ln -s target link

# Text processing (POSIX)
grep "pattern" file
sed 's/old/new/g' file
awk '{print $1}' file
cut -d: -f1 file
sort file
uniq file
```

### Variables and Quoting

```bash
# Variable assignment (POSIX)
VAR="value"
VAR="${OTHER:-default}"

# Always quote variables (POSIX best practice)
echo "$VAR"
cp "$SOURCE" "$DEST"

# Parameter expansion (POSIX subset)
${VAR}          # Variable expansion
${VAR:-default} # Default if unset
${VAR:=default} # Assign default if unset
${VAR:?error}   # Error if unset
${VAR:+value}   # Value if set
```

### Control Flow

```bash
# if statements (POSIX)
if [ "$VAR" = "value" ]; then
    echo "match"
elif [ "$VAR" = "other" ]; then
    echo "other"
else
    echo "default"
fi

# case statements (POSIX)
case "$VAR" in
    pattern1)
        echo "first"
        ;;
    pattern2|pattern3)
        echo "second or third"
        ;;
    *)
        echo "default"
        ;;
esac

# Loops (POSIX)
for i in 1 2 3; do
    echo "$i"
done

while read -r line; do
    echo "$line"
done < file.txt
```

### Functions

```bash
# POSIX function syntax
my_function() {
    arg1="$1"
    arg2="$2"

    echo "Processing $arg1 and $arg2"

    return 0
}

# Call function
my_function "value1" "value2"
```

## Testing POSIX Compliance

### Verification with shellcheck

Every POSIX script must pass shellcheck:

```bash
# Verify POSIX compliance
shellcheck -s sh script.sh

# No errors = POSIX compliant ✅
```

**Example Output (Non-Compliant)**:
```
script.sh:3:1: error: declare is not POSIX sh [SC3044]
script.sh:7:4: error: [[ ]] is not POSIX sh [SC3010]
script.sh:11:6: error: ${var%.ext} is not POSIX sh [SC3060]
```

**Example Output (Compliant)**:
```
# No issues found ✅
```

### Multi-Shell Testing

Test on all major POSIX shells:

```bash
#!/bin/sh
# Test script on multiple shells

for shell in sh dash ash bash ksh zsh; do
    echo "Testing with: $shell"
    if command -v "$shell" > /dev/null; then
        $shell script.sh && echo "✅ $shell: PASS" || echo "❌ $shell: FAIL"
    else
        echo "⏭️  $shell: Not installed"
    fi
done
```

**Expected Output**:
```
Testing with: sh
✅ sh: PASS
Testing with: dash
✅ dash: PASS
Testing with: ash
✅ ash: PASS
Testing with: bash
✅ bash: PASS
Testing with: ksh
✅ ksh: PASS
Testing with: zsh
✅ zsh: PASS
```

### Container Testing

Test in minimal Alpine container (busybox sh):

```bash
# Test in Alpine (busybox sh)
docker run --rm -v "$(pwd):/scripts" alpine:latest sh /scripts/script.sh

# Expected: Script runs successfully ✅
```

### Property Test: Cross-Shell Consistency

```bash
#!/bin/sh
# Test: Same output on all shells

# Run on sh
sh script.sh v1.0.0 > output_sh.txt

# Run on dash
dash script.sh v1.0.0 > output_dash.txt

# Run on bash
bash script.sh v1.0.0 > output_bash.txt

# Verify identical output
if diff output_sh.txt output_dash.txt && \
   diff output_dash.txt output_bash.txt; then
    echo "PASS: All shells produce identical output (POSIX compliant ✅)"
else
    echo "FAIL: Output differs between shells (not POSIX compliant)"
    exit 1
fi
```

## Purification Transforms

Rash purification automatically converts bash-isms to POSIX:

### Before: Bash-Specific

```bash
#!/bin/bash
# Bash-specific script

# Bash arrays
declare -a servers=("web1" "web2" "web3")

# Double brackets
if [[ -f "$CONFIG" && -r "$CONFIG" ]]; then
    echo "Config exists"
fi

# String manipulation
filename="${CONFIG%.conf}"

# Process substitution
diff <(ps aux) <(ps aux)
```

### After: POSIX-Compliant

```bash
#!/bin/sh
# Purified by Rash v6.30.1 (POSIX compliant)

# Space-separated list
servers="web1 web2 web3"

# Single brackets
if [ -f "$CONFIG" ] && [ -r "$CONFIG" ]; then
    echo "Config exists"
fi

# basename command
filename=$(basename "$CONFIG" .conf)

# Temporary files
ps aux > /tmp/ps1
ps aux > /tmp/ps2
diff /tmp/ps1 /tmp/ps2
rm -f /tmp/ps1 /tmp/ps2
```

**Transformations**:
- ✅ `declare -a` → space-separated list
- ✅ `[[ ]]` → `[ ]`
- ✅ `${var%.ext}` → `basename`
- ✅ `<(...)` → temporary files
- ✅ `#!/bin/bash` → `#!/bin/sh`

## Best Practices

### 1. Always Use #!/bin/sh Shebang

```bash
# ❌ BAD: Bash shebang (non-portable)
#!/bin/bash

# ✅ GOOD: POSIX sh shebang (portable)
#!/bin/sh
```

### 2. Use [ ] Not [[ ]]

```bash
# ❌ BAD: Double brackets
if [[ "$VAR" == "value" ]]; then
    echo "match"
fi

# ✅ GOOD: Single brackets
if [ "$VAR" = "value" ]; then
    echo "match"
fi
```

### 3. Use = Not ==

```bash
# ❌ BAD: == operator
if [ "$VAR" == "value" ]; then

# ✅ GOOD: = operator
if [ "$VAR" = "value" ]; then
```

### 4. Avoid Bash Arrays

```bash
# ❌ BAD: Bash arrays
files=("a.txt" "b.txt")

# ✅ GOOD: Space-separated lists
files="a.txt b.txt"
```

### 5. Use POSIX Commands Only

```bash
# ❌ BAD: Bash builtins
echo "${var%.txt}"

# ✅ GOOD: POSIX commands
basename "$var" .txt
```

### 6. Always Quote Variables

```bash
# ❌ BAD: Unquoted variables
cp $SOURCE $DEST

# ✅ GOOD: Quoted variables
cp "$SOURCE" "$DEST"
```

### 7. Verify with shellcheck

```bash
# Always run before release
shellcheck -s sh script.sh

# Must pass with zero errors ✅
```

## Common Patterns

### Pattern 1: Iterating Lists

```bash
# Bash (non-portable)
declare -a items=("a" "b" "c")
for item in "${items[@]}"; do
    echo "$item"
done

# POSIX (portable)
items="a b c"
for item in $items; do
    echo "$item"
done

# Or with newlines
printf '%s\n' "a" "b" "c" | while read -r item; do
    echo "$item"
done
```

### Pattern 2: Checking File Existence

```bash
# Bash (works but non-standard)
if [[ -f "$FILE" ]]; then
    echo "exists"
fi

# POSIX (portable)
if [ -f "$FILE" ]; then
    echo "exists"
fi
```

### Pattern 3: Default Values

```bash
# Both work, but POSIX uses different syntax

# Bash
VAR="${1:-default}"

# POSIX (same syntax!)
VAR="${1:-default}"

# POSIX shell supports this parameter expansion ✅
```

### Pattern 4: String Comparison

```bash
# Bash (non-standard ==)
if [ "$VAR" == "value" ]; then
    echo "match"
fi

# POSIX (standard =)
if [ "$VAR" = "value" ]; then
    echo "match"
fi
```

## Integration with Purification

POSIX compliance is the third pillar of purification:

```bash
#!/bin/sh
# Deterministic + Idempotent + POSIX = Purified

deploy() {
    version="${1}"  # Deterministic: parameter, not $RANDOM

    # Idempotent: mkdir -p, rm -f
    mkdir -p "/app/releases/${version}"
    rm -f "/app/current"
    ln -s "/app/releases/${version}" "/app/current"

    # POSIX: Works on sh, dash, ash, busybox, bash
    echo "Deployed ${version}"
}

deploy "${1}"
```

**Properties**:
- ✅ Deterministic: Same input → same output
- ✅ Idempotent: Safe to re-run
- ✅ POSIX: Works on all shells

**Verification**:
```bash
# Test determinism
sh script.sh v1.0.0 > out1.txt
sh script.sh v1.0.0 > out2.txt
diff out1.txt out2.txt  # Identical ✅

# Test idempotency
sh script.sh v1.0.0
sh script.sh v1.0.0  # No errors ✅

# Test POSIX compliance
shellcheck -s sh script.sh  # No errors ✅
dash script.sh v1.0.0        # Works ✅
ash script.sh v1.0.0         # Works ✅
```

## Compatibility Matrix

| Feature | Bash | POSIX sh | Dash | Ash | Busybox | Status |
|---------|------|----------|------|-----|---------|--------|
| `[ ]` test | ✅ | ✅ | ✅ | ✅ | ✅ | Use this |
| `[[ ]]` test | ✅ | ❌ | ❌ | ❌ | ❌ | Avoid |
| Arrays | ✅ | ❌ | ❌ | ❌ | ❌ | Avoid |
| `=` comparison | ✅ | ✅ | ✅ | ✅ | ✅ | Use this |
| `==` comparison | ✅ | ⚠️ | ⚠️ | ⚠️ | ⚠️ | Avoid |
| `local` keyword | ✅ | ⚠️ | ✅ | ✅ | ✅ | Widely supported |
| `${var%.ext}` | ✅ | ⚠️ | ✅ | ✅ | ✅ | Limited POSIX |
| `${var:-default}` | ✅ | ✅ | ✅ | ✅ | ✅ | Use this |
| Process substitution | ✅ | ❌ | ❌ | ❌ | ❌ | Avoid |
| Functions | ✅ | ✅ | ✅ | ✅ | ✅ | Use this |

Legend:
- ✅ Fully supported
- ⚠️ Not in POSIX spec, but widely supported
- ❌ Not supported

## Verification Checklist

Before marking a script as POSIX compliant:

- [ ] ✅ **Shebang**: Uses `#!/bin/sh` (not `#!/bin/bash`)
- [ ] ✅ **Shellcheck**: Passes `shellcheck -s sh` with zero errors
- [ ] ✅ **No arrays**: Uses space-separated lists instead
- [ ] ✅ **Single brackets**: Uses `[ ]` not `[[ ]]`
- [ ] ✅ **= operator**: Uses `=` not `==`
- [ ] ✅ **POSIX commands**: No bash builtins
- [ ] ✅ **Multi-shell**: Tested on sh, dash, ash, bash
- [ ] ✅ **Container**: Runs in Alpine (busybox sh)

## Real-World Usage

### Minimal Docker Images

```dockerfile
# Alpine base (5 MB) - uses busybox sh
FROM alpine:latest

COPY deploy_purified.sh /deploy.sh

# Works because script is POSIX-compliant
RUN sh /deploy.sh
```

### Bootstrap Scripts

```bash
#!/bin/sh
# Bootstrap installer (POSIX-compliant)
# Works on any Unix system

set -e

# Detect OS
if [ -f /etc/alpine-release ]; then
    OS="alpine"
elif [ -f /etc/debian_version ]; then
    OS="debian"
else
    OS="unknown"
fi

# Install based on OS
case "$OS" in
    alpine)
        apk add --no-cache myapp
        ;;
    debian)
        apt-get update
        apt-get install -y myapp
        ;;
    *)
        echo "Unsupported OS"
        exit 1
        ;;
esac

echo "Installation complete"
```

### CI/CD Pipelines

```bash
#!/bin/sh
# CI deploy script (POSIX-compliant)
# Runs on GitLab, GitHub Actions, Jenkins

version="${1}"

# Idempotent deployment
mkdir -p "/app/releases/${version}"
rm -f /app/current
ln -s "/app/releases/${version}" /app/current

# POSIX-compliant logging
echo "Deployed ${version} at $(date)"
```

## Further Reading

- [Purification Overview](./purification.md) - Complete purification process
- [Determinism Concept](./determinism.md) - Predictable script behavior
- [Idempotency Concept](./idempotency.md) - Safe re-run operations
- [POSIX Standard](https://pubs.opengroup.org/onlinepubs/9699919799/) - Official specification

---

**Key Takeaway**: POSIX compliance ensures portability. Use `#!/bin/sh`, avoid bash-isms, test with `shellcheck -s sh`, and verify on multiple shells (sh, dash, ash, busybox).
