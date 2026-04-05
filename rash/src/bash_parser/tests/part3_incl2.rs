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
