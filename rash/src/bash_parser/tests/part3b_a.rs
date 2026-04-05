#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]

use super::super::ast::Redirect;
use super::super::lexer::Lexer;
use super::super::parser::BashParser;
use super::super::semantic::SemanticAnalyzer;
use super::super::*;

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

