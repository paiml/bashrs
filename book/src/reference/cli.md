# CLI Commands

This is the reference for all bashrs CLI commands.

## `bashrs bench` - Scientific Benchmarking

Benchmark shell scripts with scientific rigor, measuring execution time and optionally memory usage.

### Usage

```bash
bashrs bench [OPTIONS] <SCRIPT>...
```

### Arguments

- `<SCRIPT>...` - Shell script(s) to benchmark

### Options

- `-w, --warmup <N>` - Number of warmup iterations (default: 3)
- `-i, --iterations <N>` - Number of measured iterations (default: 10)
- `-o, --output <FILE>` - Output results to JSON file
- `-s, --strict` - Enable quality gates (lint + determinism checks)
- `--verify-determinism` - Verify script produces identical output
- `--show-raw` - Show raw iteration times
- `-q, --quiet` - Suppress progress output
- `-m, --measure-memory` - Measure memory usage (requires `/usr/bin/time`)

### Examples

Basic benchmark:
```bash
bashrs bench script.sh
```

With memory measurement:
```bash
bashrs bench script.sh --measure-memory
```

Custom iterations and warmup:
```bash
bashrs bench script.sh --iterations 20 --warmup 5
```

Compare multiple scripts:
```bash
bashrs bench fast.sh slow.sh --measure-memory
```

JSON output for automation:
```bash
bashrs bench script.sh --output results.json --quiet
```

With quality gates:
```bash
bashrs bench script.sh --strict --verify-determinism
```

### Output

The bench command provides:
- **Statistical metrics**: Mean, median, standard deviation, min, max
- **Memory statistics** (with `-m`): Mean, median, min, max, peak RSS in KB
- **Environment metadata**: CPU, RAM, OS, hostname
- **Console display**: Formatted output with results
- **JSON export**: Machine-readable format for automation

### Memory Measurement

When using `--measure-memory` / `-m`, bashrs measures the maximum resident set size (RSS) during script execution using `/usr/bin/time`. This provides accurate memory profiling:

```text
💾 Memory Usage
  Mean:    3456.00 KB
  Median:  3456.00 KB
  Min:     3456.00 KB
  Max:     3456.00 KB
  Peak:    3456.00 KB
```

**Requirements**:
- `/usr/bin/time` must be available (standard on Linux/Unix systems)
- Memory measurement adds negligible overhead (~1-2%)

### Quality Gates

Use `--strict` to run bashrs linter before benchmarking:
- Ensures scripts follow best practices
- Catches common errors before performance testing
- Fails benchmark if lint errors are found

Use `--verify-determinism` to check output consistency:
- Runs script multiple times
- Compares output across runs
- Fails if non-deterministic behavior detected (e.g., $RANDOM, timestamps)

## Other Commands

(Additional CLI commands will be documented here)

