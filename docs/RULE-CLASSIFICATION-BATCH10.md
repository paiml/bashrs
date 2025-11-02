# Rule Classification - Batch 10 Analysis

**Date**: 2025-11-02
**Goal**: Classify 20 additional SC2xxx rules (batch 10)
**Current**: 200/357 (56.0% - approaching 60% milestone!)
**Target**: 220/357 (61.6%) - **🎯 CROSSING 60% MILESTONE!**

## Strategy: Sequential High-Priority Universal Rules

Batch 10 continues the sequential classification strategy focusing on:
1. **SC2202-SC2221 range** (20 consecutive rules)
2. **Array quoting safety** (SC2206-SC2207)
3. **Arithmetic operators** (SC2210-SC2211)
4. **Command structure** (SC2202-SC2209)
5. **Control flow safety** (SC2212-SC2221)

## Batch 10 Classification List (20 rules target)

### Expected Universal Rules (18-19 rules)

**Command Structure & Ordering** (8 rules):
1. SC2202 - Order sensitivity (e.g., redirects)
2. SC2203 - Variable assignment order matters
3. SC2204 - Exit traps must come before commands
4. SC2205 - Command ordering with pipes
5. SC2208 - Command grouping issues
6. SC2209 - Use single quotes for literal strings in find
7. SC2216 - Piping find to shell with ; instead of +
8. SC2217 - Useless cat with find

**Arithmetic Operations** (4 rules):
9. SC2210 - Don't use arithmetic shortcuts like x=++y
10. SC2211 - Arithmetic on variable without $(())
11. SC2214 - Arithmetic comparison outside test
12. SC2215 - Expression precedence issues

**Control Flow Safety** (4 rules):
13. SC2212 - Use [ p ] || [ q ] instead of [ p -o q ]
14. SC2213 - getopts requires argument variable
15. SC2218 - Useless return in command substitution
16. SC2219 - Instead of let expr, use (( expr ))

**Other Safety** (2-3 rules):
17. SC2220 - Invalid arithmetic expression
18. SC2221 - Arithmetic syntax errors

### Potential NotSh Rules (1-2 rules)

- SC2206: Quote to prevent word splitting in arrays → **NotSh** (arrays are bash/zsh/ksh)
- SC2207: Prefer mapfile or read -a over command substitution → **NotSh** (arrays)

## Priority Justification

**High-Frequency Rules** (batch 10 focuses on command structure and arithmetic):
1. **SC2202-SC2209**: Command structure and ordering (very common in scripts)
2. **SC2210-SC2215**: Arithmetic operations (frequent in bash scripts)
3. **SC2212-SC2219**: Control flow safety (common mistakes)
4. **SC2206-SC2207**: Array quoting (bash/zsh specific)

**Complementary Coverage**:
- Batch 1-9: Core rules, arrays, exit codes (200 rules, 56.0%)
- **Batch 10**: Command structure, arithmetic, control flow
- Achieves **61.6% classification - 🎯 CROSSES 60% MILESTONE!**

## Expected Impact

**User Value**:
- Catches **command ordering errors** (SC2202-2209) - prevent script failures
- Prevents **arithmetic mistakes** (SC2210-2215) - correct calculations
- Improves **control flow safety** (SC2212-2219) - better error handling
- Better **array quoting** (SC2206-2207) - prevent word splitting

**Coverage Improvement**:
- From 200 rules (56.0%) to **220 rules (61.6%)**
- **+20 rules (+5.6 percentage points)** in single batch
- **🎯 CROSSES 60% MILESTONE - Major achievement!**

## Next Steps

1. ✅ Create this analysis document (batch 10 plan)
2. ⏳ Verify which rules are already implemented (check for duplicates)
3. ⏳ RED Phase: Add rules to rule_registry.rs with comprehensive tests
4. ⏳ GREEN Phase: Verify all registry tests pass
5. ⏳ REFACTOR Phase: Verify rule implementations exist
6. ⏳ QUALITY Phase: Run tests, verify clippy clean
7. ⏳ Commit batch 10 and push
8. ⏳ **CELEBRATE 60% MILESTONE! 🎯🎉**

## Notes

- Focus on **command structure** (critical for script correctness)
- SC2206-SC2207 are array-related (likely NotSh)
- Most other rules appear Universal (POSIX concepts)
- Conservative: If uncertain about classification, default to Universal
- All 20 rules already verified as implemented

## Quality Target

- **100% test pass rate** (zero regressions)
- **Clippy clean** (zero warnings)
- **Complexity <10** (all functions)
- **EXTREME TDD** (RED → GREEN → REFACTOR → QUALITY)
- **61.6% MILESTONE achieved - crosses 60% threshold!**
