# TICKET-REPL-015-004: Help System (:help)

**Sprint**: REPL-015 (DevEx Improvements)
**Status**: IN PROGRESS
**Priority**: HIGH
**Methodology**: EXTREME TDD

## Problem Statement

The bashrs REPL currently has no built-in help system. Users must guess commands or read external documentation to discover REPL features. This creates a poor developer experience.

**Current Behavior**:
```
bashrs> :help
Error: Unknown command: :help

bashrs> help
Error: Parse error: unexpected token 'help'
```

**Desired Behavior**:
```
bashrs> :help
bashrs REPL - Interactive Shell Debugger and Purifier

COMMANDS:
  :quit, :q          Exit the REPL
  :purify            Show purified version of last input
  :lint              Show lint violations with context
  :ast               Show AST of last input
  :help [command]    Show help (this message)
  :modes             List available REPL modes

EXAMPLES:
  echo $RANDOM       # Try typing this, then :purify to see safe version
  mkdir /app         # Type this, then :lint to see violations

For command-specific help, type :help <command>
For more info, see: https://github.com/paiml/bashrs

bashrs> :help purify
:purify - Show purified version of last input

USAGE:
  :purify

DESCRIPTION:
  Displays a deterministic, idempotent version of your last bash input.
  Transformations applied:
  - $RANDOM → seeded RNG
  - Timestamps → fixed values
  - mkdir → mkdir -p (idempotent)
  - rm → rm -f (idempotent)

EXAMPLE:
  bashrs> echo $RANDOM
  bashrs> :purify

  Purified:
  echo "${SEED_BASED_VALUE}"
```

## Requirements

### Functional Requirements

1. **General Help** (`:help` with no arguments)
   - Show list of all available commands
   - Show brief description of each command
   - Show 2-3 examples for common workflows
   - Show link to full documentation

2. **Command-Specific Help** (`:help <command>`)
   - Show detailed usage for specific command
   - Show command syntax
   - Show description of what command does
   - Show 1-2 examples
   - Handle invalid commands gracefully

3. **Help Command Variants**
   - Support `:help` (colon prefix)
   - Support `help` (without colon)
   - Support `:h` (short form)
   - Support `--help` flag on REPL startup

4. **Content Quality**
   - Clear, concise descriptions
   - Practical examples
   - Beginner-friendly language
   - Links to further resources

### Non-Functional Requirements

1. **Performance**: Help display < 10ms
2. **Maintainability**: Help text easily updatable
3. **Consistency**: Same style as other REPL output
4. **Completeness**: Cover ALL REPL commands

## Data Structures

### HelpEntry

```rust
/// Help entry for a single command
#[derive(Debug, Clone, PartialEq)]
pub struct HelpEntry {
    /// Command name (e.g., "purify", "lint", "ast")
    pub command: String,

    /// Brief one-line description
    pub brief: String,

    /// Detailed description (multi-line)
    pub description: String,

    /// Usage syntax (e.g., ":purify")
    pub usage: String,

    /// Example usage (with expected output)
    pub examples: Vec<HelpExample>,

    /// Aliases (e.g., ["q", "quit"] for :quit)
    pub aliases: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HelpExample {
    /// Example input
    pub input: String,

    /// Expected output or behavior
    pub output: String,
}
```

### HelpRegistry

```rust
/// Registry of all help entries
pub struct HelpRegistry {
    entries: HashMap<String, HelpEntry>,
}

impl HelpRegistry {
    /// Create new help registry with all commands
    pub fn new() -> Self;

    /// Get help entry for a command
    pub fn get(&self, command: &str) -> Option<&HelpEntry>;

    /// Get all help entries (for general help)
    pub fn all(&self) -> Vec<&HelpEntry>;

    /// Check if command exists
    pub fn has_command(&self, command: &str) -> bool;
}
```

## Function Specifications

### 1. `show_general_help() -> String`

**Purpose**: Show general help with all commands

**Logic**:
1. Get all help entries from registry
2. Format as table with command, brief description
3. Add examples section
4. Add footer with link to docs

**Returns**: Formatted help string

**Example**:
```rust
let help = show_general_help();
assert!(help.contains("bashrs REPL"));
assert!(help.contains(":purify"));
assert!(help.contains(":lint"));
assert!(help.contains("EXAMPLES:"));
```

### 2. `show_command_help(command: &str) -> Result<String, String>`

**Purpose**: Show detailed help for specific command

**Logic**:
1. Look up command in help registry
2. If not found, return error with suggestion
3. Format detailed help with usage, description, examples
4. Include aliases if any

**Returns**:
- `Ok(help_text)` if command exists
- `Err(error_message)` if command not found

**Example**:
```rust
let help = show_command_help("purify").unwrap();
assert!(help.contains("USAGE:"));
assert!(help.contains("DESCRIPTION:"));
assert!(help.contains("EXAMPLE:"));

let err = show_command_help("invalid").unwrap_err();
assert!(err.contains("Unknown command"));
assert!(err.contains("Did you mean"));
```

### 3. `format_help_entry(entry: &HelpEntry) -> String`

**Purpose**: Format a single help entry for display

**Logic**:
1. Format command name and brief description
2. Format usage syntax
3. Format detailed description
4. Format examples with input/output
5. Format aliases if any

**Returns**: Formatted help text

### 4. `suggest_similar_command(input: &str, registry: &HelpRegistry) -> Option<String>`

**Purpose**: Suggest similar command when user types invalid command

**Logic**:
1. Calculate edit distance to all commands
2. If distance < 3, suggest closest match
3. Return None if no close matches

**Returns**: Suggested command name or None

**Example**:
```rust
let registry = HelpRegistry::new();
assert_eq!(suggest_similar_command("purfy", &registry), Some("purify".to_string()));
assert_eq!(suggest_similar_command("lnt", &registry), Some("lint".to_string()));
assert_eq!(suggest_similar_command("xyz", &registry), None);
```

## Test Specifications

### Unit Tests

#### Test: REPL-015-004-001 - Show general help
```rust
#[test]
fn test_REPL_015_004_001_show_general_help() {
    let help = show_general_help();

    // Should have header
    assert!(help.contains("bashrs REPL"));

    // Should list all commands
    assert!(help.contains(":quit"));
    assert!(help.contains(":purify"));
    assert!(help.contains(":lint"));
    assert!(help.contains(":ast"));
    assert!(help.contains(":help"));
    assert!(help.contains(":modes"));

    // Should have examples section
    assert!(help.contains("EXAMPLES:"));

    // Should have link to docs
    assert!(help.contains("github.com/paiml/bashrs"));
}
```

#### Test: REPL-015-004-002 - Show command-specific help
```rust
#[test]
fn test_REPL_015_004_002_show_command_help() {
    // Test valid command
    let help = show_command_help("purify").unwrap();
    assert!(help.contains(":purify"));
    assert!(help.contains("USAGE:"));
    assert!(help.contains("DESCRIPTION:"));
    assert!(help.contains("EXAMPLE:"));

    // Test another command
    let help = show_command_help("lint").unwrap();
    assert!(help.contains(":lint"));
    assert!(help.contains("violations"));
}
```

#### Test: REPL-015-004-003 - Invalid command with suggestion
```rust
#[test]
fn test_REPL_015_004_003_invalid_command_suggestion() {
    let err = show_command_help("purfy").unwrap_err();
    assert!(err.contains("Unknown command: purfy"));
    assert!(err.contains("Did you mean: purify"));
}
```

#### Test: REPL-015-004-004 - Help entry formatting
```rust
#[test]
fn test_REPL_015_004_004_format_help_entry() {
    let entry = HelpEntry {
        command: "purify".to_string(),
        brief: "Show purified version".to_string(),
        description: "Displays deterministic, idempotent bash".to_string(),
        usage: ":purify".to_string(),
        examples: vec![
            HelpExample {
                input: "echo $RANDOM".to_string(),
                output: "Purified: echo \"${SEED}\"".to_string(),
            }
        ],
        aliases: vec!["p".to_string()],
    };

    let formatted = format_help_entry(&entry);
    assert!(formatted.contains(":purify"));
    assert!(formatted.contains("USAGE:"));
    assert!(formatted.contains("echo $RANDOM"));
    assert!(formatted.contains("Aliases: p"));
}
```

#### Test: REPL-015-004-005 - Help registry operations
```rust
#[test]
fn test_REPL_015_004_005_help_registry() {
    let registry = HelpRegistry::new();

    // Should have all commands
    assert!(registry.has_command("purify"));
    assert!(registry.has_command("lint"));
    assert!(registry.has_command("ast"));
    assert!(registry.has_command("quit"));

    // Should get entries
    let entry = registry.get("purify").unwrap();
    assert_eq!(entry.command, "purify");

    // Should handle aliases
    assert!(registry.has_command("q")); // alias for quit

    // Should return all entries
    let all = registry.all();
    assert!(all.len() >= 5); // At least 5 commands
}
```

#### Test: REPL-015-004-006 - Command suggestion with edit distance
```rust
#[test]
fn test_REPL_015_004_006_suggest_similar() {
    let registry = HelpRegistry::new();

    // Close typos should suggest
    assert_eq!(suggest_similar_command("purfy", &registry), Some("purify".to_string()));
    assert_eq!(suggest_similar_command("lnt", &registry), Some("lint".to_string()));
    assert_eq!(suggest_similar_command("qit", &registry), Some("quit".to_string()));

    // Far typos should not suggest
    assert_eq!(suggest_similar_command("xyz123", &registry), None);
    assert_eq!(suggest_similar_command("foobar", &registry), None);
}
```

### Integration Tests

#### Test: REPL-015-004-INT-001 - Help command in REPL
```rust
#[test]
fn test_REPL_015_004_INT_001_help_in_repl() {
    // Test :help command
    let mut repl = ReplSession::new();

    let output = repl.execute(":help").unwrap();
    assert!(output.contains("bashrs REPL"));
    assert!(output.contains(":purify"));

    // Test :help <command>
    let output = repl.execute(":help purify").unwrap();
    assert!(output.contains("USAGE:"));
    assert!(output.contains("deterministic"));

    // Test help (without colon)
    let output = repl.execute("help").unwrap();
    assert!(output.contains("bashrs REPL"));

    // Test :h (short form)
    let output = repl.execute(":h").unwrap();
    assert!(output.contains("bashrs REPL"));
}
```

### Property Tests

#### Property: Help text never panics
```rust
proptest! {
    #[test]
    fn prop_help_never_panics(command in ".*{0,100}") {
        // Should never panic, even with invalid input
        let _ = show_command_help(&command);
    }
}
```

#### Property: All commands have help entries
```rust
#[test]
fn prop_all_commands_have_help() {
    let registry = HelpRegistry::new();
    let commands = vec!["purify", "lint", "ast", "quit", "help", "modes"];

    for cmd in commands {
        assert!(registry.has_command(cmd), "Missing help for: {}", cmd);
        let entry = registry.get(cmd).unwrap();
        assert!(!entry.brief.is_empty(), "Empty brief for: {}", cmd);
        assert!(!entry.description.is_empty(), "Empty description for: {}", cmd);
    }
}
```

## Help Content

### Command Entries

#### :quit / :q
- **Brief**: Exit the REPL
- **Usage**: `:quit` or `:q`
- **Description**: Exits the bashrs REPL session
- **Aliases**: `q`

#### :purify
- **Brief**: Show purified version of last input
- **Usage**: `:purify`
- **Description**: Displays a deterministic, idempotent version of your last bash input. Transformations: $RANDOM→seeded, timestamps→fixed, mkdir→mkdir -p
- **Example**: Input `echo $RANDOM`, then `:purify`

#### :lint
- **Brief**: Show lint violations with context
- **Usage**: `:lint`
- **Description**: Runs bashrs linter (355+ rules) on your last input and displays violations with source context
- **Example**: Input `mkdir /app`, then `:lint` to see IDEM001

#### :ast
- **Brief**: Show AST of last input
- **Usage**: `:ast`
- **Description**: Displays the Abstract Syntax Tree of your last bash input
- **Example**: Input `echo hello`, then `:ast`

#### :help
- **Brief**: Show help (this message)
- **Usage**: `:help [command]`
- **Description**: Show general help or command-specific help
- **Aliases**: `h`, `help`
- **Example**: `:help`, `:help purify`

#### :modes
- **Brief**: List available REPL modes
- **Usage**: `:modes`
- **Description**: Shows all available REPL modes (bash, sh, posix, makefile)

## EXTREME TDD Phases

### RED Phase ✅ (Write Failing Tests)
1. Create test file: `rash/src/repl/help.rs`
2. Write 6 unit tests (all should fail/panic):
   - `test_REPL_015_004_001_show_general_help`
   - `test_REPL_015_004_002_show_command_help`
   - `test_REPL_015_004_003_invalid_command_suggestion`
   - `test_REPL_015_004_004_format_help_entry`
   - `test_REPL_015_004_005_help_registry`
   - `test_REPL_015_004_006_suggest_similar`
3. Write 1 integration test:
   - `test_REPL_015_004_INT_001_help_in_repl`
4. Write 2 property tests:
   - `prop_help_never_panics`
   - `prop_all_commands_have_help`
5. Run tests: `cargo test test_REPL_015_004` (should FAIL ❌)

### GREEN Phase 🟢 (Make Tests Pass)
1. Implement `HelpEntry` and `HelpExample` structs
2. Implement `HelpRegistry::new()` with all command entries
3. Implement `show_general_help()`
4. Implement `show_command_help()`
5. Implement `format_help_entry()`
6. Implement `suggest_similar_command()` with edit distance
7. Integrate help commands into REPL command handler
8. Run tests: `cargo test test_REPL_015_004` (should PASS ✅)

### REFACTOR Phase 🔄 (Clean Up)
1. Extract help content to constants/config
2. Run `cargo clippy --lib` (no warnings)
3. Check function complexity < 10
4. Add rustdoc comments
5. Run full test suite: `cargo test --lib` (all pass)

### PROPERTY Phase 🎲 (Generative Testing)
1. Run property tests with 100+ cases
2. Add fuzzing for help command parsing
3. Verify no panics on any input

### MUTATION Phase 🧬 (Mutation Testing)
1. Run `cargo mutants --file rash/src/repl/help.rs`
2. Target: ≥90% mutation kill rate
3. Add tests for surviving mutants

### COMMIT Phase 📝 (Git Commit)
1. Update `docs/REPL-DEBUGGER-ROADMAP.yaml` (mark REPL-015-004 as completed)
2. Create commit:
   ```
   feat: REPL-015-004 - Help system (:help)

   Implemented comprehensive help system for bashrs REPL.

   Features:
   - General help with all commands
   - Command-specific help
   - Did-you-mean suggestions
   - Examples and usage

   Quality: 9 tests passing, ≥90% mutation score

   🤖 Generated with [Claude Code](https://claude.com/claude-code)
   Co-Authored-By: Claude <noreply@anthropic.com>
   ```

## Quality Gates

- [ ] ✅ All unit tests pass (6 tests)
- [ ] ✅ Integration test passes (1 test)
- [ ] ✅ Property tests pass (2 tests, 100+ cases each)
- [ ] ✅ No clippy warnings
- [ ] ✅ Function complexity < 10
- [ ] ✅ Mutation score ≥ 90%
- [ ] ✅ All commands have help entries
- [ ] ✅ Help text is clear and beginner-friendly

## Dependencies

None (self-contained feature)

## Risks

1. **Help content becomes stale** - Mitigation: Property test ensures all commands have help
2. **Edit distance suggestions incorrect** - Mitigation: Extensive unit tests
3. **Help text too verbose** - Mitigation: Keep brief, link to full docs

## Success Criteria

1. User types `:help` and sees list of all commands ✅
2. User types `:help purify` and sees detailed help ✅
3. User types `:help purfy` (typo) and gets suggestion ✅
4. All REPL commands have help entries ✅
5. Help system is fast (<10ms) ✅
6. Help text is clear and actionable ✅

---

**Created**: 2024-10-31
**Author**: Claude (EXTREME TDD)
**Roadmap**: docs/REPL-DEBUGGER-ROADMAP.yaml
**Sprint**: REPL-015 (DevEx Improvements)
