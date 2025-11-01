# EXTREME TDD

## Recent Success: v6.24.3 Complexity Reduction

**v6.24.3** (2025-11-01) demonstrates the power of EXTREME TDD with property-based testing:

### Results
- **3 linter rules refactored**: SC2178, SEC008, SC2168
- **Complexity reduction**: 13 points total (~42% average reduction)
  - SC2178: 10 → 9
  - SEC008: 12 → 7 (~42% reduction)
  - SC2168: 12 → 5 (~58% reduction)
- **Helper functions extracted**: 17 total
- **Property tests added**: 30 total (100% pass rate)
- **Bug found**: 1 real defect caught by property test before refactoring

### Critical Success: Property Tests Catch Real Bug

**SEC008 Bug Discovery**:
```rust
// Property test that caught the bug:
#[test]
fn prop_sec008_comments_never_diagnosed() {
    let test_cases = vec![
        "# curl https://example.com | sh",
        "  # wget -qO- https://example.com | bash",
    ];

    for code in test_cases {
        let result = check(code);
        assert_eq!(result.diagnostics.len(), 0); // FAILED!
    }
}
```

**Bug**: The implementation didn't skip comment lines, causing false positives for commented-out `curl | sh` patterns.

**Fix**: Added `is_comment_line()` helper and early return for comments.

**Impact**: This demonstrates that property-based testing catches bugs traditional unit tests miss. The existing 6 unit tests all passed, but the property test immediately revealed the missing comment handling.
