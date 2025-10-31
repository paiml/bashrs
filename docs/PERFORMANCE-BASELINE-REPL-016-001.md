# Performance Baseline: REPL-016-001 Fast Linting

**Date**: 2025-10-31
**Task**: REPL-016-001 - Fast linting (<1 second for pre-commit)
**System**: Linux 6.8.0-85-generic
**Build**: cargo build --release (v6.22.0)

## Baseline Measurements

### Test Setup

**Test Scripts Generated**:
- `/tmp/test_1000.sh` - 1,004 lines (mix of commands, loops, variables)
- `/tmp/test_10000.sh` - 10,004 lines (same mix scaled up)

**Command**:
```bash
time target/release/bashrs lint <script> > /dev/null 2>&1
```

### Results

| Script Size | Execution Time | Target | Status |
|-------------|---------------|---------|--------|
| 1,000 lines | **91ms** | <100ms | ✅ **PASS** |
| 10,000 lines | **306ms** | <1000ms | ✅ **PASS** |

## Analysis

### Current Performance: EXCEEDS TARGETS ✅

The bashrs linter **already meets** both performance targets:
- 1K lines: 91ms < 100ms target (9% margin)
- 10K lines: 306ms < 1000ms target (69% margin)

### Why Is It Fast?

1. **Efficient parser**: tree-sitter-based parsing is fast
2. **Optimized rules**: Only 14 lint rules (small rule set)
3. **Single-threaded simplicity**: No parallelization overhead

### Should We Still Optimize?

**YES** - for the following reasons:

1. **Tighter margins**: 9% margin on 1K lines is narrow
2. **Future growth**: More lint rules will be added (target: 800+ rules)
3. **Large scripts**: Some production scripts are 50K+ lines
4. **EXTREME TDD demonstration**: Show optimization methodology

## Optimization Strategy (Revised)

### Phase 1: Benchmark Infrastructure (RED Phase)
- Add Criterion benchmarks
- Set stricter targets (50ms for 1K, 500ms for 10K)
- Establish regression detection

### Phase 2: Parallel Execution (GREEN Phase)
- Implement rayon-based parallel rule execution
- Target: 2-3x speedup for large scripts
- Verify no correctness regressions

### Phase 3: AST Caching (Future)
- Deferred to REPL-016-003 (Caching)
- Not needed for single-run performance
- Useful for incremental linting

## Revised Targets

| Script Size | Current | Target | Stretch Goal |
|-------------|---------|---------|--------------|
| 1,000 lines | 91ms | 50ms | 25ms |
| 10,000 lines | 306ms | 150ms | 100ms |
| 100,000 lines | ??? | <5s | <2s |

## Next Steps

1. ✅ Baseline documented
2. ⏳ Create Criterion benchmarks
3. ⏳ RED Phase: Write performance tests with stricter targets
4. ⏳ GREEN Phase: Implement parallel execution
5. ⏳ Verify 2-3x speedup achieved

---

**Conclusion**: Current performance is **excellent** but we'll optimize further for future growth and tight margins.
