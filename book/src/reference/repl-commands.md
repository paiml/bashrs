# REPL Commands Reference

Complete reference for all bashrs REPL commands (v6.19.0).

## What's New in v6.19.0

- 🚀 **Automatic Mode-Based Processing**: Commands are now automatically processed in `purify` and `lint` modes
- 🛠️ **Utility Commands**: `:history`, `:vars`, `:clear` for better session management
- ⚡ **50% Less Typing**: No more `:purify`/`:lint` prefixes when in those modes

## Command Overview

| Command | Category | Description |
|---------|----------|-------------|
| `help` | Core | Show help message with all commands |
| `quit` | Core | Exit the REPL |
| `exit` | Core | Exit the REPL (alias for `quit`) |
| `:mode` | Mode | Show current mode and available modes |
| `:mode <name>` | Mode | Switch to a different mode |
| `:parse <code>` | Analysis | Parse bash code and show AST |
| `:purify <code>` | Transform | Purify bash code (idempotent/deterministic) |
| `:lint <code>` | Analysis | Lint bash code and show diagnostics |
| `:history` | Utility | Show command history (NEW in v6.19.0) |
| `:vars` | Utility | Show session variables (NEW in v6.19.0) |
| `:clear` | Utility | Clear the screen (NEW in v6.19.0) |

## Core Commands

### `help`

Show comprehensive help message listing all available commands.

**Usage**:
```bash
bashrs [normal]> help
```

**Output**:
```
bashrs REPL Commands:
  help             - Show this help message
  quit             - Exit the REPL
  exit             - Exit the REPL
  :mode            - Show current mode and available modes
  :mode <name>     - Switch to a different mode
  :parse <code>    - Parse bash code and show AST
  :purify <code>   - Purify bash code (make idempotent/deterministic)
  :lint <code>     - Lint bash code and show diagnostics

Available modes:
  normal  - Execute bash commands directly
  purify  - Show purified version of bash commands
  lint    - Show linting results
  debug   - Step-by-step execution
  explain - Explain bash constructs
```

### `quit` / `exit`

Exit the REPL and save command history.

**Usage**:
```bash
bashrs [normal]> quit
Goodbye!
```

**Aliases**:
- `quit`
- `exit`

**Keyboard Shortcuts**:
- `Ctrl-D` (EOF) - Also exits the REPL

## Mode Commands

### `:mode` (no arguments)

Show current mode and list all available modes.

**Usage**:
```bash
bashrs [normal]> :mode
```

**Output**:
```
Current mode: normal - Execute bash commands directly

Available modes:
  normal  - Execute bash commands directly
  purify  - Show purified version of bash commands
  lint    - Show linting results for bash commands
  debug   - Debug bash commands with step-by-step execution
  explain - Explain bash constructs and syntax

Usage: :mode <mode_name>
```

### `:mode <name>`

Switch to a different analysis mode.

**Usage**:
```bash
bashrs [normal]> :mode lint
Switched to lint mode - Show linting results for bash commands
bashrs [lint]>
```

**Arguments**:
- `<name>` - Mode name (case-insensitive)

**Valid Modes**:
- `normal` - Execute bash commands directly
- `purify` - Show purified version
- `lint` - Show linting results
- `debug` - Step-by-step execution
- `explain` - Explain bash constructs

**Examples**:
```bash
# Switch to lint mode
bashrs [normal]> :mode lint
Switched to lint mode

# Case-insensitive
bashrs [lint]> :mode PURIFY
Switched to purify mode

# Switch back to normal
bashrs [purify]> :mode normal
Switched to normal mode
```

**Error Handling**:
```bash
bashrs [normal]> :mode invalid
Error: Unknown mode: valid modes are normal, purify, lint, debug, explain
```

## Analysis Commands

### `:parse <code>`

Parse bash code and display the Abstract Syntax Tree (AST).

**Usage**:
```bash
bashrs [normal]> :parse <bash_code>
```

**Arguments**:
- `<bash_code>` - Bash code to parse

**Examples**:

**Simple command**:
```bash
bashrs [normal]> :parse echo hello
✓ Parse successful!
Statements: 1
Parse time: 0ms

AST:
  [0] SimpleCommand {
    name: "echo",
    args: ["hello"]
  }
```

**Conditional statement**:
```bash
bashrs [normal]> :parse if [ -f file.txt ]; then cat file.txt; fi
✓ Parse successful!
Statements: 1
Parse time: 1ms

AST:
  [0] If {
    condition: Test { ... },
    then_body: [ SimpleCommand { name: "cat", ... } ],
    else_body: None
  }
```

**Pipeline**:
```bash
bashrs [normal]> :parse cat file.txt | grep pattern
✓ Parse successful!
Statements: 1
Parse time: 0ms

AST:
  [0] Pipeline {
    commands: [
      SimpleCommand { name: "cat", args: ["file.txt"] },
      SimpleCommand { name: "grep", args: ["pattern"] }
    ]
  }
```

**Error Handling**:
```bash
bashrs [normal]> :parse
Usage: :parse <bash_code>
Example: :parse echo hello

bashrs [normal]> :parse if then fi
✗ Parse error: Unexpected token 'then'
```

### `:lint <code>`

Lint bash code and display diagnostics with severity levels.

**Usage**:
```bash
bashrs [normal]> :lint <bash_code>
```

**Arguments**:
- `<bash_code>` - Bash code to lint

**Examples**:

**No issues**:
```bash
bashrs [normal]> :lint echo "hello"
✓ No issues found!
```

**With warnings**:
```bash
bashrs [normal]> :lint cat $FILE | grep pattern
Found 1 issue(s):
  ⚠ 1 warning(s)

[1] ⚠ SC2086 - Double quote to prevent globbing and word splitting
    Line 1
```

**Multiple issues**:
```bash
bashrs [normal]> :lint rm $DIR && echo $(cat $FILE)
Found 3 issue(s):
  ✗ 1 error(s)
  ⚠ 2 warning(s)

[1] ⚠ SC2086 - Double quote to prevent globbing and word splitting
    Line 1
    Variable: DIR

[2] ⚠ SC2046 - Quote this to prevent word splitting
    Line 1

[3] ⚠ SC2086 - Double quote to prevent globbing and word splitting
    Line 1
    Variable: FILE
```

**Severity Levels**:
- ✗ **Error**: Critical issues that will likely cause failures
- ⚠ **Warning**: Potential issues that should be fixed
- ℹ **Info**: Suggestions for improvement
- 📝 **Note**: Additional information
- ⚡ **Perf**: Performance optimization suggestions
- ⚠ **Risk**: Security or reliability risks

**Error Handling**:
```bash
bashrs [normal]> :lint
Usage: :lint <bash_code>
Example: :lint cat file.txt | grep pattern
```

## Transform Commands

### `:purify <code>`

Transform bash code to be idempotent and deterministic.

**Usage**:
```bash
bashrs [normal]> :purify <bash_code>
```

**Arguments**:
- `<bash_code>` - Bash code to purify

**Examples**:

**Non-idempotent mkdir**:
```bash
bashrs [normal]> :purify mkdir /tmp/app
✓ Purification successful!
Purified 1 statements

Transformations:
- mkdir → mkdir -p (idempotent)
- Added quotes for safety
```

**Non-deterministic $RANDOM**:
```bash
bashrs [normal]> :purify SESSION_ID=$RANDOM
✓ Purification successful!
Purified 1 statements

Transformations:
- $RANDOM → $(date +%s)-$$ (deterministic)
```

**Unsafe rm**:
```bash
bashrs [normal]> :purify rm /tmp/old
✓ Purification successful!
Purified 1 statements

Transformations:
- rm → rm -f (idempotent)
- Added quotes for safety
```

**Error Handling**:
```bash
bashrs [normal]> :purify
Usage: :purify <bash_code>
Example: :purify mkdir /tmp/test

bashrs [normal]> :purify if then fi
✗ Purification error: Parse error: Unexpected token 'then'
```

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl-C` | Cancel current line |
| `Ctrl-D` | Exit REPL (EOF) |
| `Ctrl-L` | Clear screen |
| `Ctrl-U` | Delete line before cursor |
| `Ctrl-K` | Delete line after cursor |
| `Ctrl-W` | Delete word before cursor |
| `↑` | Previous command (history) |
| `↓` | Next command (history) |
| `Ctrl-R` | Reverse search history |
| `Tab` | Auto-completion (future) |

## Exit Codes

The REPL always exits with code `0` on normal termination (quit/exit).

**Exceptions**:
- Ctrl-C during REPL initialization: Exit code `130`
- Fatal error during startup: Exit code `1`

## Configuration

REPL behavior can be configured via command-line options:

```bash
$ bashrs repl [OPTIONS]
```

### Available Options

| Option | Default | Description |
|--------|---------|-------------|
| `--debug` | false | Enable debug mode for verbose output |
| `--max-memory <MB>` | 500 | Maximum memory usage in MB |
| `--timeout <SECONDS>` | 120 | Command timeout in seconds |
| `--max-depth <N>` | 1000 | Maximum recursion depth |
| `--sandboxed` | false | Run in sandboxed mode (restricted operations) |

### Examples

**Enable debug mode**:
```bash
$ bashrs repl --debug
bashrs REPL v6.7.0 (debug mode enabled)
...
```

**Set resource limits**:
```bash
$ bashrs repl --max-memory 200 --timeout 60 --max-depth 500
```

**Sandboxed mode**:
```bash
$ bashrs repl --sandboxed
bashrs REPL v6.7.0 (sandboxed mode)
Note: Some operations are restricted in sandboxed mode
...
```

## Utility Commands (NEW in v6.19.0)

### `:history`

Show the command history for the current REPL session.

**Usage**:
```bash
bashrs [normal]> :history
```

**Output**:
```
Command History (5 commands):
  1 echo hello
  2 mkdir /tmp/test
  3 :parse if [ -f test ]; then echo found; fi
  4 :lint cat file | grep pattern
  5 :history
```

**Features**:
- Shows all commands executed in the current session
- Commands are numbered for easy reference
- Includes both regular bash commands and REPL commands
- History is automatically saved to `~/.bashrs_history`

**Examples**:
```bash
# View history after running several commands
bashrs [normal]> echo test
bashrs [normal]> mkdir /tmp/app
bashrs [normal]> :history
Command History (3 commands):
  1 echo test
  2 mkdir /tmp/app
  3 :history

# Empty history
bashrs [normal]> :history
No commands in history
```

### `:vars`

Display all session variables (for future variable assignment support).

**Usage**:
```bash
bashrs [normal]> :vars
```

**Output**:
```
No session variables set
```

**Future Support**:
```bash
# When variable assignment is implemented
bashrs [normal]> x=5
bashrs [normal]> name="test"
bashrs [normal]> :vars
Session Variables (2 variables):
  name = test
  x = 5
```

**Features**:
- Shows all variables set in the current session
- Variables are sorted alphabetically
- Displays variable names and values
- Ready for future variable assignment feature

### `:clear`

Clear the terminal screen using ANSI escape codes.

**Usage**:
```bash
bashrs [normal]> :clear
# Screen is cleared, fresh prompt appears
```

**Technical Details**:
- Uses ANSI escape sequences: `\x1B[2J\x1B[H`
- `\x1B[2J` - Clears the entire screen
- `\x1B[H` - Moves cursor to home position (0,0)
- Works on all ANSI-compatible terminals

**Examples**:
```bash
# After lots of output
bashrs [normal]> :parse long command...
... lots of AST output ...
bashrs [normal]> :lint another command...
... more output ...
bashrs [normal]> :clear
# Screen cleared, clean slate
bashrs [normal]>
```

**Keyboard Shortcut Alternative**:
- `Ctrl-L` also clears the screen (standard terminal shortcut)

## Automatic Mode-Based Processing (NEW in v6.19.0)

When you switch to `purify` or `lint` mode, commands are **automatically processed** in that mode without needing explicit `:purify` or `:lint` prefixes.

### Before v6.19.0 (Repetitive)
```bash
bashrs [purify]> :purify mkdir /tmp/test
bashrs [purify]> :purify rm /tmp/old
bashrs [purify]> :purify ln -s /tmp/new /tmp/link
```

### After v6.19.0 (Automatic)
```bash
bashrs [normal]> :mode purify
Switched to purify mode

bashrs [purify]> mkdir /tmp/test
✓ Purified: Purified 1 statement(s)

bashrs [purify]> rm /tmp/old
✓ Purified: Purified 1 statement(s)

bashrs [purify]> ln -s /tmp/new /tmp/link
✓ Purified: Purified 1 statement(s)
```

### Explicit Commands Still Work

Explicit `:parse`, `:purify`, and `:lint` commands work in **any mode**:

```bash
# In purify mode, but want to parse
bashrs [purify]> :parse echo hello
✓ Parse successful!
Statements: 1

# In lint mode, but want to purify
bashrs [lint]> :purify mkdir test
✓ Purification successful!
Purified 1 statement(s)
```

### Benefits

- **50% less typing** - No more repetitive `:purify`/`:lint` prefixes
- **Faster workflow** - Switch mode once, process many commands
- **More intuitive** - Mode-based processing matches user mental model
- **Explicit commands** - Still available when you need them

## See Also

- [Interactive REPL Guide](../getting-started/repl.md) - Getting started tutorial
- [Purifier Integration](../concepts/purification.md) - Transformation rules
- [Linter Rules Reference](./rules.md) - Complete linting rules
