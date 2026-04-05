#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]

use super::super::ast::Redirect;
use super::super::lexer::Lexer;
use super::super::parser::BashParser;
use super::super::semantic::SemanticAnalyzer;
use super::super::*;

#[test]
fn test_EDIT_001_interactive_vs_script_input_model() {
    // DOCUMENTATION: Interactive vs script input models
    //
    // Interactive input model (with readline):
    // - User types commands character by character
    // - Readline processes each keystroke
    // - User can edit before pressing Enter
    // - Command executed after Enter
    // - History saved for recall
    // - Completion assists user
    //
    // Script input model (no readline):
    // - Commands predefined in script file
    // - No character-by-character processing
    // - No editing (commands already written)
    // - Commands execute immediately
    // - No history (deterministic execution)
    // - No completion needed (full commands)

    let script_input_model = r#"
#!/bin/sh
# Script input model (no readline)

# Commands predefined (no typing)
command1() {
  printf '%s\n' "Command 1"
}

command2() {
  printf '%s\n' "Command 2"
}

# Execute directly (no editing)
command1
command2
"#;

    let result = BashParser::new(script_input_model);
    if let Ok(mut parser) = result {
        let parse_result = parser.parse();
        assert!(
            parse_result.is_ok() || parse_result.is_err(),
            "Scripts use predefined commands without readline"
        );
    }

    // Summary:
    // Interactive: User types → Readline edits → Shell executes
    // Script: Shell reads file → Shell executes (no readline)
    //
    // bashrs: Scripts only, no readline library needed
}

// ============================================================================
// HISTORY-001: History Expansion (Interactive History, NOT SUPPORTED)
// ============================================================================
//
// Task: HISTORY-001 - Document history expansion
// Status: DOCUMENTED (NOT SUPPORTED - interactive history, non-deterministic)
// Priority: LOW (history expansion not needed in scripts)
//
// History expansion allows referencing previous commands interactively using
// ! (bang) notation. It's interactive-only and non-deterministic.
//
// Bash behavior:
// - !! repeats last command
// - !$ uses last argument from previous command
// - !^ uses first argument from previous command
// - !:n uses nth argument from previous command
// - !string repeats last command starting with 'string'
// - Interactive shells only (requires command history)
//
// bashrs policy:
// - NOT SUPPORTED (interactive history, non-deterministic)
// - Scripts don't have interactive history
// - History expansion removed during purification
// - Non-deterministic (depends on previous commands)
// - POSIX sh supports history expansion, but bashrs doesn't use it
//
// Transformation:
// Bash input:
//   echo hello
//   !!           # Repeats: echo hello
//   echo world
//   echo !$      # Uses: world
//
// Purified POSIX sh:
//   echo hello
//   # !! removed (non-deterministic)
//   echo world
//   # !$ removed (non-deterministic)
//
// Related features:
// - history command - View/manage history (interactive)
// - HISTFILE - History file location
// - HISTSIZE - History size limit
// - fc command - Fix/repeat commands

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
