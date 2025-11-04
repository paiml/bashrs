# Your First Purification

This tutorial walks you through purifying your first bash script using Rash. You'll learn how to transform a messy, non-deterministic bash script into clean, safe, deterministic POSIX shell code.

## What You'll Learn

- How to purify a bash script
- What transformations Rash applies
- How to verify purified output
- How to run purified scripts

## Prerequisites

- Rash installed (`bashrs --version` should work)
- A text editor
- Basic shell scripting knowledge

## The Problem: A Messy Deployment Script

Let's start with a realistic deployment script that has several problems:

```bash
#!/bin/bash
# deploy.sh - Deploy application to production

# Generate random session ID
SESSION_ID=$RANDOM

# Create timestamped release
RELEASE="release-$(date +%s)"
BUILD_ID=$$

echo "Starting deployment..."
echo "Session: $SESSION_ID"
echo "Release: $RELEASE"
echo "Build ID: $BUILD_ID"

# Create release directory (not idempotent!)
mkdir /tmp/demo-app/releases/$RELEASE

# Copy application files
cp -r ./app/* /tmp/demo-app/releases/$RELEASE/

# Update symlink (not idempotent!)
rm /tmp/demo-app/current
ln -s /tmp/demo-app/releases/$RELEASE /tmp/demo-app/current

echo "Deployment complete!"
```

Save this as `deploy.sh` and make it executable:

```bash
chmod +x deploy.sh
```

### Problems with This Script

1. **Non-deterministic**: Uses `$RANDOM`, `$(date +%s)`, and `$$`
   - Different output every run, even with same inputs
   - Impossible to reproduce deployments
   - Breaks testing and CI/CD pipelines

2. **Non-idempotent**: Uses `mkdir`, `rm`, `ln -s` without safety flags
   - Fails on second run (mkdir: cannot create directory '/tmp/demo-app/releases/...': File exists)
   - Not safe to re-run after failure
   - Manual cleanup required between runs

3. **Bash-specific**: Uses bash shebang and constructs
   - Won't run on minimal systems (Alpine, busybox)
   - Not POSIX compliant

## Step 1: Lint the Script

Before purifying, let's see what Rash detects:

```bash
bashrs lint deploy.sh
```

Output:
```
deploy.sh:5:13: DET001 [Error] Non-deterministic: $RANDOM
deploy.sh:8:20: DET002 [Error] Non-deterministic: $(date +%s)
deploy.sh:9:11: DET003 [Error] Non-deterministic: $$

deploy.sh:18:1: IDEM001 [Error] mkdir without -p (not idempotent)
deploy.sh:24:1: IDEM002 [Error] rm without -f (not idempotent)
deploy.sh:25:1: IDEM003 [Error] ln -s without cleanup (not idempotent)

6 issues found (6 errors, 0 warnings)
```

**Analysis**: Rash found **6 critical issues** that make this script unsafe for production.

## Step 2: Purify the Script

Now let's purify the script:

```bash
bashrs purify deploy.sh -o deploy_purified.sh
```

Output:
```
Purifying deploy.sh...
✓ Removed 3 non-deterministic patterns
✓ Fixed 3 idempotency issues
✓ Converted to POSIX shell
✓ Quoted all variables

Purified script written to deploy_purified.sh

Purification Report:
  Determinism: 3 issues fixed
    - $RANDOM → function parameter
    - $(date +%s) → version parameter
    - $$ → removed (not needed)

  Idempotency: 3 issues fixed
    - mkdir → mkdir -p
    - rm → rm -f
    - ln -s → rm -f && ln -s

  POSIX Compliance: ✓
  Variable Quoting: ✓ All variables quoted

Quality: ✅ Safe for production
```

## Step 3: Review the Purified Script

Let's look at the purified output:

```bash
cat deploy_purified.sh
```

```bash
#!/bin/sh
# Purified by Rash v6.30.1

deploy_app() {
    _version="${1}"
    _session_id="${2:-default-session}"

    # Deterministic release name (was timestamp)
    release="release-${_version}"

    echo "Starting deployment..."
    echo "Session: ${_session_id}"
    echo "Release: ${release}"

    # Idempotent: mkdir -p (safe to re-run)
    mkdir -p "/tmp/demo-app/releases/${release}"

    # Copy application files
    cp -r ./app/* "/tmp/demo-app/releases/${release}/"

    # Idempotent: remove old link, create new
    rm -f "/tmp/demo-app/current"
    ln -s "/tmp/demo-app/releases/${release}" "/tmp/demo-app/current"

    echo "Deployment complete!"
}

# Parse command-line arguments
if [ -z "${1}" ]; then
    echo "Usage: $0 <version> [session-id]"
    echo "Example: $0 v1.0.0 session-abc123"
    exit 1
fi

deploy_app "${1}" "${2}"
```

### What Changed?

**Determinism** ✅:
- `$RANDOM` → Parameter `${2}` (session ID)
- `$(date +%s)` → Parameter `${1}` (version)
- `$$` → Removed (not needed for logging)

**Idempotency** ✅:
- `mkdir` → `mkdir -p` (creates parent dirs, succeeds if exists)
- `rm` → `rm -f` (force, no error if missing)
- `ln -s` → `rm -f && ln -s` (remove old link first)

**POSIX Compliance** ✅:
- `#!/bin/bash` → `#!/bin/sh` (works on any POSIX shell)
- All variables quoted: `"${variable}"` (prevents word splitting)
- Function-based structure (better organization)

**Usability** ✅:
- Added argument validation
- Added usage help
- Clear error messages

## Step 4: Verify with Shellcheck

Verify the purified script passes POSIX compliance:

```bash
shellcheck -s sh deploy_purified.sh
```

Output:
```
(no output = all checks passed ✓)
```

Perfect! The purified script passes all shellcheck validation.

## Step 5: Test the Purified Script

Let's test that the purified script works correctly:

```bash
# First run
./deploy_purified.sh v1.0.0 session-abc123
```

Output:
```
Starting deployment...
Session: session-abc123
Release: release-v1.0.0
Deployment complete!
```

```bash
# Second run (should succeed, not fail!)
./deploy_purified.sh v1.0.0 session-abc123
```

Output:
```
Starting deployment...
Session: session-abc123
Release: release-v1.0.0
Deployment complete!
```

**Success!** The script runs successfully multiple times without errors.

## Step 6: Compare Original vs Purified

Let's verify the behavioral difference:

### Original Script (Fails on 2nd Run)

```bash
# First run
./deploy.sh
```
Output:
```
Starting deployment...
Session: 12345
Release: release-1699564800
Build ID: 98765
Deployment complete!
```

```bash
# Second run (FAILS!)
./deploy.sh
```
Output:
```
Starting deployment...
Session: 67890  ← Different!
Release: release-1699564801  ← Different!
Build ID: 98766  ← Different!
mkdir: cannot create directory '/tmp/demo-app/releases/release-1699564801': File exists
rm: cannot remove '/tmp/demo-app/current': No such file or directory
```

**Problem**: Script fails on re-run and produces different output each time.

### Purified Script (Safe to Re-Run)

```bash
# First run
./deploy_purified.sh v1.0.0 session-abc123
```
Output:
```
Starting deployment...
Session: session-abc123
Release: release-v1.0.0
Deployment complete!
```

```bash
# Second run (SUCCEEDS!)
./deploy_purified.sh v1.0.0 session-abc123
```
Output:
```
Starting deployment...
Session: session-abc123  ← Same
Release: release-v1.0.0  ← Same
Deployment complete!
```

**Success!** Purified script:
- Produces identical output every run (deterministic)
- Succeeds on re-run (idempotent)
- Takes version as input (controllable)

## Understanding the Purification Formula

```
Purification = Determinism + Idempotency + POSIX Compliance
```

**Determinism**: Same input → Same output (always)
- Version-based naming instead of timestamps
- Parameter-based IDs instead of `$RANDOM`
- Reproducible deployments

**Idempotency**: Safe to re-run (multiple runs = single run)
- `mkdir -p` instead of `mkdir`
- `rm -f` instead of `rm`
- Clean before creating symlinks

**POSIX Compliance**: Runs anywhere (sh, dash, ash, busybox, bash)
- `#!/bin/sh` instead of `#!/bin/bash`
- POSIX-compliant constructs only
- Passes shellcheck validation

## What's Next?

Now that you've purified your first script, try:

1. **Lint your existing scripts**: Run `bashrs lint` on your bash scripts
2. **Purify production scripts**: Use `bashrs purify` on deployment scripts
3. **Learn advanced purification**: Read [Purification Concepts](../concepts/purification.md)
4. **Explore the REPL**: Try `bashrs repl` for interactive testing
5. **Write custom rules**: Create project-specific linting rules

## Common Use Cases

**CI/CD Pipelines**:
```bash
bashrs purify ci-deploy.sh -o ci-deploy-safe.sh
# Now safe to run in GitHub Actions, GitLab CI, etc.
```

**Configuration Management**:
```bash
bashrs purify setup-server.sh -o setup-server-safe.sh
# Idempotent server provisioning
```

**Bootstrap Installers**:
```bash
bashrs purify install.sh -o install-posix.sh
# Works on minimal Alpine containers
```

**Legacy Script Migration**:
```bash
bashrs purify legacy-backup.sh -o backup-v2.sh
# Modernize old bash scripts
```

## Tips for Best Results

1. **Start with linting**: Always lint before purifying to understand issues
2. **Review purified output**: Check that behavior is preserved
3. **Test thoroughly**: Run purified scripts in test environment first
4. **Version control**: Commit both original and purified for comparison
5. **Iterate**: Purification is a process, refine over multiple iterations

## Troubleshooting

**Q: Purified script behaves differently?**

A: Check that you're passing required parameters. Purified scripts often require explicit inputs instead of generating random values.

Before:
```bash
./deploy.sh  # Works (uses $RANDOM, timestamps)
```

After:
```bash
./deploy_purified.sh v1.0.0 session-abc123  # Requires version, session ID
```

**Q: Shellcheck still reports warnings?**

A: Run with `-s sh` flag to validate POSIX compliance:
```bash
shellcheck -s sh deploy_purified.sh
```

**Q: Script fails in production?**

A: Verify the purified script was tested in an environment similar to production. Use Docker to test:
```bash
docker run --rm -v "$PWD:/work" alpine:latest sh /work/deploy_purified.sh v1.0.0 session-test
```

## Summary

You've successfully purified your first bash script! You've learned:

- ✅ How to identify issues with `bashrs lint`
- ✅ How to purify scripts with `bashrs purify`
- ✅ What transformations Rash applies (determinism, idempotency, POSIX)
- ✅ How to verify purified output with shellcheck
- ✅ How to test purified scripts safely

**Next**: Explore the [Interactive REPL](./repl.md) to test bash constructs interactively, or dive into [Core Concepts](../concepts/purification.md) to understand purification deeply.

---

**Congratulations!** You're now ready to purify production bash scripts with confidence.
