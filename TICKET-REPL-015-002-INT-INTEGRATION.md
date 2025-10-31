# TICKET-REPL-015-002-INT: Integrate Syntax Highlighting with REPL

**Sprint**: REPL-015 (DevEx Improvements)
**Status**: IN PROGRESS
**Priority**: HIGH
**Methodology**: EXTREME TDD
**Parent**: REPL-015-002 (Syntax Highlighting - completed)

## Problem Statement

The syntax highlighting feature (REPL-015-002) is fully implemented with comprehensive tests, but it's **not integrated into the actual REPL**. The `ReplCompleter` has an empty `Highlighter` trait implementation, so users don't see colored syntax in the interactive REPL.

**Current Behavior** (No highlighting):
```
bashrs> echo $HOME
echo $HOME
```

**Desired Behavior** (With highlighting):
```
bashrs> echo $HOME
echo $HOME
^^^^      ^^^^^
cyan      yellow
(command) (variable)
```

## Requirements

### Functional Requirements

1. **Implement Highlighter trait for ReplCompleter**
   - Override `highlight()` method
   - Call `highlight_bash()` from highlighting module
   - Return `Cow::Owned` with ANSI codes

2. **Live syntax highlighting as user types**
   - Update on every keystroke
   - No perceptible lag (<10ms)
   - Works with multiline input

3. **Compatible with existing features**
   - Tab completion still works
   - History navigation still works
   - Copy/paste still works

### Non-Functional Requirements

1. **Performance**: Highlighting <10ms for typical input (<100 chars)
2. **Correctness**: Preserves all original text (verified by tests)
3. **Compatibility**: Works in all terminals supporting ANSI codes
4. **Graceful degradation**: Falls back to plain text if terminal doesn't support colors

## Design

### Simple Integration (Minimal Changes)

```rust
// rash/src/repl/completion.rs

use crate::repl::highlighting::highlight_bash;
use std::borrow::Cow;

impl Highlighter for ReplCompleter {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        // Apply syntax highlighting
        Cow::Owned(highlight_bash(line))
    }

    fn highlight_char(&self, _line: &str, _pos: usize) -> bool {
        // Highlight character-by-character as user types
        true
    }
}
```

That's it! The rustyline library will automatically:
- Call `highlight()` on every keystroke
- Display the colored output
- Strip ANSI codes when saving to history

## Test Specifications

### Unit Tests (Correctness)

#### Test: REPL-015-002-INT-001 - Highlighter integration basic
```rust
#[test]
fn test_REPL_015_002_INT_001_highlighter_basic() {
    let completer = ReplCompleter::new();

    let input = "echo hello";
    let highlighted = completer.highlight(input, 0);

    // Should contain ANSI codes
    assert!(highlighted.contains("\x1b["));

    // Should preserve original text when stripped
    let stripped = strip_ansi_codes(&highlighted);
    assert_eq!(stripped, input);
}
```

#### Test: REPL-015-002-INT-002 - Highlight with variables
```rust
#[test]
fn test_REPL_015_002_INT_002_highlight_variables() {
    let completer = ReplCompleter::new();

    let input = "echo $HOME";
    let highlighted = completer.highlight(input, 0);

    // Should highlight 'echo' as command (cyan)
    assert!(highlighted.contains("\x1b[36mecho\x1b[0m"));

    // Should highlight '$HOME' as variable (yellow)
    assert!(highlighted.contains("\x1b[33m$HOME\x1b[0m"));
}
```

#### Test: REPL-015-002-INT-003 - Highlight with keywords
```rust
#[test]
fn test_REPL_015_002_INT_003_highlight_keywords() {
    let completer = ReplCompleter::new();

    let input = "if [ -f test ]; then echo found; fi";
    let highlighted = completer.highlight(input, 0);

    // Should highlight keywords (blue)
    assert!(highlighted.contains("\x1b[1;34mif\x1b[0m"));
    assert!(highlighted.contains("\x1b[1;34mthen\x1b[0m"));
    assert!(highlighted.contains("\x1b[1;34mfi\x1b[0m"));
}
```

#### Test: REPL-015-002-INT-004 - Highlight multiline input
```rust
#[test]
fn test_REPL_015_002_INT_004_highlight_multiline() {
    let completer = ReplCompleter::new();

    let input = "for i in 1 2 3\ndo echo $i\ndone";
    let highlighted = completer.highlight(input, 0);

    // Should highlight keywords across lines
    assert!(highlighted.contains("\x1b[1;34mfor\x1b[0m"));
    assert!(highlighted.contains("\x1b[1;34mdo\x1b[0m"));
    assert!(highlighted.contains("\x1b[1;34mdone\x1b[0m"));

    // Should highlight variable
    assert!(highlighted.contains("\x1b[33m$i\x1b[0m"));
}
```

#### Test: REPL-015-002-INT-005 - Empty input
```rust
#[test]
fn test_REPL_015_002_INT_005_empty_input() {
    let completer = ReplCompleter::new();

    let highlighted = completer.highlight("", 0);

    // Should handle empty input gracefully
    assert_eq!(highlighted, "");
}
```

#### Test: REPL-015-002-INT-006 - Special characters
```rust
#[test]
fn test_REPL_015_002_INT_006_special_characters() {
    let completer = ReplCompleter::new();

    let input = "echo \"test\" | grep 'pattern' && exit 0";
    let highlighted = completer.highlight(input, 0);

    // Should highlight strings (green)
    assert!(highlighted.contains("\x1b[32m\"test\"\x1b[0m"));
    assert!(highlighted.contains("\x1b[32m'pattern'\x1b[0m"));

    // Should highlight operators (magenta)
    assert!(highlighted.contains("\x1b[35m|\x1b[0m"));
    assert!(highlighted.contains("\x1b[35m&&\x1b[0m"));
}
```

### Integration Tests (Live REPL)

#### Test: REPL-015-002-INT-INT-001 - Visual verification (manual)
```bash
# Manual test: Start REPL and verify colors appear

$ bashrs repl

# Type various bash constructs and verify colors:
bashrs> echo hello            # "echo" should be cyan
bashrs> echo $HOME            # "$HOME" should be yellow
bashrs> if true; then echo "yes"; fi   # keywords blue, string green
bashrs> for i in 1 2 3; do echo $i; done  # keywords blue, variable yellow
bashrs> # This is a comment   # comment should be gray
bashrs> cat file | grep pattern  # operators should be magenta
```

### Property Tests

#### Property: REPL-015-002-INT-PROP-001 - Highlighting never corrupts input
```rust
proptest! {
    #[test]
    fn prop_highlighting_never_corrupts_input(
        input in "[a-zA-Z ${}#|&;]{0,100}"
    ) {
        let completer = ReplCompleter::new();
        let highlighted = completer.highlight(&input, 0);

        // Strip ANSI codes and verify original text preserved
        let stripped = strip_ansi_codes(&highlighted);
        prop_assert_eq!(stripped, input);
    }
}
```

#### Property: REPL-015-002-INT-PROP-002 - Highlighting is idempotent
```rust
proptest! {
    #[test]
    fn prop_highlighting_idempotent(
        input in "[a-zA-Z ${}#|&;]{0,100}"
    ) {
        let completer = ReplCompleter::new();

        // Apply highlighting twice
        let highlighted1 = completer.highlight(&input, 0);
        let stripped = strip_ansi_codes(&highlighted1);
        let highlighted2 = completer.highlight(&stripped, 0);

        // Results should be identical
        prop_assert_eq!(highlighted1, highlighted2);
    }
}
```

## Implementation Plan

### Step 1: RED Phase (Write Failing Tests)
1. Add imports to completion.rs
2. Write 6 unit tests (all should pass immediately since highlighting module works)
3. Write 2 property tests
4. Tests verify integration, not highlighting logic itself

### Step 2: GREEN Phase (Implement Integration)
1. Implement `highlight()` method - call `highlight_bash()`
2. Implement `highlight_char()` - return true for live updates
3. Run tests - should pass
4. Manual verification in live REPL

### Step 3: REFACTOR Phase
1. Ensure no clippy warnings
2. Add rustdoc to highlight() implementation
3. Performance check (<10ms for typical input)

### Step 4: MANUAL VERIFICATION
1. Start `bashrs repl`
2. Type various bash constructs
3. Verify colors appear correctly
4. Screenshot for documentation

## Quality Gates

- [ ] ✅ All unit tests pass (6 tests)
- [ ] ✅ All property tests pass (2 tests, 100+ cases each)
- [ ] ✅ No clippy warnings
- [ ] ✅ Performance <10ms for 100-char input
- [ ] ✅ Manual verification: Colors visible in live REPL
- [ ] ✅ Terminal compatibility: Works in common terminals (xterm, gnome-terminal, iTerm2)
- [ ] ✅ Text preservation: Stripped output equals input

## Dependencies

**None** - All dependencies already in place:
- `highlight_bash()` function from REPL-015-002 ✅
- `rustyline::highlight::Highlighter` trait ✅
- `ReplCompleter` struct ✅

## Risks

1. **Performance impact on typing** - Highlighting on every keystroke might lag
   - Mitigation: Benchmark shows 31ms for 1000 lines, so 100 chars should be <1ms
   - Pre-compiled regexes ensure fast execution

2. **Terminal compatibility** - Some terminals don't support ANSI codes
   - Mitigation: Graceful degradation (rustyline handles this)
   - No errors, just plain text fallback

3. **Multiline input** - Highlighting across line boundaries
   - Mitigation: `highlight_bash()` already handles multiline correctly

## Success Criteria

1. Colors appear in live REPL when typing bash code ✅
2. Keywords highlighted in blue ✅
3. Strings highlighted in green ✅
4. Variables highlighted in yellow ✅
5. Commands highlighted in cyan ✅
6. Operators highlighted in magenta ✅
7. Comments highlighted in gray ✅
8. No perceptible lag when typing ✅
9. Text preservation verified by tests ✅

## Estimated Effort

**Time**: 30 minutes
- 10 min: Write tests
- 10 min: Implement integration (2 methods)
- 10 min: Manual verification and screenshots

**Complexity**: LOW (simple trait implementation)

---

**Created**: 2025-10-31
**Author**: Claude (EXTREME TDD)
**Roadmap**: docs/REPL-DEBUGGER-ROADMAP.yaml
**Sprint**: REPL-015 (DevEx Improvements)
**Parent Task**: REPL-015-002 (Syntax Highlighting)
