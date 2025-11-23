# Golden Trace Integration with Renacer

## Overview

Golden traces capture reference syscall patterns from known-good executions, enabling regression detection by comparing future runs against these baselines. This integration uses [renacer](https://github.com/paiml/renacer), a pure Rust system call tracer with source correlation.

## Philosophy

Following Toyota Way principles:
- **Determinism**: Same input → same syscalls → same trace
- **Regression Prevention**: Any unexpected syscall pattern change triggers review
- **EXTREME TDD**: Golden traces as executable specifications
- **Zero Defects**: Catch regressions before they reach production

## Installation

```bash
# Install renacer
cargo install renacer

# Or use the integration (auto-installs if missing)
make golden-help
```

## Quick Start

### 1. Capture a Golden Trace

```bash
# Capture trace for bashrs --version
make golden-capture TRACE=version CMD='./target/release/bashrs --version'

# Capture trace for bashrs parse
make golden-capture TRACE=parse CMD='./target/release/bashrs parse examples/hello.rs'

# Capture trace for bashrs lint
make golden-capture TRACE=lint CMD='./target/release/bashrs lint examples/unsafe.sh'
```

### 2. Compare Against Golden Trace

```bash
# After making code changes, verify syscall patterns haven't changed unexpectedly
make golden-compare TRACE=version CMD='./target/release/bashrs --version'
```

### 3. List All Golden Traces

```bash
make golden-list
```

Output:
```
=== Golden Traces ===
  bashrs_version                       11665 bytes     665 events
  bashrs_parse_hello                    8234 bytes     423 events
  bashrs_lint_unsafe                   15892 bytes     891 events

Total: 3 traces
```

### 4. Clean Golden Traces

```bash
make golden-clean
```

## Use Cases

### 1. Regression Detection

Golden traces detect when code changes introduce unexpected behavior:

- **New file accesses**: Did your "refactor" start reading config files it shouldn't?
- **New network calls**: Did a dependency update add telemetry?
- **Changed syscall counts**: Did optimization actually reduce file I/O?
- **Security regressions**: Did a change open unexpected files or sockets?

### 2. Performance Baselining

Compare syscall counts before and after optimization:

```bash
# Before optimization
make golden-capture TRACE=purify_before CMD='./target/release/bashrs purify large.sh'

# After optimization (make changes)
make golden-compare TRACE=purify_before CMD='./target/release/bashrs purify large.sh'
# Review diff - did you actually reduce syscalls?
```

### 3. CI/CD Integration

Add to CI pipeline to catch regressions automatically:

```yaml
# .github/workflows/golden-trace.yml
name: Golden Trace Verification
on: [pull_request]
jobs:
  verify:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install renacer
        run: cargo install renacer
      - name: Build bashrs
        run: cargo build --release
      - name: Compare against golden traces
        run: |
          make golden-compare TRACE=version CMD='./target/release/bashrs --version'
          make golden-compare TRACE=parse CMD='./target/release/bashrs parse examples/hello.rs'
```

### 4. Security Auditing

Capture golden traces for security-critical operations:

```bash
# Trace file access patterns for purify command
make golden-capture TRACE=purify_permissions CMD='./target/release/bashrs purify sensitive.sh'

# Verify no unexpected file accesses after code changes
make golden-compare TRACE=purify_permissions CMD='./target/release/bashrs purify sensitive.sh'
```

## Advanced Usage

### Custom Trace Analysis

Golden traces are stored as JSON in `rash/tests/golden_traces/`. You can analyze them programmatically:

```bash
# View trace details
cat rash/tests/golden_traces/version.json | jq .

# Count syscalls by type
cat rash/tests/golden_traces/version.json | jq '.syscalls[] | .name' | sort | uniq -c

# Find file operations
cat rash/tests/golden_traces/version.json | jq '.syscalls[] | select(.name | contains("open"))'
```

### Renacer Features

Renacer provides many advanced features beyond basic tracing:

```bash
# Source correlation (requires debug symbols)
renacer --source -- ./target/debug/bashrs --version

# Filter specific syscalls
renacer -e trace=file -- ./target/release/bashrs parse examples/hello.rs

# Timing analysis
renacer -T -- ./target/release/bashrs purify large.sh

# Follow forks
renacer -f -- ./target/release/bashrs build complex.rs
```

## Integration with EXTREME TDD

Golden traces fit naturally into EXTREME TDD workflow:

### RED Phase
```bash
# Write failing test that expects new behavior
cargo test test_new_feature -- --nocapture
# FAIL (as expected)
```

### GREEN Phase
```bash
# Implement feature
# ...

# Test passes
cargo test test_new_feature
# PASS

# Capture golden trace for regression prevention
make golden-capture TRACE=new_feature CMD='./target/release/bashrs new-command'
```

### REFACTOR Phase
```bash
# Refactor implementation
# ...

# Verify syscall patterns haven't changed
make golden-compare TRACE=new_feature CMD='./target/release/bashrs new-command'
# Should match golden (deterministic behavior preserved)
```

## Troubleshooting

### Timing Differences

**Problem**: Golden traces show timing differences even for identical code.

**Solution**: This is expected! Syscall timings vary between runs. The important part is:
- Syscall counts remain the same
- Syscall types remain the same
- File access patterns remain the same

If only timing differs, the trace is still valid.

### Trace Doesn't Match After Intentional Change

**Problem**: You intentionally changed behavior, so trace differs from golden.

**Solution**: Recapture the golden trace:
```bash
make golden-capture TRACE=name CMD='...'
```

Review the diff first to ensure the change is intentional!

### Renacer Not Found

**Problem**: `make golden-capture` fails with "renacer not found".

**Solution**: The Makefile auto-installs renacer, but you can manually install:
```bash
cargo install renacer
```

## Best Practices

1. **Capture golden traces for critical paths**
   - CLI entry points (`--version`, `--help`)
   - Core operations (`parse`, `purify`, `lint`)
   - Performance-sensitive code

2. **Review diffs carefully**
   - New syscalls may indicate bugs or security issues
   - Changed counts may indicate performance regressions
   - Different file paths may indicate incorrect behavior

3. **Version control golden traces**
   - Commit `rash/tests/golden_traces/*.json` to git
   - Review changes in PRs
   - Document why traces changed in commit messages

4. **Use in CI/CD**
   - Run `make golden-compare` in CI pipeline
   - Block PRs if traces differ unexpectedly
   - Require explicit golden trace updates

5. **Name traces descriptively**
   - Use format: `<command>_<scenario>`
   - Examples: `parse_hello`, `lint_unsafe`, `purify_complex`

## Makefile Targets Reference

| Target | Description | Example |
|--------|-------------|---------|
| `golden-help` | Show usage guide | `make golden-help` |
| `golden-capture` | Capture new golden trace | `make golden-capture TRACE=name CMD='...'` |
| `golden-compare` | Compare against golden | `make golden-compare TRACE=name CMD='...'` |
| `golden-list` | List all golden traces | `make golden-list` |
| `golden-clean` | Remove all golden traces | `make golden-clean` |

## Architecture

```
bashrs/
├── rash/
│   └── tests/
│       ├── golden_trace.rs          # Rust test helpers
│       └── golden_traces/           # Captured traces (JSON)
│           ├── version.json
│           ├── parse_hello.json
│           └── lint_unsafe.json
├── Makefile                         # Golden trace targets
└── docs/
    └── GOLDEN_TRACE.md             # This document
```

## FAQ

**Q: How much overhead does renacer add?**
A: Minimal (~5-10% on average). Renacer is written in pure Rust and uses efficient syscall tracing.

**Q: Can I use golden traces in tests?**
A: Yes! See `rash/tests/golden_trace.rs` for Rust test helpers.

**Q: What syscalls are traced?**
A: All syscalls by default. Use `-e trace=file` to filter specific types.

**Q: Are golden traces portable across systems?**
A: Syscall names/counts are portable, but paths and file descriptors may differ. Focus on syscall patterns, not absolute values.

**Q: How do I ignore timing differences?**
A: The current integration compares full JSON output. For production use, consider comparing only syscall names/counts (see `rash/tests/golden_trace.rs` for programmatic parsing).

## Resources

- [Renacer GitHub](https://github.com/paiml/renacer)
- [Renacer Documentation](https://docs.rs/renacer)
- [EXTREME TDD Guidelines](CLAUDE.md)
- [Toyota Way Principles](https://en.wikipedia.org/wiki/The_Toyota_Way)

## Future Enhancements

Planned improvements:

- [ ] Normalize timing data in comparisons
- [ ] Visual diff reports (HTML output)
- [ ] Integration with mutation testing
- [ ] Anomaly detection (ML-based)
- [ ] Cross-platform golden trace compatibility
- [ ] Automated trace update workflow
- [ ] Integration with book examples (mdbook)

---

**Status**: ✅ Production Ready (v6.36.0+)
**Maturity**: Stable
**Testing**: 100% coverage for test helpers
**Documentation**: Complete
