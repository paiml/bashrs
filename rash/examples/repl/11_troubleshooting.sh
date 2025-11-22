#!/bin/bash
# REPL Example 11: Troubleshooting Guide
# Comprehensive guide to debugging and resolving REPL issues
#
# This example shows:
# - Common problems and solutions
# - Error message interpretation
# - Debugging techniques
# - Recovery procedures
#
# Run interactively in: bashrs repl

cat << 'EOF'
=================================================================
REPL Example 11: Troubleshooting Guide
=================================================================

This comprehensive guide helps you debug and resolve common
issues when using the bashrs REPL.

=================================================================
PROBLEM 1: REPL Won't Start
=================================================================

SYMPTOMS:
  $ bashrs repl
  Failed to initialize REPL: Terminal error

CAUSES:
  1. Terminal doesn't support ANSI escape codes
  2. Incompatible terminal emulator
  3. Missing terminal capabilities

SOLUTIONS:

Solution 1: Check terminal type
  $ echo $TERM
  # Should show: xterm-256color, screen-256color, etc.

  # If not set or wrong, set it:
  $ export TERM=xterm-256color
  $ bashrs repl

Solution 2: Test ANSI support
  $ echo -e "\e[1mBold\e[0m \e[32mGreen\e[0m"
  # Should show "Bold" in bold and "Green" in green

  # If not working, try a different terminal:
  - iTerm2 (macOS)
  - GNOME Terminal (Linux)
  - Windows Terminal (Windows)
  - Alacritty (cross-platform)

Solution 3: Update bashrs
  $ cargo install bashrs --force
  $ bashrs repl

=================================================================
PROBLEM 2: History Not Persisting
=================================================================

SYMPTOMS:
  - Commands not saved across sessions
  - Up arrow doesn't recall previous commands
  - :history shows "No commands in history"

CAUSES:
  1. History file permission issues
  2. Disk full
  3. History file corrupted

SOLUTIONS:

Solution 1: Check history file permissions
  $ ls -la ~/.bashrs_history
  # Should be readable/writable by your user

  # Fix permissions:
  $ chmod 600 ~/.bashrs_history

Solution 2: Check disk space
  $ df -h ~
  # Ensure sufficient free space

Solution 3: Recreate history file
  $ rm ~/.bashrs_history
  $ touch ~/.bashrs_history
  $ chmod 600 ~/.bashrs_history

Solution 4: Check if history is being written
  $ bashrs repl
  bashrs [normal]> echo test
  bashrs [normal]> quit

  $ cat ~/.bashrs_history
  # Should contain "echo test"

=================================================================
PROBLEM 3: Tab Completion Not Working
=================================================================

SYMPTOMS:
  - Pressing Tab does nothing
  - No command/file completion
  - Tab key produces literal tab character

CAUSES:
  1. Terminal doesn't support tab completion
  2. Key binding conflict
  3. REPL version too old

SOLUTIONS:

Solution 1: Test tab key
  $ bashrs repl
  bashrs [normal]> :mo<TAB>
  # Should complete to :mode

  # If not working, try Ctrl-I (alternate for Tab)
  bashrs [normal]> :mo<Ctrl-I>

Solution 2: Check terminal settings
  - Ensure Tab key is not remapped
  - Check terminal preferences/settings

Solution 3: Update bashrs
  $ cargo install bashrs --force

=================================================================
PROBLEM 4: Multiline Input Stuck
=================================================================

SYMPTOMS:
  - REPL stuck showing ... > prompt
  - Can't execute or cancel input
  - Prompt won't return to normal

CAUSES:
  1. Unclosed quotes
  2. Unclosed braces/brackets
  3. Incomplete bash construct

SOLUTIONS:

Solution 1: Cancel with Ctrl-C
  bashrs [normal]> for i in 1 2 3; do
  ... >   echo "test"
  ... > ^C (multi-line input cancelled)
  bashrs [normal]>

Solution 2: Complete the construct
  bashrs [normal]> if [ -f file ]; then
  ... >   echo "found"
  ... > fi  # Complete the if statement
  ✓ Executed

Solution 3: Check for unclosed quotes
  bashrs [normal]> echo "hello
  ... > world"  # Close the quote
  hello
  world

Solution 4: Force quit if stuck
  Press Ctrl-D (EOF) to exit REPL
  Then restart: bashrs repl

=================================================================
PROBLEM 5: Out of Memory
=================================================================

SYMPTOMS:
  REPL out of memory
  Process killed
  System becomes slow

CAUSES:
  1. Loading very large scripts
  2. Memory leak
  3. Too many variables/functions

SOLUTIONS:

Solution 1: Increase memory limit
  $ bashrs repl --max-memory 1000  # 1GB

Solution 2: Avoid loading huge scripts
  # Don't load files >1MB
  $ wc -l huge_script.sh
  50000 huge_script.sh  # Too large!

  # Load specific sections instead
  $ head -100 huge_script.sh > section1.sh
  $ bashrs repl
  bashrs [normal]> :load section1.sh

Solution 3: Clear session and restart
  bashrs [normal]> quit
  $ bashrs repl --max-memory 1000

=================================================================
PROBLEM 6: Variables Not Expanding
=================================================================

SYMPTOMS:
  $ echo $var
  # Shows literal "$var" instead of value

CAUSES:
  1. Variable not set
  2. Wrong syntax
  3. Mode issue

SOLUTIONS:

Solution 1: Check if variable is set
  bashrs [normal]> :vars
  # Verify variable exists

  # If not, set it:
  bashrs [normal]> var="value"
  ✓ Variable set: var = value

Solution 2: Test expansion
  bashrs [normal]> echo $var
  value

  bashrs [normal]> echo ${var}
  value

Solution 3: Check for typos
  bashrs [normal]> myvar="test"
  bashrs [normal]> echo $myVar  # Wrong case!
  (empty)  # Variable names are case-sensitive

  bashrs [normal]> echo $myvar  # Correct
  test

=================================================================
PROBLEM 7: Parse Errors
=================================================================

SYMPTOMS:
  ✗ Parse error: Unexpected token
  ✗ Parse error: Unclosed quote
  ✗ Parse error: Invalid syntax

CAUSES:
  1. Bash syntax error
  2. Invalid command
  3. Typo in construct

SOLUTIONS:

Solution 1: Check syntax
  bashrs [normal]> :parse if then fi
  ✗ Parse error: Unexpected token 'then'

  # Fix: Add condition
  bashrs [normal]> :parse if [ -f file ]; then echo found; fi
  ✓ Parse successful!

Solution 2: Use explain mode to learn
  bashrs [normal]> :mode explain
  bashrs [explain]> if [ -f file ]
  📖 If Statement: if condition; then commands; fi
  ...

Solution 3: Test incrementally
  bashrs [normal]> :parse echo test
  ✓ Parse successful!

  bashrs [normal]> :parse echo test && echo success
  ✓ Parse successful!

=================================================================
PROBLEM 8: Linter False Positives
=================================================================

SYMPTOMS:
  - Linter warns about correct code
  - Too many warnings
  - Confusing error messages

CAUSES:
  1. Overly strict rules
  2. Misunderstanding of rules
  3. Context-specific patterns

SOLUTIONS:

Solution 1: Understand the warning
  bashrs [lint]> cat $FILE | grep pattern
  Found 1 issue(s):
  [1] ⚠ SC2086 - Unquoted variable

  # Understanding: Variables should be quoted
  # to prevent word splitting and globbing

Solution 2: Fix the issue
  bashrs [lint]> :mode purify
  bashrs [purify]> cat $FILE | grep pattern
  ✓ Purified: cat "$FILE" | grep "pattern"

Solution 3: Accept the warning if intentional
  # Sometimes you want word splitting:
  bashrs [lint]> echo $OPTIONS
  # OK to leave unquoted if OPTIONS should split

=================================================================
PROBLEM 9: Script Loading Fails
=================================================================

SYMPTOMS:
  ✗ Error: Cannot read file script.sh
  ✗ Parse error when loading script
  ✗ No functions extracted

CAUSES:
  1. File doesn't exist
  2. Permission denied
  3. Invalid bash syntax in script
  4. File path wrong

SOLUTIONS:

Solution 1: Verify file exists
  $ ls -la script.sh
  # Check file exists and is readable

  $ bashrs repl
  bashrs [normal]> :load ./script.sh  # Use ./ for current dir

Solution 2: Check permissions
  $ chmod +r script.sh
  $ bashrs repl
  bashrs [normal]> :load script.sh

Solution 3: Validate syntax first
  $ bash -n script.sh  # Check syntax
  $ # If no errors, then load in REPL

  $ bashrs repl
  bashrs [normal]> :load script.sh

Solution 4: Use absolute path
  bashrs [normal]> :load /full/path/to/script.sh

=================================================================
PROBLEM 10: Purification Unexpected Results
=================================================================

SYMPTOMS:
  - Purified code looks wrong
  - Too many transformations
  - Changed behavior

CAUSES:
  1. Aggressive purification rules
  2. Misunderstanding of idempotency
  3. Edge case in purifier

SOLUTIONS:

Solution 1: Compare before and after
  bashrs [normal]> :mode normal
  bashrs [normal]> mkdir /tmp/test
  # Original command

  bashrs [normal]> :mode purify
  bashrs [purify]> mkdir /tmp/test
  ✓ Purified: mkdir -p "/tmp/test"
  # Idempotent version

  # Understand: -p makes it safe to re-run

Solution 2: Test both versions
  $ mkdir /tmp/test  # Original
  $ mkdir /tmp/test  # Fails if exists!

  $ mkdir -p /tmp/test  # Purified
  $ mkdir -p /tmp/test  # Succeeds even if exists!

Solution 3: Use purification as suggestion
  # Purified version is a suggestion
  # You can choose to use it or not
  # Understand the trade-offs

=================================================================
Debugging Workflow:
=================================================================

When encountering any issue:

1. Identify the problem
   - What's the error message?
   - What were you trying to do?
   - When did it start happening?

2. Simplify the test case
   - Can you reproduce with minimal input?
   - Does it happen with simple commands?

3. Check the basics
   - Is bashrs up to date?
   - Is the terminal compatible?
   - Are permissions correct?

4. Test incrementally
   - Start with simple commands
   - Add complexity gradually
   - Identify where it breaks

5. Consult documentation
   - Read the error message carefully
   - Check :help for command usage
   - Review examples

6. Ask for help
   - File issue on GitHub
   - Include error message
   - Include steps to reproduce

=================================================================
Common Error Messages Explained:
=================================================================

Error: "Failed to initialize REPL"
  → Terminal compatibility issue
  → Solution: Use compatible terminal

Error: "Parse error: Unexpected token"
  → Bash syntax error
  → Solution: Check syntax with :parse

Error: "Cannot read file"
  → File not found or no permission
  → Solution: Check path and permissions

Error: "Out of memory"
  → Memory limit exceeded
  → Solution: Increase limit with --max-memory

Error: "Unknown mode"
  → Invalid mode name
  → Solution: Use :mode to see valid modes

Error: "No script to reload"
  → :reload without prior :load
  → Solution: Use :load first

=================================================================
Prevention Tips:
=================================================================

1. Keep bashrs updated
   $ cargo install bashrs --force

2. Use compatible terminal
   - xterm-256color support
   - ANSI escape codes

3. Test syntax before loading
   $ bash -n script.sh

4. Start simple, add complexity
   - Begin with basic commands
   - Add features incrementally

5. Save work frequently
   - Use :history to review
   - Copy commands to scripts

6. Check :vars regularly
   - Verify variable values
   - Clear unused variables by restarting

7. Use Ctrl-C to recover
   - Cancel multiline input
   - Escape stuck states

8. Monitor resource usage
   - Don't load huge files
   - Use --max-memory if needed

=================================================================
Emergency Recovery:
=================================================================

If REPL is completely stuck:

1. Try Ctrl-C to cancel
2. Try Ctrl-D to exit
3. Force quit terminal
4. Restart bashrs repl
5. Check ~/.bashrs_history for last commands
6. Review what went wrong
7. File issue if bug found

=================================================================
Getting Help:
=================================================================

In the REPL:
  bashrs [normal]> help

For specific commands:
  bashrs [normal]> :parse
  Usage: :parse <bash_code>

Documentation:
  - User Guide: book/src/repl/user-guide.md
  - Tutorial: book/src/repl/tutorial.md
  - Command Reference: book/src/reference/repl-commands.md

File Issues:
  https://github.com/paiml/bashrs/issues

Include:
  - bashrs version: bashrs --version
  - Error message
  - Steps to reproduce
  - Terminal type: echo $TERM

=================================================================
Key Takeaways:
=================================================================

1. Most issues have simple solutions
2. Ctrl-C is your friend (cancel/recovery)
3. Keep bashrs updated
4. Use compatible terminal
5. Test incrementally
6. Check basics first (file exists, permissions, syntax)
7. :help and documentation are helpful
8. File issues for bugs

Remember: The REPL is designed to be resilient. Most issues
can be resolved by understanding the error message and
following the appropriate solution above.

Happy scripting! 🚀
EOF
