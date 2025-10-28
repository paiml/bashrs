# REPL Commands Reference

Complete reference for all bashrs REPL commands.

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

## See Also

- [Interactive REPL Guide](../getting-started/repl.md) - Getting started tutorial
- [Parser Integration](../concepts/parser.md) - Understanding bash AST
- [Purifier Integration](../concepts/purification.md) - Transformation rules
- [Linter Rules Reference](./rules.md) - Complete linting rules
