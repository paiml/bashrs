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

include!("help_show.rs");
