# Dependency Management Guide

This guide covers the comprehensive dependency management system added to the rash project Makefile.

## Available Commands

### `make update-deps`
**Safe semver-compatible updates**

- Updates all dependencies within semver-compatible ranges
- Runs tests to verify compatibility
- Shows summary of updated packages
- **Recommended for regular maintenance**

```bash
$ make update-deps
🔄 Updating dependencies (semver-compatible)...
Step 1: Updating within semver-compatible ranges...
Step 2: Running tests to verify compatibility...
✅ Dependencies updated successfully!

📊 Updated packages summary:
[Shows updated dependency tree]
```

### `make update-deps-aggressive`
**Full dependency upgrades including major versions**

- Installs `cargo-edit` if needed for `cargo upgrade`
- Updates semver-compatible dependencies first
- Upgrades to latest incompatible versions (major bumps)
- Runs comprehensive tests and linting
- Performs security audit
- **Use with caution - may introduce breaking changes**

```bash
$ make update-deps-aggressive
🔄 Updating dependencies aggressively (requires cargo-edit)...
Installing cargo-edit for cargo upgrade command...
Step 1: Updating within semver-compatible ranges...
Step 2: Upgrading to latest incompatible versions (major bumps)...
Step 3: Running comprehensive tests...
Step 4: Checking for security vulnerabilities...
✅ Aggressive update completed!
```

### `make update-deps-check`
**Check for outdated dependencies without updating**

- Installs `cargo-outdated` if needed
- Checks all workspace crates for outdated dependencies
- Shows root dependencies only (cleaner output)
- Includes security advisory check
- **Perfect for CI/CD monitoring**

```bash
$ make update-deps-check
🔍 Checking for outdated dependencies...

📋 Outdated dependencies in main workspace:
[Lists outdated dependencies]

🔍 Security advisories check:
[Security audit results]
```

### `make update-deps-workspace`
**Safe workspace update with automatic rollback**

- Creates backup of all Cargo.lock files
- Updates workspace dependencies
- Tests build and runs tests
- **Automatically rolls back on failure**
- Cleans up backup files on success
- **Recommended for automated CI/CD**

```bash
$ make update-deps-workspace
🔄 Updating workspace dependencies with validation...
Step 1: Backup current Cargo.lock files...
Step 2: Updating workspace dependencies...
Step 3: Building to check for breaking changes...
Step 4: Running tests...
Step 5: Cleanup backup files...
✅ Workspace dependencies updated and validated!
```

## Workspace Structure

The dependency management system handles the multi-crate workspace:

```
rash/
├── Cargo.toml          # Main workspace
├── rash/
│   └── Cargo.toml      # Core library
├── rash-runtime/
│   └── Cargo.toml      # Runtime support
└── rash-tests/
    └── Cargo.toml      # Test utilities
```

## Integration with CI/CD

### Daily Dependency Checks

```yaml
# .github/workflows/dependencies.yml
name: Dependency Check
on:
  schedule:
    - cron: '0 6 * * *'  # Daily at 6 AM
jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: make update-deps-check
```

### Automated Updates

```yaml
# .github/workflows/update-deps.yml  
name: Update Dependencies
on:
  workflow_dispatch:  # Manual trigger
jobs:
  update:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: make update-deps-workspace
      - name: Create PR
        # Create PR with updated dependencies
```

## Security Features

All dependency update commands include:

- **Security audit** via `cargo audit`
- **Vulnerability scanning** of new dependencies
- **Test verification** to catch security regressions
- **Rollback capability** if issues detected

## Best Practices

### Regular Maintenance
```bash
# Weekly routine
make update-deps          # Safe updates
make test                 # Full test suite
make audit                # Security check
```

### Major Version Updates
```bash
# Quarterly or before releases
make update-deps-check    # See what's available
make update-deps-aggressive # Update with testing
# Review changes carefully
# Test extensively before merging
```

### CI/CD Integration
```bash
# In build pipeline
make update-deps-workspace  # Ensures current deps
make validate              # Full validation
```

## Troubleshooting

### Build Failures After Update

The workspace update automatically rolls back on build failures:

```bash
❌ Build failed after update, restoring backups...
# Cargo.lock files automatically restored
```

### Test Failures After Update

Tests failures also trigger automatic rollback:

```bash
❌ Tests failed after update, restoring backups...  
# Previous state restored automatically
```

### Manual Recovery

If needed, backup files are available:
```bash
# Manual rollback if needed
cp Cargo.lock.backup Cargo.lock
cp rash/Cargo.lock.backup rash/Cargo.lock
# etc.
```

## Tools Installed Automatically

The system automatically installs required tools:

- **cargo-edit** (for `cargo upgrade`)
- **cargo-outdated** (for dependency checking)  
- **cargo-audit** (for security scanning)

## Integration with Quality Gates

Dependency updates are integrated with the existing quality system:

```bash
make update-deps-aggressive
# Automatically runs:
# - make test-fast lint-check  (quality)
# - make audit                 (security)
# - Shows dependency tree      (visibility)
```

This ensures that dependency updates don't compromise code quality or security standards.