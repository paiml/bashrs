# Sprint 2: Enhanced Linting - Auto-Fix & Performance

**Sprint Goal**: Implement auto-fix application and performance benchmarking to complete v1.1.0 release

**Priority**: High (Release Blocker for v1.1.0)

---

## Ticket LINT-007: Auto-Fix Application (`--fix` flag)
**Priority**: P0 (Release Blocker)
**Estimate**: 2 hours

### Description
Implement `--fix` flag to automatically apply suggested fixes to files.

### Acceptance Criteria
1. ✅ `bashrs lint --fix <file>` applies all auto-fixes
2. ✅ Creates backup file before modification
3. ✅ Reports which fixes were applied
4. ✅ Dry-run mode (`--fix --dry-run`)
5. ✅ Tests for safe application

### Implementation
```rust
// Pseudo-code
if args.fix {
    let backup_path = format!("{}.bak", input_path);
    fs::copy(input_path, backup_path)?;

    let mut content = fs::read_to_string(input_path)?;
    for diagnostic in result.diagnostics.iter().rev() {  // Apply in reverse order
        if let Some(fix) = &diagnostic.fix {
            content = apply_fix(content, diagnostic.span, &fix.replacement)?;
        }
    }

    fs::write(input_path, content)?;
}
```

---

## Ticket LINT-008: Performance Benchmarking
**Priority**: P1 (Quality Gate)
**Estimate**: 1 hour

### Description
Create benchmarks to verify linter performance targets.

### Acceptance Criteria
1. ✅ Benchmark for 100-line script
2. ✅ Benchmark for 1000-line script
3. ✅ Target: <50ms for 1000-line script
4. ✅ Comparison with external ShellCheck

### Implementation
```rust
// benches/lint_performance.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_lint_small(c: &mut Criterion) {
    let script = include_str!("../tests/fixtures/small_script.sh");
    c.bench_function("lint_100_lines", |b| {
        b.iter(|| lint_shell(black_box(script)))
    });
}

fn bench_lint_large(c: &mut Criterion) {
    let script = include_str!("../tests/fixtures/large_script.sh");
    c.bench_function("lint_1000_lines", |b| {
        b.iter(|| lint_shell(black_box(script)))
    });
}
```

---

## Sprint 2 Timeline (4 hours)

### Phase 1: Auto-Fix (2 hours)
- Implement fix application logic
- Add backup creation
- Write tests
- Manual validation

### Phase 2: Benchmarking (1 hour)
- Create benchmark suite
- Generate test fixtures
- Run benchmarks
- Document results

### Phase 3: Documentation (1 hour)
- Update sprint report
- Create CHANGELOG for v1.1.0
- Prepare release notes

---

## Definition of Done ✅ COMPLETE

- [x] `--fix` flag working with backup creation
- [x] Performance benchmarks passing (<50ms for 1000 lines)
- [x] Documentation updated
- [x] Released (current: v6.39.0)
