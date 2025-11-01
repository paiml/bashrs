# bashrs bench - Scientific Benchmark CLI Command

**Version**: 1.0.0
**Status**: DRAFT
**Priority**: P0
**Quality Target**: NASA-level reproducibility and scientific rigor

## Executive Summary

Add a `bashrs bench` CLI sub-command to enable deterministic, scientifically rigorous performance benchmarking of shell scripts with statistical analysis, environment capture, and full reproducibility.

## Motivation

Current challenges:
1. **No deterministic benchmarking** - Manual timing is error-prone
2. **No statistical analysis** - Single runs don't capture variance
3. **No reproducibility** - Environment not captured
4. **No quality gates** - Benchmarks may have non-deterministic code

bashrs is uniquely positioned to solve this:
- Lints for non-deterministic patterns ($RANDOM, timestamps)
- Enforces deterministic practices
- Provides determinism scoring
- Can validate benchmark quality before running

## Requirements

### Functional Requirements

#### FR-001: Basic Benchmark Execution
```bash
bashrs bench script.sh
```
- Run script with warmup + measured iterations
- Output statistical analysis
- Capture environment metadata
- Default: 3 warmup, 10 measured iterations

#### FR-002: Configurable Iterations
```bash
bashrs bench script.sh --warmup 5 --iterations 20
```
- Allow custom warmup count
- Allow custom measured iteration count
- Validate: warmup ≥ 0, iterations ≥ 1

#### FR-003: Multiple Script Comparison
```bash
bashrs bench script1.sh script2.sh script3.sh
```
- Benchmark multiple scripts
- Output comparative results
- Calculate speedup ratios
- Identify fastest/slowest

#### FR-004: JSON Output
```bash
bashrs bench script.sh --output results.json
```
- Machine-readable JSON format
- Include all statistics
- Include environment metadata
- Include raw iteration results

#### FR-005: Quality Gates (bashrs Integration)
```bash
bashrs bench script.sh --strict
```
- Pre-flight: Run `bashrs lint` on script
- Fail if linting errors found
- Fail if determinism score < 0.9
- Ensure benchmark script is deterministic

#### FR-006: Determinism Verification
```bash
bashrs bench script.sh --verify-determinism
```
- Run script multiple times
- Verify identical output
- Fail if output differs between runs
- Report which runs produced different output

### Non-Functional Requirements

#### NFR-001: Scientific Rigor
- **Statistical analysis**: mean, median, stddev, min, max
- **Reproducibility**: Fixed environment capture
- **Transparency**: Report all raw results
- **Validation**: Pre-flight quality checks

#### NFR-002: Performance
- Warmup overhead < 100ms
- Result aggregation < 10ms
- JSON serialization < 50ms
- Total overhead < 1% of benchmark time

#### NFR-003: Usability
- Clear progress indicators
- Informative error messages
- Sensible defaults
- Optional verbose output

## Command-Line Interface

### Usage

```
bashrs bench [OPTIONS] <SCRIPT>...

ARGUMENTS:
  <SCRIPT>...   Shell script(s) to benchmark

OPTIONS:
  -w, --warmup <N>          Number of warmup iterations [default: 3]
  -i, --iterations <N>      Number of measured iterations [default: 10]
  -o, --output <FILE>       Output results to JSON file
  -s, --strict              Enable quality gates (lint + determinism checks)
  -v, --verify-determinism  Verify script produces identical output
  --show-raw                Show raw iteration times
  --quiet                   Suppress progress output
  -h, --help                Print help
```

### Examples

**Basic benchmark**:
```bash
bashrs bench fib.sh
```

**Scientific benchmark (strict quality)**:
```bash
bashrs bench fib.sh --strict --iterations 20 --verify-determinism
```

**Compare multiple implementations**:
```bash
bashrs bench fib-recursive.sh fib-iterative.sh fib-memoized.sh --output comparison.json
```

**Custom iterations**:
```bash
bashrs bench expensive-script.sh --warmup 5 --iterations 50
```

## Output Format

### Console Output (Default)

```
📊 Benchmarking: fib.sh
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

🔥 Warmup (3 iterations)...
  ✓ Iteration 1: 125ms
  ✓ Iteration 2: 118ms
  ✓ Iteration 3: 120ms

⏱️  Measuring (10 iterations)...
  ✓ Iteration 1: 122ms
  ✓ Iteration 2: 119ms
  ✓ Iteration 3: 121ms
  ✓ Iteration 4: 120ms
  ✓ Iteration 5: 123ms
  ✓ Iteration 6: 118ms
  ✓ Iteration 7: 122ms
  ✓ Iteration 8: 120ms
  ✓ Iteration 9: 121ms
  ✓ Iteration 10: 119ms

📈 Results for fib.sh
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Mean:    120.50ms ± 1.58ms
  Median:  120.50ms
  Min:     118.00ms
  Max:     123.00ms
  StdDev:  1.58ms
  Runs:    10

🖥️  Environment
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  CPU:     AMD Ryzen 9 5950X 16-Core
  RAM:     64GB
  OS:      Linux 6.8.0-85-generic
  Date:    2025-11-01T12:30:45-07:00
```

### JSON Output (--output)

```json
{
  "version": "1.0.0",
  "timestamp": "2025-11-01T12:30:45-07:00",
  "environment": {
    "cpu": "AMD Ryzen 9 5950X 16-Core",
    "ram": "64GB",
    "os": "Linux 6.8.0-85-generic",
    "hostname": "workstation",
    "bashrs_version": "6.24.3"
  },
  "benchmarks": [
    {
      "script": "fib.sh",
      "iterations": 10,
      "warmup": 3,
      "statistics": {
        "mean_ms": 120.50,
        "median_ms": 120.50,
        "stddev_ms": 1.58,
        "min_ms": 118.00,
        "max_ms": 123.00,
        "variance_ms": 2.49
      },
      "raw_results_ms": [122, 119, 121, 120, 123, 118, 122, 120, 121, 119],
      "quality": {
        "lint_passed": true,
        "determinism_score": 1.0,
        "output_identical": true
      }
    }
  ]
}
```

### Comparison Output (Multiple Scripts)

```
📊 Comparison Results
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Script                Mean (ms)   StdDev (ms)  Speedup
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
fib-memoized.sh       45.20       ± 1.20       2.67x 🏆
fib-iterative.sh      89.30       ± 2.10       1.35x
fib-recursive.sh      120.50      ± 1.58       1.00x (baseline)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

🏆 Winner: fib-memoized.sh (2.67x faster than baseline)
```

## Quality Gates

### Gate 1: Lint Check (--strict)

Before benchmarking, run:
```bash
bashrs lint <script>
```

**Pass criteria**:
- Zero linting errors
- All warnings at "info" level or lower
- No critical security issues

**Failure behavior**:
```
❌ Quality gate failed: Lint check
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

fib.sh:15:10 [error] DET001: Non-deterministic pattern detected
  Found: $RANDOM
  Fix: Use deterministic seed or remove

Run 'bashrs lint fib.sh' for details.
```

### Gate 2: Determinism Score (--strict)

**Pass criteria**:
- Determinism score ≥ 0.9

**Failure behavior**:
```
❌ Quality gate failed: Determinism score
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Determinism score: 0.75 (threshold: 0.90)

Non-deterministic patterns found:
  - Line 15: $RANDOM usage
  - Line 28: $(date +%s) timestamp
  - Line 42: Unquoted variable expansion

Run 'bashrs audit fib.sh' for details.
```

### Gate 3: Output Determinism (--verify-determinism)

**Pass criteria**:
- All runs produce byte-identical output

**Failure behavior**:
```
❌ Determinism verification failed
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Output differs between runs:
  Run 1: 832040 (output hash)
  Run 2: 751293 (output hash)
  Run 3: 832040 (output hash)

This indicates non-deterministic behavior in the script.
Common causes:
  - $RANDOM usage
  - Timestamp generation
  - Uninitialized variables
  - Race conditions
```

## Implementation Plan (EXTREME TDD)

### Phase 1: RED - Write Failing Tests

**Test Module**: `rash/tests/cli_bench_tests.rs`

```rust
// Test structure (15 tests minimum)

// Basic functionality
#[test] fn test_bench_basic_execution()
#[test] fn test_bench_custom_iterations()
#[test] fn test_bench_custom_warmup()
#[test] fn test_bench_multiple_scripts()

// Output formats
#[test] fn test_bench_console_output()
#[test] fn test_bench_json_output()
#[test] fn test_bench_comparison_output()

// Quality gates
#[test] fn test_bench_strict_mode_lint_fail()
#[test] fn test_bench_strict_mode_lint_pass()
#[test] fn test_bench_determinism_verification_pass()
#[test] fn test_bench_determinism_verification_fail()

// Statistics
#[test] fn test_bench_statistics_accuracy()
#[test] fn test_bench_environment_capture()

// Error handling
#[test] fn test_bench_nonexistent_script()
#[test] fn test_bench_invalid_iterations()
```

### Phase 2: GREEN - Minimal Implementation

**Code Module**: `rash/src/cli/bench.rs`

```rust
pub struct BenchOptions {
    pub scripts: Vec<PathBuf>,
    pub warmup: usize,
    pub iterations: usize,
    pub output: Option<PathBuf>,
    pub strict: bool,
    pub verify_determinism: bool,
    pub show_raw: bool,
    pub quiet: bool,
}

pub fn bench_command(options: BenchOptions) -> Result<()> {
    // 1. Quality gates (if --strict)
    // 2. Warmup runs
    // 3. Measured runs
    // 4. Statistical analysis
    // 5. Output formatting
}
```

### Phase 3: REFACTOR - Quality & Performance

- Extract helper functions (complexity <10)
- Add comprehensive unit tests
- Verify mutation test score >90%
- Run clippy, fmt
- Benchmark bench command itself (<100ms overhead)

## Test Coverage Requirements

- **Unit tests**: >85% code coverage
- **Integration tests**: All CLI flags tested
- **Property tests**: Statistical functions verified
- **Mutation tests**: >90% kill rate

## Dependencies

**New crates needed**:
```toml
[dependencies]
sysinfo = "0.30"  # For CPU/RAM detection
chrono = "0.4"     # For ISO 8601 timestamps
```

**Existing crates used**:
- `clap` (CLI parsing)
- `serde_json` (JSON output)
- `std::time` (timing)

## Acceptance Criteria

- [ ] All 15+ tests pass (RED → GREEN → REFACTOR)
- [ ] `bashrs bench fib.sh` works end-to-end
- [ ] `bashrs bench --strict` enforces quality gates
- [ ] `bashrs bench --verify-determinism` detects non-determinism
- [ ] JSON output matches specification
- [ ] Comparison mode works for multiple scripts
- [ ] Environment metadata captured correctly
- [ ] Statistical analysis accurate (verified with property tests)
- [ ] Overhead < 100ms for warmup + analysis
- [ ] All quality gates pass (clippy, fmt, mutation tests)

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Timing precision on different platforms | Medium | Use platform-specific high-resolution timers |
| Non-deterministic warmup effects | Low | Discard warmup results, only measure iterations |
| Large script output affecting timing | Medium | Redirect output to /dev/null during timing |
| Environment detection failure | Low | Gracefully handle missing environment info |

## Future Enhancements (v2.0+)

- [ ] Support for comparative benchmarking (before/after)
- [ ] Historical trend analysis
- [ ] CI/CD integration (exit code based on performance regression)
- [ ] GPU metrics capture
- [ ] Network I/O benchmarking
- [ ] Memory profiling integration
- [ ] Flame graph generation

## References

- Ruchy Benchmarking Roadmap: `/home/noah/src/ruchy-book/docs/execution/roadmap-ch23-benchmarking.md`
- Benchmark Framework Script: `/home/noah/src/ruchy-book/test/ch23-benchmarks/scripts/benchmark-framework.sh`
- EXTREME TDD Methodology: `CLAUDE.md`
