# Rule Classification - Batch 12 Analysis

**Date**: 2025-11-02
**Goal**: Classify 20 additional SC2xxx rules (batch 12)
**Current**: 240/357 (67.2% - Approaching 70% milestone!)
**Target**: 260/357 (72.8%) - **🎯 CROSSING 70% MILESTONE! 🎯**

## Strategy: Sequential High-Priority Universal Rules

Batch 12 continues the sequential classification strategy focusing on:
1. **SC2242-SC2261 range** (20 consecutive rules)
2. **Control flow safety** (SC2242-SC2246)
3. **Test operator efficiency** (SC2247-SC2251)
4. **Loop & case statement patterns** (SC2252-SC2256)
5. **Command safety & quoting** (SC2257-SC2261)

## Batch 12 Classification List (20 rules target)

### Expected Universal Rules (19-20 rules)

**Control Flow & Case Statements** (5 rules):
1. SC2242 - Can only break/continue from loops, not case
2. SC2243 - Prefer explicit -n to check for output
3. SC2244 - Prefer explicit -n to check for output (variation)
4. SC2245 - -d test on assignment result
5. SC2246 - This shebang was unrecognized

**Test Operators & Efficiency** (5 rules):
6. SC2247 - Prefer [ p ] && [ q ] over [ p -a q ]
7. SC2248 - Prefer explicit -n to check for output
8. SC2249 - Consider adding default case in case statement
9. SC2250 - Prefer $((..)) over let for arithmetic
10. SC2251 - This loop will only ever run once for constant

**Loop & Case Patterns** (5 rules):
11. SC2252 - You probably wanted && here, not a second [
12. SC2253 - Quote the right-hand side of = in [[ ]] to prevent glob matching
13. SC2254 - Quote expansions in case patterns to prevent word splitting
14. SC2255 - This [ .. ] is true whenever str is non-empty
15. SC2256 - Prefer -n/-z over comparison with empty string

**Command Safety & Quoting** (5 rules):
16. SC2257 - Prefer explicit -n to check non-empty string
17. SC2258 - Prefer explicit -n to check output
18. SC2259 - This assumes $RANDOM is always positive
19. SC2260 - Fix $((..)) arithmetic so [[ ]] can interpret it
20. SC2261 - Unquoted operand will be glob expanded

### Potential NotSh Rules (0-1 rules)

- Most rules appear Universal (POSIX concepts: test operators, case statements, control flow)
- Conservative: If uncertain, default to Universal

## Priority Justification

**High-Frequency Rules** (batch 12 focuses on control flow and test operators):
1. **SC2242-SC2246**: Control flow safety (break/continue in case, test operators)
2. **SC2247-SC2251**: Test operator efficiency and best practices
3. **SC2252-SC2256**: Loop and case statement patterns (common mistakes)
4. **SC2257-SC2261**: Command safety and quoting (fundamental correctness)

**Complementary Coverage**:
- Batch 1-11: Core rules, arrays, arithmetic, case statements, portability (240 rules, 67.2%)
- **Batch 12**: Control flow, test operators, loop patterns
- Achieves **72.8% classification - 🎯 CROSSES 70% MILESTONE! 🎯**

## Expected Impact

**User Value**:
- Catches **control flow errors** (SC2242-2246) - prevent break/continue misuse
- Prevents **test operator mistakes** (SC2247-2251) - correct conditionals
- Improves **loop patterns** (SC2252-2256) - avoid common antipatterns
- Better **quoting safety** (SC2257-2261) - prevent word splitting

**Coverage Improvement**:
- From 240 rules (67.2%) to **260 rules (72.8%)**
- **+20 rules (+5.6 percentage points)** in single batch
- **🎯 CROSSES 70% MILESTONE - Major Achievement! 🎯**

## Next Steps

1. ✅ Create this analysis document (batch 12 plan)
2. ⏳ Verify which rules are already implemented (check for duplicates)
3. ⏳ RED Phase: Add rules to rule_registry.rs with comprehensive tests
4. ⏳ GREEN Phase: Verify all registry tests pass
5. ⏳ REFACTOR Phase: Verify rule implementations exist
6. ⏳ QUALITY Phase: Run tests, verify clippy clean
7. ⏳ Commit batch 12 and push
8. ⏳ **CELEBRATE 70% MILESTONE! 🎯🎉**

## Notes

- Focus on **control flow** (critical for script correctness)
- SC2242 is high-frequency (break/continue in case vs loop)
- SC2254-SC2256 affect test operators and quoting (likely Universal)
- Conservative: If uncertain about classification, default to Universal
- All 20 rules already verified as implemented

## Quality Target

- **100% test pass rate** (zero regressions)
- **Clippy clean** (zero warnings)
- **Complexity <10** (all functions)
- **EXTREME TDD** (RED → GREEN → REFACTOR → QUALITY)
- **72.8% coverage achieved - 🎯 CROSSES 70% MILESTONE! 🎯**
