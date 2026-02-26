// REPL Help System Module
//
// Task: REPL-015-004 - Enhanced help system with contextual topics
// Test Approach: RED → GREEN → REFACTOR → PROPERTY → MUTATION
//
// Quality targets:
// - Unit tests: 8+ scenarios
// - Coverage: >85%
// - Complexity: <10 per function

/// Show help for the specified topic
///
/// # Arguments
/// * `topic` - Optional help topic. If None, shows general help.
///
/// # Examples
///
/// ```
/// use bashrs::repl::help::show_help;
///
/// // General help
/// let help = show_help(None);
/// assert!(help.contains("bashrs REPL"));
///
/// // Specific topic
/// let help = show_help(Some("purify"));
/// assert!(help.contains("purification"));
/// ```
pub fn show_help(topic: Option<&str>) -> String {
    match topic {
        None => show_general_help(),
        Some("commands" | "command") => show_commands_help(),
        Some("modes" | "mode") => show_modes_help(),
        Some("purify") => show_purify_help(),
        Some("lint") => show_lint_help(),
        Some("parse") => show_parse_help(),
        Some("explain") => show_explain_help(),
        Some("debug") => show_debug_help(),
        Some("history") => show_history_help(),
        Some("variables" | "vars") => show_variables_help(),
        Some("shortcuts" | "keys") => show_shortcuts_help(),
        Some(unknown) => format!(
            "Unknown help topic: '{}'\n\nAvailable topics:\n\
             - commands   - List all REPL commands\n\
             - modes      - Explain REPL modes\n\
             - purify     - Learn about bash purification\n\
             - lint       - Learn about linting\n\
             - parse      - Learn about parsing\n\
             - explain    - Learn about explanations\n\
             - history    - Learn about history features\n\
             - variables  - Learn about session variables\n\
             - shortcuts  - Learn about keyboard shortcuts\n\n\
             Try: :help <topic>",
            unknown
        ),
    }
}

fn show_general_help() -> String {
    format!(
        r"bashrs REPL v{} - Interactive bash purification and debugging

OVERVIEW:
  bashrs REPL provides an interactive environment for:
  • Purifying bash scripts (making them deterministic and idempotent)
  • Linting bash code for common issues
  • Parsing and analyzing bash syntax
  • Explaining bash constructs
  • Debugging bash scripts (planned)

QUICK START:
  1. Try different modes:     :mode purify
  2. Purify some code:         mkdir /tmp/test
  3. Get help on a topic:      :help purify
  4. Load a script:            :load examples/script.sh

GETTING HELP:
  :help commands   - List all available commands
  :help modes      - Learn about REPL modes
  :help purify     - Learn about purification
  :help shortcuts  - Learn keyboard shortcuts

KEYBOARD SHORTCUTS:
  Up/Down arrows   - Navigate command history
  Ctrl-R           - Reverse history search
  Ctrl-C           - Cancel current input / Exit multi-line
  Ctrl-D           - Exit REPL (EOF)
  Tab              - Auto-complete file paths (with :load/:source)

SUPPORT:
  Documentation: https://github.com/paiml/bashrs
  Report issues: https://github.com/paiml/bashrs/issues

Type ':help <topic>' for detailed help on specific topics.
",
        env!("CARGO_PKG_VERSION")
    )
}

fn show_commands_help() -> String {
    r"REPL COMMANDS

MODE SWITCHING:
  :mode                - Show current mode
  :mode <name>         - Switch to mode (normal, purify, lint, explain, debug)

CODE ANALYSIS:
  :parse <code>        - Parse bash code and show AST
  :purify <code>       - Purify bash code (show idempotent version)
  :lint <code>         - Lint bash code and show issues
  :explain <code>      - Explain bash constructs

SCRIPT LOADING:
  :load <file>         - Load bash script and extract functions
  :source <file>       - Source bash script (same as :load)
  :functions           - List all loaded functions
  :reload              - Reload the last loaded script

SESSION MANAGEMENT:
  :history             - Show command history
  :vars                - Show session variables
  :clear               - Clear the screen

HELP & EXIT:
  :help [topic]        - Show help (try :help purify)
  :quit                - Exit REPL
  :exit                - Exit REPL (same as :quit)
  help                 - Show help (no colon needed)
  quit                 - Exit REPL (no colon needed)

EXAMPLES:
  :mode purify         - Switch to purify mode
  :purify mkdir /tmp   - Show purified version (mkdir -p /tmp)
  :lint cat file.txt   - Lint a command
  :load script.sh      - Load functions from script
  :help modes          - Learn about REPL modes

Tip: In purify/lint/explain modes, you don't need to prefix with :purify/:lint/:explain
"
    .to_string()
}

fn show_modes_help() -> String {
    r#"REPL MODES

The bashrs REPL has 5 modes that change how it processes input:

1. NORMAL MODE (default)
   • Executes bash commands directly
   • Shows output from commands
   • Use for: Quick bash command testing
   • Example: echo "Hello, World!"

2. PURIFY MODE
   • Shows purified (safe) version of bash code
   • Makes code deterministic and idempotent
   • Use for: Learning safe bash practices
   • Example: mkdir /tmp/test → mkdir -p /tmp/test

3. LINT MODE
   • Shows linting results for bash code
   • Detects common issues and anti-patterns
   • Use for: Finding problems in bash scripts
   • Example: cat file.txt | grep pattern (UUOC warning)

4. EXPLAIN MODE
   • Explains bash syntax and constructs
   • Provides educational information
   • Use for: Learning bash features
   • Example: ${var:-default} (explains parameter expansion)

5. DEBUG MODE (coming soon)
   • Step-by-step execution with breakpoints
   • Variable inspection
   • Use for: Debugging complex bash scripts

SWITCHING MODES:
  :mode                - Show current mode
  :mode purify         - Switch to purify mode
  :mode lint           - Switch to lint mode
  :mode normal         - Switch to normal mode

MODE TIPS:
  • Modes affect ALL input (not just :commands)
  • In purify mode: "mkdir /tmp" automatically shows purified version
  • In lint mode: Every command is automatically linted
  • Use :mode normal to return to default behavior

Try: :mode purify, then type: mkdir /tmp/test
"#
    .to_string()
}

fn show_purify_help() -> String {
    r#"BASH PURIFICATION

Purification makes bash scripts deterministic and idempotent:

DETERMINISM - Same input → Same output (always)
  • Removes $RANDOM
  • Removes timestamps (date +%s)
  • Removes process IDs ($$)
  • Makes UUIDs deterministic

IDEMPOTENCY - Safe to run multiple times
  • mkdir → mkdir -p (doesn't fail if exists)
  • rm → rm -f (doesn't fail if missing)
  • ln → ln -sf (overwrites existing symlinks)

SAFETY - Prevents injection attacks
  • Quotes all variables: "$var" not $var
  • Escapes special characters
  • Validates file paths

EXAMPLES:

Before purification:
  mkdir /tmp/mydir
  SESSION_ID=$RANDOM
  rm /tmp/oldfile
  ln -s /new /link

After purification:
  mkdir -p /tmp/mydir              # Safe if exists
  SESSION_ID="session_default"     # Deterministic
  rm -f /tmp/oldfile               # Safe if missing
  ln -sf /new /link                # Safe if exists

USAGE:
  :purify <code>       - Purify specific code
  :mode purify         - Auto-purify all input
  :load script.sh      - Load script (shows purified functions)

WHY PURIFY?
  • Production safety: Scripts won't fail randomly
  • CI/CD reliability: Same result every run
  • Security: No injection vulnerabilities
  • Operations: Safe to re-run deployments

Try: :purify mkdir /tmp/test
"#
    .to_string()
}

fn show_lint_help() -> String {
    r"BASH LINTING

The linter detects common bash issues and anti-patterns:

SECURITY ISSUES (SEC-xxx):
  • Unquoted variables (injection risk)
  • Unsafe eval usage
  • Dangerous sudo patterns
  • Insecure temp file creation

DETERMINISM ISSUES (DET-xxx):
  • Use of $RANDOM
  • Timestamp dependencies
  • Process ID usage
  • Non-deterministic UUIDs

IDEMPOTENCY ISSUES (IDEM-xxx):
  • mkdir without -p flag
  • rm without -f flag
  • ln without -f flag
  • Non-idempotent operations

PERFORMANCE ISSUES (PERF-xxx):
  • Useless use of cat (UUOC)
  • Inefficient pipes
  • Subshell overhead

CODE QUALITY (QUAL-xxx):
  • Unused variables
  • Undefined variables
  • Unreachable code
  • Missing error handling

USAGE:
  :lint <code>         - Lint specific code
  :mode lint           - Auto-lint all input
  :lint script.sh      - Lint entire file (planned)

EXAMPLE OUTPUT:
  bashrs [lint]> cat file.txt | grep pattern

  ⚠ PERF-001: Useless Use of Cat (UUOC)
    Suggestion: grep pattern file.txt
    Impact: Performance (unnecessary process)

Try: :lint cat file.txt | grep test
"
    .to_string()
}

fn show_parse_help() -> String {
    r#"BASH PARSING

The parser converts bash code into an Abstract Syntax Tree (AST):

WHAT IS PARSING?
  Parsing analyzes bash syntax and creates a structured representation.
  This AST is used by purify, lint, and explain modes.

WHAT YOU CAN PARSE:
  • Commands: echo hello
  • Pipelines: cat file | grep pattern
  • Control flow: if/for/while/case
  • Functions: function foo() { ... }
  • Variables: x=5, ${var:-default}
  • Redirections: cmd > file, cmd 2>&1

USAGE:
  :parse <code>        - Parse code and show AST
  :parse echo hello    - Parse simple command
  :parse 'if [ -f file ]; then echo yes; fi'

EXAMPLE OUTPUT:
  bashrs> :parse echo hello world

  ✓ Parse successful!
  Statements: 1
  Parse time: 2ms

  AST:
    [0] Command {
      name: "echo",
      args: ["hello", "world"],
      redirects: [],
    }

PARSE ERRORS:
  If parsing fails, you'll see an error with line/column:

  ✗ Parse error at line 1, column 15:
    Unexpected token: expected 'fi', found 'done'

Try: :parse for i in 1 2 3; do echo $i; done
"#
    .to_string()
}

fn show_explain_help() -> String {
    r#"BASH EXPLANATIONS

The explain mode provides detailed explanations of bash constructs:

WHAT GETS EXPLAINED:
  • Parameter expansions: ${var:-default}, ${var#prefix}
  • Control flow: if/then/else, for loops, while loops
  • Redirections: >, >>, 2>&1, <
  • Special variables: $?, $!, $$, $@
  • Test expressions: [ -f file ], [[ string =~ regex ]]

USAGE:
  :explain <code>      - Explain specific construct
  :mode explain        - Auto-explain all input

EXAMPLE OUTPUT:
  bashrs> :explain ${version:-1.0.0}

  📖 Parameter Expansion: Use Default Value

  Syntax: ${parameter:-word}

  Meaning:
    If $version is unset or null, use "1.0.0" instead.
    The original variable is NOT modified.

  Example:
    version=""
    echo ${version:-1.0.0}  # Outputs: 1.0.0
    echo $version           # Outputs: (empty)

  Related:
    ${parameter:=word}  # Assign default if unset
    ${parameter:?error} # Error if unset
    ${parameter:+word}  # Use word if SET

SUPPORTED CONSTRUCTS:
  • ${var:-default}     - Default values
  • ${var:=value}       - Assign defaults
  • ${var#pattern}      - Remove prefix
  • ${var%pattern}      - Remove suffix
  • for i in ...; do    - For loops
  • while [ ... ]; do   - While loops
  • if [ ... ]; then    - Conditionals
  • case $x in          - Case statements

Try: :explain ${HOME:-/tmp}
"#
    .to_string()
}

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
mod tests {
    use super::*;

    // ===== RED PHASE: Write failing tests first =====

    #[test]
    fn test_repl_015_004_general_help_contains_overview() {
        let help = show_help(None);
        assert!(help.contains("bashrs REPL"));
        assert!(help.contains("OVERVIEW"));
        assert!(help.contains("Purifying") || help.contains("purifying"));
    }

    #[test]
    fn test_repl_015_004_commands_help_lists_all_commands() {
        let help = show_help(Some("commands"));
        assert!(help.contains(":mode"));
        assert!(help.contains(":purify"));
        assert!(help.contains(":lint"));
        assert!(help.contains(":load"));
        assert!(help.contains(":help"));
    }

    #[test]
    fn test_repl_015_004_modes_help_explains_modes() {
        let help = show_help(Some("modes"));
        assert!(help.contains("NORMAL MODE"));
        assert!(help.contains("PURIFY MODE"));
        assert!(help.contains("LINT MODE"));
        assert!(help.contains("EXPLAIN MODE"));
        assert!(help.contains("DEBUG MODE"));
    }

    #[test]
    fn test_repl_015_004_purify_help_explains_purification() {
        let help = show_help(Some("purify"));
        assert!(help.contains("DETERMINISM"));
        assert!(help.contains("IDEMPOTENCY"));
        assert!(help.contains("mkdir -p"));
        assert!(help.contains("rm -f"));
    }

    #[test]
    fn test_repl_015_004_unknown_topic_shows_error() {
        let help = show_help(Some("nonexistent"));
        assert!(help.contains("Unknown help topic"));
        assert!(help.contains("nonexistent"));
        assert!(help.contains("Available topics"));
    }

    #[test]
    fn test_repl_015_004_shortcuts_help_lists_keybindings() {
        let help = show_help(Some("shortcuts"));
        assert!(help.contains("Ctrl-R"));
        assert!(help.contains("Up arrow"));
        assert!(help.contains("Ctrl-A"));
        assert!(help.contains("HISTORY NAVIGATION"));
    }

    #[test]
    fn test_repl_015_004_history_help_mentions_ctrl_r() {
        let help = show_help(Some("history"));
        assert!(help.contains("Ctrl-R"));
        assert!(help.contains("Reverse search"));
        assert!(help.contains(".bashrs_history"));
    }

    #[test]
    fn test_repl_015_004_variables_help_explains_expansions() {
        let help = show_help(Some("variables"));
        assert!(help.contains("x=5"));
        assert!(help.contains("${var:-default}"));
        assert!(help.contains(":vars"));
        assert!(help.contains("PARAMETER EXPANSIONS"));
    }

    #[test]
    fn test_repl_015_004_lint_help_categorizes_issues() {
        let help = show_help(Some("lint"));
        assert!(help.contains("SECURITY ISSUES"));
        assert!(help.contains("DETERMINISM ISSUES"));
        assert!(help.contains("IDEMPOTENCY ISSUES"));
        assert!(help.contains("SEC-"));
        assert!(help.contains("DET-"));
    }

    #[test]
    fn test_repl_015_004_parse_help_explains_ast() {
        let help = show_help(Some("parse"));
        assert!(help.contains("Abstract Syntax Tree"));
        assert!(help.contains("AST"));
        assert!(help.contains(":parse"));
    }

    #[test]
    fn test_repl_015_004_explain_help_covers_constructs() {
        let help = show_help(Some("explain"));
        assert!(help.contains("BASH EXPLANATIONS"));
        assert!(help.contains(":explain"));
        assert!(help.contains("Parameter Expansion"));
        assert!(help.contains("${var:-default}"));
        assert!(help.contains("for i in"));
        assert!(help.contains("while"));
        assert!(help.contains("case $x in"));
    }

    #[test]
    fn test_repl_015_004_debug_help() {
        let help = show_help(Some("debug"));
        assert!(help.contains("DEBUGGING"));
    }
}
