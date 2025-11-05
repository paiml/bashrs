# Determinism Rules (DET001-DET006)

Rash includes determinism rules designed to detect non-deterministic patterns in shell scripts. Deterministic scripts produce **identical output** given identical inputs, making them testable, reproducible, and debuggable.

## Overview

Determinism linting in Rash focuses on patterns that break reproducibility:
- Random number generation (`$RANDOM`)
- Timestamp dependencies (`date`, `$(date)`)
- Unordered file glob operations (wildcards without sorting)
- Process ID usage (`$$`, `$PPID`)
- Hostname dependencies (`hostname`)
- Network queries for dynamic data

All DET rules are **Error or Warning severity** and should be addressed for production scripts.

## Why Determinism Matters

Non-deterministic scripts cause:
- **Unreproducible builds**: Different outputs on each run
- **Flaky tests**: Tests pass sometimes, fail other times
- **Debugging nightmares**: Issues can't be reproduced
- **Security risks**: Unpredictable behavior in production
- **Compliance failures**: Builds can't be audited or verified

**Deterministic = Testable = Reliable**

## Implemented Rules (DET001-DET003)

bashrs currently implements 3 determinism rules with comprehensive testing. The remaining rules (DET004-DET006) are planned for future releases.

## DET001: Non-deterministic $RANDOM Usage

**Severity**: Error (Critical)

### What it Detects

Use of `$RANDOM` which produces different values on each script execution.

### Why This Matters

Scripts using `$RANDOM` will produce different output on each run, breaking determinism and making testing/debugging impossible. Reproducible builds require deterministic inputs.

### Examples

❌ **CRITICAL ISSUE**:
```bash
#!/bin/bash
# Non-deterministic - different SESSION_ID every run
SESSION_ID=$RANDOM
echo "Session: $SESSION_ID"

# Deploy script that changes every time
RELEASE="release-$RANDOM"
mkdir "/releases/$RELEASE"
```

**Output varies**:
```text
Run 1: Session: 12847
Run 2: Session: 29103  # Different!
Run 3: Session: 5721   # Still different!
```

✅ **GOOD - DETERMINISTIC**:
```bash
#!/bin/bash
# Deterministic - same VERSION produces same SESSION_ID
VERSION="${1:-1.0.0}"
SESSION_ID="session-${VERSION}"
echo "Session: $SESSION_ID"

# Or use hash for pseudo-randomness from input
SESSION_ID=$(echo "${VERSION}" | sha256sum | cut -c1-8)

# Or use timestamp as explicit input
TIMESTAMP="$1"
RELEASE="release-${TIMESTAMP}"
mkdir -p "/releases/$RELEASE"
```

**Output is predictable**:
```text
Run 1 with VERSION=1.0.0: Session: session-1.0.0
Run 2 with VERSION=1.0.0: Session: session-1.0.0  # Same!
Run 3 with VERSION=1.0.0: Session: session-1.0.0  # Consistent!
```

### Auto-fix

**Not auto-fixable** - requires manual decision about deterministic alternative.

**Fix suggestions**:
1. **Version-based ID**: `SESSION_ID="session-${VERSION}"`
2. **Argument-based**: `SESSION_ID="$1"` (pass as parameter)
3. **Hash-based**: `SESSION_ID=$(echo "$INPUT" | sha256sum | cut -c1-8)`
4. **Build ID**: Use CI/CD build number: `SESSION_ID="${CI_BUILD_ID}"`

### Testing for $RANDOM

Property-based test to verify determinism:

```rust
#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_deterministic_session_id(version in "[0-9]+\\.[0-9]+\\.[0-9]+") {
            // Deterministic: same input → same output
            let session_id_1 = generate_session_id(&version);
            let session_id_2 = generate_session_id(&version);

            // Must be identical
            assert_eq!(session_id_1, session_id_2);
        }
    }
}
```

### Real-world Example: Deployment Script

❌ **NON-DETERMINISTIC (BAD)**:
```bash
#!/bin/bash
# deploy.sh - PROBLEMATIC

# Different deploy ID every time - can't reproduce!
DEPLOY_ID=$RANDOM
LOG_FILE="/var/log/deploy-${DEPLOY_ID}.log"

echo "Deploying with ID: $DEPLOY_ID" | tee "$LOG_FILE"
./install.sh

# Can't find the log file later - which DEPLOY_ID was it?
```

✅ **DETERMINISTIC (GOOD)**:
```bash
#!/bin/bash
# deploy.sh - REPRODUCIBLE

# Deterministic deploy ID from version
VERSION="${1:?Error: VERSION required}"
DEPLOY_ID="deploy-${VERSION}"
LOG_FILE="/var/log/deploy-${DEPLOY_ID}.log"

echo "Deploying version: $VERSION" | tee "$LOG_FILE"
./install.sh

# Log file is predictable: /var/log/deploy-1.0.0.log
# Can re-run with same VERSION and get same behavior
```

## DET002: Non-deterministic Timestamp Usage

**Severity**: Error (Critical)

### What it Detects

Use of `date` commands that produce timestamps:
- `$(date +%s)` - Unix epoch
- `$(date +%Y%m%d)` - Date formatting
- `` `date` `` - Backtick date command
- `date +%H%M%S` - Time formatting

### Why This Matters

Scripts using timestamps produce different output on each run, breaking:
- **Reproducible builds**: Can't recreate exact build artifact
- **Testing**: Tests depend on execution time
- **Debugging**: Can't reproduce issues
- **Auditing**: Can't verify build provenance

### Examples

❌ **CRITICAL ISSUE**:
```bash
#!/bin/bash
# Non-deterministic - different every second!
RELEASE="release-$(date +%s)"
echo "Creating release: $RELEASE"

# Build artifact name changes constantly
BUILD_ID=$(date +%Y%m%d%H%M%S)
ARTIFACT="myapp-${BUILD_ID}.tar.gz"
tar czf "$ARTIFACT" ./dist/

# Can't reproduce this exact build later!
```

**Output varies by time**:
```text
Run at 2025-01-15 14:30:00: release-1736951400
Run at 2025-01-15 14:30:01: release-1736951401  # Different!
Run at 2025-01-15 14:30:02: release-1736951402  # Still changing!
```

✅ **GOOD - DETERMINISTIC**:
```bash
#!/bin/bash
# Deterministic - same VERSION produces same RELEASE
VERSION="${1:?Error: VERSION required}"
RELEASE="release-${VERSION}"
echo "Creating release: $RELEASE"

# Build artifact is reproducible
ARTIFACT="myapp-${VERSION}.tar.gz"
tar czf "$ARTIFACT" ./dist/

# Same VERSION always produces same ARTIFACT
# Can reproduce exact build at any time
```

**Output is predictable**:
```text
With VERSION=1.0.0: release-1.0.0, myapp-1.0.0.tar.gz
With VERSION=1.0.0: release-1.0.0, myapp-1.0.0.tar.gz  # Same!
```

### Auto-fix

**Not auto-fixable** - requires manual decision about deterministic alternative.

**Fix suggestions**:
1. **Version-based**: `RELEASE="release-${VERSION}"`
2. **Git commit**: `RELEASE="release-$(git rev-parse --short HEAD)"`
3. **Argument-based**: `RELEASE="release-$1"` (pass as parameter)
4. **SOURCE_DATE_EPOCH**: For reproducible builds (see below)

### Reproducible Builds: SOURCE_DATE_EPOCH

For builds that MUST include a timestamp (e.g., packaging), use `SOURCE_DATE_EPOCH`:

✅ **REPRODUCIBLE BUILD TIMESTAMP**:
```bash
#!/bin/bash
# build.sh - Reproducible timestamp

# SOURCE_DATE_EPOCH is a standard for reproducible builds
# Set to git commit timestamp for determinism
if [ -z "$SOURCE_DATE_EPOCH" ]; then
    SOURCE_DATE_EPOCH=$(git log -1 --format=%ct)
fi

# This timestamp is now deterministic (same commit → same timestamp)
BUILD_DATE=$(date -u -d "@$SOURCE_DATE_EPOCH" +%Y-%m-%d)
VERSION="${VERSION:-1.0.0}"
RELEASE="release-${VERSION}-${BUILD_DATE}"

echo "Reproducible release: $RELEASE"
# Same commit always produces same RELEASE
```

**Reproducibility achieved**:
```text
Build from commit abc123: release-1.0.0-2025-01-10
Build from commit abc123: release-1.0.0-2025-01-10  # Identical!
Build from commit abc123: release-1.0.0-2025-01-10  # Still identical!
```

### Testing for Timestamps

Verify determinism with property tests:

```rust
#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_deterministic_release(version in "[0-9]+\\.[0-9]+\\.[0-9]+") {
            // Set environment for reproducibility
            std::env::set_var("SOURCE_DATE_EPOCH", "1736899200");

            // Deterministic: same input → same output
            let release_1 = generate_release(&version);
            let release_2 = generate_release(&version);

            // Must be identical
            assert_eq!(release_1, release_2);
        }
    }
}
```

### Real-world Example: CI/CD Pipeline

❌ **NON-DETERMINISTIC (BAD)**:
```bash
#!/bin/bash
# ci-build.sh - PROBLEMATIC

# Different artifact name every build run
TIMESTAMP=$(date +%s)
ARTIFACT="app-${TIMESTAMP}.tar.gz"

./build.sh
tar czf "$ARTIFACT" ./dist/

# Can't reproduce exact artifact - timestamp always changes!
# Security audits fail - can't verify provenance
```

✅ **DETERMINISTIC (GOOD)**:
```bash
#!/bin/bash
# ci-build.sh - REPRODUCIBLE

# Use git commit for deterministic artifact name
GIT_COMMIT=$(git rev-parse --short HEAD)
VERSION="${CI_BUILD_TAG:-dev}"
ARTIFACT="app-${VERSION}-${GIT_COMMIT}.tar.gz"

# Reproducible build with SOURCE_DATE_EPOCH
export SOURCE_DATE_EPOCH=$(git log -1 --format=%ct)

./build.sh
tar czf "$ARTIFACT" ./dist/

# Same commit always produces same artifact
# Security audits pass - provenance is verifiable!
```

## DET003: Unordered Wildcard Usage

**Severity**: Warning

### What it Detects

File glob wildcards without sorting:
- `$(ls *.txt)` - Unsorted file list
- `for f in *.c; do ... done` - Order varies by filesystem

### Why This Matters

File glob results vary by:
- **Filesystem implementation**: ext4, btrfs, xfs have different ordering
- **Directory entry order**: Can change between runs
- **Locale settings**: Different sorting on different systems

This breaks determinism and causes flaky tests.

### Examples

❌ **NON-DETERMINISTIC**:
```bash
#!/bin/bash
# Order varies by filesystem!
FILES=$(ls *.txt)
echo "Processing files: $FILES"

# Loop order is unpredictable
for f in *.c; do
    echo "Compiling: $f"
    gcc -c "$f"
done

# Output varies:
# Run 1: file1.c, file2.c, file3.c
# Run 2: file2.c, file1.c, file3.c  # Different order!
```

✅ **DETERMINISTIC**:
```bash
#!/bin/bash
# Explicit sorting for consistent order
FILES=$(ls *.txt | sort)
echo "Processing files: $FILES"

# Loop with sorted glob
for f in $(ls *.c | sort); do
    echo "Compiling: $f"
    gcc -c "$f"
done

# Output is consistent:
# Run 1: file1.c, file2.c, file3.c
# Run 2: file1.c, file2.c, file3.c  # Same order!
```

### Auto-fix

**Auto-fixable** - adds `| sort` to wildcard expressions.

### Better Alternative: Explicit Arrays

For bash scripts, use sorted arrays:

✅ **BASH ARRAY WITH SORTING**:
```bash
#!/bin/bash
# More robust: explicit array with sorting
mapfile -t FILES < <(ls *.txt | sort)

echo "Processing ${#FILES[@]} files"

for file in "${FILES[@]}"; do
    echo "Processing: $file"
    process_file "$file"
done
```

### Testing for Determinism

Verify ordering consistency:

```bash
#!/bin/bash
# test-determinism.sh

# Run multiple times and compare output
OUTPUT1=$(./process-files.sh)
OUTPUT2=$(./process-files.sh)
OUTPUT3=$(./process-files.sh)

# All outputs should be identical
if [ "$OUTPUT1" = "$OUTPUT2" ] && [ "$OUTPUT2" = "$OUTPUT3" ]; then
    echo "✅ DETERMINISTIC: All runs produced identical output"
else
    echo "❌ NON-DETERMINISTIC: Outputs differ between runs"
    diff <(echo "$OUTPUT1") <(echo "$OUTPUT2")
    exit 1
fi
```

### Real-world Example: Build System

❌ **NON-DETERMINISTIC (BAD)**:
```bash
#!/bin/bash
# build-all.sh - PROBLEMATIC

# Order varies by filesystem
for src in src/*.c; do
    gcc -c "$src"
done

# Link order affects final binary (on some linkers)
gcc -o myapp *.o

# Binary may differ between builds due to link order!
# Reproducible builds FAIL
```

✅ **DETERMINISTIC (GOOD)**:
```bash
#!/bin/bash
# build-all.sh - REPRODUCIBLE

# Explicit sorting for consistent order
mapfile -t SOURCES < <(ls src/*.c | sort)

for src in "${SOURCES[@]}"; do
    gcc -c "$src"
done

# Deterministic link order
mapfile -t OBJECTS < <(ls *.o | sort)
gcc -o myapp "${OBJECTS[@]}"

# Binary is identical between builds
# Reproducible builds PASS ✅
```

## DET004: Process ID Usage (Planned)

**Status**: Not yet implemented

### What it Will Detect

Use of process IDs that change on each execution:
- `$$` - Current process ID
- `$PPID` - Parent process ID
- `$BASHPID` - Bash-specific process ID

### Why This Will Matter

Process IDs are assigned sequentially by the kernel and vary unpredictably:
```bash
# Non-deterministic
LOCKFILE="/tmp/myapp-$$.lock"
# Creates /tmp/myapp-12847.lock, then /tmp/myapp-29103.lock, etc.
```

### Planned Fix

Replace with deterministic alternatives:
```bash
# Deterministic
LOCKFILE="/tmp/myapp-${USER}-${VERSION}.lock"
```

## DET005: Hostname Dependencies (Planned)

**Status**: Not yet implemented

### What it Will Detect

Scripts that depend on `hostname` command:
```bash
# Non-deterministic across hosts
SERVER_ID=$(hostname)
LOG_FILE="/var/log/app-${SERVER_ID}.log"
```

### Why This Will Matter

Scripts that depend on hostname break when:
- Moving between environments (dev, staging, prod)
- Running in containers with random hostnames
- Hostname changes during system reconfiguration

### Planned Fix

Use explicit configuration:
```bash
# Deterministic - passed as parameter
SERVER_ID="${1:?Error: SERVER_ID required}"
LOG_FILE="/var/log/app-${SERVER_ID}.log"
```

## DET006: Network Queries for Dynamic Data (Planned)

**Status**: Not yet implemented

### What it Will Detect

Scripts that query external services for dynamic data:
```bash
# Non-deterministic - result changes over time
LATEST_VERSION=$(curl -s https://api.example.com/latest)
IP_ADDRESS=$(curl -s ifconfig.me)
```

### Why This Will Matter

Network-dependent scripts break determinism because:
- API responses change over time
- Network failures cause flakiness
- Different results in different networks

### Planned Fix

Cache or pin dependencies:
```bash
# Deterministic - explicit version
LATEST_VERSION="1.2.3"

# Or use vendored/cached data
LATEST_VERSION=$(cat .version-cache)
```

## Running Determinism Linting

### Lint a Single File

```bash
bashrs lint script.sh
```

### Lint All Scripts in Project

```bash
find . -name "*.sh" -exec bashrs lint {} \;
```

### Filter Only Determinism Rules

```bash
bashrs lint --rules DET script.sh
```

### CI/CD Integration

```yaml
# .github/workflows/lint.yml
name: Determinism Lint
on: [push, pull_request]
jobs:
  determinism:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install bashrs
        run: cargo install bashrs
      - name: Check determinism
        run: |
          find . -name "*.sh" -exec bashrs lint --rules DET {} \;
```

## Testing Determinism

### Property-Based Testing

Use proptest to verify deterministic properties:

```rust,ignore
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_script_deterministic(input in "[a-z]{1,10}") {
        // Run script twice with same input
        let output1 = run_script(&input);
        let output2 = run_script(&input);

        // Outputs MUST be identical
        prop_assert_eq!(output1, output2);
    }
}
```

### Manual Testing

Run scripts multiple times and verify identical output:

```bash
#!/bin/bash
# test-determinism.sh

SCRIPT="$1"
RUNS=10

echo "Testing determinism of: $SCRIPT"

# Capture first run output
EXPECTED=$("$SCRIPT")

# Run multiple times and compare
for i in $(seq 2 $RUNS); do
    ACTUAL=$("$SCRIPT")

    if [ "$EXPECTED" != "$ACTUAL" ]; then
        echo "❌ FAIL: Run $i produced different output"
        echo "Expected: $EXPECTED"
        echo "Actual: $ACTUAL"
        exit 1
    fi
done

echo "✅ PASS: All $RUNS runs produced identical output"
```

## Common Patterns

### Pattern 1: Version-Based Identifiers

Replace random/timestamp with version:

```bash
#!/bin/bash
# Deterministic deployment
VERSION="${1:?Error: VERSION required}"
RELEASE="release-${VERSION}"
ARTIFACT="app-${VERSION}.tar.gz"

echo "Deploying: $RELEASE"
```

### Pattern 2: Git-Based Identifiers

Use git commit for reproducibility:

```bash
#!/bin/bash
# Reproducible with git
COMMIT=$(git rev-parse --short HEAD)
BUILD_ID="build-${COMMIT}"

echo "Building: $BUILD_ID"
```

### Pattern 3: Explicit Input

Pass all varying data as arguments:

```bash
#!/bin/bash
# Deterministic - all inputs explicit
SESSION_ID="$1"
TIMESTAMP="$2"
RELEASE="release-${SESSION_ID}-${TIMESTAMP}"

echo "Release: $RELEASE"
```

### Pattern 4: Sorted Operations

Always sort when order matters:

```bash
#!/bin/bash
# Deterministic file processing
mapfile -t FILES < <(find . -name "*.txt" | sort)

for file in "${FILES[@]}"; do
    process "$file"
done
```

## Benefits of Determinism

### Reproducible Builds

Same inputs always produce same outputs:
- Security auditing
- Build verification
- Compliance (SLSA, SBOM)

### Reliable Testing

Tests produce consistent results:
- No flaky tests
- Reliable CI/CD
- Faster debugging

### Easier Debugging

Issues can be reproduced:
- Same inputs recreate bugs
- Log files are predictable
- Bisection works reliably

### Better Collaboration

Team members get consistent results:
- Same build artifacts
- Predictable behavior
- Reduced "works on my machine"

## Further Reading

- [Reproducible Builds](https://reproducible-builds.org/)
- [SOURCE_DATE_EPOCH](https://reproducible-builds.org/docs/source-date-epoch/)
- [SLSA Framework](https://slsa.dev/)
- [Deterministic Algorithms](https://en.wikipedia.org/wiki/Deterministic_algorithm)

---

**Quality Guarantee**: All DET rules undergo mutation testing and property-based testing to ensure reliable detection of non-deterministic patterns.
