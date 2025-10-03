# Sprint 20: Mutation Testing Baseline Analysis

**Date**: 2025-10-03
**Version**: v0.6.0
**Status**: IN PROGRESS
**Target**: ≥90% mutation kill rate overall

---

## Executive Summary

Implementing mutation testing for Rash transpiler following `docs/specifications/MUTATION-TESTING.md`. This sprint establishes baseline metrics and improves test effectiveness beyond simple coverage numbers.

**Current Status (v0.6.0)**:
- ✅ 527/530 tests passing (99.4%)
- ✅ 24 property tests (~14,000 cases)
- ✅ 85.36% core coverage
- 🔄 Mutation kill rate: PENDING

---

## Setup (Sprint 20.1) ✅ COMPLETE

### Tools Installed
- ✅ cargo-mutants v25.3.1 (already installed)

### Configuration Created
- ✅ `.cargo/mutants.toml` - Mutation testing configuration
  - Excluded: rash-mcp (external dependency), tests, examples, binaries
  - Timeout: 60s minimum per test
  - Focus: Core transpiler modules

### Makefile Targets Added
- ✅ `make mutants` - Full analysis (7-10 hours)
- ✅ `make mutants-quick` - Recently changed files (~1 hour)
- ✅ `make mutants-parser` - Parser module only
- ✅ `make mutants-ir` - IR converter module only
- ✅ `make mutants-emitter` - Emitter module only
- ✅ `make mutants-validation` - Validation module only
- ✅ `make mutants-report` - Generate summary report
- ✅ `make mutants-clean` - Clean artifacts

---

## Baseline Analysis (Sprint 20.2) 🔄 IN PROGRESS

### Module Targets

| Module | File | Expected Mutants | Target Kill Rate | Priority |
|--------|------|-----------------|------------------|----------|
| **Parser** | `services/parser.rs` | ~150 | ≥92% | Critical |
| **IR Converter** | `ir/mod.rs` | ~120 | ≥90% | Critical |
| **Emitter** | `emitter/posix.rs` | ~100 | ≥90% | Critical |
| **Validation** | `validation/pipeline.rs` | ~60 | ≥95% | Critical |
| **AST** | `ast/restricted.rs` | ~40 | ≥85% | Medium |
| **Models** | `models/error.rs` | ~20 | ≥80% | Low |

### Running Baseline Tests

#### Validation Module
```bash
cargo mutants --package bashrs --file 'rash/src/validation/pipeline.rs' --no-times
```

**Status**: RUNNING (background process)
**Expected**: ~60 mutants, ~30-60 minutes
**Results**: PENDING

---

## Expected Mutation Examples

### Parser Mutations (from spec)

```rust
// Original: convert_match_stmt
pub fn convert_match_stmt(expr_match: &syn::ExprMatch) -> Result<Stmt> {
    let scrutinee = convert_expr(&expr_match.expr)?;
    let arms = convert_arms(&expr_match.arms)?;
    Ok(Stmt::Match { scrutinee, arms })
}

// Mutation 1: Skip scrutinee conversion (should FAIL)
let scrutinee = Expr::Literal(Literal::I32(0));

// Mutation 2: Skip arms conversion (should FAIL)
let arms = Vec::new();

// Mutation 3: Return error (should FAIL)
Err(Error::Validation("unsupported".into()))
```

**Good tests kill these mutations**:
- Assert successful transpilation (kills mutation 3)
- Assert generated case statement has correct scrutinee (kills mutation 1)
- Assert case arms present (kills mutation 2)

### IR Mutations (from spec)

```rust
// Original: Range adjustment for exclusive ranges
if !inclusive {
    if let ShellValue::String(s) = &end_val {
        if let Ok(n) = s.parse::<i32>() {
            end_val = ShellValue::String((n - 1).to_string());
        }
    }
}

// Mutation 1: Remove decrement (should FAIL)
end_val = ShellValue::String(n.to_string());

// Mutation 2: Flip inclusive check (should FAIL)
if inclusive {
```

**Good tests kill these mutations**:
- Property test: `0..3` must generate `seq 0 2` (kills mutation 1)
- Test inclusive vs exclusive ranges separately (kills mutation 2)

### Emitter Mutations (from spec)

```rust
// Original: emit_case_statement
let pattern_str = match &arm.pattern {
    CasePattern::Literal(lit) => lit.clone(),
    CasePattern::Wildcard => "*".to_string(),
};

// Mutation 1: Wrong wildcard (should FAIL)
CasePattern::Wildcard => "_".to_string(),

// Mutation 2: Swap patterns (should FAIL)
CasePattern::Literal(lit) => "*".to_string(),
CasePattern::Wildcard => lit.clone(),
```

**Good tests kill these mutations**:
- Assert wildcard generates `*)` not `_)` (kills mutation 1)
- Assert literals generate correct value (kills mutation 2)

---

## Test Improvement Strategy

### Phase 1: Identify Survivors (Current)
1. Run mutation testing on each module
2. Analyze which mutants survived
3. Categorize: acceptable vs test gaps

### Phase 2: Improve Tests
1. Add assertions for mutation survivors
2. Expand property test cases
3. Add specific edge case tests

### Phase 3: Verify Improvement
1. Re-run mutation testing
2. Measure kill rate improvement
3. Document results

---

## Workspace Configuration Issues

**Issue**: `rash-mcp` package has external dependency on `pforge-runtime` which breaks cargo-mutants when it copies workspace to temp directory.

**Solution**: Temporarily disabled `rash-mcp` from workspace members in root `Cargo.toml`:
```toml
members = [
    "rash",
    "rash-runtime",
    # "rash-mcp",  # Temporarily disabled for mutation testing
]
```

**Impact**: None on core transpiler testing. `rash-mcp` is an MCP server wrapper, not part of core transpiler logic.

---

## Next Steps

1. **Wait for validation baseline** to complete (~30-60 min)
2. **Analyze results**:
   - How many mutants caught/missed?
   - Which patterns are weak tests?
3. **Run other modules** (parser, IR, emitter)
4. **Improve tests** based on survivors
5. **Document final metrics**

---

## Success Criteria

### Sprint 20 Complete When:
- ✅ Mutation testing configured and running
- ⏳ Baseline analysis complete for all 4 core modules
- ⏳ Kill rate measured: validation, parser, IR, emitter
- ⏳ Documentation of survivors and test gaps
- ⏳ Action plan for Sprint 21 (test improvement)

### Overall Target (Sprint 20-23):
- **≥90% mutation kill rate** overall
- **≥92% parser**, **≥90% IR/emitter**, **≥95% validation**
- CI/CD integration with weekly runs
- `.quality/mutation-final-report.md` with metrics

---

---

## Pragmatic Decision: Infrastructure Complete, Full Analysis Deferred

**Rationale**: Full mutation testing requires 7-10 hours of compute time. Infrastructure is complete and ready for async execution.

**What's Ready**:
- ✅ cargo-mutants installed and configured
- ✅ `.cargo/mutants.toml` with proper exclusions
- ✅ Makefile targets for all mutation testing workflows
- ✅ Documentation and specifications complete
- ✅ Baseline analysis document created

**How to Run** (when time allows):
```bash
# Quick check (1-2 hours) - recently changed code
make mutants-quick

# Module-specific (30-60 min each)
make mutants-validation   # ~60 mutants
make mutants-parser       # ~150 mutants
make mutants-ir           # ~120 mutants
make mutants-emitter      # ~100 mutants

# Full analysis (7-10 hours) - run overnight or on CI
make mutants
make mutants-report
```

**Sprint 20.1 Status**: ✅ COMPLETE (Infrastructure)
**Sprint 20.2-20.4 Status**: 📅 DEFERRED (Analysis, Improvement, Integration)

**Recommendation**: Run `make mutants` overnight or as weekly CI job. Results will be in `mutants.out/` directory.

---

**Status**: Sprint 20.1 (Setup) ✅ COMPLETE
**Next**: Move to Option 2 (Feature Completion) - Higher immediate value
