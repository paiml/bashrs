# Probar Testing Integration

Rash v6.46.0 introduces three new CLI commands for advanced shell script testing, integrating with the Probar testing methodology.

## Overview

| Command | Purpose | Use Case |
|---------|---------|----------|
| `bashrs playbook` | State machine testing | Complex multi-state workflows |
| `bashrs mutate` | Mutation testing | Verify test quality |
| `bashrs simulate` | Deterministic replay | Debug non-deterministic scripts |

## `bashrs playbook` - State Machine Testing

Execute playbook-driven state machine tests with YAML-based test definitions.

### Usage

```bash
bashrs playbook <FILE> [OPTIONS]
```

### Arguments

- `<FILE>` - Playbook YAML file

### Options

- `--run` - Execute the playbook (default: validate only)
- `--format <FORMAT>` - Output format: `human`, `json`, `junit` (default: `human`)
- `--verbose` - Show detailed execution trace
- `--dry-run` - Parse and validate without executing

### Playbook Format

```yaml
# install.playbook.yaml
version: "1.0"
machine:
  id: "installer"
  initial: "uninstalled"

states:
  uninstalled:
    description: "Application not installed"
    transitions:
      - event: "install"
        target: "installed"
        action: "./install.sh"

  installed:
    description: "Application installed"
    transitions:
      - event: "uninstall"
        target: "uninstalled"
        action: "./uninstall.sh"
      - event: "upgrade"
        target: "installed"
        action: "./upgrade.sh"

tests:
  - name: "install_flow"
    steps:
      - trigger: "install"
        expect_state: "installed"
      - trigger: "upgrade"
        expect_state: "installed"
      - trigger: "uninstall"
        expect_state: "uninstalled"
```

### Examples

Validate playbook structure:
```bash
bashrs playbook install.playbook.yaml
```

Execute playbook tests:
```bash
bashrs playbook install.playbook.yaml --run
```

JUnit output for CI:
```bash
bashrs playbook install.playbook.yaml --run --format junit > results.xml
```

Verbose debugging:
```bash
bashrs playbook install.playbook.yaml --run --verbose
```

### Output

```text
╔══════════════════════════════════════════════════════════════╗
║                    PLAYBOOK VALIDATION                       ║
╠══════════════════════════════════════════════════════════════╣
║  Version: 1.0                                                ║
║  Machine: installer                                          ║
║  States: 2                                                   ║
║  Transitions: 3                                              ║
║  Tests: 1                                                    ║
╚══════════════════════════════════════════════════════════════╝

✓ Playbook validated successfully
```

---

## `bashrs mutate` - Mutation Testing

Mutation testing verifies test quality by introducing defects (mutants) and checking if tests catch them.

### Usage

```bash
bashrs mutate <FILE> [OPTIONS]
```

### Arguments

- `<FILE>` - Shell script to mutate

### Options

- `--config <FILE>` - Mutation configuration file
- `--format <FORMAT>` - Output format: `human`, `json`, `junit` (default: `human`)
- `--count <N>` - Number of mutants to generate (default: 10)
- `--show-survivors` - Display mutants that survived (tests didn't catch)
- `-o, --output <FILE>` - Output file for report

### Mutation Operators

Rash applies 10 mutation operators:

| Operator | Description | Example |
|----------|-------------|---------|
| String Empty | Replace string with empty | `"hello"` → `""` |
| String Quote | Change quote style | `'hello'` → `"hello"` |
| Escape Remove | Remove escape sequences | `"\n"` → `"n"` |
| Command Replace | Replace command | `cat` → `echo` |
| Conditional Invert | Invert conditions | `if [ -f` → `if [ ! -f` |
| Redirect Flip | Flip redirect direction | `>` → `>>` |
| Variable Empty | Empty variable value | `$VAR` → `""` |
| Exit Code | Change exit codes | `exit 0` → `exit 1` |
| Operator Swap | Swap operators | `-eq` → `-ne` |
| Remove Statement | Delete statements | `echo "x"` → (removed) |

### Examples

Basic mutation testing:
```bash
bashrs mutate script.sh
```

Generate more mutants:
```bash
bashrs mutate script.sh --count 20
```

Show surviving mutants:
```bash
bashrs mutate script.sh --show-survivors
```

JSON report:
```bash
bashrs mutate script.sh --format json -o mutations.json
```

### Output

```text
╔══════════════════════════════════════════════════════════════╗
║                     MUTATION TESTING                         ║
╠══════════════════════════════════════════════════════════════╣
║  Script: script.sh                                           ║
║  Mutants Generated: 10                                       ║
╚══════════════════════════════════════════════════════════════╝

Mutations Applied:
  1. [StringEmpty] Line 5: "hello" → ""
  2. [ConditionalInvert] Line 8: if [ -f → if [ ! -f
  3. [ExitCode] Line 12: exit 0 → exit 1
  ...

╔══════════════════════════════════════════════════════════════╗
║  Kill Rate: 90.0% (9/10 mutants killed)                      ║
║  Surviving Mutants: 1                                        ║
╚══════════════════════════════════════════════════════════════╝
```

### Interpreting Results

- **Kill Rate > 90%**: Excellent test quality
- **Kill Rate 70-90%**: Good, but room for improvement
- **Kill Rate < 70%**: Tests need strengthening

Surviving mutants indicate gaps in your test coverage.

---

## `bashrs simulate` - Deterministic Simulation

Replay script execution with deterministic seeding for reproducible debugging.

### Usage

```bash
bashrs simulate <FILE> [OPTIONS]
```

### Arguments

- `<FILE>` - Shell script to simulate

### Options

- `--seed <N>` - Random seed for determinism (default: 42)
- `--verify` - Verify deterministic behavior
- `--mock-externals` - Mock external commands
- `--format <FORMAT>` - Output format: `human`, `json` (default: `human`)
- `--trace` - Show detailed execution trace

### What It Does

1. **Detects Non-Deterministic Patterns**: Finds `$RANDOM`, `$$`, `date`, etc.
2. **Seeds Randomness**: Provides consistent values for reproducible runs
3. **Mocks Externals**: Optionally mock network, filesystem, time
4. **Verifies Consistency**: Runs multiple times to check output matches

### Examples

Basic simulation:
```bash
bashrs simulate script.sh
```

Custom seed:
```bash
bashrs simulate script.sh --seed 12345
```

Verify determinism:
```bash
bashrs simulate script.sh --verify
```

Mock external dependencies:
```bash
bashrs simulate script.sh --mock-externals
```

Full trace:
```bash
bashrs simulate script.sh --trace
```

### Output

```text
╔══════════════════════════════════════════════════════════════╗
║                  DETERMINISTIC SIMULATION                    ║
╠══════════════════════════════════════════════════════════════╣
║  Script: script.sh                                           ║
║  Seed: 42                                                    ║
╚══════════════════════════════════════════════════════════════╝

Non-Deterministic Patterns Detected:
  Line 3: $RANDOM → seeded to 16838
  Line 7: $(date +%s) → mocked to 1703180400
  Line 12: $$ → mocked to 12345

Simulation Mode: Deterministic replay active

✓ Script simulated successfully
```

### Use Cases

1. **Debugging Flaky Tests**: Reproduce intermittent failures
2. **CI Consistency**: Ensure same results across runs
3. **Regression Testing**: Compare behavior across versions
4. **Documentation**: Generate reproducible examples

---

## Best Practices

### Combining All Three Commands

For comprehensive testing:

```bash
# 1. Validate playbook structure
bashrs playbook tests/install.playbook.yaml

# 2. Run state machine tests
bashrs playbook tests/install.playbook.yaml --run --format junit > playbook-results.xml

# 3. Mutation test the script
bashrs mutate install.sh --count 20 --show-survivors

# 4. Verify deterministic behavior
bashrs simulate install.sh --verify --seed 42
```

### CI/CD Integration

```yaml
# .github/workflows/test.yml
jobs:
  test:
    steps:
      - name: Run Playbook Tests
        run: bashrs playbook tests/*.playbook.yaml --run --format junit > playbook.xml

      - name: Mutation Testing
        run: |
          bashrs mutate scripts/*.sh --format json > mutations.json
          # Fail if kill rate < 80%
          jq -e '.kill_rate >= 0.80' mutations.json

      - name: Verify Determinism
        run: bashrs simulate scripts/*.sh --verify
```

### When to Use Each

| Scenario | Command |
|----------|---------|
| Multi-state workflows | `bashrs playbook` |
| Verifying test coverage | `bashrs mutate` |
| Debugging flaky scripts | `bashrs simulate` |
| CI quality gates | All three |

---

## See Also

- [CLI Commands Reference](../reference/cli.md)
- [Property Testing](./property-testing.md)
- [Mutation Testing](./mutation-testing.md)
- [PMAT Integration](./pmat-integration.md)
