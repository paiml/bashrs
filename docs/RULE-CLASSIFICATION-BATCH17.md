# Rule Classification - Batch 17 Analysis

**Date**: 2025-11-03
**Goal**: Classify remaining 21 unclassified SC2xxx rules (batch 17)
**Current**: 309/357 (86.6% - Approaching 90% milestone!)
**Target**: 321-330/357 (89.9-92.4%) - **🎯 REACH AND EXCEED 90% MILESTONE! 🎯**

## Strategy: Final Push to 90% Milestone

Batch 17 classifies the **21 remaining implemented but unclassified rules** to reach and exceed the **90% milestone (321 rules)**:
1. **Complete gap filling** (SC2036-SC2037, SC2119, SC2123-SC2125, SC2292-SC2305)
2. **Mix of Universal and NotSh** (backtick quoting, parameter expansion, function patterns)
3. **Reaches 90%+ threshold** (321+ rules = 89.9%+, batch 17 achieves 321-330)

## Batch 17 Classification List (21 rules - ALL REMAINING)

### Expected Universal Rules (15-17 rules)

**Backtick & Command Substitution** (2 rules):
1. SC2036 - Quotes in backticks need escaping (Universal - backticks work in all shells)
2. SC2037 - (to be classified based on implementation)

**Function & Parameter Usage** (4 rules):
3. SC2119 - Use foo "$@" if function's $1 should mean script's $1 (Universal - POSIX positional params)
4. SC2123 - (to be classified)
5. SC2124 - (to be classified)
6. SC2125 - (to be classified)

**Parameter Expansion & Command Optimization** (9-11 rules):
7. SC2293 - (to be classified)
8. SC2294 - (to be classified)
9. SC2295 - (to be classified)
10. SC2296 - (to be classified)
11. SC2297 - (to be classified)
12. SC2298 - (to be classified)
13. SC2299 - (to be classified)
14. SC2300 - (to be classified)
15. SC2301 - (to be classified)
16. SC2302 - (to be classified)
17. SC2303 - (to be classified)
18. SC2304 - (to be classified)
19. SC2305 - (to be classified)

### Expected NotSh Rules (4-6 rules)

**Bash-Specific Parameter Expansion** (potentially 4-6 rules):
20. SC2292 - Prefer ${var:0:1} over expr substr (NotSh - ${var:pos:len} is bash substring expansion)
21. Others in SC2292-SC2305 range that suggest bash-specific ${var:pos:len} syntax

## Priority Justification

**Milestone Importance** (batch 17 crosses 90%):
1. **90% milestone** is critical psychological threshold (321 rules = 89.9%)
2. Batch 17 reaches **321-330 rules (89.9-92.4%)** - **ACHIEVES 90% MILESTONE!**
3. **Completes all implemented rules** - 100% of existing implementations classified

**Rule Focus**:
- **Backtick quoting**: Legacy command substitution patterns (Universal)
- **Function arguments**: Positional parameter usage in functions (Universal)
- **Parameter expansion**: expr vs ${var} optimization (mix of Universal and NotSh)
- **Command optimization**: Modernization suggestions

**Complementary Coverage**:
- Batches 1-16: Core rules, arrays, control flow, parameters (309 rules, 86.6%)
- **Batch 17**: All remaining gaps filled - **COMPLETE IMPLEMENTATION COVERAGE**
- Achieves **89.9-92.4% classification - 🎯🎯🎯 CROSSES 90% MILESTONE! 🎯🎯🎯**

## Expected Impact

**User Value**:
- Catches **backtick quoting errors** (SC2036-SC2037) - prevent shell parsing issues
- Improves **function parameter usage** (SC2119, SC2123-SC2125) - better function design
- Optimizes **parameter expansion** (SC2292-SC2305) - use ${var} instead of expr
- **100% of implemented rules classified** - complete shell-specific filtering

**Coverage Improvement**:
- From 309 rules (86.6%) to **321-330 rules (89.9-92.4%)**
- **+12-21 rules (+3.3-6.0 percentage points)** in single batch
- **🎯🎯🎯 CROSSES 90% MILESTONE - MISSION ACCOMPLISHED! 🎯🎯🎯**

## Next Steps

1. ✅ Create this analysis document (batch 17 plan)
2. ⏳ Read all 21 rule implementations to verify classifications
3. ⏳ RED Phase: Add 21 rules to rule_registry.rs with comprehensive tests
4. ⏳ GREEN Phase: Verify all registry tests pass
5. ⏳ REFACTOR Phase: Verify rule implementations exist
6. ⏳ QUALITY Phase: Run tests, verify clippy clean
7. ⏳ Commit batch 17 and push
8. ⏳ **CELEBRATE 90% MILESTONE! 🎯🎯🎯**

## Notes

- Focus on **reaching 90% milestone** (321 rules minimum, batch 17 achieves 321-330)
- **All 21 remaining rules** will be classified in this batch (100% implementation coverage)
- Conservative: If uncertain about classification, default to Universal
- Priority: Verify ${var:pos:len} syntax is bash-specific (NotSh) vs POSIX (Universal)

## Quality Target

- **100% test pass rate** (zero regressions)
- **Clippy clean** (zero warnings)
- **Complexity <10** (all functions)
- **EXTREME TDD** (RED → GREEN → REFACTOR → QUALITY → DOCUMENTATION → COMMIT)
- **89.9-92.4% coverage achieved - 🎯🎯🎯 CROSSED 90% MILESTONE! 🎯🎯🎯**
