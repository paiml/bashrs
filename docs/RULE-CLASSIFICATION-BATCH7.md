# Rule Classification - Batch 7 Analysis

**Date**: 2025-11-02
**Goal**: Classify 20 additional SC2xxx rules (batch 7)
**Current**: 140/357 (39.2%)
**Target**: 160/357 (44.8%)

## Strategy: Remaining High-Priority Universal Rules

Batch 7 aims to reach the **45% milestone** for v6.28.0 release by focusing on:
1. **Remaining high-frequency Universal rules** (SC2137-SC2145 range)
2. **Loop and iteration patterns** (common script issues)
3. **Test operator safety** (additional [ ] and [[ ]] rules)
4. **Expansion and substitution rules** (parameter expansion issues)

## Batch 7 Classification List (20 rules target)

### Universal Rules (Estimated 18-20 rules)

**Test Operator Safety** (5-7 rules):
1. SC2137 - Unnecessary braces in arithmetic (style)
2. SC2138 - Function defined in wrong context (detect functions in if/loop, reserved names)
3. SC2139 - Alias variable expands at definition time (warn about early expansion)
4. SC2140 - Malformed quote concatenation (detect unquoted words between quotes)
5. SC2141 - Command receives stdin but ignores it (find, ls, echo, sudo)
6. SC2142 - Aliases can't use positional parameters (recommend functions instead)
7. SC2143 - Use grep -q for efficiency (exits on first match)

**Find and Loop Safety** (4-5 rules):
8. SC2144 - -e check on globs that never match (glob safety)
9. SC2145 - Array expansion in quotes (word splitting prevention)
10. SC2146 - find -o action grouping with parentheses
11. SC2147 - Literal tilde in PATH doesn't expand
12. SC2148 - Add shebang to indicate interpreter (portability)

**Variable and Expansion** (5-6 rules):
13. SC2149 - Remove quotes from unset variable names
14. SC2150 - Use -exec + instead of \\; for batch processing
15. Additional expansion rules from SC2137-SC2160 range

### NotSh Rules (Estimated 0-2 rules)

No obvious NotSh rules in this batch - focus on Universal coverage.

## Priority Justification

**High-Frequency Rules** (batch 7 focuses on test operators and find patterns):
1. **SC2137-2143**: Test operator safety (very common in conditionals)
2. **SC2144-2150**: Find and glob safety (common in file iteration)
3. **Expansion rules**: Parameter expansion issues (common in scripts)

**Complementary Coverage**:
- Batch 1: SEC, DET, IDEM + bash-isms
- Batch 2: Arithmetic, function keyword, quoting basics
- Batch 3: Loop safety, test operators, CRITICAL security
- Batch 4: Variable safety, redirection, CRITICAL dangerous rm
- Batch 5: Command optimization, tr classes, CRITICAL word splitting
- Batch 6: File iteration safety, unused vars, function exports
- **Batch 7**: Test operator safety, find patterns, expansion rules

## Expected Impact

**User Value**:
- Catches **test operator issues** (SC2137-2143) - very common in conditionals
- Prevents **find pattern mistakes** (SC2144-2150) - common in file operations
- Improves **expansion safety** (parameter expansion issues)
- Better **alias and function handling** (SC2139, SC2142)

**Coverage Improvement**:
- From 140 rules (39.2%) to **160 rules (44.8%)**
- **+20 rules (+5.6 percentage points)** in single batch
- Achieves 45% target for v6.28.0 release

## Next Steps

1. ✅ Create this analysis document (batch 7 plan)
2. ⏳ Verify which rules are already implemented (check for duplicates)
3. ⏳ RED Phase: Add rules to rule_registry.rs with comprehensive tests
4. ⏳ GREEN Phase: Verify all registry tests pass
5. ⏳ REFACTOR Phase: Verify rule implementations exist
6. ⏳ QUALITY Phase: Run tests, verify clippy clean
7. ⏳ Commit batch 7 and push
8. ⏳ Consider v6.28.0-beta release at 45% coverage

## Notes

- Focus on **test operator safety** (very common issue in shell scripts)
- SC2137-2143 form a cohesive group about proper test usage
- SC2144-2150 complete the find/glob safety coverage
- Conservative: If uncertain about classification, default to Universal
- Some rules may already be classified - need to verify before adding

## Quality Target

- **100% test pass rate** (zero regressions)
- **Clippy clean** (zero warnings)
- **Complexity <10** (all functions)
- **EXTREME TDD** (RED → GREEN → REFACTOR → QUALITY)
