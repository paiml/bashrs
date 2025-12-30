# Container-Based Testing

The installer framework supports testing across multiple platforms using containers.

## Test Matrix

Run your installer across multiple platforms:

```bash
bashrs installer test my-installer --matrix "ubuntu:22.04,debian:12,fedora:39,alpine:3.19"
```

## Configuring Test Matrix

Define the test matrix in `installer.toml`:

```toml
[installer]
name = "my-app"
version = "1.0.0"

[installer.test_matrix]
platforms = [
    "ubuntu:22.04",
    "ubuntu:20.04",
    "debian:12",
    "debian:11",
    "fedora:39",
    "alpine:3.19"
]
architectures = ["amd64", "arm64"]
parallelism = 4  # Run 4 containers in parallel
runtime = "docker"  # or "podman"
```

## Running Tests

```bash
# Run full test matrix
bashrs installer test my-installer

# Run specific platforms
bashrs installer test my-installer --matrix "ubuntu:22.04,debian:12"

# Enable coverage reporting
bashrs installer test my-installer --coverage
```

## Test Output

```text
Container Test Matrix
══════════════════════════════════════════════════════════════════════════════
  Platform          Arch    Status    Duration    Notes
──────────────────────────────────────────────────────────────────────────────
  ubuntu:22.04      amd64   ✓ PASS    12.3s
  ubuntu:22.04      arm64   ✓ PASS    14.1s
  debian:12         amd64   ✓ PASS    11.8s
  debian:12         arm64   ✓ PASS    13.2s
  fedora:39         amd64   ✓ PASS    15.4s
  alpine:3.19       amd64   ✗ FAIL    8.2s        Missing libc
══════════════════════════════════════════════════════════════════════════════
  Results: 5/6 passed (83.3%)
```

## TDD-First Testing

The generated test harness includes falsification tests:

```rust
// tests/falsification.rs

/// FALSIFIABLE: "Every step is idempotent"
/// DISPROOF: Run step twice, system state differs
#[test]
fn falsify_step_idempotency() {
    let step = load_step("install-app");
    let state_after_first = execute_and_capture_state(&step);
    let state_after_second = execute_and_capture_state(&step);
    assert_eq!(state_after_first, state_after_second,
        "FALSIFIED: Step is not idempotent");
}

/// FALSIFIABLE: "Dry-run accurately predicts changes"
#[test]
fn falsify_dry_run_accuracy() {
    let predicted = execute_dry_run(&installer);
    let actual = execute_and_capture_diff(&installer);
    assert_eq!(predicted, actual,
        "FALSIFIED: Dry-run prediction was inaccurate");
}
```

## Custom Test Scripts

Add custom test scripts:

```toml
[[step]]
id = "install-app"
name = "Install Application"
action = "script"

[step.verification]
commands = [
    { cmd = "my-app --version", expect = "1.0" },
    { cmd = "my-app self-test" },
    { cmd = "test -f ~/.config/my-app/config.toml" }
]

[step.script]
content = '''
# Installation commands
'''
```

## Integration with CI/CD

### GitHub Actions

```yaml
name: Test Installer

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        platform: [ubuntu:22.04, debian:12, fedora:39]

    steps:
      - uses: actions/checkout@v4

      - name: Install bashrs
        run: cargo install bashrs

      - name: Test Installer
        run: |
          bashrs installer test my-installer \
            --matrix "${{ matrix.platform }}"
```

### GitLab CI

```yaml
test-installer:
  image: rust:latest
  script:
    - cargo install bashrs
    - bashrs installer test my-installer --matrix "ubuntu:22.04,debian:12"
```

## Coverage Reporting

Generate test coverage reports:

```bash
bashrs installer test my-installer --coverage
```

Output:
```text
Coverage Report
══════════════════════════════════════════════════════════════════════════════
  Step                    Executed    Verified    Coverage
──────────────────────────────────────────────────────────────────────────────
  create-dirs             ✓           ✓           100%
  download-app            ✓           ✓           100%
  install-app             ✓           ✓           100%
  configure               ✓           ✗           50%
  verify                  ✓           ✓           100%
══════════════════════════════════════════════════════════════════════════════
  Total Coverage: 90%
```

## Debugging Failed Tests

When a test fails:

```bash
# Run with verbose output
bashrs installer test my-installer --matrix "alpine:3.19" --verbose

# Keep container running for inspection
bashrs installer test my-installer --matrix "alpine:3.19" --keep-container

# Attach to failed container
docker exec -it bashrs-test-alpine-3.19 /bin/sh
```

## Best Practices

1. **Test all target platforms**: Include every platform in your requirements
2. **Test both architectures**: If supporting arm64, test it
3. **Use realistic base images**: Don't use custom images with pre-installed deps
4. **Test idempotency**: Run installer twice, verify same result
5. **Test rollback**: Verify rollback actually works
6. **Run tests in CI**: Catch regressions early

## Next Steps

- [Hermetic Builds](./hermetic.md) - Reproducible installations
- [Artifacts](./artifacts.md) - Download and verify files
- [CLI Reference](./cli-reference.md) - All command options
