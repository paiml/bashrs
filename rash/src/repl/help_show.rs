fn show_debug_help() -> String {
    r"BASH DEBUGGING (Coming Soon)

Debug mode will provide step-by-step execution with breakpoints:

PLANNED FEATURES:
  • Set breakpoints at specific lines
  • Step through code line-by-line
  • Inspect variables at each step
  • View call stack
  • Compare original vs purified at breakpoint

COMMANDS (Planned):
  :break <line>        - Set breakpoint
  :step                - Execute one line
  :next                - Execute one line (skip over functions)
  :continue            - Run until next breakpoint
  :vars                - Show all variables
  :backtrace           - Show call stack

STATUS:
  Debug mode is currently under development (REPL-007-001 in roadmap).
  Expected in a future release.

WORKAROUNDS (Current):
  • Use explain mode to understand constructs
  • Use purify mode to see safe versions
  • Use lint mode to find issues
  • Use :vars to inspect session variables

For now, try: :mode explain
"
    .to_string()
}

fn show_history_help() -> String {
    format!(
        r#"COMMAND HISTORY

The REPL maintains a persistent history of your commands:

HISTORY FEATURES:
  • Persistent across sessions (~/.bashrs_history)
  • Up to 1000 commands stored
  • Duplicate commands filtered (configurable)
  • Commands starting with space are private (not saved)

KEYBOARD SHORTCUTS:
  Up arrow             - Previous command
  Down arrow           - Next command
  Ctrl-R               - Reverse search (type to search backwards)
  Ctrl-S               - Forward search (type to search forwards)

COMMANDS:
  :history             - Show all commands in history
  :history | grep foo  - Search history (planned)

EXAMPLES:

Browsing history:
  1. Press Up arrow to see previous command
  2. Press Down arrow to move forward
  3. Press Enter to execute

Searching history:
  1. Press Ctrl-R
  2. Type search term (e.g., "purify")
  3. Press Ctrl-R again to cycle through matches
  4. Press Enter to execute, Esc to cancel

Private commands:
  bashrs> echo normal      # Saved to history
  bashrs>  echo private    # NOT saved (note leading space)

CONFIGURATION:
  History settings can be configured in ReplConfig:
  • max_history: {} (default)
  • history_ignore_dups: true (default)
  • history_ignore_space: true (default)

Try: Press Ctrl-R and type "mode"
"#,
        1000
    )
}

fn show_variables_help() -> String {
    r#"SESSION VARIABLES

The REPL maintains session variables that persist during your session:

SETTING VARIABLES:
  x=5                  - Set variable x to 5
  name="bashrs"        - Set string variable
  path=/tmp/test       - Set path variable

USING VARIABLES:
  echo $x              - Print variable value
  echo ${x}            - Same, with braces (preferred)
  echo ${x:-default}   - Use default if unset
  mkdir $path          - Use in commands

VIEWING VARIABLES:
  :vars                - Show all session variables
  :vars | grep x       - Search variables (planned)

VARIABLE EXPANSION:
  Variables are expanded before command execution:

  bashrs> x=hello
  bashrs> echo $x world
  hello world

  bashrs> dir=/tmp/test
  bashrs> mkdir -p $dir
  (creates /tmp/test directory)

SPECIAL VARIABLES (Read-only):
  $?                   - Exit code of last command
  $$                   - Current process ID
  $!                   - PID of last background job
  $@                   - All positional parameters
  $#                   - Number of positional parameters

PARAMETER EXPANSIONS:
  ${var:-default}      - Use default if unset
  ${var:=value}        - Assign and use default
  ${var#prefix}        - Remove shortest prefix
  ${var##prefix}       - Remove longest prefix
  ${var%suffix}        - Remove shortest suffix
  ${var%%suffix}       - Remove longest suffix

EXAMPLES:
  bashrs> version=1.0.0
  bashrs> echo ${version}
  1.0.0

  bashrs> echo ${version:-unknown}
  1.0.0

  bashrs> :vars
  Session Variables (1 variables):
    version = 1.0.0

Try: x=42, then: echo $x
"#
    .to_string()
}

fn show_shortcuts_help() -> String {
    r#"KEYBOARD SHORTCUTS

HISTORY NAVIGATION:
  Up arrow             - Previous command in history
  Down arrow           - Next command in history
  Ctrl-R               - Reverse search history (start typing to search)
  Ctrl-S               - Forward search history

LINE EDITING:
  Ctrl-A               - Move to beginning of line
  Ctrl-E               - Move to end of line
  Ctrl-B               - Move back one character (same as Left arrow)
  Ctrl-F               - Move forward one character (same as Right arrow)
  Alt-B                - Move back one word
  Alt-F                - Move forward one word

DELETING TEXT:
  Ctrl-H               - Delete character before cursor (same as Backspace)
  Ctrl-D               - Delete character under cursor (or EOF if line empty)
  Ctrl-K               - Delete from cursor to end of line
  Ctrl-U               - Delete from cursor to beginning of line
  Ctrl-W               - Delete word before cursor
  Alt-D                - Delete word after cursor

MULTI-LINE INPUT:
  (open quote/brace)   - Automatically enter multi-line mode
  ... >                - Continuation prompt
  Ctrl-C               - Cancel multi-line input (return to normal)
  Ctrl-D               - Submit multi-line input (if valid)

COMPLETION:
  Tab                  - Auto-complete file paths (with :load/:source)
  Tab Tab              - Show all completions

CONTROL:
  Ctrl-C               - Interrupt current input / Cancel multi-line
  Ctrl-D               - Exit REPL (if line is empty)
  Ctrl-L               - Clear screen (same as :clear)

EXAMPLES:

Editing a long command:
  1. Type: echo this is a very long command
  2. Press Ctrl-A to jump to beginning
  3. Press Ctrl-K to delete everything
  4. Type new command

Searching history:
  1. Press Ctrl-R
  2. Type "purify"
  3. Press Ctrl-R again to cycle through matches
  4. Press Enter to execute

Multi-line function:
  1. Type: function test() {
  2. Notice prompt changes to "... >"
  3. Type:   echo "hello"
  4. Type: }
  5. Press Enter to submit

These shortcuts are provided by the rustyline library and follow
standard GNU Readline keybindings (like bash, python REPL, etc.).
"#
    .to_string()
}

#[cfg(test)]
#[path = "help_tests_extracted.rs"]
mod tests_extracted;
