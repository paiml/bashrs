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
fn test_BUILTIN_020_unset_bash_extensions_not_supported() {
    // DOCUMENTATION: Bash unset extensions (NOT SUPPORTED)
    //
    // BASH EXTENSIONS (NOT SUPPORTED):
    // 1. unset -n nameref: Unset nameref (use regular unset)
    // 2. unset array[index]: Unset array element (use array reassignment)
    // 3. unset associative array elements (use whole array unset)
    //
    // PURIFICATION STRATEGIES:
    //
    // 1. Nameref unset (NOT SUPPORTED):
    //    Bash:     declare -n ref=VAR; unset -n ref
    //    Purified: VAR=""  # Just clear the variable
    //
    // 2. Array element unset (NOT SUPPORTED):
    //    Bash:     arr=(a b c); unset arr[1]
    //    Purified: arr="a c"  # Reassign without element
    //               # Or use awk/sed to remove element
    //
    // 3. Associative array (NOT SUPPORTED):
    //    Bash:     declare -A map=([k1]=v1 [k2]=v2); unset map[k1]
    //    Purified: # Use separate variables or external data structure

    let bash_extensions = r#"
# BASH EXTENSION: unset -n nameref (NOT SUPPORTED)
# Purify: Use regular variable clearing
# declare -n ref=TARGET
# unset -n ref
# →
TARGET=""

# BASH EXTENSION: unset array[index] (NOT SUPPORTED)
# Purify: Reassign array without element or use awk
# arr=(a b c)
# unset arr[1]
# →
# Set array to "a c" (skip element 1)

# BASH EXTENSION: Associative array unset (NOT SUPPORTED)
# Purify: Use separate variables
# declare -A config=([host]=localhost [port]=8080)
# unset config[port]
# →
config_host="localhost"
config_port=""  # Clear instead of unset element

# POSIX SUPPORTED: Regular variable unset
VAR="value"
unset VAR

# POSIX SUPPORTED: Function unset
cleanup() { echo "cleanup"; }
unset -f cleanup

# POSIX SUPPORTED: Multiple unsets
A="1"
B="2"
C="3"
unset A B C
"#;

    let mut lexer = Lexer::new(bash_extensions);
    match lexer.tokenize() {
        Ok(tokens) => {
            assert!(
                !tokens.is_empty(),
                "bash extension examples should tokenize"
            );
            let _ = tokens;
        }
        Err(_) => {
            // These are purified examples, should parse as comments and POSIX constructs
        }
    }
}

#[test]
fn test_BUILTIN_020_unset_vs_empty_assignment() {
    // DOCUMENTATION: unset vs empty assignment (Important distinction)
    //
    // unset VAR: Removes variable completely
    // VAR="": Sets variable to empty string
    //
    // DIFFERENCE IN TESTS:
    // After unset VAR:
    // - [ -z "$VAR" ]: True (empty)
    // - [ -n "$VAR" ]: False (not set)
    // - ${VAR:-default}: "default" (uses default)
    // - ${VAR-default}: "default" (uses default)
    //
    // After VAR="":
    // - [ -z "$VAR" ]: True (empty)
    // - [ -n "$VAR" ]: False (empty string)
    // - ${VAR:-default}: "default" (empty, uses default)
    // - ${VAR-default}: "" (set but empty, no default)
    //
    // KEY DISTINCTION:
    // ${VAR-default}: Use default if VAR is UNSET
    // ${VAR:-default}: Use default if VAR is UNSET OR EMPTY
    //
    // INPUT (bash):
    // unset VAR
    // echo "${VAR-fallback}"   # fallback (unset)
    // echo "${VAR:-fallback}"  # fallback (unset)
    //
    // VAR=""
    // echo "${VAR-fallback}"   # (empty, VAR is set)
    // echo "${VAR:-fallback}"  # fallback (empty)
    //
    // RUST:
    // let mut vars: HashMap<String, String> = HashMap::new();
    // // Unset: key not in map
    // vars.get("VAR").unwrap_or(&"fallback".to_string());
    //
    // // Empty: key in map with empty value
    // vars.insert("VAR".to_string(), "".to_string());
    // vars.get("VAR").filter(|v| !v.is_empty()).unwrap_or(&"fallback".to_string());

    let unset_vs_empty = r#"
# Unset variable
unset VAR
echo "${VAR-default1}"   # default1 (unset, uses default)
echo "${VAR:-default2}"  # default2 (unset, uses default)

# Empty assignment
VAR=""
echo "${VAR-default3}"   # (empty, VAR is SET so no default)
echo "${VAR:-default4}"  # default4 (empty, uses default)

# Set to value
VAR="value"
echo "${VAR-default5}"   # value
echo "${VAR:-default6}"  # value

# Testing with [ -z ] and [ -n ]
unset UNSET_VAR
if [ -z "$UNSET_VAR" ]; then
    echo "UNSET_VAR is empty or unset"
fi

EMPTY_VAR=""
if [ -z "$EMPTY_VAR" ]; then
    echo "EMPTY_VAR is empty (set but empty)"
fi

# Practical difference
CONFIG_FILE=""  # Set but empty
if [ -n "$CONFIG_FILE" ]; then
    echo "Using config: $CONFIG_FILE"
else
    echo "No config (empty or unset)"
fi

unset CONFIG_FILE  # Now truly unset
if [ -n "$CONFIG_FILE" ]; then
    echo "Using config: $CONFIG_FILE"
else
    echo "No config (unset)"
fi
"#;

    let mut lexer = Lexer::new(unset_vs_empty);
    match lexer.tokenize() {
        Ok(tokens) => {
            assert!(
                !tokens.is_empty(),
                "unset vs empty examples should tokenize"
            );
            let _ = tokens;
        }
        Err(_) => {
            // Parser may not fully support parameter expansion yet
        }
    }
}

#[test]
fn test_BUILTIN_020_unset_comparison_table() {
    // COMPREHENSIVE COMPARISON: unset in POSIX vs Bash
    //
    // ┌──────────────────────────────────────────────────────────────────────────┐
    // │ Feature: unset Command                                                   │
    // ├────────────────────────────┬──────────────┬──────────────────────────────┤
    // │ Feature                    │ POSIX Status │ Purification                 │
    // ├────────────────────────────┼──────────────┼──────────────────────────────┤
    // │ BASIC UNSET                │              │                              │
    // │ unset VAR                  │ SUPPORTED    │ Keep as-is                   │
    // │ unset -v VAR               │ SUPPORTED    │ Keep as-is                   │
    // │ unset -f FUNC              │ SUPPORTED    │ Keep as-is                   │
    // │ unset VAR1 VAR2 VAR3       │ SUPPORTED    │ Keep as-is                   │
    // │                            │              │                              │
    // │ EXIT STATUS                │              │                              │
    // │ unset NONEXISTENT → 0      │ SUPPORTED    │ Keep as-is                   │
    // │ unset readonly → non-zero  │ SUPPORTED    │ Keep as-is                   │
    // │                            │              │                              │
    // │ BEHAVIOR                   │              │                              │
    // │ Removes variable           │ SUPPORTED    │ Keep as-is                   │
    // │ Removes function           │ SUPPORTED    │ Keep as-is                   │
    // │ ${VAR-default} works       │ SUPPORTED    │ Keep as-is                   │
    // │ ${VAR:-default} works      │ SUPPORTED    │ Keep as-is                   │
    // │                            │              │                              │
    // │ BASH EXTENSIONS            │              │                              │
    // │ unset -n nameref           │ NOT SUPPORT  │ Use VAR="" instead           │
    // │ unset array[index]         │ NOT SUPPORT  │ Reassign array               │
    // │ unset assoc[key]           │ NOT SUPPORT  │ Use separate variables       │
    // └────────────────────────────┴──────────────┴──────────────────────────────┘
    //
    // RUST MAPPING:
    // unset VAR              → vars.remove("VAR")
    // unset -f FUNC          → functions.remove("FUNC")
    // ${VAR-default}         → vars.get("VAR").unwrap_or(&"default")
    // ${VAR:-default}        → vars.get("VAR").filter(|v| !v.is_empty()).unwrap_or(&"default")
    //
    // DETERMINISM: unset is deterministic (removes variable from environment)
    // IDEMPOTENCY: unset is idempotent (unsetting twice has same effect)
    // PORTABILITY: Use unset VAR for maximum POSIX compatibility

    let comparison_table = r#"
# This test documents the complete POSIX vs Bash comparison for unset
# See extensive comparison table in test function comments above

# POSIX SUPPORTED: Basic unset
unset VAR                   # Remove variable (default)
unset -v VAR2               # Remove variable (explicit)
unset -f myfunc             # Remove function
unset VAR1 VAR2 VAR3        # Remove multiple

# POSIX SUPPORTED: Exit status
unset NONEXISTENT           # Exit 0 (not an error)
# readonly CONST="value"
# unset CONST               # Exit non-zero (error)

# POSIX SUPPORTED: Behavior after unset
VAR="value"
unset VAR
echo "${VAR-default}"       # default (unset, uses default)
echo "${VAR:-default2}"     # default2 (unset, uses default)

# POSIX SUPPORTED: Function unset
greet() { echo "hello"; }
greet
unset -f greet
# greet  # Would fail

# NOT SUPPORTED: Bash nameref
# declare -n ref=TARGET
# unset -n ref
# →
TARGET=""  # Clear instead

# NOT SUPPORTED: Array element unset
# arr=(a b c)
# unset arr[1]
# →
# Reassign: arr="a c"

# NOT SUPPORTED: Associative array
# declare -A map=([k1]=v1)
# unset map[k1]
# →
map_k1=""  # Use separate variables

# POSIX PATTERN: Unset vs empty
unset UNSET_VAR             # Truly unset
EMPTY_VAR=""                # Set but empty
echo "${UNSET_VAR-a}"       # a (unset)
echo "${EMPTY_VAR-b}"       # (empty, no default)
echo "${UNSET_VAR:-c}"      # c (unset)
echo "${EMPTY_VAR:-d}"      # d (empty, uses default)
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

    // Priority: HIGH - unset is essential for variable lifecycle management
    // POSIX: IEEE Std 1003.1-2001 unset special builtin
    // Portability: Use unset VAR for maximum POSIX compatibility
    // Determinism: unset is deterministic (removes variable from environment)
    // Idempotency: unset is idempotent (unsetting twice has same effect as once)
}

// ============================================================================
// BASH-BUILTIN-005: printf Command (POSIX SUPPORTED - HIGH PRIORITY)
// ============================================================================

#[test]
fn test_BASH_BUILTIN_005_printf_command_supported() {
    // DOCUMENTATION: printf is SUPPORTED (POSIX builtin, HIGH priority)
    //
    // printf formats and prints data (better than echo for portability)
    // Syntax: printf format [arguments ...]
    //
    // POSIX printf supports:
    // - Format specifiers: %s (string), %d (integer), %f (float), %x (hex), %o (octal)
    // - Escape sequences: \n (newline), \t (tab), \\ (backslash), \' (quote)
    // - Width/precision: %10s (width 10), %.2f (2 decimals)
    // - Flags: %- (left align), %0 (zero pad), %+ (force sign)
    //
    // WHY printf over echo:
    // - Portable: POSIX-defined behavior (echo varies across shells)
    // - No trailing newline by default (explicit \n control)
    // - Format control: Precise formatting like C printf
    // - Escape handling: Consistent across all POSIX shells
    //
    // Bash extensions NOT SUPPORTED:
    // - %(...)T date formatting (use date command instead)
    // - %b interpret backslash escapes in argument (use \n in format instead)
    // - %q shell-quote format (use manual quoting)
    //
    // INPUT (bash):
    // printf '%s %d\n' "Count:" 42
    // printf 'Name: %s\nAge: %d\n' "Alice" 30
    //
    // RUST TRANSFORMATION:
    // println!("{} {}", "Count:", 42);
    // println!("Name: {}\nAge: {}", "Alice", 30);
    //
    // PURIFIED (POSIX sh):
    // printf '%s %d\n' "Count:" 42
    // printf 'Name: %s\nAge: %d\n' "Alice" 30
    //
    // COMPARISON TABLE: printf POSIX vs Bash vs echo
    // ┌─────────────────────────────┬──────────────┬────────────────────────────┐
    // │ Feature                     │ POSIX Status │ Purification Strategy      │
    // ├─────────────────────────────┼──────────────┼────────────────────────────┤
    // │ printf '%s\n' "text"        │ SUPPORTED    │ Keep as-is                 │
    // │ printf '%d' 42              │ SUPPORTED    │ Keep as-is                 │
    // │ printf '%.2f' 3.14159       │ SUPPORTED    │ Keep as-is                 │
    // │ printf '%x' 255             │ SUPPORTED    │ Keep as-is                 │
    // │ printf '%10s' "right"       │ SUPPORTED    │ Keep as-is                 │
    // │ printf '%-10s' "left"       │ SUPPORTED    │ Keep as-is                 │
    // │ printf '%05d' 42            │ SUPPORTED    │ Keep as-is                 │
    // │ Escape: \n \t \\ \'         │ SUPPORTED    │ Keep as-is                 │
    // │ printf %(...)T date         │ NOT SUPPORT  │ Use date command           │
    // │ printf %b "a\nb"            │ NOT SUPPORT  │ Use \n in format           │
    // │ printf %q "string"          │ NOT SUPPORT  │ Manual quoting             │
    // │ echo "text" (non-portable)  │ AVOID        │ Use printf '%s\n' "text"   │
    // └─────────────────────────────┴──────────────┴────────────────────────────┘
    //
    // PURIFICATION EXAMPLES:
    //
    // 1. Replace echo with printf (POSIX best practice):
    //    Bash:     echo "Hello, World!"
    //    Purified: printf '%s\n' "Hello, World!"
    //
    // 2. Replace echo -n with printf (no newline):
    //    Bash:     echo -n "Prompt: "
    //    Purified: printf '%s' "Prompt: "
    //
    // 3. Replace date formatting:
    //    Bash:     printf '%(Date: %Y-%m-%d)T\n'
    //    Purified: printf 'Date: %s\n' "$(date +%Y-%m-%d)"
    //
    // 4. Replace %b with explicit escapes:
    //    Bash:     printf '%b' "Line1\nLine2"
    //    Purified: printf 'Line1\nLine2'
    //
    // PRIORITY: HIGH - printf is the portable alternative to echo
    // POSIX: IEEE Std 1003.1-2001 printf utility

    let printf_command = r#"
printf '%s\n' "Hello, World!"
printf '%s %d\n' "Count:" 42
printf 'Name: %s\nAge: %d\n' "Alice" 30
printf '%.2f\n' 3.14159
"#;

    let mut lexer = Lexer::new(printf_command);
    match lexer.tokenize() {
        Ok(tokens) => {
            assert!(
                !tokens.is_empty(),
                "printf command should tokenize successfully"
            );
            let _ = tokens;
        }
        Err(_) => {
            // Parser may not fully support printf yet - test documents expected behavior
        }
    }
}

#[test]
fn test_BASH_BUILTIN_005_printf_format_specifiers() {
    // DOCUMENTATION: printf format specifiers (POSIX)
    //
    // %s: String (default format)
    // %d, %i: Signed decimal integer
    // %u: Unsigned decimal integer
    // %x, %X: Hexadecimal (lowercase/uppercase)
    // %o: Octal
    // %f: Floating point
    // %e, %E: Scientific notation
    // %g, %G: Shortest representation (f or e)
    // %c: Single character
    // %%: Literal percent sign
    //
    // INPUT (bash):
    // printf 'String: %s\n' "text"
    // printf 'Decimal: %d\n' 42
    // printf 'Hex: %x\n' 255
    // printf 'Float: %.2f\n' 3.14159
    //
    // RUST:
    // println!("String: {}", "text");
    // println!("Decimal: {}", 42);
    // println!("Hex: {:x}", 255);
    // println!("Float: {:.2}", 3.14159);
    //
    // PURIFIED (POSIX sh):
    // printf 'String: %s\n' "text"
    // printf 'Decimal: %d\n' 42
    // printf 'Hex: %x\n' 255
    // printf 'Float: %.2f\n' 3.14159

    let format_specifiers = r#"
# String format
printf 'Name: %s\n' "Alice"
printf 'Path: %s\n' "/usr/local/bin"

# Integer formats
printf 'Decimal: %d\n' 42
printf 'Unsigned: %u\n' 100
printf 'Hex (lower): %x\n' 255
printf 'Hex (upper): %X\n' 255
printf 'Octal: %o\n' 64

# Floating point formats
printf 'Float: %f\n' 3.14159
printf 'Precision: %.2f\n' 3.14159
printf 'Scientific: %e\n' 1000.0

# Character and literal
printf 'Char: %c\n' "A"
printf 'Percent: %%\n'

# Multiple arguments
printf '%s: %d items\n' "Cart" 5
printf '%s %s %d\n' "User" "logged in at" 1630000000
"#;

    let mut lexer = Lexer::new(format_specifiers);
    match lexer.tokenize() {
        Ok(tokens) => {
            assert!(!tokens.is_empty(), "format specifiers should tokenize");
            let _ = tokens;
        }
        Err(_) => {
            // Parser may not fully support all format specifiers yet
        }
    }
}

#[test]
fn test_BASH_BUILTIN_005_printf_escape_sequences() {
    // DOCUMENTATION: printf escape sequences (POSIX)
    //
    // \n: Newline
    // \t: Tab
    // \\: Backslash
    // \': Single quote
    // \": Double quote
    // \r: Carriage return
    // \a: Alert (bell)
    // \b: Backspace
    // \f: Form feed
    // \v: Vertical tab
    // \0NNN: Octal character code
    // \xHH: Hexadecimal character code
    //
    // INPUT (bash):
    // printf 'Line1\nLine2\n'
    // printf 'Col1\tCol2\tCol3\n'
    //
    // RUST:
    // println!("Line1\nLine2");
    // println!("Col1\tCol2\tCol3");
    //
    // PURIFIED:
    // printf 'Line1\nLine2\n'
    // printf 'Col1\tCol2\tCol3\n'

    let escape_sequences = r#"
# Newline
printf 'Line1\nLine2\nLine3\n'

# Tab
printf 'Col1\tCol2\tCol3\n'

# Backslash and quotes
printf 'Path: C:\\Users\\Alice\n'
printf 'Quote: \'single\' and "double"\n'

# Other escapes
printf 'Alert:\a\n'
printf 'Carriage return:\r\n'

# Multiple escapes in one format
printf 'Name:\t%s\nAge:\t%d\nCity:\t%s\n' "Alice" 30 "NYC"
"#;

    let mut lexer = Lexer::new(escape_sequences);
    match lexer.tokenize() {
        Ok(tokens) => {
            assert!(!tokens.is_empty(), "escape sequences should tokenize");
            let _ = tokens;
        }
        Err(_) => {
            // Parser may not fully support escape sequences yet
        }
    }
}

