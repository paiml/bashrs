#!/bin/bash
# REPL Example 04: Explain Mode for Learning
# Demonstrates using explain mode to learn bash constructs
#
# This example shows:
# - Parameter expansions
# - Control flow constructs
# - Redirection operators
# - Common bash patterns
#
# Run interactively in: bashrs repl

cat << 'EOF'
=================================================================
REPL Example 04: Explain Mode for Learning Bash
=================================================================

This example demonstrates how to use explain mode to learn bash
constructs interactively.

STEP 1: Start in explain mode
------------------------------
$ bashrs repl
bashrs [normal]> :mode explain
Switched to explain mode - Explain bash constructs and syntax

STEP 2: Learn parameter expansions
-----------------------------------
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
  $ echo "$var"               # Now set to: fallback

bashrs [explain]> ${var:?error message}
📖 Parameter Expansion: ${parameter:?word}
   Error if Null or Unset

If parameter is unset or null, print 'word' to stderr and exit.
Useful for required variables.

Example:
  $ unset CONFIG
  $ echo "${CONFIG:?Error: CONFIG not set}"
  bash: CONFIG: Error: CONFIG not set

bashrs [explain]> ${var:+alternate}
📖 Parameter Expansion: ${parameter:+word}
   Use Alternate Value

If parameter is set and not null, expand to 'word'.
Otherwise expand to empty string.

Example:
  $ var="value"
  $ echo "${var:+set}"  # Outputs: set
  $ unset var
  $ echo "${var:+set}"  # Outputs: (empty)

bashrs [explain]> ${#var}
📖 Parameter Expansion: ${#parameter}
   String Length

Expands to the length of parameter's value in characters.

Example:
  $ var="hello"
  $ echo "${#var}"  # Outputs: 5

STEP 3: Learn control flow
---------------------------
bashrs [explain]> for i in *.txt
📖 For Loop: for name in words
   Iterate Over List

Loop variable 'name' takes each value from the word list.
Executes commands for each iteration.

Example:
  for file in *.txt; do
    echo "Processing: $file"
    wc -l "$file"
  done

bashrs [explain]> if [ -f file ]
📖 If Statement: if condition; then commands; fi
   Conditional Execution

Execute commands only if condition succeeds (exit status 0).
Optional elif and else clauses for alternatives.

Example:
  if [ -f config.txt ]; then
    echo "Config found"
  elif [ -f config.yaml ]; then
    echo "YAML config found"
  else
    echo "No config found"
  fi

bashrs [explain]> while true
📖 While Loop: while condition; do commands; done
   Conditional Loop

Execute commands repeatedly while condition succeeds.
Checks condition before each iteration.

Example:
  counter=0
  while [ $counter -lt 5 ]; do
    echo "Count: $counter"
    counter=$((counter + 1))
  done

bashrs [explain]> case $var in
📖 Case Statement: case word in patterns) commands;; esac
   Pattern Matching

Match 'word' against patterns and execute corresponding commands.
Patterns can include wildcards.

Example:
  case "$1" in
    start)
      echo "Starting..."
      ;;
    stop)
      echo "Stopping..."
      ;;
    *)
      echo "Usage: $0 {start|stop}"
      ;;
  esac

STEP 4: Learn redirection
--------------------------
bashrs [explain]> echo test > file
📖 Output Redirection: command > file
   Redirect Standard Output

Redirects stdout to a file, overwriting existing content.
Use >> to append instead.

Example:
  echo "Log entry" > app.log      # Overwrite
  echo "Another entry" >> app.log # Append

bashrs [explain]> cat < file
📖 Input Redirection: command < file
   Redirect Standard Input

Redirects stdin to read from a file instead of keyboard.

Example:
  while read line; do
    echo "Line: $line"
  done < input.txt

bashrs [explain]> cat file | grep pattern
📖 Pipe: command1 | command2
   Connect Commands

Redirects stdout of command1 to stdin of command2.
Enables chaining multiple commands together.

Example:
  cat access.log | grep ERROR | wc -l

bashrs [explain]> command 2>&1
📖 Redirection: 2>&1
   Redirect stderr to stdout

Sends error messages (file descriptor 2) to stdout (fd 1).
Useful for capturing all output.

Example:
  make build 2>&1 | tee build.log

STEP 5: Learn string operations
--------------------------------
bashrs [explain]> ${var#prefix}
📖 String Operation: ${parameter#pattern}
   Remove Shortest Match from Beginning

Removes shortest match of pattern from the beginning.
Use ## for longest match.

Example:
  $ file="dir/subdir/file.txt"
  $ echo "${file#*/}"     # Outputs: subdir/file.txt
  $ echo "${file##*/}"    # Outputs: file.txt

bashrs [explain]> ${var%suffix}
📖 String Operation: ${parameter%pattern}
   Remove Shortest Match from End

Removes shortest match of pattern from the end.
Use %% for longest match.

Example:
  $ file="archive.tar.gz"
  $ echo "${file%.*}"     # Outputs: archive.tar
  $ echo "${file%%.*}"    # Outputs: archive

bashrs [explain]> ${var/pattern/replacement}
📖 String Operation: ${parameter/pattern/replacement}
   Replace First Match

Replaces first occurrence of pattern with replacement.
Use // to replace all occurrences.

Example:
  $ path="/usr/local/bin"
  $ echo "${path/local/share}"   # Outputs: /usr/share/bin
  $ echo "${path//\//\\}"        # Outputs: \usr\local\bin

=================================================================
Learning Workflow with Explain Mode:
=================================================================

1. Start in explain mode
2. Type bash constructs you want to understand
3. Read the explanations and examples
4. Switch to normal mode to try them
5. Switch to lint mode to check for issues
6. Switch to purify mode to see safe versions

Example learning session:

  # Learn about parameter expansion
  bashrs [explain]> ${var:-default}
  (explanation shown)

  # Try it in normal mode
  bashrs [explain]> :mode normal
  bashrs [normal]> echo ${missing:-"fallback"}
  fallback

  # Test with set variable
  bashrs [normal]> var="value"
  bashrs [normal]> echo ${var:-"fallback"}
  value

  # Check for issues
  bashrs [normal]> :lint echo ${var:-fallback}
  ✓ No issues found!

=================================================================
Common Bash Patterns Explained:
=================================================================

1. Default Values
   ${VAR:-default}     - Use default if unset
   ${VAR:=default}     - Assign default if unset

2. Required Variables
   ${VAR:?error}       - Error if unset

3. Conditional Values
   ${VAR:+alternate}   - Use alternate if set

4. String Length
   ${#VAR}             - Get length

5. Substring Removal
   ${VAR#prefix}       - Remove prefix (shortest)
   ${VAR##prefix}      - Remove prefix (longest)
   ${VAR%suffix}       - Remove suffix (shortest)
   ${VAR%%suffix}      - Remove suffix (longest)

6. String Replacement
   ${VAR/old/new}      - Replace first
   ${VAR//old/new}     - Replace all

7. Array Operations
   ${array[@]}         - All elements
   ${#array[@]}        - Array length
   ${array[0]}         - First element

=================================================================
Practice Exercises:
=================================================================

Try explaining these constructs:

1. ${file%.txt}
2. ${path##*/}
3. ${var//old/new}
4. for file in *.log; do ... done
5. if [[ -f $file ]]; then ... fi
6. while read line; do ... done < file
7. case $option in ... esac
8. command 2> /dev/null
9. { command1; command2; } | command3
10. $(command)

=================================================================
Key Takeaways:
=================================================================

1. Explain mode provides interactive bash learning
2. Each construct includes syntax, explanation, and examples
3. Combine with normal mode to try constructs immediately
4. Use lint mode to verify you're using them correctly
5. Parameter expansions are powerful and safe alternatives to sed/awk

Next Steps:
-----------
Try example 05_script_loading.sh to learn about loading and
analyzing complete bash scripts!
EOF
