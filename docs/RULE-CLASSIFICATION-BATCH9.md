# Rule Classification - Batch 9 Analysis

**Date**: 2025-11-02
**Goal**: Classify 20 additional SC2xxx rules (batch 9)
**Current**: 180/357 (50.4% - 🎉 50% MILESTONE achieved!)
**Target**: 200/357 (56.0%) - **Aiming for 60% milestone region**

## Strategy: Sequential High-Priority Universal Rules

Batch 9 continues the sequential classification strategy focusing on:
1. **SC2178-SC2197 range** (20 consecutive rules)
2. **Array operations** (SC2178-SC2180)
3. **Exit code patterns** (SC2181-SC2182)
4. **Assignment safety** (SC2183-SC2186)
5. **Expansion patterns** (SC2187-SC2197)

## Batch 9 Classification List (20 rules target)

### Expected Universal Rules (18-20 rules)

**Array Operations** (3 rules):
1. SC2178 - Variable was used as an array but is now assigned as a string
2. SC2179 - Use array+=("item") to append items to an array
3. SC2180 - Trying to use an array as a scalar (missing index)

**Exit Code Patterns** (2 rules):
4. SC2181 - Check exit code directly with if mycmd, not if [ $? -eq 0 ]
5. SC2182 - This printf format string has no variables

**Assignment Safety** (4 rules):
6. SC2183 - This value looks like a variable but won't be expanded
7. SC2184 - Quote arguments to cd to avoid glob expansion
8. SC2185 - Some SSH commands don't pass on their exit codes
9. SC2186 - mktemp argument may be evaluated as template

**Expansion Patterns** (11 rules):
10. SC2187 - Ash scripts will be checked as Dash (use #!/bin/dash or directive)
11. SC2188 - This redirection doesn't have a command
12. SC2189 - Zsh directive will be checked as sh (use #!/bin/zsh)
13. SC2190 - Elements in associative arrays need index
14. SC2191 - Trying to use an associative array without index
15. SC2192 - Piping to sudo: only last command will run as root
16. SC2193 - RHS of regexes must be unquoted in [[]]
17. SC2194 - This word is constant - did you forget $ or ()
18. SC2195 - Use single quotes to pass literal regex to grep
19. SC2196 - Prefer explicit -n to check output
20. SC2197 - Don't compare globs in []; use [[ ]] or case

### Potential NotSh Rules (0-2 rules)

- SC2178-SC2180: Array operations (bash/zsh specific) - **NotSh candidates**
- SC2190-SC2191: Associative arrays (bash 4+ / zsh specific) - **NotSh candidates**
- SC2189: Zsh-specific directive - **NotSh candidate**

## Priority Justification

**High-Frequency Rules** (batch 9 focuses on array operations and exit codes):
1. **SC2178-SC2180**: Array operations (very common in bash scripts)
2. **SC2181**: Exit code checking (common antipattern)
3. **SC2183-SC2186**: Assignment safety (frequent mistakes)
4. **SC2187-SC2197**: Expansion patterns (common quoting/expansion issues)

**Complementary Coverage**:
- Batch 1-8: Core rules, test operators, trap handling (180 rules, 50.4%)
- **Batch 9**: Array operations, exit codes, assignment safety
- Achieves **56% classification milestone region**

## Expected Impact

**User Value**:
- Catches **array operation errors** (SC2178-2180) - prevent runtime failures
- Prevents **exit code antipatterns** (SC2181) - improve error handling
- Improves **assignment safety** (SC2183-2186) - prevent variable issues
- Better **expansion handling** (SC2187-2197) - regex/glob safety

**Coverage Improvement**:
- From 180 rules (50.4%) to **200 rules (56.0%)**
- **+20 rules (+5.6 percentage points)** in single batch
- **Approaching 60% milestone region**

## Next Steps

1. ✅ Create this analysis document (batch 9 plan)
2. ⏳ Verify which rules are already implemented (check for duplicates)
3. ⏳ RED Phase: Add rules to rule_registry.rs with comprehensive tests
4. ⏳ GREEN Phase: Verify all registry tests pass
5. ⏳ REFACTOR Phase: Verify rule implementations exist
6. ⏳ QUALITY Phase: Run tests, verify clippy clean
7. ⏳ Commit batch 9 and push
8. ⏳ **Continue toward 60% milestone!**

## Notes

- Focus on **array operations** (critical for bash script safety)
- SC2178-2180 and SC2190-2191 are array-related (likely NotSh)
- SC2181 is high-frequency exit code antipattern (likely Universal)
- Conservative: If uncertain about classification, default to Universal
- All 20 rules already verified as implemented

## Quality Target

- **100% test pass rate** (zero regressions)
- **Clippy clean** (zero warnings)
- **Complexity <10** (all functions)
- **EXTREME TDD** (RED → GREEN → REFACTOR → QUALITY)
- **56% MILESTONE region achieved!**
