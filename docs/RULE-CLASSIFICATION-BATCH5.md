# Rule Classification - Batch 5 Analysis

**Date**: 2025-11-02
**Goal**: Classify 20-25 additional SC2xxx rules (batch 5)
**Current**: 100/357 (28.0%)
**Target**: 120-125/357 (33.6-35.0%)

## Strategy: Remaining High-Value Universal Rules

Batch 5 aims to reach the **34% milestone** for v6.28.0 release by focusing on:
1. **Remaining high-frequency Universal rules** (SC2001-2037 range)
2. **Missing core safety rules** (quoting, expansion, command execution)
3. **Bash-specific array/test rules** (NotSh classification)
4. **Style and best practice rules** (Universal)

## Batch 5 Classification List (20-25 rules target)

### Universal Rules (Estimated 18-22 rules)

**Missing Core Rules** (10 rules):
1. SC2001 - Use ${var//pattern/replacement} instead of sed (for simple substitutions)
2. SC2002 - Already classified as NotSh (useless cat - process substitution)
3. SC2010 - Don't use ls | grep (use wildcards or find)
4. SC2011 - Use while read to iterate over find output
5. SC2012 - Use find instead of ls to better handle non-alphanumeric filenames
6. SC2013 - To read lines rather than words, pipe/redirect to while read loop
7. SC2014 - This doesn't read from the command; use < or pipe instead
8. SC2027 - Quote or escape $ in double quotes
9. SC2028 - echo may not expand \\n (use printf)
10. SC2029 - Note: variable must be local in remote SSH command

**ShellCheck Best Practices** (8-12 rules):
11. SC2033 - Shell functions can't be exported (use scripts or ENV)
12. SC2034 - Variable appears unused (verify with shellcheck)
13. SC2035 - Use ./*glob* or -- *glob* to match files starting with -
14. SC2086 - **CRITICAL**: Quote to prevent word splitting and globbing
15. SC2099 - Use $(...) instead of backticks (already done? check)
16. SC2100 - Use $((..)) instead of expr (already done? check)
17. SC2101 - Named POSIX class needs outer [] (already done? check)
18. SC2102 - Ranges only work with single chars (already done? check)
19. SC2106 - Consider using pgrep (already done? check)
20. SC2117 - Unreachable code after exit/return (already done? check)

### NotSh Rules (Estimated 2-3 rules)

**Bash-Specific Constructs**:
21. SC2076 - Don't quote RHS of =~ in [[ ]] (already done? check)
22. More array-related rules if not yet classified
23. Process substitution rules if not yet classified

## Priority Justification

**High-Frequency Rules** (batch 5 focuses on missing core safety):
1. **SC2086**: Quote to prevent word splitting (EXTREMELY common, CRITICAL)
2. **SC2001**: sed vs parameter expansion (very common optimization)
3. **SC2010-2014**: ls/find safety (common mistakes in loops)
4. **SC2027-2029**: Quoting and echo safety (common pitfalls)
5. **SC2033-2035**: Export, unused vars, glob safety

**Complementary Coverage**:
- Batch 1: SEC, DET, IDEM + bash-isms (arrays, [[]], process substitution)
- Batch 2: Arithmetic, function keyword, quoting basics
- Batch 3: Loop safety, test operators, CRITICAL security (SC2059, SC2064)
- Batch 4: Variable safety, redirection, CRITICAL dangerous rm (SC2114/SC2115)
- **Batch 5**: Missing core Universal rules (word splitting, sed vs param expansion, ls/find safety)

## Expected Impact

**User Value**:
- Catches **word splitting bugs** (SC2086) - extremely common, CRITICAL safety
- Prevents **ls | grep** anti-patterns (SC2010-2014) - common in loops
- Improves **quoting safety** (SC2027-2029) - prevents injection
- Enforces **best practices** (sed → param expansion, echo → printf)

**Coverage Improvement**:
- From 100 rules (28.0%) to **120-125 rules (33.6-35.0%)**
- **+20-25 rules (+5.6-7.0 percentage points)** in single batch
- Achieves 34% target for v6.28.0 release

## Next Steps

1. ✅ Create this analysis document (batch 5 plan)
2. ⏳ Verify which rules are already implemented (check for duplicates)
3. ⏳ RED Phase: Add rules to rule_registry.rs with comprehensive tests
4. ⏳ GREEN Phase: Verify all registry tests pass
5. ⏳ REFACTOR Phase: Expand lint_shell_filtered() with batch 5 rules
6. ⏳ QUALITY Phase: Run tests, verify clippy clean
7. ⏳ Commit batch 5 and push
8. ⏳ Consider v6.28.0-alpha release at 34% coverage

## Notes

- **CRITICAL**: SC2086 (quote to prevent word splitting) is EXTREMELY common and should be Universal
- Batch 5 emphasizes **missing core safety** that should have been in earlier batches
- Focus on **real-world pain points** (ls loops, word splitting, sed usage)
- Conservative: If uncertain about classification, default to Universal
- Some rules may already be classified - need to verify before adding

## Quality Target

- **100% test pass rate** (zero regressions)
- **Clippy clean** (zero warnings)
- **Complexity <10** (all functions)
- **EXTREME TDD** (RED → GREEN → REFACTOR → QUALITY)
- **Mutation score ≥90%** on new code
