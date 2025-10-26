# bashrs Git Hooks

This directory contains git hooks that enforce NASA-level quality standards for bashrs development.

## Quick Start

To install the hooks, run:

```bash
./scripts/hooks/install-hooks.sh
```

## Pre-commit Hook

The pre-commit hook enforces **3 quality gates** before allowing commits:

### 1. Zero Clippy Warnings
```bash
cargo clippy --lib -- -D warnings
```
- **Purpose**: Enforce correctness and code quality
- **Standard**: Zero tolerance for warnings
- **Fix**: `cargo clippy --lib --fix --allow-dirty`

### 2. Performance Linting
```bash
cargo clippy --release -- -W clippy::perf
```
- **Purpose**: Catch performance anti-patterns in hot paths
- **Standard**: No performance issues in release mode
- **Fix**: Refactor inefficient code patterns

### 3. All Tests Passing
```bash
cargo test --lib
```
- **Purpose**: Ensure no regressions
- **Standard**: 100% test pass rate (currently 4,706 tests)
- **Fix**: Fix failing tests before committing

## Hook Execution

When you run `git commit`, the pre-commit hook will:

```
🔍 Running pre-commit quality checks...
  📋 Checking clippy (zero warnings required)...
  ✅ Clippy: Zero warnings

  ⚡ Checking performance lints (release mode)...
  ✅ Performance: No issues

  🧪 Running tests...
  ✅ Tests: All passing

✅ Pre-commit checks passed - proceeding with commit
```

**Execution time**: ~14 seconds (acceptable for commit safety)

## Bypassing Hooks (NOT RECOMMENDED)

In rare cases where you need to bypass hooks:

```bash
git commit --no-verify
```

**WARNING**: This defeats the purpose of quality gates. Only use in emergencies.

## Philosophy

Following Toyota Way principles:

- **自働化 (Jidoka)**: Build quality in automatically, not inspect it in later
- **反省 (Hansei)**: Fix problems at the source, not after the fact
- **改善 (Kaizen)**: Continuously improve quality standards

These hooks embody "stop the line" mentality - defects are caught immediately, not deferred.

## Maintenance

### Adding New Hooks

1. Create the hook file in `scripts/hooks/`
2. Make it executable: `chmod +x scripts/hooks/new-hook`
3. Update `install-hooks.sh` to install the new hook
4. Update this README

### Updating Existing Hooks

1. Edit the hook in `scripts/hooks/`
2. Team members re-run `./scripts/hooks/install-hooks.sh` to get updates
3. Document changes in this README

## Files

- `pre-commit` - The actual hook script (executable)
- `install-hooks.sh` - Installation script (executable)
- `README.md` - This file

## Quality Metrics

The pre-commit hook has maintained these standards since v6.3.0:

- **Zero clippy warnings**: 675 warnings eliminated → 0
- **Zero test failures**: 4,706 tests passing (100%)
- **Zero performance issues**: All hot paths optimized
- **Zero defects**: NASA-level quality enforced

## Troubleshooting

### Hook not running
```bash
# Verify hook is installed
ls -la .git/hooks/pre-commit

# Reinstall if missing
./scripts/hooks/install-hooks.sh
```

### Hook execution too slow
- This is intentional - quality takes time
- ~14 seconds ensures production-ready code
- Consider it an investment in zero defects

### Clippy warnings blocking commit
```bash
# See what's wrong
cargo clippy --lib -- -D warnings

# Auto-fix where possible
cargo clippy --lib --fix --allow-dirty

# Commit again
git commit
```

### Tests failing
```bash
# Run tests to see failures
cargo test --lib

# Fix the failing tests
# Then commit again
git commit
```

## Contributing

When contributing to bashrs, these hooks ensure your code meets project standards before it reaches code review. This saves time and maintains code quality.

Thank you for maintaining NASA-level quality standards!
