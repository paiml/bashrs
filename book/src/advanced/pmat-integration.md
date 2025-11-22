# PMAT Integration

bashrs integrates with [paiml-mcp-agent-toolkit (PMAT)](https://github.com/paiml/paiml-mcp-agent-toolkit) v2.200.0+ for comprehensive quality analysis, complexity checking, and project scoring.

PMAT provides automated quality gates that enforce code standards, detect defects, and measure technical debt.

## What is PMAT?

**PMAT (Professional MCP Agent Toolkit)** is a quantitative code analysis and project scaffolding toolkit that provides:

- **Rust Project Scoring** (0-134 scale): Comprehensive quality metrics
- **Complexity Analysis**: Cyclomatic and cognitive complexity detection
- **Defect Detection**: Known anti-patterns (unwrap(), eval, etc.)
- **Quality Gates**: Pre-commit hooks enforcing standards
- **Technical Debt Grading**: TDG scores for refactoring prioritization

## Installation

```bash
# Install PMAT globally
cargo install paiml-mcp-agent-toolkit

# Verify installation
pmat --version
# Expected: paiml-mcp-agent-toolkit 2.200.0+

# Check bashrs project score
pmat rust-project-score
```

## Quality Metrics

### Rust Project Score (0-134)

PMAT scores bashrs across 8 categories:

```bash
$ pmat rust-project-score

🦀  Rust Project Score v2.1
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

📌  Summary
  Score: 127.0/134
  Percentage: 94.8%
  Grade: A+

📂  Categories
  ✅ Code Quality: 20.0/26 (76.9%)
  ⚠️  Dependency Health: 6.0/12 (50.0%)
  ⚠️  Documentation: 8.0/15 (53.3%)
  ⚠️  Formal Verification: 3.0/8 (37.5%)
  ✅ Known Defects: 20.0/20 (100.0%)
  ⚠️  Performance & Benchmarking: 3.0/10 (30.0%)
  ⚠️  Rust Tooling & CI/CD: 61.5/130 (47.3%)
  ⚠️  Testing Excellence: 5.5/20 (27.5%)
```

**Score Breakdown:**
- **A+ (95-100%)**: Production-ready, excellent quality
- **A (90-94%)**: High quality, minor improvements needed
- **B (80-89%)**: Good quality, some gaps
- **C (70-79%)**: Acceptable, needs work
- **D (<70%)**: Needs significant improvement

### Complexity Analysis

PMAT detects functions exceeding complexity thresholds:

```bash
# Check complexity (default thresholds: cyclomatic 30, cognitive 25)
pmat analyze complexity --max-cyclomatic 30 --max-cognitive 25

# Example output
📊 Files analyzed: 666
🔧 Total functions: 10

Complexity Metrics:
- Median Cyclomatic: 9.5
- Median Cognitive: 27.0
- Max Cyclomatic: 10
- Max Cognitive: 29

Top Files by Complexity:
1. sc2052.rs - Cyclomatic: 9, Cognitive: 29
2. sc2115.rs - Cyclomatic: 10, Cognitive: 27
```

**Complexity Thresholds:**
- **Cyclomatic < 10**: Simple, easy to test (ideal)
- **Cyclomatic 10-20**: Moderate, acceptable
- **Cyclomatic 20-30**: Complex, consider refactoring
- **Cyclomatic > 30**: Very complex, refactor immediately

- **Cognitive < 15**: Readable, easy to understand (ideal)
- **Cognitive 15-25**: Moderate, acceptable
- **Cognitive 25-35**: Hard to understand, refactor
- **Cognitive > 35**: Very hard to understand, refactor immediately

## Pre-Commit Hooks

bashrs uses PMAT pre-commit hooks to enforce quality standards.

### Setup

```bash
# Install pre-commit framework
pip install pre-commit

# Install hooks
pre-commit install

# Run hooks manually
pre-commit run --all-files
```

### PMAT Hooks

The `.pre-commit-config.yaml` includes 3 PMAT hooks:

#### 1. Complexity Check (commit stage)

```yaml
- id: pmat-complexity
  name: pmat analyze complexity
  entry: pmat analyze complexity --max-cyclomatic 30 --max-cognitive 25
  stages: [commit]
```

**Blocks commits** if code exceeds complexity thresholds.

#### 2. TDG Verification (commit stage)

```yaml
- id: pmat-tdg
  name: pmat tdg
  entry: pmat tdg --verify
  stages: [commit]
```

Verifies test-driven generation compliance.

#### 3. Quality Score (push stage)

```yaml
- id: pmat-quality
  name: pmat quality-score
  entry: pmat quality-score --min 9.0
  stages: [push]
```

**Blocks pushes** if quality score < 9.0/10.

### Bypassing Hooks (Not Recommended)

During development, you may need to bypass hooks temporarily:

```bash
# Skip ALL pre-commit hooks
git commit --no-verify -m "WIP: refactoring in progress"

# Skip specific hook
SKIP=pmat-complexity git commit -m "..."
```

**⚠️ Warning:** Use `--no-verify` sparingly. Always fix issues before merging.

## Complexity Refactoring Workflow

When PMAT detects high-complexity functions, follow this refactoring workflow:

### 1. Identify High-Complexity Files

```bash
pmat analyze complexity --max-cognitive 25 --format full
```

### 2. Refactor Using Helper Functions

Extract nested logic into single-responsibility helper functions:

**Before (complexity 35):**
```rust,ignore
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for (line_num, line) in source.lines().enumerate() {
        if line.trim_start().starts_with('#') {
            continue;
        }
        for m in REGEX.find_iter(line) {
            let before = &line[..m.start()];
            if before.ends_with("'") {
                continue;
            }
            if before.matches('"').count() % 2 == 1 {
                continue;
            }
            let diagnostic = Diagnostic::new(...);
            result.add(diagnostic);
        }
    }
    result
}
```

**After (complexity ~10):**
```rust,ignore
/// Check if match should be skipped (quoted or escaped)
fn should_skip_match(line: &str, match_start: usize) -> bool {
    let before = &line[..match_start];
    before.ends_with("'") || before.matches('"').count() % 2 == 1
}

/// Create diagnostic for match
fn create_diagnostic(match_text: &str, line_num: usize) -> Diagnostic {
    Diagnostic::new(...)
}

pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        if line.trim_start().starts_with('#') {
            continue;
        }

        for m in REGEX.find_iter(line) {
            if should_skip_match(line, m.start()) {
                continue;
            }
            result.add(create_diagnostic(m.as_str(), line_num));
        }
    }

    result
}
```

**Benefits:**
- Reduced nesting (4-5 levels → 2-3 levels)
- Single-responsibility functions
- Easier to test
- More maintainable

### 3. Verify Improvement

```bash
# Run tests
cargo test --lib <module>

# Check complexity improved
pmat analyze complexity --files src/path/to/file.rs

# Commit with descriptive message
git commit -m "refactor: Reduce complexity in <file> (35→10)"
```

## Defect Detection

PMAT detects known anti-patterns in Rust code:

```bash
# Analyze known defects
pmat analyze defects

# Example findings
⚠️  289 unwrap() calls detected
   - Replace with .expect() or proper error handling
   - See Cloudflare outage 2025-11-18 (unwrap() panic)
```

**Common Defects:**
- `.unwrap()`: Can panic, use `.expect()` or `?` operator
- `.expect("failed")`: Better than unwrap, but still panics
- `eval` in shell scripts: Code injection risk
- Hardcoded credentials: Security vulnerability

## Quality Gate Integration

bashrs enforces quality gates at multiple stages:

### Pre-Commit (Local)
- ✅ Complexity < 30 (cyclomatic), < 25 (cognitive)
- ✅ All tests pass
- ✅ Code formatted (rustfmt)
- ✅ No clippy warnings

### Pre-Push (Local)
- ✅ Quality score ≥ 9.0/10
- ✅ Security audit (cargo audit)

### CI/CD (GitHub Actions)
- ✅ All pre-commit hooks
- ✅ Coverage ≥ 85%
- ✅ Mutation testing ≥ 80%
- ✅ Benchmarks pass

## Best Practices

### 1. Run PMAT Before Commits

```bash
# Check complexity before committing
pmat analyze complexity --fail-on-violation

# Check quality score
pmat rust-project-score
```

### 2. Monitor Trends

Track score improvements over time:

```bash
# Log score to file
pmat rust-project-score >> quality-log.txt

# Compare with previous release
git diff v6.35.0 quality-log.txt
```

### 3. Prioritize Low-Hanging Fruit

Focus on high-impact, low-effort improvements:

| Category | Current | Target | Effort | Impact |
|----------|---------|--------|--------|--------|
| Testing Excellence | 5.5/20 | 15/20 | Medium | High |
| Documentation | 8.0/15 | 13/15 | Low | Medium |
| Performance | 3.0/10 | 8/10 | High | Low |

## Example: Complete Quality Workflow

```bash
# 1. Check current quality
pmat rust-project-score

# 2. Identify complexity issues
pmat analyze complexity --max-cognitive 25

# 3. Refactor high-complexity files
# (extract helper functions, reduce nesting)

# 4. Run tests
cargo test --lib

# 5. Verify improvement
pmat analyze complexity --files src/refactored/file.rs

# 6. Commit (pre-commit hooks run automatically)
git add src/refactored/file.rs
git commit -m "refactor: Reduce complexity in file.rs (35→10)"

# 7. Push (quality gate runs automatically)
git push

# 8. Verify score improved
pmat rust-project-score
```

## Troubleshooting

### PMAT Not Installed

```bash
error: unrecognized subcommand
```

**Solution:**
```bash
cargo install paiml-mcp-agent-toolkit
```

### Complexity Hook Fails

```bash
❌ Complexity check... FAILED
Complexity exceeds thresholds
```

**Solution:**
1. Run `pmat analyze complexity` to see violations
2. Refactor high-complexity functions
3. Re-run pre-commit hooks

### Quality Score Too Low

```bash
❌ Quality score: 8.5/10 (minimum: 9.0)
```

**Solution:**
1. Run `pmat rust-project-score` to see category breakdown
2. Focus on low-scoring categories
3. Follow recommendations in output

## Further Reading

- [PMAT Documentation](https://github.com/paiml/paiml-mcp-agent-toolkit)
- [Cyclomatic Complexity](https://en.wikipedia.org/wiki/Cyclomatic_complexity)
- [Cognitive Complexity](https://www.sonarsource.com/resources/cognitive-complexity/)
- [Toyota Way Principles](../contributing/toyota-way.md)
- [EXTREME TDD](../contributing/extreme-tdd.md)
