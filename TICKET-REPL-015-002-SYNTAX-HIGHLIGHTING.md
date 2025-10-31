# TICKET-REPL-015-002: Syntax Highlighting in REPL

**Sprint**: REPL-015 (DevEx Improvements)
**Status**: IN PROGRESS
**Priority**: MEDIUM
**Methodology**: EXTREME TDD

## Problem Statement

The bashrs REPL currently displays all input in plain text. Users cannot visually distinguish between different bash constructs (keywords, strings, variables, commands).

**Current Behavior** (No highlighting):
```
bashrs> if [ -f /tmp/test ]; then echo "found"; fi
if [ -f /tmp/test ]; then echo "found"; fi
```

**Desired Behavior** (With highlighting):
```
bashrs> if [ -f /tmp/test ]; then echo "found"; fi
if [ -f /tmp/test ]; then echo "found"; fi
^^ keyword        ^^ string    ^^ keyword
```

(Note: In actual terminal, these would be colored)

## Requirements

### Functional Requirements

1. **Keyword Highlighting**
   - Bash keywords: `if`, `then`, `else`, `fi`, `for`, `while`, `do`, `done`, `case`, `esac`, `function`
   - Color: Bold Blue

2. **String Highlighting**
   - Double-quoted strings: `"hello world"`
   - Single-quoted strings: `'hello world'`
   - Color: Green

3. **Variable Highlighting**
   - Simple variables: `$var`, `${var}`
   - Special variables: `$?`, `$$`, `$!`, `$@`, `$#`
   - Color: Yellow

4. **Command Highlighting**
   - First word in command: `echo`, `mkdir`, `grep`
   - Color: Cyan

5. **Comment Highlighting**
   - Comments: `# this is a comment`
   - Color: Gray (dim)

6. **Operator Highlighting**
   - Pipes: `|`, `|&`
   - Redirects: `>`, `>>`, `<`, `2>&1`
   - Logic: `&&`, `||`, `;`
   - Color: Magenta

### Non-Functional Requirements

1. **Performance**: Highlighting < 1ms for typical input (<1000 chars)
2. **Compatibility**: Works in all terminals (graceful degradation)
3. **Consistency**: Same colors as popular bash syntax highlighters
4. **No Dependencies**: Use rustyline's built-in ANSI support

## Design

### Rustyline Integration

We'll implement the `Highlighter` trait from rustyline:

```rust
pub trait Highlighter {
    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        let _ = pos;
        Borrowed(line)
    }

    fn highlight_char(&self, line: &str, pos: usize) -> bool {
        let _ = (line, pos);
        false
    }

    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> Cow<'b, str> {
        let _ = default;
        Borrowed(prompt)
    }
}
```

We'll override the `highlight()` method to add ANSI color codes.

### ANSI Color Codes

```rust
const RESET: &str = "\x1b[0m";
const BOLD_BLUE: &str = "\x1b[1;34m";    // Keywords
const GREEN: &str = "\x1b[32m";          // Strings
const YELLOW: &str = "\x1b[33m";         // Variables
const CYAN: &str = "\x1b[36m";           // Commands
const GRAY: &str = "\x1b[90m";           // Comments
const MAGENTA: &str = "\x1b[35m";        // Operators
```

### Token Types

```rust
#[derive(Debug, Clone, PartialEq)]
enum TokenType {
    Keyword,      // if, then, while, for, do, done, case, esac, function
    String,       // "..." or '...'
    Variable,     // $var, ${var}, $?
    Command,      // First word in pipeline
    Comment,      // # comment
    Operator,     // |, >, <, &&, ||, ;
    Whitespace,   // Spaces, tabs
    Text,         // Everything else
}

#[derive(Debug, Clone)]
struct Token {
    token_type: TokenType,
    text: String,
    start: usize,
    end: usize,
}
```

### Algorithm

1. **Tokenize**: Parse input into tokens
2. **Classify**: Determine token type (keyword, string, variable, etc.)
3. **Colorize**: Wrap each token with appropriate ANSI codes
4. **Reconstruct**: Build highlighted string

## Function Specifications

### 1. `tokenize(input: &str) -> Vec<Token>`

**Purpose**: Break input into tokens

**Logic**:
1. Iterate through characters
2. Recognize strings (quote handling)
3. Recognize variables ($, ${})
4. Recognize words (split on whitespace/operators)
5. Recognize operators (|, >, <, etc.)

**Returns**: Vector of tokens

### 2. `highlight_token(token: &Token) -> String`

**Purpose**: Wrap token with ANSI color codes

**Logic**:
1. Match on token_type
2. Return `COLOR + text + RESET`

**Returns**: Colored string

### 3. `highlight_bash(input: &str) -> String`

**Purpose**: Main highlighting function

**Logic**:
1. Tokenize input
2. Highlight each token
3. Concatenate results

**Returns**: Highlighted string with ANSI codes

### 4. `is_keyword(word: &str) -> bool`

**Purpose**: Check if word is bash keyword

**Logic**:
1. Check against keyword list
2. Return true/false

**Returns**: Boolean

### 5. Implementation in ReplCompleter

```rust
impl Highlighter for ReplCompleter {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        // Apply syntax highlighting
        Cow::Owned(highlight_bash(line))
    }
}
```

## Test Specifications

### Unit Tests

#### Test: REPL-015-002-001 - Highlight keywords
```rust
#[test]
fn test_REPL_015_002_001_highlight_keywords() {
    let input = "if then else fi";
    let highlighted = highlight_bash(input);

    assert!(highlighted.contains("\x1b[1;34mif\x1b[0m"));
    assert!(highlighted.contains("\x1b[1;34mthen\x1b[0m"));
    assert!(highlighted.contains("\x1b[1;34melse\x1b[0m"));
    assert!(highlighted.contains("\x1b[1;34mfi\x1b[0m"));
}
```

#### Test: REPL-015-002-002 - Highlight strings
```rust
#[test]
fn test_REPL_015_002_002_highlight_strings() {
    let input = r#"echo "hello world" 'single'"#;
    let highlighted = highlight_bash(input);

    assert!(highlighted.contains("\x1b[32m\"hello world\"\x1b[0m"));
    assert!(highlighted.contains("\x1b[32m'single'\x1b[0m"));
}
```

#### Test: REPL-015-002-003 - Highlight variables
```rust
#[test]
fn test_REPL_015_002_003_highlight_variables() {
    let input = "echo $HOME ${USER} $?";
    let highlighted = highlight_bash(input);

    assert!(highlighted.contains("\x1b[33m$HOME\x1b[0m"));
    assert!(highlighted.contains("\x1b[33m${USER}\x1b[0m"));
    assert!(highlighted.contains("\x1b[33m$?\x1b[0m"));
}
```

#### Test: REPL-015-002-004 - Highlight commands
```rust
#[test]
fn test_REPL_015_002_004_highlight_commands() {
    let input = "mkdir -p /tmp";
    let highlighted = highlight_bash(input);

    // First word should be highlighted as command
    assert!(highlighted.contains("\x1b[36mmkdir\x1b[0m"));
}
```

#### Test: REPL-015-002-005 - Highlight comments
```rust
#[test]
fn test_REPL_015_002_005_highlight_comments() {
    let input = "echo hello # this is a comment";
    let highlighted = highlight_bash(input);

    assert!(highlighted.contains("\x1b[90m# this is a comment\x1b[0m"));
}
```

#### Test: REPL-015-002-006 - Highlight operators
```rust
#[test]
fn test_REPL_015_002_006_highlight_operators() {
    let input = "cat file | grep pattern && echo done";
    let highlighted = highlight_bash(input);

    assert!(highlighted.contains("\x1b[35m|\x1b[0m"));
    assert!(highlighted.contains("\x1b[35m&&\x1b[0m"));
}
```

### Integration Tests

#### Test: REPL-015-002-INT-001 - Full bash statement
```rust
#[test]
fn test_REPL_015_002_INT_001_full_statement() {
    let input = r#"if [ -f "$file" ]; then echo "found"; fi"#;
    let highlighted = highlight_bash(input);

    // Should have keyword highlighting
    assert!(highlighted.contains("\x1b[1;34mif\x1b[0m"));
    assert!(highlighted.contains("\x1b[1;34mthen\x1b[0m"));
    assert!(highlighted.contains("\x1b[1;34mfi\x1b[0m"));

    // Should have string highlighting
    assert!(highlighted.contains("\x1b[32m\"$file\"\x1b[0m"));
    assert!(highlighted.contains("\x1b[32m\"found\"\x1b[0m"));

    // Should have operator highlighting
    assert!(highlighted.contains("\x1b[35m;\x1b[0m"));
}
```

### Property Tests

#### Property: Highlighting never panics
```rust
proptest! {
    #[test]
    fn prop_highlighting_never_panics(input in ".*{0,1000}") {
        // Should never panic on any input
        let _ = highlight_bash(&input);
    }
}
```

#### Property: Highlighting preserves text
```rust
proptest! {
    #[test]
    fn prop_highlighting_preserves_text(input in "[a-zA-Z ]{0,100}") {
        let highlighted = highlight_bash(&input);

        // Strip ANSI codes
        let stripped = strip_ansi_codes(&highlighted);

        // Text should be preserved
        assert_eq!(stripped, input);
    }
}
```

## EXTREME TDD Phases

### RED Phase ✅ (Write Failing Tests)
1. Create module: `rash/src/repl/highlighting.rs`
2. Define Token and TokenType enums
3. Write stub functions (all return unimplemented!())
4. Write 6 unit tests (all should fail)
5. Write 1 integration test
6. Write 2 property tests
7. Update ReplCompleter to use highlighting
8. Run: `cargo test test_REPL_015_002` (should FAIL ❌)

### GREEN Phase 🟢 (Make Tests Pass)
1. Implement `tokenize()`
2. Implement `is_keyword()`
3. Implement `highlight_token()`
4. Implement `highlight_bash()`
5. Implement `strip_ansi_codes()` for testing
6. Update ReplCompleter Highlighter trait
7. Run: `cargo test test_REPL_015_002` (should PASS ✅)

### REFACTOR Phase 🔄 (Clean Up)
1. Extract constants (keywords, colors)
2. Optimize tokenization
3. Run `cargo clippy --lib`
4. Check complexity < 10
5. Add rustdoc comments

### PROPERTY Phase 🎲 (Generative Testing)
1. Run property tests with 100+ cases
2. Verify no panics
3. Verify text preservation

### MUTATION Phase 🧬 (Mutation Testing)
1. Run `cargo mutants --file rash/src/repl/highlighting.rs`
2. Target: ≥90% kill rate
3. Add tests for surviving mutants

### COMMIT Phase 📝 (Git Commit)
1. Update `docs/REPL-DEBUGGER-ROADMAP.yaml`
2. Mark REPL-015-002 and REPL-015 sprint as completed
3. Create commit with quality metrics

## Quality Gates

- [ ] ✅ All unit tests pass (6 tests)
- [ ] ✅ Integration test passes (1 test)
- [ ] ✅ Property tests pass (2 tests, 100+ cases each)
- [ ] ✅ No clippy warnings
- [ ] ✅ Function complexity < 10
- [ ] ✅ Mutation score ≥ 90%
- [ ] ✅ Performance < 1ms for typical input
- [ ] ✅ Visual verification in terminal

## Dependencies

- `rustyline::highlight::Highlighter` - Trait to implement
- ANSI escape codes - For terminal colors

## Risks

1. **Terminal compatibility** - Some terminals don't support colors
   - Mitigation: Graceful degradation (no colors)
2. **Performance** - Complex highlighting might be slow
   - Mitigation: Simple tokenization, no regex
3. **False positives** - Might highlight non-keywords
   - Mitigation: Strict keyword list

## Success Criteria

1. Keywords highlighted in blue ✅
2. Strings highlighted in green ✅
3. Variables highlighted in yellow ✅
4. Commands highlighted in cyan ✅
5. Operators highlighted in magenta ✅
6. Performance < 1ms ✅
7. No crashes on any input ✅
8. Visual verification in terminal ✅

## Example Output

Before (no highlighting):
```
bashrs> for i in 1 2 3; do echo $i; done
for i in 1 2 3; do echo $i; done
```

After (with highlighting - colors shown as text):
```
bashrs> [BLUE]for[RESET] i [BLUE]in[RESET] 1 2 3[MAGENTA];[RESET] [BLUE]do[RESET] [CYAN]echo[RESET] [YELLOW]$i[RESET][MAGENTA];[RESET] [BLUE]done[RESET]
```

---

**Created**: 2024-10-31
**Author**: Claude (EXTREME TDD)
**Roadmap**: docs/REPL-DEBUGGER-ROADMAP.yaml
**Sprint**: REPL-015 (DevEx Improvements)
