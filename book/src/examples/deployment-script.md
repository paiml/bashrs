# Deployment Script Example

This chapter demonstrates purifying a real-world deployment script, transforming it from messy, non-deterministic bash into safe, deterministic, idempotent POSIX shell.

## The Problem: Messy Deployment Scripts

Typical deployment scripts have serious issues:
- **Non-deterministic**: Using `$RANDOM`, timestamps, process IDs
- **Non-idempotent**: Operations fail on re-run
- **Error-prone**: No validation, poor error handling
- **Unportable**: Bash-specific constructs

### Example: Problematic Deployment Script

```bash
#!/bin/bash
# deploy-messy.sh - PROBLEMATIC bash script

# Non-deterministic: uses $RANDOM
SESSION_ID=$RANDOM

# Non-deterministic: uses timestamps
RELEASE_TAG="release-$(date +%Y%m%d-%H%M%S)"

# Process-dependent paths
WORK_DIR="/tmp/deploy-$$"
LOG_FILE="/var/log/deploy-$SECONDS.log"

# Non-idempotent operations
rm /app/current                          # ❌ Fails if doesn't exist
mkdir /app/releases/$RELEASE_TAG         # ❌ Fails if already exists

# Extract archive
tar xzf app.tar.gz -C /app/releases/$RELEASE_TAG

# Create symlink (fails if exists)
ln -s /app/releases/$RELEASE_TAG /app/current  # ❌ Fails on second run

# Record deployment
echo "Session $SESSION_ID: Deployed $RELEASE_TAG at $(date)" >> $LOG_FILE

echo "Deployment complete: $RELEASE_TAG"
```

### Issues Detected by Rash

Running `bashrs lint deploy-messy.sh`:

```text
deploy-messy.sh:6:13: DET001 [Error] Non-deterministic: $RANDOM
  SESSION_ID=$RANDOM
  Fix: Use configuration parameter or version-based ID

deploy-messy.sh:9:14: DET002 [Error] Non-deterministic: timestamp
  RELEASE_TAG="release-$(date +%Y%m%d-%H%M%S)"
  Fix: Use fixed release tag from input parameter

deploy-messy.sh:12:11: DET003 [Error] Non-deterministic: process ID
  WORK_DIR="/tmp/deploy-$$"
  Fix: Use fixed directory or version-based name

deploy-messy.sh:13:11: DET003 [Error] Non-deterministic: $SECONDS
  LOG_FILE="/var/log/deploy-$SECONDS.log"
  Fix: Use version-based log file name

deploy-messy.sh:16:1: IDEM002 [Error] Non-idempotent: rm without -f
  rm /app/current
  Fix: Use rm -f for idempotent deletion

deploy-messy.sh:17:1: IDEM001 [Error] Non-idempotent: mkdir without -p
  mkdir /app/releases/$RELEASE_TAG
  Fix: Use mkdir -p for idempotent directory creation

deploy-messy.sh:23:1: IDEM003 [Error] Non-idempotent: ln -s without cleanup
  ln -s /app/releases/$RELEASE_TAG /app/current
  Fix: Remove existing link before creating new one

7 issues found (7 errors, 0 warnings)
```

---

## The Solution: Purified Deployment Script

### Step 1: Purify the Script

```bash
bashrs purify deploy-messy.sh -o deploy-purified.sh --report
```

### Purified Output

```bash
#!/bin/sh
# deploy-purified.sh - Purified by bashrs v6.31.0
# This is safe, deterministic, idempotent POSIX shell

deploy_app() {
    _version="$1"
    _build_dir="$2"

    # ✅ Purified: Deterministic (was $RANDOM and timestamp)
    session_id="session-${_version}"
    release_tag="release-${_version}"

    # ✅ Purified: All variables quoted (was SC2086)
    target_dir="/app/releases/${release_tag}"

    # ✅ Purified: Idempotent (was mkdir)
    mkdir -p "${target_dir}" || return 1

    # ✅ Purified: Safe directory iteration (was unquoted $(ls))
    if [ ! -d "${_build_dir}" ]; then
        printf 'Build directory does not exist: %s\n' "${_build_dir}" >&2
        return 1
    fi

    # ✅ Purified: Properly quoted iteration (was SC2046)
    for entry in "${_build_dir}"/*; do
        if [ -f "${entry}" ]; then
            cp "${entry}" "${target_dir}/" || return 1
        elif [ -d "${entry}" ]; then
            cp -r "${entry}" "${target_dir}/" || return 1
        fi
    done

    # ✅ Purified: Idempotent (remove if exists, then create)
    current_link="/app/current"
    if [ -e "${current_link}" ] || [ -L "${current_link}" ]; then
        rm -f "${current_link}" || return 1
    fi

    # ✅ Purified: All variables quoted
    ln -s "${target_dir}" "${current_link}" || return 1

    printf 'Deployed %s to %s\n' "${release_tag}" "${target_dir}"
    return 0
}

# Main execution
_version="${1:-1.0.0}"
_build_dir="${2:-/app/build}"

deploy_app "${_version}" "${_build_dir}"
```

### Purification Report

```text
Purification Report
===================

Issues Fixed: 7

Determinism (4 fixes):
  1. $RANDOM → version-based ID (session-${_version})
  2. $(date +%Y%m%d-%H%M%S) → version-based tag (release-${_version})
  3. $$ (process ID) → removed (using input parameter)
  4. $SECONDS → removed (using version-based naming)

Idempotency (3 fixes):
  1. mkdir → mkdir -p (safe to re-run)
  2. rm → rm -f with existence check (no error if missing)
  3. ln -s → rm -f before ln -s (idempotent symlink)

Safety (all operations):
  - All variables quoted
  - Error checking on all operations (|| return 1)
  - Input validation (directory existence checks)

POSIX Compliance:
  ✅ Passes shellcheck -s sh
  ✅ Works on sh, dash, ash, bash, busybox
  ✅ No bash-isms
```

---

## Verification: Testing the Purified Script

### Test 1: Deterministic Output

```bash
# Run twice with same version
bashrs bench deploy-purified.sh --verify-determinism

Result:
✅ DETERMINISTIC: Output identical across 10 runs
✅ No $RANDOM, no timestamps, no process IDs
```

### Test 2: Idempotent Behavior

```bash
# Run multiple times - should succeed every time
for i in 1 2 3; do
    ./deploy-purified.sh 1.0.0 /app/build
    echo "Run $i: $?"
done

Result:
Run 1: 0  ✅ First deployment succeeds
Run 2: 0  ✅ Second deployment succeeds (idempotent)
Run 3: 0  ✅ Third deployment succeeds (idempotent)
```

### Test 3: POSIX Compliance

```bash
# Test on multiple shells
for shell in sh dash ash bash; do
    echo "Testing with $shell..."
    $shell deploy-purified.sh 1.0.0 /app/build
done

Result:
Testing with sh...    ✅ Works
Testing with dash...  ✅ Works
Testing with ash...   ✅ Works
Testing with bash...  ✅ Works
```

### Test 4: Quality Score

```bash
bashrs score deploy-purified.sh --detailed
```

Result:
```
Quality Score: A+ (98/100)

Safety:         100/100 ✅ No security issues
Determinism:    100/100 ✅ No non-deterministic patterns
Idempotency:    100/100 ✅ Safe to re-run
POSIX:          100/100 ✅ Fully portable
Code Quality:    90/100 ⚠️ Minor style improvements possible

Overall: EXCELLENT - Production ready
```

---

## Production-Ready Deployment Script

For production deployments, add error handling, logging, and health checks:

```bash
#!/bin/sh
# deploy-production.sh - Production-ready deployment script
# Purified by bashrs v6.31.0

set -eu

# Configuration
readonly APP_NAME='myapp'
readonly DEPLOY_DIR="/var/www/${APP_NAME}"
readonly LOG_DIR="/var/log/${APP_NAME}"
readonly HEALTH_CHECK_URL='http://localhost:8080/health'

# Logging functions
log() {
    printf '[INFO] %s: %s\n' "$(date +%Y-%m-%d)" "$*"
}

error() {
    printf '[ERROR] %s: %s\n' "$(date +%Y-%m-%d)" "$*" >&2
    exit 1
}

# Pre-deployment checks
check_requirements() {
    log "Checking requirements..."

    command -v git >/dev/null 2>&1 || error "git is required"
    command -v docker >/dev/null 2>&1 || error "docker is required"
    command -v curl >/dev/null 2>&1 || error "curl is required"

    [ -d "${DEPLOY_DIR}" ] || error "Deploy directory not found: ${DEPLOY_DIR}"

    log "All requirements satisfied"
}

# Deploy new version
deploy_version() {
    version="$1"

    log "Deploying version: ${version}"

    cd "${DEPLOY_DIR}" || error "Cannot cd to ${DEPLOY_DIR}"

    # Fetch and checkout version
    git fetch origin || error "Git fetch failed"
    git checkout "${version}" || error "Version ${version} not found"

    # Build containers
    docker-compose build || error "Docker build failed"

    # Deploy with zero downtime
    docker-compose up -d || error "Docker deployment failed"

    log "Deployment successful!"
}

# Health check with retries
health_check() {
    log "Running health check..."

    max_attempts=30
    attempt=0

    while [ "${attempt}" -lt "${max_attempts}" ]; do
        if curl -sf "${HEALTH_CHECK_URL}" >/dev/null 2>&1; then
            log "Health check passed!"
            return 0
        fi

        attempt=$((attempt + 1))
        sleep 1
    done

    error "Health check failed after ${max_attempts} attempts"
}

# Backup previous version
backup_previous() {
    log "Creating backup..."

    backup_dir="${LOG_DIR}/backups"
    mkdir -p "${backup_dir}" || error "Cannot create backup directory"

    backup_file="${backup_dir}/backup-$(date +%Y%m%d-%H%M%S).tar.gz"

    tar czf "${backup_file}" -C "${DEPLOY_DIR}" . || error "Backup failed"

    log "Backup created: ${backup_file}"
}

# Rollback to previous version
rollback() {
    log "Rolling back to previous version..."

    cd "${DEPLOY_DIR}" || error "Cannot cd to ${DEPLOY_DIR}"

    git checkout HEAD~1 || error "Rollback failed"
    docker-compose up -d || error "Rollback deployment failed"

    log "Rollback complete"
}

# Main deployment workflow
deploy_app() {
    version="$1"

    log "Starting deployment of ${APP_NAME} version ${version}"

    # Pre-flight checks
    check_requirements

    # Backup current version
    backup_previous

    # Deploy new version
    deploy_version "${version}"

    # Verify deployment
    if health_check; then
        log "Deployment completed successfully!"
        return 0
    else
        error "Deployment verification failed!"
        rollback
        return 1
    fi
}

# Validate input
if [ $# -eq 0 ]; then
    error "Usage: $0 <version>"
fi

version="$1"

# Run deployment
deploy_app "${version}"
```

### Production Script Features

✅ **Error Handling**:
- `set -eu` for strict error mode
- Error checks on all critical operations
- Automatic rollback on failure

✅ **Logging**:
- Structured log format
- Timestamped entries
- Error vs info distinction

✅ **Pre-flight Checks**:
- Verify all dependencies installed
- Check directory structure
- Validate permissions

✅ **Health Checks**:
- Automated health verification
- Retry logic with timeout
- Fail fast on errors

✅ **Backup & Rollback**:
- Automatic backups before deployment
- One-command rollback
- Version history preserved

✅ **Zero Downtime**:
- Docker-compose orchestration
- Graceful container replacement
- Health check before switching

---

## Complete Workflow: From Messy to Production

### Step 1: Lint Existing Script

```bash
bashrs lint deploy-messy.sh
# Identifies 7 issues (determinism + idempotency)
```

### Step 2: Purify Script

```bash
bashrs purify deploy-messy.sh -o deploy-purified.sh --report
# Fixes all 7 issues automatically
```

### Step 3: Verify Purified Script

```bash
# Verify determinism
bashrs bench deploy-purified.sh --verify-determinism

# Verify idempotency
for i in 1 2 3; do ./deploy-purified.sh 1.0.0 /app/build; done

# Quality audit
bashrs audit deploy-purified.sh --detailed
```

### Step 4: Test in Staging

```bash
# Deploy to staging
./deploy-purified.sh 1.0.0 /staging/build

# Verify deployment
curl -f http://staging:8080/health
```

### Step 5: Deploy to Production

```bash
# Use production-ready version with rollback
./deploy-production.sh 1.0.0
```

### Step 6: Monitor & Verify

```bash
# Check logs
tail -f /var/log/myapp/deploy.log

# Verify health
watch -n 1 curl -sf http://localhost:8080/health
```

---

## Common Deployment Patterns

### Pattern 1: Blue-Green Deployment

```bash
#!/bin/sh
# blue-green-deploy.sh

deploy_blue_green() {
    version="$1"
    current_color=$(cat /app/current_color)

    if [ "${current_color}" = "blue" ]; then
        new_color="green"
    else
        new_color="blue"
    fi

    # Deploy to inactive color
    deploy_to_color "${new_color}" "${version}"

    # Health check
    health_check_color "${new_color}"

    # Switch traffic
    switch_traffic "${new_color}"

    # Update current color
    printf '%s\n' "${new_color}" > /app/current_color
}
```

### Pattern 2: Canary Deployment

```bash
#!/bin/sh
# canary-deploy.sh

deploy_canary() {
    version="$1"
    canary_percent="${2:-10}"

    # Deploy canary version
    deploy_canary_version "${version}"

    # Route 10% traffic to canary
    route_traffic_percent "${canary_percent}" canary

    # Monitor metrics
    monitor_canary_metrics 300  # 5 minutes

    # If healthy, roll out to 100%
    if canary_is_healthy; then
        rollout_full "${version}"
    else
        rollback_canary
    fi
}
```

### Pattern 3: Rolling Deployment

```bash
#!/bin/sh
# rolling-deploy.sh

deploy_rolling() {
    version="$1"
    batch_size="${2:-1}"

    instances=$(get_instance_list)

    for instance in ${instances}; do
        # Deploy to instance
        deploy_to_instance "${instance}" "${version}"

        # Health check
        health_check_instance "${instance}"

        # Wait before next batch
        sleep 30
    done
}
```

---

## Integration with CI/CD

### GitHub Actions

```yaml
name: Deploy

on:
  push:
    tags:
      - 'v*'

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install bashrs
        run: cargo install bashrs

      - name: Lint deployment script
        run: bashrs lint deploy.sh --strict

      - name: Purify deployment script
        run: bashrs purify deploy.sh -o deploy-purified.sh

      - name: Verify determinism
        run: bashrs bench deploy-purified.sh --verify-determinism

      - name: Deploy to production
        run: ./deploy-purified.sh ${{ github.ref_name }}
```

### GitLab CI

```yaml
deploy:
  stage: deploy
  script:
    - cargo install bashrs
    - bashrs lint deploy.sh --strict
    - bashrs purify deploy.sh -o deploy-purified.sh
    - bashrs audit deploy-purified.sh --min-grade A
    - ./deploy-purified.sh $CI_COMMIT_TAG
  only:
    - tags
```

---

## Best Practices

### 1. Always Use Version Parameters

❌ **Bad**: `RELEASE_TAG="release-$(date +%s)"`
✅ **Good**: `RELEASE_TAG="release-${VERSION}"`

**Why**: Deterministic, reproducible, traceable

### 2. Make Operations Idempotent

❌ **Bad**: `mkdir /app/releases/${VERSION}`
✅ **Good**: `mkdir -p /app/releases/${VERSION}`

**Why**: Safe to re-run, no errors on retry

### 3. Always Quote Variables

❌ **Bad**: `cd $DEPLOY_DIR`
✅ **Good**: `cd "${DEPLOY_DIR}"`

**Why**: Prevents injection, handles spaces safely

### 4. Check Errors

❌ **Bad**: `docker-compose up -d`
✅ **Good**: `docker-compose up -d || error "Deployment failed"`

**Why**: Fail fast, prevent cascading failures

### 5. Use POSIX Shell

❌ **Bad**: `#!/bin/bash` with bash arrays
✅ **Good**: `#!/bin/sh` with POSIX constructs

**Why**: Portable, works everywhere

### 6. Add Health Checks

❌ **Bad**: Deploy and assume success
✅ **Good**: Deploy → health check → verify → rollback on failure

**Why**: Catch failures early, automatic recovery

### 7. Implement Rollback

❌ **Bad**: Manual rollback procedure
✅ **Good**: Automated rollback on health check failure

**Why**: Fast recovery, minimal downtime

---

## Troubleshooting

### Issue: Deployment Not Idempotent

**Symptom**: Second run fails with "File exists" or similar

**Solution**:
```bash
# Lint to find issues
bashrs lint deploy.sh

# Purify to fix
bashrs purify deploy.sh --fix
```

### Issue: Deployment Not Deterministic

**Symptom**: Different output on each run

**Solution**:
```bash
# Verify determinism
bashrs bench deploy.sh --verify-determinism

# Fix detected issues
bashrs lint deploy.sh --format json | grep DET
```

### Issue: Deployment Fails on Different Shells

**Symptom**: Works on bash, fails on sh/dash

**Solution**:
```bash
# Check POSIX compliance
shellcheck -s sh deploy.sh

# Purify for POSIX
bashrs purify deploy.sh --target posix
```

---

## Summary

**Key Takeaways**:

1. ✅ Use `bashrs purify` to transform messy deployment scripts
2. ✅ Verify determinism with `bashrs bench --verify-determinism`
3. ✅ Test idempotency by running multiple times
4. ✅ Add error handling and rollback logic
5. ✅ Integrate quality checks in CI/CD
6. ✅ Monitor deployments with health checks

**Results**:
- **Before**: 7 issues (determinism + idempotency)
- **After**: 0 issues, production-ready, portable

**Next Steps**:
- [Bootstrap Installer Example](./bootstrap-installer.md)
- [CI/CD Integration Example](./ci-cd-integration.md)
- [Configuration Management](./config-files.md)
- [Purification Concepts](../concepts/purification.md)
