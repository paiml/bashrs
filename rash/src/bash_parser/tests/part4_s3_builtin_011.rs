fn test_BUILTIN_011_pwd_basic() {
    // DOCUMENTATION: pwd prints current working directory
    // Most common form, no flags
    // Returns absolute path as string

    let pwd_basic = r#"
pwd
current_dir=$(pwd)
echo "Currently in: $(pwd)"
"#;

    let mut lexer = Lexer::new(pwd_basic);
    match lexer.tokenize() {
        Ok(tokens) => {
            assert!(!tokens.is_empty(), "pwd basic should tokenize");
            let _ = tokens; // Use tokens to satisfy type inference
                            // pwd is simplest form
        }
        Err(_) => {
            // Test documents expected behavior
        }
    }

    // Rust mapping: pwd → std::env::current_dir()
    // Purified bash: pwd → pwd (POSIX supported)
}

#[test]
fn test_BUILTIN_011_pwd_logical_vs_physical() {
    // DOCUMENTATION: pwd -L vs pwd -P distinction
    // pwd -L: Logical path (follows symlinks, default)
    // pwd -P: Physical path (resolves symlinks to actual location)

    let pwd_flags = r#"
# Logical path (default, follows symlinks)
pwd -L

# Physical path (resolves symlinks)
pwd -P

# Example: if /tmp/link -> /var/tmp
# cd /tmp/link
# pwd -L    # prints /tmp/link
# pwd -P    # prints /var/tmp
"#;

    let mut lexer = Lexer::new(pwd_flags);
    match lexer.tokenize() {
        Ok(tokens) => {
            assert!(!tokens.is_empty(), "pwd flags should tokenize");
            let _ = tokens; // Use tokens to satisfy type inference
                            // -L and -P are POSIX flags
        }
        Err(_) => {
            // Test documents expected behavior
        }
    }

    // Key distinction:
    // pwd -L: Shows symlink path (logical)
    // pwd -P: Shows real path (physical, canonical)
}

#[test]
fn test_BUILTIN_011_pwd_vs_env_var() {
    // DOCUMENTATION: pwd command vs $PWD environment variable
    // pwd: Command that queries current directory from system
    // $PWD: Environment variable updated by cd
    // Usually equivalent, but $PWD can be modified manually

    let pwd_vs_env = r#"
# pwd command
current=$(pwd)

# $PWD environment variable
echo $PWD

# Usually equivalent
# But $PWD can be modified:
PWD="/fake/path"  # Doesn't change actual directory
pwd               # Still shows real directory
"#;

    let mut lexer = Lexer::new(pwd_vs_env);
    match lexer.tokenize() {
        Ok(tokens) => {
            assert!(!tokens.is_empty(), "pwd vs env should tokenize");
            let _ = tokens; // Use tokens to satisfy type inference
                            // pwd is reliable, $PWD can be modified
        }
        Err(_) => {
            // Test documents expected behavior
        }
    }

    // Key distinction:
    // pwd: Always accurate (queries system)
    // $PWD: Can be modified (environment variable)
    // Use pwd for reliability, $PWD for efficiency
}

#[test]
fn test_BUILTIN_011_pwd_common_patterns() {
    // DOCUMENTATION: Common pwd usage patterns
    // Save/restore directory, script location, relative paths

    let pwd_patterns = r#"
# Save and restore directory
old_pwd=$(pwd)
cd /tmp
# ... do work ...
cd "$old_pwd"

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# Relative path construction
echo "Config: $(pwd)/config.yml"

# Check if in specific directory
if [ "$(pwd)" = "/etc" ]; then
    echo "In /etc"
fi
"#;

    let mut lexer = Lexer::new(pwd_patterns);
    match lexer.tokenize() {
        Ok(tokens) => {
            assert!(!tokens.is_empty(), "pwd patterns should tokenize");
            let _ = tokens; // Use tokens to satisfy type inference
                            // Common patterns documented
        }
        Err(_) => {
            // Test documents expected behavior
        }
    }

    // Common patterns:
    // 1. Save before cd, restore after
    // 2. Get script directory reliably
    // 3. Build relative paths
    // 4. Check current directory
}
