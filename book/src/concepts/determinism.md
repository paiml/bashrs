# Determinism

**Determinism** means that a script produces the same output given the same input, every time it runs. There are no surprises, no randomness, and no dependence on when or where the script executes.

## Definition

A script is **deterministic** if and only if:

**Given the same input → Always produces the same output**

**Formula**: `f(input) = output` (consistently, every time)

## Why Determinism Matters

### The Problem: Non-Deterministic Scripts

Non-deterministic scripts are unpredictable and hard to debug:

```bash
#!/bin/bash
# Non-deterministic deployment

RELEASE_ID=$RANDOM                    # Random number (different each run)
TIMESTAMP=$(date +%s)                 # Unix timestamp (changes every second)
HOSTNAME=$(hostname)                  # Varies by machine
PID=$$                                # Process ID (different each run)

echo "Deploying release: $RELEASE_ID-$TIMESTAMP"
mkdir /tmp/deploy-$PID
```

**Problems**:
- `$RANDOM` generates different values: `12345`, `67890`, `24680`...
- `$(date +%s)` changes every second: `1699564800`, `1699564801`, `1699564802`...
- `$(hostname)` varies: `server1`, `server2`, `server3`...
- `$$` differs: `12345`, `12346`, `12347`...

**Impact**:
- Can't reproduce issues (bug happened once, can't recreate)
- Can't verify deployments (different ID each time)
- Can't test reliably (tests pass/fail randomly)
- Can't rollback (which version was deployed?)

### The Solution: Deterministic Scripts

Deterministic scripts are predictable and reliable:

```bash
#!/bin/sh
# Deterministic deployment

release_version="${1}"                # Input parameter (controlled)
release_id="${2}"                     # Input parameter (controlled)

echo "Deploying release: ${release_id}-${release_version}"
mkdir -p "/tmp/deploy-${release_version}"
```

**Benefits**:
- ✅ Same input → Same output: `v1.0.0` always deploys `v1.0.0`
- ✅ Reproducible: Can recreate exact same deployment
- ✅ Testable: Tests always produce same results
- ✅ Debuggable: Issues can be reproduced reliably

## Sources of Non-Determinism

Rash detects and eliminates these common sources:

### 1. $RANDOM (DET001)

**Problem**: Generates random numbers

```bash
# Non-deterministic
SESSION_ID=$RANDOM
# Output: 12345 (first run), 67890 (second run), 24680 (third run)
```

**Solution**: Pass value as parameter or use fixed seed

```bash
# Deterministic
session_id="${1:-default-session}"
# Output: "default-session" (every run)
```

### 2. Timestamps (DET002)

**Problem**: Time-based values change constantly

```bash
# Non-deterministic
BUILD_TIME=$(date +%s)                # Unix timestamp
BUILD_DATE=$(date +%Y%m%d)            # YYYYMMDD format
START_TIME=$(date)                    # Human-readable

# Different each second/day
echo "Built at: $BUILD_TIME"  # 1699564800, 1699564801, 1699564802...
```

**Solution**: Pass timestamp as parameter or use version

```bash
# Deterministic
build_version="${1}"
build_timestamp="${2}"

echo "Built at: ${build_timestamp}"  # Same value each run
```

### 3. Process IDs (DET003)

**Problem**: $$ changes for each process

```bash
# Non-deterministic
LOCK_FILE="/var/run/deploy.$$"
# Output: /var/run/deploy.12345 (first run), /var/run/deploy.12346 (second run)
```

**Solution**: Use predictable names or parameters

```bash
# Deterministic
lock_file="/var/run/deploy-${1}.lock"
# Output: /var/run/deploy-v1.0.0.lock (same each run with same input)
```

### 4. Hostnames (DET004)

**Problem**: $(hostname) varies by machine

```bash
# Non-deterministic
SERVER=$(hostname)
echo "Deploying on: $SERVER"
# Output: server1 (on server1), server2 (on server2), server3 (on server3)
```

**Solution**: Pass hostname as parameter or use configuration

```bash
# Deterministic
server="${1}"
echo "Deploying on: ${server}"
# Output: Predictable based on input parameter
```

### 5. UUIDs/GUIDs (DET005)

**Problem**: Universally unique identifiers are... unique

```bash
# Non-deterministic
DEPLOY_ID=$(uuidgen)
# Output: 550e8400-e29b-41d4-a716-446655440000 (different every time)
```

**Solution**: Derive IDs from input or use version

```bash
# Deterministic
deploy_id="deploy-${1}-${2}"  # Constructed from version + timestamp
# Output: deploy-v1.0.0-20231109 (predictable)
```

### 6. Network Queries (DET006)

**Problem**: DNS, API calls return different results

```bash
# Non-deterministic
CURRENT_IP=$(curl -s https://api.ipify.org)
# Output: 192.0.2.1 (changes based on network, time, location)
```

**Solution**: Pass values as parameters

```bash
# Deterministic
current_ip="${1}"
# Output: Controlled by input
```

## Testing Determinism

### Property Test: Same Input → Same Output

```bash
#!/bin/sh
# Test: Run script twice with same input, verify identical output

# Run 1
sh deploy.sh v1.0.0 session-123 > output1.txt

# Run 2
sh deploy.sh v1.0.0 session-123 > output2.txt

# Verify identical
diff output1.txt output2.txt
# Expected: No differences (deterministic ✅)
```

### Property Test: Different Input → Different Output

```bash
#!/bin/sh
# Test: Run script with different input, verify different output

# Run with version 1.0.0
sh deploy.sh v1.0.0 > output_v1.txt

# Run with version 2.0.0
sh deploy.sh v2.0.0 > output_v2.txt

# Verify different
if diff output_v1.txt output_v2.txt > /dev/null; then
    echo "FAIL: Different inputs produced same output (not deterministic)"
else
    echo "PASS: Different inputs produced different outputs (deterministic ✅)"
fi
```

### Repeatability Test

```bash
#!/bin/sh
# Test: Run script 100 times, verify all outputs identical

sh deploy.sh v1.0.0 > baseline.txt

for i in $(seq 1 100); do
    sh deploy.sh v1.0.0 > run_$i.txt
    if ! diff baseline.txt run_$i.txt > /dev/null; then
        echo "FAIL: Run $i produced different output"
        exit 1
    fi
done

echo "PASS: All 100 runs produced identical output (deterministic ✅)"
```

## Linter Detection

Rash linter detects non-determinism:

```bash
bashrs lint deploy.sh
```

Output:
```text
deploy.sh:3:12: DET001 [Error] Non-deterministic: $RANDOM detected
deploy.sh:4:14: DET002 [Error] Non-deterministic: timestamp $(date +%s)
deploy.sh:5:12: DET003 [Error] Non-deterministic: process ID $$ detected
deploy.sh:6:10: DET004 [Error] Non-deterministic: hostname command detected
```

## Purification Transforms

Rash purification automatically fixes determinism issues:

### Before: Non-Deterministic

```bash
#!/bin/bash
SESSION_ID=$RANDOM
RELEASE="release-$(date +%s)"
LOCK="/var/run/deploy.$$"

echo "Deploying $RELEASE (session $SESSION_ID, lock $LOCK)"
```

### After: Deterministic

```bash
#!/bin/sh
# Purified by Rash v6.30.1

deploy() {
    _version="${1}"
    _session="${2:-default-session}"

    release="release-${_version}"
    lock="/var/run/deploy-${_version}.lock"

    echo "Deploying ${release} (session ${_session}, lock ${lock})"
}

deploy "${1}" "${2}"
```

**Transformations**:
- ✅ $RANDOM → parameter `_session`
- ✅ $(date +%s) → parameter `_version`
- ✅ $$ → version-based `_version`

## Best Practices

### 1. Always Use Input Parameters

```bash
# ❌ BAD: Non-deterministic
SESSION_ID=$RANDOM

# ✅ GOOD: Deterministic
session_id="${1}"
```

### 2. Avoid Time-Based Values

```bash
# ❌ BAD: Changes every second
BACKUP_NAME="backup-$(date +%s).tar.gz"

# ✅ GOOD: Based on version
backup_name="backup-${1}.tar.gz"
```

### 3. Derive Values from Inputs

```bash
# ❌ BAD: Random UUID
DEPLOY_ID=$(uuidgen)

# ✅ GOOD: Constructed from inputs
deploy_id="deploy-${VERSION}-${ENVIRONMENT}"
```

### 4. Use Configuration Files

```bash
# ❌ BAD: Query at runtime
CURRENT_IP=$(curl -s https://api.ipify.org)

# ✅ GOOD: Read from config
current_ip=$(cat /etc/myapp/ip.conf)
```

### 5. Seed Randomness Explicitly

If you MUST use randomness:

```bash
# ❌ BAD: Unseeded random
echo $RANDOM

# ✅ GOOD: Seeded with parameter
seed="${1}"
RANDOM=$seed  # Set seed explicitly
echo $RANDOM  # Now deterministic for given seed
```

## Common Patterns

### Pattern 1: Version-Based Naming

```bash
# Non-deterministic
RELEASE_DIR="/app/releases/$(date +%Y%m%d-%H%M%S)"

# Deterministic
release_dir="/app/releases/${VERSION}"
```

### Pattern 2: Environment Configuration

```bash
# Non-deterministic
SERVER=$(hostname)

# Deterministic (read from config)
server=$(cat /etc/environment)
```

### Pattern 3: Input-Based IDs

```bash
# Non-deterministic
BUILD_ID=$(uuidgen)

# Deterministic
build_id="build-${GIT_COMMIT}-${BUILD_NUMBER}"
```

## Integration with Idempotency

Determinism and idempotency work together:

```bash
#!/bin/sh
# Both deterministic AND idempotent

deploy() {
    version="${1}"  # Deterministic: same input always

    # Idempotent: safe to re-run
    mkdir -p "/app/releases/${version}"
    rm -f "/app/current"
    ln -sf "/app/releases/${version}" "/app/current"

    echo "Deployed ${version}"  # Deterministic output
}

deploy "${1}"
```

**Properties**:
- ✅ Deterministic: Same version always produces same output
- ✅ Idempotent: Running twice with same version is safe

## Further Reading

- [Purification Overview](./purification.md) - Complete purification process
- [Idempotency Concept](./idempotency.md) - Safe re-run operations
- [POSIX Compliance](./posix.md) - Portable shell scripts
- [DET Rules](../linting/determinism.md) - Linter rules for determinism

---

**Key Takeaway**: Determinism makes scripts predictable and reproducible. Always use input parameters instead of random values, timestamps, or runtime queries.
