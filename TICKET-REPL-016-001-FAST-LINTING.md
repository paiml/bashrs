# TICKET-REPL-016-001: Fast Linting (<1 Second for Pre-Commit)

**Sprint**: REPL-016 (Performance Optimization)
**Status**: IN PROGRESS
**Priority**: HIGH
**Methodology**: EXTREME TDD

## Problem Statement

The bashrs linter currently takes too long for large scripts, making it impractical for pre-commit hooks. Developers need near-instant feedback (<1 second) for typical scripts to maintain flow state.

**Current Behavior** (Baseline needed):
```bash
$ time bashrs lint large_script.sh
# Need to establish baseline
real    0m??.???s
```

**Desired Behavior** (Performance target):
```bash
$ time bashrs lint large_script.sh  # 10,000 lines
real    0m00.950s  # <1 second

$ time bashrs lint typical_script.sh  # 1,000 lines
real    0m00.050s  # <100ms
```

## Requirements

### Performance Requirements

1. **Fast linting for typical scripts**
   - Target: <100ms for 1,000-line scripts
   - Baseline: Measure current performance
   - Optimization: Multi-threading, AST caching, rule optimization

2. **Sub-second for large scripts**
   - Target: <1s for 10,000-line scripts
   - Technique: Parallel rule execution, incremental parsing (future)
   - Measurement: Criterion benchmarks

3. **No quality degradation**
   - Requirement: 100% of lint rules still apply
   - Verification: All existing tests must pass
   - Trade-off: Performance WITHOUT sacrificing correctness

### Functional Requirements

1. **Parallel Rule Execution**
   - Run independent lint rules concurrently
   - Use rayon for data parallelism
   - Collect results efficiently

2. **AST Caching**
   - Cache parsed AST to avoid re-parsing
   - Invalidate cache on source changes
   - Memory-bounded cache (LRU eviction)

3. **Rule Optimization**
   - Profile slowest rules
   - Optimize regex patterns
   - Short-circuit early when possible

4. **Benchmarking Infrastructure**
   - Criterion.rs benchmarks
   - Compare before/after performance
   - Regression detection

## Design

### Architecture: Multi-Threaded Linting Pipeline

```
Input Script
     |
     v
Parse to AST (single-threaded, cached)
     |
     v
Lint Rules (parallel execution via rayon)
     |
     +-- Rule 1 (SEC001) --+
     +-- Rule 2 (DET001) --+
     +-- Rule 3 (IDEM001) -+
     +-- ... (14 rules)  --+
     |
     v
Collect Diagnostics (ordered by line number)
     |
     v
Format Output
```

### Parallel Execution with Rayon

```rust
use rayon::prelude::*;

pub fn lint_shell_fast(source: &str) -> LintResult {
    // Step 1: Parse AST (cached if possible)
    let ast = parse_cached(source)?;

    // Step 2: Run rules in parallel
    let diagnostics: Vec<Diagnostic> = ALL_RULES
        .par_iter()  // Parallel iterator
        .flat_map(|rule| rule.check(&ast, source))
        .collect();

    // Step 3: Sort by line number (deterministic output)
    let mut diagnostics = diagnostics;
    diagnostics.sort_by_key(|d| (d.span.start_line, d.span.start_col));

    LintResult { diagnostics }
}
```

### AST Caching Strategy

```rust
use lru::LruCache;
use std::sync::Mutex;

// Global cache (thread-safe)
lazy_static! {
    static ref AST_CACHE: Mutex<LruCache<u64, BashProgram>> =
        Mutex::new(LruCache::new(100)); // Cache 100 ASTs
}

fn parse_cached(source: &str) -> Result<BashProgram, ParseError> {
    // Hash source for cache key
    let hash = hash_source(source);

    // Check cache
    if let Some(ast) = AST_CACHE.lock().unwrap().get(&hash) {
        return Ok(ast.clone());
    }

    // Cache miss: parse and store
    let ast = parse_bash(source)?;
    AST_CACHE.lock().unwrap().put(hash, ast.clone());

    Ok(ast)
}

fn hash_source(source: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    source.hash(&mut hasher);
    hasher.finish()
}
```

### Rule Optimization: Regex Compilation

```rust
// BEFORE: Compile regex on every invocation
fn detect_random(source: &str) -> bool {
    let re = Regex::new(r"\$RANDOM").unwrap(); // ❌ Compiled every time
    re.is_match(source)
}

// AFTER: Compile regex once (lazy_static)
lazy_static! {
    static ref RANDOM_PATTERN: Regex = Regex::new(r"\$RANDOM").unwrap();
}

fn detect_random(source: &str) -> bool {
    RANDOM_PATTERN.is_match(source) // ✅ Compiled once
}
```

## Test Specifications

### Performance Benchmarks (Criterion.rs)

#### Benchmark: REPL-016-001-BENCH-001 - Lint 1,000-line script
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_lint_1000_lines(c: &mut Criterion) {
    let script = generate_bash_script(1000); // 1,000 lines

    c.bench_function("lint_1000_lines", |b| {
        b.iter(|| lint_shell(black_box(&script)))
    });
}

criterion_group!(benches, bench_lint_1000_lines);
criterion_main!(benches);
```

**Target**: <100ms (mean execution time)

#### Benchmark: REPL-016-001-BENCH-002 - Lint 10,000-line script
```rust
fn bench_lint_10000_lines(c: &mut Criterion) {
    let script = generate_bash_script(10_000); // 10,000 lines

    c.bench_function("lint_10000_lines", |b| {
        b.iter(|| lint_shell(black_box(&script)))
    });
}
```

**Target**: <1 second (mean execution time)

#### Benchmark: REPL-016-001-BENCH-003 - AST cache hit vs miss
```rust
fn bench_cache_performance(c: &mut Criterion) {
    let script = generate_bash_script(1000);

    c.bench_function("parse_cache_miss", |b| {
        b.iter(|| {
            clear_cache(); // Force cache miss
            parse_cached(black_box(&script))
        })
    });

    c.bench_function("parse_cache_hit", |b| {
        // Prime cache
        let _ = parse_cached(&script);

        b.iter(|| parse_cached(black_box(&script)))
    });
}
```

**Target**: Cache hit >10x faster than cache miss

### Unit Tests (Correctness)

#### Test: REPL-016-001-001 - Parallel linting produces same results
```rust
#[test]
fn test_REPL_016_001_001_parallel_results_match_sequential() {
    let source = r#"
echo $RANDOM
mkdir /tmp/test
rm /tmp/file
    "#;

    // Run sequential (original)
    let sequential = lint_shell(source);

    // Run parallel (optimized)
    let parallel = lint_shell_fast(source);

    // Should produce identical diagnostics
    assert_eq!(sequential.diagnostics.len(), parallel.diagnostics.len());

    for (s, p) in sequential.diagnostics.iter().zip(parallel.diagnostics.iter()) {
        assert_eq!(s.code, p.code);
        assert_eq!(s.span, p.span);
        assert_eq!(s.message, p.message);
    }
}
```

#### Test: REPL-016-001-002 - AST cache correctness
```rust
#[test]
fn test_REPL_016_001_002_cache_correctness() {
    let source = "echo hello";

    // Parse twice
    let ast1 = parse_cached(source).unwrap();
    let ast2 = parse_cached(source).unwrap();

    // Should be identical (from cache)
    assert_eq!(ast1, ast2);
}
```

#### Test: REPL-016-001-003 - Cache invalidation on source change
```rust
#[test]
fn test_REPL_016_001_003_cache_invalidation() {
    let source1 = "echo hello";
    let source2 = "echo world";

    let ast1 = parse_cached(source1).unwrap();
    let ast2 = parse_cached(source2).unwrap();

    // Different source = different AST
    assert_ne!(ast1, ast2);
}
```

#### Test: REPL-016-001-004 - No false negatives (all rules still fire)
```rust
#[test]
fn test_REPL_016_001_004_no_false_negatives() {
    // Script with violations from all 14 rules
    let source = include_str!("../fixtures/all_violations.sh");

    let result = lint_shell_fast(source);

    // Should detect all 14 violation types
    let violation_codes: HashSet<_> = result.diagnostics.iter()
        .map(|d| d.code.as_str())
        .collect();

    assert!(violation_codes.contains("SEC001"));
    assert!(violation_codes.contains("DET001"));
    assert!(violation_codes.contains("IDEM001"));
    // ... assert all 14 rules
}
```

### Property Tests

#### Property: REPL-016-001-PROP-001 - Parallel execution is deterministic
```rust
proptest! {
    #[test]
    fn prop_parallel_execution_deterministic(
        script in bash_script_generator(1..100)
    ) {
        // Run parallel linting multiple times
        let result1 = lint_shell_fast(&script);
        let result2 = lint_shell_fast(&script);
        let result3 = lint_shell_fast(&script);

        // All runs should produce identical results
        prop_assert_eq!(result1, result2);
        prop_assert_eq!(result2, result3);
    }
}
```

#### Property: REPL-016-001-PROP-002 - Cache never corrupts AST
```rust
proptest! {
    #[test]
    fn prop_cache_never_corrupts_ast(
        script in bash_script_generator(1..100)
    ) {
        // Parse without cache
        clear_cache();
        let uncached = parse_bash(&script).unwrap();

        // Parse with cache
        let cached = parse_cached(&script).unwrap();

        // Should be identical
        prop_assert_eq!(uncached, cached);
    }
}
```

## Implementation Plan

### Step 1: Establish Baseline (Measurement)
1. Create criterion benchmarks for current performance
2. Measure 1,000-line and 10,000-line scripts
3. Document baseline performance

### Step 2: RED Phase (Failing Performance Tests)
1. Write benchmark tests with <100ms and <1s targets
2. Run benchmarks - should FAIL (current impl too slow)
3. Write correctness tests (parallel = sequential results)

### Step 3: GREEN Phase (Optimization)
1. Implement parallel rule execution with rayon
2. Implement AST caching with LRU
3. Optimize regex compilation (lazy_static)
4. Run benchmarks - should PASS
5. Run correctness tests - should PASS

### Step 4: REFACTOR Phase
1. Extract caching logic to separate module
2. Add monitoring/metrics (cache hit rate)
3. Run clippy, ensure complexity <10
4. Add rustdoc documentation

### Step 5: PROPERTY Phase
1. Run property tests (100+ cases)
2. Verify determinism of parallel execution
3. Verify cache correctness

### Step 6: MUTATION Phase (Optional)
1. Run cargo mutants on optimized code
2. Target: ≥90% kill rate
3. Add tests for surviving mutants

## Quality Gates

- [ ] ✅ Baseline performance measured and documented
- [ ] ✅ Criterion benchmarks pass (<100ms for 1K lines, <1s for 10K lines)
- [ ] ✅ All unit tests pass (6 tests)
- [ ] ✅ All property tests pass (2 tests, 100+ cases each)
- [ ] ✅ No false negatives (all 14 lint rules still fire)
- [ ] ✅ Parallel execution is deterministic
- [ ] ✅ No clippy warnings
- [ ] ✅ Function complexity < 10
- [ ] ✅ Cache hit rate >80% in typical usage

## Dependencies

**Crates to add**:
```toml
[dependencies]
rayon = "1.10"         # Parallel execution
lru = "0.12"           # LRU cache
lazy_static = "1.5"    # Lazy regex compilation

[dev-dependencies]
criterion = "0.5"      # Benchmarking
```

## Risks

1. **Thread safety** - Lint rules must be thread-safe
   - Mitigation: Use immutable data structures, verify with property tests

2. **Memory usage** - AST cache could consume excessive memory
   - Mitigation: LRU cache with bounded size (100 entries)

3. **False negatives** - Parallel execution might miss violations
   - Mitigation: Comprehensive correctness tests, anti-fraud verification

## Success Criteria

1. Lint 1,000-line script in <100ms ✅
2. Lint 10,000-line script in <1 second ✅
3. All 14 lint rules still fire correctly ✅
4. No false negatives or false positives ✅
5. Deterministic parallel execution ✅
6. >80% cache hit rate in typical usage ✅

---

**Created**: 2025-10-31
**Author**: Claude (EXTREME TDD)
**Roadmap**: docs/REPL-DEBUGGER-ROADMAP.yaml
**Sprint**: REPL-016 (Performance Optimization)
