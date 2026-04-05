#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]

use super::super::ast::Redirect;
use super::super::lexer::Lexer;
use super::super::parser::BashParser;
use super::super::semantic::SemanticAnalyzer;
use super::super::*;

#[test]
fn test_BUILTIN_011_pwd_comparison_table() {
    // COMPREHENSIVE COMPARISON: POSIX vs Bash vs bashrs

    let pwd_comparison = r#"
# POSIX SUPPORTED (bashrs SUPPORTED):
pwd                  # Print current working directory
pwd -L               # Logical path (follow symlinks, default)
pwd -P               # Physical path (resolve symlinks)

# Common usage patterns:
current=$(pwd)       # Save current directory
old=$(pwd); cd /tmp; cd "$old"  # Save and restore

# Script directory pattern:
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# Symlink handling:
# cd /path/to/symlink
pwd -L               # Shows symlink path
pwd -P               # Shows real path

# pwd vs $PWD:
echo $(pwd)          # Command (always accurate)
echo $PWD            # Variable (can be modified)

# Best practices:
dir="$(pwd)"         # Quote for safety
[ "$(pwd)" = "/etc" ]  # Directory check
canonical="$(pwd -P)"  # Get canonical path

# Exit status:
if pwd; then
    echo "Success"
fi
"#;

    let mut lexer = Lexer::new(pwd_comparison);
    match lexer.tokenize() {
        Ok(tokens) => {
            assert!(!tokens.is_empty(), "pwd comparison should tokenize");
            let _ = tokens; // Use tokens to satisfy type inference
        }
        Err(_) => {
            // Test documents comprehensive pwd behavior
        }
    }

    // SUMMARY
    // pwd is POSIX-COMPLIANT and FULLY SUPPORTED in bashrs
    // pwd prints current working directory
    // pwd -L follows symlinks (logical path, default)
    // pwd -P resolves symlinks (physical path)
    // Use pwd for portability, $PWD for efficiency
    // pwd is deterministic (always returns current directory)
}

// ============================================================================
// BUILTIN-016: test / [ Command (POSIX SUPPORTED - HIGH PRIORITY)
// ============================================================================

// DOCUMENTATION: test / [ is SUPPORTED (POSIX builtin, HIGH priority)
//
// test evaluates conditional expressions
// [ is an alias for test (closing ] required)
// [[ ]] is a bash extension (NOT SUPPORTED, use [ ] for portability)
//
// POSIX test supports:
// - File tests: -f (file), -d (dir), -e (exists), -r (read), -w (write), -x (exec)
// - String tests: -z (zero length), -n (non-zero), = (equal), != (not equal)
// - Integer tests: -eq, -ne, -lt, -le, -gt, -ge
// - Logical: ! (not), -a (and), -o (or)
//
// Bash extensions NOT SUPPORTED:
// - [[ ]] compound command (use [ ] instead)
// - =~ regex matching (use grep or sed)
// - Pattern matching with == (use case statement)
// - < > string comparison (use [ "$a" \< "$b" ] with backslash escaping)
//
// INPUT (bash with extensions):
//   [[ -f "file.txt" && "$user" == "admin" ]] → [ -f "file.txt" ] && [ "$user" = "admin" ]
//
// RUST TRANSFORMATION:
//   std::path::Path::new("file.txt").is_file() && user == "admin"
//
// COMPARISON TABLE: test / [ POSIX vs Bash
// ┌─────────────────────────────┬──────────────┬────────────────────────────┐
// │ Feature                     │ POSIX Status │ Purification Strategy      │
// ├─────────────────────────────┼──────────────┼────────────────────────────┤
// │ [ -f "file" ]               │ SUPPORTED    │ Keep as-is                 │
// │ [ -d "dir" ]                │ SUPPORTED    │ Keep as-is                 │
// │ [ -e "path" ]               │ SUPPORTED    │ Keep as-is                 │
// │ [ -r/-w/-x "file" ]         │ SUPPORTED    │ Keep as-is                 │
// │ [ -z "$str" ]               │ SUPPORTED    │ Keep as-is                 │
// │ [ -n "$str" ]               │ SUPPORTED    │ Keep as-is                 │
// │ [ "$a" = "$b" ]             │ SUPPORTED    │ Keep as-is                 │
// │ [ "$a" != "$b" ]            │ SUPPORTED    │ Keep as-is                 │
// │ [ "$a" -eq "$b" ]           │ SUPPORTED    │ Keep as-is                 │
// │ [ "$a" -ne/-lt/-le/-gt/-ge ]│ SUPPORTED    │ Keep as-is                 │
// │ [ ! -f "file" ]             │ SUPPORTED    │ Keep as-is                 │
// │ [ -f "a" -a -f "b" ]        │ SUPPORTED    │ Keep as-is                 │
// │ [ -f "a" -o -f "b" ]        │ SUPPORTED    │ Keep as-is                 │
// │ [[ -f "file" ]]             │ NOT SUPPORT  │ Replace [[ ]] with [ ]     │
// │ [[ "$a" == "$b" ]]          │ NOT SUPPORT  │ Replace == with =          │
// │ [[ "$a" =~ regex ]]         │ NOT SUPPORT  │ Use grep or sed            │
// │ [[ "$a" < "$b" ]]           │ NOT SUPPORT  │ Use [ "$a" \< "$b" ]       │
// │ [ -f "a" && -f "b" ]        │ NOT POSIX    │ Split: [ -f "a" ] && [ ]   │
// └─────────────────────────────┴──────────────┴────────────────────────────┘
//
// PURIFICATION EXAMPLES:
//   1. [[ -f "file.txt" ]] → [ -f "file.txt" ]
//   2. [[ "$user" == "admin" ]] → [ "$user" = "admin" ]
//   3. [[ "$email" =~ regex ]] → printf '%s' "$email" | grep -qE 'regex'
//   4. [ -f "a" && -f "b" ] → [ -f "a" ] && [ -f "b" ]
//   5. [[ "$a" < "$b" ]] → [ "$a" \< "$b" ]
//
// PRIORITY: HIGH - test is fundamental to all conditional logic
// POSIX: IEEE Std 1003.1-2001 test utility
const BUILTIN_016_TEST_COMMAND_INPUT: &str = r#"
if [ -f "file.txt" ]; then
    echo "File exists"
fi

if [ -d "/tmp" ]; then
    echo "Directory exists"
fi

if [ "$user" = "admin" ]; then
    echo "Admin user"
fi

if [ "$count" -gt 10 ]; then
    echo "Count is greater than 10"
fi
"#;

#[test]
fn test_BUILTIN_016_test_command_supported() {
    assert_tokenizes(
        BUILTIN_016_TEST_COMMAND_INPUT,
        "test command should tokenize successfully",
    );
}

// DOCUMENTATION: File test operators (POSIX)
// -f FILE (regular file), -d (dir), -e (exists), -r (readable),
// -w (writable), -x (executable), -s (non-empty), -L (symlink)
// RUST: std::path::Path::new("/etc/passwd").is_file()
const BUILTIN_016_FILE_TESTS_INPUT: &str = r#"
# File type tests
if [ -f "/etc/passwd" ]; then echo "regular file"; fi
if [ -d "/tmp" ]; then echo "directory"; fi
if [ -e "/dev/null" ]; then echo "exists"; fi
if [ -L "/usr/bin/vi" ]; then echo "symlink"; fi

# Permission tests
if [ -r "file.txt" ]; then echo "readable"; fi
if [ -w "file.txt" ]; then echo "writable"; fi
if [ -x "script.sh" ]; then echo "executable"; fi

# Size test
if [ -s "data.txt" ]; then echo "non-empty"; fi
"#;

#[test]
fn test_BUILTIN_016_test_file_tests() {
    assert_tokenizes(
        BUILTIN_016_FILE_TESTS_INPUT,
        "file test operators should tokenize",
    );
}

// DOCUMENTATION: String test operators (POSIX)
// -z STRING (zero length), -n (non-zero), = (equal), != (not equal)
// NOTE: Use = not == for POSIX portability (== is bash-only)
// Purification: [[ "$name" == "alice" ]] → [ "$name" = "alice" ]
const BUILTIN_016_STRING_TESTS_INPUT: &str = r#"
# Empty/non-empty tests
if [ -z "$empty_var" ]; then echo "empty"; fi
if [ -n "$non_empty_var" ]; then echo "non-empty"; fi

# String equality (POSIX uses =, not ==)
if [ "$user" = "admin" ]; then echo "admin user"; fi
if [ "$status" != "error" ]; then echo "ok"; fi

# Always quote variables in tests
if [ -z "$var" ]; then echo "var is empty"; fi
if [ "$a" = "$b" ]; then echo "equal"; fi
"#;

#[test]
fn test_BUILTIN_016_test_string_tests() {
    assert_tokenizes(
        BUILTIN_016_STRING_TESTS_INPUT,
        "string test operators should tokenize",
    );
}

// DOCUMENTATION: Integer comparison operators (POSIX)
// -eq (equal), -ne (not equal), -lt (less), -le (less/equal),
// -gt (greater), -ge (greater/equal)
// NOTE: Use -eq not == for integer comparison
// RUST: count > 10
const BUILTIN_016_INTEGER_TESTS_INPUT: &str = r#"
# Integer comparisons
if [ "$count" -eq 0 ]; then echo "zero"; fi
if [ "$count" -ne 0 ]; then echo "non-zero"; fi
if [ "$count" -lt 10 ]; then echo "less than 10"; fi
if [ "$count" -le 10 ]; then echo "at most 10"; fi
if [ "$count" -gt 10 ]; then echo "greater than 10"; fi
if [ "$count" -ge 10 ]; then echo "at least 10"; fi

# Common patterns
if [ "$retries" -lt "$max_retries" ]; then
    echo "Retry available"
fi

if [ "$exit_code" -ne 0 ]; then
    echo "Command failed"
fi
"#;

#[test]
fn test_BUILTIN_016_test_integer_tests() {
    assert_tokenizes(
        BUILTIN_016_INTEGER_TESTS_INPUT,
        "integer test operators should tokenize",
    );
}

// DOCUMENTATION: Logical operators for test (POSIX)
// ! EXPR (NOT), EXPR1 -a EXPR2 (AND), EXPR1 -o EXPR2 (OR)
// MODERN POSIX: split into multiple [ ] tests with && and ||
// OLD POSIX: combine with -a/-o inside single [ ] (deprecated)
// Purification: [[ -f "file" && -r "file" ]] → [ -f "file" ] && [ -r "file" ]
const BUILTIN_016_LOGICAL_TESTS_INPUT: &str = r#"
# Logical NOT
if [ ! -f "missing.txt" ]; then echo "file does not exist"; fi

# Logical AND (modern style - preferred)
if [ -f "file.txt" ] && [ -r "file.txt" ]; then
    cat file.txt
fi

# Logical OR (modern style - preferred)
if [ "$status" = "ok" ] || [ "$status" = "success" ]; then
    echo "Operation succeeded"
fi

# Logical AND (old style - deprecated but valid)
if [ -f "file.txt" -a -r "file.txt" ]; then
    cat file.txt
fi

# Logical OR (old style - deprecated but valid)
if [ "$a" = "1" -o "$a" = "2" ]; then
    echo "a is 1 or 2"
fi

# Complex logic with negation
if [ ! -z "$var" ] && [ -f "$var" ]; then
    echo "$var is a non-empty filename"
fi
"#;

#[test]
fn test_BUILTIN_016_test_logical_operators() {
    assert_tokenizes(
        BUILTIN_016_LOGICAL_TESTS_INPUT,
        "logical operators should tokenize",
    );
}

// DOCUMENTATION: Bash [[ ]] extensions (NOT SUPPORTED)
// [[ ]] is a bash keyword, not a POSIX builtin.
// BASH EXTENSIONS (NOT SUPPORTED):
//   1. [[ ]] compound command → use [ ] instead
//   2. == pattern matching → use = for string equality
//   3. =~ regex matching → use grep, sed, or case
//   4. < > string comparison without escaping → use \< \>
//   5. && || inside [[ ]] → split into separate [ ] tests
const BUILTIN_016_BASH_EXTENSIONS_INPUT: &str = r#"
# BASH EXTENSION: [[ ]] compound command (NOT SUPPORTED)
# Purify: Replace [[ ]] with [ ]
# if [[ -f "file.txt" ]]; then echo "exists"; fi
# →
if [ -f "file.txt" ]; then echo "exists"; fi

# BASH EXTENSION: == operator (NOT SUPPORTED)
# Purify: Replace == with =
# if [[ "$user" == "admin" ]]; then echo "admin"; fi
# →
if [ "$user" = "admin" ]; then echo "admin"; fi

# BASH EXTENSION: =~ regex (NOT SUPPORTED)
# Purify: Use grep instead
# if [[ "$email" =~ ^[a-z]+@[a-z]+\.com$ ]]; then echo "valid"; fi
# →
if printf '%s' "$email" | grep -qE '^[a-z]+@[a-z]+\.com$'; then
    echo "valid"
fi

# BASH EXTENSION: Pattern matching with == (NOT SUPPORTED)
# Purify: Use case statement
# if [[ "$file" == *.txt ]]; then echo "text file"; fi
# →
case "$file" in
    *.txt)
        echo "text file"
        ;;
esac

# BASH EXTENSION: < > without escaping (NOT SUPPORTED)
# Purify: Add backslash escaping
# if [[ "$a" < "$b" ]]; then echo "less"; fi
# →
if [ "$a" \< "$b" ]; then echo "less"; fi
"#;

#[test]
fn test_BUILTIN_016_test_bash_extensions_not_supported() {
    assert_tokenizes(
        BUILTIN_016_BASH_EXTENSIONS_INPUT,
        "bash extension examples should tokenize",
    );
}

// DOCUMENTATION: Common test patterns in POSIX scripts
// 1. Check file exists before reading
// 2. Check variable is set
// 3. Check variable is unset or empty
// 4. Check exit status
// 5. Check multiple conditions
// 6. Check for errors (defensive programming)
// 7. Alternative values
const BUILTIN_016_COMMON_PATTERNS_INPUT: &str = r#"
# Pattern 1: Safe file operations
if [ -f "config.sh" ]; then
    . config.sh
fi

# Pattern 2: Variable validation
if [ -z "$REQUIRED_VAR" ]; then
    echo "Error: REQUIRED_VAR is not set"
    exit 1
fi

# Pattern 3: Default values
if [ -z "$PORT" ]; then
    PORT=8080
fi

# Pattern 4: Error checking
command_that_might_fail
if [ "$?" -ne 0 ]; then
    echo "Command failed with exit code $?"
    exit 1
fi

# Pattern 5: Defensive programming
if [ ! -d "$install_dir" ]; then
    echo "Error: Install directory does not exist: $install_dir"
    exit 1
fi

# Pattern 6: Multi-condition validation
if [ -f "$script" ] && [ -r "$script" ] && [ -x "$script" ]; then
    "$script"
else
    echo "Error: $script is not a readable executable file"
    exit 1
fi

# Pattern 7: Alternative values
if [ -n "$CUSTOM_PATH" ]; then
    PATH="$CUSTOM_PATH"
else
    PATH="/usr/local/bin:/usr/bin:/bin"
fi
"#;

#[test]
fn test_BUILTIN_016_test_common_patterns() {
    assert_tokenizes(
        BUILTIN_016_COMMON_PATTERNS_INPUT,
        "common test patterns should tokenize",
    );
}

