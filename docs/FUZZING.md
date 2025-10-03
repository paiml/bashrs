# Rash Fuzzing Infrastructure

## Overview

Rash uses a multi-layered fuzzing strategy to discover edge cases, panics, and injection vulnerabilities:

1. **Property-Based Fuzzing** (Primary): 60 proptest properties with 100K+ generated test cases
2. **Cargo-Fuzz** (Advanced): Coverage-guided fuzzing with sanitizers (requires system dependencies)

## Quick Start: Property-Based Fuzzing

Property-based fuzzing is production-ready and runs in CI:

```bash
# Run with default 256 cases per property (15,360 total cases)
cargo test prop_ --lib

# Run extensive fuzzing campaign (120,000 cases)
env PROPTEST_CASES=2000 cargo test prop_ --lib

# Run specific fuzzing targets
env PROPTEST_CASES=5000 cargo test testing::quickcheck_tests::fuzz --lib
```

### Current Property Tests (60 total)

**Emitter Properties** (8):
- `prop_commands_emit_valid_shell` - Command generation validity
- `prop_concatenation_preserves_order` - String concatenation correctness
- `prop_emission_deterministic` - Deterministic output
- `prop_exit_codes_valid` - Exit code range validation
- `prop_if_statements_balanced` - Control flow balance
- `prop_let_statements_valid` - Variable declaration validity
- `prop_shell_values_emit_valid_code` - Value emission correctness
- `prop_special_chars_escaped` - Injection prevention

**Formal Properties** (4):
- `prop_assignment_preserves_env` - Variable assignment semantics
- `prop_echo_preserves_output` - Output preservation
- `prop_emitter_produces_valid_posix` - POSIX compliance
- `prop_semantic_equivalence` - Rust/Shell equivalence

**Fuzz Integration** (2):
- `prop_graceful_failure_on_invalid_input` - Error handling robustness
- `prop_no_panics_on_valid_input` - Panic freedom

**Playground Properties** (5):
- `prop_computation_graph_no_cycles` - DAG invariant
- `prop_highlighting_cache_consistency` - Cache correctness
- `prop_session_state_roundtrip` - Serialization safety
- `prop_syntax_highlighting_deterministic` - Determinism

**And 41 more** across IR generation, parsing, validation, and transformation.

## Advanced: Cargo-Fuzz

cargo-fuzz provides coverage-guided fuzzing with AddressSanitizer for deep bug discovery.

### System Requirements

```bash
# Install dependencies (Ubuntu/Debian)
sudo apt-get install clang libstdc++-dev

# Install cargo-fuzz
cargo install cargo-fuzz
```

### Fuzz Targets

#### 1. Transpile Robustness

Tests that transpilation never panics on any input:

```bash
# List targets
cargo fuzz list

# Run transpile_robustness
cargo fuzz run transpile_robustness -- -max_total_time=3600

# With corpus
cargo fuzz run transpile_robustness \
  --corpus=fuzz/corpus/transpile_robustness \
  -- -dict=fuzz/rust_keywords.dict -max_len=8192
```

**Target**: `fuzz/fuzz_targets/transpile_robustness.rs`
```rust
fuzz_target!(|data: &[u8]| {
    if let Ok(source) = std::str::from_utf8(data) {
        let _ = bashrs::transpile(source, bashrs::Config::default());
    }
});
```

#### 2. Injection Detection

Validates shell output for injection safety:

```bash
cargo fuzz run injection_detection -- -max_total_time=3600
```

**Target**: `fuzz/fuzz_targets/injection_detection.rs`
- Checks for eval usage
- Validates variable quoting
- Detects command injection patterns
- Verifies backtick escaping

### Fuzzing Corpus

Seed corpus located in `fuzz/corpus/`:

**transpile_robustness/**:
- `001_simple_function.rs` - Basic function
- `002_string_ops.rs` - String operations
- `003_control_flow.rs` - If/for loops
- `004_while_loop.rs` - While loops
- `005_match_expr.rs` - Match expressions

**injection_detection/**:
- All transpile_robustness seeds
- `006_special_chars.rs` - Shell metacharacters
- `007_backticks.rs` - Command substitution

### Fuzzing Dictionary

`fuzz/rust_keywords.dict` contains Rust-specific tokens to guide fuzzing:
- Keywords: fn, let, mut, if, else, for, while, match, return
- Macros: println!, format!
- Types: Vec, String, Option, Result, i32, u32, bool
- Operators: ==, !=, <, >, <=, >=, &&, ||, +, -, *, /

## Continuous Fuzzing

### GitHub Actions Workflow

Create `.github/workflows/continuous-fuzz.yml`:

```yaml
name: Continuous Fuzzing

on:
  schedule:
    - cron: '0 */6 * * *'  # Every 6 hours
  workflow_dispatch:

jobs:
  proptest-fuzz:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable

      - name: Run property-based fuzzing
        run: |
          env PROPTEST_CASES=5000 cargo test prop_ --lib
        timeout-minutes: 30

      - name: Upload artifacts on failure
        if: failure()
        uses: actions/upload-artifact@v3
        with:
          name: proptest-failures
          path: proptest-regressions/

  cargo-fuzz:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        target: [transpile_robustness, injection_detection]

    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@nightly

      - name: Install dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y clang libstdc++-dev

      - name: Install cargo-fuzz
        run: cargo install cargo-fuzz

      - name: Run fuzzing
        run: |
          cargo fuzz run ${{ matrix.target }} -- \
            -max_total_time=1800 \
            -dict=fuzz/rust_keywords.dict
        timeout-minutes: 35

      - name: Upload crashes
        if: failure()
        uses: actions/upload-artifact@v3
        with:
          name: fuzz-crashes-${{ matrix.target }}
          path: fuzz/artifacts/${{ matrix.target }}/
```

### Local Development

```bash
# Quick smoke test (1 minute)
env PROPTEST_CASES=500 cargo test prop_ --lib

# Daily fuzzing (10 minutes)
env PROPTEST_CASES=10000 cargo test prop_ --lib

# Weekend campaign (hours)
cargo fuzz run transpile_robustness -- -max_total_time=14400  # 4 hours
```

## Analyzing Results

### Property Test Failures

Failures are saved in `proptest-regressions/`:

```bash
# View regression
cat proptest-regressions/testing/quickcheck_tests.txt

# Reproduce
cargo test prop_no_panics_on_valid_input -- --exact
```

### Cargo-Fuzz Crashes

Crashes saved in `fuzz/artifacts/`:

```bash
# List crashes
ls fuzz/artifacts/transpile_robustness/

# Reproduce crash
cargo fuzz run transpile_robustness fuzz/artifacts/transpile_robustness/crash-abc123

# Minimize crash case
cargo fuzz cmin transpile_robustness
```

## Quality Metrics (Sprint 34)

**Achieved**:
- ✅ 60 property tests (up from 52)
- ✅ 120,000+ test cases executed (60 properties × 2,000 cases)
- ✅ Zero panics discovered
- ✅ Injection safety validated
- ✅ Fuzzing infrastructure ready for CI

**Testing Spec Compliance**:
- ✅ Section 1.5: Fuzzing infrastructure established
- ✅ Coverage-guided fuzzing setup (cargo-fuzz)
- ✅ Corpus-based fuzzing with real Rust code
- ✅ Injection detection validation
- ✅ 100K+ executions target exceeded (120K achieved)

## Troubleshooting

### cargo-fuzz build fails

**Issue**: `cannot find -lstdc++`

**Solution**:
```bash
sudo apt-get install libstdc++-dev clang
```

### Property tests timeout

**Solution**: Reduce PROPTEST_CASES or run specific targets:
```bash
env PROPTEST_CASES=500 cargo test prop_no_panics --lib
```

### High memory usage

**Solution**: Limit parallelism:
```bash
cargo test prop_ --lib -- --test-threads=2
```

## Future Enhancements

1. **Differential Fuzzing**: Compare Rust execution with shell output
2. **Structured Fuzzing**: Generate valid Rust ASTs with arbitrary crate
3. **Multi-Shell Fuzzing**: Test on dash, ash, busybox simultaneously
4. **AFL Integration**: Combine cargo-fuzz with AFL for hybrid fuzzing
5. **Crash Deduplication**: Automatic crash bucketing by root cause

## References

- [cargo-fuzz Book](https://rust-fuzz.github.io/book/cargo-fuzz.html)
- [proptest Guide](https://proptest-rs.github.io/proptest/)
- [Testing Spec v1.2](specifications/rash-testing-spec-v1.2-final.md#15-layer-5-fuzzing---automated-edge-case-discovery)
- [libFuzzer](https://llvm.org/docs/LibFuzzer.html)
