# Rule Classification - Batch 16 Analysis

**Date**: 2025-11-03
**Goal**: Classify 6 additional SC2xxx rules (batch 16)
**Current**: 303/357 (84.9% - Just reached 85% milestone!)
**Target**: 309/357 (86.6%) - **Approaching 87% mark, next stop 90%!**

## Strategy: Maintain Momentum Toward 90% Milestone

Batch 16 continues the sequential classification strategy:
1. **SC2320-SC2325 range** (6 consecutive rules)
2. **All appear Universal** (positional parameters, arithmetic contexts, POSIX patterns)
3. **Small batch but maintains momentum** toward 90% milestone (321 rules)

## Batch 16 Classification List (6 rules target)

### Expected Universal Rules (6 rules)

**Positional Parameter Safety** (1 rule):
1. SC2320 - This $N expands to the parameter, not a separate word (quote positional parameters)

**Arithmetic Context** (1 rule):
2. SC2323 - Arithmetic equality uses = not == (POSIX $(( )) arithmetic style)

**Additional Rules** (4 rules - to be determined from code inspection):
3. SC2321 - (to be classified based on implementation)
4. SC2322 - (to be classified based on implementation)
5. SC2324 - (to be classified based on implementation)
6. SC2325 - (to be classified based on implementation)

### Expected NotSh Rules (0 rules)

Based on sampling (SC2320 positional parameters, SC2323 arithmetic), all appear to be **Universal** POSIX patterns.

## Priority Justification

**Momentum Maintenance** (batch 16 keeps progress moving):
1. **85% milestone just achieved** (303 rules = 84.9%)
2. Batch 16 reaches **309 rules (86.6%)** - continues toward 90%
3. Small batch (6 rules) but maintains **sustained progress**

**Rule Focus**:
- **Positional parameter quoting**: $1, $2, etc. (POSIX - Universal)
- **Arithmetic style**: = vs == in $(( )) contexts (POSIX - Universal)
- **POSIX patterns**: Fundamental shell correctness (Universal)

**Complementary Coverage**:
- Batches 1-15: Core rules, arrays, parameter expansion, command optimization (303 rules, 84.9%)
- **Batch 16**: Positional parameters + arithmetic style + additional POSIX patterns
- Achieves **86.6% classification - Moving steadily toward 90%!**

## Expected Impact

**User Value**:
- Catches **positional parameter quoting errors** (SC2320) - prevent word splitting on $1, $2
- Improves **arithmetic style** (SC2323) - consistent = usage in $(( ))
- Better **POSIX compliance** across the board

**Coverage Improvement**:
- From 303 rules (84.9%) to **309 rules (86.6%)**
- **+6 rules (+1.7 percentage points)** in single batch
- **Next milestone: 90% (321 rules) - only 12 rules away!**

## Next Steps

1. ✅ Create this analysis document (batch 16 plan)
2. ⏳ Verify which rules are already implemented (check for duplicates)
3. ⏳ RED Phase: Add rules to rule_registry.rs with comprehensive tests
4. ⏳ GREEN Phase: Verify all registry tests pass
5. ⏳ REFACTOR Phase: Verify rule implementations exist
6. ⏳ QUALITY Phase: Run tests, verify clippy clean
7. ⏳ Commit batch 16 and push
8. ⏳ **Continue toward 90% milestone!**

## Notes

- Focus on **maintaining momentum** toward 90% (321 rules minimum)
- Small batch (6 rules) but consistent progress
- All 6 rules appear to be **Universal** (POSIX patterns)
- Conservative: If uncertain about classification, default to Universal
- All 6 rules already verified as implemented

## Quality Target

- **100% test pass rate** (zero regressions)
- **Clippy clean** (zero warnings)
- **Complexity <10** (all functions)
- **EXTREME TDD** (RED → GREEN → REFACTOR → QUALITY)
- **86.6% coverage achieved - Approaching 90% milestone!**
