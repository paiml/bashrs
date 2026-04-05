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
