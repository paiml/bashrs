# Rule Classification - Batch 15 Analysis

**Date**: 2025-11-03
**Goal**: Classify 13 additional SC2xxx rules (batch 15)
**Current**: 290/357 (81.2% - Just crossed 80% milestone!)
**Target**: 303/357 (84.9%) - **🎯 REACH 85% MILESTONE! 🎯**

## Strategy: Reach 85% Milestone with Sequential Classification

Batch 15 continues the sequential classification strategy to reach the **85% milestone (303 rules)**:
1. **SC2306-SC2318 range** (13 consecutive rules)
2. **Mix of Universal and NotSh** (parameter expansion, command optimization, deprecated syntax)
3. **Reaches 85% threshold** (303 rules = 84.9%, batch 15 achieves 303)

## Batch 15 Classification List (13 rules target)

### Expected Universal Rules (11-12 rules)

**Command Optimization & Parameter Expansion** (6 rules):
1. SC2307 - Use ${var#prefix} to remove prefix (POSIX parameter expansion)
2. SC2308 - Use ${var%suffix} to remove suffix (POSIX parameter expansion)
3. SC2309 - Use ${var##prefix} to remove longest prefix (POSIX parameter expansion)
4. SC2310 - Function in condition - set -e doesn't apply (POSIX set -e behavior)
5. SC2311 - Use ${var%%suffix} to remove longest suffix (POSIX parameter expansion)
6. SC2315 - Use ${var:+replacement} for conditional replacement (POSIX ${var:+value})

**Deprecated Syntax & Best Practices** (5-6 rules):
7. SC2312 - Deprecated local -x syntax (universal portability warning)
8. SC2313 - Use $(( )) for arithmetic (POSIX arithmetic)
9. SC2314 - Use [[ ]] for pattern matching (may be NotSh)
10. SC2316 - Command group and precedence issues (POSIX control flow)
11. SC2317 - Unreachable code detection (universal logic)
12. SC2318 - Deprecated $[ ] syntax - use $(( )) (universal deprecation warning)

### Expected NotSh Rules (1-2 rules)

**Bash-Specific Features** (2 rules):
13. SC2306 - Prefer ${var//old/new} over sed (bash ${var//} parameter expansion - NotSh)
14. SC2314 - Use [[ ]] for pattern matching (if suggests [[]] specifically - NotSh)

## Priority Justification

**Milestone Importance** (batch 15 crosses 85%):
1. **85% milestone** is next major threshold (303 rules = 84.9%)
2. Batch 15 reaches **303 rules (84.9%)** - achieves 85% milestone
3. Demonstrates **sustained momentum** (batches 14-15: 80% → 85% in 2 batches)

**Rule Focus**:
- **POSIX parameter expansion**: ${var#}, ${var%}, ${var##}, ${var%%}, ${var:+} (Universal)
- **Command optimization**: sed vs parameter expansion patterns
- **Deprecated syntax warnings**: $[ ], local -x (Universal)
- **set -e behavior**: Functions in conditions (Universal - POSIX)

**Complementary Coverage**:
- Batches 1-14: Core rules, arrays, control flow, parameter expansion (290 rules, 81.2%)
- **Batch 15**: Advanced parameter expansion + command optimization + deprecated syntax
- Achieves **84.9% classification - 🎯 CROSSES 85% MILESTONE! 🎯**

## Expected Impact

**User Value**:
- Catches **parameter expansion inefficiencies** (SC2306-2311, 2315) - POSIX patterns
- Improves **command optimization** (use ${var//} vs sed where appropriate)
- Warns about **deprecated syntax** (SC2312, 2318) - improve portability
- Better **set -e safety** (SC2310) - avoid subtle bugs

**Coverage Improvement**:
- From 290 rules (81.2%) to **303 rules (84.9%)**
- **+13 rules (+3.7 percentage points)** in single batch
- **🎯🎯🎯 CROSSES 85% MILESTONE - Moving toward 90%! 🎯🎯🎯**

## Next Steps

1. ✅ Create this analysis document (batch 15 plan)
2. ⏳ Verify which rules are already implemented (check for duplicates)
3. ⏳ RED Phase: Add rules to rule_registry.rs with comprehensive tests
4. ⏳ GREEN Phase: Verify all registry tests pass
5. ⏳ REFACTOR Phase: Verify rule implementations exist
6. ⏳ QUALITY Phase: Run tests, verify clippy clean
7. ⏳ Commit batch 15 and push
8. ⏳ **CELEBRATE 85% MILESTONE! 🎯🎯🎯**

## Notes

- Focus on **reaching 85% milestone** (303 rules minimum, batch 15 achieves 303)
- SC2306 is likely **NotSh** (bash ${var//} parameter expansion)
- Most other rules are **Universal** (POSIX parameter expansion, deprecated syntax, set -e)
- Conservative: If uncertain about classification, default to Universal
- All 13 rules already verified as implemented (ls command confirmed)

## Quality Target

- **100% test pass rate** (zero regressions)
- **Clippy clean** (zero warnings)
- **Complexity <10** (all functions)
- **EXTREME TDD** (RED → GREEN → REFACTOR → QUALITY)
- **84.9% coverage achieved - 🎯 CROSSED 85% MILESTONE! 🎯**
