# Interactive REPL

The bashrs REPL (Read-Eval-Print Loop) provides an interactive environment for bash script analysis, transformation, and learning.

## Starting the REPL

```bash
$ bashrs repl

bashrs REPL v6.32.1
Type 'quit' or 'exit' to exit, 'help' for commands
Current mode: normal - Execute bash commands directly
bashrs [normal]>
```

## Features

- 🎯 **5 Interactive Modes**: Switch between different analysis modes
- ⌨️ **Tab Completion**: Auto-complete commands, modes, file paths, and bash constructs
- 📝 **Multi-line Input**: Natural support for loops, functions, and conditionals
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

### Utility Commands

| Command | Description |
|---------|-------------|
| `:history` | Show command history for this session |
| `:vars` | Show session variables |
| `:clear` | Clear the screen |

### Script Loading Commands

**NEW in v6.20.0**: Load bash scripts from files, extract functions, and manage your interactive development workflow.

| Command | Description |
|---------|-------------|
| `:load <file>` | Load a bash script and extract functions |
| `:source <file>` | Source a bash script (load and add to session) |
| `:functions` | List all loaded functions |
| `:reload` | Reload the last loaded script |

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

### Automatic Mode-Based Processing

**NEW in v6.19.0**: When you switch to `purify` or `lint` mode, commands are automatically processed in that mode without needing explicit `:purify` or `:lint` prefixes.

#### Purify Mode
```bash
bashrs [normal]> :mode purify
Switched to purify mode

# Commands are automatically purified
bashrs [purify]> mkdir /tmp/test
✓ Purified:
Purified 1 statement(s)
(Full bash output coming soon)

bashrs [purify]> rm /old/file
✓ Purified:
Purified 1 statement(s)
```

#### Lint Mode
```bash
bashrs [normal]> :mode lint
Switched to lint mode

# Commands are automatically linted
bashrs [lint]> cat file.txt | grep pattern
Found 1 issue(s):
  ⚠ 1 warning(s)

[1] ⚠ SC2086 - Useless cat
```

#### Explicit Commands Still Work
Explicit commands (`:parse`, `:purify`, `:lint`) work in **any mode**:

```bash
bashrs [purify]> :parse echo hello
✓ Parse successful!
Statements: 1

bashrs [lint]> :purify mkdir test
✓ Purification successful!
Purified 1 statement(s)
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

### Example 5: Using Utility Commands

**NEW in v6.19.0**: The REPL now includes utility commands for managing your session.

#### View Command History
```bash
bashrs [normal]> echo hello
Would execute: echo hello

bashrs [normal]> mkdir test
Would execute: mkdir test

bashrs [normal]> :history
Command History (3 commands):
  1 echo hello
  2 mkdir test
  3 :history
```

#### View Session Variables
```bash
bashrs [normal]> :vars
No session variables set

# After assigning variables
bashrs [normal]> x=5
✓ Variable set: x = 5

bashrs [normal]> name="Alice"
✓ Variable set: name = Alice

bashrs [normal]> :vars
Session Variables (2 variables):
  name = Alice
  x = 5
```

#### Clear Screen
```bash
bashrs [normal]> echo "lots of output..."
bashrs [normal]> echo "more output..."
bashrs [normal]> :clear
# Screen cleared, fresh prompt
bashrs [normal]>
```

### Example 6: Automatic Mode Processing Workflow

**NEW in v6.19.0**: The killer feature - automatic command processing in purify/lint modes.

```bash
# Switch to purify mode
bashrs [normal]> :mode purify
Switched to purify mode

# Commands are AUTOMATICALLY purified
bashrs [purify]> mkdir /tmp/test
✓ Purified:
Purified 1 statement(s)

bashrs [purify]> rm /tmp/old
✓ Purified:
Purified 1 statement(s)

# Switch to lint mode
bashrs [purify]> :mode lint
Switched to lint mode

# Commands are AUTOMATICALLY linted
bashrs [lint]> cat file | grep pattern
Found 1 issue(s):
  ⚠ 1 warning(s)

# View what you've done
bashrs [lint]> :history
Command History (6 commands):
  1 :mode purify
  2 mkdir /tmp/test
  3 rm /tmp/old
  4 :mode lint
  5 cat file | grep pattern
  6 :history
```

### Example 7: Using Explain Mode for Learning Bash

**NEW in v6.19.0**: The Explain Mode provides interactive explanations of bash constructs to help you learn shell scripting.

#### Switch to Explain Mode
```bash
bashrs [normal]> :mode explain
Switched to explain mode - Explain bash constructs and syntax
bashrs [explain]>
```

#### Explain Parameter Expansions
```bash
bashrs [explain]> ${var:-default}
📖 Parameter Expansion: ${parameter:-word}
   Use Default Value

If parameter is unset or null, expand to 'word'.
The original parameter remains unchanged.

Example:
  $ var=""
  $ echo "${var:-fallback}"  # Outputs: fallback
  $ echo "$var"               # Still empty

bashrs [explain]> ${var:=default}
📖 Parameter Expansion: ${parameter:=word}
   Assign Default Value

If parameter is unset or null, assign 'word' to it.
Then expand to the new value.

Example:
  $ unset var
  $ echo "${var:=fallback}"  # Outputs: fallback
  $ echo "$var"               # Now set to fallback

bashrs [explain]> ${#var}
📖 Parameter Expansion: ${#parameter}
   String Length

Expands to the length of the parameter's value in characters.

Example:
  $ var="hello"
  $ echo "${#var}"  # Outputs: 5
```

#### Explain Control Flow Constructs
```bash
bashrs [explain]> for i in *.txt
📖 For Loop: for name in words
   Iterate Over List

Loop variable 'name' takes each value from the word list.
Executes commands for each iteration.

Example:
  for file in *.txt; do
    echo "Processing: $file"
  done

bashrs [explain]> if [ -f file ]
📖 If Statement: if condition; then commands; fi
   Conditional Execution

Execute commands only if condition succeeds (exit status 0).
Optional elif and else clauses for alternatives.

Example:
  if [ -f file.txt ]; then
    echo "File exists"
  fi

bashrs [explain]> while true
📖 While Loop: while condition; do commands; done
   Conditional Loop

Execute commands repeatedly while condition succeeds.
Checks condition before each iteration.

Example:
  counter=0
  while [ $counter -lt 5 ]; do
    echo $counter
    counter=$((counter + 1))
  done
```

#### Explain I/O Redirection
```bash
bashrs [explain]> echo test > file
📖 Output Redirection: command > file
   Redirect Standard Output

Redirects stdout to a file, overwriting existing content.
Use >> to append instead.

Example:
  echo "text" > file.txt   # Overwrite
  echo "more" >> file.txt  # Append

bashrs [explain]> cat < file
📖 Input Redirection: command < file
   Redirect Standard Input

Redirects stdin to read from a file instead of keyboard.

Example:
  while read line; do
    echo "Line: $line"
  done < file.txt

bashrs [explain]> cat file | grep pattern
📖 Pipe: command1 | command2
   Connect Commands

Redirects stdout of command1 to stdin of command2.
Enables chaining multiple commands together.

Example:
  cat file.txt | grep pattern | wc -l
```

#### Unknown Constructs
```bash
bashrs [explain]> unknown_command_xyz
No explanation available for: unknown_command_xyz
Try parameter expansions (${var:-default}), control flow (for, if, while), or redirections (>, <, |)
```

#### Combining Explain Mode with Other Commands
```bash
# Explain a construct, then lint it
bashrs [explain]> ${var:-default}
📖 Parameter Expansion: ${parameter:-word}
   Use Default Value
...

bashrs [explain]> :lint echo ${var:-default}
✓ No issues found!

# Switch to purify mode to see transformations
bashrs [explain]> :mode purify
Switched to purify mode

bashrs [purify]> mkdir /tmp/test
✓ Purified:
Purified 1 statement(s)
```

### Example 8: Variable Assignment and Expansion

**NEW in v6.20.0**: The REPL now supports bash-style variable assignment and automatic expansion in commands.

**NEW in v6.21.0**: Normal mode now executes commands directly with full bash compatibility.

#### Simple Variable Assignment
```bash
bashrs [normal]> x=5
✓ Variable set: x = 5

bashrs [normal]> echo $x
5
```

#### Variable Assignment with Quotes
```bash
# Double quotes
bashrs [normal]> name="Alice Johnson"
✓ Variable set: name = Alice Johnson

# Single quotes
bashrs [normal]> path='/usr/local/bin'
✓ Variable set: path = /usr/local/bin
```

#### Variable Expansion Syntax
```bash
# Simple expansion
bashrs [normal]> version=1.0.0
✓ Variable set: version = 1.0.0

bashrs [normal]> echo $version
Would execute: echo 1.0.0

# Braced expansion
bashrs [normal]> echo Release: ${version}
Would execute: echo Release: 1.0.0
```

#### Multiple Variables
```bash
bashrs [normal]> x=10
✓ Variable set: x = 10

bashrs [normal]> y=20
✓ Variable set: y = 20

bashrs [normal]> echo $x + $y = 30
Would execute: echo 10 + 20 = 30

bashrs [normal]> :vars
Session Variables (2 variables):
  x = 10
  y = 20
```

#### Variables with Purify Mode
```bash
# Switch to purify mode
bashrs [normal]> :mode purify
Switched to purify mode

# Assign variable
bashrs [purify]> dir=/tmp/myapp
✓ Variable set: dir = /tmp/myapp

# Variable is expanded before purification
bashrs [purify]> mkdir $dir
✓ Purified:
Purified 1 statement(s)
```

#### Variables with Lint Mode
```bash
bashrs [normal]> :mode lint
Switched to lint mode

bashrs [lint]> filename=config.txt
✓ Variable set: filename = config.txt

bashrs [lint]> cat $filename | grep pattern
Found 1 issue(s):
  ⚠ 1 warning(s)
```

#### Unknown Variables
```bash
# Unknown variables expand to empty string (bash behavior)
bashrs [normal]> echo $unknown_var
Would execute: echo

bashrs [normal]> echo Before:$missing:After
Would execute: echo Before::After
```

#### Variable Reassignment
```bash
bashrs [normal]> status=pending
✓ Variable set: status = pending

bashrs [normal]> status=complete
✓ Variable set: status = complete

bashrs [normal]> echo $status
Would execute: echo complete
```

#### Viewing Variables
```bash
# View all session variables
bashrs [normal]> user=alice
✓ Variable set: user = alice

bashrs [normal]> role=admin
✓ Variable set: role = admin

bashrs [normal]> :vars
Session Variables (2 variables):
  role = admin
  user = alice
```

**Notes**:
- Variables persist throughout your REPL session
- Variables persist across mode switches
- Variable names must start with a letter or underscore
- Variable names can contain letters, numbers, and underscores
- Unknown variables expand to empty string (matching bash behavior)
- Use `:vars` to view all session variables

### Example 9: Script Loading and Function Extraction

**NEW in v6.20.0**: Load bash scripts from files to inspect their structure, extract functions, and develop scripts interactively.

#### Loading a Simple Script
```bash
bashrs [normal]> :load examples/hello.sh
✓ Loaded: examples/hello.sh (no functions, 3 lines)
```

#### Loading a Script with Functions
```bash
bashrs [normal]> :load examples/utils.sh
✓ Loaded: examples/utils.sh (3 functions, 25 lines)
```

#### Viewing Loaded Functions
```bash
bashrs [normal]> :functions
Available functions (3 total):
  1 log_info
  2 log_error
  3 check_dependencies
```

#### Sourcing a Script (Load and Execute)
```bash
# Source adds functions to your session
bashrs [normal]> :source examples/lib.sh
✓ Sourced: examples/lib.sh (2 functions)

bashrs [normal]> :functions
Available functions (2 total):
  1 greet
  2 farewell
```

#### Reloading a Script After Changes
```bash
# Edit the script in another window...
# Then reload it in the REPL
bashrs [normal]> :reload
Reloading: examples/utils.sh
✓ Reloaded: examples/utils.sh (4 functions)

bashrs [normal]> :functions
Available functions (4 total):
  1 log_info
  2 log_error
  3 log_warning
  4 check_dependencies
```

#### Script Loading Workflow
```bash
# Step 1: Load a script to inspect it
bashrs [normal]> :load deploy.sh
✓ Loaded: deploy.sh (5 functions, 120 lines)

# Step 2: View extracted functions
bashrs [normal]> :functions
Available functions (5 total):
  1 validate_env
  2 build_app
  3 run_tests
  4 deploy_staging
  5 deploy_production

# Step 3: Switch to lint mode to check quality
bashrs [normal]> :mode lint
Switched to lint mode

# Step 4: Reload to check latest changes
bashrs [lint]> :reload
Reloading: deploy.sh
✓ Reloaded: deploy.sh (5 functions, 125 lines)
```

#### Error Handling
```bash
# Nonexistent file
bashrs [normal]> :load missing.sh
✗ Error: Cannot read file missing.sh: No such file or directory

# Invalid syntax
bashrs [normal]> :load broken.sh
✗ Parse error: Parse error: unexpected token

# No script to reload
bashrs [normal]> :reload
No script to reload. Use :load <file> first.
```

**Use Cases**:
- **Interactive Development**: Load your script while editing to see structure
- **Function Exploration**: Quickly see all functions in a complex script
- **Live Reload**: Edit script externally, use `:reload` to see changes
- **Quality Workflow**: Load → Inspect → Lint → Purify → Reload cycle
- **Learning**: Explore example scripts to understand bash patterns

**Notes**:
- `:load` parses the script and extracts function names
- `:source` is similar to bash `source`/`.` command
- Functions are tracked in REPL state across mode switches
- `:reload` reloads the most recently loaded script
- Scripts must have valid bash syntax to load successfully
- Use `:functions` to see all currently loaded functions

## Tab Completion

**NEW in v6.20.0**: The REPL now includes intelligent tab completion to speed up your workflow and reduce typing errors.

### Command Completion

Press `Tab` to auto-complete REPL commands:

```bash
bashrs [normal]> :mo<TAB>
# Completes to: :mode

bashrs [normal]> :p<TAB>
# Shows: :parse  :purify

bashrs [normal]> :h<TAB>
# Completes to: :history
```

### Mode Name Completion

After typing `:mode`, press `Tab` to complete mode names:

```bash
bashrs [normal]> :mode pur<TAB>
# Completes to: :mode purify

bashrs [normal]> :mode <TAB>
# Shows all modes: normal  purify  lint  debug  explain
```

### Bash Construct Completion

Tab completion also works for common bash constructs:

```bash
bashrs [explain]> for<TAB>
# Completes to: for i in

bashrs [explain]> ${var:<TAB>
# Shows: ${var:-default}  ${var:=default}  ${var:?error}  ${var:+alternate}
```

### File Path Completion

**NEW in v6.20.0**: Tab completion for file paths makes loading scripts effortless:

```bash
bashrs [normal]> :load ex<TAB>
# Completes to: :load examples/

bashrs [normal]> :load examples/te<TAB>
# Completes to: :load examples/test.sh

bashrs [normal]> :source script<TAB>
# Shows all files starting with "script": script1.sh  script2.sh  script_utils.sh
```

**Features**:
- Directories are shown with trailing `/` and listed first
- Hidden files (starting with `.`) are excluded by default
- File paths are completed relative to current directory
- Works with both `:load` and `:source` commands

### Case-Insensitive Completion

Tab completion is case-insensitive for convenience:

```bash
bashrs [normal]> :MO<TAB>
# Completes to: :mode

bashrs [normal]> :mode PUR<TAB>
# Completes to: :mode purify
```

### Completion Examples

**Example 1: Quick mode switching**
```bash
bashrs [normal]> :m<TAB>pur<TAB><ENTER>
# Result: :mode purify
Switched to purify mode
```

**Example 2: Exploring commands**
```bash
bashrs [normal]> :<TAB>
# Shows all commands:
# :clear  :functions  :history  :lint  :load  :mode  :parse  :purify  :reload  :source  :vars
```

**Example 3: Learning bash constructs**
```bash
bashrs [explain]> $<TAB>
# Shows parameter expansions:
# ${var:-default}  ${var:=default}  ${var:?error}  ${var:+alternate}  ${#var}
```

## Multi-line Input

**NEW in v6.20.0**: The REPL now supports multi-line input for complex bash constructs like functions, loops, and conditionals.

When the REPL detects incomplete input, it automatically switches to continuation mode with a `... >` prompt.

### Functions

```bash
bashrs [normal]> function greet() {
... >   echo "Hello, $1"
... >   echo "Welcome to bashrs"
... > }
✓ Function 'greet' defined

bashrs [normal]> greet "Alice"
Hello, Alice
Welcome to bashrs
```

### For Loops

```bash
bashrs [normal]> for i in 1 2 3; do
... >   echo "Iteration $i"
... > done
Iteration 1
Iteration 2
Iteration 3
```

### If Statements

```bash
bashrs [normal]> if [ -f "/etc/passwd" ]; then
... >   echo "File exists"
... > fi
File exists
```

### While Loops

```bash
bashrs [normal]> count=0
✓ Variable set: count = 0

bashrs [normal]> while [ $count -lt 3 ]; do
... >   echo "Count: $count"
... >   count=$((count + 1))
... > done
Count: 0
Count: 1
Count: 2
```

### Case Statements

```bash
bashrs [normal]> case "apple" in
... >   apple) echo "It's an apple";;
... >   banana) echo "It's a banana";;
... >   *) echo "Unknown fruit";;
... > esac
It's an apple
```

### Cancelling Multi-line Input

Press `Ctrl-C` to cancel multi-line input and return to the main prompt:

```bash
bashrs [normal]> for i in 1 2 3; do
... >   echo "This is a loop"
... > ^C (multi-line input cancelled)
bashrs [normal]>
```

### Automatic Detection

The REPL intelligently detects when input is incomplete by checking for:
- Unclosed quotes (single or double)
- Unclosed braces, parentheses, or brackets
- Bash keywords expecting continuation (`if`, `for`, `while`, `function`, `case`)
- Line ending with backslash (`\`)

This allows natural, interactive development of complex bash scripts within the REPL.

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
- **Tab**: Auto-complete commands, modes, and bash constructs
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

Use the REPL to interactively learn bash constructs and transformations:

```bash
# Learn about parameter expansions
bashrs [normal]> :mode explain
Switched to explain mode

bashrs [explain]> ${var:-default}
📖 Parameter Expansion: ${parameter:-word}
   Use Default Value
...

# Learn about control flow
bashrs [explain]> for i in *.txt
📖 For Loop: for name in words
   Iterate Over List
...

# Switch to purify mode to see transformations
bashrs [explain]> :mode purify
Switched to purify mode

bashrs [purify]> rm -rf /tmp/data
✓ Purified:
Purified 1 statement(s)
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
- [Purifier Integration](../concepts/purification.md) - Transformation rules
- [Linter Integration](../linting/security.md) - Linting rules reference
