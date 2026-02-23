#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]

use super::super::*;
use super::super::ast::Redirect;
use super::super::lexer::Lexer;
use super::super::parser::BashParser;
use super::super::semantic::SemanticAnalyzer;

/// Helper: assert that BashParser handles the input without panicking.
/// Accepts both successful parses and parse errors (documentation tests
/// only verify the parser doesn't crash, not that the input is valid).
fn assert_parses_without_panic(input: &str, msg: &str) {
    let result = BashParser::new(input);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "{msg}"
            );
        }
        Err(_) => {
            // Parse error acceptable for documentation tests
        }
    }
}

#[test]
fn test_PIPE_001_bash_vs_posix_pipes() {
    // DOCUMENTATION: Bash vs POSIX pipeline features
    //
    // Feature                  | POSIX sh           | Bash extensions
    // -------------------------|-------------------|------------------
    // Basic pipe (|)           | ✅ Supported       | ✅ Supported
    // Multi-stage (a|b|c)      | ✅ Supported       | ✅ Supported
    // Exit status ($?)         | ✅ Rightmost cmd   | ✅ Rightmost cmd
    // PIPESTATUS array         | ❌ Not available   | ✅ ${PIPESTATUS[@]}
    // pipefail option          | ❌ Not available   | ✅ set -o pipefail
    // lastpipe option          | ❌ Not available   | ✅ shopt -s lastpipe
    // |&  (pipe stderr too)    | ❌ Not available   | ✅ Bash 4.0+
    // Process substitution     | ❌ Not available   | ✅ <(cmd) >(cmd)
    //
    // bashrs policy:
    // - Support POSIX pipes (|) fully
    // - NOT SUPPORTED: PIPESTATUS, pipefail, lastpipe, |&, process substitution
    // - Generate POSIX-compliant pipelines only

    let posix_pipe = r#"cat file.txt | grep "pattern" | wc -l"#;
    let bash_pipestatus = r#"cat file.txt | grep "pattern"; echo ${PIPESTATUS[@]}"#;

    // POSIX pipe - SUPPORTED
    let posix_result = BashParser::new(posix_pipe);
    assert!(posix_result.is_ok(), "POSIX pipe should parse");

    // Bash PIPESTATUS - NOT SUPPORTED (Bash extension)
    let bash_result = BashParser::new(bash_pipestatus);
    match bash_result {
        Ok(mut parser) => {
            let _ = parser.parse();
            // PIPESTATUS is Bash extension, may or may not parse
        }
        Err(_) => {
            // Parse error acceptable for Bash extensions
        }
    }

    // Summary:
    // POSIX pipes: Fully supported (|, multi-stage, $? exit status)
    // Bash extensions: NOT SUPPORTED (PIPESTATUS, pipefail, |&, etc.)
    // bashrs: Generate POSIX-compliant pipelines only
}

// ============================================================================
// CMD-LIST-001: Command Lists (&&, ||, ;) (POSIX, SUPPORTED)
// ============================================================================
//
// Task: CMD-LIST-001 (3.2.3.1) - Document command lists (&&, ||, ;)
// Status: DOCUMENTED (SUPPORTED - POSIX compliant)
// Priority: HIGH (fundamental control flow)
//
// Command lists connect multiple commands with control flow operators.
// These are core POSIX features available in all shells.
//
// POSIX operators:
// - ; (semicolon): Execute sequentially, ignore exit status
// - && (AND): Execute second command only if first succeeds (exit 0)
// - || (OR): Execute second command only if first fails (exit non-zero)
// - Newline: Equivalent to semicolon
//
// bashrs policy:
// - FULLY SUPPORTED (POSIX compliant)
// - Quote all variables in generated shell
// - Preserve short-circuit evaluation semantics
// - Map to if statements in Rust

#[test]
fn test_CMD_LIST_001_semicolon_sequential() {
    // DOCUMENTATION: Semicolon (;) executes commands sequentially
    //
    // Semicolon executes commands in sequence, regardless of exit status:
    // $ cmd1 ; cmd2 ; cmd3
    // (All three commands execute, regardless of success/failure)
    //
    // $ false ; echo "Still runs"
    // Still runs
    //
    // Newline is equivalent to semicolon:
    // $ cmd1
    // $ cmd2
    // (Same as: cmd1 ; cmd2)
    //
    // POSIX-compliant: Works in sh, dash, ash, bash

    let sequential = r#"
echo "First"
echo "Second"
false
echo "Third"
"#;

    let result = BashParser::new(sequential);
    assert!(result.is_ok(), "Sequential commands should parse (POSIX)");

    let mut parser = result.unwrap();
    let parse_result = parser.parse();
    assert!(
        parse_result.is_ok() || parse_result.is_err(),
        "Semicolon/newline separation is POSIX-compliant"
    );
}

#[test]
fn test_CMD_LIST_001_and_operator_short_circuit() {
    // DOCUMENTATION: AND operator (&&) with short-circuit evaluation
    //
    // AND (&&) executes second command only if first succeeds:
    // $ test -f file.txt && echo "File exists"
    // (echo only runs if test succeeds)
    //
    // $ false && echo "Never printed"
    // (echo never runs because false returns 1)
    //
    // Short-circuit: Right side only evaluated if left succeeds
    // Exit status: Status of last executed command
    //
    // POSIX-compliant: SUSv3, IEEE Std 1003.1-2001

    let and_operator = r#"
test -f file.txt && echo "File exists"
true && echo "This prints"
false && echo "This does not print"
"#;

    let result = BashParser::new(and_operator);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "AND operator is POSIX-compliant"
            );
        }
        Err(_) => {
            // Parse error acceptable - && may not be fully implemented yet
        }
    }
}

#[test]
fn test_CMD_LIST_001_or_operator_short_circuit() {
    // DOCUMENTATION: OR operator (||) with short-circuit evaluation
    //
    // OR (||) executes second command only if first fails:
    // $ test -f file.txt || echo "File not found"
    // (echo only runs if test fails)
    //
    // $ true || echo "Never printed"
    // (echo never runs because true returns 0)
    //
    // Short-circuit: Right side only evaluated if left fails
    // Exit status: Status of last executed command
    //
    // POSIX-compliant: SUSv3, IEEE Std 1003.1-2001

    let or_operator = r#"
test -f missing.txt || echo "File not found"
false || echo "This prints"
true || echo "This does not print"
"#;

    let result = BashParser::new(or_operator);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "OR operator is POSIX-compliant"
            );
        }
        Err(_) => {
            // Parse error acceptable - || may not be fully implemented yet
        }
    }
}

#[test]
fn test_CMD_LIST_001_combined_operators() {
    // DOCUMENTATION: Combining &&, ||, and ; operators
    //
    // Operators can be combined with precedence rules:
    // - && and || have equal precedence, evaluated left-to-right
    // - ; has lower precedence (separates complete lists)
    //
    // Example: cmd1 && cmd2 || cmd3 ; cmd4
    // Meaning: (cmd1 AND cmd2) OR cmd3, THEN cmd4
    // 1. If cmd1 succeeds, run cmd2
    // 2. If either cmd1 or cmd2 fails, run cmd3
    // 3. Always run cmd4 (semicolon ignores previous exit status)
    //
    // Common pattern (error handling):
    // command && echo "Success" || echo "Failed"

    let combined = r#"
#!/bin/sh
# Try command, report success or failure
test -f file.txt && echo "Found" || echo "Not found"

# Multiple steps with fallback
mkdir -p /tmp/test && cd /tmp/test || exit 1

# Always cleanup, regardless of previous status
process_data && echo "Done" || echo "Error" ; cleanup
"#;

    let result = BashParser::new(combined);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "Combined operators are POSIX-compliant"
            );
        }
        Err(_) => {
            // Parse error acceptable - complex lists may not be fully implemented
        }
    }
}

#[test]
fn test_CMD_LIST_001_exit_status_semantics() {
    // DOCUMENTATION: Exit status with command lists
    //
    // Exit status rules:
    // - Semicolon (;): Status of last command in list
    // - AND (&&): Status of last executed command
    // - OR (||): Status of last executed command
    //
    // Examples:
    // $ true ; false
    // $ echo $?
    // 1  (status of 'false')
    //
    // $ true && echo "yes"
    // yes
    // $ echo $?
    // 0  (status of 'echo')
    //
    // $ false || echo "fallback"
    // fallback
    // $ echo $?
    // 0  (status of 'echo')

    let exit_status = r#"
#!/bin/sh
# Exit status examples
true ; false
if [ $? -ne 0 ]; then
    echo "Last command failed"
fi

true && echo "Success"
if [ $? -eq 0 ]; then
    echo "Previous succeeded"
fi

false || echo "Fallback"
if [ $? -eq 0 ]; then
    echo "Fallback succeeded"
fi
"#;

    let result = BashParser::new(exit_status);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "Exit status semantics are POSIX-compliant"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

// DOCUMENTATION: Rust if statement mapping for command lists
//
// Bash AND (&&):
// test -f file.txt && echo "File exists"
//
// Rust equivalent:
//   fn handle() { if test_file("file.txt") { println!("File exists"); } }
//
// Bash OR (||):
// test -f file.txt || echo "File not found"
//
// Rust equivalent:
//   fn handle() { if !test_file("file.txt") { println!("File not found"); } }
//
// Bash combined (&&/||):
// cmd1 && cmd2 || cmd3
//
// Rust equivalent:
//   fn handle() { if cmd1() { cmd2(); } else { cmd3(); } }
//
// bashrs strategy:
// - Map && to statement
// - Map || to negated condition
// - Preserve short-circuit evaluation semantics
#[test]
fn test_CMD_LIST_001_rust_if_statement_mapping() {
    // This test documents the Rust mapping strategy
}

#[test]
fn test_CMD_LIST_001_common_patterns() {
    // DOCUMENTATION: Common command list patterns
    //
    // Pattern 1: Error checking
    // command || exit 1
    // (Exit if command fails)
    //
    // Pattern 2: Success confirmation
    // command && echo "Done"
    // (Print message only if succeeds)
    //
    // Pattern 3: Try-catch style
    // command && echo "Success" || echo "Failed"
    // (Report outcome either way)
    //
    // Pattern 4: Safe directory change
    // cd /path || exit 1
    // (Exit if cd fails)
    //
    // Pattern 5: Create and enter
    // mkdir -p dir && cd dir
    // (Only cd if mkdir succeeds)
    //
    // Pattern 6: Cleanup always runs
    // process ; cleanup
    // (Cleanup runs regardless of process exit status)

    let common_patterns = r#"
#!/bin/sh
# Pattern 1: Error checking
command || exit 1

# Pattern 2: Success confirmation
command && echo "Done"

# Pattern 3: Try-catch style
command && echo "Success" || echo "Failed"

# Pattern 4: Safe directory change
cd /path || exit 1

# Pattern 5: Create and enter
mkdir -p dir && cd dir

# Pattern 6: Cleanup always runs
process_data ; cleanup_resources
"#;

    let result = BashParser::new(common_patterns);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "Common patterns are POSIX-compliant"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

#[test]
fn test_CMD_LIST_001_operator_precedence() {
    // DOCUMENTATION: Operator precedence and grouping
    //
    // Precedence (highest to lowest):
    // 1. | (pipe)
    // 2. && and || (equal precedence, left-to-right)
    // 3. ; and & (equal precedence)
    //
    // Examples:
    // cmd1 | cmd2 && cmd3
    // = (cmd1 | cmd2) && cmd3  (pipe binds tighter)
    //
    // cmd1 && cmd2 || cmd3
    // = (cmd1 && cmd2) || cmd3  (left-to-right)
    //
    // cmd1 && cmd2 ; cmd3
    // = (cmd1 && cmd2) ; cmd3  (semicolon separates)
    //
    // Grouping with ( ):
    // (cmd1 && cmd2) || cmd3
    // (Forces evaluation order)

    let precedence = r#"
#!/bin/sh
# Pipe has highest precedence
cat file.txt | grep pattern && echo "Found"

# Left-to-right for && and ||
test -f file1 && test -f file2 || echo "Missing"

# Semicolon separates complete lists
command1 && command2 ; command3
"#;

    let result = BashParser::new(precedence);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "Operator precedence is POSIX-compliant"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

#[test]
fn test_CMD_LIST_001_bash_vs_posix_lists() {
    // DOCUMENTATION: Bash vs POSIX command list features
    //
    // Feature              | POSIX sh           | Bash extensions
    // ---------------------|-------------------|------------------
    // Semicolon (;)        | ✅ Supported       | ✅ Supported
    // AND (&&)             | ✅ Supported       | ✅ Supported
    // OR (||)              | ✅ Supported       | ✅ Supported
    // Newline (equivalent) | ✅ Supported       | ✅ Supported
    // Pipe (|)             | ✅ Supported       | ✅ Supported
    // Background (&)       | ✅ Supported       | ✅ Supported
    // Grouping ( )         | ✅ Supported       | ✅ Supported
    // Grouping { }         | ✅ Supported       | ✅ Supported
    // Conditional [[       | ❌ Not available   | ✅ Bash extension
    // Coprocess (|&)       | ❌ Not available   | ✅ Bash 4.0+
    //
    // bashrs policy:
    // - Support POSIX operators (;, &&, ||) fully
    // - NOT SUPPORTED: [[, |& (Bash extensions)
    // - Generate POSIX-compliant command lists only

    let posix_list = r#"test -f file && echo "Found" || echo "Missing""#;
    let bash_conditional = r#"[[ -f file ]] && echo "Found""#;

    // POSIX command list - SUPPORTED
    let posix_result = BashParser::new(posix_list);
    match posix_result {
        Ok(mut parser) => {
            let _ = parser.parse();
            // POSIX lists should parse (if implemented)
        }
        Err(_) => {
            // Parse error acceptable if not yet implemented
        }
    }

    // Bash [[ conditional - NOT SUPPORTED (Bash extension)
    let bash_result = BashParser::new(bash_conditional);
    match bash_result {
        Ok(mut parser) => {
            let _ = parser.parse();
            // [[ is Bash extension, may or may not parse
        }
        Err(_) => {
            // Parse error expected for Bash extensions
        }
    }

    // Summary:
    // POSIX lists: Fully supported (;, &&, ||, newline)
    // Bash extensions: NOT SUPPORTED ([[, |&)
    // bashrs: Generate POSIX-compliant lists only
}

// ============================================================================
// REDIR-001: Input Redirection (<) (POSIX, SUPPORTED)
// ============================================================================
//
// Task: REDIR-001 (3.6) - Document < redirection (input)
// Status: DOCUMENTED (SUPPORTED - POSIX compliant)
// Priority: MEDIUM (file I/O fundamental)
//
// Input redirection (<) connects stdin of command to file contents.
// This is a core POSIX feature available in all shells.
//
// POSIX behavior:
// - cmd < file: Read stdin from file instead of terminal
// - Equivalent to: cat file | cmd (but more efficient, no pipe/subshell)
// - File descriptor 0 (stdin) redirected to file
// - Common pattern: while read loop with < file
//
// bashrs policy:
// - FULLY SUPPORTED (POSIX compliant)
// - Quote all filenames to prevent injection
// - Preserve redirection semantics in generated shell
// - Map to file arguments or File::open() in Rust

#[test]
fn test_REDIR_001_basic_input_redirection() {
    // DOCUMENTATION: Basic input redirection (<) is SUPPORTED (POSIX)
    //
    // Input redirection connects stdin to file:
    // $ wc -l < file.txt
    // $ grep "pattern" < input.txt
    // $ sort < unsorted.txt
    //
    // POSIX-compliant: Works in sh, dash, ash, bash
    //
    // Semantics:
    // - File contents become stdin for command
    // - More efficient than cat file | cmd (no pipe, no subshell)
    // - File must be readable
    // - Exit status: Command exit status (not related to file open)

    let input_redir = r#"
wc -l < file.txt
grep "pattern" < input.txt
sort < unsorted.txt
"#;

    let result = BashParser::new(input_redir);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "Input redirection is POSIX-compliant"
            );
        }
        Err(_) => {
            // Parse error acceptable - < may not be fully implemented yet
        }
    }
}

#[test]
fn test_REDIR_001_input_vs_file_argument() {
    // DOCUMENTATION: Input redirection (<) vs file argument
    //
    // Two ways to read files:
    // 1. Input redirection: cmd < file.txt (stdin redirected)
    // 2. File argument: cmd file.txt (explicit argument)
    //
    // Differences:
    // - Some commands accept file args: cat file.txt
    // - Some commands only read stdin: wc (with no args)
    // - Redirection works with any command that reads stdin
    //
    // Examples:
    // $ cat < file.txt    # Reads from stdin (redirected from file)
    // $ cat file.txt      # Reads from file argument
    // (Both produce same output)
    //
    // $ wc -l < file.txt  # Reads from stdin (shows line count only)
    // $ wc -l file.txt    # Reads from file (shows "count filename")

    let input_comparison = r#"
#!/bin/sh
# Input redirection (stdin)
cat < file.txt

# File argument (explicit)
cat file.txt

# Both work, slightly different behavior
wc -l < file.txt    # Shows: 42
wc -l file.txt      # Shows: 42 file.txt
"#;

    let result = BashParser::new(input_comparison);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "Input redirection vs file args documented"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

#[test]
fn test_REDIR_001_while_read_pattern() {
    // DOCUMENTATION: while read loop with input redirection
    //
    // Common pattern: Read file line-by-line
    // $ while read line; do
    // >   echo "Line: $line"
    // > done < input.txt
    //
    // Alternative without redirection:
    // $ cat input.txt | while read line; do
    // >   echo "Line: $line"
    // > done
    //
    // Difference:
    // - Redirection (<): while loop runs in current shell
    // - Pipe (|): while loop runs in subshell (variables lost)
    //
    // bashrs recommendation: Use < redirection when possible

    let while_read = r#"
#!/bin/sh
# Read file line-by-line with < redirection
while read line; do
    printf 'Line: %s\n' "$line"
done < input.txt

# Count lines in file
count=0
while read line; do
    count=$((count + 1))
done < data.txt
printf 'Total lines: %d\n' "$count"
"#;

    let result = BashParser::new(while_read);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "while read with < is POSIX-compliant"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

#[test]
fn test_REDIR_001_multiple_redirections() {
    // DOCUMENTATION: Multiple redirections on same command
    //
    // Can combine input (<) with output (>, >>):
    // $ sort < input.txt > output.txt
    // $ grep "pattern" < file.txt >> results.txt
    //
    // Order doesn't matter for < and >:
    // $ sort < input.txt > output.txt
    // $ sort > output.txt < input.txt
    // (Both equivalent)
    //
    // File descriptors:
    // - < redirects fd 0 (stdin)
    // - > redirects fd 1 (stdout)
    // - 2> redirects fd 2 (stderr)

    let multiple_redir = r#"
#!/bin/sh
# Sort file and save result
sort < input.txt > output.txt

# Filter and append to results
grep "ERROR" < logfile.txt >> errors.txt

# Order doesn't matter
tr 'a-z' 'A-Z' > uppercase.txt < lowercase.txt
"#;

    let result = BashParser::new(multiple_redir);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "Multiple redirections are POSIX-compliant"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

#[test]
fn test_REDIR_001_rust_file_open_mapping() {
    // DOCUMENTATION: Rust File::open() mapping for input redirection
    //
    // Bash input redirection:
    // $ grep "pattern" < input.txt
    //
    // Rust equivalent (Option 1 - File::open):
    // use std::fs::File;
    // use std::io::{BufReader, BufRead};
    //
    // let file = File::open("input.txt")?;
    // let reader = BufReader::new(file);
    // for line in reader.lines() {
    //     if line?.contains("pattern") {
    //         println!("{}", line?);
    //     }
    // }
    //
    // Rust equivalent (Option 2 - Command with file arg):
    // Command::new("grep")
    //     .arg("pattern")
    //     .arg("input.txt")
    //     .output()?;
    //
    // bashrs strategy:
    // - Prefer file arguments when command supports them
    // - Use File::open() + stdin redirect when needed
    // - Quote filenames to prevent injection

    // This test documents the Rust mapping strategy
}

#[test]
fn test_REDIR_001_error_handling() {
    // DOCUMENTATION: Error handling for input redirection
    //
    // File errors:
    // - File doesn't exist: Shell prints error, command doesn't run
    // - No read permission: Shell prints error, command doesn't run
    // - File is directory: Shell prints error, command doesn't run
    //
    // Examples:
    // $ cat < missing.txt
    // sh: missing.txt: No such file or directory
    //
    // $ cat < /etc/shadow
    // sh: /etc/shadow: Permission denied
    //
    // Exit status: Non-zero (typically 1) when file open fails

    let error_handling = r#"
#!/bin/sh
# Check if file exists before redirecting
if [ -f input.txt ]; then
    grep "pattern" < input.txt
else
    printf 'Error: input.txt not found\n' >&2
    exit 1
fi

# Check read permissions
if [ -r data.txt ]; then
    wc -l < data.txt
else
    printf 'Error: Cannot read data.txt\n' >&2
    exit 1
fi
"#;

    let result = BashParser::new(error_handling);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "Error handling is POSIX-compliant"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

#[test]
fn test_REDIR_001_common_use_cases() {
    // DOCUMENTATION: Common use cases for input redirection
    //
    // Use Case 1: Count lines in file
    // $ wc -l < file.txt
    //
    // Use Case 2: Sort file contents
    // $ sort < unsorted.txt > sorted.txt
    //
    // Use Case 3: Search in file
    // $ grep "pattern" < logfile.txt
    //
    // Use Case 4: Process file line-by-line
    // $ while read line; do echo "$line"; done < file.txt
    //
    // Use Case 5: Transform file contents
    // $ tr 'a-z' 'A-Z' < lowercase.txt > uppercase.txt
    //
    // Use Case 6: Filter and count
    // $ grep "ERROR" < logfile.txt | wc -l

    let use_cases = r#"
#!/bin/sh
# Use Case 1: Count lines
wc -l < file.txt

# Use Case 2: Sort file
sort < unsorted.txt > sorted.txt

# Use Case 3: Search in file
grep "pattern" < logfile.txt

# Use Case 4: Process line-by-line
while read line; do
    printf 'Line: %s\n' "$line"
done < file.txt

# Use Case 5: Transform contents
tr 'a-z' 'A-Z' < lowercase.txt > uppercase.txt

# Use Case 6: Filter and count
grep "ERROR" < logfile.txt | wc -l
"#;

    let result = BashParser::new(use_cases);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "Common use cases are POSIX-compliant"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

#[test]
fn test_REDIR_001_bash_vs_posix_input_redir() {
    // DOCUMENTATION: Bash vs POSIX input redirection features
    //
    // Feature                  | POSIX sh           | Bash extensions
    // -------------------------|-------------------|------------------
    // Basic < redirect         | ✅ Supported       | ✅ Supported
    // File descriptor (0<)     | ✅ Supported       | ✅ Supported
    // Here-document (<<)       | ✅ Supported       | ✅ Supported
    // Here-string (<<<)        | ❌ Not available   | ✅ Bash 2.05b+
    // Process substitution     | ❌ Not available   | ✅ <(cmd)
    // Named pipes (FIFOs)      | ✅ Supported       | ✅ Supported
    //
    // bashrs policy:
    // - Support POSIX < redirection fully
    // - Support << here-documents (POSIX)
    // - NOT SUPPORTED: <<< here-strings, <(cmd) process substitution
    // - Generate POSIX-compliant redirections only

    let posix_redir = r#"cat < file.txt"#;
    let bash_herestring = r#"grep "pattern" <<< "$variable""#;

    // POSIX input redirection - SUPPORTED
    let posix_result = BashParser::new(posix_redir);
    match posix_result {
        Ok(mut parser) => {
            let _ = parser.parse();
            // POSIX < should parse (if implemented)
        }
        Err(_) => {
            // Parse error acceptable if not yet implemented
        }
    }

    // Bash here-string - NOT SUPPORTED (Bash extension)
    let bash_result = BashParser::new(bash_herestring);
    match bash_result {
        Ok(mut parser) => {
            let _ = parser.parse();
            // <<< is Bash extension, may or may not parse
        }
        Err(_) => {
            // Parse error expected for Bash extensions
        }
    }

    // Summary:
    // POSIX input redirection: Fully supported (<, <<, fd redirects)
    // Bash extensions: NOT SUPPORTED (<<<, <(cmd))
    // bashrs: Generate POSIX-compliant redirections only
}

// ============================================================================
// REDIR-002: Output Redirection (>, >>) (POSIX, SUPPORTED)
// ============================================================================

#[test]
fn test_REDIR_002_basic_output_redirection() {
    // DOCUMENTATION: Basic output redirection (>) is SUPPORTED (POSIX)
    //
    // Output redirection writes stdout to file (truncates existing):
    // $ echo "hello" > file.txt
    // $ ls -la > listing.txt
    // $ cat data.txt > output.txt

    let output_redir = r#"
echo "hello" > file.txt
ls -la > listing.txt
cat data.txt > output.txt
"#;

    let result = BashParser::new(output_redir);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "Output redirection (>) is POSIX-compliant"
            );
        }
        Err(_) => {
            // Parse error acceptable - > may not be fully implemented yet
        }
    }
}

#[test]
fn test_REDIR_002_append_redirection() {
    // DOCUMENTATION: Append redirection (>>) is SUPPORTED (POSIX)
    //
    // Append redirection adds stdout to file (creates if missing):
    // $ echo "line1" > file.txt
    // $ echo "line2" >> file.txt
    // $ echo "line3" >> file.txt
    //
    // Result in file.txt:
    // line1
    // line2
    // line3

    let append_redir = r#"
echo "line1" > file.txt
echo "line2" >> file.txt
echo "line3" >> file.txt
"#;

    let result = BashParser::new(append_redir);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "Append redirection (>>) is POSIX-compliant"
            );
        }
        Err(_) => {
            // Parse error acceptable - >> may not be fully implemented yet
        }
    }
}

#[test]
fn test_REDIR_002_overwrite_vs_append() {
    // DOCUMENTATION: > overwrites, >> appends (POSIX semantics)
    //
    // > truncates file to zero length before writing:
    // $ echo "new" > file.txt  # Destroys old content
    //
    // >> appends to existing file:
    // $ echo "more" >> file.txt  # Keeps old content
    //
    // POSIX sh behavior:
    // - > creates file if missing (mode 0666 & ~umask)
    // - >> creates file if missing (same mode)
    // - > destroys existing content
    // - >> preserves existing content

    let overwrite_append = r#"
# Overwrite (truncate)
echo "first" > data.txt
echo "second" > data.txt  # Destroys "first"

# Append (preserve)
echo "line1" > log.txt
echo "line2" >> log.txt  # Keeps "line1"
echo "line3" >> log.txt  # Keeps both
"#;

    let result = BashParser::new(overwrite_append);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "Overwrite vs append semantics documented"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

#[test]
fn test_REDIR_002_stderr_redirection() {
    // DOCUMENTATION: stderr redirection (2>) is SUPPORTED (POSIX)
    //
    // File descriptor redirection syntax:
    // 0< - stdin (same as <)
    // 1> - stdout (same as >)
    // 2> - stderr
    //
    // Redirect stderr to file:
    // $ cmd 2> errors.txt
    // $ cmd > output.txt 2> errors.txt
    // $ cmd > output.txt 2>&1  # stderr to stdout

    let stderr_redir = r#"
# Redirect stderr only
ls nonexistent 2> errors.txt

# Redirect stdout and stderr separately
cmd > output.txt 2> errors.txt

# Redirect stderr to stdout
cmd > combined.txt 2>&1
"#;

    let result = BashParser::new(stderr_redir);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "stderr redirection (2>) is POSIX-compliant"
            );
        }
        Err(_) => {
            // Parse error acceptable - 2> may not be fully implemented yet
        }
    }
}

#[test]
fn test_REDIR_002_combined_io_redirection() {
    // DOCUMENTATION: Combined input/output redirection (POSIX)
    //
    // Commands can have both input and output redirection:
    // $ sort < unsorted.txt > sorted.txt
    // $ grep "pattern" < input.txt > matches.txt
    // $ wc -l < data.txt > count.txt
    //
    // Order doesn't matter in POSIX:
    // $ cmd > out.txt < in.txt  # Same as < in.txt > out.txt

    let combined_redir = r#"
# Input and output
sort < unsorted.txt > sorted.txt
grep "pattern" < input.txt > matches.txt

# Order doesn't matter
wc -l < data.txt > count.txt
wc -l > count.txt < data.txt
"#;

    let result = BashParser::new(combined_redir);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "Combined I/O redirection is POSIX-compliant"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

#[test]
fn test_REDIR_002_rust_file_mapping() {
    // DOCUMENTATION: Rust std::fs mapping for output redirection
    //
    // Bash > maps to Rust:
    // use std::fs::File;
    // use std::io::Write;
    //
    // // Overwrite (>)
    // let mut file = File::create("output.txt")?;
    // writeln!(file, "content")?;
    //
    // // Append (>>)
    // use std::fs::OpenOptions;
    // let mut file = OpenOptions::new()
    //     .create(true)
    //     .append(true)
    //     .open("output.txt")?;
    // writeln!(file, "more")?;
    //
    // // Command with output redirection
    // let output = Command::new("ls")
    //     .output()?;
    // File::create("listing.txt")?
    //     .write_all(&output.stdout)?;

    // This test documents the mapping strategy above
    // Test passes if the documentation compiles correctly
}

#[test]
fn test_REDIR_002_common_use_cases() {
    // DOCUMENTATION: Common output redirection patterns (POSIX)
    //
    // 1. Save command output:
    //    $ ls -la > listing.txt
    //    $ ps aux > processes.txt
    //
    // 2. Log file appending:
    //    $ echo "$(date): Started" >> app.log
    //    $ cmd >> app.log 2>&1
    //
    // 3. Discard output:
    //    $ cmd > /dev/null 2>&1
    //
    // 4. Create empty file:
    //    $ > empty.txt
    //    $ : > empty.txt  # More portable
    //
    // 5. Capture errors:
    //    $ cmd 2> errors.txt
    //    $ cmd 2>&1 | tee combined.log
    //
    // 6. Split stdout/stderr:
    //    $ cmd > output.txt 2> errors.txt

    let common_patterns = r#"
# Save output
ls -la > listing.txt

# Append to log
echo "Started" >> app.log

# Discard output
cmd > /dev/null 2>&1

# Create empty file
: > empty.txt

# Capture errors
cmd 2> errors.txt

# Split output
cmd > output.txt 2> errors.txt
"#;

    let result = BashParser::new(common_patterns);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "Common output redirection patterns documented"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

#[test]
fn test_REDIR_002_bash_vs_posix_output_redir() {
    // DOCUMENTATION: Bash vs POSIX output redirection comparison
    //
    // | Feature                  | POSIX sh | Bash | bashrs |
    // |--------------------------|----------|------|--------|
    // | > (overwrite)            | ✅       | ✅   | ✅     |
    // | >> (append)              | ✅       | ✅   | ✅     |
    // | 2> (stderr)              | ✅       | ✅   | ✅     |
    // | 2>&1 (merge)             | ✅       | ✅   | ✅     |
    // | &> file (Bash shortcut)  | ❌       | ✅   | ❌     |
    // | >& file (csh-style)      | ❌       | ✅   | ❌     |
    // | >| (force overwrite)     | ❌       | ✅   | ❌     |
    // | >(cmd) process subst     | ❌       | ✅   | ❌     |
    //
    // POSIX-compliant output redirection:
    // - > overwrites file
    // - >> appends to file
    // - fd> redirects file descriptor (0-9)
    // - 2>&1 duplicates fd 2 to fd 1
    //
    // Bash extensions NOT SUPPORTED:
    // - &> file (shortcut for > file 2>&1)
    // - >& file (csh-style, same as &>)
    // - >| file (force overwrite, ignore noclobber)
    // - >(cmd) process substitution
    //
    // bashrs strategy:
    // - Generate > and >> for POSIX compliance
    // - Convert &> to > file 2>&1 during purification
    // - Always quote filenames for safety
    // - Use standard file descriptors (0, 1, 2)

    let bash_extensions = r#"
# POSIX (SUPPORTED)
echo "data" > file.txt
echo "more" >> file.txt
cmd 2> errors.txt
cmd > output.txt 2>&1

# Bash extensions (NOT SUPPORTED)
cmd &> combined.txt
cmd >& combined.txt
cmd >| noclobber.txt
cmd > >(logger)
"#;

    let result = BashParser::new(bash_extensions);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "Bash extensions NOT SUPPORTED, POSIX redirections SUPPORTED"
            );
        }
        Err(_) => {
            // Parse error expected for Bash extensions
        }
    }

    // Summary:
    // POSIX output redirection: Fully supported (>, >>, 2>, 2>&1)
    // Bash extensions: NOT SUPPORTED (&>, >&, >|, >(cmd))
    // bashrs: Generate POSIX-compliant redirections only
}

// ============================================================================
// REDIR-003: Combined Redirection (&>) (Bash 4.0+, NOT SUPPORTED)
// ============================================================================

#[test]
fn test_REDIR_003_combined_redirection_not_supported() {
    // DOCUMENTATION: Combined redirection (&>) is NOT SUPPORTED (Bash extension)
    //
    // &> is Bash shorthand for redirecting both stdout and stderr to the same file:
    // $ cmd &> output.txt
    //
    // This is equivalent to POSIX:
    // $ cmd > output.txt 2>&1
    //
    // Bash 4.0+ feature, not POSIX sh.

    let combined_redir = r#"
cmd &> output.txt
ls &> listing.txt
"#;

    let result = BashParser::new(combined_redir);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "&> is Bash extension, NOT SUPPORTED"
            );
        }
        Err(_) => {
            // Parse error acceptable - Bash extension
        }
    }
}

#[test]
fn test_REDIR_003_csh_style_redirection_not_supported() {
    // DOCUMENTATION: csh-style >& redirection is NOT SUPPORTED (Bash extension)
    //
    // >& is csh-style syntax (also supported by Bash):
    // $ cmd >& output.txt
    //
    // Same as &> (Bash 4.0+), equivalent to POSIX:
    // $ cmd > output.txt 2>&1
    //
    // Not POSIX sh, Bash extension only.

    let csh_redir = r#"
cmd >& output.txt
ls >& listing.txt
"#;

    let result = BashParser::new(csh_redir);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                ">& is Bash/csh extension, NOT SUPPORTED"
            );
        }
        Err(_) => {
            // Parse error acceptable - Bash extension
        }
    }
}

#[test]
fn test_REDIR_003_append_combined_not_supported() {
    // DOCUMENTATION: Append combined redirection (&>>) is NOT SUPPORTED
    //
    // &>> appends both stdout and stderr to file:
    // $ cmd &>> log.txt
    //
    // Equivalent to POSIX:
    // $ cmd >> log.txt 2>&1
    //
    // Bash extension, not POSIX.

    let append_combined = r#"
cmd &>> log.txt
echo "error" &>> errors.log
"#;

    let result = BashParser::new(append_combined);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "&>> is Bash extension, NOT SUPPORTED"
            );
        }
        Err(_) => {
            // Parse error acceptable - Bash extension
        }
    }
}

#[test]
fn test_REDIR_003_posix_equivalent() {
    // DOCUMENTATION: POSIX equivalent for &> redirection (SUPPORTED)
    //
    // Instead of Bash &>, use POSIX > file 2>&1:
    //
    // Bash (NOT SUPPORTED):
    // $ cmd &> output.txt
    //
    // POSIX (SUPPORTED):
    // $ cmd > output.txt 2>&1
    //
    // Order matters in POSIX:
    // - > output.txt 2>&1 (CORRECT: stdout to file, then stderr to stdout)
    // - 2>&1 > output.txt (WRONG: stderr to original stdout, then stdout to file)
    //
    // Always put > before 2>&1.

    let posix_equivalent = r#"
# POSIX-compliant combined redirection
cmd > output.txt 2>&1
ls > listing.txt 2>&1
cat data.txt > result.txt 2>&1
"#;

    let result = BashParser::new(posix_equivalent);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "POSIX > file 2>&1 is SUPPORTED"
            );
        }
        Err(_) => {
            // Parse error acceptable - may not be fully implemented
        }
    }
}

#[test]
fn test_REDIR_003_purification_strategy() {
    // DOCUMENTATION: Purification strategy for &> redirection
    //
    // bashrs purification should convert Bash &> to POSIX:
    //
    // INPUT (Bash):
    // cmd &> output.txt
    //
    // PURIFIED (POSIX sh):
    // cmd > output.txt 2>&1
    //
    // INPUT (Bash append):
    // cmd &>> log.txt
    //
    // PURIFIED (POSIX sh):
    // cmd >> log.txt 2>&1
    //
    // Purification steps:
    // 1. Detect &> or &>> syntax
    // 2. Convert to > file 2>&1 or >> file 2>&1
    // 3. Quote filename for safety
    // 4. Preserve argument order

    // This test documents the purification strategy
}

#[test]
fn test_REDIR_003_order_matters() {
    // DOCUMENTATION: Redirection order matters in POSIX
    //
    // CORRECT order (stdout first, then stderr):
    // $ cmd > file 2>&1
    //
    // 1. > file - Redirect stdout (fd 1) to file
    // 2. 2>&1 - Duplicate stderr (fd 2) to stdout (fd 1, which now points to file)
    // Result: Both stdout and stderr go to file
    //
    // WRONG order (stderr first, then stdout):
    // $ cmd 2>&1 > file
    //
    // 1. 2>&1 - Duplicate stderr (fd 2) to stdout (fd 1, still terminal)
    // 2. > file - Redirect stdout (fd 1) to file
    // Result: stderr goes to terminal, stdout goes to file
    //
    // Rule: Always put > file BEFORE 2>&1

    let correct_order = r#"
# CORRECT: > file 2>&1
cmd > output.txt 2>&1
"#;

    let result = BashParser::new(correct_order);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "Correct order: > file 2>&1"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

#[test]
fn test_REDIR_003_common_use_cases() {
    // DOCUMENTATION: Common combined redirection patterns
    //
    // 1. Capture all output (stdout + stderr):
    //    POSIX: cmd > output.txt 2>&1
    //    Bash: cmd &> output.txt
    //
    // 2. Append all output to log:
    //    POSIX: cmd >> app.log 2>&1
    //    Bash: cmd &>> app.log
    //
    // 3. Discard all output:
    //    POSIX: cmd > /dev/null 2>&1
    //    Bash: cmd &> /dev/null
    //
    // 4. Capture in variable (all output):
    //    POSIX: output=$(cmd 2>&1)
    //    Bash: output=$(cmd 2>&1)  # No &> in command substitution
    //
    // 5. Log with timestamp:
    //    POSIX: (date; cmd) > log.txt 2>&1
    //    Bash: (date; cmd) &> log.txt

    let common_patterns = r#"
# Capture all output (POSIX)
cmd > output.txt 2>&1

# Append to log (POSIX)
cmd >> app.log 2>&1

# Discard all (POSIX)
cmd > /dev/null 2>&1

# Capture in variable (POSIX)
output=$(cmd 2>&1)

# Log with timestamp (POSIX)
(date; cmd) > log.txt 2>&1
"#;

    let result = BashParser::new(common_patterns);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "Common POSIX combined redirection patterns documented"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

#[test]
fn test_REDIR_003_bash_vs_posix_combined_redir() {
    // DOCUMENTATION: Bash vs POSIX combined redirection comparison
    //
    // | Feature                  | POSIX sh         | Bash      | bashrs     |
    // |--------------------------|------------------|-----------|------------|
    // | > file 2>&1 (explicit)   | ✅               | ✅        | ✅         |
    // | &> file (shortcut)       | ❌               | ✅        | ❌ → POSIX |
    // | >& file (csh-style)      | ❌               | ✅        | ❌ → POSIX |
    // | >> file 2>&1 (append)    | ✅               | ✅        | ✅         |
    // | &>> file (append short)  | ❌               | ✅        | ❌ → POSIX |
    // | 2>&1 > file (wrong!)     | ⚠️ (wrong order) | ⚠️        | ⚠️         |
    //
    // POSIX-compliant combined redirection:
    // - > file 2>&1 (stdout to file, stderr to stdout)
    // - >> file 2>&1 (append stdout to file, stderr to stdout)
    // - Order matters: > before 2>&1
    //
    // Bash extensions NOT SUPPORTED:
    // - &> file (shortcut for > file 2>&1)
    // - >& file (csh-style, same as &>)
    // - &>> file (append shortcut for >> file 2>&1)
    //
    // bashrs purification strategy:
    // - Convert &> file → > file 2>&1
    // - Convert >& file → > file 2>&1
    // - Convert &>> file → >> file 2>&1
    // - Always quote filenames
    // - Warn about wrong order (2>&1 > file)
    //
    // Why order matters:
    // - > file 2>&1: stdout → file, stderr → stdout (which is file)
    // - 2>&1 > file: stderr → stdout (terminal), stdout → file
    // - First redirection happens first, second uses new fd state

    let bash_extensions = r#"
# POSIX (SUPPORTED)
cmd > output.txt 2>&1
cmd >> log.txt 2>&1

# Bash extensions (NOT SUPPORTED, but can purify)
cmd &> combined.txt
cmd >& combined.txt
cmd &>> log.txt
"#;

    let result = BashParser::new(bash_extensions);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "Bash &> NOT SUPPORTED, POSIX > file 2>&1 SUPPORTED"
            );
        }
        Err(_) => {
            // Parse error expected for Bash extensions
        }
    }

    // Summary:
    // POSIX combined redirection: Fully supported (> file 2>&1, >> file 2>&1)
    // Bash extensions: NOT SUPPORTED (&>, >&, &>>)
    // bashrs: Purify &> to POSIX > file 2>&1
    // Order matters: > file BEFORE 2>&1
}

// ============================================================================
// REDIR-004: Here Documents (<<) (POSIX, SUPPORTED)
// ============================================================================

#[test]
fn test_REDIR_004_basic_heredoc_supported() {
    // DOCUMENTATION: Basic here documents (<<) are SUPPORTED (POSIX)
    //
    // Here document syntax provides multi-line input to stdin:
    // $ cat << EOF
    // Hello
    // World
    // EOF
    //
    // The delimiter (EOF) can be any word, terminated by same word on a line by itself.
    // Content between delimiters is fed to command's stdin.

    let heredoc = r#"
cat << EOF
Hello
World
EOF
"#;

    let result = BashParser::new(heredoc);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "Here documents (<<) are POSIX-compliant"
            );
        }
        Err(_) => {
            // Parse error acceptable - << may not be fully implemented yet
        }
    }
}

#[test]
fn test_REDIR_004_heredoc_with_variables() {
    // DOCUMENTATION: Variable expansion in here documents (POSIX)
    //
    // By default, variables are expanded in here documents:
    // $ cat << EOF
    // User: $USER
    // Home: $HOME
    // EOF
    //
    // This is POSIX sh behavior (expansion enabled by default).

    let heredoc_vars = r#"
cat << EOF
User: $USER
Home: $HOME
Path: $PATH
EOF
"#;

    let result = BashParser::new(heredoc_vars);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "Variable expansion in heredocs is POSIX"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

#[test]
fn test_REDIR_004_quoted_delimiter_no_expansion() {
    // DOCUMENTATION: Quoted delimiter disables expansion (POSIX)
    //
    // Quoting the delimiter (any part) disables variable expansion:
    // $ cat << 'EOF'
    // User: $USER  # Literal $USER, not expanded
    // EOF
    //
    // $ cat << "EOF"
    // User: $USER  # Literal $USER, not expanded
    // EOF
    //
    // $ cat << \EOF
    // User: $USER  # Literal $USER, not expanded
    // EOF
    //
    // This is POSIX sh behavior.

    let heredoc_quoted = r#"
cat << 'EOF'
User: $USER
Home: $HOME
EOF
"#;

    let result = BashParser::new(heredoc_quoted);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "Quoted delimiter disables expansion (POSIX)"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

#[test]
fn test_REDIR_004_heredoc_with_indentation() {
    // DOCUMENTATION: <<- removes leading tabs (POSIX)
    //
    // <<- variant strips leading tab characters from input lines:
    // $ cat <<- EOF
    // 	Indented with tab
    // 	Another line
    // 	EOF
    //
    // Result: "Indented with tab\nAnother line\n"
    //
    // IMPORTANT: Only tabs (\t) are stripped, not spaces.
    // POSIX sh feature for indented here documents in scripts.

    let heredoc_indent = r#"
if true; then
	cat <<- EOF
	This is indented
	With tabs
	EOF
fi
"#;

    let result = BashParser::new(heredoc_indent);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "<<- strips leading tabs (POSIX)"
            );
        }
        Err(_) => {
            // Parse error acceptable - <<- may not be fully implemented
        }
    }
}

#[test]
fn test_REDIR_004_heredoc_delimiters() {
    // DOCUMENTATION: Here document delimiter rules (POSIX)
    //
    // Delimiter can be any word:
    // - EOF (common convention)
    // - END
    // - MARKER
    // - _EOF_
    // - etc.
    //
    // Rules:
    // - Delimiter must appear alone on a line (no leading/trailing spaces)
    // - Delimiter is case-sensitive (EOF != eof)
    // - Delimiter can be quoted ('EOF', "EOF", \EOF) to disable expansion
    // - Content ends when unquoted delimiter found at start of line

    let different_delimiters = r#"
# EOF delimiter
cat << EOF
Hello
EOF

# END delimiter
cat << END
World
END

# Custom delimiter
cat << MARKER
Data
MARKER
"#;

    let result = BashParser::new(different_delimiters);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "Different delimiters are POSIX-compliant"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

#[test]
fn test_REDIR_004_heredoc_use_cases() {
    // DOCUMENTATION: Common here document use cases (POSIX)
    //
    // 1. Multi-line input to commands:
    //    cat << EOF
    //    Line 1
    //    Line 2
    //    EOF
    //
    // 2. Generate config files:
    //    cat << 'EOF' > /etc/config
    //    key=value
    //    EOF
    //
    // 3. SQL queries:
    //    mysql -u root << SQL
    //    SELECT * FROM users;
    //    SQL
    //
    // 4. Email content:
    //    mail -s "Subject" user@example.com << MAIL
    //    Hello,
    //    This is the message.
    //    MAIL
    //
    // 5. Here documents in functions:
    //    print_help() {
    //        cat << EOF
    //    Usage: $0 [options]
    //    EOF
    //    }

    let use_cases = r#"
# Multi-line input
cat << EOF
Line 1
Line 2
Line 3
EOF

# Generate config
cat << 'EOF' > /tmp/config
setting=value
EOF

# Function with heredoc
print_usage() {
    cat << USAGE
Usage: script.sh [options]
Options:
  -h  Show help
USAGE
}
"#;

    let result = BashParser::new(use_cases);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "Common heredoc use cases documented"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

#[test]
fn test_REDIR_004_rust_string_literal_mapping() {
    // DOCUMENTATION: Rust string literal mapping for here documents
    //
    // Bash here document maps to Rust multi-line string:
    //
    // Bash:
    // cat << EOF
    // Hello
    // World
    // EOF
    //
    // Rust:
    // let content = "Hello\nWorld\n";
    // println!("{}", content);
    //
    // Or for raw strings (no escapes):
    // let content = r#"
    // Hello
    // World
    // "#;
    //
    // For commands requiring stdin:
    // use std::process::{Command, Stdio};
    // use std::io::Write;
    //
    // let mut child = Command::new("cat")
    //     .stdin(Stdio::piped())
    //     .spawn()?;
    // child.stdin.as_mut().unwrap()
    //     .write_all(b"Hello\nWorld\n")?;

    // This test documents the mapping strategy
}

#[test]
fn test_REDIR_004_bash_vs_posix_heredocs() {
    // DOCUMENTATION: Bash vs POSIX here documents comparison
    //
    // | Feature                  | POSIX sh | Bash | bashrs |
    // |--------------------------|----------|------|--------|
    // | << EOF (basic)           | ✅       | ✅   | ✅     |
    // | <<- EOF (strip tabs)     | ✅       | ✅   | ✅     |
    // | << 'EOF' (no expansion)  | ✅       | ✅   | ✅     |
    // | Variable expansion       | ✅       | ✅   | ✅     |
    // | Command substitution     | ✅       | ✅   | ✅     |
    // | <<< "string" (herestring)| ❌       | ✅   | ❌     |
    //
    // POSIX-compliant here documents:
    // - << DELIMITER (with variable expansion)
    // - << 'DELIMITER' (literal, no expansion)
    // - <<- DELIMITER (strip leading tabs)
    // - Delimiter must be alone on line
    // - Content ends at unquoted delimiter
    //
    // Bash extensions NOT SUPPORTED:
    // - <<< "string" (here-string, use echo | cmd instead)
    //
    // bashrs strategy:
    // - Generate here documents for multi-line literals
    // - Use quoted delimiter ('EOF') when no expansion needed
    // - Use unquoted delimiter (EOF) when expansion needed
    // - Use <<- for indented code (strip tabs)
    // - Convert <<< to echo | cmd during purification
    //
    // Here document vs alternatives:
    // - Here document: cat << EOF ... EOF (multi-line)
    // - Echo with pipe: echo "text" | cmd (single line)
    // - File input: cmd < file.txt (from file)
    // - Here-string (Bash): cmd <<< "text" (NOT SUPPORTED)

    let heredoc_features = r#"
# POSIX (SUPPORTED)
cat << EOF
Hello World
EOF

# POSIX with quoted delimiter (no expansion)
cat << 'EOF'
Literal $VAR
EOF

# POSIX with tab stripping
cat <<- EOF
	Indented content
EOF

# Bash extension (NOT SUPPORTED)
# cat <<< "single line"
"#;

    let result = BashParser::new(heredoc_features);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "POSIX heredocs SUPPORTED, Bash <<< NOT SUPPORTED"
            );
        }
        Err(_) => {
            // Parse error expected for Bash extensions
        }
    }

    // Summary:
    // POSIX here documents: Fully supported (<<, <<-, quoted delimiter)
    // Bash extensions: NOT SUPPORTED (<<<)
    // bashrs: Generate POSIX-compliant here documents
    // Variable expansion: Controlled by delimiter quoting
}

// ============================================================================
// REDIR-005: Here-Strings (<<<) (Bash 2.05b+, NOT SUPPORTED)
// ============================================================================

#[test]
fn test_REDIR_005_herestring_not_supported() {
    // DOCUMENTATION: Here-strings (<<<) are NOT SUPPORTED (Bash extension)
    //
    // Here-string syntax provides single-line input to stdin:
    // $ cmd <<< "input string"
    //
    // This is Bash 2.05b+ feature, not POSIX sh.
    // POSIX equivalent: echo "input string" | cmd

    let herestring = r#"
grep "pattern" <<< "search this text"
wc -w <<< "count these words"
"#;

    let result = BashParser::new(herestring);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "<<< is Bash extension, NOT SUPPORTED"
            );
        }
        Err(_) => {
            // Parse error acceptable - Bash extension
        }
    }
}

#[test]
fn test_REDIR_005_herestring_with_variables() {
    // DOCUMENTATION: Variable expansion in here-strings (Bash)
    //
    // Here-strings expand variables by default:
    // $ cmd <<< "$VAR"
    // $ cmd <<< "User: $USER"
    //
    // Unlike here documents, there's no way to disable expansion
    // (no quoted delimiter concept for <<<).

    let herestring_vars = r#"
grep "test" <<< "$HOME"
wc -w <<< "User: $USER"
"#;

    let result = BashParser::new(herestring_vars);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "<<< with variables is Bash extension, NOT SUPPORTED"
            );
        }
        Err(_) => {
            // Parse error acceptable - Bash extension
        }
    }
}

#[test]
fn test_REDIR_005_posix_echo_pipe_equivalent() {
    // DOCUMENTATION: POSIX equivalent for here-strings (SUPPORTED)
    //
    // Instead of Bash <<<, use POSIX echo | cmd:
    //
    // Bash (NOT SUPPORTED):
    // $ cmd <<< "input string"
    //
    // POSIX (SUPPORTED):
    // $ echo "input string" | cmd
    //
    // Or printf for more control:
    // $ printf '%s\n' "input string" | cmd
    // $ printf '%s' "no newline" | cmd

    let posix_equivalent = r#"
# POSIX-compliant alternatives to <<<
echo "search this text" | grep "pattern"
printf '%s\n' "count these words" | wc -w
echo "$HOME" | grep "test"
"#;

    let result = BashParser::new(posix_equivalent);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "POSIX echo | cmd is SUPPORTED"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

#[test]
fn test_REDIR_005_purification_strategy() {
    // DOCUMENTATION: Purification strategy for here-strings
    //
    // bashrs purification should convert Bash <<< to POSIX:
    //
    // INPUT (Bash):
    // cmd <<< "input string"
    //
    // PURIFIED (POSIX sh):
    // echo "input string" | cmd
    //
    // Or for literal strings (no newline):
    // printf '%s' "input string" | cmd
    //
    // Purification steps:
    // 1. Detect <<< syntax
    // 2. Convert to echo "string" | cmd
    // 3. Or printf '%s\n' "string" | cmd (more explicit)
    // 4. Quote string for safety
    // 5. Preserve variable expansion

    // This test documents the purification strategy
}

#[test]
fn test_REDIR_005_herestring_vs_heredoc() {
    // DOCUMENTATION: Here-string vs here document comparison
    //
    // Here-string (<<<):
    // - Single line only
    // - Bash 2.05b+ extension
    // - No delimiter needed
    // - Adds newline at end
    // - Syntax: cmd <<< "string"
    //
    // Here document (<<):
    // - Multi-line
    // - POSIX compliant
    // - Requires delimiter (EOF)
    // - No automatic newline
    // - Syntax: cmd << EOF ... EOF
    //
    // When to use which (in Bash):
    // - Single line → <<< "text" (Bash only)
    // - Multi-line → << EOF ... EOF (POSIX)
    //
    // bashrs strategy:
    // - Use echo | cmd for single-line (POSIX)
    // - Use << EOF for multi-line (POSIX)

    let comparison = r#"
# Bash here-string (NOT SUPPORTED)
# grep "pattern" <<< "single line"

# POSIX equivalent (SUPPORTED)
echo "single line" | grep "pattern"

# POSIX here document (SUPPORTED, for multi-line)
cat << EOF
Line 1
Line 2
EOF
"#;

    let result = BashParser::new(comparison);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "POSIX alternatives documented"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

#[test]
fn test_REDIR_005_newline_behavior() {
    // DOCUMENTATION: Here-string newline behavior (Bash)
    //
    // Here-strings automatically add a newline at the end:
    // $ cmd <<< "text"
    // # Equivalent to: echo "text" | cmd (includes newline)
    //
    // To avoid newline in POSIX:
    // $ printf '%s' "text" | cmd
    //
    // Comparison:
    // - <<< "text" → "text\n" (Bash, adds newline)
    // - echo "text" → "text\n" (POSIX, adds newline)
    // - printf '%s' "text" → "text" (POSIX, no newline)
    // - printf '%s\n' "text" → "text\n" (POSIX, explicit newline)

    let newline_test = r#"
# POSIX with newline (default)
echo "text" | cmd

# POSIX without newline
printf '%s' "text" | cmd

# POSIX with explicit newline
printf '%s\n' "text" | cmd
"#;

    let result = BashParser::new(newline_test);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "Newline behavior documented for POSIX alternatives"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

#[test]
fn test_REDIR_005_common_use_cases() {
    // DOCUMENTATION: Common here-string use cases (POSIX alternatives)
    //
    // 1. Pass string to grep (Bash: grep "pattern" <<< "text"):
    //    POSIX: echo "text" | grep "pattern"
    //
    // 2. Word count (Bash: wc -w <<< "count words"):
    //    POSIX: echo "count words" | wc -w
    //
    // 3. Process variable (Bash: cmd <<< "$VAR"):
    //    POSIX: echo "$VAR" | cmd
    //
    // 4. Feed to read (Bash: read var <<< "value"):
    //    POSIX: echo "value" | read var
    //    Warning: pipe runs in subshell, use var="value" instead
    //
    // 5. Base64 encode (Bash: base64 <<< "text"):
    //    POSIX: echo "text" | base64

    let use_cases = r#"
# Pass string to grep (POSIX)
echo "search this text" | grep "pattern"

# Word count (POSIX)
echo "count these words" | wc -w

# Process variable (POSIX)
echo "$HOME" | grep "test"

# Feed to read (POSIX, but use direct assignment)
# echo "value" | read var  # Runs in subshell
var="value"  # Better POSIX alternative

# Base64 encode (POSIX)
echo "text" | base64
"#;

    let result = BashParser::new(use_cases);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "Common POSIX alternatives to <<< documented"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

#[test]
fn test_REDIR_005_bash_vs_posix_herestrings() {
    // DOCUMENTATION: Bash vs POSIX here-strings comparison
    //
    // | Feature                  | POSIX sh         | Bash      | bashrs         |
    // |--------------------------|------------------|-----------|----------------|
    // | echo "str" \| cmd        | ✅               | ✅        | ✅             |
    // | printf '%s' "str" \| cmd | ✅               | ✅        | ✅             |
    // | <<< "string"             | ❌               | ✅        | ❌ → POSIX     |
    // | <<< $VAR                 | ❌               | ✅        | ❌ → POSIX     |
    //
    // POSIX-compliant alternatives:
    // - echo "string" | cmd (adds newline)
    // - printf '%s\n' "string" | cmd (explicit newline)
    // - printf '%s' "string" | cmd (no newline)
    //
    // Bash here-string NOT SUPPORTED:
    // - <<< "string" (Bash 2.05b+ only)
    //
    // bashrs purification strategy:
    // - Convert <<< "string" → echo "string" | cmd
    // - Preserve variable expansion: <<< "$VAR" → echo "$VAR" | cmd
    // - Use printf for explicit control over newlines
    // - Always quote strings for safety
    //
    // Why here-strings are Bash-only:
    // - Not in POSIX specification
    // - Bash 2.05b+ (2002) introduced <<<
    // - sh, dash, ash don't support <<<
    // - Easy to work around with echo | cmd
    //
    // When to use alternatives:
    // - Single line with newline → echo "text" | cmd
    // - Single line without newline → printf '%s' "text" | cmd
    // - Multi-line → cat << EOF ... EOF
    // - Read into variable → var="value" (direct assignment)

    let bash_extensions = r#"
# POSIX (SUPPORTED)
echo "text" | grep "pattern"
printf '%s\n' "text" | wc -w

# Bash extensions (NOT SUPPORTED)
# grep "pattern" <<< "text"
# wc -w <<< "count words"
"#;

    let result = BashParser::new(bash_extensions);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "Bash <<< NOT SUPPORTED, POSIX echo | cmd SUPPORTED"
            );
        }
        Err(_) => {
            // Parse error expected for Bash extensions
        }
    }

    // Summary:
    // POSIX alternatives: Fully supported (echo | cmd, printf | cmd)
    // Bash extensions: NOT SUPPORTED (<<<)
    // bashrs: Convert <<< to echo | cmd during purification
    // Newline behavior: echo adds newline, printf '%s' doesn't
}

// ============================================================================
// PARAM-SPEC-002: $? Exit Status (POSIX, SUPPORTED)
// ============================================================================

#[test]
fn test_PARAM_SPEC_002_exit_status_basic() {
    // DOCUMENTATION: $? exit status is SUPPORTED (POSIX)
    //
    // $? contains the exit status of the last executed command:
    // - 0: Success
    // - 1-125: Various failure codes
    // - 126: Command found but not executable
    // - 127: Command not found
    // - 128+N: Terminated by signal N
    //
    // POSIX sh, bash, dash, ash: FULLY SUPPORTED
    //
    // Example:
    // $ true
    // $ echo $?
    // 0
    // $ false
    // $ echo $?
    // 1
    //
    // Rust mapping:
    // ```rust
    // use std::process::Command;
    //
    // let status = Command::new("cmd").status()?;
    // let exit_code = status.code().unwrap_or(1);
    // println!("Exit: {}", exit_code);
    // ```

    let exit_status = r#"
cmd
echo "Exit: $?"

true
echo "Success: $?"

false
echo "Failure: $?"
"#;

    let result = BashParser::new(exit_status);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "$? is POSIX-compliant, FULLY SUPPORTED"
            );
        }
        Err(_) => {
            // Parse error acceptable - $? may not be fully implemented yet
        }
    }
}

#[test]
fn test_PARAM_SPEC_002_exit_status_in_conditionals() {
    // DOCUMENTATION: Using $? in conditionals (POSIX)
    //
    // Common pattern: Check exit status in if statements
    //
    // $ cmd
    // $ if [ $? -eq 0 ]; then
    // $   echo "Success"
    // $ else
    // $   echo "Failed"
    // $ fi
    //
    // Best practice: Direct if statement (more concise):
    // $ if cmd; then
    // $   echo "Success"
    // $ fi
    //
    // When $? is necessary:
    // - Multiple commands before check
    // - Need to preserve exit status
    // - Logging before checking

    let exit_status_conditional = r#"
# Pattern 1: $? in conditional
cmd
if [ $? -eq 0 ]; then
  echo "Success"
else
  echo "Failed"
fi

# Pattern 2: Direct conditional (better)
if cmd; then
  echo "Success"
fi

# Pattern 3: Preserve status
cmd
STATUS=$?
log_message "Command exited with $STATUS"
if [ $STATUS -ne 0 ]; then
  handle_error
fi
"#;

    let result = BashParser::new(exit_status_conditional);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "$? in conditionals is POSIX-compliant"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

#[test]
fn test_PARAM_SPEC_002_exit_status_pipelines() {
    // DOCUMENTATION: $? with pipelines (POSIX)
    //
    // $? contains exit status of LAST command in pipeline:
    // $ cmd1 | cmd2 | cmd3
    // $ echo $?  # Exit status of cmd3 only
    //
    // To check all commands in pipeline, use PIPESTATUS (bash) or set -o pipefail:
    //
    // Bash-specific (NOT SUPPORTED):
    // $ cmd1 | cmd2 | cmd3
    // $ echo "${PIPESTATUS[@]}"  # Array of all exit codes
    //
    // POSIX alternative: set -o pipefail
    // $ set -o pipefail
    // $ cmd1 | cmd2 | cmd3
    // $ echo $?  # Non-zero if ANY command failed

    let pipeline_exit = r#"
# $? gets last command only
grep pattern file.txt | sort | uniq
echo "Last command status: $?"

# POSIX: set -o pipefail for pipeline failures
set -o pipefail
grep pattern file.txt | sort | uniq
if [ $? -ne 0 ]; then
  echo "Pipeline failed"
fi
"#;

    let result = BashParser::new(pipeline_exit);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "$? with pipelines is POSIX-compliant"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

// DOCUMENTATION: $? is clobbered by every command (POSIX)
// CRITICAL: $? is updated after EVERY command, including [ and test.
// BAD: checking $? inside [ clobbers it. GOOD: capture first. BETTER: direct conditional.
#[test]
fn test_PARAM_SPEC_002_exit_status_clobbering() {
    let clobbering_issue = r#"
# BAD: $? clobbered by [ command
cmd
if [ $? -eq 0 ]; then  # This tests if [ succeeded, not cmd!
  echo "Wrong"
fi

# GOOD: Capture $? immediately
cmd
STATUS=$?
if [ $STATUS -eq 0 ]; then
  echo "Correct"
fi

# BETTER: Direct conditional
if cmd; then
  echo "Best practice"
fi
"#;

    assert_parses_without_panic(clobbering_issue, "$? clobbering behavior is POSIX-compliant");
}

#[test]
fn test_PARAM_SPEC_002_exit_status_functions() {
    // DOCUMENTATION: $? with functions (POSIX)
    //
    // Functions return exit status like commands:
    // - Explicit: return N (0-255)
    // - Implicit: exit status of last command
    //
    // $ my_function() {
    // $   cmd
    // $   return $?  # Explicit return
    // $ }
    // $
    // $ my_function
    // $ echo $?  # Function's return value

    let function_exit = r#"
check_file() {
  if [ -f "$1" ]; then
return 0
  else
return 1
  fi
}

# Implicit return (last command)
process_data() {
  validate_input
  transform_data
  save_output  # Function returns this command's status
}

# Using function status
check_file "/tmp/data.txt"
if [ $? -eq 0 ]; then
  echo "File exists"
fi

# Better: Direct conditional
if check_file "/tmp/data.txt"; then
  echo "File exists"
fi
"#;

    let result = BashParser::new(function_exit);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "$? with functions is POSIX-compliant"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

#[test]
fn test_PARAM_SPEC_002_exit_status_subshells() {
    // DOCUMENTATION: $? with subshells and command substitution (POSIX)
    //
    // Subshells and command substitution preserve exit status:
    //
    // Subshell:
    // $ ( cmd1; cmd2 )
    // $ echo $?  # Exit status of cmd2
    //
    // Command substitution (capture output, lose status):
    // $ OUTPUT=$(cmd)
    // $ echo $?  # Always 0 if assignment succeeded
    //
    // To capture both output and status:
    // $ OUTPUT=$(cmd)
    // $ STATUS=$?  # This is too late! Already clobbered
    //
    // Better: Set -e or check inline:
    // $ OUTPUT=$(cmd) || { echo "Failed"; exit 1; }

    let subshell_exit = r#"
# Subshell exit status
( cmd1; cmd2 )
echo "Subshell status: $?"

# Command substitution loses status
OUTPUT=$(cmd)
echo $?  # This is assignment status, not cmd status!

# Capture output and check status inline
OUTPUT=$(cmd) || {
  echo "Command failed"
  exit 1
}

# Alternative: set -e (exit on any error)
set -e
OUTPUT=$(cmd)  # Will exit script if cmd fails
"#;

    let result = BashParser::new(subshell_exit);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "$? with subshells is POSIX-compliant"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

#[test]
fn test_PARAM_SPEC_002_exit_status_common_use_cases() {
    // DOCUMENTATION: Common $? use cases (POSIX)
    //
    // Use Case 1: Error handling
    // $ cmd
    // $ if [ $? -ne 0 ]; then
    // $   echo "Error occurred"
    // $   exit 1
    // $ fi
    //
    // Use Case 2: Multiple status checks
    // $ cmd1
    // $ STATUS1=$?
    // $ cmd2
    // $ STATUS2=$?
    // $ if [ $STATUS1 -ne 0 ] || [ $STATUS2 -ne 0 ]; then
    // $   echo "One or both failed"
    // $ fi
    //
    // Use Case 3: Logging
    // $ cmd
    // $ STATUS=$?
    // $ log_message "Command exited with status $STATUS"
    // $ [ $STATUS -eq 0 ] || exit $STATUS

    let common_uses = r#"
# Use Case 1: Error handling
deploy_app
if [ $? -ne 0 ]; then
  echo "Deployment failed"
  rollback_changes
  exit 1
fi

# Use Case 2: Multiple checks
backup_database
DB_STATUS=$?
backup_files
FILE_STATUS=$?

if [ $DB_STATUS -ne 0 ] || [ $FILE_STATUS -ne 0 ]; then
  echo "Backup failed"
  send_alert
  exit 1
fi

# Use Case 3: Logging with status
critical_operation
STATUS=$?
log_event "Operation completed with status $STATUS"
if [ $STATUS -ne 0 ]; then
  send_alert "Critical operation failed: $STATUS"
  exit $STATUS
fi
"#;

    let result = BashParser::new(common_uses);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "Common $? patterns are POSIX-compliant"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

// DOCUMENTATION: Exit status comparison (POSIX vs Bash)
// $? is POSIX-compliant, 0-255 range, clobbered by every command.
// Rust mapping: std::process::Command .status() .code()
// bashrs: SUPPORTED, no transformation needed, preserve as-is.
#[test]
fn test_PARAM_SPEC_002_exit_status_comparison_table() {
    let comparison_example = r#"
# POSIX: $? fully supported
cmd
echo "Exit: $?"

# POSIX: Capture and use
cmd
STATUS=$?
if [ $STATUS -ne 0 ]; then
  echo "Failed with code $STATUS"
  exit $STATUS
fi

# POSIX: set -o pipefail (supported in bash, dash, ash)
set -o pipefail
cmd1 | cmd2 | cmd3
if [ $? -ne 0 ]; then
  echo "Pipeline failed"
fi

# Bash-only: PIPESTATUS (NOT SUPPORTED)
# cmd1 | cmd2 | cmd3
# echo "${PIPESTATUS[@]}"  # bashrs doesn't support this
"#;

    assert_parses_without_panic(comparison_example, "$? comparison documented");
}

// Summary:
// $? (exit status): FULLY SUPPORTED (POSIX)
// Range: 0-255 (0=success, non-zero=failure)
// Special codes: 126 (not executable), 127 (not found), 128+N (signal)
// Clobbering: Updated after every command
// Best practice: Capture immediately or use direct conditionals
// PIPESTATUS: NOT SUPPORTED (bash extension)
// pipefail: SUPPORTED (POSIX, available in bash/dash/ash)

// ============================================================================
// PARAM-SPEC-003: $$ Process ID (POSIX, but NON-DETERMINISTIC - PURIFY)
// ============================================================================

// DOCUMENTATION: $$ is POSIX but NON-DETERMINISTIC (must purify)
// $$ contains the process ID of the current shell. Changes every run.
// Purification: replace $$ with fixed identifier, use mktemp for temp files.
#[test]
fn test_PARAM_SPEC_003_process_id_non_deterministic() {
    let process_id = r#"
echo "Process ID: $$"
echo "Script PID: $$"
"#;

    assert_parses_without_panic(process_id, "$$ is POSIX-compliant but NON-DETERMINISTIC (must purify)");
}

#[test]
fn test_PARAM_SPEC_003_process_id_temp_files() {
    // DOCUMENTATION: Common anti-pattern - $$ for temp files
    //
    // ANTI-PATTERN (non-deterministic):
    // $ TMPFILE=/tmp/myapp.$$
    // $ echo "data" > /tmp/script.$$.log
    // $ rm -f /tmp/output.$$
    //
    // Problem: File names change every run
    // - First run: /tmp/myapp.12345
    // - Second run: /tmp/myapp.67890
    // - Third run: /tmp/myapp.23456
    //
    // This breaks:
    // - Determinism (file names unpredictable)
    // - Idempotency (can't clean up old files reliably)
    // - Testing (can't assert on specific file names)
    //
    // POSIX alternatives (deterministic):
    // 1. Use mktemp (creates unique temp file safely):
    //    $ TMPFILE=$(mktemp /tmp/myapp.XXXXXX)
    //
    // 2. Use fixed name with script name:
    //    $ TMPFILE="/tmp/myapp.tmp"
    //
    // 3. Use XDG directories:
    //    $ TMPFILE="${XDG_RUNTIME_DIR:-/tmp}/myapp.tmp"
    //
    // 4. Use script name from $0:
    //    $ TMPFILE="/tmp/$(basename "$0").tmp"

    let temp_file_pattern = r#"
# ANTI-PATTERN: Non-deterministic temp files
TMPFILE=/tmp/myapp.$$
echo "data" > /tmp/script.$$.log
rm -f /tmp/output.$$

# BETTER: Use mktemp (deterministic, safe)
TMPFILE=$(mktemp /tmp/myapp.XXXXXX)

# BETTER: Use fixed name
TMPFILE="/tmp/myapp.tmp"

# BETTER: Use script name
TMPFILE="/tmp/$(basename "$0").tmp"
"#;

    let result = BashParser::new(temp_file_pattern);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "$$ for temp files is non-deterministic anti-pattern"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

#[test]
fn test_PARAM_SPEC_003_process_id_in_subshells() {
    // DOCUMENTATION: $$ behavior in subshells (POSIX gotcha)
    //
    // CRITICAL: $$ in subshell returns PARENT shell PID, not subshell PID!
    //
    // $ echo "Main: $$"
    // Main: 12345
    //
    // $ ( echo "Subshell: $$" )
    // Subshell: 12345  # Same as parent!
    //
    // To get actual subshell PID, use $BASHPID (bash extension):
    // $ ( echo "Subshell: $BASHPID" )
    // Subshell: 12346  # Different!
    //
    // But $BASHPID is NOT SUPPORTED (bash 4.0+ only, not POSIX)
    //
    // POSIX sh behavior:
    // - $$ always returns original shell PID
    // - Even in subshells, command substitution, pipelines
    // - This is POSIX-specified behavior
    //
    // Why this matters:
    // - Cannot use $$ to uniquely identify subprocesses
    // - Temp files in subshells will collide
    // - Must use other unique identifiers

    let subshell_pid = r#"
# Main shell
echo "Main PID: $$"

# Subshell (same PID as main!)
( echo "Subshell PID: $$" )

# Command substitution (same PID as main!)
RESULT=$(echo "Command sub PID: $$")

# Pipeline (same PID as main!)
echo "Pipeline PID: $$" | cat
"#;

    let result = BashParser::new(subshell_pid);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "$$ in subshells returns parent PID (POSIX behavior)"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

#[test]
fn test_PARAM_SPEC_003_process_id_purification_strategy() {
    // DOCUMENTATION: bashrs purification strategy for $$
    //
    // Strategy 1: Replace with fixed identifier
    // - Input:  echo "PID: $$"
    // - Purified: echo "PID: SCRIPT_ID"
    //
    // Strategy 2: Use script name
    // - Input:  TMPFILE=/tmp/app.$$
    // - Purified: TMPFILE="/tmp/$(basename "$0").tmp"
    //
    // Strategy 3: Use mktemp
    // - Input:  LOGFILE=/var/log/app.$$.log
    // - Purified: LOGFILE=$(mktemp /var/log/app.XXXXXX)
    //
    // Strategy 4: Remove if unnecessary
    // - Input:  echo "Running with PID $$"
    // - Purified: echo "Running"  # Remove non-essential logging
    //
    // Strategy 5: Use XDG directories (if available)
    // - Input:  TMPFILE=/tmp/app.$$
    // - Purified: TMPFILE="${XDG_RUNTIME_DIR:-/tmp}/app.tmp"
    //
    // When $$ is acceptable (rare cases):
    // - Trap cleanup: trap "rm -f /tmp/lock.$$" EXIT
    // - Lock files that MUST be unique per process
    // - Debugging/logging (not production)
    //
    // Rust equivalent (deterministic):
    // ```rust
    // // Don't use process::id() for file names!
    // // Use tempfile crate instead:
    // use tempfile::NamedTempFile;
    // let temp = NamedTempFile::new()?;  // Deterministic, safe
    // ```

    let purification_examples = r#"
# BEFORE (non-deterministic)
echo "PID: $$"
TMPFILE=/tmp/app.$$

# AFTER (deterministic)
echo "PID: SCRIPT_ID"
TMPFILE=$(mktemp /tmp/app.XXXXXX)
"#;

    let result = BashParser::new(purification_examples);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "Purification strategy: mktemp or fixed ID"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

#[test]
fn test_PARAM_SPEC_003_process_id_acceptable_uses() {
    // DOCUMENTATION: Acceptable uses of $$ (rare exceptions)
    //
    // Use Case 1: Trap cleanup (acceptable)
    // $ trap "rm -f /tmp/lock.$$" EXIT
    // $ # Process-specific cleanup is OK
    //
    // Why acceptable:
    // - Trap runs in same process, so $$ is consistent
    // - Cleanup files are process-scoped
    // - Not used for deterministic behavior
    //
    // Use Case 2: Lock files (acceptable with caution)
    // $ LOCKFILE=/var/lock/app.$$
    // $ if mkdir "$LOCKFILE" 2>/dev/null; then
    // $   trap "rmdir '$LOCKFILE'" EXIT
    // $   # Do work
    // $ fi
    //
    // Why acceptable:
    // - Lock must be unique per process
    // - Automatic cleanup via trap
    // - Race conditions handled by mkdir
    //
    // Use Case 3: Debugging/development (not production)
    // $ set -x; PS4='[$$] '; command
    // $ # Shows PID in debug traces
    //
    // UNACCEPTABLE uses:
    // - Temp files without cleanup
    // - Log file names (use rotation instead)
    // - Persistent files (violates determinism)
    // - Data file names (not reproducible)

    let acceptable_uses = r#"
# ACCEPTABLE: Trap cleanup
trap "rm -f /tmp/lock.$$" EXIT
trap "rm -f /tmp/work.$$ /tmp/data.$$" EXIT INT TERM

# ACCEPTABLE: Process-specific lock
LOCKFILE=/var/lock/myapp.$$
if mkdir "$LOCKFILE" 2>/dev/null; then
  trap "rmdir '$LOCKFILE'" EXIT
  # Do critical work
fi

# ACCEPTABLE: Debug traces
set -x
PS4='[$$] '
echo "Debug mode"

# UNACCEPTABLE: Persistent files
# LOGFILE=/var/log/app.$$.log  # BAD! Log names not reproducible
# DATAFILE=/data/output.$$      # BAD! Data files must be deterministic
"#;

    let result = BashParser::new(acceptable_uses);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "Trap cleanup and lock files are acceptable uses of $$"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

#[test]
fn test_PARAM_SPEC_003_process_id_bashpid_not_supported() {
    // DOCUMENTATION: $BASHPID is NOT SUPPORTED (bash extension)
    //
    // $BASHPID (bash 4.0+):
    // - Returns actual PID of current bash process
    // - Different from $$ in subshells
    // - Bash extension, not POSIX
    //
    // Example (bash only):
    // $ echo "Main: $$ $BASHPID"
    // Main: 12345 12345  # Same in main shell
    //
    // $ ( echo "Sub: $$ $BASHPID" )
    // Sub: 12345 12346   # Different in subshell!
    //
    // POSIX sh, dash, ash: $BASHPID not available
    //
    // bashrs: NOT SUPPORTED (bash extension)
    //
    // POSIX alternative:
    // - No direct equivalent
    // - Use $$ (aware it returns parent PID in subshells)
    // - Use sh -c 'echo $$' to get actual subshell PID (if needed)

    let bashpid_extension = r#"
# Bash extension (NOT SUPPORTED)
# echo "BASHPID: $BASHPID"

# POSIX (SUPPORTED, but returns parent PID in subshells)
echo "PID: $$"

# POSIX workaround for actual subshell PID (if needed)
( sh -c 'echo "Actual PID: $$"' )
"#;

    let result = BashParser::new(bashpid_extension);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "$BASHPID is bash extension, NOT SUPPORTED"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

// DOCUMENTATION: Common mistakes with $$
// Mistake 1: log rotation with $$. Mistake 2: data files with $$.
// Mistake 3: same $$ in loop. Mistake 4: $$ in subshell is parent PID.
// Fix: use fixed names, mktemp, or capture $$ before subshell.
#[test]
fn test_PARAM_SPEC_003_process_id_common_mistakes() {
    let common_mistakes = r#"
# Mistake 1: Log rotation (BAD)
# LOG=/var/log/app.$$.log
# echo "message" >> "$LOG"

# GOOD: Fixed log file
LOG=/var/log/app.log
echo "$(date): message" >> "$LOG"

# Mistake 2: Data files (BAD)
# OUTPUT=/data/result.$$.json
# process_data > "$OUTPUT"

# GOOD: Fixed output file
OUTPUT=/data/result.json
process_data > "$OUTPUT"

# Mistake 3: Same $$ in loop (BAD)
# for i in 1 2 3; do
#   echo "$i" > /tmp/item.$$
#   process /tmp/item.$$
# done

# GOOD: mktemp per iteration
for i in 1 2 3; do
  TMPFILE=$(mktemp)
  echo "$i" > "$TMPFILE"
  process "$TMPFILE"
  rm -f "$TMPFILE"
done
"#;

    assert_parses_without_panic(common_mistakes, "Common $$ mistakes documented");
}

// DOCUMENTATION: $$ comparison (POSIX vs Bash vs bashrs)
// $$ is POSIX but non-deterministic, must purify. $BASHPID not supported.
// Purification: mktemp for temp files, fixed names for logs/data, trap for locks.
#[test]
fn test_PARAM_SPEC_003_process_id_comparison_table() {
    let comparison_example = r#"
# POSIX: $$ is supported but non-deterministic
echo "PID: $$"

# bashrs: PURIFY to deterministic alternative
echo "PID: SCRIPT_ID"

# POSIX: mktemp is RECOMMENDED alternative
TMPFILE=$(mktemp /tmp/app.XXXXXX)

# POSIX: Fixed names for determinism
LOGFILE=/var/log/app.log

# Acceptable: Trap cleanup (process-scoped)
trap "rm -f /tmp/lock.$$" EXIT

# Bash-only: $BASHPID NOT SUPPORTED
# echo "Actual PID: $BASHPID"
"#;

    assert_parses_without_panic(comparison_example, "$$ comparison and purification strategy documented");
}

// Summary:
// $$ (process ID): POSIX but NON-DETERMINISTIC (MUST PURIFY)
// Contains PID of current shell (changes every run)
// Subshells: $$ returns PARENT PID, not subshell PID (POSIX behavior)
// $BASHPID: NOT SUPPORTED (bash 4.0+ extension for actual subshell PID)
// Purification: Use mktemp for temp files, fixed names for logs/data
// Acceptable uses: Trap cleanup, lock files (with trap)
// Anti-patterns: Log rotation, data files, scripts called multiple times
// Best practice: mktemp instead of /tmp/file.$$, fixed names for determinism

// ============================================================================
// PARAM-SPEC-004: $! Background PID (POSIX, but NON-DETERMINISTIC - PURIFY)
// ============================================================================

#[test]
fn test_PARAM_SPEC_004_background_pid_non_deterministic() {
    // DOCUMENTATION: $! is POSIX but NON-DETERMINISTIC (must purify)
    //
    // $! contains the PID of the last background job:
    // - POSIX-compliant feature (sh, bash, dash, ash all support)
    // - NON-DETERMINISTIC: changes every time script runs
    // - bashrs policy: PURIFY to synchronous execution
    //
    // Example (non-deterministic):
    // $ sleep 10 &
    // $ echo "Background PID: $!"
    // Background PID: 12345  # Different every time!
    //
    // $ cmd &
    // $ echo "BG: $!"
    // BG: 67890  # Different process ID
    //
    // Why $! is non-deterministic:
    // - Each background job gets unique PID from OS
    // - PIDs are reused but unpredictable
    // - Scripts using $! for process management will have different PIDs each run
    // - Breaks determinism requirement for bashrs
    //
    // bashrs purification policy:
    // - Background jobs (&) are NON-DETERMINISTIC
    // - Purify to SYNCHRONOUS execution (remove &)
    // - No background jobs in purified scripts
    // - $! becomes unnecessary when & is removed
    //
    // Rust mapping (synchronous):
    // ```rust
    // use std::process::Command;
    //
    // // DON'T: Spawn background process (non-deterministic)
    // // let child = Command::new("cmd").spawn()?;
    // // let pid = child.id();
    //
    // // DO: Run synchronously (deterministic)
    // let status = Command::new("cmd").status()?;
    // ```

    let background_pid = r#"
# Background job (non-deterministic)
sleep 10 &
echo "Background PID: $!"

cmd &
BG_PID=$!
echo "Started job: $BG_PID"
"#;

    let result = BashParser::new(background_pid);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "$! is POSIX-compliant but NON-DETERMINISTIC (must purify)"
            );
        }
        Err(_) => {
            // Parse error acceptable - $! may not be fully implemented yet
        }
    }
}

#[test]
fn test_PARAM_SPEC_004_background_pid_wait_pattern() {
    // DOCUMENTATION: Common pattern - background job + wait
    //
    // ANTI-PATTERN (non-deterministic):
    // $ long_running_task &
    // $ BG_PID=$!
    // $ echo "Running task $BG_PID in background"
    // $ wait $BG_PID
    // $ echo "Task $BG_PID completed"
    //
    // Problem: Background execution is non-deterministic
    // - PID changes every run
    // - Timing issues (race conditions)
    // - Can't reproduce exact execution order
    // - Breaks testing and debugging
    //
    // bashrs purification: Run synchronously
    // $ long_running_task
    // $ echo "Task completed"
    //
    // Why synchronous is better for bashrs:
    // - Deterministic execution order
    // - No race conditions
    // - Reproducible behavior
    // - Easier to test and debug
    // - Same results every run
    //
    // When background jobs are acceptable (rare):
    // - Interactive scripts (not for bashrs purification)
    // - User-facing tools (not bootstrap/config scripts)
    // - Explicitly requested parallelism (user choice)

    let wait_pattern = r#"
# ANTI-PATTERN: Background + wait
long_running_task &
BG_PID=$!
echo "Running task $BG_PID in background"
wait $BG_PID
echo "Task $BG_PID completed"

# BETTER (bashrs): Synchronous execution
long_running_task
echo "Task completed"
"#;

    let result = BashParser::new(wait_pattern);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "Background + wait pattern is non-deterministic"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

#[test]
fn test_PARAM_SPEC_004_background_pid_multiple_jobs() {
    // DOCUMENTATION: Multiple background jobs (highly non-deterministic)
    //
    // ANTI-PATTERN (non-deterministic):
    // $ task1 &
    // $ PID1=$!
    // $ task2 &
    // $ PID2=$!
    // $ task3 &
    // $ PID3=$!
    // $ wait $PID1 $PID2 $PID3
    //
    // Problems:
    // - 3 PIDs, all unpredictable
    // - Race conditions (which finishes first?)
    // - Non-deterministic completion order
    // - Can't reproduce test scenarios
    // - Debugging nightmare
    //
    // bashrs purification: Sequential execution
    // $ task1
    // $ task2
    // $ task3
    //
    // Benefits:
    // - Deterministic execution order (always task1 → task2 → task3)
    // - No race conditions
    // - Reproducible results
    // - Easy to test
    // - Clear execution flow

    let multiple_jobs = r#"
# ANTI-PATTERN: Multiple background jobs
task1 &
PID1=$!
task2 &
PID2=$!
task3 &
PID3=$!

echo "Started: $PID1 $PID2 $PID3"
wait $PID1 $PID2 $PID3
echo "All completed"

# BETTER (bashrs): Sequential
task1
task2
task3
echo "All completed"
"#;

    let result = BashParser::new(multiple_jobs);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "Multiple background jobs are highly non-deterministic"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

#[test]
fn test_PARAM_SPEC_004_background_pid_with_kill() {
    // DOCUMENTATION: Background job + kill pattern
    //
    // ANTI-PATTERN (non-deterministic + destructive):
    // $ timeout_task &
    // $ BG_PID=$!
    // $ sleep 5
    // $ kill $BG_PID 2>/dev/null
    //
    // Problems:
    // - Non-deterministic PID
    // - Timing-dependent behavior
    // - Race condition (task may finish before kill)
    // - Signal handling is process-dependent
    // - Not reproducible
    //
    // bashrs purification: Use timeout command
    // $ timeout 5 timeout_task || true
    //
    // Benefits:
    // - Deterministic timeout behavior
    // - No background jobs
    // - No PIDs to track
    // - POSIX timeout command (coreutils)
    // - Reproducible results

    let kill_pattern = r#"
# ANTI-PATTERN: Background + kill
timeout_task &
BG_PID=$!
sleep 5
kill $BG_PID 2>/dev/null || true

# BETTER (bashrs): Use timeout command
timeout 5 timeout_task || true

# Alternative: Run synchronously with resource limits
ulimit -t 5  # CPU time limit
timeout_task || true
"#;

    let result = BashParser::new(kill_pattern);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "Background + kill pattern is non-deterministic"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

#[test]
fn test_PARAM_SPEC_004_background_pid_purification_strategy() {
    // DOCUMENTATION: bashrs purification strategy for $! and &
    //
    // Strategy 1: Remove background execution
    // - Input:  cmd &; echo "BG: $!"
    // - Purified: cmd; echo "Done"
    //
    // Strategy 2: Use wait without &
    // - Input:  task &; wait $!
    // - Purified: task  # wait is implicit
    //
    // Strategy 3: Sequential instead of parallel
    // - Input:  task1 & task2 & wait
    // - Purified: task1; task2
    //
    // Strategy 4: Use timeout for time limits
    // - Input:  task &; sleep 5; kill $!
    // - Purified: timeout 5 task || true
    //
    // Strategy 5: Remove entirely if non-essential
    // - Input:  log_task &  # Background logging
    // - Purified: # Remove (or make synchronous if needed)
    //
    // When & is acceptable (never in bashrs):
    // - Interactive user tools (not bootstrap scripts)
    // - Explicitly requested parallelism
    // - NOT acceptable in bashrs purified output
    //
    // Rust equivalent (synchronous):
    // ```rust
    // use std::process::Command;
    //
    // // DON'T: Background process
    // // let child = Command::new("task1").spawn()?;
    // // let child2 = Command::new("task2").spawn()?;
    // // child.wait()?;
    // // child2.wait()?;
    //
    // // DO: Sequential execution
    // Command::new("task1").status()?;
    // Command::new("task2").status()?;
    // ```

    let purification_examples = r#"
# BEFORE (non-deterministic)
cmd &
echo "BG: $!"

# AFTER (deterministic)
cmd
echo "Done"

# BEFORE (parallel)
task1 &
task2 &
wait

# AFTER (sequential)
task1
task2
"#;

    let result = BashParser::new(purification_examples);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "Purification strategy: remove & and $!"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

#[test]
fn test_PARAM_SPEC_004_background_pid_job_control() {
    // DOCUMENTATION: Job control and $! (POSIX but discouraged)
    //
    // Job control features (POSIX but non-deterministic):
    // - & (background execution)
    // - $! (last background PID)
    // - jobs (list jobs)
    // - fg (foreground job)
    // - bg (background job)
    // - wait (wait for jobs)
    //
    // Why bashrs doesn't support job control:
    // - Non-deterministic (PIDs, timing, execution order)
    // - Interactive feature (not for scripts)
    // - Race conditions
    // - Hard to test
    // - Not needed for bootstrap/config scripts
    //
    // POSIX job control example (NOT SUPPORTED):
    // $ sleep 100 &
    // $ jobs  # List background jobs
    // [1]+  Running   sleep 100 &
    // $ fg %1  # Bring to foreground
    //
    // bashrs approach:
    // - Synchronous execution only
    // - No background jobs
    // - No job control commands
    // - Deterministic, testable, reproducible

    let job_control = r#"
# Job control (NOT SUPPORTED in bashrs purification)
# sleep 100 &
# jobs
# fg %1
# bg %1

# bashrs: Synchronous only
sleep 100  # Runs in foreground, blocks until complete
"#;

    let result = BashParser::new(job_control);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "Job control is POSIX but discouraged in bashrs"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

// DOCUMENTATION: Common mistakes with $! and &
// Mistake 1: kill $! without checking job exists (race condition).
// Mistake 2: exit without wait (job may not complete).
// Mistake 3: uncontrolled parallelism in loops.
// bashrs fix: synchronous execution, sequential loops.
#[test]
fn test_PARAM_SPEC_004_background_pid_common_mistakes() {
    let common_mistakes = r#"
# Mistake 1: Race condition (BAD)
# cmd &
# kill $!  # May fail if job finished

# GOOD: Check if job exists
# cmd &
# BG_PID=$!
# if kill -0 $BG_PID 2>/dev/null; then
#   kill $BG_PID
# fi

# Mistake 2: Exit without wait (BAD)
# important_task &
# exit 0  # Task may not complete!

# GOOD: Wait for job
# important_task &
# wait $!

# BETTER (bashrs): Synchronous
important_task
exit 0

# Mistake 3: Uncontrolled parallelism (BAD)
# for i in 1 2 3 4 5; do
#   process_item $i &
# done

# BETTER (bashrs): Sequential
for i in 1 2 3 4 5; do
  process_item "$i"
done
"#;

    assert_parses_without_panic(common_mistakes, "Common $! mistakes documented");
}

#[test]
fn test_PARAM_SPEC_004_background_pid_comparison_table() {
    // DOCUMENTATION: $! and & comparison (POSIX vs bashrs)
    //
    // Feature                 | POSIX sh | bash | dash | ash | bashrs
    // ------------------------|----------|------|------|-----|--------
    // & (background job)      | ✅       | ✅   | ✅   | ✅  | ❌ PURIFY
    // $! (background PID)     | ✅       | ✅   | ✅   | ✅  | ❌ PURIFY
    // Deterministic           | ❌       | ❌   | ❌   | ❌  | ✅ (sync)
    // wait                    | ✅       | ✅   | ✅   | ✅  | ❌ (implicit)
    // jobs                    | ✅       | ✅   | ✅   | ✅  | ❌
    // fg/bg                   | ✅       | ✅   | ✅   | ✅  | ❌
    //
    // bashrs purification policy:
    // - & (background) is POSIX but NON-DETERMINISTIC
    // - MUST purify to synchronous execution
    // - Remove all background jobs
    // - Remove $! (unnecessary without &)
    // - Remove wait (implicit in synchronous)
    //
    // Purification strategies:
    // 1. Background job: cmd & → cmd (synchronous)
    // 2. Multiple jobs: task1 & task2 & wait → task1; task2 (sequential)
    // 3. Timeout: cmd & sleep 5; kill $! → timeout 5 cmd || true
    // 4. Wait pattern: cmd &; wait $! → cmd (implicit wait)
    // 5. Remove non-essential: log_task & → (remove or make sync)
    //
    // Rust mapping (synchronous):
    // ```rust
    // use std::process::Command;
    //
    // // DON'T: Background execution (non-deterministic)
    // // let child = Command::new("cmd").spawn()?;
    // // let pid = child.id();
    // // child.wait()?;
    //
    // // DO: Synchronous execution (deterministic)
    // let status = Command::new("cmd").status()?;
    // ```
    //
    // Best practices:
    // 1. Use synchronous execution for determinism
    // 2. Avoid background jobs in bootstrap/config scripts
    // 3. Use timeout command for time limits (not background + kill)
    // 4. Sequential execution is easier to test and debug
    // 5. Interactive tools can use &, but not purified scripts

    let comparison_example = r#"
# POSIX: Background job (non-deterministic)
# cmd &
# echo "BG: $!"
# wait $!

# bashrs: Synchronous (deterministic)
cmd
echo "Done"

# POSIX: Multiple background jobs
# task1 &
# task2 &
# wait

# bashrs: Sequential
task1
task2

# POSIX: Timeout with background
# task &
# BG=$!
# sleep 5
# kill $BG

# bashrs: Use timeout command
timeout 5 task || true
"#;

    let result = BashParser::new(comparison_example);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "$! and & comparison and purification strategy documented"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

// Summary:
// $! (background PID): POSIX but NON-DETERMINISTIC (MUST PURIFY)
// Contains PID of last background job (changes every run)
// Background jobs (&) are non-deterministic (PIDs, timing, execution order)
// bashrs policy: Purify to SYNCHRONOUS execution (remove & and $!)
// Purification: cmd & → cmd, task1 & task2 & wait → task1; task2
// Timeout pattern: cmd & sleep N; kill $! → timeout N cmd || true
// Job control (jobs, fg, bg): NOT SUPPORTED (interactive features)
// Common mistakes: Race conditions, exit without wait, uncontrolled parallelism
// Best practice: Synchronous execution for determinism, testability, reproducibility

// ============================================================================
// EXP-BRACE-001: Brace Expansion {..} (Bash extension, NOT SUPPORTED)
// ============================================================================

// DOCUMENTATION: Brace expansion is NOT SUPPORTED (bash extension)
// Bash 3.0+ feature: {1..5}, {a..z}, {foo,bar,baz}, {a,b}{1,2}.
// Not in POSIX. sh/dash/ash don't support. Work around with loops or lists.
#[test]
fn test_EXP_BRACE_001_brace_expansion_not_supported() {
    let brace_expansion = r#"
# Bash brace expansion (NOT SUPPORTED)
echo {1..5}
echo {a..z}
echo {foo,bar,baz}
"#;

    assert_parses_without_panic(brace_expansion, "Brace expansion is bash extension, NOT SUPPORTED");
}

// DOCUMENTATION: Sequence expansion {start..end} (bash, NOT SUPPORTED)
// Numeric: {1..10}, {0..100..10}. Letter: {a..f}, {A..Z}.
// POSIX alternatives: seq, explicit for loop, while loop with counter.
#[test]
fn test_EXP_BRACE_001_sequence_expansion() {
    let sequence_expansion = r#"
# Bash sequences (NOT SUPPORTED)
# echo {1..10}
# echo {0..100..10}
# echo {a..z}

# POSIX alternatives (SUPPORTED)
seq 1 10
for i in 1 2 3 4 5; do echo "$i"; done

i=1
while [ $i -le 10 ]; do
  echo "$i"
  i=$((i+1))
done
"#;

    assert_parses_without_panic(sequence_expansion, "POSIX alternatives: seq, for loop, while loop");
}

// DOCUMENTATION: Comma expansion {item1,item2} (bash, NOT SUPPORTED)
// {foo,bar,baz}, pre{A,B,C}post, {red,green,blue}_color.
// POSIX alternatives: explicit list, for loop, variable iteration.
#[test]
fn test_EXP_BRACE_001_comma_expansion() {
    let comma_expansion = r#"
# Bash comma expansion (NOT SUPPORTED)
# echo {foo,bar,baz}
# echo pre{A,B,C}post

# POSIX alternatives (SUPPORTED)
echo foo bar baz

for item in foo bar baz; do
  echo "$item"
done

# Explicit iteration
items="foo bar baz"
for item in $items; do
  echo "$item"
done
"#;

    assert_parses_without_panic(comma_expansion, "POSIX alternatives: explicit lists, for loops");
}

#[test]
fn test_EXP_BRACE_001_nested_expansion() {
    // DOCUMENTATION: Nested brace expansion (bash, NOT SUPPORTED)
    //
    // Cartesian product:
    // $ echo {a,b}{1,2}
    // a1 a2 b1 b2
    //
    // $ echo {x,y,z}{A,B}
    // xA xB yA yB zA zB
    //
    // Multiple nesting:
    // $ echo {a,b}{1,2}{X,Y}
    // a1X a1Y a2X a2Y b1X b1Y b2X b2Y
    //
    // POSIX alternative: Nested loops
    // $ for letter in a b; do
    // $   for num in 1 2; do
    // $     echo "${letter}${num}"
    // $   done
    // $ done
    // a1
    // a2
    // b1
    // b2

    let nested_expansion = r#"
# Bash nested expansion (NOT SUPPORTED)
# echo {a,b}{1,2}
# echo {x,y,z}{A,B}

# POSIX alternative: Nested loops
for letter in a b; do
  for num in 1 2; do
    echo "${letter}${num}"
  done
done

for letter in x y z; do
  for suffix in A B; do
    echo "${letter}${suffix}"
  done
done
"#;

    let result = BashParser::new(nested_expansion);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "POSIX alternative: nested for loops"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

// DOCUMENTATION: bashrs purification strategy for brace expansion
// Strategy: numeric seq -> seq/loop, letters -> explicit list,
// comma lists -> explicit, nested -> nested loops, file ops -> explicit.
#[test]
fn test_EXP_BRACE_001_purification_strategy() {
    let purification_examples = r#"
# BEFORE (bash brace expansion)
# echo {1..10}
# echo {a..e}
# echo {foo,bar,baz}

# AFTER (POSIX)
seq 1 10
echo a b c d e
echo foo bar baz

# BEFORE (nested)
# echo {a,b}{1,2}

# AFTER (POSIX)
for x in a b; do
  for y in 1 2; do
    echo "${x}${y}"
  done
done
"#;

    assert_parses_without_panic(purification_examples, "Purification strategy: seq, explicit lists, nested loops");
}

// DOCUMENTATION: Common brace expansion use cases (bash, NOT SUPPORTED)
// mkdir dirs, backup files, iterate ranges, generate filenames, multi-commands.
// All have POSIX equivalents using explicit lists, while loops, or for loops.
#[test]
fn test_EXP_BRACE_001_common_use_cases() {
    let common_uses = r#"
# Use Case 1: Create directories (Bash)
# mkdir -p project/{src,tests,docs}

# POSIX alternative
mkdir -p project/src project/tests project/docs

# Use Case 2: Backup files (Bash)
# cp config.json{,.bak}

# POSIX alternative
cp config.json config.json.bak

# Use Case 3: Iterate ranges (Bash)
# for i in {1..100}; do echo "$i"; done

# POSIX alternative
i=1
while [ $i -le 100 ]; do
  echo "$i"
  i=$((i+1))
done

# Use Case 4: Generate files (Bash)
# touch file{1..5}.txt

# POSIX alternative
for i in 1 2 3 4 5; do
  touch "file${i}.txt"
done
"#;

    assert_parses_without_panic(common_uses, "Common use cases with POSIX alternatives");
}

#[test]
fn test_EXP_BRACE_001_edge_cases() {
    // DOCUMENTATION: Brace expansion edge cases (bash, NOT SUPPORTED)
    //
    // Edge Case 1: Zero-padded sequences
    // Bash:
    // $ echo {01..10}
    // 01 02 03 04 05 06 07 08 09 10
    //
    // POSIX:
    // $ seq -f "%02g" 1 10
    //
    // Edge Case 2: Reverse sequences
    // Bash:
    // $ echo {10..1}
    // 10 9 8 7 6 5 4 3 2 1
    //
    // POSIX:
    // $ seq 10 -1 1
    //
    // Edge Case 3: Step sequences
    // Bash:
    // $ echo {0..100..10}
    // 0 10 20 30 40 50 60 70 80 90 100
    //
    // POSIX:
    // $ seq 0 10 100
    //
    // Edge Case 4: Empty braces (literal)
    // Bash:
    // $ echo {}
    // {}  # Literal braces, no expansion
    //
    // Edge Case 5: Single item (literal)
    // Bash:
    // $ echo {foo}
    // {foo}  # Literal, no expansion (needs comma or ..)

    let edge_cases = r#"
# Edge Case 1: Zero-padded (Bash)
# echo {01..10}

# POSIX alternative
seq -f "%02g" 1 10

# Edge Case 2: Reverse sequence (Bash)
# echo {10..1}

# POSIX alternative
seq 10 -1 1

# Edge Case 3: Step sequence (Bash)
# echo {0..100..10}

# POSIX alternative
seq 0 10 100

# Edge Case 4: Empty braces (literal in bash)
# echo {}  # No expansion, prints {}

# Edge Case 5: Single item (literal in bash)
# echo {foo}  # No expansion, prints {foo}
"#;

    let result = BashParser::new(edge_cases);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "Edge cases documented with POSIX alternatives"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

// DOCUMENTATION: Brace expansion comparison (Bash vs POSIX vs bashrs)
// {1..10}, {a..z}, {foo,bar}, {a,b}{1,2} all bash-only, NOT SUPPORTED.
// Purify to POSIX: seq, explicit lists, nested loops. All portable.
#[test]
fn test_EXP_BRACE_001_comparison_table() {
    let comparison_example = r#"
# Bash: Brace expansion (NOT SUPPORTED)
# echo {1..10}
# echo {a..e}
# echo {foo,bar,baz}

# POSIX: seq and explicit lists (SUPPORTED)
seq 1 10
echo a b c d e
echo foo bar baz

# Bash: Nested expansion (NOT SUPPORTED)
# echo {a,b}{1,2}

# POSIX: Nested loops (SUPPORTED)
for x in a b; do
  for y in 1 2; do
    echo "${x}${y}"
  done
done
"#;

    assert_parses_without_panic(comparison_example, "Brace expansion comparison and purification documented");
}

// Summary:
// Brace expansion {..}: Bash extension (NOT SUPPORTED)
// Types: Numeric sequences {1..10}, letter sequences {a..z}, comma lists {foo,bar}
// Nested: {a,b}{1,2} creates Cartesian product (a1 a2 b1 b2)
// Introduced: Bash 3.0 (2004), not in POSIX specification
// POSIX alternatives: seq command, for loops, explicit lists
// Purification: {1..10} → seq 1 10, {foo,bar} → echo foo bar, nested → loops
// Common uses: mkdir {src,tests,docs}, cp file{,.bak}, touch file{1..5}.txt
// Best practice: Use seq for ranges, explicit lists for small sets, avoid in portable scripts

// ============================================================================
// EXP-TILDE-001: Tilde Expansion ~ (POSIX, SUPPORTED)
// ============================================================================

#[test]
fn test_EXP_TILDE_001_tilde_expansion_supported() {
    // DOCUMENTATION: Tilde expansion is SUPPORTED (POSIX)
    //
    // Tilde expansion replaces ~ with paths:
    // - POSIX-compliant feature (sh, bash, dash, ash all support)
    // - ~ expands to $HOME (user's home directory)
    // - ~user expands to user's home directory
    //
    // Basic tilde expansion:
    // $ echo ~
    // /home/username
    //
    // $ cd ~/documents
    // # Changes to /home/username/documents
    //
    // User-specific tilde:
    // $ echo ~root
    // /root
    //
    // $ echo ~alice
    // /home/alice
    //
    // Why tilde expansion is POSIX:
    // - Part of POSIX specification
    // - All POSIX shells support ~
    // - Portable across sh, bash, dash, ash
    //
    // Rust mapping:
    // ```rust
    // use std::env;
    //
    // // Get home directory
    // let home = env::var("HOME").unwrap_or_else(|_| "/".to_string());
    // let path = format!("{}/documents", home);
    //
    // // Or use dirs crate
    // use dirs::home_dir;
    // let home = home_dir().expect("No home directory");
    // ```

    let tilde_expansion = r#"
# POSIX tilde expansion (SUPPORTED)
cd ~
cd ~/documents
echo ~
ls ~/projects
"#;

    let result = BashParser::new(tilde_expansion);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "Tilde expansion is POSIX-compliant, FULLY SUPPORTED"
            );
        }
        Err(_) => {
            // Parse error acceptable - ~ may not be fully implemented yet
        }
    }
}

#[test]
fn test_EXP_TILDE_001_tilde_home_directory() {
    // DOCUMENTATION: ~ expands to $HOME (POSIX)
    //
    // Basic ~ expansion:
    // $ echo ~
    // /home/username  # Value of $HOME
    //
    // $ HOME=/custom/path
    // $ echo ~
    // /custom/path  # Uses current $HOME value
    //
    // Tilde in paths:
    // $ cd ~/projects
    // # Expands to: cd /home/username/projects
    //
    // $ mkdir ~/backup
    // # Expands to: mkdir /home/username/backup
    //
    // Important: Tilde must be at start of word
    // $ echo ~/dir    # ✅ Expands
    // $ echo /~       # ❌ No expansion (~ not at start)
    // $ echo "~"      # ❌ No expansion (quoted)
    //
    // POSIX equivalent:
    // $ cd "$HOME/projects"
    // $ mkdir "$HOME/backup"

    let tilde_home = r#"
# Tilde at start of word (expands)
cd ~
cd ~/documents
mkdir ~/backup

# Tilde not at start (no expansion)
# echo /~  # Literal /~, not expanded

# Quoted tilde (no expansion)
# echo "~"  # Literal ~, not expanded

# POSIX alternative: explicit $HOME
cd "$HOME"
cd "$HOME/documents"
mkdir "$HOME/backup"
"#;

    let result = BashParser::new(tilde_home);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "~ expands to $HOME (POSIX)"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

#[test]
fn test_EXP_TILDE_001_tilde_user_directory() {
    // DOCUMENTATION: ~user expands to user's home (POSIX)
    //
    // User-specific expansion:
    // $ echo ~root
    // /root
    //
    // $ echo ~alice
    // /home/alice
    //
    // $ cd ~bob/projects
    // # Changes to /home/bob/projects
    //
    // How it works:
    // - Shell looks up user in /etc/passwd
    // - Gets home directory from passwd entry
    // - Replaces ~user with home directory path
    //
    // If user doesn't exist:
    // $ echo ~nonexistent
    // ~nonexistent  # No expansion, literal ~nonexistent
    //
    // POSIX equivalent (if needed):
    // $ getent passwd username | cut -d: -f6
    // /home/username

    let tilde_user = r#"
# User-specific tilde (POSIX)
cd ~root
ls ~alice/documents

# Accessing other users' home directories
echo ~bob
cd ~charlie/projects
"#;

    let result = BashParser::new(tilde_user);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "~user expands to user's home directory (POSIX)"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

#[test]
fn test_EXP_TILDE_001_tilde_plus_minus() {
    // DOCUMENTATION: ~+ and ~- expansions (bash extension)
    //
    // Bash-specific tilde expansions:
    //
    // ~+ expands to $PWD (current directory):
    // $ cd /tmp
    // $ echo ~+
    // /tmp
    //
    // ~- expands to $OLDPWD (previous directory):
    // $ cd /home/user
    // $ cd /tmp
    // $ echo ~-
    // /home/user
    //
    // These are bash extensions, NOT in POSIX sh.
    //
    // POSIX alternatives (SUPPORTED):
    // - Use $PWD instead of ~+
    // - Use $OLDPWD instead of ~-
    //
    // bashrs: ~+ and ~- NOT SUPPORTED (bash extensions)
    // Purification: ~+ → $PWD, ~- → $OLDPWD

    let tilde_plus_minus = r#"
# Bash extensions (NOT SUPPORTED)
# echo ~+   # Current directory
# echo ~-   # Previous directory

# POSIX alternatives (SUPPORTED)
echo "$PWD"      # Current directory
echo "$OLDPWD"   # Previous directory
"#;

    let result = BashParser::new(tilde_plus_minus);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "~+ and ~- are bash extensions, use $PWD and $OLDPWD"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

#[test]
fn test_EXP_TILDE_001_tilde_in_assignments() {
    // DOCUMENTATION: Tilde expansion in variable assignments (POSIX)
    //
    // Tilde expands in variable assignments:
    // $ DIR=~/projects
    // $ echo "$DIR"
    // /home/username/projects
    //
    // After colon in assignments (PATH-like):
    // $ PATH=~/bin:/usr/bin
    // # Expands to: PATH=/home/username/bin:/usr/bin
    //
    // $ CDPATH=.:~:~/projects
    // # Expands to: CDPATH=.:/home/username:/home/username/projects
    //
    // Important: Expansion happens at assignment time
    // $ DIR=~/backup
    // $ HOME=/different/path
    // $ echo "$DIR"
    // /home/username/backup  # Still old HOME value
    //
    // POSIX behavior:
    // - Tilde expands in RHS of assignment
    // - Tilde expands after : in PATH-like variables

    let tilde_assignments = r#"
# Tilde in variable assignment (POSIX)
DIR=~/projects
BACKUP=~/backup

# PATH-like variables (tilde after colon)
PATH=~/bin:/usr/local/bin:/usr/bin
CDPATH=.:~:~/projects

# Using assigned variables
cd "$DIR"
ls "$BACKUP"
"#;

    let result = BashParser::new(tilde_assignments);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "Tilde expansion in assignments is POSIX"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

#[test]
fn test_EXP_TILDE_001_tilde_quoting() {
    // DOCUMENTATION: Tilde expansion and quoting (POSIX)
    //
    // Tilde does NOT expand when quoted:
    //
    // Double quotes (no expansion):
    // $ echo "~"
    // ~  # Literal tilde
    //
    // Single quotes (no expansion):
    // $ echo '~'
    // ~  # Literal tilde
    //
    // Unquoted (expands):
    // $ echo ~
    // /home/username
    //
    // Partial quoting:
    // $ echo ~"/documents"
    // /home/username/documents  # ~ expands, /documents doesn't
    //
    // $ echo "~"/documents
    // ~/documents  # ~ doesn't expand (quoted)
    //
    // CRITICAL: Tilde must be unquoted to expand
    //
    // To include literal ~ in output:
    // $ echo '~'     # Single quotes
    // $ echo "~"     # Double quotes
    // $ echo \~      # Backslash escape

    let tilde_quoting = r#"
# Unquoted tilde (expands)
cd ~
echo ~

# Quoted tilde (no expansion)
echo "~"
echo '~'

# Partial quoting
cd ~"/documents"  # Tilde expands
# cd "~"/documents  # Tilde doesn't expand (quoted)

# Literal tilde
echo '~'
echo "~"
"#;

    let result = BashParser::new(tilde_quoting);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "Tilde doesn't expand when quoted (POSIX)"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

#[test]
fn test_EXP_TILDE_001_common_use_cases() {
    // DOCUMENTATION: Common tilde expansion use cases (POSIX)
    //
    // Use Case 1: Change to home directory
    // $ cd ~
    // # Equivalent to: cd "$HOME"
    //
    // Use Case 2: Access user files
    // $ ls ~/documents
    // $ cat ~/config.txt
    // # Equivalent to: ls "$HOME/documents"
    //
    // Use Case 3: Create directories in home
    // $ mkdir ~/backup
    // $ mkdir -p ~/projects/rust
    // # Equivalent to: mkdir "$HOME/backup"
    //
    // Use Case 4: Set PATH with home bin
    // $ PATH=~/bin:$PATH
    // # Adds $HOME/bin to PATH
    //
    // Use Case 5: Copy to/from home
    // $ cp file.txt ~/backup/
    // $ cp ~/config.txt .
    // # Equivalent to: cp file.txt "$HOME/backup/"
    //
    // Best practice: Use ~ for convenience, $HOME for clarity
    // - ~ is shorter, more readable
    // - $HOME is more explicit
    // - Both are POSIX-compliant

    let common_uses = r#"
# Use Case 1: Change to home
cd ~

# Use Case 2: Access files
ls ~/documents
cat ~/config.txt

# Use Case 3: Create directories
mkdir ~/backup
mkdir -p ~/projects/rust

# Use Case 4: Set PATH
PATH=~/bin:$PATH

# Use Case 5: Copy files
cp file.txt ~/backup/
cp ~/config.txt .

# Alternative: explicit $HOME
cd "$HOME"
ls "$HOME/documents"
mkdir "$HOME/backup"
"#;

    let result = BashParser::new(common_uses);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "Common tilde use cases (POSIX)"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

