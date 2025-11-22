#!/bin/bash
# REPL Example 01: Basic Workflow
# Demonstrates the fundamental REPL workflow for beginners
#
# This example shows:
# - Starting the REPL
# - Basic commands (parse, lint, purify)
# - Mode switching
# - Variables and history
#
# Run this example interactively:
#   $ bashrs repl
# Then follow along with the commands below

cat << 'EOF'
=================================================================
REPL Example 01: Basic Workflow
=================================================================

This example demonstrates a basic REPL workflow for beginners.

STEP 1: Starting the REPL
--------------------------
$ bashrs repl

You should see:
  bashrs REPL v6.32.1
  Type 'quit' or 'exit' to exit, 'help' for commands
  Current mode: normal - Execute bash commands directly
  bashrs [normal]>

STEP 2: Try basic commands
---------------------------
bashrs [normal]> echo "Hello, REPL!"
Hello, REPL!

bashrs [normal]> pwd
/home/user/projects

STEP 3: Parse bash code
------------------------
bashrs [normal]> :parse echo "Hello, World!"
✓ Parse successful!
Statements: 1

STEP 4: Lint for security issues
---------------------------------
bashrs [normal]> :lint cat $FILE | grep pattern
Found 1 issue(s):
  ⚠ 1 warning(s)
[1] ⚠ SC2086 - Unquoted variable

STEP 5: Switch to purify mode
------------------------------
bashrs [normal]> :mode purify
Switched to purify mode

bashrs [purify]> mkdir /tmp/myapp
✓ Purified: mkdir -p "/tmp/myapp"

STEP 6: Work with variables
----------------------------
bashrs [purify]> :mode normal
Switched to normal mode

bashrs [normal]> app_name="myapp"
✓ Variable set: app_name = myapp

bashrs [normal]> version=1.0.0
✓ Variable set: version = 1.0.0

bashrs [normal]> echo $app_name v$version
myapp v1.0.0

bashrs [normal]> :vars
Session Variables (2 variables):
  app_name = myapp
  version = 1.0.0

STEP 7: View history
--------------------
bashrs [normal]> :history
Command History:
  1 echo "Hello, REPL!"
  2 pwd
  3 :parse echo "Hello, World!"
  ... (truncated)

STEP 8: Exit
------------
bashrs [normal]> quit
Goodbye!

=================================================================
Key Takeaways:
=================================================================

1. Use :parse to understand how bash interprets your code
2. Use :lint to find security and safety issues
3. Use :purify to fix idempotency problems
4. Switch modes with :mode <name> for automatic processing
5. Variables persist throughout your session
6. History is saved to ~/.bashrs_history

Next Steps:
-----------
Try example 02_security_audit.sh to learn about security linting!
EOF
