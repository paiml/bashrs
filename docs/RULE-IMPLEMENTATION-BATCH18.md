# Rule Implementation - Batch 18 Analysis

**Date**: 2025-11-03
**Goal**: Implement 7 missing SC2xxx rules (SC2008-SC2014)
**Current**: 330/357 (92.4% classified, 7 core rules unimplemented)
**Target**: 337/357 (94.4%) - Continue toward 100%

## Strategy: High-Priority Core Rules

Batch 18 implements the **7 missing core SC2xxx rules (SC2008-SC2014)** that are in the documented ShellCheck sequence but not yet implemented in bashrs.

These rules focus on **common shell anti-patterns** and file handling best practices.

## Batch 18 Implementation List (7 rules - ALL UNIVERSAL)

### File Handling & Command Best Practices (7 rules)

1. **SC2008** - echo doesn't read from stdin
   - Issue: `command | echo` doesn't work (echo ignores stdin)
   - Fix: Use `command | cat` or `command | xargs echo`
   - Compatibility: **Universal** (POSIX echo behavior)

2. **SC2009** - Consider using pgrep instead of grepping ps output
   - Issue: `ps aux | grep process_name` is inefficient and fragile
   - Fix: Use `pgrep process_name` instead
   - Compatibility: **Universal** (pgrep is widely available)

3. **SC2010** - Don't use ls | grep, use a glob or for loop
   - Issue: `ls | grep pattern` breaks on filenames with spaces
   - Fix: Use `for f in *pattern*; do` or glob patterns
   - Compatibility: **Universal** (glob patterns are POSIX)

4. **SC2011** - Use 'find -print0 | xargs -0' instead of ls | xargs
   - Issue: `ls | xargs command` breaks on filenames with spaces/newlines
   - Fix: Use `find . -print0 | xargs -0 command` for safety
   - Compatibility: **Universal** (find -print0 is standard)

5. **SC2012** - Use find instead of ls to better handle non-alphanumeric filenames
   - Issue: `ls | while read` breaks on special characters
   - Fix: Use `find . -exec` or `find . -print0 | while IFS= read -r -d ''`
   - Compatibility: **Universal** (find is POSIX)

6. **SC2013** - To read lines rather than words, pipe/redirect to 'while read' loop
   - Issue: `for line in $(cat file)` splits on whitespace (reads words, not lines)
   - Fix: Use `while IFS= read -r line; do` instead
   - Compatibility: **Universal** (while read is POSIX)

7. **SC2014** - This will expand before brace expansion happens
   - Issue: Variables in brace expansions like `{$start..$end}` don't work
   - Fix: Use `seq` or `for ((i=start; i<=end; i++))` instead
   - Compatibility: **NotSh** for (( )) loops, **Universal** for seq

## Priority Justification

**High-Impact Rules** (batch 18 addresses common mistakes):
1. **File safety**: SC2010, SC2011, SC2012 prevent filename handling bugs
2. **Process efficiency**: SC2009 improves process management
3. **I/O correctness**: SC2008, SC2013 fix common pipe/redirect mistakes
4. **Brace expansion**: SC2014 catches variable expansion errors

**Complementary Coverage**:
- Batches 1-17: 330 rules classified (shell-specific filtering complete)
- **Batch 18**: 7 core missing rules (common anti-patterns)
- Achieves **337/357 rules (94.4%)** - Approaching 95% milestone

## Expected Impact

**User Value**:
- Catches **file handling errors** (SC2010, SC2011, SC2012) - prevent data loss/corruption
- Improves **process management** (SC2009) - more efficient scripts
- Fixes **I/O bugs** (SC2008, SC2013) - correct pipe/redirect behavior
- Prevents **brace expansion errors** (SC2014) - common beginner mistake

**Coverage Improvement**:
- From 330 rules (92.4%) to **337 rules (94.4%)**
- **+7 rules (+2.0 percentage points)** in single batch
- **Approaching 95% milestone (339 rules = 95.0%)**

## Next Steps

1. ✅ Create this analysis document (batch 18 plan)
2. ⏳ RED Phase: Write 7 failing tests for SC2008-SC2014
3. ⏳ GREEN Phase: Implement lint logic for all 7 rules
4. ⏳ Add 7 rules to rule_registry.rs with Universal compatibility
5. ⏳ REFACTOR Phase: Clean up implementation, complexity <10
6. ⏳ QUALITY Phase: Run tests, verify clippy clean, check mutation testing
7. ⏳ Commit batch 18 and push
8. ⏳ **Continue toward 95% milestone!**

## Notes

- Focus on **common anti-patterns** (SC2008-SC2014 are frequently violated)
- **All 7 rules** are likely Universal (apply to all shells)
- SC2014 may be NotSh if it suggests bash-specific workarounds
- Conservative: If uncertain about classification, default to Universal
- Priority: File safety (SC2010-SC2012) and I/O correctness (SC2008, SC2013)

## Quality Target

- **100% test pass rate** (zero regressions)
- **Clippy clean** (zero warnings)
- **Complexity <10** (all functions)
- **EXTREME TDD** (RED → GREEN → REFACTOR → QUALITY → DOCUMENTATION → COMMIT)
- **94.4% coverage achieved - Approaching 95% milestone!**
