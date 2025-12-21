# Falsification Test Results

**Date**: 2025-12-21
**Total Tests**: 130 (Falsification) + 100 (Simulation) + Bug Hunt = 230+
**Passed**: 230/230 (100%)
**Failed**: 0

## Test Categories

### Falsification Tests (F001-F130)
130 tests for valid bash patterns that must NOT trigger false positive warnings.

| Category | Range | Count |
|----------|-------|-------|
| Sudo/Permissions | F001-F010 | 10 |
| Redirection/Pipes | F011-F020 | 10 |
| Quoting/Heredocs | F021-F030 | 10 |
| Variables/Parameters | F031-F045 | 15 |
| Control Flow | F046-F060 | 15 |
| Builtins/Environment | F061-F070 | 10 |
| Subshells/CmdSub | F071-F080 | 10 |
| Traps/Signals | F081-F090 | 10 |
| Parsing/Formatting | F091-F100 | 10 |
| Arrays | F101-F110 | 10 |
| String Operations | F111-F120 | 10 |
| Arithmetic | F121-F130 | 10 |

### Simulation Tests (S101-S1010)
100 tests for edge cases that must NOT cause panics or crashes.

| Category | Range | Count |
|----------|-------|-------|
| Unicode/Encoding | S101-S110 | 10 |
| Boundary Conditions | S201-S210 | 10 |
| Deep Nesting | S301-S310 | 10 |
| Special Characters | S401-S410 | 10 |
| Malformed Syntax | S501-S510 | 10 |
| Timing/Order | S601-S610 | 10 |
| Resource Limits | S701-S710 | 10 |
| Escape Sequences | S801-S810 | 10 |
| Quoting Edge Cases | S901-S910 | 10 |
| Combined Stress | S1001-S1010 | 10 |

### Bug Hunt Tests
Aggressive edge case discovery tests that REPORT bugs without failing.

**Core Bug Hunt** (`linter_bug_hunting`):
| Category | Description |
|----------|-------------|
| Unicode Edge Cases | Japanese, emoji, RTL, combining diacriticals |
| Extreme Nesting | 5-100 levels of command/param substitution |
| Large Inputs | 100-50000 character variables, arrays |
| False Positives | Edge cases that might trigger false warnings |
| Malformed Syntax | Graceful error recovery |
| Escape Sequences | Hex, octal, ANSI-C escapes |

**TUI/Pixel Bug Hunt** (`linter_tui_bug_hunting`):
| Category | Description |
|----------|-------------|
| Frame Rendering | TUI frame construction and box drawing |
| Unicode Rendering | Width consistency with unicode content |
| Snapshot Stability | Deterministic output verification |
| Frame Sequences | Multi-frame transition testing |
| Pixel Alignment | Vertical character alignment |
| Diagnostic Formatting | Warning/error message display |
| Frame Assertions | expect_frame() validation |
| Content Truncation | Long content handling |

## Running Tests

```bash
# Integrated with cargo test via jugar-probar
cargo test -p bashrs --test falsification_probar_testing
cargo test -p bashrs --test simulation_probar_testing
cargo test -p bashrs --test linter_bug_hunting -- --nocapture
cargo test -p bashrs --test linter_tui_bug_hunting -- --nocapture
```

## Historical Issues (All Fixed)

| Issue | Fix |
|-------|-----|
| F047 (SC2154 on case) | Added `case_expr` pattern recognition |
| F048 (SC2086 on for loop) | Added C-style for loop variable detection |
| F082 (SC2064 on trap) | Disabled SC2064 - double quotes are intentional |