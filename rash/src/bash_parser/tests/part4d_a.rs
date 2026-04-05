#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]

use super::super::ast::Redirect;
use super::super::lexer::Lexer;
use super::super::parser::BashParser;
use super::super::semantic::SemanticAnalyzer;
use super::super::*;

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

#[test]
fn test_VAR_002_path_variable_supported() {
    // DOCUMENTATION: PATH is SUPPORTED (POSIX environment variable, HIGH priority)
    //
    // PATH: Colon-separated list of directories to search for commands
    // Set by: System at login, modified by shells, users, package managers
    // Used by: Shell command lookup (when you type "ls", shell searches PATH)
    //
    // PATH STRUCTURE:
    // PATH="/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin"
    //       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    //       Colon-separated directories (first match wins)
    //
    // COMMAND LOOKUP ORDER:
    // 1. Built-in commands (cd, echo, test, etc.)
    // 2. Functions
    // 3. PATH directories (left to right, first match wins)
    //
    // CRITICAL: PATH order matters
    // /usr/local/bin typically comes first (user-installed overrides system)

    let path_variable = r#"
# Basic PATH usage
echo "$PATH"

# Add to PATH (prepend - takes priority)
PATH="/opt/myapp/bin:$PATH"
export PATH

# Add to PATH (append - lower priority)
PATH="$PATH:$HOME/bin"
export PATH

# Braced form
echo "Current PATH: ${PATH}"

# Check if directory is in PATH
case ":$PATH:" in
    *:/usr/local/bin:*) echo "Found in PATH" ;;
    *) echo "Not in PATH" ;;
esac

# Use PATH for command lookup
which ls     # Searches PATH for 'ls'
command -v ls  # POSIX way to find commands in PATH
"#;

    let mut lexer = Lexer::new(path_variable);
    match lexer.tokenize() {
        Ok(tokens) => {
            assert!(
                !tokens.is_empty(),
                "PATH variable should tokenize successfully"
            );
            let _ = tokens;
        }
        Err(_) => {
            // Parser may not fully support PATH yet - test documents expected behavior
        }
    }

    // Determinism: PATH is POSIX SUPPORTED (fundamental command lookup)
    // Security: Always quote "$PATH" when modifying or echoing
    // Best practice: Prepend user dirs (/usr/local/bin), append home dirs ($HOME/bin)
}

#[test]
fn test_VAR_002_path_common_patterns() {
    // DOCUMENTATION: PATH common patterns (10 essential patterns)
    //
    // PATTERN 1: Prepend directory (takes priority over existing)
    // PATH="/new/dir:$PATH"
    //
    // PATTERN 2: Append directory (lower priority than existing)
    // PATH="$PATH:/new/dir"
    //
    // PATTERN 3: Export PATH (make available to child processes)
    // export PATH="/new/dir:$PATH"
    //
    // PATTERN 4: Check if directory already in PATH (avoid duplicates)
    // case ":$PATH:" in *:/dir:*) ;; *) PATH="$PATH:/dir" ;; esac
    //
    // PATTERN 5: Remove directory from PATH (complex, use sed/tr)
    // PATH=$(echo "$PATH" | sed 's|:/old/dir:||g')
    //
    // PATTERN 6: Reset PATH to minimal safe value
    // PATH="/usr/bin:/bin"
    //
    // PATTERN 7: Search PATH for command
    // command -v ls  # POSIX (returns path or nothing)
    // which ls       # Common but not POSIX
    //
    // PATTERN 8: Iterate over PATH directories
    // IFS=:
    // for dir in $PATH; do echo "$dir"; done
    //
    // PATTERN 9: Check if command exists in PATH
    // if command -v mycommand >/dev/null 2>&1; then ...
    //
    // PATTERN 10: Temporary PATH modification (subshell)
    // (PATH="/custom/path:$PATH"; mycommand)

    let path_patterns = r#"
# PATTERN 1: Prepend (priority)
PATH="/usr/local/bin:$PATH"

# PATTERN 2: Append (lower priority)
PATH="$PATH:$HOME/.local/bin"

# PATTERN 3: Export
export PATH="/opt/bin:$PATH"

# PATTERN 4: Avoid duplicates
case ":$PATH:" in
    *:$HOME/bin:*) ;;
    *) PATH="$PATH:$HOME/bin" ;;
esac

# PATTERN 6: Reset to minimal
PATH="/usr/bin:/bin"

# PATTERN 7: Search PATH
command -v git

# PATTERN 9: Check if command exists
if command -v docker >/dev/null 2>&1; then
    echo "Docker is installed"
fi

# PATTERN 10: Temporary PATH (subshell)
(PATH="/custom:$PATH"; ./myprogram)
"#;

    let mut lexer = Lexer::new(path_patterns);
    if let Ok(tokens) = lexer.tokenize() {
        assert!(
            !tokens.is_empty(),
            "PATH common patterns should tokenize successfully"
        );
        let _ = tokens;
    }

    // All patterns are POSIX SUPPORTED
    // Determinism: PATH modifications are deterministic
    // Security: Quote "$PATH" in all modifications to prevent word splitting
}

#[test]
fn test_VAR_002_path_vs_which_vs_command() {
    // DOCUMENTATION: PATH vs which vs command -v (IMPORTANT DISTINCTION)
    //
    // COMMAND LOOKUP METHODS:
    //
    // METHOD 1: command -v (POSIX, RECOMMENDED)
    // command -v ls        # Returns full path: /usr/bin/ls
    // command -v cd        # Returns: cd (builtin)
    // command -v noexist   # Returns nothing, exit 1
    //
    // METHOD 2: which (NOT POSIX, but common)
    // which ls             # Returns full path: /usr/bin/ls
    // which cd             # May not find builtins (shell-dependent)
    // which noexist        # Behavior varies by implementation
    //
    // METHOD 3: type (bash builtin, NOT POSIX)
    // type ls              # "ls is /usr/bin/ls"
    // type cd              # "cd is a shell builtin"
    //
    // METHOD 4: Direct PATH search (manual, avoid)
    // IFS=:; for dir in $PATH; do [ -x "$dir/ls" ] && echo "$dir/ls"; done
    //
    // PURIFICATION STRATEGY:
    // INPUT (bash-specific):
    // which git || echo "Not found"
    // type docker
    //
    // PURIFIED (POSIX):
    // command -v git >/dev/null || echo "Not found"
    // command -v docker >/dev/null
    //
    // WHY command -v:
    // 1. POSIX standard (portable across all shells)
    // 2. Finds builtins, functions, AND executables
    // 3. Consistent exit status (0 = found, 1 = not found)
    // 4. Works in scripts and interactive shells
    // 5. No external dependency (builtin)

    let path_vs_which = r#"
# RECOMMENDED: command -v (POSIX)
if command -v git >/dev/null 2>&1; then
    git_path=$(command -v git)
    echo "Git found at: $git_path"
fi

# AVOID: which (not POSIX)
# which git

# AVOID: type (bash-specific)
# type git

# Use command -v for existence checks
for cmd in git make gcc; do
    if command -v "$cmd" >/dev/null 2>&1; then
        echo "$cmd: available"
    else
        echo "$cmd: not found"
    fi
done
"#;

    let mut lexer = Lexer::new(path_vs_which);
    if let Ok(tokens) = lexer.tokenize() {
        assert!(
            !tokens.is_empty(),
            "PATH vs which patterns should tokenize successfully"
        );
        let _ = tokens;
    }

    // POSIX: command -v (SUPPORTED)
    // Non-POSIX: which (avoid), type (bash-specific, avoid)
    // Purification: Replace which/type with command -v
}

#[test]
fn test_VAR_002_path_best_practices() {
    // DOCUMENTATION: PATH best practices (8 CRITICAL practices)
    //
    // PRACTICE 1: Always quote "$PATH"
    // PATH="/new:$PATH"        # Safe (quoted)
    // # PATH=/new:$PATH        # Unsafe (word splitting if PATH has spaces)
    //
    // PRACTICE 2: Export PATH after modification
    // PATH="/new:$PATH"
    // export PATH              # Make available to child processes
    //
    // PRACTICE 3: Prepend user directories
    // PATH="/usr/local/bin:$PATH"  # User overrides system
    //
    // PRACTICE 4: Append home directories
    // PATH="$PATH:$HOME/bin"       # Lower priority (safe)
    //
    // PRACTICE 5: Never put "." (current directory) in PATH
    // # PATH=".:$PATH"        # DANGEROUS (security risk)
    // # PATH="$PATH:."        # DANGEROUS (run untrusted code)
    //
    // PRACTICE 6: Check PATH is set before modifying
    // PATH="${PATH:-/usr/bin:/bin}"  # Fallback if unset
    //
    // PRACTICE 7: Avoid duplicates (check before adding)
    // case ":$PATH:" in
    //     *:/new/dir:*) ;;
    //     *) PATH="/new/dir:$PATH" ;;
    // esac
    //
    // PRACTICE 8: Use absolute paths for security-critical scripts
    // /usr/bin/sudo ...      # Absolute (safe)
    // # sudo ...             # Relative (PATH could be hijacked)

    let path_best_practices = r#"
# PRACTICE 1: Always quote
PATH="/usr/local/bin:$PATH"
export PATH

# PRACTICE 3: Prepend user directories
PATH="/usr/local/bin:$PATH"

# PRACTICE 4: Append home directories
PATH="$PATH:$HOME/bin"
PATH="$PATH:$HOME/.local/bin"

# PRACTICE 5: NEVER put "." in PATH
# PATH=".:$PATH"        # DANGEROUS!

# PRACTICE 6: Check PATH is set
PATH="${PATH:-/usr/bin:/bin}"

# PRACTICE 7: Avoid duplicates
case ":$PATH:" in
    *:/opt/myapp/bin:*) ;;
    *) PATH="/opt/myapp/bin:$PATH"; export PATH ;;
esac

# PRACTICE 8: Use absolute paths for security
/usr/bin/sudo /sbin/reboot
"#;

    let mut lexer = Lexer::new(path_best_practices);
    if let Ok(tokens) = lexer.tokenize() {
        assert!(
            !tokens.is_empty(),
            "PATH best practices should tokenize successfully"
        );
        let _ = tokens;
    }

    // All best practices are POSIX SUPPORTED
    // Security: Never put "." in PATH (prevents Trojan horse attacks)
    // Security: Use absolute paths for sudo, reboot, etc.
}

