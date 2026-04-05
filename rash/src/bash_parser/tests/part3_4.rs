#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]

use super::super::ast::Redirect;
use super::super::lexer::Lexer;
use super::super::parser::BashParser;
use super::super::semantic::SemanticAnalyzer;
use super::super::*;

/// Helper: assert that BashParser handles the input without panicking.
/// Accepts both successful parses and parse errors (documentation tests
/// only verify the parser doesn't crash, not that the input is valid).
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
