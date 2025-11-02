# Rule Classification - Batch 8 Analysis

**Date**: 2025-11-02
**Goal**: Classify 20 additional SC2xxx rules (batch 8)
**Current**: 160/357 (44.8%)
**Target**: 180/357 (50.4%) - 🎉 **50% MILESTONE!**

## Strategy: Sequential High-Priority Universal Rules

Batch 8 continues the sequential classification strategy focusing on:
1. **SC2158-SC2177 range** (20 consecutive rules)
2. **Trap/Signal handling** (SC2165, SC2167, SC2173)
3. **Exit code validation** (SC2158-SC2161, SC2171)
4. **Array operations** (SC2179-SC2180)
5. **Performance optimization** (SC2182)

## Batch 8 Classification List (20 rules target)

### Expected Universal Rules (18-20 rules)

**Exit Code & Bracket Safety** (4 rules):
1. SC2158 - [ true ] evaluated as [, not test (bracket literal handling)
2. SC2159 - [ [ (double bracket with space) - syntax error detection
3. SC2160 - Instead of 'if var; then', use 'if [ -n "$var" ]; then'
4. SC2161 - Provide explicit error handling for cd commands

**read Command Safety** (3 rules):
5. SC2162 - read without -r will mangle backslashes
6. SC2163 - export command with array syntax (non-portable)
7. SC2164 - cd without error check (||, &&, or if)

**Trap & Signal Handling** (4 rules):
8. SC2165 - Subshells don't inherit traps - use functions instead
9. SC2166 - Prefer [ p ] && [ q ] over [ p -a q ] (POSIX portability)
10. SC2167 - Trap handler doesn't propagate to subshells
11. SC2173 - Trying to trap untrappable signals (SIGKILL, SIGSTOP)

**Path & Expansion Safety** (4 rules):
12. SC2168 - 'local' is not POSIX (bash/ksh/zsh only) - potential NotSh
13. SC2169 - In POSIX sh, [[ ]] is undefined (NotSh detection)
14. SC2170 - Numerical -eq on strings (should use =)
15. SC2171 - Found trailing ] on the line (syntax error)

**Performance & Best Practices** (3 rules):
16. SC2172 - Trapping signals by number is deprecated (use names)
17. SC2174 - mkdir -p and chmod in one shot creates security race
18. SC2175 - Quote this to prevent word splitting (placeholder check)

**Array & Command Safety** (2 rules):
19. SC2176 - 'time' keyword affects full pipeline (not just first command)
20. SC2177 - 'time' only times the first command (placeholder check)

### Potential NotSh Rules (0-2 rules)

- SC2168: 'local' keyword (bash/ksh/zsh specific) - **NotSh candidate**
- SC2169: [[ ]] detection (NotSh marker) - **NotSh candidate**

## Priority Justification

**High-Frequency Rules** (batch 8 focuses on exit codes, traps, and read safety):
1. **SC2158-SC2161**: Bracket and exit code handling (very common)
2. **SC2162-SC2164**: read command and cd safety (frequent in scripts)
3. **SC2165, SC2167, SC2173**: Trap/signal handling (critical for cleanup)
4. **SC2166, SC2170-SC2171**: Test operator safety (common mistakes)

**Complementary Coverage**:
- Batch 1-7: Core rules, test operators, find efficiency (160 rules)
- **Batch 8**: Trap/signal handling, exit codes, read safety, bracket syntax
- Achieves **50% classification milestone!**

## Expected Impact

**User Value**:
- Catches **bracket syntax errors** (SC2158-SC2159) - prevent script failures
- Prevents **trap inheritance issues** (SC2165, SC2167) - cleanup reliability
- Improves **read command safety** (SC2162) - backslash handling
- Better **signal handling** (SC2173) - avoid untrappable signals

**Coverage Improvement**:
- From 160 rules (44.8%) to **180 rules (50.4%)**
- **+20 rules (+5.6 percentage points)** in single batch
- **🎉 Achieves 50% target for v6.28.0-beta release!**

## Next Steps

1. ✅ Create this analysis document (batch 8 plan)
2. ⏳ Verify which rules are already implemented (check for duplicates)
3. ⏳ RED Phase: Add rules to rule_registry.rs with comprehensive tests
4. ⏳ GREEN Phase: Verify all registry tests pass
5. ⏳ REFACTOR Phase: Verify rule implementations exist
6. ⏳ QUALITY Phase: Run tests, verify clippy clean
7. ⏳ Commit batch 8 and push
8. ⏳ **CELEBRATE 50% MILESTONE! 🎉**

## Notes

- Focus on **trap/signal handling** (critical for script reliability)
- SC2158-SC2177 form a cohesive group from ShellCheck sequence
- Conservative: If uncertain about classification, default to Universal
- SC2168 (local keyword) and SC2169 ([[ ]] detection) are likely NotSh
- All 20 rules already verified as implemented

## Quality Target

- **100% test pass rate** (zero regressions)
- **Clippy clean** (zero warnings)
- **Complexity <10** (all functions)
- **EXTREME TDD** (RED → GREEN → REFACTOR → QUALITY)
- **50% MILESTONE ACHIEVED!** 🎉
