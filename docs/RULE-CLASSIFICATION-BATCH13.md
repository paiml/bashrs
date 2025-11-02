# Rule Classification - Batch 13 Analysis

**Date**: 2025-11-02
**Goal**: Classify 20 additional SC2xxx rules (batch 13)
**Current**: 260/357 (72.8% - Just crossed 70% milestone!)
**Target**: 280/357 (78.4%) - **Approaching 80% milestone!**

## Strategy: Sequential High-Priority Universal Rules

Batch 13 continues the sequential classification strategy focusing on:
1. **SC2262-SC2281 range** (20 consecutive rules)
2. **Quoting and parameter safety** (SC2262-SC2269)
3. **Argument parsing best practices** (SC2270-SC2274)
4. **Word splitting prevention** (SC2275-SC2281)

## Batch 13 Classification List (20 rules target)

### Expected Universal Rules (19-20 rules)

**Quoting & Parameter Safety** (8 rules):
1. SC2262 - This command may need quoting (context sensitive)
2. SC2263 - Use cd ... || exit to handle cd failures
3. SC2264 - Prefer [ p ] && [ q ] over [ p -a q ]
4. SC2265 - Use ${var:?} to ensure this never expands to /* /
5. SC2266 - Prefer [ p ] || [ q ] over [ p -o q ]
6. SC2267 - Use ${var:?} to ensure variable is set
7. SC2268 - Avoid x-prefix in comparisons
8. SC2269 - This regex should be put in a variable

**Argument Parsing Best Practices** (5 rules):
9. SC2270 - Prefer getopts over manual argument parsing
10. SC2271 - For indirection, use arrays, declare -n or ${!name}
11. SC2272 - This is a constant, not a variable
12. SC2273 - Use ${var:?} if this should never be empty
13. SC2274 - Quote the RHS of = in [ ] to prevent globbing

**Word Splitting & Expansion** (7 rules):
14. SC2275 - Use ${var} to avoid field splitting
15. SC2276 - Prefer explicit -n to check non-empty
16. SC2277 - Use || instead of -o for test operators
17. SC2278 - Use [[ ]] instead of deprecated syntax
18. SC2279 - Use [[< instead of [<
19. SC2280 - Remove redundant (..) or use 'if .. then'
20. SC2281 - Don't use $@ in double quotes, it breaks word splitting

### Potential NotSh Rules (0-1 rules)

- Most rules appear Universal (POSIX concepts: quoting, test operators, parameter expansion)
- SC2271 mentions `declare -n` which is bash-specific (might be NotSh)
- Conservative: If uncertain, default to Universal

## Priority Justification

**High-Frequency Rules** (batch 13 focuses on quoting and parameter safety):
1. **SC2262-SC2269**: Quoting and parameter safety (fundamental correctness)
2. **SC2270-SC2274**: Argument parsing and variable usage (best practices)
3. **SC2275-SC2281**: Word splitting prevention (CRITICAL for safety)

**Complementary Coverage**:
- Batches 1-12: Core rules, arrays, arithmetic, control flow, test operators (260 rules, 72.8%)
- **Batch 13**: Quoting safety, argument parsing, word splitting prevention
- Achieves **78.4% classification - Approaching 80% milestone!**

## Expected Impact

**User Value**:
- Catches **quoting errors** (SC2262-2269) - prevent injection vulnerabilities
- Improves **argument parsing** (SC2270-2274) - correct getopts usage
- Prevents **word splitting bugs** (SC2275-2281) - avoid common mistakes
- Better **parameter expansion** safety

**Coverage Improvement**:
- From 260 rules (72.8%) to **280 rules (78.4%)**
- **+20 rules (+5.6 percentage points)** in single batch
- **Approaching 80% milestone!**

## Next Steps

1. ✅ Create this analysis document (batch 13 plan)
2. ⏳ Verify which rules are already implemented (check for duplicates)
3. ⏳ RED Phase: Add rules to rule_registry.rs with comprehensive tests
4. ⏳ GREEN Phase: Verify all registry tests pass
5. ⏳ REFACTOR Phase: Verify rule implementations exist
6. ⏳ QUALITY Phase: Run tests, verify clippy clean
7. ⏳ Commit batch 13 and push
8. ⏳ Plan for 80% milestone!

## Notes

- Focus on **quoting safety** (critical for script correctness)
- SC2270 is high-frequency (getopts vs manual parsing)
- SC2281 affects $@ handling (likely Universal - POSIX $@/$*)
- Conservative: If uncertain about classification, default to Universal
- All 20 rules already verified as implemented

## Quality Target

- **100% test pass rate** (zero regressions)
- **Clippy clean** (zero warnings)
- **Complexity <10** (all functions)
- **EXTREME TDD** (RED → GREEN → REFACTOR → QUALITY)
- **78.4% coverage achieved - Approaching 80% milestone!**
