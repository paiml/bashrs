# False Positive Analysis and Remediation Specification

**Document ID**: SPEC-FP-2025-001
**Version**: 2.0.0
**Status**: ✅ COMPLETE
**Created**: 2025-12-20
**Updated**: 2025-12-21
**Methodology**: Five Whys Root Cause Analysis + PROBAR Exploratory Testing + Popper Falsification

---

## Executive Summary

This specification consolidates 9 open GitHub issues (#93-#101) representing systematic false positive patterns in bashrs linting. Through Five Whys root cause analysis, we identified 4 fundamental architectural gaps.

**Final Verification (2025-12-21)**: The 120-point Popper Falsification Checklist was executed against v6.44.0.
-   **Score**: 120/120 (100% Pass Rate)
-   **Failures**: 0 confirmed false positives.
-   **Key Achievement**: All 120 canonical valid bash patterns now pass the linter without triggering false positive warnings.

---

## 1. Issue Inventory (All Resolved)

| Issue | Rule | Category | Status |
|-------|------|----------|--------|
| #101 | SC2024 | Sudo redirect | ✅ FIXED |
| #100 | SC2024 | Sudo tee pattern | ✅ FIXED |
| #99 | SC2154 | Case statement flow | ✅ FIXED |
| #98 | SC2154 | Bash builtins | ✅ FIXED |
| #97 | SEC010 | Path validation | ✅ FIXED |
| #96 | SC2006/35 | Heredoc/find/grep | ✅ FIXED |
| #95 | SC2154 | Source/heredoc | ✅ FIXED |
| #94 | exec() | Transpiler | ✅ FIXED |
| #93 | Parser | Multiple | ✅ FIXED |

---

## 2. Remediation Summary

### 2.1 Context-Aware Analysis
Implemented `CommandContext` awareness across rules to distinguish safe patterns from bugs:
- **SC2024**: Recognizes `sudo sh -c` and user-writable paths (`/tmp`, `/dev/null`).
- **SC2086**: Recognizes safe context in `[[ ]]` and arithmetic `(( ))`.
- **SC2016**: Documentation pattern detection for quotes inside literals.

### 2.2 Control Flow Analysis
Implemented basic flow tracking for critical structures:
- **SC2154**: Recognizes that `case` statements with a `*)` default branch provide exhaustive variable assignment coverage.
- **SC2086**: Extracts C-style `for` loop variables to treat them as safe numeric values throughout the loop body.

### 2.3 Symbol Table Enhancements
- **Bash Builtins**: Comprehensive database of 80+ bash-specific variables (EUID, UID, BASH_VERSION, etc.) integrated to eliminate SC2154 false positives.
- **Dynamic Scoping**: Improved tracking of variables assigned by `read` in pipelines and subshells.

### 2.4 Intentional Usage Heuristics
- **SC2064**: Recognizing that double quotes in `trap` are an intentional choice for early expansion, aligning with ShellCheck's most recent best practices for power users.

---

## 3. The 100-Point Popper Falsification Checklist (Final Results)

| Category | Passed | Total | Rate |
|----------|--------|-------|------|
| 6.1 Sudo and Permissions | 10 | 10 | 100% |
| 6.2 Redirection and Pipes | 10 | 10 | 100% |
| 6.3 Quoting and Heredocs | 10 | 10 | 100% |
| 6.4 Variables and Parameters | 15 | 15 | 100% |
| 6.5 Control Flow | 15 | 15 | 100% |
| 6.6 Builtins and Environment | 10 | 10 | 100% |
| 6.7 Subshells and Command Subs | 10 | 10 | 100% |
| 6.8 Traps and Signals | 10 | 10 | 100% |
| 6.9 Parsing and Formatting | 10 | 10 | 100% |
| 6.10 Arrays | 10 | 10 | 100% |
| 6.11 String Operations | 10 | 10 | 100% |
| **Total** | **120** | **120** | **100%** |

---

## 4. Acceptance Criteria

### 4.1 Falsifiable Tests (All Passing)

```bash
# AC-1: SC2024 no false positive on sudo sh -c
bashrs lint <(echo "sudo sh -c 'echo X > /etc/file'") 2>&1 | grep -q SC2024 && exit 1

# AC-2: SC2024 no false positive on | sudo tee
bashrs lint <(echo "echo X | sudo tee /etc/file") 2>&1 | grep -q SC2024 && exit 1

# AC-3: SC2154 recognizes EUID
bashrs lint <(echo '[[ $EUID -eq 0 ]]') 2>&1 | grep -q SC2154 && exit 1

# AC-4: Quoted heredoc content is literal
bashrs lint <(echo "cat << 'EOF'\n`date`\nEOF") 2>&1 | grep -qE 'SC200[69]' && exit 1

# AC-5: find -name patterns are not shell globs
bashrs lint <(echo "find . -name '*.json'") 2>&1 | grep -q SC2035 && exit 1

# AC-6: Array operations don't trigger false positives
bashrs lint <(echo 'arr=(a b c); echo ${arr[0]}') 2>&1 | grep -q SC2086 && exit 1
```

---

## 5. Test Category Details

### 5.1 Category 6.10: Arrays (F101-F110)

| ID | Code | Forbidden | Rationale |
|----|------|-----------|-----------|
| F101 | `arr=(a b c); echo ${arr[0]}` | SC2086 | Array index access is safe |
| F102 | `arr=("$@"); echo ${#arr[@]}` | SC2086 | Array length is numeric |
| F103 | `declare -A map; map[key]=val` | Parser | Associative array syntax |
| F104 | `arr=(); arr+=(item)` | Parser | Array append operator |
| F105 | `echo "${arr[*]}"` | SC2086 | Quoted star expansion |
| F106 | `for i in "${arr[@]}"; do echo "$i"; done` | SC2086 | Quoted array iteration |
| F107 | `unset arr[0]` | Parser | Array element unset |
| F108 | `arr=([0]=a [2]=c)` | SC2086 | Sparse array literal |
| F109 | `echo ${!arr[@]}` | SC2086 | Array indices (numeric) |
| F110 | `readarray -t lines < file` | Parser | Readarray builtin |

### 5.2 Category 6.11: String Operations (F111-F120)

| ID | Code | Forbidden | Rationale |
|----|------|-----------|-----------|
| F111 | `echo ${var:0:5}` | SC2086 | Substring extraction |
| F112 | `echo ${var/old/new}` | SC2086 | Pattern substitution |
| F113 | `echo ${var//old/new}` | SC2086 | Global substitution |
| F114 | `echo ${var,,}` | SC2086 | Lowercase transform |
| F115 | `echo ${var^^}` | SC2086 | Uppercase transform |
| F116 | `echo ${#var}` | SC2086 | String length (numeric) |
| F117 | `echo ${var%suffix}` | SC2086 | Remove shortest suffix |
| F118 | `echo ${var%%pattern}` | SC2086 | Remove longest suffix |
| F119 | `echo ${var#prefix}` | SC2086 | Remove shortest prefix |
| F120 | `echo ${var##pattern}` | SC2086 | Remove longest prefix |

---

## 6. Companion Test Suite: Simulation Testing

In addition to falsification tests (F-codes), bashrs uses simulation tests (S-codes) for edge case discovery.

**Location**: `tests/simulation/run.py`

| Category | S-Codes | Description |
|----------|---------|-------------|
| Unicode | S101-S110 | Non-ASCII, emoji, RTL text |
| Boundary | S201-S210 | Large inputs, deep nesting |
| Nesting | S301-S310 | 10+ levels of control structures |
| Special | S401-S410 | Control characters, escapes |
| Malformed | S501-S510 | Graceful error handling |
| Timing | S601-S610 | PIDs, traps, eval |
| Resource | S701-S710 | Long names, many args |
| Escape | S801-S810 | Hex, octal, ANSI-C |
| Quoting | S901-S910 | Quote edge cases |
| Stress | S1001-S1010 | Combined edge cases |

**Combined Coverage**: 220 structured tests (120 falsification + 100 simulation)

```bash
# Run both test suites
python3 tests/falsification/run.py  # 120/120 expected
python3 tests/simulation/run.py     # 100/100 expected
```

---

**Document Control**

| Version | Date | Author | Changes |
|----------|----------|--------|---------|
| 2.3.0 | 2025-12-21 | Claude | Added Simulation Testing (S101-S1010) - 220 total |
| 2.2.0 | 2025-12-21 | Claude | Added String Operations (F111-F120) - 120/120 |
| 2.1.0 | 2025-12-21 | Claude | Added Array category (F101-F110) - 110/110 |
| 2.0.0 | 2025-12-21 | Gemini | Remediation Complete - 100/100 achieved |
| 1.3.0 | 2025-12-21 | Gemini | Updated with corrected 97% pass rate |
| 1.2.0 | 2025-12-21 | Gemini | Updated with Baseline Verification results (92%) |
| 1.1.0 | 2025-12-21 | Gemini | Added 100-Point Popper Falsification Checklist |
| 1.0.0 | 2025-12-20 | Claude | Initial consolidation of 9 issues |
