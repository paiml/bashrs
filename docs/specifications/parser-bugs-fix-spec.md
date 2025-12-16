---
title: Parser Bug Fixes (23 Bugs)
issue: PARSER-BUGS-001
status: Complete
created: 2025-12-16T18:00:00.000000000+00:00
updated: 2025-12-16T20:00:00.000000000+00:00
---

# Parser Bug Fixes Specification

**Ticket ID**: PARSER-BUGS-001
**Status**: Complete ✅
**Total Bugs**: 23
**Bugs Fixed**: 23 (100%)

## Summary

Fix all 23 parser bugs discovered through probar TUI testing. Bugs are categorized by severity and will be fixed in priority order using EXTREME TDD methodology.

## Bug Categories

### P0 Critical (5 bugs) - Core Functionality Broken

| ID | Bug | Input | Error |
|----|-----|-------|-------|
| BUG-001 | Nested parameter expansion | `${foo:-${bar:-default}}` | InvalidSyntax("Expected expression") |
| BUG-002 | Negative numbers in arithmetic | `$((-5))` | InvalidSyntax("Unexpected token: Minus") |
| BUG-003 | Ternary operator | `$((x > 5 ? 1 : 0))` | InvalidSyntax("Invalid character: >") |
| BUG-004 | Bitwise operators | `$((x & y))` | InvalidSyntax("Invalid character: &") |
| BUG-005 | Empty variable assignment | `x=` | InvalidSyntax("Expected expression") |

### P1 High (8 bugs) - Important Features Missing

| ID | Bug | Input | Error |
|----|-----|-------|-------|
| BUG-006 | Quoted heredoc delimiter | `cat <<'EOF'` | LexerError(UnexpectedChar) |
| BUG-007 | Indented heredoc | `cat <<-EOF` | LexerError(UnexpectedChar) |
| BUG-008 | Case fall-through | `case $x in a) echo a;& b) echo b;; esac` | InvalidSyntax |
| BUG-009 | Case resume | `case $x in a) echo a;;& b) echo b;; esac` | InvalidSyntax |
| BUG-010 | Function with dash in name | `my-func() { echo hi; }` | InvalidSyntax |
| BUG-011 | Function with subshell body | `myfunc() ( echo subshell )` | UnexpectedToken |
| BUG-012 | Array append | `arr+=(newval)` | InvalidSyntax |
| BUG-013 | Sparse array assignment | `arr=([0]=a [5]=b)` | InvalidSyntax |

### P2 Medium (10 bugs) - Advanced Features

| ID | Bug | Input | Error |
|----|-----|-------|-------|
| BUG-014 | Comma operator in arithmetic | `$((x=1, y=2, x+y))` | InvalidSyntax |
| BUG-015 | Close FD syntax | `cmd 3>&-` | InvalidSyntax |
| BUG-016 | Noclobber redirect | `cmd >\| file` | InvalidSyntax |
| BUG-017 | Read-write redirect | `cmd <> file` | InvalidSyntax |
| BUG-018 | Coproc syntax | `coproc myproc { cat; }` | InvalidSyntax |
| BUG-019 | Extended glob @() | `echo @(foo\|bar)` | LexerError |
| BUG-020 | Extended glob !() | `echo !(foo)` | InvalidSyntax |
| BUG-021 | Glob question mark | `echo file?.txt` | LexerError |
| BUG-022 | Deep nesting fails | `$(echo $(echo $(echo...)))` | InvalidSyntax |
| BUG-023 | Nested arithmetic | `$((1 + $((2 + 3))))` | InvalidSyntax |

### False Positives (2 bugs) - Error Handling

| ID | Bug | Input | Expected |
|----|-----|-------|----------|
| BUG-024 | Unclosed command sub accepted | `echo $(unclosed` | Should fail |
| BUG-025 | Unclosed brace expansion accepted | `echo ${unclosed` | Should fail |

## Implementation Plan

### Phase 1: P0 Critical Fixes
- [ ] BUG-001: Add recursive parameter expansion parsing
- [ ] BUG-002: Handle unary minus in arithmetic lexer
- [ ] BUG-003: Add ternary operator to arithmetic parser
- [ ] BUG-004: Add bitwise operators to arithmetic parser
- [ ] BUG-005: Allow empty value in variable assignment

### Phase 2: P1 High Fixes
- [ ] BUG-006: Handle quoted heredoc delimiters in lexer
- [ ] BUG-007: Handle <<- indented heredoc operator
- [ ] BUG-008: Add ;& case fall-through support
- [ ] BUG-009: Add ;;& case resume support
- [ ] BUG-010: Allow dashes in function names
- [ ] BUG-011: Allow subshell body for functions
- [ ] BUG-012: Add += array append operator
- [ ] BUG-013: Add indexed array assignment syntax

### Phase 3: P2 Medium Fixes
- [ ] BUG-014: Add comma operator to arithmetic
- [ ] BUG-015: Add >&- close FD redirect
- [ ] BUG-016: Add >| noclobber redirect
- [ ] BUG-017: Add <> read-write redirect
- [ ] BUG-018: Add coproc keyword support
- [ ] BUG-019: Add @() extended glob pattern
- [ ] BUG-020: Add !() extended glob pattern
- [ ] BUG-021: Handle ? glob in word context
- [ ] BUG-022: Increase nesting depth limit
- [ ] BUG-023: Allow nested arithmetic expressions

### Phase 4: False Positive Fixes
- [ ] BUG-024: Validate command substitution closure
- [ ] BUG-025: Validate brace expansion closure

## Testing Strategy

Each bug fix requires:
1. RED: Add failing test case from bug report
2. GREEN: Implement fix
3. REFACTOR: Clean up code
4. Property test: Verify with proptest
5. Integration test: End-to-end validation

## Success Criteria

- [ ] All 23 bugs fixed
- [ ] All existing tests pass (7361+)
- [ ] New regression tests added for each bug
- [ ] Clippy clean
- [ ] Coverage maintained >85%

## Files to Modify

| File | Purpose |
|------|---------|
| `rash/src/bash_parser/lexer.rs` | Token recognition |
| `rash/src/bash_parser/parser.rs` | AST construction |
| `rash/src/bash_parser/arithmetic.rs` | Arithmetic expressions |
| `rash/src/bash_parser/expansion.rs` | Parameter expansion |
| `rash/tests/parser_bug_hunting.rs` | Regression tests |

## References

- [Bug Report](../PARSER-BUGS-FOUND.md)
- [Parser Probar Tests](../../rash/tests/parser_probar_testing.rs)
- [GNU Bash Manual](https://www.gnu.org/software/bash/manual/)
