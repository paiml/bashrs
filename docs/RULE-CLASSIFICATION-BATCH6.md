# Rule Classification - Batch 6 Analysis

**Date**: 2025-11-02
**Goal**: Classify 20 additional SC2xxx rules (batch 6)
**Current**: 120/357 (33.6%)
**Target**: 140/357 (39.2%)

## Strategy: Remaining Implemented Universal Rules

Batch 6 aims to reach the **39% milestone** for v6.28.0 release by focusing on:
1. **Remaining implemented Universal rules** (SC2033-2037, SC2010-2014)
2. **File iteration safety** (ls | grep anti-patterns)
3. **Variable and export safety** (unused vars, shell functions)
4. **Best practices** (glob safety, command substitution)

## Batch 6 Classification List (20 rules target)

### Universal Rules (Estimated 18-20 rules)

**Shell Function and Variable Safety** (5 rules):
1. SC2033 - Shell functions can't be exported (use scripts or ENV)
2. SC2034 - Variable appears unused (verify with shellcheck)
3. SC2035 - Use ./*glob* or -- *glob* to match files starting with -
4. SC2036 - Unescaped quotes in backticks (old-style command substitution)
5. SC2037 - Redirect to variable vs command substitution

**File Iteration Safety** (5 rules):
6. SC2010 - Don't use ls | grep (use wildcards or find)
7. SC2011 - Use while read to iterate over find output
8. SC2012 - Use find instead of ls to better handle non-alphanumeric filenames
9. SC2013 - To read lines rather than words, pipe/redirect to while read loop
10. SC2014 - This doesn't read from the command; use < or pipe instead

**Command Best Practices** (5 rules):
11. SC2099 - Use $(...) instead of backticks (already done? check)
12. SC2100 - Use $((..)) instead of expr (already done? check)
13. SC2101 - Named POSIX class needs outer [] (already done? check)
14. SC2102 - Ranges only work with single chars (already done? check)
15. SC2106 - Consider using pgrep (already done? check)

**Additional Safety Rules** (5 rules):
16. SC2117 - Unreachable code after exit/return (already done? check)
17-20. Other high-priority rules from batch 5 planning if needed

### NotSh Rules (Estimated 0-2 rules)

No obvious NotSh rules in this batch - focus on Universal coverage.

## Priority Justification

**High-Frequency Rules** (batch 6 focuses on file iteration safety):
1. **SC2010-2014**: ls | grep and find safety (common mistakes in scripts)
2. **SC2033-2037**: Variable/function/quote safety (export confusion, unused vars)
3. **SC2035**: Glob safety for files starting with - (security issue)

**Complementary Coverage**:
- Batch 1: SEC, DET, IDEM + bash-isms
- Batch 2: Arithmetic, function keyword, quoting basics
- Batch 3: Loop safety, test operators, CRITICAL security
- Batch 4: Variable safety, redirection, CRITICAL dangerous rm
- Batch 5: Command optimization, tr classes, CRITICAL word splitting
- **Batch 6**: File iteration safety, unused vars, function exports

## Expected Impact

**User Value**:
- Catches **ls | grep anti-patterns** (SC2010-2014) - very common in legacy scripts
- Prevents **unused variable bugs** (SC2034) - helps catch typos
- Enforces **glob safety** (SC2035) - prevents security issues with filenames starting with -
- Improves **function export understanding** (SC2033) - common shell misconception

**Coverage Improvement**:
- From 120 rules (33.6%) to **140 rules (39.2%)**
- **+20 rules (+5.6 percentage points)** in single batch
- Achieves 39% target for v6.28.0 release

## Next Steps

1. ✅ Create this analysis document (batch 6 plan)
2. ⏳ Verify which rules are already implemented (check for duplicates)
3. ⏳ RED Phase: Add rules to rule_registry.rs with comprehensive tests
4. ⏳ GREEN Phase: Verify all registry tests pass
5. ⏳ REFACTOR Phase: Expand lint_shell_filtered() with batch 6 rules
6. ⏳ QUALITY Phase: Run tests, verify clippy clean
7. ⏳ Commit batch 6 and push
8. ⏳ Consider v6.28.0-alpha release at 39% coverage

## Notes

- Focus on **file iteration safety** (very common issue in shell scripts)
- SC2010-2014 form a cohesive group about proper file handling
- SC2033-2037 complete the variable/function safety coverage
- Conservative: If uncertain about classification, default to Universal
- Some rules may already be classified - need to verify before adding

## Quality Target

- **100% test pass rate** (zero regressions)
- **Clippy clean** (zero warnings)
- **Complexity <10** (all functions)
- **EXTREME TDD** (RED → GREEN → REFACTOR → QUALITY)
