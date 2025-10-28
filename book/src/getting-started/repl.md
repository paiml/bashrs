# Interactive REPL

The bashrs REPL (Read-Eval-Print Loop) provides an interactive environment for bash script analysis, transformation, and learning.

## Starting the REPL

```bash
$ bashrs repl

bashrs REPL v6.7.0
Type 'quit' or 'exit' to exit, 'help' for commands
Current mode: normal - Execute bash commands directly
bashrs [normal]>
```

## Features

- 🎯 **5 Interactive Modes**: Switch between different analysis modes
- 🔍 **Parser Integration**: Parse bash code and inspect AST
- 🧹 **Purifier Integration**: Transform bash to idempotent/deterministic code
- 🔎 **Linter Integration**: Real-time diagnostics with severity levels
- 📚 **Command History**: Persistent history in `~/.bashrs_history`

## Available Commands

### Core Commands

| Command | Description |
|---------|-------------|
| `help` | Show all available commands |
| `quit` or `exit` | Exit the REPL |
| `:mode` | Show current mode and available modes |
| `:mode <name>` | Switch to a different mode |
| `:parse <code>` | Parse bash code and show AST |
| `:purify <code>` | Purify bash code (idempotent/deterministic) |
| `:lint <code>` | Lint bash code and show diagnostics |

### Mode Switching

The REPL supports 5 interactive modes:

```bash
bashrs [normal]> :mode
Current mode: normal - Execute bash commands directly

Available modes:
  normal  - Execute bash commands directly
  purify  - Show purified version of bash commands
  lint    - Show linting results for bash commands
  debug   - Debug bash commands with step-by-step execution
  explain - Explain bash constructs and syntax

Usage: :mode <mode_name>
```

Switch modes with `:mode <name>`:

```bash
bashrs [normal]> :mode lint
Switched to lint mode - Show linting results for bash commands
bashrs [lint]>
```

## Examples

### Example 1: Parsing Bash Code

```bash
bashrs [normal]> :parse echo hello world
✓ Parse successful!
Statements: 1
Parse time: 0ms

AST:
  [0] SimpleCommand {
    name: "echo",
    args: ["hello", "world"]
  }
```

### Example 2: Purifying Non-Idempotent Code

```bash
bashrs [normal]> :purify mkdir /tmp/myapp
✓ Purification successful!
Purified 1 statements

Original:
  mkdir /tmp/myapp

Purified:
  mkdir -p "/tmp/myapp"  # Idempotent + quoted
```

### Example 3: Linting for Safety Issues

```bash
bashrs [normal]> :lint cat file.txt | grep $PATTERN
Found 1 issue(s):
  ⚠ 1 warning(s)

[1] ⚠ SC2086 - Double quote to prevent globbing and word splitting
    Line 1
    Variable: PATTERN

Fix: cat file.txt | grep "$PATTERN"
```

### Example 4: Mode-Based Workflow

```bash
# Start in normal mode
bashrs [normal]> :parse if [ -f config.txt ]; then cat config.txt; fi
✓ Parse successful!

# Switch to lint mode
bashrs [normal]> :mode lint
Switched to lint mode

# Lint the code
bashrs [lint]> :lint if [ -f config.txt ]; then cat config.txt; fi
✓ No issues found!

# Switch to purify mode
bashrs [lint]> :mode purify
Switched to purify mode

# See the purified version
bashrs [purify]> :purify mkdir /var/log/app
✓ Purification successful!
Purified: mkdir -p "/var/log/app"
```

## Command History

The REPL automatically saves command history to `~/.bashrs_history`:

```bash
# Commands are saved across sessions
$ bashrs repl
bashrs [normal]> :parse echo test
...
bashrs [normal]> quit
Goodbye!

# Later...
$ bashrs repl
bashrs [normal]> # Press ↑ to recall previous commands
```

### History Navigation

- **↑ (Up Arrow)**: Previous command
- **↓ (Down Arrow)**: Next command
- **Ctrl-R**: Reverse search through history
- **Ctrl-C**: Cancel current line
- **Ctrl-D**: Exit REPL (EOF)

## REPL Configuration

Configure REPL behavior with command-line flags:

```bash
# Enable debug mode
$ bashrs repl --debug

# Set resource limits
$ bashrs repl --max-memory 200 --timeout 60

# Sandboxed mode (restricted operations)
$ bashrs repl --sandboxed
```

### Available Options

| Option | Default | Description |
|--------|---------|-------------|
| `--debug` | false | Enable debug mode |
| `--max-memory` | 500MB | Maximum memory usage |
| `--timeout` | 120s | Command timeout |
| `--max-depth` | 1000 | Maximum recursion depth |
| `--sandboxed` | false | Run in sandboxed mode |

## Use Cases

### Learning Bash

Use the REPL to interactively learn bash transformations:

```bash
bashrs [normal]> :mode explain
bashrs [explain]> # Future: Get explanations of bash constructs

bashrs [explain]> :mode purify
bashrs [purify]> :purify rm -rf /tmp/data
✓ Purified: rm -rf "/tmp/data"  # Safe (quoted path)
```

### Quick Validation

Validate bash snippets before committing:

```bash
$ bashrs repl
bashrs [normal]> :lint $(cat deploy.sh)
Found 3 issue(s):
  ⚠ 2 warning(s)
  ℹ 1 info

# Fix issues...

bashrs [normal]> :lint $(cat deploy.sh)
✓ No issues found!
```

### Experimenting with Transformations

Test purification transformations interactively:

```bash
bashrs [normal]> :purify SESSION_ID=$RANDOM
✓ Purified: SESSION_ID="$(date +%s)-$$"  # Deterministic

bashrs [normal]> :purify RELEASE="release-$(date +%s)"
✓ Purified: RELEASE="release-${VERSION}"  # Version-based
```

## Troubleshooting

### REPL Won't Start

**Error**: `Failed to initialize REPL`

**Solution**: Check that your terminal supports ANSI escape codes:

```bash
# Test terminal support
$ echo -e "\e[1mBold\e[0m"

# If that doesn't work, use a different terminal
```

### History Not Persisting

**Error**: Commands not saved across sessions

**Solution**: Check history file permissions:

```bash
$ ls -la ~/.bashrs_history
# Should be readable/writable by your user

# Fix permissions if needed
$ chmod 600 ~/.bashrs_history
```

### Out of Memory

**Error**: `REPL out of memory`

**Solution**: Increase memory limit:

```bash
$ bashrs repl --max-memory 1000  # 1GB
```

## Next Steps

- [REPL Commands Reference](../reference/repl-commands.md) - Complete command reference
- [Parser Integration](../concepts/parser.md) - Understanding bash AST
- [Purifier Integration](../concepts/purification.md) - Transformation rules
- [Linter Integration](../linting/security.md) - Linting rules reference
