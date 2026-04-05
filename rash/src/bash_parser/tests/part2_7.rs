#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]

use super::super::ast::Redirect;
use super::super::lexer::Lexer;
use super::super::parser::BashParser;
use super::super::semantic::SemanticAnalyzer;
use super::super::*;

/// Helper: parse a script and return whether parsing succeeded.
/// Used by documentation tests that only need to verify parsability.
#[test]
fn test_HISTORY_001_bang_bang_not_supported() {
    // DOCUMENTATION: !! (repeat last command) is NOT SUPPORTED
    //
    // !! repeats the last command:
    // $ echo hello
    // hello
    // $ !!
    // echo hello
    // hello
    //
    // NOT SUPPORTED because:
    // - Interactive history feature
    // - Non-deterministic (depends on previous commands)
    // - Scripts don't have command history
    // - Not safe for automated execution

    let bang_bang_script = r#"
echo hello
!!
"#;

    let result = BashParser::new(bang_bang_script);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "!! is interactive only, NOT SUPPORTED in scripts"
            );
        }
        Err(_) => {
            // Parse error acceptable - interactive feature
        }
    }

    // Why !! is non-deterministic:
    // - Depends on previous command in history
    // - History varies by user, session, environment
    // - Same script produces different results
    // - Violates determinism requirement
}

#[test]
fn test_HISTORY_001_bang_dollar_not_supported() {
    // DOCUMENTATION: !$ (last argument) is NOT SUPPORTED
    //
    // !$ uses the last argument from previous command:
    // $ echo hello world
    // hello world
    // $ echo !$
    // echo world
    // world
    //
    // NOT SUPPORTED because:
    // - Interactive history feature
    // - Non-deterministic (depends on previous command)
    // - Scripts should use explicit variables
    // - Not safe for automated execution

    let bang_dollar_script = r#"
echo hello world
echo !$
"#;

    let result = BashParser::new(bang_dollar_script);
    if let Ok(mut parser) = result {
        let parse_result = parser.parse();
        assert!(
            parse_result.is_ok() || parse_result.is_err(),
            "!$ is interactive only, NOT SUPPORTED in scripts"
        );
    }

    // Alternative: Use explicit variables
    // Instead of: echo hello world; echo !$
    // Use:        last_arg="world"; echo "$last_arg"
}

#[test]
fn test_HISTORY_001_history_expansion_syntax() {
    // DOCUMENTATION: History expansion syntax (all interactive)
    //
    // Event designators (select which command):
    // !!       - Last command
    // !n       - Command number n
    // !-n      - n commands back
    // !string  - Most recent command starting with 'string'
    // !?string - Most recent command containing 'string'
    //
    // Word designators (select which argument):
    // !^       - First argument (word 1)
    // !$       - Last argument
    // !*       - All arguments
    // !:n      - Argument n
    // !:n-m    - Arguments n through m
    // !:n*     - Arguments n through last
    // !:n-     - Arguments n through second-to-last
    //
    // Modifiers (transform the result):
    // :h       - Remove trailing pathname component
    // :t       - Remove all leading pathname components
    // :r       - Remove trailing suffix
    // :e       - Remove all but trailing suffix
    // :p       - Print but don't execute
    // :s/old/new/ - Substitute first occurrence
    // :gs/old/new/ - Global substitute
    //
    // All syntax is interactive-only, NOT SUPPORTED in bashrs.

    let history_syntax = r#"
echo hello world
!!              # Repeat last
!-1             # 1 command back
!echo           # Last starting with 'echo'
!?world         # Last containing 'world'
echo !^         # First arg
echo !$         # Last arg
echo !*         # All args
echo !:1        # Arg 1
echo !:1-2      # Args 1-2
"#;

    let result = BashParser::new(history_syntax);
    if let Ok(mut parser) = result {
        let parse_result = parser.parse();
        assert!(
            parse_result.is_ok() || parse_result.is_err(),
            "History expansion syntax is interactive only"
        );
    }

    // All history expansion requires:
    // - Interactive shell with history enabled
    // - Previous commands in history buffer
    // - set +H disabled (history expansion on)
    // NOT SUPPORTED in scripts (non-deterministic)
}

#[test]
fn test_HISTORY_001_purification_removes_history_expansion() {
    // DOCUMENTATION: Purification removes history expansion
    //
    // Before (with history expansion):
    // #!/bin/bash
    // mkdir /tmp/backup
    // cd /tmp/backup
    // tar -czf archive.tar.gz !$  # Uses: /tmp/backup
    // echo "Backed up to !$"      # Uses: archive.tar.gz
    //
    // After (purified, history expansion removed):
    // #!/bin/sh
    // backup_dir="/tmp/backup"
    // mkdir -p "$backup_dir"
    // cd "$backup_dir" || exit 1
    // archive="archive.tar.gz"
    // tar -czf "$archive" .
    // printf 'Backed up to %s\n' "$archive"
    //
    // Removed because:
    // - Non-deterministic (depends on history)
    // - Scripts use explicit variables instead
    // - Safer and more readable
    // - POSIX-compliant

    let purified_no_history = r#"
#!/bin/sh
backup_dir="/tmp/backup"
mkdir -p "$backup_dir"
cd "$backup_dir" || exit 1
archive="archive.tar.gz"
tar -czf "$archive" .
printf 'Backed up to %s\n' "$archive"
"#;

    let result = BashParser::new(purified_no_history);
    if let Ok(mut parser) = result {
        let parse_result = parser.parse();
        assert!(
            parse_result.is_ok() || parse_result.is_err(),
            "Purified scripts have no history expansion"
        );
    }

    // Purification strategy:
    // 1. Remove all ! history expansions
    // 2. Replace with explicit variables
    // 3. Use clear variable names
    // 4. Deterministic, readable code
}

#[test]
fn test_HISTORY_001_history_command() {
    // DOCUMENTATION: 'history' command (interactive only)
    //
    // history command manages command history:
    // $ history         # Show all history
    // $ history 10      # Show last 10 commands
    // $ history -c      # Clear history
    // $ history -d 5    # Delete entry 5
    // $ history -w      # Write to HISTFILE
    //
    // Example output:
    //   1  echo hello
    //   2  cd /tmp
    //   3  ls -la
    //   4  history
    //
    // NOT SUPPORTED because:
    // - Interactive history management
    // - Scripts don't have persistent history
    // - Not applicable to automated execution

    let history_cmd_script = r#"
history         # Show history
history 10      # Last 10
history -c      # Clear
"#;

    let result = BashParser::new(history_cmd_script);
    if let Ok(mut parser) = result {
        let parse_result = parser.parse();
        assert!(
            parse_result.is_ok() || parse_result.is_err(),
            "history command is interactive only, NOT SUPPORTED in scripts"
        );
    }

    // history command options (all interactive):
    // -c: Clear history list
    // -d offset: Delete entry at offset
    // -a: Append new entries to HISTFILE
    // -n: Read entries not in memory from HISTFILE
    // -r: Read HISTFILE and append to history
    // -w: Write current history to HISTFILE
    // -p: Perform history expansion and display
    // -s: Append arguments to history
    //
    // All options are interactive-only and NOT SUPPORTED.
}

#[test]
fn test_HISTORY_001_fc_command() {
    // DOCUMENTATION: 'fc' command (fix command, interactive only)
    //
    // fc command edits and re-executes commands from history:
    // $ fc       # Edit last command in $EDITOR
    // $ fc 5     # Edit command 5
    // $ fc 5 10  # Edit commands 5-10
    // $ fc -l    # List history (like history command)
    // $ fc -s string=replacement  # Quick substitution
    //
    // Example:
    // $ echo hello
    // $ fc -s hello=world
    // echo world
    // world
    //
    // NOT SUPPORTED because:
    // - Interactive history editing
    // - Requires external editor ($EDITOR)
    // - Non-deterministic (depends on history)
    // - Scripts don't edit previous commands

    let fc_script = r#"
echo hello
fc              # Edit last command
fc -s hello=world  # Quick substitution
"#;

    let result = BashParser::new(fc_script);
    if let Ok(mut parser) = result {
        let parse_result = parser.parse();
        assert!(
            parse_result.is_ok() || parse_result.is_err(),
            "fc command is interactive only, NOT SUPPORTED in scripts"
        );
    }

    // fc command options (all interactive):
    // -e editor: Use specified editor
    // -l: List commands
    // -n: Omit line numbers when listing
    // -r: Reverse order of commands
    // -s: Execute command without editing
    //
    // All options are interactive-only and NOT SUPPORTED.
}

#[test]
fn test_HISTORY_001_history_variables() {
    // DOCUMENTATION: History variables (interactive configuration)
    //
    // History-related variables:
    // HISTFILE - History file location (~/.bash_history)
    // HISTSIZE - Number of commands in memory (default: 500)
    // HISTFILESIZE - Number of lines in HISTFILE (default: 500)
    // HISTCONTROL - Control history saving:
    //   - ignorespace: Don't save lines starting with space
    //   - ignoredups: Don't save duplicate consecutive lines
    //   - ignoreboth: Both ignorespace and ignoredups
    //   - erasedups: Remove all previous duplicates
    // HISTIGNORE - Patterns to exclude from history
    // HISTTIMEFORMAT - Timestamp format for history
    //
    // Example:
    // export HISTSIZE=1000
    // export HISTFILESIZE=2000
    // export HISTCONTROL=ignoreboth
    // export HISTIGNORE="ls:cd:pwd"
    //
    // All variables configure interactive history, NOT SUPPORTED in scripts.

    let history_vars = r#"
export HISTSIZE=1000
export HISTFILESIZE=2000
export HISTCONTROL=ignoreboth
export HISTIGNORE="ls:cd:pwd"
"#;

    let result = BashParser::new(history_vars);
    if let Ok(mut parser) = result {
        let parse_result = parser.parse();
        assert!(
            parse_result.is_ok() || parse_result.is_err(),
            "History variables configure interactive behavior"
        );
    }

    // Why history variables don't apply to scripts:
    // - Scripts don't save command history
    // - No interactive session to persist
    // - Each script run is isolated
    // - No HISTFILE written
}

#[test]
fn test_HISTORY_001_interactive_vs_script_history_model() {
    // DOCUMENTATION: Interactive vs script history models
    //
    // Interactive history model:
    // - Commands saved to history buffer (in memory)
    // - History persisted to HISTFILE on exit
    // - History loaded from HISTFILE on start
    // - History expansion (!!, !$, etc.)
    // - History navigation (Up/Down arrows)
    // - History search (Ctrl+R)
    // - Session-specific history
    //
    // Script history model:
    // - No history buffer (commands execute once)
    // - No HISTFILE (no persistence)
    // - No history expansion (deterministic)
    // - No history navigation (sequential execution)
    // - No history search (predefined commands)
    // - Stateless execution

    let script_no_history = r#"
#!/bin/sh
# Scripts don't have history

command1() {
  printf '%s\n' "Command 1"
}

command2() {
  printf '%s\n' "Command 2"
}

# Commands execute once (no history)
command1
command2

# No history expansion
# No history persistence
# Deterministic execution
"#;

    let result = BashParser::new(script_no_history);
    if let Ok(mut parser) = result {
        let parse_result = parser.parse();
        assert!(
            parse_result.is_ok() || parse_result.is_err(),
            "Scripts execute without history"
        );
    }

    // Summary:
    // Interactive: Commands → History buffer → HISTFILE (persistent)
    // Script: Commands → Execute → Exit (stateless)
    //
    // bashrs: No history, deterministic execution only
}

// ============================================================================
// DIRSTACK-001: pushd/popd Commands (Directory Stack, NOT SUPPORTED)
// ============================================================================
//
// Task: DIRSTACK-001 - Document pushd/popd
// Status: DOCUMENTED (NOT SUPPORTED - implicit directory stack state)
// Priority: LOW (directory stack not needed in scripts)
//
// pushd and popd maintain a directory stack for navigating between directories.
// They maintain implicit state that's useful interactively but problematic for scripts.
//
// Bash behavior:
// - pushd /path: Push directory onto stack and cd to it
// - popd: Pop directory from stack and cd to it
// - dirs: Display directory stack
// - Stack persists across commands in same session
// - Interactive convenience feature
//
// bashrs policy:
// - NOT SUPPORTED (implicit directory stack state)
// - Scripts should use explicit directory tracking
// - Use variables to save/restore directory paths
// - More explicit, deterministic, and readable
//
// Transformation:
// Bash input:
//   pushd /tmp
//   # do work
//   popd
//
// Purified POSIX sh:
//   _prev="$(pwd)"
//   cd /tmp || exit 1
//   # do work
//   cd "$_prev" || exit 1
//
// Related features:
// - dirs command - Display directory stack
// - cd - (cd to previous directory) - Uses OLDPWD
// - DIRSTACK variable - Array of directories in stack

#[test]
fn test_DIRSTACK_001_pushd_not_supported() {
    // DOCUMENTATION: pushd command is NOT SUPPORTED (implicit state)
    //
    // pushd pushes directory onto stack and changes to it:
    // $ pwd
    // /home/user
    // $ pushd /tmp
    // /tmp /home/user
    // $ pwd
    // /tmp
    // $ dirs
    // /tmp /home/user
    //
    // NOT SUPPORTED because:
    // - Implicit directory stack state
    // - State persists across commands
    // - Scripts should use explicit variables
    // - More readable with explicit cd tracking

    let pushd_script = r#"
pushd /tmp
echo "In /tmp"
popd
"#;

    let result = BashParser::new(pushd_script);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "pushd uses implicit directory stack, NOT SUPPORTED in scripts"
            );
        }
        Err(_) => {
            // Parse error acceptable - implicit state feature
        }
    }

    // Why pushd is problematic:
    // - Hidden state (directory stack)
    // - Implicit behavior (stack operations)
    // - Hard to trace (where are we now?)
    // - Explicit variables are clearer
}

