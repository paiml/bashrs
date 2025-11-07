# Release Process

This guide documents the mandatory release protocol for Rash (bashrs). Following Toyota Way principles, **a release is NOT complete until it's available on BOTH GitHub AND crates.io**.

## Release Philosophy

Rash follows **zero-defect quality standards** for all releases:

- **🚨 Jidoka (自働化)**: Build quality into the release process - all tests must pass
- **🔍 Hansei (反省)**: Reflect on what could be improved in release process
- **📈 Kaizen (改善)**: Continuously improve release automation
- **🎯 Genchi Genbutsu (現地現物)**: Verify the release works for real users (test install)

**Critical**: GitHub releases alone are insufficient for Rust projects. Users install via `cargo install bashrs`, which pulls from crates.io. If you don't publish to crates.io, users cannot get the update.

## The 5-Phase Release Process

Every release (major, minor, or patch) MUST follow all 5 phases in order.

### Phase 1: Quality Verification

**STOP THE LINE if ANY check fails**. Do not proceed to Phase 2 until all quality gates pass.

- [ ] ✅ **All tests pass**: `cargo test --lib` (100% pass rate required)
- [ ] ✅ **Integration tests pass**: All CLI and end-to-end tests
- [ ] ✅ **Clippy clean**: `cargo clippy --all-targets -- -D warnings`
- [ ] ✅ **Format check**: `cargo fmt -- --check`
- [ ] ✅ **No regressions**: All existing features still work
- [ ] ✅ **Shellcheck**: All generated scripts pass `shellcheck -s sh`
- [ ] ✅ **Book updated**: `./scripts/check-book-updated.sh` (enforces book examples pass)

**Example verification**:
```bash
# Run all quality gates
cargo test --lib                    # All tests passing?
cargo clippy --all-targets -- -D warnings  # Zero warnings?
cargo fmt -- --check                 # Formatted?
./scripts/check-book-updated.sh      # Book updated?
```

If any check fails, fix it immediately before continuing.

### Phase 2: Documentation

Update all documentation **before** creating the release commit.

- [ ] ✅ **CHANGELOG.md updated**: Complete release notes with:
  - Version number and date
  - All bug fixes with issue numbers
  - All new features
  - Breaking changes (if any)
  - Migration guide (if breaking changes)
  - Quality metrics (tests passing, coverage, mutation scores)

- [ ] ✅ **README.md updated**: If new features added

- [ ] ✅ **Version bumped**: Update `Cargo.toml` workspace version
  ```toml
  [workspace.package]
  version = "6.30.2"  # Update this
  ```

- [ ] ✅ **Book updated**: New features documented in `book/` with tested examples
  ```bash
  # Verify all book examples compile and pass
  mdbook test book
  ```

  Update relevant chapters:
  - `getting-started/` - Installation, quick start
  - `concepts/` - Core concepts if changed
  - `linting/` - New rules or rule changes
  - `config/` - New configuration options
  - `examples/` - Practical examples

**CRITICAL**: Cannot release without book update (enforced by quality gates).

### Phase 3: Git Release

Create the release commit and tag.

**Step 1: Create Release Commit**
```bash
# Stage all changes
git add CHANGELOG.md Cargo.toml book/ rash/ docs/

# Create commit with detailed release notes
git commit -m "release: v6.30.2 - Brief description

Detailed release notes:
- Feature 1: Description
- Feature 2: Description
- Bug fix: Issue #X description

Quality Metrics:
- Tests: 6321 passing (100%)
- Coverage: 87.3%
- Mutation: 81.2% average (SEC rules)
- Book: Updated with tested examples

🤖 Generated with Claude Code
Co-Authored-By: Claude <noreply@anthropic.com>"
```

**Step 2: Create Annotated Tag**
```bash
# Create tag with release notes summary
git tag -a v6.30.2 -m "v6.30.2 - Brief description

## Highlights
- Key feature or fix 1
- Key feature or fix 2

## Quality
- 6321 tests passing
- Book updated

See CHANGELOG.md for full details."
```

**Step 3: Push to GitHub**
```bash
# Push both commit and tags
git push && git push --tags
```

**Verify**: Check https://github.com/paiml/bashrs/releases

### Phase 4: crates.io Release

**MANDATORY - DO NOT SKIP THIS PHASE**

This is the most critical phase. If you skip this, users cannot install the new version.

**Step 1: Dry Run Verification**
```bash
# Test the publish process (does NOT actually publish)
cargo publish --dry-run
```

Review the output for any warnings or errors. Common issues:
- Missing metadata in Cargo.toml
- Files excluded by .gitignore that should be included
- Dependencies not available on crates.io

**Step 2: Review Package Contents**
```bash
# See exactly what will be published
cargo package --list
```

Verify all necessary files are included:
- `src/` - Source code
- `Cargo.toml` - Package metadata
- `README.md` - User documentation
- `LICENSE` - License file (MIT)

**Step 3: Publish to crates.io**
```bash
# Actually publish the release
cargo publish
```

This will:
1. Build the package
2. Upload to crates.io
3. Trigger documentation build on docs.rs

**Step 4: Verify Publication**

Check that the release is live:
```bash
# Verify on crates.io
open https://crates.io/crates/bashrs

# Verify version is listed
open https://crates.io/crates/bashrs/versions
```

**Step 5: Test Installation**
```bash
# Test that users can install
cargo install bashrs --version 6.30.2

# Verify installed version
bashrs --version
# Should output: bashrs 6.30.2
```

### Phase 5: Post-Release Verification

Verify the release is accessible through all channels.

- [ ] ✅ **GitHub release visible**: https://github.com/paiml/bashrs/releases
- [ ] ✅ **crates.io listing updated**: https://crates.io/crates/bashrs
- [ ] ✅ **Installation works**: `cargo install bashrs`
- [ ] ✅ **Documentation builds**: https://docs.rs/bashrs
- [ ] ✅ **Version correct**: `bashrs --version` shows new version

**Example verification commands**:
```bash
# Check GitHub releases
open https://github.com/paiml/bashrs/releases/tag/v6.30.2

# Check crates.io
open https://crates.io/crates/bashrs

# Test fresh install
cargo install bashrs --force --version 6.30.2
bashrs --version
bashrs lint examples/security/sec001_eval.sh
```

## Semantic Versioning

Rash follows [Semantic Versioning 2.0.0](https://semver.org/) strictly.

### MAJOR Version (x.0.0) - Breaking Changes

Increment when you make incompatible API changes:
- Removal of public APIs
- Changed function signatures
- Removal of CLI commands or options
- Major workflow changes

**Example**: v1.0.0 → v2.0.0
```text
Breaking Changes:
- Removed deprecated `rash compile` command (use `rash transpile`)
- Changed CLI: `--output-dir` renamed to `--out-dir`
- API: Removed `purify::legacy_mode()` function

Migration Guide:
1. Replace `rash compile` with `rash transpile`
2. Update scripts: `--output-dir` → `--out-dir`
3. Remove calls to `purify::legacy_mode()`
```

### MINOR Version (0.x.0) - New Features

Increment when you add functionality in a backward-compatible manner:
- New CLI commands
- New linter rules
- New configuration options
- Performance improvements
- New features that don't break existing code

**Example**: v2.0.0 → v2.1.0
```text
New Features:
- Added SEC009 rule: Detect unsafe shell redirects
- New command: `bashrs bench` for performance measurement
- Configuration: Added `linter.max_warnings` option
- Performance: 40% faster parsing with new incremental parser

All existing code continues to work without changes.
```

### PATCH Version (0.0.x) - Bug Fixes Only

Increment when you make backward-compatible bug fixes:
- Critical bug fixes
- Security fixes
- Documentation fixes
- No new features
- No API changes

**Example**: v2.0.0 → v2.0.1
```text
Bug Fixes:
- Fixed Issue #1: Auto-fix incorrectly handled nested quotes
- Security: Fixed SEC001 false positive on commented eval
- Docs: Updated installation instructions for Arch Linux

No new features. No breaking changes.
```

## Example: Complete v2.0.1 Release

This example shows the actual release process for v2.0.1 (Issue #1 fix):

```bash
# ============================================================
# Phase 1: Quality Verification
# ============================================================
cargo test --lib
# Output: test result: ok. 1,545 passed ✅

cargo clippy --all-targets -- -D warnings
# Output: 0 warnings ✅

cargo fmt -- --check
# Output: (no output = formatted) ✅

./scripts/check-book-updated.sh
# Output: Book examples passing ✅

# ============================================================
# Phase 2: Documentation
# ============================================================
# Updated CHANGELOG.md with Issue #1 fix details
# Bumped Cargo.toml: 2.0.0 → 2.0.1
# Updated book/src/linting/auto-fix.md with corrected example

# ============================================================
# Phase 3: Git Release
# ============================================================
git add CHANGELOG.md Cargo.toml book/ rash/src/linter/rules/sec001.rs \
        rash/tests/test_issue_001_autofix.rs docs/

git commit -m "fix: v2.0.1 - Critical auto-fix bug (Issue #1)

Fixed auto-fix incorrectly handling nested quotes in SEC001 rule.

Bug: Auto-fix for eval with nested quotes produced invalid syntax
Fix: Improved quote escaping in auto-fix transformer
Tests: Added test_issue_001_autofix regression test

Quality Metrics:
- Tests: 1,545 passing (100%)
- Regression test added and passing
- Book updated with corrected examples

Closes #1

🤖 Generated with Claude Code
Co-Authored-By: Claude <noreply@anthropic.com>"

git tag -a v2.0.1 -m "v2.0.1 - Critical Auto-Fix Bug Fix

## Bug Fix
- Fixed Issue #1: Auto-fix nested quote handling

## Quality
- 1,545 tests passing
- Regression test added
- Book examples corrected

This is a critical patch release fixing auto-fix behavior."

git push && git push --tags
# Pushed to GitHub ✅

# ============================================================
# Phase 4: crates.io Release (MANDATORY)
# ============================================================
cargo publish --dry-run
# Output: Packaging bashrs v2.0.1... ✅

cargo package --list
# Verify contents look correct ✅

cargo publish
# Uploading bashrs v2.0.1 to crates.io... ✅
# Published successfully! ✅

# ============================================================
# Phase 5: Verification
# ============================================================
open https://github.com/paiml/bashrs/releases/tag/v2.0.1
# GitHub release visible ✅

open https://crates.io/crates/bashrs
# Version 2.0.1 listed ✅

cargo install bashrs --version 2.0.1 --force
# Installed successfully ✅

bashrs --version
# bashrs 2.0.1 ✅

# ============================================================
# RELEASE COMPLETE ✅
# ============================================================
```

## Common Mistakes to Avoid

### ❌ DO NOT:

1. **Skip crates.io publishing** (users won't get the update)
   ```bash
   # Wrong: Only push to GitHub
   git push && git push --tags
   # Right: Also publish to crates.io
   git push && git push --tags && cargo publish
   ```

2. **Release without updating CHANGELOG.md**
   ```text
   # Wrong: Empty or outdated CHANGELOG
   # Right: Complete, detailed release notes
   ```

3. **Release with failing tests**
   ```bash
   # Wrong: Skip test verification
   git tag v6.30.2

   # Right: Verify all tests pass first
   cargo test --lib && git tag v6.30.2
   ```

4. **Release without testing the package**
   ```bash
   # Wrong: Publish without dry run
   cargo publish

   # Right: Always dry run first
   cargo publish --dry-run && cargo publish
   ```

5. **Create release without git tag**
   ```bash
   # Wrong: Only commit
   git commit -m "release v6.30.2"

   # Right: Commit AND tag
   git commit -m "release v6.30.2" && git tag -a v6.30.2
   ```

6. **Push tag before verifying local tests**
   ```bash
   # Wrong: Push untested code
   git tag v6.30.2 && git push --tags

   # Right: Test first, then push
   cargo test --lib && git tag v6.30.2 && git push --tags
   ```

### ✅ ALWAYS:

1. **Publish to BOTH GitHub and crates.io**
2. **Follow all 5 phases in order**
3. **Test the package before publishing** (dry run)
4. **Update all documentation** (CHANGELOG, README, book)
5. **Verify the release after publishing** (test install)

## crates.io Publishing Requirements

Before publishing to crates.io, ensure your `Cargo.toml` has complete metadata:

```toml
[package]
name = "bashrs"
version = "6.30.2"
description = "Shell safety and purification tool with linting"
license = "MIT"
repository = "https://github.com/paiml/bashrs"
homepage = "https://github.com/paiml/bashrs"
keywords = ["shell", "bash", "linter", "security", "posix"]
categories = ["command-line-utilities", "development-tools"]
```

**Required**:
- `description` - Clear package description
- `license` - License identifier (MIT)
- `repository` - GitHub repository URL
- `homepage` - Project homepage
- `keywords` - Relevant keywords (max 5)
- `categories` - Cargo categories

**Authentication**:
```bash
# Configure crates.io API token (first time only)
cargo login <your-api-token>
```

Get your API token from https://crates.io/me

**Verification Before Publishing**:
```bash
# Ensure no uncommitted changes
git status  # Should be clean

# Verify version not already published
open https://crates.io/crates/bashrs/versions

# Cannot republish same version
```

## Release Frequency

**Patch releases** (bug fixes):
- **When**: As needed, within 24-48 hours of critical bugs
- **Example**: v6.30.1 → v6.30.2 (SEC001 false positive fix)

**Minor releases** (new features):
- **When**: Monthly or when significant feature is complete
- **Example**: v6.30.0 → v6.32.1 (added SEC009-SEC012 rules)

**Major releases** (breaking changes):
- **When**: Quarterly or when necessary for major improvements
- **Example**: v6.0.0 → v7.0.0 (removed deprecated APIs, new architecture)

## Release Checklist (Quick Reference)

Copy this checklist for each release:

```markdown
## Release vX.Y.Z Checklist

### Phase 1: Quality Verification
- [ ] All tests pass (`cargo test --lib`)
- [ ] Integration tests pass
- [ ] Clippy clean (`cargo clippy --all-targets -- -D warnings`)
- [ ] Format check (`cargo fmt -- --check`)
- [ ] No regressions
- [ ] Shellcheck passes
- [ ] Book updated (`./scripts/check-book-updated.sh`)

### Phase 2: Documentation
- [ ] CHANGELOG.md updated with complete notes
- [ ] README.md updated (if needed)
- [ ] Cargo.toml version bumped
- [ ] Book updated with tested examples
- [ ] `mdbook test book` passes

### Phase 3: Git Release
- [ ] Release commit created
- [ ] Git tag created (annotated)
- [ ] Pushed to GitHub (commit + tags)
- [ ] GitHub release visible

### Phase 4: crates.io Release
- [ ] Dry run passed (`cargo publish --dry-run`)
- [ ] Package contents reviewed (`cargo package --list`)
- [ ] Published to crates.io (`cargo publish`)
- [ ] crates.io listing updated
- [ ] Test install works

### Phase 5: Verification
- [ ] GitHub release visible
- [ ] crates.io listing shows new version
- [ ] `cargo install bashrs` works
- [ ] docs.rs documentation built
- [ ] `bashrs --version` shows correct version
```

## Troubleshooting

### Publication Failed: "crate name is already taken"

This means the version is already published. You cannot republish the same version.

**Solution**: Bump the version number and try again.
```bash
# Update version in Cargo.toml
version = "6.30.3"  # Increment

# Re-run Phase 4
cargo publish --dry-run
cargo publish
```

### Publication Failed: "missing field `description`"

Your `Cargo.toml` is missing required metadata.

**Solution**: Add all required fields to `Cargo.toml`:
```toml
description = "Shell safety and purification tool with linting"
license = "MIT"
repository = "https://github.com/paiml/bashrs"
```

### Tests Failing After Version Bump

Likely a test hardcodes the version string.

**Solution**: Update version-checking tests:
```rust
#[test]
fn test_version() {
    assert_eq!(VERSION, "6.30.2"); // Update this
}
```

### docs.rs Build Failed

Check build status at https://docs.rs/crate/bashrs

**Common causes**:
- Missing dependencies in Cargo.toml
- Doc tests failing
- Feature flags not configured

**Solution**: Fix the issue and publish a patch release.

## Summary

A complete release requires:

1. ✅ **All quality gates pass** (tests, clippy, format, shellcheck)
2. ✅ **Documentation updated** (CHANGELOG, README, book, version)
3. ✅ **Git release created** (commit, tag, push)
4. ✅ **crates.io published** (dry run, review, publish)
5. ✅ **Verification complete** (GitHub, crates.io, install, docs)

**Remember**: A release is NOT complete until it's available on crates.io. GitHub releases alone are insufficient for Rust projects.

---

**Toyota Way Applied**:
- **Jidoka**: Build quality in - all tests must pass before release
- **Hansei**: Reflect on release process after each release
- **Kaizen**: Continuously improve release automation
- **Genchi Genbutsu**: Verify release works for real users
