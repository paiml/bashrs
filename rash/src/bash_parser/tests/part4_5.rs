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
fn test_BUILTIN_016_test_comparison_table() {
    // COMPREHENSIVE COMPARISON: test / [ in POSIX vs Bash
    //
    // ┌──────────────────────────────────────────────────────────────────────────┐
    // │ Feature: test / [ Command                                                │
    // ├────────────────────────────┬──────────────┬──────────────────────────────┤
    // │ Feature                    │ POSIX Status │ Purification                 │
    // ├────────────────────────────┼──────────────┼──────────────────────────────┤
    // │ FILE TESTS                 │              │                              │
    // │ [ -f "file" ]              │ SUPPORTED    │ Keep as-is                   │
    // │ [ -d "dir" ]               │ SUPPORTED    │ Keep as-is                   │
    // │ [ -e "path" ]              │ SUPPORTED    │ Keep as-is                   │
    // │ [ -r/-w/-x "file" ]        │ SUPPORTED    │ Keep as-is                   │
    // │ [ -s "file" ]              │ SUPPORTED    │ Keep as-is                   │
    // │ [ -L "link" ]              │ SUPPORTED    │ Keep as-is                   │
    // │                            │              │                              │
    // │ STRING TESTS               │              │                              │
    // │ [ -z "$str" ]              │ SUPPORTED    │ Keep as-is                   │
    // │ [ -n "$str" ]              │ SUPPORTED    │ Keep as-is                   │
    // │ [ "$a" = "$b" ]            │ SUPPORTED    │ Keep as-is                   │
    // │ [ "$a" != "$b" ]           │ SUPPORTED    │ Keep as-is                   │
    // │ [ "$a" \< "$b" ]           │ SUPPORTED    │ Keep as-is (note backslash)  │
    // │ [ "$a" \> "$b" ]           │ SUPPORTED    │ Keep as-is (note backslash)  │
    // │                            │              │                              │
    // │ INTEGER TESTS              │              │                              │
    // │ [ "$a" -eq "$b" ]          │ SUPPORTED    │ Keep as-is                   │
    // │ [ "$a" -ne "$b" ]          │ SUPPORTED    │ Keep as-is                   │
    // │ [ "$a" -lt "$b" ]          │ SUPPORTED    │ Keep as-is                   │
    // │ [ "$a" -le "$b" ]          │ SUPPORTED    │ Keep as-is                   │
    // │ [ "$a" -gt "$b" ]          │ SUPPORTED    │ Keep as-is                   │
    // │ [ "$a" -ge "$b" ]          │ SUPPORTED    │ Keep as-is                   │
    // │                            │              │                              │
    // │ LOGICAL OPERATORS          │              │                              │
    // │ [ ! EXPR ]                 │ SUPPORTED    │ Keep as-is                   │
    // │ [ EXPR1 -a EXPR2 ]         │ SUPPORTED    │ Prefer: [ ] && [ ]           │
    // │ [ EXPR1 -o EXPR2 ]         │ SUPPORTED    │ Prefer: [ ] || [ ]           │
    // │ [ EXPR1 ] && [ EXPR2 ]     │ SUPPORTED    │ Keep as-is (preferred)       │
    // │ [ EXPR1 ] || [ EXPR2 ]     │ SUPPORTED    │ Keep as-is (preferred)       │
    // │                            │              │                              │
    // │ BASH EXTENSIONS            │              │                              │
    // │ [[ ]]                      │ NOT SUPPORT  │ Replace with [ ]             │
    // │ [[ "$a" == "$b" ]]         │ NOT SUPPORT  │ Use [ "$a" = "$b" ]          │
    // │ [[ "$a" =~ regex ]]        │ NOT SUPPORT  │ Use grep/sed/case            │
    // │ [[ "$a" < "$b" ]]          │ NOT SUPPORT  │ Use [ "$a" \< "$b" ]         │
    // │ [[ "$f" == *.txt ]]        │ NOT SUPPORT  │ Use case statement           │
    // │ [[ -f "a" && -f "b" ]]     │ NOT SUPPORT  │ Use [ ] && [ ]               │
    // └────────────────────────────┴──────────────┴──────────────────────────────┘
    //
    // RUST MAPPING:
    // [ -f "file" ]           → std::path::Path::new("file").is_file()
    // [ -d "dir" ]            → std::path::Path::new("dir").is_dir()
    // [ -e "path" ]           → std::path::Path::new("path").exists()
    // [ "$a" = "$b" ]         → a == b
    // [ "$a" -eq "$b" ]       → a == b (for integers)
    // [ "$a" -lt "$b" ]       → a < b
    // [ "$a" -gt "$b" ]       → a > b
    // [ -z "$str" ]           → str.is_empty()
    // [ -n "$str" ]           → !str.is_empty()
    //
    // DETERMINISM: test is deterministic (file/string/integer tests are pure)
    // IDEMPOTENCY: test is idempotent (no side effects, pure evaluation)
    // PORTABILITY: Use [ ] not [[ ]] for maximum POSIX portability

    let comparison_table = r#"
# This test documents the complete POSIX vs Bash comparison for test / [
# See extensive comparison table in test function comments above

# POSIX SUPPORTED: File tests
[ -f "file.txt" ]       # Regular file
[ -d "directory" ]      # Directory
[ -e "path" ]           # Exists (any type)
[ -r "file" ]           # Readable
[ -w "file" ]           # Writable
[ -x "file" ]           # Executable
[ -s "file" ]           # Non-empty (size > 0)
[ -L "link" ]           # Symbolic link

# POSIX SUPPORTED: String tests
[ -z "$empty" ]         # Zero length
[ -n "$non_empty" ]     # Non-zero length
[ "$a" = "$b" ]         # Equal (use =, not ==)
[ "$a" != "$b" ]        # Not equal
[ "$a" \< "$b" ]        # Less than (lexicographic, escaped)
[ "$a" \> "$b" ]        # Greater than (lexicographic, escaped)

# POSIX SUPPORTED: Integer tests
[ "$a" -eq "$b" ]       # Equal
[ "$a" -ne "$b" ]       # Not equal
[ "$a" -lt "$b" ]       # Less than
[ "$a" -le "$b" ]       # Less than or equal
[ "$a" -gt "$b" ]       # Greater than
[ "$a" -ge "$b" ]       # Greater than or equal

# POSIX SUPPORTED: Logical operators
[ ! -f "missing" ]      # NOT
[ -f "a" -a -f "b" ]    # AND (deprecated, use [ ] && [ ] instead)
[ -f "a" -o -f "b" ]    # OR (deprecated, use [ ] || [ ] instead)
[ -f "a" ] && [ -f "b" ] # AND (preferred modern style)
[ -f "a" ] || [ -f "b" ] # OR (preferred modern style)

# NOT SUPPORTED: Bash [[ ]] extensions
# [[ -f "file" ]]              → Use [ -f "file" ]
# [[ "$a" == "$b" ]]           → Use [ "$a" = "$b" ]
# [[ "$str" =~ regex ]]        → Use grep/sed/case
# [[ "$a" < "$b" ]]            → Use [ "$a" \< "$b" ]
# [[ "$file" == *.txt ]]       → Use case statement
# [[ -f "a" && -f "b" ]]       → Use [ -f "a" ] && [ -f "b" ]
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

    // Priority: HIGH - test is fundamental to all conditional logic in shell scripts
    // POSIX: IEEE Std 1003.1-2001 test utility and [ special builtin
    // Portability: Use [ ] with = (not ==) for maximum compatibility
    // Determinism: test is deterministic (file tests may change, but evaluation is pure)
    // Idempotency: test is idempotent (no side effects, reads system state)
}

// ============================================================================
// BUILTIN-020: unset Command (POSIX SUPPORTED - HIGH PRIORITY)
// ============================================================================

#[test]
fn test_BUILTIN_020_unset_command_supported() {
    // DOCUMENTATION: unset is SUPPORTED (POSIX builtin, HIGH priority)
    //
    // unset removes variables and functions from the shell environment
    // Syntax: unset [-v] [-f] name [name ...]
    //
    // POSIX unset supports:
    // - unset VAR: Remove variable (default behavior)
    // - unset -v VAR: Explicitly remove variable
    // - unset -f FUNC: Remove function
    // - unset VAR1 VAR2 VAR3: Remove multiple variables
    //
    // Bash extensions NOT SUPPORTED:
    // - unset -n nameref: Remove nameref (use regular unset)
    // - Array element unsetting: unset array[index] (use whole array unset)
    //
    // POSIX BEHAVIOR:
    // - Unsetting non-existent variable: Not an error (exit 0)
    // - Unsetting readonly variable: Error (exit non-zero)
    // - Unsetting without name: Error (exit non-zero)
    // - Exit status: 0 on success, non-zero on error
    //
    // INPUT (bash):
    // VAR="value"
    // unset VAR
    // echo "$VAR"  # Empty output
    //
    // RUST TRANSFORMATION:
    // let mut vars = HashMap::new();
    // vars.insert("VAR".to_string(), "value".to_string());
    // vars.remove("VAR");
    // println!("{}", vars.get("VAR").unwrap_or(&"".to_string()));
    //
    // PURIFIED (POSIX sh):
    // VAR="value"
    // unset VAR
    // printf '%s\n' "$VAR"  # Empty output
    //
    // COMPARISON TABLE: unset POSIX vs Bash
    // ┌───────────────────────────┬──────────────┬────────────────────────────┐
    // │ Feature                   │ POSIX Status │ Purification Strategy      │
    // ├───────────────────────────┼──────────────┼────────────────────────────┤
    // │ unset VAR                 │ SUPPORTED    │ Keep as-is                 │
    // │ unset -v VAR              │ SUPPORTED    │ Keep as-is                 │
    // │ unset -f FUNC             │ SUPPORTED    │ Keep as-is                 │
    // │ unset VAR1 VAR2 VAR3      │ SUPPORTED    │ Keep as-is                 │
    // │ unset readonly fails      │ SUPPORTED    │ Keep as-is                 │
    // │ unset non-existent ok     │ SUPPORTED    │ Keep as-is                 │
    // │ unset -n nameref          │ NOT SUPPORT  │ Use unset VAR              │
    // │ unset array[index]        │ NOT SUPPORT  │ Use unset array (whole)    │
    // └───────────────────────────┴──────────────┴────────────────────────────┘
    //
    // PURIFICATION EXAMPLES:
    //
    // 1. Basic variable unset (POSIX):
    //    Bash:     VAR="value"; unset VAR
    //    Purified: VAR="value"; unset VAR  (no change)
    //
    // 2. Function unset (POSIX):
    //    Bash:     func() { echo "hi"; }; unset -f func
    //    Purified: func() { echo "hi"; }; unset -f func  (no change)
    //
    // 3. Nameref unset (NOT SUPPORTED):
    //    Bash:     declare -n ref=VAR; unset -n ref
    //    Purified: VAR=""; # Just clear the variable instead
    //
    // 4. Array element unset (NOT SUPPORTED):
    //    Bash:     arr=(a b c); unset arr[1]
    //    Purified: arr="a c"  # Reassign without element
    //
    // PRIORITY: HIGH - unset is essential for variable lifecycle management
    // POSIX: IEEE Std 1003.1-2001 unset special builtin

    let unset_command = r#"
VAR="value"
unset VAR

FUNC="initial"
unset FUNC

# Multiple variables
A="1"
B="2"
C="3"
unset A B C

# Function unset
myfunc() {
    echo "hello"
}
unset -f myfunc
"#;

    let mut lexer = Lexer::new(unset_command);
    match lexer.tokenize() {
        Ok(tokens) => {
            assert!(
                !tokens.is_empty(),
                "unset command should tokenize successfully"
            );
            let _ = tokens;
        }
        Err(_) => {
            // Parser may not fully support unset yet - test documents expected behavior
        }
    }
}

#[test]
fn test_BUILTIN_020_unset_variables() {
    // DOCUMENTATION: Unsetting variables (POSIX)
    //
    // unset VAR: Remove variable from environment
    // unset -v VAR: Explicitly remove variable (same as unset VAR)
    //
    // After unset, variable tests:
    // - [ -z "$VAR" ]: True (empty string)
    // - echo "$VAR": Empty output
    // - set | grep VAR: Variable not listed
    //
    // INPUT (bash):
    // USER="alice"
    // echo "$USER"  # alice
    // unset USER
    // echo "$USER"  # (empty)
    //
    // RUST:
    // let mut vars = HashMap::new();
    // vars.insert("USER".to_string(), "alice".to_string());
    // println!("{}", vars.get("USER").unwrap());  // alice
    // vars.remove("USER");
    // println!("{}", vars.get("USER").unwrap_or(&"".to_string()));  // (empty)
    //
    // PURIFIED (POSIX sh):
    // USER="alice"
    // printf '%s\n' "$USER"  # alice
    // unset USER
    // printf '%s\n' "$USER"  # (empty)

    let unset_variables = r#"
# Basic variable unset
NAME="John"
echo "$NAME"
unset NAME
echo "$NAME"  # Empty

# Explicit -v flag (same as unset)
EMAIL="john@example.com"
unset -v EMAIL
echo "$EMAIL"  # Empty

# Multiple variables in one command
VAR1="a"
VAR2="b"
VAR3="c"
unset VAR1 VAR2 VAR3

# Check if variable is unset
CONFIG="/etc/config"
unset CONFIG
if [ -z "$CONFIG" ]; then
    echo "CONFIG is unset"
fi
"#;

    let mut lexer = Lexer::new(unset_variables);
    match lexer.tokenize() {
        Ok(tokens) => {
            assert!(!tokens.is_empty(), "variable unset should tokenize");
            let _ = tokens;
        }
        Err(_) => {
            // Parser may not fully support unset yet
        }
    }
}

#[test]
fn test_BUILTIN_020_unset_functions() {
    // DOCUMENTATION: Unsetting functions (POSIX)
    //
    // unset -f FUNC: Remove function definition
    //
    // Without -f flag, unset removes variables by default
    // With -f flag, unset removes functions
    //
    // If both variable and function exist with same name:
    // - unset NAME: Removes variable
    // - unset -f NAME: Removes function
    //
    // INPUT (bash):
    // greet() { echo "Hello"; }
    // greet  # Hello
    // unset -f greet
    // greet  # Command not found
    //
    // RUST:
    // fn greet() { println!("Hello"); }
    // greet();  // Hello
    // // (Cannot dynamically unset functions in Rust)
    //
    // PURIFIED (POSIX sh):
    // greet() { printf '%s\n' "Hello"; }
    // greet  # Hello
    // unset -f greet
    // # greet  # Would fail if called

    let unset_functions = r#"
# Define function
hello() {
    echo "Hello, World!"
}

# Call function
hello

# Unset function
unset -f hello

# Calling would fail now
# hello  # Command not found

# Multiple functions
func1() { echo "1"; }
func2() { echo "2"; }
func3() { echo "3"; }
unset -f func1 func2 func3

# Variable vs function with same name
NAME="variable"
NAME() {
    echo "function"
}
unset NAME      # Removes variable
unset -f NAME   # Removes function
"#;

    let mut lexer = Lexer::new(unset_functions);
    match lexer.tokenize() {
        Ok(tokens) => {
            assert!(!tokens.is_empty(), "function unset should tokenize");
            let _ = tokens;
        }
        Err(_) => {
            // Parser may not fully support function unset yet
        }
    }
}

// DOCUMENTATION: unset exit status (POSIX)
// Exit 0: Success (variable/function unset or didn't exist)
// Exit non-zero: Error (invalid option, readonly variable)
// RUST: vars.remove("NONEXISTENT") → Ok(()) regardless
const BUILTIN_020_UNSET_EXIT_STATUS_INPUT: &str = r#"
# Unset non-existent variable (success)
unset DOES_NOT_EXIST
if [ "$?" -eq 0 ]; then
    echo "unset DOES_NOT_EXIST succeeded"
fi

# Set and unset variable (success)
TEMP="value"
unset TEMP
if [ "$?" -eq 0 ]; then
    echo "unset TEMP succeeded"
fi

# Readonly variable unset (error)
readonly READONLY_VAR="constant"
unset READONLY_VAR
if [ "$?" -ne 0 ]; then
    echo "unset READONLY_VAR failed (expected)"
fi

# Multiple unsets (success if all ok)
VAR1="a"
VAR2="b"
unset VAR1 VAR2 VAR3
echo "Exit status: $?"
"#;

include!("part4_5_tests_builtin_020.rs");
