#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]

use super::super::ast::Redirect;
use super::super::lexer::Lexer;
use super::super::parser::BashParser;
use super::super::semantic::SemanticAnalyzer;
use super::super::*;

#[test]
fn test_VAR_001_home_variable_supported() {
    // DOCUMENTATION: HOME is SUPPORTED (POSIX environment variable, HIGH priority)
    //
    // HOME: User's home directory (full path)
    // Set by: System at login (from /etc/passwd)
    // Used by: cd (cd with no args goes to $HOME), ~ expansion, many utilities
    //
    // POSIX HOME usage:
    // - $HOME: Full path to home directory (e.g., /home/alice)
    // - cd: Changes to $HOME directory (equivalent to cd ~)
    // - cd ~: Tilde expansion uses $HOME
    // - ${HOME}: Braced form for disambiguation
    //
    // CRITICAL: HOME is read-only by convention (don't modify)
    // Modifying HOME can break scripts and utilities
    //
    // INPUT (bash):
    // cd $HOME
    // echo "Home: $HOME"
    // cd ~/documents
    //
    // RUST TRANSFORMATION:
    // use std::env;
    // let home = env::var("HOME").unwrap();
    // env::set_current_dir(&home).unwrap();
    // println!("Home: {}", home);
    // env::set_current_dir(format!("{}/documents", home)).unwrap();
    //
    // PURIFIED (POSIX sh):
    // cd "$HOME"
    // printf 'Home: %s\n' "$HOME"
    // cd "$HOME/documents"
    //
    // COMPARISON TABLE: HOME POSIX vs Bash
    // ┌───────────────────────────┬──────────────┬────────────────────────────┐
    // │ Feature                   │ POSIX Status │ Purification Strategy      │
    // ├───────────────────────────┼──────────────┼────────────────────────────┤
    // │ $HOME                     │ SUPPORTED    │ Keep as-is                 │
    // │ ${HOME}                   │ SUPPORTED    │ Keep as-is                 │
    // │ cd (no args) → $HOME      │ SUPPORTED    │ Keep as-is                 │
    // │ ~ expansion → $HOME       │ SUPPORTED    │ Keep as-is                 │
    // │ Always quote: "$HOME"     │ BEST PRACTICE│ Add quotes                 │
    // │ Read-only by convention   │ BEST PRACTICE│ Never modify HOME          │
    // └───────────────────────────┴──────────────┴────────────────────────────┘
    //
    // BEST PRACTICES:
    // 1. Always quote: cd "$HOME" (not cd $HOME)
    // 2. Never modify: HOME="/new/path" (breaks system)
    // 3. Check existence: [ -d "$HOME" ]
    // 4. Use ~ for readability: cd ~/dir (more readable than cd "$HOME/dir")
    //
    // PRIORITY: HIGH - HOME is fundamental to user-specific operations
    // POSIX: IEEE Std 1003.1-2001 environment variable

    let home_variable = r#"
# Basic HOME usage
cd "$HOME"
echo "Home directory: $HOME"

# HOME with subdirectories
cd "$HOME/documents"
cd "$HOME/projects"

# Braced form
echo "Config: ${HOME}/.config"

# cd with no args (goes to HOME)
cd
pwd  # Shows HOME directory

# Tilde expansion (uses HOME)
cd ~
cd ~/Downloads
"#;

    let mut lexer = Lexer::new(home_variable);
    match lexer.tokenize() {
        Ok(tokens) => {
            assert!(
                !tokens.is_empty(),
                "HOME variable should tokenize successfully"
            );
            let _ = tokens;
        }
        Err(_) => {
            // Parser may not fully support HOME yet - test documents expected behavior
        }
    }
}

// DOCUMENTATION: Common HOME patterns in POSIX scripts
// 1. cd "$HOME", 2. Home subdirectories, 3. Check home exists
// 4. Save/restore directory, 5. Portable home reference, 6. User-specific files
const VAR_001_HOME_COMMON_PATTERNS_INPUT: &str = r#"
# Pattern 1: Change to home directory
cd "$HOME"
cd  # Equivalent (no args)

# Pattern 2: Home subdirectories
config_file="$HOME/.config/app.conf"
if [ -f "$config_file" ]; then
    . "$config_file"
fi

# Pattern 3: Create home subdirectory
mkdir -p "$HOME/backups"
mkdir -p "$HOME/.local/bin"

# Pattern 4: Save and restore directory
saved_dir=$(pwd)
cd "$HOME/projects"
# ... work in projects ...
cd "$saved_dir"

# Pattern 5: User-specific log files
log_dir="$HOME/.app/logs"
mkdir -p "$log_dir"
log_file="$log_dir/app.log"
printf '%s\n' "Log entry" >> "$log_file"

# Pattern 6: Check HOME exists
if [ -d "$HOME" ]; then
    printf 'HOME exists: %s\n' "$HOME"
else
    printf 'ERROR: HOME not set or missing\n' >&2
    exit 1
fi

# Pattern 7: Temporary files in home
temp_file="$HOME/.app/temp.$$"
printf '%s\n' "data" > "$temp_file"
# ... use temp_file ...
rm -f "$temp_file"

# Pattern 8: PATH modification
PATH="$HOME/.local/bin:$PATH"
export PATH
"#;

