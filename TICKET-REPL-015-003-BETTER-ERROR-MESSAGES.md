# TICKET-REPL-015-003: Better Error Messages

**Sprint**: REPL-015 (DevEx Improvements)
**Status**: IN PROGRESS
**Priority**: HIGH
**Methodology**: EXTREME TDD

## Problem Statement

Current REPL error messages are basic and unhelpful. They don't provide enough context, suggest fixes, or guide users to solutions.

**Current Behavior** (Poor UX):
```
bashrs> if [ -f test
Error: Parse error

bashrs> mkdir /tmp
Error: Lint error: IDEM001

bashrs> :unknown
Error: Unknown command
```

**Desired Behavior** (Excellent UX):
```
bashrs> if [ -f test
✗ Parse Error at line 1, column 15

    1 | if [ -f test
      |              ^ expected: 'then' or ';'

  Suggestion: Did you forget 'then'?
  Try: if [ -f test ]; then

bashrs> mkdir /tmp
⚠ Lint Issue: IDEM001 (idempotency)

    1 | mkdir /tmp
      | ^^^^^^^^^^ mkdir without -p flag

  Problem: Command will fail if directory exists
  Fix: Use 'mkdir -p /tmp' for idempotent operation

  Auto-fix available: Run ':purify' to see safe version

bashrs> :unknown
✗ Unknown command: ':unknown'

  Did you mean: ':quit'?

  Available commands:
    :quit, :purify, :lint, :ast, :help

  Type ':help' for full command list
```

## Requirements

### Functional Requirements

1. **Parse Errors**
   - Show line and column number
   - Show source context with caret indicator
   - Suggest likely fix
   - Show what was expected vs what was found

2. **Lint Errors**
   - Show violation code (DET001, IDEM001, SEC001, etc.)
   - Show severity (error, warning, info)
   - Show source context with caret
   - Explain the problem clearly
   - Suggest fix when available
   - Mention auto-fix availability

3. **Command Errors**
   - Show what command user typed
   - Suggest similar command (edit distance)
   - List available commands
   - Link to help system

4. **Runtime Errors**
   - Show clear error message
   - Show stack trace (if applicable)
   - Suggest recovery actions
   - Don't crash the REPL

### Non-Functional Requirements

1. **Clarity**: Errors easy to understand for beginners
2. **Actionability**: Always suggest next step
3. **Consistency**: Same format across error types
4. **Performance**: Error formatting < 10ms
5. **Color**: Use ANSI colors when terminal supports it

## Inspiration: Rust Compiler Errors

Rust has best-in-class error messages. We should emulate:

```rust
error[E0308]: mismatched types
 --> src/main.rs:2:13
  |
2 |     let x: i32 = "hello";
  |            ---   ^^^^^^^ expected `i32`, found `&str`
  |            |
  |            expected due to this type

help: try using a conversion method
  |
2 |     let x: i32 = "hello".parse();
  |                        +++++++
```

Key features:
- Error code (E0308)
- File location (src/main.rs:2:13)
- Source context with carets
- Clear explanation
- Actionable suggestion

## Data Structures

### ErrorMessage

```rust
/// Structured error message
#[derive(Debug, Clone)]
pub struct ErrorMessage {
    /// Error type (Parse, Lint, Command, Runtime)
    pub error_type: ErrorType,

    /// Error code (optional, e.g., "DET001", "E001")
    pub code: Option<String>,

    /// Severity level
    pub severity: Severity,

    /// Main error message
    pub message: String,

    /// Source code context (line with error)
    pub context: Option<SourceContext>,

    /// Detailed explanation (optional)
    pub explanation: Option<String>,

    /// Suggested fix (optional)
    pub suggestion: Option<Suggestion>,

    /// Related help topics (optional)
    pub help_topics: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorType {
    Parse,
    Lint,
    Command,
    Runtime,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone)]
pub struct SourceContext {
    /// Line number (1-indexed)
    pub line: usize,

    /// Column number (1-indexed)
    pub column: usize,

    /// Source code line
    pub source_line: String,

    /// Length of problematic section
    pub length: usize,
}

#[derive(Debug, Clone)]
pub struct Suggestion {
    /// Description of the fix
    pub description: String,

    /// Fixed code (if applicable)
    pub fixed_code: Option<String>,

    /// Auto-fix available flag
    pub auto_fixable: bool,
}
```

## Function Specifications

### 1. `format_error(error: &ErrorMessage) -> String`

**Purpose**: Format error message for display

**Logic**:
1. Format header with icon, type, and code
2. Add source context with caret
3. Add explanation if available
4. Add suggestion if available
5. Add help topics if available

**Returns**: Formatted error string

**Example**:
```rust
let error = ErrorMessage {
    error_type: ErrorType::Parse,
    code: Some("P001".to_string()),
    severity: Severity::Error,
    message: "Unexpected token".to_string(),
    context: Some(SourceContext {
        line: 1,
        column: 15,
        source_line: "if [ -f test".to_string(),
        length: 1,
    }),
    explanation: Some("Expected 'then' or ';' after test expression".to_string()),
    suggestion: Some(Suggestion {
        description: "Add 'then' keyword".to_string(),
        fixed_code: Some("if [ -f test ]; then".to_string()),
        auto_fixable: false,
    }),
    help_topics: vec!["conditionals".to_string()],
};

let formatted = format_error(&error);
assert!(formatted.contains("✗ Parse Error [P001]"));
assert!(formatted.contains("if [ -f test"));
assert!(formatted.contains("^"));
assert!(formatted.contains("Expected 'then'"));
```

### 2. `format_parse_error(error: &str, line: usize, column: usize, source: &str) -> ErrorMessage`

**Purpose**: Create error message for parse errors

**Logic**:
1. Extract source context around error
2. Determine likely cause
3. Suggest fix based on context
4. Create ErrorMessage with all details

**Returns**: ErrorMessage structure

### 3. `format_lint_error(diagnostic: &Diagnostic, source: &str) -> ErrorMessage`

**Purpose**: Create error message for lint violations

**Logic**:
1. Extract source context from diagnostic span
2. Add explanation based on violation code
3. Include fix suggestion if available
4. Mark as auto-fixable if purifier can fix it

**Returns**: ErrorMessage structure

### 4. `format_command_error(command: &str, available_commands: &[&str]) -> ErrorMessage`

**Purpose**: Create error message for unknown commands

**Logic**:
1. Calculate edit distance to all commands
2. Suggest closest match if distance < 3
3. List all available commands
4. Link to help system

**Returns**: ErrorMessage structure

### 5. `suggest_command(input: &str, commands: &[&str]) -> Option<String>`

**Purpose**: Suggest similar command using edit distance

**Logic**:
1. Calculate Levenshtein distance to each command
2. Return closest match if distance < 3
3. Return None if no close matches

**Returns**: Suggested command or None

**Example**:
```rust
let commands = vec!["purify", "lint", "quit", "ast", "help"];
assert_eq!(suggest_command("purfy", &commands), Some("purify".to_string()));
assert_eq!(suggest_command("lnt", &commands), Some("lint".to_string()));
assert_eq!(suggest_command("xyz", &commands), None);
```

## Test Specifications

### Unit Tests

#### Test: REPL-015-003-001 - Format parse error
```rust
#[test]
fn test_REPL_015_003_001_format_parse_error() {
    let source = "if [ -f test";
    let error = format_parse_error("Expected 'then'", 1, 13, source);

    assert_eq!(error.error_type, ErrorType::Parse);
    assert_eq!(error.severity, Severity::Error);
    assert!(error.message.contains("Expected"));

    let formatted = format_error(&error);
    assert!(formatted.contains("✗"));
    assert!(formatted.contains("if [ -f test"));
    assert!(formatted.contains("^"));
}
```

#### Test: REPL-015-003-002 - Format lint error with fix
```rust
#[test]
fn test_REPL_015_003_002_format_lint_error() {
    let diagnostic = Diagnostic {
        code: "IDEM001".to_string(),
        severity: Severity::Error,
        message: "mkdir without -p".to_string(),
        span: Span::new(1, 1, 1, 11),
        fix: Some(Fix::new("mkdir -p /tmp")),
    };

    let source = "mkdir /tmp";
    let error = format_lint_error(&diagnostic, source);

    assert_eq!(error.error_type, ErrorType::Lint);
    assert!(error.code == Some("IDEM001".to_string()));
    assert!(error.suggestion.is_some());
    assert!(error.suggestion.as_ref().unwrap().auto_fixable);

    let formatted = format_error(&error);
    assert!(formatted.contains("IDEM001"));
    assert!(formatted.contains("mkdir /tmp"));
    assert!(formatted.contains("Auto-fix"));
}
```

#### Test: REPL-015-003-003 - Format command error with suggestion
```rust
#[test]
fn test_REPL_015_003_003_format_command_error() {
    let commands = vec!["purify", "lint", "quit", "ast"];
    let error = format_command_error("purfy", &commands);

    assert_eq!(error.error_type, ErrorType::Command);
    assert!(error.message.contains("Unknown command"));
    assert!(error.suggestion.is_some());

    let formatted = format_error(&error);
    assert!(formatted.contains("purfy"));
    assert!(formatted.contains("Did you mean"));
    assert!(formatted.contains("purify"));
}
```

#### Test: REPL-015-003-004 - Command suggestion with edit distance
```rust
#[test]
fn test_REPL_015_003_004_suggest_command() {
    let commands = vec!["purify", "lint", "quit", "ast", "help"];

    // Close matches
    assert_eq!(suggest_command("purfy", &commands), Some("purify".to_string()));
    assert_eq!(suggest_command("lnt", &commands), Some("lint".to_string()));
    assert_eq!(suggest_command("qit", &commands), Some("quit".to_string()));

    // No close matches
    assert_eq!(suggest_command("foobar", &commands), None);
    assert_eq!(suggest_command("xyz123", &commands), None);
}
```

#### Test: REPL-015-003-005 - Source context extraction
```rust
#[test]
fn test_REPL_015_003_005_source_context() {
    let source = "echo hello\nif [ -f test\necho world";
    let context = SourceContext {
        line: 2,
        column: 13,
        source_line: "if [ -f test".to_string(),
        length: 1,
    };

    let formatted = format_source_context(&context);
    assert!(formatted.contains("2 |"));
    assert!(formatted.contains("if [ -f test"));
    assert!(formatted.contains("^"));
}
```

#### Test: REPL-015-003-006 - Error severity formatting
```rust
#[test]
fn test_REPL_015_003_006_severity_formatting() {
    let error = ErrorMessage {
        error_type: ErrorType::Lint,
        code: Some("PERF001".to_string()),
        severity: Severity::Warning,
        message: "Inefficient code".to_string(),
        context: None,
        explanation: None,
        suggestion: None,
        help_topics: vec![],
    };

    let formatted = format_error(&error);
    assert!(formatted.contains("⚠")); // Warning icon
    assert!(formatted.contains("Warning"));
}
```

### Integration Tests

#### Test: REPL-015-003-INT-001 - Error messages in REPL
```rust
#[test]
fn test_REPL_015_003_INT_001_errors_in_repl() {
    let mut repl = ReplSession::new();

    // Parse error
    let output = repl.execute("if [ -f test").unwrap_err();
    assert!(output.contains("Parse Error"));
    assert!(output.contains("^"));

    // Command error
    let output = repl.execute(":purfy").unwrap_err();
    assert!(output.contains("Unknown command"));
    assert!(output.contains("purify"));

    // Lint error (in lint mode)
    repl.execute(":mode lint").unwrap();
    let output = repl.execute("mkdir /tmp").unwrap();
    assert!(output.contains("IDEM001"));
    assert!(output.contains("Auto-fix"));
}
```

### Property Tests

#### Property: Error formatting never panics
```rust
proptest! {
    #[test]
    fn prop_error_formatting_never_panics(
        message in ".*{0,1000}",
        line in 1usize..1000,
        column in 1usize..1000
    ) {
        let error = ErrorMessage {
            error_type: ErrorType::Runtime,
            code: None,
            severity: Severity::Error,
            message,
            context: None,
            explanation: None,
            suggestion: None,
            help_topics: vec![],
        };

        // Should never panic
        let _ = format_error(&error);
    }
}
```

## EXTREME TDD Phases

### RED Phase ✅ (Write Failing Tests)
1. Create module: `rash/src/repl/errors.rs`
2. Define data structures (ErrorMessage, ErrorType, Severity, etc.)
3. Write stub functions (all return `unimplemented!()`)
4. Write 6 unit tests (all should fail)
5. Write 1 integration test
6. Write 1 property test
7. Run: `cargo test test_REPL_015_003` (should FAIL ❌)

### GREEN Phase 🟢 (Make Tests Pass)
1. Implement `format_error()`
2. Implement `format_parse_error()`
3. Implement `format_lint_error()`
4. Implement `format_command_error()`
5. Implement `suggest_command()` with Levenshtein distance
6. Integrate error formatting into REPL
7. Run: `cargo test test_REPL_015_003` (should PASS ✅)

### REFACTOR Phase 🔄 (Clean Up)
1. Extract constants (error icons, colors)
2. Add ANSI color support (optional)
3. Run `cargo clippy --lib`
4. Check complexity < 10
5. Add rustdoc comments
6. Run full test suite

### PROPERTY Phase 🎲 (Generative Testing)
1. Add more property tests
2. Fuzz error messages with random input
3. Verify no panics

### MUTATION Phase 🧬 (Mutation Testing)
1. Run `cargo mutants --file rash/src/repl/errors.rs`
2. Target: ≥90% kill rate
3. Add tests for surviving mutants

### COMMIT Phase 📝 (Git Commit)
1. Update `docs/REPL-DEBUGGER-ROADMAP.yaml`
2. Create commit with quality metrics

## Quality Gates

- [ ] ✅ All unit tests pass (6 tests)
- [ ] ✅ Integration test passes (1 test)
- [ ] ✅ Property tests pass (1 test, 100+ cases)
- [ ] ✅ No clippy warnings
- [ ] ✅ Function complexity < 10
- [ ] ✅ Mutation score ≥ 90%
- [ ] ✅ Error messages are clear and actionable

## Dependencies

- `rash/src/linter.rs` - For Diagnostic type
- `rash/src/repl/help.rs` - For suggesting help topics

## Success Criteria

1. Parse errors show source context with caret ✅
2. Lint errors explain the problem clearly ✅
3. Command errors suggest similar commands ✅
4. All errors are actionable (suggest next step) ✅
5. Error formatting is fast (<10ms) ✅
6. Errors don't crash the REPL ✅

---

**Created**: 2024-10-31
**Author**: Claude (EXTREME TDD)
**Roadmap**: docs/REPL-DEBUGGER-ROADMAP.yaml
**Sprint**: REPL-015 (DevEx Improvements)
