# Rule Classification - Batch 2 Analysis

**Date**: 2025-11-02
**Goal**: Classify 50-80 additional SC2xxx rules (batch 2)
**Current**: 20/357 (6%)
**Target**: 70-100/357 (20-28%)

## Classification Strategy

### Feature-Based Classification

#### 1. Arrays (NotSh - bash/zsh/ksh only)
POSIX sh does NOT support arrays. These rules are **NotSh**:
- Already classified: SC2198-2201 ✅
- Additional array rules to classify:
  - SC2068 - Double quote array expansions
  - SC2069 - Wrong redirection (check if array-related)
  - SC2179 - Array append syntax
  - SC2180 - Multidimensional arrays (bash doesn't support, but warning applies to bash)
  - SC2182 - Printf with arrays

#### 2. [[ ]] Test Syntax (NotSh - bash/zsh/ksh only)
POSIX sh only supports [ ], not [[ ]]. These rules are **NotSh**:
- SC2107 - (check if [[ ]] related)
- SC2108 - In [[ ]], use && instead of -a
  - SC2109 - In [[ ]], use || instead of -o
- SC2110 - Don't mix && and || with -a and -o

#### 3. Process Substitution <(...) (NotSh - bash/zsh/ksh only)
POSIX sh does NOT support process substitution. These rules are **NotSh**:
- SC2002 - Useless cat (often suggests process substitution) ✅ Already classified

#### 4. Arithmetic Expansion $((...)) (Universal - POSIX)
These are **Universal**:
- SC2003 - expr is antiquated, use $((...))
- SC2004 - $ unnecessary in arithmetic
- SC2005 - Useless echo in command substitution
- SC2007 - Use $((..)) instead of deprecated $[..]
- SC2015 - Note that A && B || C is not if-then-else
- SC2079 - Decimals not supported in arithmetic
- SC2080 - Leading zero = octal
- SC2084 - Arithmetic as command
- SC2085 - Local with arithmetic
- SC2133 - Unexpected tokens in arithmetic
- SC2134 - Use (( )) for numeric tests
- SC2137 - Unnecessary braces in arithmetic

#### 5. Quoting and Expansion (Universal - POSIX)
These are **Universal** - quoting rules apply to all shells:
- SC2006 - Use $(...) instead of backticks
- SC2016 - Expressions don't expand in single quotes
- SC2017 - Variable names can't start with digits
- SC2018-SC2021 - Character class warnings
- SC2022-SC2025 - Shell command safety
- SC2026 - Quote parameters to avoid word splitting
- SC2027-SC2029 - Quote variables
- SC2030-SC2032 - Subshell variable scope
- SC2033 - Shell commands that read stdin
- SC2036-SC2037 - Backtick quoting
- SC2046 - Quote to prevent word splitting ✅ (in original linter)
- SC2047-SC2053 - Quoting in tests
- SC2054-SC2065 - Quote/glob patterns
- SC2067 - Missing $ on array index (wait, this might be array-specific)
- SC2073-SC2075 - Quoting and escaping
- SC2077-SC2078 - Regex quoting
- SC2082 - Variable indirection
- SC2086 - Quote variables ✅ (in original linter)
- SC2087-SC2093 - Quoting in sh -c, exec, etc.

#### 6. bash-Specific Keywords (NotSh)
- SC2039 - bash features undefined in sh ✅ Already classified
- SC2111-SC2112 - 'function' keyword (bash/ksh, not POSIX)
- SC2113 - 'function' with () is redundant
- SC2118 - ksh set -A arrays

#### 7. Loop and Control Flow (Universal - POSIX)
These apply to all shells:
- SC2038-SC2042 - Loop safety
- SC2117 - Unreachable code
- SC2121-SC2122 - Assignment and operators in tests
- SC2126-SC2127 - Grep efficiency, constant comparisons
- SC2135-SC2136 - Control flow keywords

#### 8. Deprecated Syntax (Universal)
- SC2099 - Use $(...) instead of backticks
- SC2100 - Use $((..)) instead of expr

#### 9. Functions and Aliases (Universal - POSIX sh has functions)
- SC2138-SC2142 - Function context, alias safety

#### 10. POSIX Compatibility Warnings (ShOnly? or Universal?)
- SC2040 - Avoid -o flag confusion
- SC2042 - printf instead of echo -e (portability)
- SC2148 - Add shebang

## Batch 2 Classification List (50 rules)

### NotSh (bash/zsh/ksh, not POSIX sh) - 15 rules
1. SC2068 - Array expansion quoting
2. SC2108 - [[ ]] use && not -a
3. SC2109 - [[ ]] use || not -o
4. SC2110 - Don't mix && || with -a -o in [[ ]]
5. SC2111 - function keyword in sh
6. SC2112 - function keyword non-standard
7. SC2113 - function with () redundant
8. SC2118 - ksh set -A
9. SC2179 - Array append syntax
10. SC2180 - Multidimensional arrays
11. SC2182 - Printf with arrays (check this)
12. SC2034 - Variable unused (check if bash-local scope related)
13. SC2043 - for loop over one value (check if bash for...in specific)
14. SC2102 - Range + quantifier (check if bash-specific)
15. SC2101 - POSIX class needs outer brackets (actually universal!)

### Universal (all shells) - 35 rules
1. SC2003 - expr antiquated
2. SC2004 - $ unnecessary in arithmetic
3. SC2005 - Useless echo
4. SC2006 - Use $(...) not backticks
5. SC2007 - Use $((..)) not $[..]
6. SC2015 - && || not if-then-else
7. SC2016 - Single quotes don't expand
8. SC2017 - Variable names start restrictions
9. SC2018-SC2021 - Character classes
10. SC2022-SC2025 - Shell command safety
11. SC2026 - Quote parameters
12. SC2027-SC2029 - Quote variables
13. SC2030-SC2032 - Subshell scope
14. SC2033 - Commands that read stdin
15. SC2036-SC2037 - Backtick quoting
16. SC2038 - find loop safety
17. SC2040 - -o flag confusion
18. SC2041 - read in for loop
19. SC2042 - printf vs echo -e
20. SC2047-SC2053 - Quoting in tests
21. SC2054-SC2065 - Patterns and traps
22. SC2067 - Missing $ (check if array-specific)
23. SC2073-SC2075 - Escaping
24. SC2077-SC2078 - Regex quoting
25. SC2079-SC2080 - Arithmetic numbers
26. SC2082 - Variable indirection
27. SC2084-SC2085 - Arithmetic context
28. SC2087-SC2093 - sh -c quoting, exec
29. SC2099-SC2100 - Deprecated syntax
30. SC2117 - Unreachable code
31. SC2121-SC2122 - Assignment operators
32. SC2126-SC2127 - Grep, constants
33. SC2133-SC2134 - Arithmetic tests
34. SC2135-SC2137 - Control flow, arithmetic braces
35. SC2138-SC2142 - Functions, aliases

## Next Steps

1. ✅ Create this analysis document
2. ⏳ Validate classifications by reading 5-10 representative rules
3. ⏳ Update rule_registry.rs with batch 2 (RED phase - write tests first)
4. ⏳ Expand lint_shell_filtered() with apply_rule! calls
5. ⏳ Add integration tests for batch 2 filtering
6. ⏳ Run mutation testing (target ≥90% kill rate)
7. ⏳ Update CHANGELOG.md and docs

## Notes

- Some rules need validation (marked "check if")
- Priority: Get to 70-100 rules (20-28%) before v6.28.0 release
- Conservative approach: If unsure, classify as Universal
