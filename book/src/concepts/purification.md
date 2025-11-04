# What is Purification?

**Purification** is the process of transforming messy, unsafe, non-deterministic bash scripts into clean, safe, deterministic POSIX shell scripts. It's the core philosophy of Rash (bashrs).

## Overview

Purification combines three fundamental properties:

1. **Determinism** - Remove all sources of randomness
2. **Idempotency** - Make operations safe to re-run
3. **POSIX Compliance** - Generate standard, portable shell

**Formula**: `Purification = Determinism + Idempotency + POSIX Compliance`

## Why Purification Matters

Real-world bash scripts accumulate problems over time:

❌ **Problems with unpurified scripts**:
- **Non-deterministic** - Different output each run ($RANDOM, timestamps)
- **Non-idempotent** - Breaks when re-run (mkdir without -p, rm without -f)
- **Unsafe** - Vulnerable to injection (unquoted variables)
- **Non-portable** - Uses bash-isms instead of POSIX

✅ **Benefits of purified scripts**:
- **Predictable** - Same input always produces same output
- **Reliable** - Safe to re-run without errors
- **Secure** - All variables quoted, no injection vectors
- **Portable** - Runs on any POSIX shell (sh, dash, ash, busybox)

## The Purification Pipeline

Rash follows a systematic 3-stage pipeline:

```
┌─────────────┐      ┌──────────────┐      ┌─────────────────┐
│ Messy Bash  │  →   │  Transform   │  →   │  Purified POSIX │
│ Script      │      │  (Parse AST) │      │  Shell          │
└─────────────┘      └──────────────┘      └─────────────────┘
   Input                 Process                 Output
```

### Stage 1: Parse

Parse the messy bash script into an Abstract Syntax Tree (AST):

```bash
# Input: Messy bash
#!/bin/bash
SESSION_ID=$RANDOM
mkdir /tmp/session$SESSION_ID
```

Parsed as:
- Variable assignment: `SESSION_ID=$RANDOM` (non-deterministic!)
- Command: `mkdir /tmp/session$SESSION_ID` (non-idempotent!)

### Stage 2: Transform

Apply semantic transformations to enforce determinism and idempotency:

**Transformation Rules**:
1. Replace `$RANDOM` with function parameters
2. Replace timestamps with fixed values or parameters
3. Add `-p` flag to `mkdir` (idempotent)
4. Add `-f` flag to `rm` (idempotent)
5. Quote all variables (safety)
6. Remove bash-isms (POSIX compliance)

### Stage 3: Generate

Emit purified POSIX shell:

```bash
# Output: Purified POSIX sh
#!/bin/sh
session_id="${1}"
mkdir -p "/tmp/session${session_id}"
```

**Improvements**:
- ✅ Deterministic: Uses parameter instead of $RANDOM
- ✅ Idempotent: `mkdir -p` won't fail if directory exists
- ✅ Safe: Variables quoted
- ✅ POSIX: Runs on any shell

## Complete Example

### Input: Messy Deployment Script

```bash
#!/bin/bash
# deploy.sh - PROBLEMATIC

# Non-deterministic
SESSION_ID=$RANDOM
RELEASE="release-$(date +%s)"
PID=$$

# Non-idempotent
mkdir /app/releases/$RELEASE
mkdir /app/logs

# Unsafe (unquoted variables)
rm /app/current
ln -s /app/releases/$RELEASE /app/current

# Non-POSIX (bash arrays)
declare -a servers=("web1" "web2" "web3")

echo "Deployed $RELEASE to ${#servers[@]} servers (PID: $PID)"
```

**Problems**:
1. `$RANDOM` - Different value each time
2. `$(date +%s)` - Timestamp changes every second
3. `$$` - Process ID varies
4. `mkdir` - Fails if directory exists
5. `rm` - Fails if file doesn't exist
6. Unquoted `$RELEASE` - Unsafe
7. `declare -a` - Bash-specific, not POSIX

### Output: Purified Script

```bash
#!/bin/sh
# Purified by Rash v6.30.1

deploy_app() {
    # Input parameters (replaces non-deterministic sources)
    _version="${1}"
    _session_id="${2:-default-session}"

    # Deterministic release name
    release="release-${_version}"

    # Idempotent directory creation
    mkdir -p "/app/releases/${release}"
    mkdir -p "/app/logs"

    # Idempotent file operations
    rm -f "/app/current"

    # Idempotent symlink (remove old, create new)
    rm -f "/app/current"
    ln -s "/app/releases/${release}" "/app/current"

    # POSIX-compliant (no arrays, use space-separated list)
    servers="web1 web2 web3"
    server_count=0
    for server in $servers; do
        server_count=$((server_count + 1))
    done

    echo "Deployed ${release} to ${server_count} servers"
}

# Call with version parameter
deploy_app "${1}" "${2}"
```

**Transformations Applied**:
1. ✅ `$RANDOM` → Parameter `${2}`
2. ✅ `$(date +%s)` → Version parameter `${1}`
3. ✅ `$$` → Removed (not needed in purified version)
4. ✅ `mkdir` → `mkdir -p` (idempotent)
5. ✅ `rm` → `rm -f` (idempotent)
6. ✅ All variables quoted: `"${release}"`
7. ✅ Bash array → POSIX loop with space-separated list
8. ✅ Wrapped in function for reusability

## Purification Report

After purification, Rash generates a report:

```
Purification Report
===================

Input:  deploy.sh (18 lines, 412 bytes)
Output: deploy_purified.sh (32 lines, 687 bytes)

Issues Fixed: 7
✅ Replaced $RANDOM with parameter (line 5)
✅ Replaced $(date +%s) with parameter (line 6)
✅ Made mkdir idempotent with -p flag (line 10, 11)
✅ Made rm idempotent with -f flag (line 14)
✅ Quoted all variable references (lines 6, 10, 11, 14, 15, 22)
✅ Converted bash array to POSIX loop (line 18)

Quality Checks:
✅ Deterministic: No $RANDOM, timestamps, or process IDs
✅ Idempotent: Safe to re-run without errors
✅ POSIX Compliant: Passes shellcheck -s sh
✅ Security: All variables quoted
```

## Verification

Purified scripts must pass rigorous verification:

### 1. Shellcheck Validation

Every purified script MUST pass POSIX shellcheck:

```bash
shellcheck -s sh deploy_purified.sh
# No errors - POSIX compliant ✅
```

### 2. Behavioral Equivalence

Purified script must behave identically to original:

```bash
# Test original bash
bash deploy.sh 1.0.0 > original_output.txt

# Test purified sh
sh deploy_purified.sh 1.0.0 default-session > purified_output.txt

# Verify outputs are equivalent
diff original_output.txt purified_output.txt
# No differences - behaviorally equivalent ✅
```

### 3. Multi-Shell Testing

Purified scripts must work on all POSIX shells:

```bash
# Test on multiple shells
for shell in sh dash ash bash busybox; do
    echo "Testing with: $shell"
    $shell deploy_purified.sh 1.0.0
done

# All shells succeed ✅
```

### 4. Idempotency Testing

Must be safe to run multiple times:

```bash
# Run twice
sh deploy_purified.sh 1.0.0
sh deploy_purified.sh 1.0.0  # Should not fail

# Exit code: 0 - Safe to re-run ✅
```

## Limitations and Trade-offs

Purification has intentional trade-offs:

### What Purification CAN Fix

✅ Non-deterministic patterns ($RANDOM, timestamps, $$)
✅ Non-idempotent operations (mkdir, rm, ln)
✅ Unquoted variables
✅ Basic bash-isms (arrays, [[ ]], string manipulation)
✅ Security issues (command injection vectors)

### What Purification CANNOT Fix

❌ Complex bash features (associative arrays, co-processes)
❌ Bash-specific syntax that has no POSIX equivalent
❌ Logic errors in the original script
❌ Performance optimizations (purified code may be slightly slower)
❌ External dependencies (if script calls non-POSIX tools)

### Trade-offs

**Readability vs. Safety**:
- Purified code may be more verbose
- Extra quoting reduces readability slightly
- But safety and reliability are worth it

**Performance vs. Portability**:
- POSIX code may be slower than bash-specific features
- But portability enables running on minimal systems (Alpine, embedded)

**Determinism vs. Flexibility**:
- Removing $RANDOM requires passing seeds/values as parameters
- But determinism enables reproducible deployments

## Use Cases

Purification is ideal for:

### 1. CI/CD Pipelines

Ensure deployment scripts are deterministic and idempotent:

```bash
# Before: Non-deterministic deploy
./deploy.sh  # May behave differently each time

# After: Deterministic deploy
./deploy_purified.sh v1.2.3 session-ci-build-456  # Always same behavior
```

### 2. Configuration Management

Generate safe configuration scripts:

```bash
# Before: Breaks on re-run
mkdir /etc/myapp
echo "config=value" > /etc/myapp/config.conf

# After: Safe to re-run
mkdir -p /etc/myapp
echo "config=value" > /etc/myapp/config.conf
```

### 3. Container Initialization

Bootstrap scripts for minimal container images:

```bash
# Purified scripts run on Alpine (uses busybox sh)
FROM alpine:latest
COPY init_purified.sh /init.sh
RUN sh /init.sh
```

### 4. Legacy Script Migration

Clean up old bash scripts for modern infrastructure:

```bash
# Migrate legacy scripts to POSIX
bashrs purify legacy/*.sh --output purified/
```

## Integration with Linting

Purification works with the linter to ensure quality:

### Before Purification: Detect Issues

```bash
bashrs lint deploy.sh
```

Output:
```
deploy.sh:5:12: DET001 [Error] Non-deterministic: $RANDOM detected
deploy.sh:6:10: DET002 [Error] Non-deterministic: timestamp $(date +%s)
deploy.sh:10:1: IDEM001 [Error] Non-idempotent: mkdir without -p flag
deploy.sh:14:1: IDEM002 [Error] Non-idempotent: rm without -f flag
```

### After Purification: Verify Fixed

```bash
bashrs lint deploy_purified.sh
```

Output:
```
No issues found. ✅
```

## Command-Line Usage

### Basic Purification

```bash
# Purify a single script
bashrs purify script.sh --output script_purified.sh
```

### Batch Purification

```bash
# Purify all scripts in a directory
find . -name "*.sh" -exec bashrs purify {} --output purified/{} \;
```

### With Verification

```bash
# Purify and verify with shellcheck
bashrs purify deploy.sh --output deploy_purified.sh --verify
```

### With Report

```bash
# Generate detailed purification report
bashrs purify deploy.sh --output deploy_purified.sh --report report.txt
```

## Testing Purified Scripts

Use property-based testing to verify purification:

```bash
# Property 1: Determinism
# Same input always produces same output
bashrs purify script.sh --output v1.sh
bashrs purify script.sh --output v2.sh
diff v1.sh v2.sh  # Should be identical

# Property 2: Idempotency
# Safe to run multiple times
sh purified.sh
sh purified.sh  # Should not fail

# Property 3: POSIX Compliance
# Passes shellcheck
shellcheck -s sh purified.sh  # No errors

# Property 4: Behavioral Equivalence
# Original and purified have same behavior
bash original.sh 1.0.0 > orig.txt
sh purified.sh 1.0.0 > purif.txt
diff orig.txt purif.txt  # Should be equivalent
```

## Best Practices

### 1. Always Verify After Purification

```bash
# ALWAYS run shellcheck on purified output
bashrs purify script.sh --output purified.sh
shellcheck -s sh purified.sh
```

### 2. Test on Target Shells

```bash
# Test on the shells you'll actually use
dash purified.sh   # Debian/Ubuntu sh
ash purified.sh    # Alpine sh
busybox sh purified.sh  # Embedded systems
```

### 3. Pass Randomness as Parameters

```bash
# Don't rely on $RANDOM - pass seeds explicitly
sh purified.sh --version 1.0.0 --session-id abc123
```

### 4. Review Purified Output

Purification is not perfect - always review:
```bash
# Use diff to see what changed
diff -u original.sh purified.sh
```

### 5. Keep Both Versions

Keep original for reference:
```bash
# Version control both
git add deploy.sh deploy_purified.sh
git commit -m "Add purified version of deploy.sh"
```

## Further Reading

- [Determinism Concept](./determinism.md) - Understanding deterministic scripts
- [Idempotency Concept](./idempotency.md) - Making operations safe to re-run
- [POSIX Compliance](./posix.md) - Writing portable shell scripts
- [Security Linting](../linting/security.md) - Detecting vulnerabilities

---

**Quality Guarantee**: All purified scripts are verified with shellcheck and tested across multiple POSIX shells to ensure reliability and portability.
