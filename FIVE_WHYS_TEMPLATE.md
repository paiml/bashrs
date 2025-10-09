# Five Whys Analysis Template
# Toyota Way: Hansei (反省) - Reflection and Root Cause Analysis

**Date**: YYYY-MM-DD
**Issue**: [Brief description of the problem]
**Methodology**: Toyota Way Five Whys + Root Cause Analysis
**Severity**: [P0/P1/P2/P3]

## The Problem Statement

[Detailed description of what went wrong, when it was discovered, and its impact]

Example:
```
When transpiling code with nested match expressions:
$ rash transpile examples/nested_match.rs
Error: Stack overflow in pattern matching
```

## Five Whys Analysis

### Why #1: Why did the problem occur?

**Answer**: [First-level cause - what immediately caused the symptom]

Example: Because the parser entered infinite recursion when processing nested patterns.

### Why #2: Why did [Answer from Why #1]?

**Answer**: [Second-level cause - what caused the first cause]

Example: Because the `convert_pattern` function calls itself without checking recursion depth.

### Why #3: Why did [Answer from Why #2]?

**Answer**: [Third-level cause - getting closer to root cause]

Example: Because we didn't implement a recursion guard when adding pattern matching in Sprint 19.

### Why #4: Why did [Answer from Why #3]?

**Answer**: [Fourth-level cause - organizational/process issue]

Example: Because the acceptance criteria for TICKET-5009 didn't include edge cases for deeply nested patterns.

### Why #5 (ROOT CAUSE): Why did [Answer from Why #4]?

**Answer**: **DESIGN FLAW** or **PROCESS FLAW** - The fundamental issue

Example: **DESIGN FLAW** - We designed the parser without considering recursion limits, assuming patterns would be shallow. This assumption is invalid for real-world code.

## Root Cause Identified

**Category**: [Design Flaw / Process Flaw / Knowledge Gap / Tooling Gap]

**The fundamental issue**:
[Clear statement of the root cause]

**Why this matters**:
[Explain the broader implications]

Example:
```
Category: Design Flaw + Process Flaw

The fundamental issue:
The parser was designed without recursion guards, and our test suite didn't
include property tests for deeply nested structures.

Why this matters:
1. Security: Stack overflow could be exploited (DoS attack)
2. Reliability: Crashes on valid Rust code
3. Process: Our property tests should catch these edge cases
```

## Impact Analysis

### Symptoms Observed
- [List all observed symptoms]
- [Include error messages, test failures, etc.]

### Affected Components
- [List all affected modules/files]
- [Include downstream impacts]

### Severity Assessment
**Business Impact**: [Critical/High/Medium/Low]
**Technical Impact**: [Critical/High/Medium/Low]
**User Impact**: [Critical/High/Medium/Low]

## The Better Design

### Current Design (Flawed)
```
[Pseudo-code or description of current implementation]
```

### Proposed Design (Fixed)
```
[Pseudo-code or description of fixed implementation]
```

### Design Principles Violated
- [List design principles that were violated]
- [Reference: Clean Code, SOLID, etc.]

Example:
```
Principles Violated:
1. Fail-Safe Defaults: Should have had recursion guards by default
2. Defense in Depth: Single point of failure (no depth checking)
3. Principle of Least Surprise: Valid Rust code shouldn't crash transpiler
```

## The Fix: Implementation Strategy

### Short-term Fix (Immediate)
**Ticket**: RASH-XXXX
**Duration**: [hours/days]

**Requirements**:
- [Minimal fix to stop the bleeding]
- [Must not introduce new issues]

**Tests**:
- [ ] test_fix_prevents_original_issue
- [ ] test_fix_no_regressions
- [ ] proptest_fix_handles_edge_cases

**Acceptance**:
- [ ] Original issue resolved
- [ ] No new failures introduced
- [ ] Property tests pass

### Long-term Solution (Robust)
**Ticket**: RASH-YYYY
**Duration**: [hours/days/weeks]

**Requirements**:
- [Comprehensive solution addressing root cause]
- [Prevents entire class of issues]

**Tests**:
- [ ] test_solution_comprehensive
- [ ] proptest_solution_properties
- [ ] test_solution_performance

**Acceptance**:
- [ ] Root cause eliminated
- [ ] Process updated to prevent recurrence
- [ ] Documentation complete

## Implementation Plan (EXTREME TDD)

### Phase 1: RED - Write Failing Tests

```rust
#[test]
fn test_deeply_nested_patterns_dont_overflow() {
    // Generate 100-level deep pattern
    let code = generate_deep_pattern(100);
    let result = transpile(&code);
    assert!(result.is_ok()); // Currently fails
}

proptest! {
    #[test]
    fn prop_pattern_depth_bounded(depth in 1..1000) {
        let code = generate_deep_pattern(depth);
        let result = transpile(&code);
        // Should either succeed or return controlled error
        assert!(result.is_ok() || matches!(result, Err(Error::MaxDepthExceeded(_))));
    }
}
```

### Phase 2: GREEN - Implement Minimal Fix

```rust
const MAX_PATTERN_DEPTH: usize = 64;

fn convert_pattern(&mut self, pat: &syn::Pat, depth: usize) -> Result<Pattern> {
    if depth > MAX_PATTERN_DEPTH {
        return Err(Error::MaxDepthExceeded(depth));
    }

    match pat {
        syn::Pat::TupleStruct(inner) => {
            // Recursive call with depth tracking
            self.convert_pattern(&inner.pat, depth + 1)?
        }
        // ... other cases
    }
}
```

### Phase 3: REFACTOR - Clean Implementation

```rust
// Extract depth tracking into trait
trait DepthBounded {
    const MAX_DEPTH: usize;
    fn check_depth(&self, depth: usize) -> Result<()>;
}

impl DepthBounded for Parser {
    const MAX_DEPTH: usize = 64;

    fn check_depth(&self, depth: usize) -> Result<()> {
        if depth > Self::MAX_DEPTH {
            Err(Error::MaxDepthExceeded(depth))
        } else {
            Ok(())
        }
    }
}
```

### Phase 4: VERIFY - Quality Gates

```bash
# Run all tests
make test-all

# Check complexity didn't increase
pmat analyze complexity src/parser/

# Verify mutation testing
make mutants TARGET=src/parser/patterns.rs

# Performance regression check
make bench
```

## Prevention Strategy

### Process Improvements

**What we'll change**:
1. [Process change to prevent recurrence]
2. [Additional checks/reviews]
3. [Documentation updates]

Example:
```
1. Add "Edge Case Checklist" to PR template:
   - [ ] Tested with deeply nested structures (100+ levels)
   - [ ] Recursion guards in place
   - [ ] Property tests for unbounded inputs

2. Update acceptance criteria template to include:
   - [ ] Edge cases identified and tested
   - [ ] Resource limits (recursion, memory, time) documented

3. Add to CLAUDE.md:
   - All recursive functions MUST have depth guards
   - MAX_DEPTH constants must be tested at boundary
```

### Code Quality Improvements

**What we'll add**:
1. [Static analysis rules]
2. [Linting rules]
3. [Test requirements]

Example:
```
1. Add clippy lint: #![deny(clippy::unconditional_recursion)]
2. Add pre-commit hook: Check for recursive functions without guards
3. Require property tests for all recursive functions
```

### Documentation Updates

**What we'll document**:
1. [Architecture decisions]
2. [Known limitations]
3. [Design constraints]

Example:
```
1. Add to docs/architecture/CONSTRAINTS.md:
   - Maximum pattern nesting depth: 64
   - Maximum expression depth: 128
   - Rationale: Prevents stack overflow, matches rustc limits

2. Update docs/testing/PROPERTY_TESTS.md:
   - All recursive functions require depth property tests
   - Template for depth-bounded property tests
```

## Lessons Learned

### What Went Well
- [Positive aspects]
- [What caught the issue]
- [Good responses]

### What Went Wrong
- [Issues in process]
- [Gaps in testing]
- [Missing checks]

### What We'll Do Differently
- [Specific changes to make]
- [New practices to adopt]
- [Checks to add]

## Toyota Way Principles Applied

### ✅ Genchi Genbutsu (現地現物) - Go and See
**Applied**: [How we observed the actual problem]

Example: Dogfooded the transpiler on real-world Rust code with complex pattern matching.

### ✅ Hansei (反省) - Reflection
**Applied**: This Five Whys analysis

### ✅ Kaizen (改善) - Continuous Improvement
**Applied**: [Process improvements made]

Example: Updated acceptance criteria template to include edge case requirements.

### ✅ Jidoka (自働化) - Build Quality In
**Applied**: [Quality measures added]

Example: Added property tests for recursion depth, added clippy lint for unconditional recursion.

## Metrics and Validation

### Before Fix
- Test coverage: [X%]
- Mutation score: [Y%]
- Issue count: [N issues]
- Performance: [baseline]

### After Fix
- Test coverage: [X+Z%]
- Mutation score: [Y+M%]
- Issue count: [0 issues]
- Performance: [no regression]

### Validation Checklist
- [ ] Original issue resolved
- [ ] All tests pass
- [ ] No performance regression
- [ ] Documentation updated
- [ ] Process improved
- [ ] Team trained on fix

## References

### Related Issues
- [Link to GitHub issues]
- [Related bugs/features]

### Related Documentation
- [Architecture docs]
- [Design docs]
- [Test documentation]

### External Resources
- [Rust reference]
- [Best practices]
- [Similar issues in other projects]

---

## Follow-up Actions

### Immediate (This Sprint)
- [ ] [Action item with owner and deadline]
- [ ] [Action item with owner and deadline]

### Short-term (Next Sprint)
- [ ] [Action item with owner and deadline]
- [ ] [Action item with owner and deadline]

### Long-term (Roadmap)
- [ ] [Action item with owner and deadline]
- [ ] [Action item with owner and deadline]

---

**Status**: [Draft / Under Review / Approved / Implemented]
**Reviewers**: [Names]
**Approval Date**: YYYY-MM-DD
**Implementation Date**: YYYY-MM-DD

---

## Appendix

### Error Messages
```
[Full error messages and stack traces]
```

### Test Output
```
[Relevant test output]
```

### Code Snippets
```rust
[Relevant code before and after]
```

### Performance Data
```
[Benchmark results if applicable]
```
