# Makefile Testing

Bashrs provides comprehensive test generation for purified Makefiles, ensuring your build scripts are deterministic, idempotent, and POSIX-compliant.

## Overview

When purifying Makefiles with the `--with-tests` flag, bashrs automatically generates a complete test suite that validates:

- **Determinism**: Same input produces same output every time
- **Idempotency**: Safe to run multiple times without errors
- **POSIX Compliance**: Works with POSIX make implementations

## Basic Usage

### Generate Tests with Purification

```bash
# Purify Makefile and generate test suite
bashrs make purify Makefile --with-tests -o Makefile.purified

# This creates two files:
# - Makefile.purified         # Purified Makefile
# - Makefile.purified.test.sh # Test suite
```

### Run the Generated Tests

```bash
# Make test executable and run
chmod +x Makefile.purified.test.sh
./Makefile.purified.test.sh

# Or run with sh directly
sh Makefile.purified.test.sh
```

## Test Suite Components

The generated test suite includes three core tests:

### 1. Determinism Test

Verifies that running make multiple times with the same input produces identical output:

```bash
test_determinism() {
    # Run make twice
    make -f "Makefile.purified" > /tmp/output1.txt 2>&1
    make -f "Makefile.purified" > /tmp/output2.txt 2>&1

    # Compare outputs (sorted to handle parallel execution)
    if diff <(sort /tmp/output1.txt) <(sort /tmp/output2.txt); then
        echo "✓ Determinism test passed"
    else
        echo "✗ Determinism test failed"
        return 1
    fi
}
```

**What it catches:**
- Timestamps in build outputs
- Random number generation (`$RANDOM`)
- Process IDs (`$$`)
- Unpredictable command ordering

### 2. Idempotency Test

Ensures the Makefile can be run multiple times safely:

```bash
test_idempotency() {
    # Run make three times
    make -f "Makefile.purified" || true
    make -f "Makefile.purified" || exit_code1=$?
    make -f "Makefile.purified" || exit_code2=$?

    # Second and third runs should succeed
    if [ "${exit_code1:-0}" -eq 0 ] && [ "${exit_code2:-0}" -eq 0 ]; then
        echo "✓ Idempotency test passed"
    else
        echo "✗ Idempotency test failed"
        return 1
    fi
}
```

**What it catches:**
- Missing `-p` flag on `mkdir` (fails on second run)
- Missing `-f` flag on `cp` (fails if file exists)
- Non-idempotent operations that break on re-run

### 3. POSIX Compliance Test

Verifies the Makefile works with POSIX make implementations:

```bash
test_posix_compliance() {
    # Test with different make implementations
    for make_cmd in make pmake bmake; do
        if command -v "$make_cmd" > /dev/null 2>&1; then
            echo "Testing with $make_cmd..."
            if "$make_cmd" -f "Makefile.purified" > /dev/null 2>&1; then
                echo "✓ Works with $make_cmd"
            else
                echo "✗ Failed with $make_cmd"
                return 1
            fi
        fi
    done
}
```

**What it catches:**
- GNU make-specific extensions
- Non-portable shell constructs
- Platform-specific commands

## Example Workflow

### Starting with a Non-Deterministic Makefile

```makefile
# Original Makefile with issues
.PHONY: build

build:
    mkdir build
    echo "Build time: $(shell date)" > build/timestamp.txt
    echo "Build ID: $$RANDOM" >> build/timestamp.txt
    gcc -o build/myapp src/main.c
```

**Problems:**
- `mkdir build` - Not idempotent (fails on second run)
- `$(shell date)` - Non-deterministic timestamp
- `$$RANDOM` - Non-deterministic random number

### Purify with Test Generation

```bash
bashrs make purify Makefile --with-tests -o Makefile.purified
```

### Purified Output

```makefile
# Purified Makefile
.PHONY: build

build:
    mkdir -p build
    echo "Build info" > build/timestamp.txt
    gcc -o build/myapp src/main.c
```

**Improvements:**
- ✅ `mkdir -p` - Idempotent (safe to re-run)
- ✅ Removed `$(shell date)` - Deterministic
- ✅ Removed `$$RANDOM` - Deterministic

### Run Tests

```bash
sh Makefile.purified.test.sh
```

**Output:**
```text
Testing determinism for Makefile.purified...
✓ Determinism test passed

Testing idempotency for Makefile.purified...
✓ Idempotency test passed

Testing POSIX compliance for Makefile.purified...
Testing with make...
✓ Works with make

============================================
Test Results Summary
============================================
Total Tests: 3
Passed: 3
Failed: 0

Status: ✓ All tests passed!
```

## Advanced Testing Features

### Property-Based Testing (Future)

```bash
# Generate property-based tests (planned)
bashrs make purify Makefile --with-tests --property-tests -o Makefile.purified
```

This will generate tests with:
- Randomized input variations
- Edge case exploration
- Fuzz testing for robustness

### Custom Test Assertions

You can extend the generated test suite with custom assertions:

```bash
# Add custom test after generation
cat >> Makefile.purified.test.sh <<'EOF'

# Custom test: Check build artifacts
test_artifacts() {
    make -f "Makefile.purified" build

    if [ -f build/myapp ]; then
        echo "✓ Build artifact exists"
    else
        echo "✗ Build artifact missing"
        return 1
    fi
}
EOF
```

## Integration with CI/CD

### GitHub Actions Example

```yaml
name: Makefile Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install bashrs
        run: cargo install bashrs

      - name: Purify Makefile with tests
        run: bashrs make purify Makefile --with-tests -o Makefile.purified

      - name: Run test suite
        run: sh Makefile.purified.test.sh

      - name: Verify purified Makefile works
        run: make -f Makefile.purified all
```

### GitLab CI Example

```yaml
makefile-tests:
  stage: test
  script:
    - cargo install bashrs
    - bashrs make purify Makefile --with-tests -o Makefile.purified
    - sh Makefile.purified.test.sh
    - make -f Makefile.purified all
```

## Troubleshooting

### Test Failures

#### Determinism Test Fails

**Symptom:** Outputs differ between runs

**Common Causes:**
- Timestamps in output
- Random numbers
- Process IDs
- System-dependent paths

**Solution:**
```bash
# Check what's different
bashrs make purify Makefile --with-tests -o Makefile.purified
sh Makefile.purified.test.sh

# Review the diff output to identify non-deterministic sources
```

#### Idempotency Test Fails

**Symptom:** Second or third run fails

**Common Causes:**
- `mkdir` without `-p`
- `cp` without `-f`
- Operations that fail on existing files

**Solution:**
```bash
# Bashrs should auto-fix these, but verify:
grep -E 'mkdir[^-]' Makefile.purified  # Should have -p flag
grep -E 'cp[^-]' Makefile.purified     # Should have -f flag
```

#### POSIX Compliance Test Fails

**Symptom:** Works with GNU make but fails with other implementations

**Common Causes:**
- GNU-specific extensions (`:=`, `+=`, etc.)
- Bash-specific syntax in recipes
- Non-portable commands

**Solution:**
```bash
# Use POSIX-only features
# bashrs should warn about non-portable constructs
bashrs make lint Makefile
```

## Best Practices

### 1. Always Generate Tests

```bash
# Good: Generate tests with every purification
bashrs make purify Makefile --with-tests -o Makefile.purified

# Bad: Purify without tests
bashrs make purify Makefile -o Makefile.purified  # No tests!
```

### 2. Run Tests Before Deployment

```bash
# Ensure tests pass before using purified Makefile
bashrs make purify Makefile --with-tests -o Makefile.purified
sh Makefile.purified.test.sh || exit 1
cp Makefile.purified Makefile
```

### 3. Version Control Test Suites

```bash
# Keep test suites in version control
git add Makefile.purified Makefile.purified.test.sh
git commit -m "Add purified Makefile with test suite"
```

### 4. Test on Multiple Platforms

```bash
# Run tests on different OSes and make implementations
# - Linux (GNU make)
# - macOS (BSD make)
# - FreeBSD (BSD make)
```

## Example: Complete Workflow

Try the complete workflow with the provided example:

```bash
# Run the comprehensive example
cargo run --example makefile_purify_with_tests

# This demonstrates:
# - Creating a problematic Makefile
# - Running purification with test generation
# - Showing purified output
# - Displaying generated tests
# - Validating syntax
# - Explaining improvements
```

The example provides a 10-step walkthrough of the entire process, showing exactly what bashrs does and how the tests work.

## See Also

- [Makefile Overview](./overview.md) - Basic Makefile purification
- [Makefile Security](./security.md) - Security-focused purification
- [Makefile Best Practices](./best-practices.md) - Writing better Makefiles
- [Property Testing](../advanced/property-testing.md) - Advanced testing techniques
