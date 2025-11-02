# Rule Classification - Batch 4 Analysis

**Date**: 2025-11-02
**Goal**: Classify 28-40 additional SC2xxx rules (batch 4)
**Current**: 72/357 (20.2%)
**Target**: 100-112/357 (28.0-31.4%)

## Strategy: Complete Commonly-Encountered Rule Coverage

Batch 4 aims to reach the **28-34% milestone** for v6.28.0 release by focusing on:
1. **Frequently flagged rules** in real-world scripts
2. **Syntax validation rules** (quotes, redirections, operators)
3. **Portability rules** (POSIX vs bash-specific constructs)
4. **Remaining high-value Universal rules**

## Batch 4 Classification List (30 rules target)

### Universal Rules (Estimated 25-27 rules)

**Variable and Parameter Safety** (8 rules):
1. SC2067 - Missing $ on array lookup (${array[$index]} vs ${array[index]})
2. SC2068 - Quote $@ to prevent word splitting
3. SC2069 - To redirect stdout+stderr, use &> or 2>&1, not 1>&2
4. SC2070 - -n doesn't work with unquoted arguments (use [[ ]] or quotes)
5. SC2071 - Arithmetic operators in [ ] (use [[ ]] or (( )))
6. SC2072 - Lexicographic comparison in [ ] (use -lt/-gt for numbers)
7. SC2073 - Escape \d in character class (use [[:digit:]])
8. SC2074 - Can't use =~ in [ ]. Use [[ ]] instead

**Quote and Expansion Safety** (7 rules):
9. SC2075 - Escaping quotes in single quotes doesn't work
10. SC2076 - Don't quote RHS of =~ in [[ ]]
11. SC2077 - Quote regex argument to prevent word splitting
12. SC2078 - This expression is constant (did you forget $ on variable?)
13. SC2081 - Escape [ in globs (or use [[ ]])
14. SC2082 - Variable indirection with $$ (use ${!var})
15. SC2083 - Don't add spaces after shebang (#! /bin/sh → #!/bin/sh)

**Command and Redirection Safety** (6 rules):
16. SC2094 - Don't use same file for input and output (will truncate)
17. SC2095 - ssh -t/-T in loops may consume stdin
18. SC2096 - Use #! shebang, not just # comment
19. SC2097 - Assign and use variable separately (VAR=val echo $VAR doesn't work)
20. SC2098 - Variable assignment vs redirection confusion
21. SC2103 - cd without error check (use cd ... || exit)

**Test and Conditional Safety** (4 rules):
22. SC2104 - In [[ ]], == is literal. Use = or [[
23. SC2105 - Break outside loop
24. SC2107 - Instead of [ a -o b ], use [ a ] || [ b ]
25. SC2114 - Dangerous rm -rf without validation ($VAR might be empty)

**Function and Scope Safety** (2 rules):
26. SC2115 - Use "${var:?}" to ensure var is set before rm -rf
27. SC2116 - Useless echo $(cmd) - just use cmd

### NotSh Rules (Estimated 3-5 rules)

**Bash/Zsh-Specific Constructs**:
28. SC2097 - Assign and export separately in sh (export VAR=val requires bash)
29. SC2120 - Function references $1 but none passed (requires call-site analysis)
30. SC2128 - Expanding array without index in bash

## Classification Rationale

### Universal Rules (25-27 rules)
These apply to all POSIX shells and represent:
- **Variable safety**: Missing $, word splitting, array lookups
- **Quote safety**: Escaping, expansion, glob patterns
- **Redirection safety**: Input/output, file truncation
- **Conditional safety**: Test operators, break/continue
- **Critical safety**: SC2114/SC2115 (dangerous rm -rf without validation)

### NotSh Rules (3-5 rules)
- **SC2097**: export VAR=val is bash-specific (sh requires separate assign + export)
- **SC2120**: Function parameter analysis (bash arrays/functions)
- **SC2128**: Array expansion without index (bash-specific)

## Priority Justification

**High-Frequency Rules** (batch 4 focuses on common real-world issues):
1. **SC2068**: Quote $@ (extremely common in scripts handling arguments)
2. **SC2094**: Same file for input/output (common mistake, causes data loss)
3. **SC2103**: cd without error check (very common in build scripts)
4. **SC2114/SC2115**: Dangerous rm -rf (CRITICAL safety, prevents disasters)
5. **SC2116**: Useless echo $(cmd) (common anti-pattern)

**Complementary Coverage**:
- Batch 1: SEC, DET, IDEM + bash-isms (arrays, [[]], process substitution)
- Batch 2: Arithmetic, function keyword, quoting basics
- Batch 3: Loop safety, test operators, CRITICAL security (SC2059, SC2064)
- **Batch 4**: Variable safety, redirection, dangerous commands, POSIX portability

## Expected Impact

**User Value**:
- Catches **dangerous rm -rf** patterns (SC2114/SC2115) - prevents data loss
- Prevents **file truncation** bugs (SC2094) - common in pipelines
- Improves **argument handling** (SC2068) - critical for robust scripts
- Enforces **cd safety** (SC2103) - prevents operations in wrong directory

**Coverage Improvement**:
- From 72 rules (20.2%) to **100-112 rules (28.0-31.4%)**
- **+28-40 rules (+7.8-11.2 percentage points)** in single batch
- Achieves target for v6.28.0 release (28-34% coverage)

## Next Steps

1. ✅ Create this analysis document (batch 4 plan)
2. ⏳ RED Phase: Add 30 rules to rule_registry.rs with comprehensive tests
3. ⏳ GREEN Phase: Verify all registry tests pass
4. ⏳ REFACTOR Phase: Expand lint_shell_filtered() with batch 4 rules
5. ⏳ PROPERTY Phase: Add integration tests for critical rules (SC2114/SC2115)
6. ⏳ MUTATION Phase: Run mutation testing (target ≥90% kill rate)
7. ⏳ Update CHANGELOG.md and ROADMAP.yaml
8. ⏳ Commit batch 4 and consider v6.28.0-alpha release (28-31% coverage)

## Notes

- Batch 4 emphasizes **safety** (SC2114/SC2115 dangerous rm, SC2094 file truncation)
- Focus on **real-world pain points** (cd without checks, argument handling)
- Conservative: If uncertain about classification, default to Universal
- SC2097 has dual nature: Universal concern but sh requires different syntax
- SC2120 requires call-site analysis (may defer if too complex)

## Quality Target

- **100% test pass rate** (zero regressions)
- **Clippy clean** (zero warnings)
- **Complexity <10** (all functions)
- **EXTREME TDD** (RED → GREEN → REFACTOR → PROPERTY → MUTATION)
- **Mutation score ≥90%** on new code
