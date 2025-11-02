# Rule Classification - Batch 3 Analysis

**Date**: 2025-11-02
**Goal**: Classify 25-30 additional SC2xxx rules (batch 3)
**Current**: 45/357 (12.6%)
**Target**: 70-75/357 (19.6-21.0%)

## Strategy: Focus on High-Frequency Universal Rules

Batch 3 prioritizes **commonly encountered rules** that apply universally, maximizing immediate value for users.

## Batch 3 Classification List (30 rules)

### Universal Rules (28 rules)

**Loop and Iteration Safety** (5 rules):
1. SC2038 - Use -print0/-0 or find -exec instead of for loop over find
2. SC2040 - Avoid passing -o to other commands (shell option confusion)
3. SC2041 - Use while read, not read in for loop
4. SC2042 - Use printf instead of echo with backslash escapes
5. SC2043 - This loop will only run once (for x in y without wildcards)

**Test Operators and Conditionals** (8 rules):
6. SC2044 - For loops over find: use find -exec or process substitution
7. SC2045 - Iterating over ls output is fragile
8. SC2046 - Quote to prevent word splitting (CRITICAL - already in linter, add to registry)
9. SC2047 - Quote variables in [ ] to prevent word splitting
10. SC2048 - Use "$@" (with quotes) to prevent word splitting
11. SC2049 - Use =~ for regex matching (not = in [ ])
12. SC2050 - This expression is constant (forgot $ on variable?)
13. SC2051 - Bash doesn't expand variables in brace ranges {$a..$b}

**Quoting and Glob Safety** (7 rules):
14. SC2052 - Use [[ ]] instead of [ ] for glob patterns (wait - NotSh!)
15. SC2053 - Quote RHS of = in [ ] to prevent glob matching
16. SC2054 - Comma is just literal in [[ ]]; use array or separate comparison
17. SC2055 - Deprecated -a operator in test (use &&)
18. SC2056 - Deprecated -o operator in test (use ||)
19. SC2057 - Unknown binary operator (===, =!, <>)
20. SC2058 - Unknown unary operator in test

**Command Safety and Redirection** (4 rules):
21. SC2059 - Printf format string injection (CRITICAL security)
22. SC2060 - Unquoted tr parameters (glob expansion)
23. SC2061 - Quote parameters to tr to prevent globbing
24. SC2062 - Grep pattern glob expansion prevention

**Trap and Signal Handling** (4 rules):
25. SC2063 - Grep regex vs literal string matching
26. SC2064 - Trap command quoting (CRITICAL - timing bug)
27. SC2065 - Shell redirection interpretation in strings
28. SC2066 - Missing semicolon before done in for loop

### NotSh Rules (2 rules)

**Glob Patterns** (1 rule):
29. SC2052 - Use [[ ]] for glob patterns (bash/zsh, not POSIX sh)

**Process Substitution** (1 rule):
30. SC2044 - For loops over find (suggests process substitution)

## Classification Rationale

### Universal Rules (28 rules)
These apply to all POSIX shells:
- Loop safety (for, while, find usage)
- Test operators ([ ], deprecated -a/-o)
- Quoting rules (word splitting prevention)
- Security rules (SC2059 format injection, SC2064 trap timing)
- Portability rules (echo vs printf)

### NotSh Rules (2 rules)
- SC2052: [[ ]] is bash/zsh/ksh only
- SC2044: Process substitution <(...) is bash/zsh/ksh only

## Priority Justification

**High-Frequency Rules** (batch 3 focuses on common issues):
1. **SC2046**: Quote to prevent word splitting (CRITICAL, extremely common)
2. **SC2059**: Printf format injection (SECURITY CRITICAL)
3. **SC2064**: Trap timing bug (CRITICAL, subtle bug)
4. **SC2038**: Find loop safety (common mistake)
5. **SC2045**: Iterating over ls (common anti-pattern)

**Complementary Coverage**:
- Batch 1: SEC, DET, IDEM + arrays/[[ ]] basics
- Batch 2: Arithmetic, function keyword, more quoting
- **Batch 3**: Loop safety, test operators, critical security rules

## Expected Impact

**User Value**:
- Eliminates false positives on POSIX sh scripts
- Catches critical security issues (SC2059, SC2064)
- Prevents common mistakes (SC2046, SC2038, SC2045)

**Coverage Improvement**:
- From 45 rules (12.6%) to 75 rules (21.0%)
- **+8.4 percentage points** in single batch
- Approaches target of 20-28% for v6.28.0 release

## Next Steps

1. ✅ Create this analysis document
2. ⏳ Add 30 rules to rule_registry.rs (RED phase)
3. ⏳ Add tests for batch 3 classifications
4. ⏳ Expand lint_shell_filtered() with batch 3 rules
5. ⏳ Add integration tests
6. ⏳ Run mutation testing (target ≥90% kill rate)
7. ⏳ Update CHANGELOG.md and ROADMAP.yaml
8. ⏳ Commit and tag as approaching v6.28.0 release threshold

## Notes

- SC2046 is already in the linter, we're just adding it to the registry for filtering
- Batch 3 emphasizes **security** (SC2059, SC2064) and **common mistakes** (SC2046, SC2038, SC2045)
- Conservative: If uncertain about classification, default to Universal
- Focus on value: Rules users will encounter frequently
