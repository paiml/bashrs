# REPL User Guide

The bashrs REPL (Read-Eval-Print Loop) is an interactive shell analysis environment that helps you write safer, more maintainable shell scripts through real-time feedback and transformations.

## Why Use the REPL?

The bashrs REPL transforms how you work with shell scripts:

1. **Learn by Doing**: Experiment with bash constructs and see immediate transformations
2. **Rapid Development**: Test script snippets without creating files
3. **Quality Assurance**: Get real-time linting and purification feedback
4. **Interactive Debugging**: Step through scripts and understand their behavior
5. **Safe Experimentation**: Try dangerous commands in a safe environment

## Core Philosophy

The REPL embodies three key principles:

### 1. Immediate Feedback

Every command you type is instantly analyzed, providing feedback on:
- Syntax correctness (parsing)
- Security issues (linting)
- Idempotency problems (purification)
- Determinism violations (transformation)

```bash
bashrs [normal]> :lint rm -rf $DIR
Found 1 issue(s):
  ⚠ 1 warning(s)

[1] ⚠ SC2086 - Double quote to prevent globbing and word splitting
    Variable: DIR
```

### 2. Mode-Based Workflow

The REPL supports 5 distinct modes, each optimized for a specific task:

| Mode | Purpose | Best For |
|------|---------|----------|
| **normal** | Direct execution | Testing commands, learning bash |
| **purify** | Automatic purification | Fixing idempotency issues |
| **lint** | Automatic linting | Finding security problems |
| **debug** | Step-by-step execution | Understanding complex scripts |
| **explain** | Interactive explanations | Learning bash constructs |

Switch modes with `:mode <name>` and let the REPL automatically process your commands.

### 3. Persistent State

The REPL maintains session state across commands:

- **Variables**: Set once, use everywhere
- **Functions**: Load scripts, extract functions
- **History**: Full command history saved to `~/.bashrs_history`
- **Context**: Mode and settings persist until changed

## Key Features

### 🎯 Five Interactive Modes

Each mode provides a different lens for analyzing your scripts:

```bash
# Normal: Execute commands directly
bashrs [normal]> echo "Hello, World!"
Hello, World!

# Purify: Automatic idempotency transformation
bashrs [normal]> :mode purify
bashrs [purify]> mkdir /tmp/app
✓ Purified: mkdir -p "/tmp/app"

# Lint: Automatic security analysis
bashrs [purify]> :mode lint
bashrs [lint]> cat $FILE | grep pattern
Found 1 issue(s): ⚠ SC2086 - Unquoted variable

# Explain: Interactive learning
bashrs [lint]> :mode explain
bashrs [explain]> ${var:-default}
📖 Parameter Expansion: Use Default Value...

# Debug: Step-by-step execution (future)
bashrs [explain]> :mode debug
bashrs [debug]> # Step through complex scripts
```

### ⌨️ Tab Completion

Intelligent completion speeds up your workflow:

```bash
# Command completion
bashrs [normal]> :mo<TAB>
# Completes to: :mode

# Mode completion
bashrs [normal]> :mode pur<TAB>
# Completes to: :mode purify

# File path completion
bashrs [normal]> :load ex<TAB>
# Completes to: :load examples/

# Bash construct completion
bashrs [explain]> for<TAB>
# Shows: for i in ...
```

### 📝 Multi-line Input

Natural support for complex constructs:

```bash
bashrs [normal]> function deploy() {
... >   echo "Deploying application..."
... >   mkdir -p /var/log/app
... >   systemctl restart app
... > }
✓ Function 'deploy' defined

bashrs [normal]> for i in 1 2 3; do
... >   echo "Processing item $i"
... > done
Processing item 1
Processing item 2
Processing item 3
```

### 🔍 Parser Integration

See exactly how bash interprets your code:

```bash
bashrs [normal]> :parse if [ -f config.txt ]; then cat config.txt; fi
✓ Parse successful!
Statements: 1

AST:
  [0] If {
    condition: Test { operator: "-f", args: ["config.txt"] },
    then_body: [ SimpleCommand { name: "cat", args: ["config.txt"] } ],
    else_body: None
  }
```

### 🧹 Purifier Integration

Transform non-deterministic, non-idempotent code:

```bash
# Automatic purification in purify mode
bashrs [purify]> mkdir /tmp/test
✓ Purified:
mkdir -p "/tmp/test"

bashrs [purify]> SESSION_ID=$RANDOM
✓ Purified:
SESSION_ID="$(date +%s)-$$"

bashrs [purify]> rm old_file
✓ Purified:
rm -f "old_file"
```

### 🔎 Linter Integration

Real-time diagnostics with severity levels:

```bash
bashrs [lint]> cat file.txt | grep $PATTERN
Found 1 issue(s):
  ⚠ 1 warning(s)

[1] ⚠ SC2086 - Double quote to prevent globbing and word splitting
    Line 1
    Variable: PATTERN

Fix: cat file.txt | grep "$PATTERN"
```

### 📚 Command History

Persistent history with powerful navigation:

```bash
# Navigate with arrow keys
↑/↓   - Previous/Next command

# Search history
Ctrl-R - Reverse search

# View session history
bashrs [normal]> :history
Command History (5 commands):
  1 echo hello
  2 :parse if [ -f test ]; then echo found; fi
  3 :lint cat file | grep pattern
  4 :mode purify
  5 :history
```

### 📦 Script Loading

Load and analyze complete scripts:

```bash
# Load a script to inspect it
bashrs [normal]> :load deploy.sh
✓ Loaded: deploy.sh (5 functions, 120 lines)

# View extracted functions
bashrs [normal]> :functions
Available functions (5 total):
  1 validate_env
  2 build_app
  3 run_tests
  4 deploy_staging
  5 deploy_production

# Reload after editing
bashrs [normal]> :reload
Reloading: deploy.sh
✓ Reloaded: deploy.sh (5 functions, 125 lines)
```

### 🔧 Session Variables

Bash-style variable assignment and expansion:

```bash
bashrs [normal]> app_name="myapp"
✓ Variable set: app_name = myapp

bashrs [normal]> version=1.0.0
✓ Variable set: version = 1.0.0

bashrs [normal]> echo $app_name v$version
myapp v1.0.0

# View all variables
bashrs [normal]> :vars
Session Variables (2 variables):
  app_name = myapp
  version = 1.0.0
```

## Common Workflows

### Workflow 1: Learning Bash

Perfect for beginners and experienced users exploring new constructs:

```bash
# Step 1: Start in explain mode
$ bashrs repl
bashrs [normal]> :mode explain
Switched to explain mode

# Step 2: Explore bash constructs
bashrs [explain]> ${var:-default}
📖 Parameter Expansion: ${parameter:-word}
   Use Default Value

If parameter is unset or null, expand to 'word'.
Example:
  $ var=""
  $ echo "${var:-fallback}"  # Outputs: fallback

# Step 3: Try variations
bashrs [explain]> ${var:=default}
📖 Parameter Expansion: ${parameter:=word}
   Assign Default Value...

# Step 4: Test with parse
bashrs [explain]> :parse echo ${var:-default}
✓ Parse successful!

# Step 5: Switch to normal mode and try it
bashrs [explain]> :mode normal
bashrs [normal]> echo ${missing:-"fallback"}
fallback
```

### Workflow 2: Script Development

Iterative development with immediate feedback:

```bash
# Step 1: Start in normal mode, load your script
$ bashrs repl
bashrs [normal]> :load myapp.sh
✓ Loaded: myapp.sh (3 functions, 50 lines)

# Step 2: Check for issues in lint mode
bashrs [normal]> :mode lint
bashrs [lint]> :load myapp.sh
Found 5 issue(s):
  ⚠ 3 warning(s)
  ℹ 2 info

# Step 3: Fix issues in your editor
# (Edit myapp.sh externally)

# Step 4: Reload and verify
bashrs [lint]> :reload
Reloading: myapp.sh
✓ No issues found!

# Step 5: Check purification
bashrs [lint]> :mode purify
bashrs [purify]> :load myapp.sh
✓ Script is already purified!
```

### Workflow 3: Quick Validation

Validate bash snippets before committing:

```bash
# Step 1: Copy code to clipboard
# (e.g., from your editor)

# Step 2: Start REPL in lint mode
$ bashrs repl
bashrs [normal]> :mode lint
Switched to lint mode

# Step 3: Paste and check
bashrs [lint]> rm -rf $TEMP_DIR && mkdir $TEMP_DIR
Found 2 issue(s):
  ⚠ 2 warning(s)

[1] ⚠ SC2086 - Unquoted variable: TEMP_DIR
[2] ⚠ SC2086 - Unquoted variable: TEMP_DIR

# Step 4: Get purified version
bashrs [lint]> :mode purify
bashrs [purify]> rm -rf $TEMP_DIR && mkdir $TEMP_DIR
✓ Purified:
rm -rf "$TEMP_DIR" && mkdir -p "$TEMP_DIR"

# Step 5: Copy purified code back to your editor
```

### Workflow 4: Interactive Debugging

Understand complex scripts step by step:

```bash
# Step 1: Load a complex script
bashrs [normal]> :load complex_deploy.sh
✓ Loaded: complex_deploy.sh (10 functions, 300 lines)

# Step 2: Parse specific sections
bashrs [normal]> :parse cat complex_deploy.sh | grep "function validate"
✓ Parse successful!

# Step 3: Explain confusing constructs
bashrs [normal]> :mode explain
bashrs [explain]> ${VERSION:?Error: VERSION not set}
📖 Parameter Expansion: ${parameter:?word}
   Error if Null or Unset...

# Step 4: Test in normal mode
bashrs [explain]> :mode normal
bashrs [normal]> unset VERSION
bashrs [normal]> echo ${VERSION:?Error: VERSION not set}
bash: VERSION: Error: VERSION not set
```

### Workflow 5: CI/CD Pipeline Development

Build and test deployment scripts:

```bash
# Step 1: Create variables for your environment
bashrs [normal]> env=staging
bashrs [normal]> region=us-west-2
bashrs [normal]> app_version=v2.1.0

# Step 2: Test deployment commands
bashrs [normal]> echo "Deploying $app_version to $env in $region"
Deploying v2.1.0 to staging in us-west-2

# Step 3: Check for issues
bashrs [normal]> :lint docker build -t $app_version .
Found 1 issue(s):
  ⚠ 1 warning(s)

# Step 4: Build purified script
bashrs [normal]> :mode purify
bashrs [purify]> docker build -t $app_version .
✓ Purified:
docker build -t "$app_version" .

# Step 5: Save history to script file
bashrs [purify]> :history
Command History (...)
# Copy relevant commands to deploy.sh
```

## Advanced Tips

### Tip 1: Combine Modes with Explicit Commands

Even in purify/lint mode, you can use explicit commands:

```bash
bashrs [purify]> mkdir /tmp/test
✓ Purified: mkdir -p "/tmp/test"

# But you can still parse or lint explicitly
bashrs [purify]> :parse mkdir /tmp/test
✓ Parse successful!

bashrs [purify]> :lint mkdir /tmp/test
✓ No issues found!
```

### Tip 2: Use Variables for Complex Commands

Build up complex commands incrementally:

```bash
bashrs [normal]> host=example.com
bashrs [normal]> port=8080
bashrs [normal]> protocol=https

bashrs [normal]> url="$protocol://$host:$port/api"
✓ Variable set: url = https://example.com:8080/api

bashrs [normal]> curl $url/health
# Test the command
```

### Tip 3: Load Multiple Scripts

Build a development environment:

```bash
bashrs [normal]> :load lib/utils.sh
✓ Loaded: lib/utils.sh (5 functions)

bashrs [normal]> :load lib/deploy.sh
✓ Loaded: lib/deploy.sh (3 functions)

bashrs [normal]> :functions
Available functions (8 total):
  1 log_info
  2 log_error
  3 check_deps
  4 validate_env
  5 retry_command
  6 deploy_app
  7 rollback_app
  8 health_check
```

### Tip 4: Use History Search

Find previous commands quickly:

```bash
# Press Ctrl-R, then type search term
(reverse-i-search)'purify': :purify mkdir /tmp/app

# Press Ctrl-R again to cycle through matches
```

### Tip 5: Clear Screen for Fresh Start

Keep your terminal organized:

```bash
bashrs [normal]> :clear
# Or use Ctrl-L

# Fresh prompt, clean slate
bashrs [normal]>
```

## Configuration Options

Customize REPL behavior with command-line flags:

```bash
# Enable debug mode for verbose output
$ bashrs repl --debug

# Set resource limits
$ bashrs repl --max-memory 200 --timeout 60

# Run in sandboxed mode (restricted operations)
$ bashrs repl --sandboxed

# Combine options
$ bashrs repl --debug --max-memory 1000 --timeout 300
```

### Available Options

| Option | Default | Description |
|--------|---------|-------------|
| `--debug` | false | Enable debug mode with verbose output |
| `--max-memory <MB>` | 500 | Maximum memory usage in MB |
| `--timeout <SECONDS>` | 120 | Command timeout in seconds |
| `--max-depth <N>` | 1000 | Maximum recursion depth |
| `--sandboxed` | false | Run in sandboxed mode (restricted operations) |

## Troubleshooting

### Problem: REPL Won't Start

**Symptoms**:
```bash
$ bashrs repl
Failed to initialize REPL: Terminal error
```

**Solutions**:
1. Check terminal compatibility:
   ```bash
   $ echo $TERM
   # Should show: xterm-256color, screen-256color, etc.
   ```

2. Test ANSI support:
   ```bash
   $ echo -e "\e[1mBold\e[0m"
   # Should display "Bold" in bold text
   ```

3. Try a different terminal (e.g., iTerm2, GNOME Terminal, Windows Terminal)

### Problem: History Not Persisting

**Symptoms**: Commands not saved across sessions

**Solutions**:
1. Check history file permissions:
   ```bash
   $ ls -la ~/.bashrs_history
   # Should be readable/writable by your user

   $ chmod 600 ~/.bashrs_history
   ```

2. Check disk space:
   ```bash
   $ df -h ~
   ```

3. Verify write permissions:
   ```bash
   $ touch ~/.bashrs_history
   $ echo "test" >> ~/.bashrs_history
   ```

### Problem: Tab Completion Not Working

**Symptoms**: Tab key doesn't complete commands

**Solutions**:
1. Ensure you're using a compatible terminal
2. Update to latest bashrs version
3. Check if Ctrl-I works as alternative to Tab

### Problem: Out of Memory

**Symptoms**:
```bash
REPL out of memory
```

**Solutions**:
1. Increase memory limit:
   ```bash
   $ bashrs repl --max-memory 1000  # 1GB
   ```

2. Avoid loading very large scripts (>1MB)

3. Clear history periodically:
   ```bash
   $ rm ~/.bashrs_history
   ```

### Problem: Multiline Input Stuck

**Symptoms**: REPL shows `... >` prompt indefinitely

**Solutions**:
1. Press `Ctrl-C` to cancel multiline input
2. Complete the construct (add missing `fi`, `done`, `}`, etc.)
3. Check for unclosed quotes

### Problem: Variables Not Expanding

**Symptoms**: `echo $var` shows literal `$var`

**Solutions**:
1. Ensure variable was set:
   ```bash
   bashrs [normal]> :vars
   ```

2. Check variable name syntax (must start with letter/underscore)

3. Verify you're in normal mode (variable expansion works in all modes)

## Best Practices

### 1. Start in Normal Mode

Begin every session in normal mode to establish your environment:

```bash
$ bashrs repl
bashrs [normal]> # Set up variables
bashrs [normal]> env=production
bashrs [normal]> region=us-east-1
```

### 2. Use Modes for Specific Tasks

Switch to specialized modes when needed:

- **Purify mode**: When fixing idempotency issues
- **Lint mode**: When checking security
- **Explain mode**: When learning new constructs
- **Debug mode**: When troubleshooting complex scripts

### 3. Leverage Tab Completion

Save time and reduce errors:

```bash
bashrs [normal]> :mo<TAB>pur<TAB><ENTER>
# Result: :mode purify
```

### 4. Use :history to Build Scripts

Review your session and extract working commands:

```bash
bashrs [normal]> :history
# Copy successful commands to a script file
```

### 5. Load Scripts Early

Start with your script loaded for context:

```bash
$ bashrs repl
bashrs [normal]> :load myproject.sh
bashrs [normal]> # Now work with your script context
```

### 6. Combine Linting and Purification

Always check both:

```bash
bashrs [normal]> :lint rm $FILE
# Check for issues

bashrs [normal]> :purify rm $FILE
# Get safe version
```

### 7. Use Variables for Repetitive Values

Reduce typing and errors:

```bash
bashrs [normal]> base_dir=/var/lib/myapp
bashrs [normal]> config_dir=$base_dir/config
bashrs [normal]> data_dir=$base_dir/data
```

## Keyboard Shortcuts Reference

| Shortcut | Action |
|----------|--------|
| `Tab` | Auto-complete commands, modes, files |
| `Ctrl-C` | Cancel current line / Cancel multiline input |
| `Ctrl-D` | Exit REPL (EOF) |
| `Ctrl-L` | Clear screen (alias for `:clear`) |
| `Ctrl-U` | Delete line before cursor |
| `Ctrl-K` | Delete line after cursor |
| `Ctrl-W` | Delete word before cursor |
| `↑` | Previous command (history) |
| `↓` | Next command (history) |
| `Ctrl-R` | Reverse search history |
| `Ctrl-A` | Move cursor to start of line |
| `Ctrl-E` | Move cursor to end of line |

## Next Steps

Now that you understand the REPL fundamentals, explore:

1. **[Tutorial: Your First REPL Session](./tutorial.md)** - Hands-on walkthrough
2. **[REPL Commands Reference](../reference/repl-commands.md)** - Complete command reference
3. **[Purification Concepts](../concepts/purification.md)** - Understanding transformations
4. **[Linting Rules](../linting/security.md)** - Security and quality rules

## See Also

- [Interactive REPL (Getting Started)](../getting-started/repl.md) - Quick start guide
- [CLI Commands Reference](../reference/cli.md) - bashrs CLI documentation
- [Configuration](../reference/configuration.md) - Configuration options
