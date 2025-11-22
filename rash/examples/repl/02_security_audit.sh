#!/bin/bash
# REPL Example 02: Security Audit Workflow
# Demonstrates how to use the REPL for security auditing
#
# This example shows:
# - Linting for security issues
# - Common security vulnerabilities
# - How to fix security problems
# - Using lint mode effectively
#
# Run interactively in: bashrs repl

cat << 'EOF'
=================================================================
REPL Example 02: Security Audit Workflow
=================================================================

This example demonstrates how to use the REPL to audit bash scripts
for common security vulnerabilities.

STEP 1: Start in lint mode
---------------------------
$ bashrs repl
bashrs [normal]> :mode lint
Switched to lint mode

STEP 2: Check for unquoted variables
-------------------------------------
bashrs [lint]> cat $CONFIG_FILE | grep $PATTERN
Found 2 issue(s):
  ⚠ 2 warning(s)

[1] ⚠ SC2086 - Double quote to prevent globbing and word splitting
    Variable: CONFIG_FILE

[2] ⚠ SC2086 - Double quote to prevent globbing and word splitting
    Variable: PATTERN

Fix: cat "$CONFIG_FILE" | grep "$PATTERN"

STEP 3: Check for command injection risks
------------------------------------------
bashrs [lint]> eval $USER_INPUT
Found 1 issue(s):
  ⚠ 1 warning(s)

[1] ⚠ SEC001 - Dangerous: eval with untrusted input
    Risk: Command injection
    Fix: Avoid eval, use safer alternatives

STEP 4: Check for insecure file operations
-------------------------------------------
bashrs [lint]> rm -rf $TEMP_DIR
Found 1 issue(s):
  ⚠ 1 warning(s)

[1] ⚠ SC2086 - Unquoted variable in rm -rf
    Risk: Accidental deletion if TEMP_DIR is empty
    Fix: rm -rf "${TEMP_DIR:?Error: TEMP_DIR not set}"

STEP 5: Check for path traversal risks
---------------------------------------
bashrs [lint]> cat /var/log/$FILENAME
Found 1 issue(s):
  ⚠ 1 warning(s)

[1] ⚠ SC2086 - Unquoted variable in file path
    Risk: Path traversal if FILENAME contains ../
    Fix: Validate and quote: cat "/var/log/${FILENAME}"

STEP 6: Bulk audit a script
----------------------------
bashrs [lint]> :load vulnerable_script.sh
Found 15 issue(s):
  ✗ 3 error(s)
  ⚠ 10 warning(s)
  ℹ 2 info

Top Issues:
  1. Unquoted variables: 8 occurrences
  2. eval usage: 2 occurrences
  3. Insecure temp file creation: 1 occurrence
  4. Missing error handling: 2 occurrences

STEP 7: Get purified version
-----------------------------
bashrs [lint]> :mode purify
Switched to purify mode

bashrs [purify]> rm -rf $TEMP_DIR
✓ Purified:
rm -rf "${TEMP_DIR:?Error: TEMP_DIR not set}"

bashrs [purify]> cat $FILE | grep $PATTERN
✓ Purified:
cat "$FILE" | grep "$PATTERN"

=================================================================
Common Security Issues Detected by bashrs REPL:
=================================================================

1. SC2086 - Unquoted variables
   Impact: Word splitting, globbing, injection attacks
   Fix: Always quote variables: "$VAR"

2. SEC001 - eval usage
   Impact: Command injection
   Fix: Avoid eval, use arrays or functions instead

3. SEC002 - Insecure temp files
   Impact: Race conditions, predictable filenames
   Fix: Use mktemp: TEMP=$(mktemp)

4. SC2046 - Unquoted command substitution
   Impact: Word splitting
   Fix: Quote substitutions: "$(command)"

5. SEC003 - curl | sh patterns
   Impact: Code execution without verification
   Fix: Download, verify, then execute separately

6. SC2006 - Backticks instead of $()
   Impact: Harder to nest, less readable
   Fix: Use $() instead of backticks

7. SEC004 - Predictable random values
   Impact: Weak randomness, predictable tokens
   Fix: Use /dev/urandom instead of $RANDOM

=================================================================
Security Audit Checklist:
=================================================================

Use this checklist when auditing bash scripts:

□ All variables quoted: "$VAR"
□ No eval usage
□ Temp files created with mktemp
□ Error checking for critical operations
□ Input validation for user-provided data
□ No predictable random values ($RANDOM)
□ No curl | sh patterns
□ File permissions set explicitly (chmod)
□ No hardcoded credentials
□ Logging for security-relevant events

=================================================================
Real-World Example: Auditing a Deployment Script
=================================================================

bashrs [lint]> :load deploy.sh

# Review issues found
bashrs [lint]> :history

# Fix high-priority issues first
bashrs [lint]> :mode purify

# Get purified version for each issue
bashrs [purify]> eval $DEPLOY_COMMAND
✓ Suggestion: Replace eval with function call

bashrs [purify]> TOKEN=$RANDOM
✓ Purified: TOKEN=$(< /dev/urandom tr -dc 'a-zA-Z0-9' | head -c 32)

bashrs [purify]> curl -sSL https://install.sh | bash
✓ Suggestion:
  wget -O install.sh https://install.sh
  # Review install.sh manually
  chmod +x install.sh
  ./install.sh

=================================================================
Key Takeaways:
=================================================================

1. Lint mode automatically checks all commands for security issues
2. Always quote variables to prevent injection attacks
3. Avoid eval - it's almost always dangerous
4. Use purify mode to get secure versions of commands
5. Security audit should be part of every script review

Next Steps:
-----------
Try example 03_purification_workflow.sh to learn about making
scripts idempotent and deterministic!
EOF
