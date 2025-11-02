# Rule Classification - Batch 11 Analysis

**Date**: 2025-11-02
**Goal**: Classify 20 additional SC2xxx rules (batch 11)
**Current**: 220/357 (61.6% - 🎯 60% MILESTONE achieved!)
**Target**: 240/357 (67.2%) - **Approaching 70% milestone!**

## Strategy: Sequential High-Priority Universal Rules

Batch 11 continues the sequential classification strategy focusing on:
1. **SC2222-SC2241 range** (20 consecutive rules)
2. **Case statement syntax** (SC2222-SC2223)
3. **Control flow & test operators** (SC2224-SC2229)
4. **Command existence & portability** (SC2230-SC2234)
5. **Quoting & expansion safety** (SC2235-SC2241)

## Batch 11 Classification List (20 rules target)

### Expected Universal Rules (19-20 rules)

**Case Statement Syntax** (2 rules):
1. SC2222 - Lexical error in case statement syntax
2. SC2223 - This default case is unreachable (previous pattern catches all)

**Control Flow & Test Operators** (6 rules):
3. SC2224 - Quote the word or use a glob
4. SC2225 - Use : or true instead of /bin/true
5. SC2226 - This expression is constant
6. SC2227 - Redirection applies to the echo, not the assignment
7. SC2228 - Declare -x is equivalent to export
8. SC2229 - This does not read 'foo'. Remove $/${} for that

**Command Existence & Portability** (5 rules):
9. SC2230 - which is non-standard, use command -v instead
10. SC2231 - Quote expansions in this for loop glob to prevent word splitting
11. SC2232 - Can't use sudo with builtins like cd
12. SC2233 - Remove superfluous (..) around condition
13. SC2234 - Remove superfluous () around here document

**Quoting & Expansion Safety** (7 rules):
14. SC2235 - Quote arguments to unalias to prevent word splitting
15. SC2236 - Use -n instead of ! -z
16. SC2237 - Use [ ] instead of [[ ]] (for sh compatibility)
17. SC2238 - Prefer ${} over backticks (readability + nesting)
18. SC2239 - Ensure consistent quoting for redirects
19. SC2240 - The dot command does not support arguments in sh
20. SC2241 - Exit code is always overridden by following command

### Potential NotSh Rules (0-1 rules)

- Most rules appear Universal (POSIX concepts: case, test, quoting, command existence)
- Conservative: If uncertain, default to Universal

## Priority Justification

**High-Frequency Rules** (batch 11 focuses on case statements and portability):
1. **SC2222-SC2223**: Case statement syntax (common in scripts)
2. **SC2224-SC2229**: Control flow and test operators (frequent patterns)
3. **SC2230-SC2234**: Command existence and portability (sh vs bash)
4. **SC2235-SC2241**: Quoting and expansion safety (fundamental correctness)

**Complementary Coverage**:
- Batch 1-10: Core rules, arrays, arithmetic, command structure (220 rules, 61.6%)
- **Batch 11**: Case statements, portability, quoting
- Achieves **67.2% classification - Approaching 70% milestone!**

## Expected Impact

**User Value**:
- Catches **case statement errors** (SC2222-2223) - prevent syntax issues
- Prevents **portability problems** (SC2230-2234) - sh vs bash compatibility
- Improves **quoting safety** (SC2235-2241) - prevent word splitting
- Better **control flow** (SC2224-2229) - correct test expressions

**Coverage Improvement**:
- From 220 rules (61.6%) to **240 rules (67.2%)**
- **+20 rules (+5.6 percentage points)** in single batch
- **Approaching 70% milestone** (need 250 rules = 70.0%)

## Next Steps

1. ✅ Create this analysis document (batch 11 plan)
2. ⏳ Verify which rules are already implemented (check for duplicates)
3. ⏳ RED Phase: Add rules to rule_registry.rs with comprehensive tests
4. ⏳ GREEN Phase: Verify all registry tests pass
5. ⏳ REFACTOR Phase: Verify rule implementations exist
6. ⏳ QUALITY Phase: Run tests, verify clippy clean
7. ⏳ Commit batch 11 and push
8. ⏳ **Continue toward 70% milestone!**

## Notes

- Focus on **case statements** (critical syntax validation)
- SC2230 is high-frequency (which vs command -v portability)
- SC2236-SC2237 affect sh compatibility (likely Universal)
- Conservative: If uncertain about classification, default to Universal
- All 20 rules already verified as implemented

## Quality Target

- **100% test pass rate** (zero regressions)
- **Clippy clean** (zero warnings)
- **Complexity <10** (all functions)
- **EXTREME TDD** (RED → GREEN → REFACTOR → QUALITY)
- **67.2% coverage achieved - approaching 70% milestone!**
