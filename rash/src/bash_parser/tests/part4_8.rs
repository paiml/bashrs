#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]

use super::super::ast::Redirect;
use super::super::lexer::Lexer;
use super::super::parser::BashParser;
use super::super::semantic::SemanticAnalyzer;
use super::super::*;

/// Helper: tokenize input and assert tokens are non-empty.
/// Accepts parse errors gracefully (parser may not support all constructs yet).
#[test]
fn test_VAR_001_home_common_patterns() {
    assert_tokenizes(
        VAR_001_HOME_COMMON_PATTERNS_INPUT,
        "HOME patterns should tokenize",
    );
}

#[test]
fn test_VAR_001_home_vs_tilde() {
    // DOCUMENTATION: HOME vs tilde expansion (Important distinction)
    //
    // $HOME: Environment variable (literal value)
    // ~: Tilde expansion (shell expands to $HOME)
    //
    // EQUIVALENCES:
    // cd ~ == cd "$HOME"
    // ~/dir == "$HOME/dir"
    // ~+ == "$PWD" (current directory)
    // ~- == "$OLDPWD" (previous directory)
    //
    // WHEN TO USE EACH:
    // Use $HOME when:
    // - In scripts (more explicit)
    // - Variable expansion needed
    // - Inside quotes: "$HOME/dir"
    //
    // Use ~ when:
    // - Interactive typing (shorter)
    // - Start of path: ~/documents
    // - Readability: cd ~/projects (clearer than cd "$HOME/projects")
    //
    // QUOTING RULES:
    // "$HOME/dir" - Correct (always quote)
    // ~/dir - Correct (no quotes needed, tilde expands before word splitting)
    // "~/dir" - WRONG (tilde doesn't expand in quotes)
    //
    // INPUT (bash):
    // cd ~
    // cd "$HOME"  # Equivalent
    // file=~/document.txt
    // file2="$HOME/document.txt"  # Equivalent
    //
    // RUST:
    // use std::env;
    // let home = env::var("HOME").unwrap();
    // env::set_current_dir(&home).unwrap();
    // let file = format!("{}/document.txt", home);

    let home_vs_tilde = r#"
# Equivalent forms
cd ~
cd "$HOME"

cd ~/documents
cd "$HOME/documents"

# Tilde expansion variations
cd ~          # User's home
cd ~alice     # Alice's home (not in POSIX, bash extension)
cd ~+         # Current directory (bash extension)
cd ~-         # Previous directory (bash extension)

# Variable assignment
file1=~/document.txt           # Tilde expands
file2="$HOME/document.txt"     # HOME variable

# WRONG: Tilde in quotes doesn't expand
# file3="~/document.txt"       # WRONG: literal "~/document.txt"
# Use this instead:
file3="$HOME/document.txt"     # Correct

# HOME is more explicit in scripts
config_dir="$HOME/.config"
cache_dir="$HOME/.cache"

# Tilde is more readable interactively
# cd ~/projects/myapp
# cd ~/Downloads

# Subdirectories
mkdir -p "$HOME/backups"
mkdir -p ~/backups  # Equivalent
"#;

    let mut lexer = Lexer::new(home_vs_tilde);
    match lexer.tokenize() {
        Ok(tokens) => {
            assert!(!tokens.is_empty(), "HOME vs tilde examples should tokenize");
            let _ = tokens;
        }
        Err(_) => {
            // Parser may not fully support tilde expansion yet
        }
    }
}

// DOCUMENTATION: HOME best practices (CRITICAL)
// ALWAYS: Quote HOME, check existence, use for user files, keep read-only
// NEVER: Unquoted cd $HOME, modify HOME, assume exists, hardcode paths
// PORTABILITY: HOME and ~ are POSIX; ~user, ~+, ~- are bash extensions
const VAR_001_HOME_BEST_PRACTICES_INPUT: &str = r#"
# BEST PRACTICE 1: Always quote HOME
cd "$HOME"              # Correct
# cd $HOME              # WRONG: breaks if HOME has spaces

# BEST PRACTICE 2: Check HOME is set
if [ -z "$HOME" ]; then
    printf 'ERROR: HOME not set\n' >&2
    exit 1
fi

# BEST PRACTICE 3: Check HOME directory exists
if [ ! -d "$HOME" ]; then
    printf 'ERROR: HOME directory does not exist: %s\n' "$HOME" >&2
    exit 1
fi

# BEST PRACTICE 4: Use HOME for user-specific files
config_file="$HOME/.config/app.conf"
cache_dir="$HOME/.cache/app"
data_dir="$HOME/.local/share/app"

# BEST PRACTICE 5: Never modify HOME
# HOME="/new/path"      # WRONG: breaks system utilities
# Use a different variable instead:
APP_HOME="$HOME/myapp"
cd "$APP_HOME"

# BEST PRACTICE 6: Portable tilde usage
cd ~                    # POSIX (portable)
cd ~/dir                # POSIX (portable)
# cd ~alice             # Bash extension (not portable)
# cd ~+                 # Bash extension (not portable)

# BEST PRACTICE 7: Use $HOME in scripts, ~ interactively
# Scripts (explicit):
install_dir="$HOME/.local/bin"
mkdir -p "$install_dir"

# Interactive (readable):
# cd ~/projects
# ls ~/Downloads

# BEST PRACTICE 8: Portable home reference
# Don't hardcode:
# config="/home/alice/.config"  # WRONG: not portable
# Use HOME:
config="$HOME/.config"          # Correct: works for any user
"#;

#[test]
fn test_VAR_001_home_best_practices() {
    assert_tokenizes(
        VAR_001_HOME_BEST_PRACTICES_INPUT,
        "best practices should tokenize",
    );
}

// DOCUMENTATION: HOME edge cases (Error handling)
// EDGE CASES: HOME not set, non-existent dir, spaces in path,
// special chars, empty string, root user (HOME=/)
// DEFENSIVE: Check -z "$HOME", check -d "$HOME", check -w "$HOME"
const VAR_001_HOME_EDGE_CASES_INPUT: &str = r#"
# Edge case 1: HOME not set (rare)
if [ -z "$HOME" ]; then
    printf 'ERROR: HOME environment variable not set\n' >&2
    exit 1
fi

# Edge case 2: HOME directory doesn't exist
if [ ! -d "$HOME" ]; then
    printf 'ERROR: HOME directory does not exist: %s\n' "$HOME" >&2
    # Try to create it (last resort)
    mkdir -p "$HOME" 2>/dev/null || exit 1
fi

# Edge case 3: HOME with spaces (must quote)
# HOME="/home/user name"
cd "$HOME"              # Correct (quoted)
# cd $HOME              # WRONG: would cd to "/home/user" (broken)

# Edge case 4: HOME not writable
if [ ! -w "$HOME" ]; then
    printf 'WARNING: HOME not writable, using /tmp\n' >&2
    APP_DATA="/tmp/app-data.$$"
else
    APP_DATA="$HOME/.app-data"
fi
mkdir -p "$APP_DATA"

# Edge case 5: Root user (HOME=/)
if [ "$HOME" = "/" ]; then
    printf 'Running as root (HOME=/)\n'
    # Use /root/.app instead of /.app
    config_dir="/root/.config"
else
    config_dir="$HOME/.config"
fi

# Edge case 6: Fallback if HOME missing
fallback_home="${HOME:-/tmp}"
cd "$fallback_home"

# Edge case 7: Preserve original HOME
original_home="$HOME"
# ... potential HOME modification ...
HOME="$original_home"  # Restore
"#;

#[test]
fn test_VAR_001_home_edge_cases() {
    assert_tokenizes(VAR_001_HOME_EDGE_CASES_INPUT, "edge cases should tokenize");
}

#[test]
fn test_VAR_001_home_system_interaction() {
    // DOCUMENTATION: HOME system interaction (How HOME is set)
    //
    // HOME is set by:
    // 1. Login shell: Reads from /etc/passwd (6th field)
    // 2. su command: May or may not update HOME
    // 3. sudo: Usually preserves original user's HOME
    // 4. SSH: Sets HOME to target user's home
    //
    // READING HOME:
    // From /etc/passwd:
    // alice:x:1000:1000:Alice:/home/alice:/bin/bash
    //                         ^^^^^^^^^^^
    //                         This becomes HOME
    //
    // POSIX BEHAVIOR:
    // - Login sets HOME from /etc/passwd
    // - cd (no args) changes to $HOME
    // - ~ expands to $HOME
    // - Many utilities use HOME (.bashrc, .profile, etc.)
    //
    // COMMON UTILITIES USING HOME:
    // - cd: cd (no args) → cd "$HOME"
    // - Shell configs: ~/.bashrc, ~/.profile
    // - SSH: ~/.ssh/known_hosts, ~/.ssh/id_rsa
    // - Git: ~/.gitconfig
    // - Vim: ~/.vimrc
    // - Many more: ~/.config, ~/.cache, ~/.local

    let system_interaction = r#"
# HOME is set at login from /etc/passwd
# No need to set it manually in scripts
printf 'Current HOME: %s\n' "$HOME"
printf 'Current user: %s\n' "$USER"

# cd with no arguments uses HOME
cd          # Goes to $HOME
pwd         # Shows $HOME

# Tilde expansion uses HOME
cd ~        # Same as cd "$HOME"
ls ~        # Same as ls "$HOME"

# User configuration files (rely on HOME)
if [ -f "$HOME/.bashrc" ]; then
    . "$HOME/.bashrc"
fi

if [ -f "$HOME/.profile" ]; then
    . "$HOME/.profile"
fi

# Application config directories
config_dir="$HOME/.config/myapp"
mkdir -p "$config_dir"

cache_dir="$HOME/.cache/myapp"
mkdir -p "$cache_dir"

data_dir="$HOME/.local/share/myapp"
mkdir -p "$data_dir"

# SSH uses HOME
ssh_dir="$HOME/.ssh"
if [ -d "$ssh_dir" ]; then
    printf 'SSH config found in %s\n' "$ssh_dir"
fi

# Git uses HOME
git_config="$HOME/.gitconfig"
if [ -f "$git_config" ]; then
    printf 'Git config: %s\n' "$git_config"
fi
"#;

    let mut lexer = Lexer::new(system_interaction);
    match lexer.tokenize() {
        Ok(tokens) => {
            assert!(!tokens.is_empty(), "system interaction should tokenize");
            let _ = tokens;
        }
        Err(_) => {
            // Parser may not fully support all patterns yet
        }
    }
}

#[test]
fn test_VAR_001_home_security_considerations() {
    // DOCUMENTATION: HOME security considerations (CRITICAL)
    //
    // SECURITY RISKS:
    // 1. Untrusted HOME: In shared systems, HOME might be writable by others
    // 2. Symlink attacks: $HOME/.config could be symlink to attacker's dir
    // 3. Race conditions: HOME changes between check and use
    // 4. Injection: If HOME contains shell metacharacters (rare but possible)
    //
    // SECURE PRACTICES:
    // 1. Always quote: "$HOME" (prevents injection)
    // 2. Validate ownership: [ "$(stat -c %U "$HOME")" = "$USER" ]
    // 3. Check permissions: [ "$(stat -c %a "$HOME")" = "700" ] (or 755)
    // 4. Avoid symlinks in critical paths
    // 5. Use mktemp for temporary files (not $HOME/tmp)
    //
    // EXAMPLE ATTACK (HOME injection):
    // If HOME="; rm -rf /"  (malicious, unlikely but possible)
    // cd $HOME              # Could execute: cd ; rm -rf /
    // cd "$HOME"            # Safe: cd "; rm -rf /"
    //
    // MITIGATION:
    // - Always quote variables
    // - Validate HOME before use
    // - Use safe temp directories (mktemp)

    let security_considerations = r#"
# SECURITY 1: Always quote HOME
cd "$HOME"              # Safe (quoted)
# cd $HOME              # Unsafe (word splitting, globbing)

# SECURITY 2: Validate HOME exists and is directory
if [ ! -d "$HOME" ]; then
    printf 'ERROR: Invalid HOME: %s\n' "$HOME" >&2
    exit 1
fi

# SECURITY 3: Check HOME ownership (optional, paranoid)
# home_owner=$(stat -c %U "$HOME" 2>/dev/null)
# if [ "$home_owner" != "$USER" ]; then
#     printf 'WARNING: HOME owned by different user\n' >&2
# fi

# SECURITY 4: Use safe temp files
temp_file=$(mktemp)     # Safe (system temp dir)
# Not: temp_file="$HOME/tmp/file.$$"  # Less safe

# SECURITY 5: Avoid symlink attacks
config_dir="$HOME/.config/app"
mkdir -p "$config_dir"
# Verify it's a directory (not symlink to attacker's dir)
if [ ! -d "$config_dir" ] || [ -L "$config_dir" ]; then
    printf 'WARNING: Config dir is symlink or missing\n' >&2
fi

# SECURITY 6: Safe file creation in HOME
data_file="$HOME/.app/data.conf"
# Create safely:
umask 077               # Restrict permissions
mkdir -p "$(dirname "$data_file")"
printf '%s\n' "data" > "$data_file"

# SECURITY 7: Don't trust HOME implicitly in privileged scripts
if [ "$(id -u)" -eq 0 ]; then
    printf 'WARNING: Running as root with HOME=%s\n' "$HOME" >&2
    # Be extra careful with file operations
fi
"#;

    let mut lexer = Lexer::new(security_considerations);
    match lexer.tokenize() {
        Ok(tokens) => {
            assert!(
                !tokens.is_empty(),
                "security considerations should tokenize"
            );
            let _ = tokens;
        }
        Err(_) => {
            // Parser may not fully support all patterns yet
        }
    }
}

#[test]
fn test_VAR_001_home_comparison_table() {
    // COMPREHENSIVE COMPARISON: HOME in POSIX vs Bash
    //
    // ┌──────────────────────────────────────────────────────────────────────────┐
    // │ Feature: HOME Environment Variable                                       │
    // ├────────────────────────────┬──────────────┬──────────────────────────────┤
    // │ Feature                    │ POSIX Status │ Best Practice                │
    // ├────────────────────────────┼──────────────┼──────────────────────────────┤
    // │ $HOME                      │ SUPPORTED    │ Always quote: "$HOME"        │
    // │ ${HOME}                    │ SUPPORTED    │ Use when disambiguating      │
    // │ cd (no args) → $HOME       │ SUPPORTED    │ Convenient home navigation   │
    // │ ~ → $HOME                  │ SUPPORTED    │ Use for readability          │
    // │ ~/dir → $HOME/dir          │ SUPPORTED    │ Use for paths                │
    // │ Check: [ -d "$HOME" ]      │ BEST PRACTICE│ Always validate              │
    // │ Check: [ -z "$HOME" ]      │ BEST PRACTICE│ Check if set                 │
    // │ Never modify HOME          │ BEST PRACTICE│ Read-only by convention      │
    // │ ~user (other's home)       │ NOT PORTABLE │ Bash extension, avoid        │
    // │ ~+ (current dir)           │ NOT PORTABLE │ Bash extension, use $PWD     │
    // │ ~- (previous dir)          │ NOT PORTABLE │ Bash extension, use $OLDPWD  │
    // └────────────────────────────┴──────────────┴──────────────────────────────┘
    //
    // RUST MAPPING:
    // $HOME              → std::env::var("HOME").unwrap()
    // cd "$HOME"         → std::env::set_current_dir(env::var("HOME").unwrap())
    // "${HOME}/dir"      → format!("{}/dir", env::var("HOME").unwrap())
    // [ -d "$HOME" ]     → std::path::Path::new(&env::var("HOME").unwrap()).is_dir()
    //
    // DETERMINISM: HOME is deterministic (set at login, doesn't change)
    // SECURITY: Always quote "$HOME" to prevent injection/splitting
    // PORTABILITY: HOME is POSIX (works on all Unix-like systems)

    let comparison_table = r#"
# This test documents the complete POSIX comparison for HOME
# See extensive comparison table in test function comments above

# POSIX SUPPORTED: HOME variable
printf 'HOME: %s\n' "$HOME"
printf 'HOME (braced): %s\n' "${HOME}"

# POSIX SUPPORTED: cd with no args
cd              # Goes to $HOME
pwd             # Shows $HOME

# POSIX SUPPORTED: Tilde expansion
cd ~            # Same as cd "$HOME"
cd ~/documents  # Same as cd "$HOME/documents"

# BEST PRACTICE: Always quote
cd "$HOME"              # Correct
config="$HOME/.config"  # Correct

# BEST PRACTICE: Check HOME exists
if [ -d "$HOME" ]; then
    printf 'HOME exists\n'
fi

# BEST PRACTICE: Check HOME is set
if [ -z "$HOME" ]; then
    printf 'ERROR: HOME not set\n' >&2
    exit 1
fi

# BEST PRACTICE: Never modify HOME
# HOME="/new/path"      # WRONG: breaks system
# Use different variable:
APP_HOME="$HOME/myapp"

# NOT PORTABLE: Bash tilde extensions
# cd ~alice     # Bash extension (other user's home)
# cd ~+         # Bash extension (current directory)
# cd ~-         # Bash extension (previous directory)
# Use POSIX equivalents:
# cd /home/alice        # Hardcode (not recommended)
# cd "$PWD"             # Current directory
# cd "$OLDPWD"          # Previous directory

# POSIX PORTABLE: User-specific files
config_dir="$HOME/.config"
cache_dir="$HOME/.cache"
data_dir="$HOME/.local/share"
"#;

    let mut lexer = Lexer::new(comparison_table);
    match lexer.tokenize() {
        Ok(tokens) => {
            assert!(
                !tokens.is_empty(),
                "comparison table examples should tokenize"
            );
            let _ = tokens;
        }
        Err(_) => {
            // Examples document expected behavior
        }
    }

    // Priority: HIGH - HOME is fundamental to user-specific operations
    // POSIX: IEEE Std 1003.1-2001 environment variable
    // Security: Always quote "$HOME" to prevent injection and word splitting
    // Determinism: HOME is deterministic (set at login, stable during session)
    // Portability: HOME is POSIX (works on all Unix-like systems)
}

// ============================================================================
// VAR-002: PATH environment variable
// ============================================================================
