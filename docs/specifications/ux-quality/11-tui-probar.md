# TUI + Probar Testing Specification

**Document ID**: SPEC-TUI-2025-001
**Version**: 1.0.0
**Status**: DRAFT
**Created**: 2025-12-15
**Methodology**: Toyota Production System (TPS) + Probar Testing

---

## 1. Overview

This specification defines the TUI (Terminal User Interface) implementation for bashrs using ratatui, with comprehensive testing via the probar framework (`jugar-probar`).

### 1.1 Goals

1. **Multi-panel TUI** for interactive shell analysis
2. **Edge case detection** through fuzzing and property testing
3. **95%+ GUI coverage** via probar testing
4. **Deterministic replay** for bug reproduction

---

## 2. TUI Architecture

### 2.1 Panel Layout

```
┌─────────────────────────────────────────────────────────────────┐
│ bashrs TUI v6.44.0                              [F1:Help] [q:Quit] │
├─────────────────────────────┬───────────────────────────────────┤
│ EDITOR                      │ LINT RESULTS                      │
│                             │                                   │
│ #!/bin/bash                 │ [!] SEC003:2 - Command injection  │
│ rm -rf $dir                 │ [!] DET001:2 - Non-deterministic  │
│ echo $RANDOM                │ [i] SC2086:2 - Unquoted variable  │
│                             │                                   │
├─────────────────────────────┼───────────────────────────────────┤
│ PURIFIED                    │ QUALITY METRICS                   │
│                             │                                   │
│ #!/bin/sh                   │ Coverage: 95.6% ████████████░░    │
│ rm -rf "$dir"               │ Mutation: 87.2% █████████░░░░░    │
│ # $RANDOM removed           │ Lint:     A-    No critical       │
│                             │ Grade:    B+    85/100 points     │
├─────────────────────────────┴───────────────────────────────────┤
│ STATUS: Ready │ Mode: Normal │ Tests: 7305 │ Edges: 42 detected │
└─────────────────────────────────────────────────────────────────┘
```

### 2.2 Panels

| Panel | Purpose | Key Binding |
|-------|---------|-------------|
| **Editor** | Script input/editing | `Tab` to focus |
| **Lint Results** | Real-time lint diagnostics | `F2` |
| **Purified** | Deterministic/idempotent output | `F3` |
| **Quality Metrics** | Coverage, mutation, grades | `F4` |
| **Edge Cases** | Detected edge cases from fuzzing | `F5` |
| **Status Bar** | Mode, test count, stats | Always visible |

### 2.3 Modes

| Mode | Description | Key |
|------|-------------|-----|
| Normal | Direct editing and viewing | `1` |
| Purify | Show purification transforms | `2` |
| Lint | Highlight lint issues | `3` |
| Debug | Step-through execution | `4` |
| Explain | Educational explanations | `5` |
| Fuzz | Monte Carlo edge case detection | `6` |

---

## 3. Probar Testing Requirements

### 3.1 Dependencies

```toml
[dev-dependencies]
jugar-probar = "0.3"   # From ../probar

[dependencies]
ratatui = { version = "0.29", default-features = false, features = ["crossterm"] }
crossterm = "0.28"
```

### 3.2 GUI Coverage Requirements

**REQ-TUI-001**: GUI coverage MUST achieve >= 95%.

```rust
use jugar_probar::prelude::*;

#[test]
fn test_tui_gui_coverage() {
    let mut gui = gui_coverage! {
        panels: ["editor", "lint", "purified", "quality", "edges", "status"],
        buttons: ["run", "purify", "lint", "debug", "explain", "fuzz", "help", "quit"],
        modes: ["normal", "purify", "lint", "debug", "explain", "fuzz"],
        screens: ["main", "help", "settings", "edge_details"]
    };

    // Test all panels
    gui.visit("editor"); gui.visit("lint"); gui.visit("purified");
    gui.visit("quality"); gui.visit("edges"); gui.visit("status");

    // Test all modes
    gui.click("normal"); gui.click("purify"); gui.click("lint");
    gui.click("debug"); gui.click("explain"); gui.click("fuzz");

    // Test all buttons
    gui.click("run"); gui.click("help"); gui.click("quit");

    // Verify 95%+ coverage
    assert!(gui.meets(95.0), "GUI coverage: {}", gui.summary());
}
```

### 3.3 TUI Frame Testing

**REQ-TUI-002**: TUI frames MUST be captured and asserted.

```rust
use jugar_probar::tui::*;

#[test]
fn test_tui_frame_capture() {
    let mut tui = TuiTestHarness::new();

    // Render initial state
    tui.render();

    // Capture frame
    let frame = tui.capture_frame();

    // Assert frame contains expected elements
    assert!(frame.contains("bashrs TUI"));
    assert!(frame.contains("EDITOR"));
    assert!(frame.contains("LINT RESULTS"));
    assert!(frame.contains("PURIFIED"));
    assert!(frame.contains("QUALITY METRICS"));
}
```

### 3.4 Deterministic Replay

**REQ-TUI-003**: TUI sessions MUST be replayable deterministically.

```rust
use jugar_probar::replay::*;

#[test]
fn test_tui_deterministic_replay() {
    let seed = 42u64;

    // Record session
    let session = TuiSession::record_with_seed(seed, || {
        // Simulate user actions
        send_key(KeyCode::Tab);  // Focus editor
        send_text("echo $RANDOM");
        send_key(KeyCode::F2);   // Show lint
    });

    // Replay with same seed
    let replay1 = session.replay_with_seed(seed);
    let replay2 = session.replay_with_seed(seed);

    // Must be identical
    assert_eq!(replay1.final_state(), replay2.final_state());
}
```

### 3.5 Monte Carlo Fuzzing

**REQ-TUI-004**: Edge cases MUST be discovered via Monte Carlo fuzzing.

```rust
use jugar_probar::monte_carlo::*;

#[test]
fn test_tui_monte_carlo_fuzzing() {
    let mut fuzzer = MonteCarloFuzzer::new()
        .iterations(1000)
        .seed(12345);

    let results = fuzzer.fuzz(|input| {
        let mut tui = TuiTestHarness::new();

        // Feed random input
        tui.send_text(&input);
        tui.render();

        // Invariant: TUI must not panic
        // Invariant: TUI must remain responsive
        assert!(tui.is_responsive());
    });

    // Report edge cases found
    println!("Edge cases discovered: {}", results.edge_cases.len());
    for edge in &results.edge_cases {
        println!("  - {}: {}", edge.category, edge.description);
    }
}
```

### 3.6 Visual Regression Testing

**REQ-TUI-005**: TUI appearance MUST not regress.

```rust
use jugar_probar::visual::*;

#[test]
fn test_tui_visual_regression() {
    let mut tui = TuiTestHarness::new();
    tui.set_size(80, 24);
    tui.render();

    // Compare against golden snapshot
    let snapshot = tui.capture_snapshot();
    assert_snapshot_matches!("tui_main_screen", snapshot);
}
```

---

## 4. Edge Case Detection

### 4.1 Categories

| Category | Description | Detection Method |
|----------|-------------|------------------|
| **Parser** | Malformed bash syntax | Proptest + fuzzing |
| **Unicode** | Non-ASCII characters | Monte Carlo |
| **Large Input** | Scripts > 1MB | Boundary testing |
| **Deep Nesting** | Deeply nested structures | Recursive fuzzing |
| **Special Chars** | Control chars, null bytes | Injection testing |
| **Race Conditions** | Concurrent operations | Stress testing |

### 4.2 Edge Case Registry

**REQ-TUI-006**: All discovered edge cases MUST be tracked.

```rust
// Edge case tracking
pub struct EdgeCase {
    pub id: String,
    pub category: EdgeCategory,
    pub input: String,
    pub expected: String,
    pub actual: String,
    pub discovered: DateTime<Utc>,
    pub status: EdgeStatus,  // Open, Fixed, WontFix
}

pub enum EdgeCategory {
    Parser,
    Unicode,
    LargeInput,
    DeepNesting,
    SpecialChars,
    RaceCondition,
}
```

### 4.3 Proptest Integration

**REQ-TUI-007**: Property tests MUST cover TUI components.

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_tui_handles_any_bash_input(input in ".*") {
        let mut tui = TuiTestHarness::new();
        tui.send_text(&input);
        tui.render();

        // Must not panic, must remain valid state
        prop_assert!(tui.is_valid_state());
    }

    #[test]
    fn prop_tui_lint_deterministic(script in valid_bash_script()) {
        let mut tui1 = TuiTestHarness::new();
        let mut tui2 = TuiTestHarness::new();

        tui1.send_text(&script);
        tui2.send_text(&script);

        tui1.run_lint();
        tui2.run_lint();

        // Same input must produce same output
        prop_assert_eq!(tui1.lint_results(), tui2.lint_results());
    }
}
```

---

## 5. State Machine Validation

### 5.1 TUI State Machine

```yaml
# tui-playbook.yaml
name: bashrs-tui-states
initial: idle

states:
  idle:
    transitions:
      - on: focus_editor -> editing
      - on: press_help -> help_screen
      - on: press_quit -> terminated

  editing:
    transitions:
      - on: press_f2 -> linting
      - on: press_f3 -> purifying
      - on: press_escape -> idle

  linting:
    transitions:
      - on: lint_complete -> showing_results
      - on: lint_error -> error_state

  purifying:
    transitions:
      - on: purify_complete -> showing_purified
      - on: purify_error -> error_state

  showing_results:
    transitions:
      - on: press_enter -> editing
      - on: press_escape -> idle

  help_screen:
    transitions:
      - on: press_escape -> idle
      - on: press_any -> idle

  error_state:
    transitions:
      - on: acknowledge -> idle

  terminated:
    final: true
```

### 5.2 Playbook Validation

**REQ-TUI-008**: State machine MUST be validated with probador.

```bash
# Validate state machine
probador playbook tui-playbook.yaml --validate

# Run with mutation testing
probador playbook tui-playbook.yaml --mutate

# Export state diagram
probador playbook tui-playbook.yaml --export svg -o tui-states.svg
```

---

## 6. Implementation Phases

### Phase 1: Core TUI (Week 1-2)
- [ ] Add ratatui/crossterm dependencies
- [ ] Implement basic 4-panel layout
- [ ] Wire up existing REPL components
- [ ] Basic keyboard navigation

### Phase 2: Probar Integration (Week 3)
- [ ] Add jugar-probar dev-dependency
- [ ] Write GUI coverage tests
- [ ] Implement frame capture
- [ ] Create golden snapshots

### Phase 3: Edge Case Detection (Week 4)
- [ ] Monte Carlo fuzzer integration
- [ ] Edge case registry
- [ ] Proptest TUI properties
- [ ] Visual regression baseline

### Phase 4: State Machine (Week 5)
- [ ] Define state machine YAML
- [ ] probador validation
- [ ] Full GUI coverage (95%+)
- [ ] Documentation

---

## 7. Testing Checklist

### 7.1 Probar Test Suite

| Test ID | Description | Coverage Target |
|---------|-------------|-----------------|
| TUI-001 | GUI coverage >= 95% | All panels, modes, buttons |
| TUI-002 | Frame capture assertions | All screens |
| TUI-003 | Deterministic replay | 100% reproducible |
| TUI-004 | Monte Carlo fuzzing | 1000+ iterations |
| TUI-005 | Visual regression | Golden snapshots |
| TUI-006 | Edge case registry | All categories tracked |
| TUI-007 | Proptest properties | 100+ cases each |
| TUI-008 | State machine validation | All transitions |

### 7.2 Quality Gates

```bash
# All TUI tests must pass
cargo test --lib -p bashrs --features tui

# GUI coverage check
cargo test test_tui_gui_coverage -- --nocapture

# Probar playbook validation
probador playbook docs/specifications/ux-quality/tui-playbook.yaml --validate

# Visual regression
cargo test test_tui_visual_regression
```

---

## 8. Falsifiable Acceptance Criteria

**REQ-TUI-ACCEPT-001**: TUI MUST achieve 95%+ GUI coverage.
```bash
cargo test test_tui_gui_coverage 2>&1 | grep "GUI coverage"
# EXPECTED: "GUI coverage: 95%+" or higher
```

**REQ-TUI-ACCEPT-002**: TUI MUST pass 1000 Monte Carlo iterations without panic.
```bash
cargo test test_tui_monte_carlo_fuzzing 2>&1 | grep "iterations"
# EXPECTED: "1000 iterations completed, 0 panics"
```

**REQ-TUI-ACCEPT-003**: TUI state machine MUST pass probador validation.
```bash
probador playbook tui-playbook.yaml --validate
# EXPECTED: "State machine valid: 8 states, 12 transitions"
```

**REQ-TUI-ACCEPT-004**: All discovered edge cases MUST be tracked.
```bash
cargo test --lib | grep "edge_cases"
# EXPECTED: Edge case count tracked in registry
```

---

---

## 9. Simulation Testing (Edge Case Discovery)

### 9.1 Overview

Simulation testing uses structured test cases to surface edge cases across 10 categories.
Each test validates that bashrs handles the input without panicking.

**Location**: `tests/simulation/run.py`

### 9.2 Test Categories

| Category | S-Codes | Count | Description |
|----------|---------|-------|-------------|
| Unicode | S101-S110 | 10 | Non-ASCII, emoji, RTL, combining chars |
| Boundary | S201-S210 | 10 | Large inputs, 10KB vars, 500-line heredocs |
| Nesting | S301-S310 | 10 | Deep nesting (10+ levels of if/while/for) |
| Special | S401-S410 | 10 | Control chars, tabs, CR, escape sequences |
| Malformed | S501-S510 | 10 | Unclosed braces, missing fi/done/esac |
| Timing | S601-S610 | 10 | PIDs, RANDOM, trap, eval, FD manipulation |
| Resource | S701-S710 | 10 | Long names (100+ chars), many args |
| Escape | S801-S810 | 10 | Hex, octal, unicode, ANSI-C escapes |
| Quoting | S901-S910 | 10 | Empty quotes, mixed quotes, escapes |
| Stress | S1001-S1010 | 10 | Combined edge cases, unicode + arrays |

**Total**: 100 simulation tests

### 9.3 Test Expectations

Each test has an expected outcome:
- **pass**: Must not panic, may produce warnings
- **error**: Must handle gracefully (exit code may be non-zero)
- **parse**: Parser must handle without crashing

### 9.4 Running Simulation Tests

```bash
# Run all simulation tests
python3 tests/simulation/run.py

# Expected output:
# Total: 100
# Passed: 100
# Failed: 0
```

### 9.5 Adding New Simulation Tests

```python
# In tests/simulation/run.py
SimulationCase("S1011", "your_code_here", "pass", "Description", "category"),
```

### 9.6 Edge Case Examples

**S101 - Unicode Latin Extended**:
```bash
echo 'héllo wörld'
```

**S208 - 20 Nested Expansions**:
```bash
echo ${x:-${x:-${x:-...default...}}}
```

**S501 - Unclosed Brace (Graceful Error)**:
```bash
echo ${
```

**S1003 - Emoji Array**:
```bash
arr=(🚀 🔥 💻); echo ${arr[@]}
```

---

## 10. Quality Gate Summary

| Test Suite | Location | Count | Target |
|------------|----------|-------|--------|
| Falsification | `tests/falsification/run.py` | 120 | 100% pass |
| Simulation | `tests/simulation/run.py` | 100 | 100% pass |
| Unit Tests | `cargo test --lib` | 7400+ | 100% pass |
| Property Tests | proptest | 100+/feature | All pass |
| TUI Coverage | probar | N/A | ≥95% |

**Combined Edge Case Coverage**: 220 structured tests

---

## 11. References

- [Probar Documentation](https://github.com/paiml/probar)
- [jugar-probar crate](https://crates.io/crates/jugar-probar)
- [ratatui Documentation](https://docs.rs/ratatui)
- [bashrs REPL Architecture](../../../rash/src/repl/mod.rs)
