# Idempotency Rules (IDEM001-IDEM006)

Rash includes idempotency rules designed to detect operations that fail when run multiple times. Idempotent scripts can be **safely re-run** without side effects or failures, making them reliable for automation and recovery.

## Overview

Idempotency linting in Rash focuses on operations that fail on second execution:
- Directory creation without `-p` (`mkdir`)
- File removal without `-f` (`rm`)
- Symlink creation without cleanup (`ln -s`)
- Non-idempotent variable appends
- File creation with `>` (truncating)
- Database inserts without existence checks

All IDEM rules are **Warning severity** by default to indicate improvements without blocking.

## Why Idempotency Matters

Non-idempotent scripts cause:
- **Deployment failures**: Re-running fails instead of succeeding
- **Recovery problems**: Can't safely retry after partial failures
- **Automation issues**: Cron jobs and systemd timers break
- **Manual headaches**: Operators fear running scripts twice
- **Rollback failures**: Can't cleanly undo then redo

**Idempotent = Safe to Re-run = Reliable**

## Core Principle

An operation is idempotent if:
```
f(x) = f(f(x)) = f(f(f(x))) = ...
```

Running it once or N times produces the same result.

## Implemented Rules (IDEM001-IDEM003)

bashrs currently implements 3 idempotency rules with comprehensive testing. The remaining rules (IDEM004-IDEM006) are planned for future releases.

## IDEM001: Non-idempotent mkdir

**Severity**: Warning

### What it Detects

`mkdir` commands without the `-p` flag.

### Why This Matters

`mkdir` without `-p` fails if the directory already exists:

```bash
$ mkdir /app/releases
$ mkdir /app/releases  # FAILS with "File exists" error
mkdir: cannot create directory '/app/releases': File exists
```

This breaks idempotency - the script fails on second run even though the desired state (directory exists) is achieved.

### Examples

❌ **NON-IDEMPOTENT (BAD)**:
```bash
#!/bin/bash
# deploy.sh - FAILS on second run

mkdir /app/releases
mkdir /app/releases/v1.0.0
ln -s /app/releases/v1.0.0 /app/current
```

**Behavior**:
```
First run:  ✅ SUCCESS - directories created
Second run: ❌ FAILURE - mkdir fails with "File exists"
```

✅ **IDEMPOTENT (GOOD)**:
```bash
#!/bin/bash
# deploy.sh - SAFE to re-run

mkdir -p /app/releases
mkdir -p /app/releases/v1.0.0
rm -f /app/current && ln -s /app/releases/v1.0.0 /app/current
```

**Behavior**:
```
First run:  ✅ SUCCESS - directories created
Second run: ✅ SUCCESS - no-op (directories exist)
Third run:  ✅ SUCCESS - still safe!
```

### Auto-fix

**Auto-fixable with assumptions** - automatically adds `-p` flag.

**Assumption**: Directory creation failure is not a critical error condition.

If directory creation failure MUST be detected (rare), keep `mkdir` without `-p` and explicitly handle errors:

```bash
#!/bin/bash
# Only use this if you NEED to detect pre-existing directories
if ! mkdir /app/releases 2>/dev/null; then
    echo "ERROR: Directory /app/releases already exists or cannot be created"
    exit 1
fi
```

### Testing for Idempotency

Verify scripts can run multiple times:

```bash
#!/bin/bash
# test-idempotency.sh

SCRIPT="$1"

echo "Testing idempotency of: $SCRIPT"

# Run once
"$SCRIPT"
RESULT1=$?

# Run twice
"$SCRIPT"
RESULT2=$?

# Both should succeed
if [ $RESULT1 -eq 0 ] && [ $RESULT2 -eq 0 ]; then
    echo "✅ PASS: Script is idempotent"
else
    echo "❌ FAIL: Script is not idempotent"
    echo "First run: exit $RESULT1"
    echo "Second run: exit $RESULT2"
    exit 1
fi
```

### Real-world Example: Application Setup

❌ **NON-IDEMPOTENT (BAD)**:
```bash
#!/bin/bash
# setup.sh - FAILS on re-run

# Create directory structure
mkdir /opt/myapp
mkdir /opt/myapp/bin
mkdir /opt/myapp/lib
mkdir /opt/myapp/data

# Install application
cp myapp /opt/myapp/bin/
cp lib/*.so /opt/myapp/lib/

# Second run FAILS at first mkdir!
```

✅ **IDEMPOTENT (GOOD)**:
```bash
#!/bin/bash
# setup.sh - SAFE to re-run

# Create directory structure (idempotent)
mkdir -p /opt/myapp/bin
mkdir -p /opt/myapp/lib
mkdir -p /opt/myapp/data

# Install application (use -f to force overwrite)
cp -f myapp /opt/myapp/bin/
cp -f lib/*.so /opt/myapp/lib/

# Safe to run multiple times - always succeeds!
```

## IDEM002: Non-idempotent rm

**Severity**: Warning

### What it Detects

`rm` commands without the `-f` flag.

### Why This Matters

`rm` without `-f` fails if the file doesn't exist:

```bash
$ rm /app/current
$ rm /app/current  # FAILS with "No such file or directory"
rm: cannot remove '/app/current': No such file or directory
```

This breaks idempotency - the script fails on second run even though the desired state (file doesn't exist) is achieved.

### Examples

❌ **NON-IDEMPOTENT (BAD)**:
```bash
#!/bin/bash
# cleanup.sh - FAILS on second run

rm /tmp/build.log
rm /tmp/cache.dat
rm /app/old-version
```

**Behavior**:
```
First run:  ✅ SUCCESS - files deleted
Second run: ❌ FAILURE - rm fails with "No such file"
```

✅ **IDEMPOTENT (GOOD)**:
```bash
#!/bin/bash
# cleanup.sh - SAFE to re-run

rm -f /tmp/build.log
rm -f /tmp/cache.dat
rm -f /app/old-version
```

**Behavior**:
```
First run:  ✅ SUCCESS - files deleted
Second run: ✅ SUCCESS - no-op (files don't exist)
Third run:  ✅ SUCCESS - still safe!
```

### Auto-fix

**Auto-fixable with assumptions** - automatically adds `-f` flag.

**Assumption**: Missing file is not an error condition.

If file existence MUST be verified (rare), explicitly check before removing:

```bash
#!/bin/bash
# Only use this if you NEED to ensure file exists
if [ ! -f /app/critical-file ]; then
    echo "ERROR: Expected file /app/critical-file not found"
    exit 1
fi

rm /app/critical-file
```

### When to Use rm Without -f

Very rare cases where missing file indicates a problem:

```bash
#!/bin/bash
# uninstall.sh - Verify installed before uninstalling

# Check installation exists
if [ ! -f /usr/local/bin/myapp ]; then
    echo "ERROR: myapp not installed (expected /usr/local/bin/myapp)"
    exit 1
fi

# Remove (without -f to detect unexpected deletion)
rm /usr/local/bin/myapp
```

But even here, idempotent version is usually better:

```bash
#!/bin/bash
# uninstall.sh - Idempotent version

# Idempotent: remove if exists, succeed if not
rm -f /usr/local/bin/myapp

# Report status
if [ -f /usr/local/bin/myapp ]; then
    echo "ERROR: Failed to remove /usr/local/bin/myapp"
    exit 1
else
    echo "✅ myapp uninstalled (or was already removed)"
fi
```

### Real-world Example: Log Rotation

❌ **NON-IDEMPOTENT (BAD)**:
```bash
#!/bin/bash
# rotate-logs.sh - FAILS on second run

# Rotate logs
mv /var/log/app.log /var/log/app.log.1
rm /var/log/app.log.2  # FAILS if doesn't exist!

# Restart app to create fresh log
systemctl restart myapp
```

✅ **IDEMPOTENT (GOOD)**:
```bash
#!/bin/bash
# rotate-logs.sh - SAFE to re-run

# Rotate logs (idempotent - -f means no error if missing)
rm -f /var/log/app.log.2
mv -f /var/log/app.log.1 /var/log/app.log.2 2>/dev/null || true
mv -f /var/log/app.log /var/log/app.log.1 2>/dev/null || true

# Restart app to create fresh log
systemctl restart myapp

# Safe to run multiple times!
```

## IDEM003: Non-idempotent ln -s

**Severity**: Warning

### What it Detects

`ln -s` (symbolic link creation) without removing existing link first.

### Why This Matters

`ln -s` fails if the target already exists:

```bash
$ ln -s /app/v1.0.0 /app/current
$ ln -s /app/v1.0.0 /app/current  # FAILS
ln: failed to create symbolic link '/app/current': File exists
```

This is especially problematic for deployment scripts that update symlinks.

### Examples

❌ **NON-IDEMPOTENT (BAD)**:
```bash
#!/bin/bash
# deploy.sh - FAILS on second run

VERSION="$1"
RELEASE_DIR="/app/releases/$VERSION"

# Create symlink (FAILS if exists)
ln -s "$RELEASE_DIR" /app/current
```

**Behavior**:
```
First deploy (v1.0.0):  ✅ SUCCESS - symlink created
Second deploy (v1.0.0): ❌ FAILURE - ln fails with "File exists"
Update deploy (v1.0.1): ❌ FAILURE - ln fails, current still points to v1.0.0!
```

✅ **IDEMPOTENT (GOOD)**:
```bash
#!/bin/bash
# deploy.sh - SAFE to re-run

VERSION="$1"
RELEASE_DIR="/app/releases/$VERSION"

# Remove old symlink first (idempotent)
rm -f /app/current

# Create new symlink
ln -s "$RELEASE_DIR" /app/current
```

**Behavior**:
```
First deploy (v1.0.0):  ✅ SUCCESS - symlink created to v1.0.0
Second deploy (v1.0.0): ✅ SUCCESS - symlink recreated (no-op)
Update deploy (v1.0.1): ✅ SUCCESS - symlink updated to v1.0.1!
```

### Auto-fix Options

**Not auto-fixable** - requires manual choice of strategy.

**Option 1: Remove then link** (recommended):
```bash
rm -f /target && ln -s /source /target
```

**Option 2: ln -sf flag** (not always portable):
```bash
# Works on Linux, may not work on some Unix systems
ln -sf /source /target
```

**Option 3: Conditional link** (explicit):
```bash
[ -e /target ] && rm /target
ln -s /source /target
```

### Testing for Idempotency

Verify symlink update works:

```bash
#!/bin/bash
# test-symlink-idempotency.sh

SCRIPT="./deploy.sh"

echo "Testing symlink idempotency"

# Deploy v1.0.0
"$SCRIPT" v1.0.0
TARGET1=$(readlink /app/current)

# Deploy v1.0.0 again (idempotent)
"$SCRIPT" v1.0.0
TARGET2=$(readlink /app/current)

# Update to v1.0.1
"$SCRIPT" v1.0.1
TARGET3=$(readlink /app/current)

# Verify results
if [ "$TARGET1" = "/app/releases/v1.0.0" ] &&
   [ "$TARGET2" = "/app/releases/v1.0.0" ] &&
   [ "$TARGET3" = "/app/releases/v1.0.1" ]; then
    echo "✅ PASS: Symlink updates are idempotent"
else
    echo "❌ FAIL: Symlink not idempotent"
    echo "Deploy 1: $TARGET1"
    echo "Deploy 2: $TARGET2"
    echo "Deploy 3: $TARGET3"
    exit 1
fi
```

### Real-world Example: Blue-Green Deployment

❌ **NON-IDEMPOTENT (BAD)**:
```bash
#!/bin/bash
# switch-version.sh - FAILS on re-run

NEW_VERSION="$1"
BLUE_DIR="/srv/app-blue"
GREEN_DIR="/srv/app-green"

# Determine which slot is active
if [ -L /srv/app-current ] && [ "$(readlink /srv/app-current)" = "$BLUE_DIR" ]; then
    INACTIVE_DIR="$GREEN_DIR"
else
    INACTIVE_DIR="$BLUE_DIR"
fi

# Deploy to inactive slot
rsync -a "dist/" "$INACTIVE_DIR/"

# Switch symlink (FAILS if already switched!)
ln -s "$INACTIVE_DIR" /srv/app-current
```

✅ **IDEMPOTENT (GOOD)**:
```bash
#!/bin/bash
# switch-version.sh - SAFE to re-run

NEW_VERSION="$1"
BLUE_DIR="/srv/app-blue"
GREEN_DIR="/srv/app-green"

# Determine which slot is active
if [ -L /srv/app-current ] && [ "$(readlink /srv/app-current)" = "$BLUE_DIR" ]; then
    INACTIVE_DIR="$GREEN_DIR"
else
    INACTIVE_DIR="$BLUE_DIR"
fi

# Deploy to inactive slot
rsync -a "dist/" "$INACTIVE_DIR/"

# Switch symlink (idempotent - remove first)
rm -f /srv/app-current
ln -s "$INACTIVE_DIR" /srv/app-current

# Safe to run multiple times!
```

## IDEM004: Non-idempotent Variable Appends (Planned)

**Status**: Not yet implemented

### What it Will Detect

Variable append operations that duplicate values on re-run:

```bash
# Non-idempotent
PATH="$PATH:/opt/myapp/bin"
# Second run: PATH has /opt/myapp/bin twice!
```

### Why This Will Matter

Repeated execution causes:
- PATH pollution with duplicates
- Growing environment variables
- Performance degradation (PATH search)

### Planned Fix

Use idempotent append pattern:

```bash
# Idempotent - only add if not present
if [[ ":$PATH:" != *":/opt/myapp/bin:"* ]]; then
    PATH="$PATH:/opt/myapp/bin"
fi
```

## IDEM005: File Creation with > (Planned)

**Status**: Not yet implemented

### What it Will Detect

File creation with `>` that truncates existing content:

```bash
# Non-idempotent
echo "data" > /var/lib/myapp/config
# Re-run appends "data" again? Truncates? Unclear!
```

### Why This Will Matter

`>` truncates files, making behavior unclear:
- Loses existing data on re-run
- Not obvious if intentional
- Hard to reason about state

### Planned Fix

Use explicit patterns:

```bash
# Idempotent - only create if doesn't exist
if [ ! -f /var/lib/myapp/config ]; then
    echo "data" > /var/lib/myapp/config
fi

# Or use >> for append (but check for duplicates)
grep -qF "data" /var/lib/myapp/config || echo "data" >> /var/lib/myapp/config
```

## IDEM006: Database Inserts Without Checks (Planned)

**Status**: Not yet implemented

### What it Will Detect

SQL inserts without existence checks:

```bash
# Non-idempotent - fails on second run if unique constraint
mysql -e "INSERT INTO users VALUES (1, 'admin')"
```

### Why This Will Matter

Database operations often fail on duplicate:
- Unique constraint violations
- Breaks migration scripts
- Manual re-runs fail

### Planned Fix

Use idempotent SQL patterns:

```bash
# Idempotent - upsert pattern
mysql -e "INSERT INTO users VALUES (1, 'admin')
          ON DUPLICATE KEY UPDATE name='admin'"

# Or check first
mysql -e "INSERT INTO users SELECT 1, 'admin'
          WHERE NOT EXISTS (SELECT 1 FROM users WHERE id=1)"
```

## Running Idempotency Linting

### Lint a Single File

```bash
bashrs lint script.sh
```

### Filter Only Idempotency Rules

```bash
bashrs lint --rules IDEM script.sh
```

### Lint All Scripts

```bash
find . -name "*.sh" -exec bashrs lint --rules IDEM {} \;
```

### CI/CD Integration

```yaml
# .github/workflows/lint.yml
name: Idempotency Lint
on: [push, pull_request]
jobs:
  idempotency:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install bashrs
        run: cargo install bashrs
      - name: Check idempotency
        run: |
          find . -name "*.sh" -exec bashrs lint --rules IDEM {} \;
```

## Testing Idempotency

### Property-Based Testing

Verify scripts are idempotent:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_script_idempotent(input in "[a-z]{1,10}") {
        // Run script twice with same input
        let state1 = run_script(&input);
        let state2 = run_script(&input);

        // Final state MUST be identical
        prop_assert_eq!(state1, state2);
    }
}
```

### Manual Testing

Run scripts multiple times and verify success:

```bash
#!/bin/bash
# test-idempotency.sh

SCRIPT="$1"
RUNS=5

echo "Testing idempotency of: $SCRIPT"

for i in $(seq 1 $RUNS); do
    echo "Run $i..."
    if ! "$SCRIPT"; then
        echo "❌ FAIL: Run $i failed"
        exit 1
    fi
done

echo "✅ PASS: All $RUNS runs succeeded"
```

### State Verification

Verify final state is consistent:

```bash
#!/bin/bash
# test-state-idempotency.sh

SCRIPT="$1"

echo "Testing state idempotency"

# Run once and capture state
"$SCRIPT"
STATE1=$(get_system_state)

# Run again and capture state
"$SCRIPT"
STATE2=$(get_system_state)

# States should be identical
if [ "$STATE1" = "$STATE2" ]; then
    echo "✅ PASS: System state is idempotent"
else
    echo "❌ FAIL: System state differs"
    diff <(echo "$STATE1") <(echo "$STATE2")
    exit 1
fi
```

## Common Patterns

### Pattern 1: Idempotent Directory Setup

Always use `-p`:

```bash
#!/bin/bash
# setup-dirs.sh

# Idempotent directory creation
mkdir -p /opt/myapp/{bin,lib,data,logs}
mkdir -p /var/log/myapp
mkdir -p /etc/myapp
```

### Pattern 2: Idempotent Cleanup

Always use `-f`:

```bash
#!/bin/bash
# cleanup.sh

# Idempotent file removal
rm -f /tmp/build-*
rm -f /var/cache/myapp/*
rm -rf /tmp/myapp-temp
```

### Pattern 3: Idempotent Symlinks

Remove before linking:

```bash
#!/bin/bash
# update-links.sh

# Idempotent symlink updates
rm -f /usr/local/bin/myapp
ln -s /opt/myapp/v2.0/bin/myapp /usr/local/bin/myapp
```

### Pattern 4: Idempotent Configuration

Check before modifying:

```bash
#!/bin/bash
# configure.sh

CONFIG_FILE="/etc/myapp/config"

# Idempotent config line addition
if ! grep -qF "setting=value" "$CONFIG_FILE"; then
    echo "setting=value" >> "$CONFIG_FILE"
fi
```

## Benefits of Idempotency

### Reliable Automation

Scripts can run repeatedly:
- Cron jobs safe to re-run
- Systemd timers don't accumulate errors
- CI/CD pipelines are resilient

### Easy Recovery

Failed operations can be retried:
- Partial failures can be re-run
- No manual cleanup needed
- Rollbacks work cleanly

### Safe Operations

Operators can run without fear:
- "Did I already run this?" - doesn't matter!
- Re-running is safe
- No destructive side effects

### Better Testing

Tests are more reliable:
- Can run tests multiple times
- No test pollution
- Easier to debug

## Further Reading

- [Idempotence (Wikipedia)](https://en.wikipedia.org/wiki/Idempotence)
- [Ansible Idempotency](https://docs.ansible.com/ansible/latest/reference_appendices/glossary.html#term-Idempotency)
- [Infrastructure as Code: Idempotency](https://www.terraform.io/docs/glossary#idempotent)

---

**Quality Guarantee**: All IDEM rules undergo comprehensive testing including multiple-run verification to ensure idempotency detection is accurate.
