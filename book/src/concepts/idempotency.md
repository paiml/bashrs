# Idempotency

**Idempotency** means that running an operation multiple times has the same effect as running it once. There are no errors, no side effects, and the system reaches the same final state regardless of how many times the script executes.

## Definition

An operation is **idempotent** if and only if:

**Running it multiple times = Running it once (same final state)**

**Formula**: `f(f(state)) = f(state)` (consistent, every time)

## Why Idempotency Matters

### The Problem: Non-Idempotent Scripts

Non-idempotent scripts fail when re-run, making deployments and automation fragile:

```bash
#!/bin/bash
# Non-idempotent deployment

# Fails if directory already exists
mkdir /app/releases/v1.0.0

# Fails if file already deleted
rm /app/old-config.txt

# Creates duplicate symlink or fails
ln -s /app/releases/v1.0.0 /app/current

# Appends duplicate entries
echo "export PATH=/app/bin:$PATH" >> ~/.bashrc
```

**Problems**:
- `mkdir /app/releases/v1.0.0` → **ERROR**: "File exists" (fails on 2nd run)
- `rm /app/old-config.txt` → **ERROR**: "No such file" (fails if already deleted)
- `ln -s ...` → **ERROR**: "File exists" (fails if link exists)
- `echo ... >> ~/.bashrc` → Appends duplicate entries every run

**Impact**:
- ❌ Can't re-run deployments (fail on retry)
- ❌ Can't recover from failures (script breaks halfway)
- ❌ Can't repeat operations (inconsistent state)
- ❌ Manual cleanup required (error-prone)

### The Solution: Idempotent Scripts

Idempotent scripts are safe to re-run without errors:

```bash
#!/bin/sh
# Idempotent deployment

# Safe: creates directory if missing, succeeds if exists
mkdir -p /app/releases/v1.0.0

# Safe: removes file if exists, succeeds if missing
rm -f /app/old-config.txt

# Safe: remove old link, create new one
rm -f /app/current
ln -s /app/releases/v1.0.0 /app/current

# Safe: only add if not already present
grep -q "export PATH=/app/bin" ~/.bashrc || \
    echo "export PATH=/app/bin:\$PATH" >> ~/.bashrc
```

**Benefits**:
- ✅ Safe to re-run: No errors on 2nd, 3rd, Nth execution
- ✅ Recoverable: Can retry after failures
- ✅ Predictable: Always reaches same final state
- ✅ Automated: No manual intervention needed

## Sources of Non-Idempotency

Rash detects and eliminates these common patterns:

### 1. mkdir without -p (IDEM001)

**Problem**: Fails if directory exists

```bash
# Non-idempotent
mkdir /app/releases/v1.0.0
# First run: ✅ Success
# Second run: ❌ mkdir: cannot create directory '/app/releases/v1.0.0': File exists
```

**Solution**: Always use `-p` flag

```bash
# Idempotent
mkdir -p /app/releases/v1.0.0
# First run: ✅ Creates directory
# Second run: ✅ Directory exists (no error)
# Nth run: ✅ Always succeeds
```

### 2. rm without -f (IDEM002)

**Problem**: Fails if file doesn't exist

```bash
# Non-idempotent
rm /app/old-config.txt
# First run: ✅ File deleted
# Second run: ❌ rm: cannot remove '/app/old-config.txt': No such file or directory
```

**Solution**: Always use `-f` flag

```bash
# Idempotent
rm -f /app/old-config.txt
# First run: ✅ File deleted
# Second run: ✅ File already gone (no error)
# Nth run: ✅ Always succeeds
```

### 3. ln -s without cleanup (IDEM003)

**Problem**: Fails if symlink exists

```bash
# Non-idempotent
ln -s /app/releases/v1.0.0 /app/current
# First run: ✅ Symlink created
# Second run: ❌ ln: failed to create symbolic link '/app/current': File exists
```

**Solution**: Remove old link first

```bash
# Idempotent
rm -f /app/current
ln -s /app/releases/v1.0.0 /app/current
# First run: ✅ Creates symlink
# Second run: ✅ Replaces symlink
# Nth run: ✅ Always succeeds
```

### 4. Appending to files (IDEM004)

**Problem**: Creates duplicate entries

```bash
# Non-idempotent
echo "export PATH=/app/bin:\$PATH" >> ~/.bashrc
# First run: Adds line (correct)
# Second run: Adds duplicate line
# Nth run: N duplicate lines (wrong!)
```

**Solution**: Check before appending

```bash
# Idempotent
grep -q "export PATH=/app/bin" ~/.bashrc || \
    echo "export PATH=/app/bin:\$PATH" >> ~/.bashrc
# First run: ✅ Adds line
# Second run: ✅ Line exists (skips)
# Nth run: ✅ Always one line
```

### 5. Creating files with > (IDEM005)

**Problem**: Not idempotent if file must not exist

```bash
# Non-idempotent (if uniqueness required)
echo "config=value" > /etc/myapp/config.conf
# Creates new file each time (might overwrite important data)
```

**Solution**: Use conditional creation or explicit overwrite

```bash
# Idempotent (explicit overwrite intended)
mkdir -p /etc/myapp
echo "config=value" > /etc/myapp/config.conf
# Always writes same config (idempotent for config management)

# Or conditional creation
if [ ! -f /etc/myapp/config.conf ]; then
    echo "config=value" > /etc/myapp/config.conf
fi
```

### 6. Database inserts (IDEM006)

**Problem**: Duplicate records

```bash
# Non-idempotent
psql -c "INSERT INTO users (name) VALUES ('admin')"
# First run: Creates user
# Second run: Creates duplicate user (or fails with constraint violation)
```

**Solution**: Use INSERT ... ON CONFLICT or upserts

```bash
# Idempotent
psql -c "INSERT INTO users (name) VALUES ('admin') ON CONFLICT (name) DO NOTHING"
# First run: Creates user
# Second run: User exists (no duplicate)
# Nth run: Always one user
```

## Testing Idempotency

### Property Test: Multiple Runs → Same State

```bash
#!/bin/sh
# Test: Run script 3 times, verify same final state

# Clean state
rm -rf /tmp/test_deploy

# Run 1
sh deploy.sh v1.0.0 2>&1 | tee run1.log
state1=$(ls -la /tmp/test_deploy)

# Run 2
sh deploy.sh v1.0.0 2>&1 | tee run2.log
state2=$(ls -la /tmp/test_deploy)

# Run 3
sh deploy.sh v1.0.0 2>&1 | tee run3.log
state3=$(ls -la /tmp/test_deploy)

# Verify identical state
if [ "$state1" = "$state2" ] && [ "$state2" = "$state3" ]; then
    echo "PASS: All runs produced same state (idempotent ✅)"
else
    echo "FAIL: State differs between runs (not idempotent)"
    exit 1
fi
```

### Property Test: No Errors on Re-Run

```bash
#!/bin/sh
# Test: Run script twice, verify both succeed

# Run 1
sh deploy.sh v1.0.0
exit_code1=$?

# Run 2 (should not fail)
sh deploy.sh v1.0.0
exit_code2=$?

if [ $exit_code1 -eq 0 ] && [ $exit_code2 -eq 0 ]; then
    echo "PASS: Both runs succeeded (idempotent ✅)"
else
    echo "FAIL: Run 1: $exit_code1, Run 2: $exit_code2 (not idempotent)"
    exit 1
fi
```

### Repeatability Test

```bash
#!/bin/sh
# Test: Run script 100 times, verify all succeed

for i in $(seq 1 100); do
    sh deploy.sh v1.0.0 > /dev/null 2>&1
    if [ $? -ne 0 ]; then
        echo "FAIL: Run $i failed (not idempotent)"
        exit 1
    fi
done

echo "PASS: All 100 runs succeeded (idempotent ✅)"
```

## Linter Detection

Rash linter detects non-idempotent patterns:

```bash
bashrs lint deploy.sh
```

Output:
```text
deploy.sh:3:1: IDEM001 [Error] Non-idempotent: mkdir without -p flag
deploy.sh:4:1: IDEM002 [Error] Non-idempotent: rm without -f flag
deploy.sh:5:1: IDEM003 [Error] Non-idempotent: ln -s without cleanup
deploy.sh:6:1: IDEM004 [Error] Non-idempotent: append without duplicate check
```

## Purification Transforms

Rash purification automatically fixes idempotency issues:

### Before: Non-Idempotent

```bash
#!/bin/bash
# Non-idempotent deployment

mkdir /app/releases/v1.0.0
mkdir /app/logs
rm /app/old-config.txt
ln -s /app/releases/v1.0.0 /app/current
echo "export PATH=/app/bin:\$PATH" >> ~/.bashrc
```

### After: Idempotent

```bash
#!/bin/sh
# Purified by Rash v6.30.1

deploy() {
    _version="${1}"

    # Idempotent: mkdir -p (safe to re-run)
    mkdir -p "/app/releases/${_version}"
    mkdir -p "/app/logs"

    # Idempotent: rm -f (safe if file missing)
    rm -f "/app/old-config.txt"

    # Idempotent: remove old link, create new
    rm -f "/app/current"
    ln -s "/app/releases/${_version}" "/app/current"

    # Idempotent: conditional append
    grep -q "export PATH=/app/bin" ~/.bashrc || \
        echo "export PATH=/app/bin:\$PATH" >> ~/.bashrc
}

deploy "${1}"
```

**Transformations**:
- ✅ `mkdir` → `mkdir -p` (idempotent)
- ✅ `rm` → `rm -f` (idempotent)
- ✅ `ln -s` → `rm -f && ln -s` (idempotent)
- ✅ `echo >>` → `grep -q || echo >>` (idempotent)

## Best Practices

### 1. Always Use -p for mkdir

```bash
# ❌ BAD: Fails if exists
mkdir /app/config

# ✅ GOOD: Always succeeds
mkdir -p /app/config
```

### 2. Always Use -f for rm

```bash
# ❌ BAD: Fails if missing
rm /tmp/old-file.txt

# ✅ GOOD: Always succeeds
rm -f /tmp/old-file.txt
```

### 3. Clean Before Creating Symlinks

```bash
# ❌ BAD: Fails if exists
ln -s /app/new /app/link

# ✅ GOOD: Remove old, create new
rm -f /app/link
ln -s /app/new /app/link
```

### 4. Check Before Appending

```bash
# ❌ BAD: Creates duplicates
echo "line" >> file.txt

# ✅ GOOD: Add only if missing
grep -q "line" file.txt || echo "line" >> file.txt
```

### 5. Use Conditional File Creation

```bash
# ❌ BAD: Blindly overwrites
echo "data" > /etc/config.txt

# ✅ GOOD: Create only if missing
if [ ! -f /etc/config.txt ]; then
    echo "data" > /etc/config.txt
fi

# Or explicit overwrite (if idempotent config management)
echo "data" > /etc/config.txt  # Idempotent for config files
```

## Common Patterns

### Pattern 1: Idempotent Directory Setup

```bash
# Non-idempotent
mkdir /app
mkdir /app/bin
mkdir /app/config

# Idempotent
mkdir -p /app/bin /app/config
```

### Pattern 2: Idempotent Cleanup

```bash
# Non-idempotent
rm /tmp/*.log

# Idempotent
rm -f /tmp/*.log
```

### Pattern 3: Idempotent Configuration

```bash
# Non-idempotent
echo "setting=value" >> /etc/config.conf

# Idempotent
config_file="/etc/config.conf"
grep -q "setting=value" "$config_file" || \
    echo "setting=value" >> "$config_file"
```

### Pattern 4: Idempotent Service Management

```bash
# Non-idempotent
systemctl start myservice

# Idempotent
systemctl is-active myservice || systemctl start myservice

# Or simpler (systemctl start is already idempotent)
systemctl start myservice  # Safe to re-run
```

## Integration with Determinism

Idempotency and determinism work together:

```bash
#!/bin/sh
# Both deterministic AND idempotent

deploy() {
    version="${1}"  # Deterministic: same input always

    # Idempotent: safe to re-run
    mkdir -p "/app/releases/${version}"
    rm -f "/app/current"
    ln -s "/app/releases/${version}" "/app/current"

    echo "Deployed ${version}"  # Deterministic output
}

deploy "${1}"
```

**Properties**:
- ✅ Deterministic: Same version always produces same output
- ✅ Idempotent: Running twice with same version is safe

**Testing Both Properties**:

```bash
#!/bin/sh
# Test determinism + idempotency

# Test 1: Determinism (same input → same output)
sh deploy.sh v1.0.0 > output1.txt
sh deploy.sh v1.0.0 > output2.txt
diff output1.txt output2.txt
# Expected: Identical (deterministic ✅)

# Test 2: Idempotency (multiple runs → same state)
sh deploy.sh v1.0.0
state1=$(ls -la /app)
sh deploy.sh v1.0.0
state2=$(ls -la /app)
[ "$state1" = "$state2" ]
# Expected: Same state (idempotent ✅)
```

## Advanced Patterns

### Atomic Operations

Some operations are naturally atomic and idempotent:

```bash
# Idempotent: Overwriting files
cp /source/config.txt /dest/config.txt
# Run 1: Copies file
# Run N: Overwrites with same content (idempotent)

# Idempotent: Setting environment
export PATH="/app/bin:$PATH"
# Run N: Same PATH value (idempotent)

# Idempotent: Kill processes
killall -q myprocess || true
# Run N: Process killed or already dead (idempotent)
```

### Database Migrations

```bash
# Idempotent: Schema migrations
psql -c "CREATE TABLE IF NOT EXISTS users (id SERIAL, name TEXT)"
# Run N: Table exists or created (idempotent)

# Idempotent: Upserts
psql -c "INSERT INTO settings (key, value) VALUES ('timeout', '30') \
         ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value"
# Run N: Setting updated (idempotent)
```

### Container Initialization

```bash
#!/bin/sh
# Idempotent container init script

# Idempotent: Directory structure
mkdir -p /data/logs /data/config /data/cache

# Idempotent: Default config (only if missing)
if [ ! -f /data/config/app.conf ]; then
    cp /defaults/app.conf /data/config/app.conf
fi

# Idempotent: Permissions
chown -R app:app /data

# Idempotent: Service start (systemd handles idempotency)
exec /app/bin/myservice
```

## Verification Checklist

Before marking a script as idempotent, verify:

- [ ] ✅ **mkdir has -p flag**: All directory creation is idempotent
- [ ] ✅ **rm has -f flag**: All file removal is idempotent
- [ ] ✅ **Symlinks cleaned**: Old links removed before creating new ones
- [ ] ✅ **No duplicate appends**: Appends check for existing content
- [ ] ✅ **Multiple runs succeed**: Script runs 100+ times without errors
- [ ] ✅ **Same final state**: All runs produce identical final state
- [ ] ✅ **No side effects**: No accumulating files, processes, or data

## Error Handling for Idempotency

```bash
#!/bin/sh
# Idempotent error handling

deploy() {
    version="${1}"

    # Idempotent: Create or verify directory exists
    mkdir -p "/app/releases/${version}" || {
        echo "ERROR: Cannot create release directory"
        return 1
    }

    # Idempotent: Remove old link (ignore errors if not exists)
    rm -f "/app/current"

    # Idempotent: Create new link
    ln -s "/app/releases/${version}" "/app/current" || {
        echo "ERROR: Cannot create symlink"
        return 1
    }

    echo "Deployed ${version} successfully"
}
```

## Further Reading

- [Purification Overview](./purification.md) - Complete purification process
- [Determinism Concept](./determinism.md) - Predictable script behavior
- [POSIX Compliance](./posix.md) - Portable shell scripts
- [IDEM Rules](../linting/README.md) - Linter rules for idempotency

---

**Key Takeaway**: Idempotency makes scripts safe to re-run. Always use `-p` for mkdir, `-f` for rm, cleanup before creating symlinks, and check before appending to files.
