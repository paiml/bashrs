# Tutorial: Your First REPL Session

This hands-on tutorial will guide you through your first bashrs REPL session, teaching you the fundamentals through practical exercises.

**Time**: 15-20 minutes
**Skill Level**: Beginner
**Prerequisites**: bashrs installed (`cargo install bashrs`)

## What You'll Learn

By the end of this tutorial, you'll be able to:

1. Start and navigate the REPL
2. Use all 5 interactive modes
3. Parse, lint, and purify bash code
4. Work with variables and functions
5. Load and analyze scripts
6. Use tab completion and multi-line input

Let's get started!

## Step 1: Starting the REPL

Open your terminal and start the REPL:

```bash
$ bashrs repl
```

You should see:

```text
bashrs REPL v6.32.1
Type 'quit' or 'exit' to exit, 'help' for commands
Current mode: normal - Execute bash commands directly
bashrs [normal]>
```

**What just happened?**
- The REPL initialized successfully
- You're in `normal` mode (shown in the prompt)
- The cursor is ready for your first command

**Try it**: Type `help` and press Enter to see all available commands:

```bash
bashrs [normal]> help
```

## Step 2: Your First Command

Let's try a simple bash command:

```bash
bashrs [normal]> echo "Hello, REPL!"
Hello, REPL!
```

**What happened?**
- In normal mode, the REPL executes bash commands directly
- The output is displayed immediately

**Try it**: Experiment with a few more commands:

```bash
bashrs [normal]> pwd
/home/user/projects

bashrs [normal]> ls -la
# Lists files in current directory

bashrs [normal]> date
Fri Nov 22 10:30:00 UTC 2024
```

## Step 3: Parsing Bash Code

Now let's see how bash interprets your code. Use the `:parse` command:

```bash
bashrs [normal]> :parse echo "Hello, World!"
✓ Parse successful!
Statements: 1
Parse time: 0ms

AST:
  [0] SimpleCommand {
    name: "echo",
    args: ["Hello, World!"]
  }
```

**What happened?**
- The `:parse` command shows you the Abstract Syntax Tree (AST)
- This reveals how bash understands your command
- Useful for debugging complex syntax

**Try it**: Parse a more complex command:

```bash
bashrs [normal]> :parse if [ -f /etc/passwd ]; then echo "File exists"; fi
✓ Parse successful!
Statements: 1

AST:
  [0] If {
    condition: Test {
      operator: "-f",
      args: ["/etc/passwd"]
    },
    then_body: [
      SimpleCommand {
        name: "echo",
        args: ["File exists"]
      }
    ],
    else_body: None
  }
```

## Step 4: Linting for Security Issues

Let's check some bash code for problems. Use the `:lint` command:

```bash
bashrs [normal]> :lint cat file.txt | grep $PATTERN
Found 1 issue(s):
  ⚠ 1 warning(s)

[1] ⚠ SC2086 - Double quote to prevent globbing and word splitting
    Line 1
    Variable: PATTERN

Fix: cat file.txt | grep "$PATTERN"
```

**What happened?**
- The linter found an unquoted variable (`$PATTERN`)
- This could cause security issues or unexpected behavior
- The linter suggests adding quotes: `"$PATTERN"`

**Try it**: Test some problematic code:

```bash
bashrs [normal]> :lint rm -rf $DIR && mkdir $DIR
Found 2 issue(s):
  ⚠ 2 warning(s)

[1] ⚠ SC2086 - Double quote to prevent globbing and word splitting
    Variable: DIR

[2] ⚠ SC2086 - Double quote to prevent globbing and word splitting
    Variable: DIR
```

## Step 5: Switching to Lint Mode

Instead of typing `:lint` before every command, switch to lint mode:

```bash
bashrs [normal]> :mode lint
Switched to lint mode - Show linting results for bash commands
bashrs [lint]>
```

**What happened?**
- The prompt changed to `[lint]`
- Now all commands are automatically linted

**Try it**: Type commands without the `:lint` prefix:

```bash
bashrs [lint]> cat $FILE | grep pattern
Found 1 issue(s):
  ⚠ 1 warning(s)

[1] ⚠ SC2086 - Double quote to prevent globbing and word splitting
    Variable: FILE

bashrs [lint]> echo "Hello, World!"
✓ No issues found!
```

**Key Point**: Automatic mode-based processing saves you 50% typing!

## Step 6: Purifying Non-Idempotent Code

Switch to purify mode to fix idempotency issues:

```bash
bashrs [lint]> :mode purify
Switched to purify mode - Show purified version of bash commands
bashrs [purify]>
```

Now try some non-idempotent commands:

```bash
bashrs [purify]> mkdir /tmp/myapp
✓ Purified:
mkdir -p "/tmp/myapp"

bashrs [purify]> rm /tmp/old_file
✓ Purified:
rm -f "/tmp/old_file"

bashrs [purify]> ln -s /tmp/source /tmp/link
✓ Purified:
ln -sf "/tmp/source" "/tmp/link"
```

**What happened?**
- `mkdir` became `mkdir -p` (won't fail if directory exists)
- `rm` became `rm -f` (won't fail if file doesn't exist)
- `ln -s` became `ln -sf` (won't fail if link already exists)
- All paths are quoted for safety

**Try it**: Test non-deterministic code:

```bash
bashrs [purify]> SESSION_ID=$RANDOM
✓ Purified:
SESSION_ID="$(date +%s)-$$"
```

The purifier replaced `$RANDOM` (non-deterministic) with a timestamp-based ID (deterministic).

## Step 7: Learning with Explain Mode

Switch to explain mode to learn bash constructs:

```bash
bashrs [purify]> :mode explain
Switched to explain mode - Explain bash constructs and syntax
bashrs [explain]>
```

Try some bash constructs:

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
```

**What happened?**
- Each bash construct gets a detailed explanation
- You see the syntax, purpose, and examples
- Perfect for learning new bash patterns

## Step 8: Working with Variables

Switch back to normal mode and create some variables:

```bash
bashrs [explain]> :mode normal
Switched to normal mode
bashrs [normal]>
```

Set variables like in bash:

```bash
bashrs [normal]> app_name="myapp"
✓ Variable set: app_name = myapp

bashrs [normal]> version=1.0.0
✓ Variable set: version = 1.0.0

bashrs [normal]> env=production
✓ Variable set: env = production
```

Use variables in commands:

```bash
bashrs [normal]> echo "Deploying $app_name version $version to $env"
Deploying myapp version 1.0.0 to production
```

View all session variables:

```bash
bashrs [normal]> :vars
Session Variables (3 variables):
  app_name = myapp
  env = production
  version = 1.0.0
```

**Key Point**: Variables persist across mode switches and throughout your session.

## Step 9: Multi-line Input

The REPL supports complex multi-line constructs. Try defining a function:

```bash
bashrs [normal]> function greet() {
... >   echo "Hello, $1"
... >   echo "Welcome to bashrs!"
... > }
✓ Function 'greet' defined
```

**What happened?**
- The REPL detected incomplete input after the opening `{`
- It switched to continuation mode with the `... >` prompt
- After the closing `}`, the function was defined

**Try it**: Create a for loop:

```bash
bashrs [normal]> for i in 1 2 3; do
... >   echo "Processing item $i"
... > done
Processing item 1
Processing item 2
Processing item 3
```

Create an if statement:

```bash
bashrs [normal]> if [ -f /etc/passwd ]; then
... >   echo "File exists"
... > else
... >   echo "File not found"
... > fi
File exists
```

**Tip**: Press `Ctrl-C` to cancel multi-line input if you make a mistake:

```bash
bashrs [normal]> for i in 1 2 3; do
... >   echo "This is wrong"
... > ^C (multi-line input cancelled)
bashrs [normal]>
```

## Step 10: Tab Completion

Tab completion speeds up your workflow. Try these:

```bash
# Command completion
bashrs [normal]> :mo<TAB>
# Completes to: :mode

# Mode completion
bashrs [normal]> :mode pur<TAB>
# Completes to: :mode purify

# View all commands
bashrs [normal]> :<TAB>
# Shows: :clear :functions :history :lint :load :mode :parse :purify :reload :source :vars
```

**Key Point**: Use Tab liberally to discover commands and reduce typos.

## Step 11: Loading Scripts

Create a simple test script first:

```bash
$ cat > /tmp/test_script.sh << 'EOF'
#!/bin/bash

function log_info() {
  echo "[INFO] $1"
}

function log_error() {
  echo "[ERROR] $1" >&2
}

function deploy() {
  log_info "Starting deployment..."
  log_info "Deployment complete"
}

log_info "Script loaded"
EOF
```

Now load it in the REPL:

```bash
bashrs [normal]> :load /tmp/test_script.sh
✓ Loaded: /tmp/test_script.sh (3 functions, 15 lines)
```

View the extracted functions:

```bash
bashrs [normal]> :functions
Available functions (3 total):
  1 log_info
  2 log_error
  3 deploy
```

Edit the script externally, then reload:

```bash
# Edit /tmp/test_script.sh in your editor
# Add a new function...

bashrs [normal]> :reload
Reloading: /tmp/test_script.sh
✓ Reloaded: /tmp/test_script.sh (4 functions, 20 lines)

bashrs [normal]> :functions
Available functions (4 total):
  1 log_info
  2 log_error
  3 deploy
  4 rollback
```

## Step 12: Viewing History

See all commands you've executed:

```bash
bashrs [normal]> :history
Command History (25 commands):
  1 help
  2 echo "Hello, REPL!"
  3 pwd
  4 :parse echo "Hello, World!"
  5 :lint cat file.txt | grep $PATTERN
  6 :mode lint
  7 cat $FILE | grep pattern
  ... (truncated)
  25 :history
```

**Tip**: Use the up/down arrow keys to navigate history:

```bash
bashrs [normal]> # Press ↑ to recall previous commands
```

**Tip**: Use `Ctrl-R` for reverse search:

```bash
# Press Ctrl-R, then type "parse"
(reverse-i-search)'parse': :parse echo "Hello, World!"
```

## Step 13: Clearing the Screen

When your terminal gets cluttered:

```bash
bashrs [normal]> :clear
# Screen cleared, fresh prompt
bashrs [normal]>
```

Or use the keyboard shortcut `Ctrl-L`.

## Step 14: Practical Workflow Example

Let's put it all together with a real-world workflow. Imagine you're writing a deployment script.

**Step 14.1**: Set up your environment variables:

```bash
bashrs [normal]> env=staging
✓ Variable set: env = staging

bashrs [normal]> region=us-west-2
✓ Variable set: region = us-west-2

bashrs [normal]> app_version=v2.1.0
✓ Variable set: app_version = v2.1.0
```

**Step 14.2**: Test a command in lint mode:

```bash
bashrs [normal]> :mode lint
Switched to lint mode

bashrs [lint]> docker build -t $app_version .
Found 1 issue(s):
  ⚠ 1 warning(s)

[1] ⚠ SC2086 - Double quote to prevent globbing and word splitting
    Variable: app_version
```

**Step 14.3**: Get the purified version:

```bash
bashrs [lint]> :mode purify
Switched to purify mode

bashrs [purify]> docker build -t $app_version .
✓ Purified:
docker build -t "$app_version" .
```

**Step 14.4**: Build up your script incrementally:

```bash
bashrs [purify]> docker build -t "$app_version" .
✓ Purified: docker build -t "$app_version" .

bashrs [purify]> docker tag "$app_version" "registry.example.com/$app_version"
✓ Purified: docker tag "$app_version" "registry.example.com/$app_version"

bashrs [purify]> docker push "registry.example.com/$app_version"
✓ Purified: docker push "registry.example.com/$app_version"
```

**Step 14.5**: Review your history:

```bash
bashrs [purify]> :history
Command History (10 commands):
  ...
  7 docker build -t "$app_version" .
  8 docker tag "$app_version" "registry.example.com/$app_version"
  9 docker push "registry.example.com/$app_version"
  10 :history
```

**Step 14.6**: Copy the successful commands to your deploy script!

## Step 15: Exiting the REPL

When you're done, exit the REPL:

```bash
bashrs [normal]> quit
Goodbye!
$
```

Or use the `exit` command, or press `Ctrl-D`.

**What happened?**
- Your command history was saved to `~/.bashrs_history`
- Next time you start the REPL, you can use ↑ to recall these commands

## Congratulations!

You've completed your first REPL session and learned:

✅ How to start and exit the REPL
✅ Using all 5 interactive modes (normal, purify, lint, debug, explain)
✅ Parsing bash code to see the AST
✅ Linting for security issues
✅ Purifying non-idempotent code
✅ Learning bash constructs with explain mode
✅ Working with variables and functions
✅ Multi-line input for complex constructs
✅ Tab completion for faster typing
✅ Loading and reloading scripts
✅ Viewing and searching command history
✅ Practical workflow for script development

## Practice Exercises

Ready to practice? Try these exercises:

### Exercise 1: Security Audit

Lint this problematic script and identify all issues:

```bash
bashrs [normal]> :mode lint
bashrs [lint]> rm -rf $TEMP_DIR
bashrs [lint]> cat $CONFIG_FILE | grep $SEARCH_TERM
bashrs [lint]> eval $COMMAND
```

### Exercise 2: Purification

Purify these non-idempotent commands:

```bash
bashrs [normal]> :mode purify
bashrs [purify]> mkdir /var/log/myapp
bashrs [purify]> ln -s /opt/myapp/current /usr/local/bin/myapp
bashrs [purify]> cp config.txt config.txt.bak
```

### Exercise 3: Learning Bash

Use explain mode to understand these constructs:

```bash
bashrs [normal]> :mode explain
bashrs [explain]> ${var:=default}
bashrs [explain]> case $option in
bashrs [explain]> [[ -f file ]]
```

### Exercise 4: Script Development

Create a simple script with 2 functions, load it in the REPL, lint it, purify it, and reload after making changes.

### Exercise 5: Real-World Workflow

Build a complete CI/CD deployment command using:
1. Variables for environment settings
2. Linting to check for issues
3. Purification to ensure idempotency
4. History to review and save your work

## Next Steps

Now that you've mastered the basics:

1. **[REPL User Guide](./user-guide.md)** - Comprehensive reference for all features
2. **[REPL Commands Reference](../reference/repl-commands.md)** - Complete command documentation
3. **[Examples](../../examples/repl/)** - Real-world REPL usage examples
4. **[Purification Concepts](../concepts/purification.md)** - Deep dive into transformations
5. **[Linting Rules](../linting/security.md)** - Understanding security rules

## Tips for Success

### Tip 1: Start Every Session with Variables

Build your context first:

```bash
bashrs [normal]> env=production
bashrs [normal]> region=us-east-1
bashrs [normal]> # Now use these throughout your session
```

### Tip 2: Use Modes Strategically

- Start in **normal** to set up
- Switch to **lint** to find issues
- Switch to **purify** to fix them
- Use **explain** to learn
- Return to **normal** to test

### Tip 3: Leverage History

Your history is a goldmine:

```bash
bashrs [normal]> :history
# Find working commands and build scripts from them
```

### Tip 4: Combine with External Editors

Edit scripts in your favorite editor, reload in the REPL:

```bash
# Terminal 1: Your editor
$ vim deploy.sh

# Terminal 2: REPL
bashrs [normal]> :load deploy.sh
bashrs [normal]> # Make changes in vim
bashrs [normal]> :reload
```

### Tip 5: Use Tab Completion Everywhere

Press Tab liberally to:
- Discover commands
- Complete mode names
- Find files quickly
- Reduce typos

## Getting Help

If you get stuck:

```bash
# In the REPL
bashrs [normal]> help

# View current mode
bashrs [normal]> :mode

# For specific commands
bashrs [normal]> :parse
Usage: :parse <bash_code>
```

Happy scripting! 🚀
