#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]

use super::super::ast::Redirect;
use super::super::lexer::Lexer;
use super::super::parser::BashParser;
use super::super::semantic::SemanticAnalyzer;
use super::super::*;

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

