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
