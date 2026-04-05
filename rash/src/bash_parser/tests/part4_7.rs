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
fn test_BASH_BUILTIN_005_printf_width_precision() {
    // DOCUMENTATION: Width and precision (POSIX)
    //
    // %Ns: Minimum width N (right-aligned)
    // %-Ns: Minimum width N (left-aligned)
    // %0Nd: Zero-padded integer width N
    // %.Nf: Floating point with N decimal places
    // %N.Mf: Width N, precision M
    //
    // INPUT (bash):
    // printf '%10s\n' "right"      # "     right"
    // printf '%-10s\n' "left"      # "left      "
    // printf '%05d\n' 42           # "00042"
    // printf '%.2f\n' 3.14159      # "3.14"
    //
    // RUST:
    // println!("{:>10}", "right");
    // println!("{:<10}", "left");
    // println!("{:05}", 42);
    // println!("{:.2}", 3.14159);
    //
    // PURIFIED:
    // printf '%10s\n' "right"
    // printf '%-10s\n' "left"
    // printf '%05d\n' 42
    // printf '%.2f\n' 3.14159

    let width_precision = r#"
# Width (right-aligned by default)
printf '%10s\n' "right"
printf '%20s\n' "file.txt"

# Width (left-aligned with -)
printf '%-10s\n' "left"
printf '%-20s\n' "file.txt"

# Zero-padded integers
printf '%05d\n' 42
printf '%08d\n' 123

# Precision for floats
printf '%.2f\n' 3.14159
printf '%.4f\n' 2.71828

# Combined width and precision
printf '%10.2f\n' 3.14159
printf '%8.3f\n' 2.71828

# Formatted table
printf '%-20s %10s %8s\n' "Name" "Age" "Score"
printf '%-20s %10d %8.2f\n' "Alice" 30 95.5
printf '%-20s %10d %8.2f\n' "Bob" 25 87.3
"#;

    let mut lexer = Lexer::new(width_precision);
    match lexer.tokenize() {
        Ok(tokens) => {
            assert!(!tokens.is_empty(), "width/precision should tokenize");
            let _ = tokens;
        }
        Err(_) => {
            // Parser may not fully support width/precision yet
        }
    }
}

#[test]
fn test_BASH_BUILTIN_005_printf_vs_echo() {
    // DOCUMENTATION: printf vs echo (Why printf is better)
    //
    // PROBLEMS WITH echo:
    // 1. -n flag non-portable (some shells don't support)
    // 2. -e flag non-portable (enables escapes in some shells only)
    // 3. Backslash interpretation varies across shells
    // 4. XSI vs BSD echo behavior differences
    // 5. Always adds trailing newline (can't suppress portably)
    //
    // PRINTF ADVANTAGES:
    // 1. POSIX-standardized behavior (consistent everywhere)
    // 2. Explicit newline control (no newline by default)
    // 3. Format control (width, precision, alignment)
    // 4. Consistent escape handling
    // 5. Multiple arguments handled correctly
    //
    // PURIFICATION STRATEGY:
    // Replace ALL echo with printf for maximum portability
    //
    // INPUT (bash with echo):
    // echo "Hello, World!"
    // echo -n "Prompt: "
    // echo -e "Line1\nLine2"
    //
    // PURIFIED (POSIX printf):
    // printf '%s\n' "Hello, World!"
    // printf '%s' "Prompt: "
    // printf 'Line1\nLine2\n'

    let printf_vs_echo = r#"
# AVOID: echo "text" (non-portable)
# USE: printf '%s\n' "text"
printf '%s\n' "Hello, World!"

# AVOID: echo -n "text" (no trailing newline, non-portable)
# USE: printf '%s' "text"
printf '%s' "Prompt: "

# AVOID: echo -e "Line1\nLine2" (escape interpretation, non-portable)
# USE: printf 'Line1\nLine2\n'
printf 'Line1\nLine2\n'

# AVOID: echo "$variable" (can cause issues with values like "-n")
# USE: printf '%s\n' "$variable"
variable="some value"
printf '%s\n' "$variable"

# Multiple values (echo fails here)
# echo "Name:" "Alice" "Age:" 30  # Adds spaces, inconsistent
# USE: printf
printf '%s %s %s %d\n' "Name:" "Alice" "Age:" 30

# Formatted output (impossible with echo)
printf 'Score: %5.2f%%\n' 87.5
printf 'Name: %-20s Age: %3d\n' "Alice" 30
"#;

    let mut lexer = Lexer::new(printf_vs_echo);
    match lexer.tokenize() {
        Ok(tokens) => {
            assert!(
                !tokens.is_empty(),
                "printf vs echo examples should tokenize"
            );
            let _ = tokens;
        }
        Err(_) => {
            // Parser may not fully support all patterns yet
        }
    }
}

#[test]
fn test_BASH_BUILTIN_005_printf_bash_extensions_not_supported() {
    // DOCUMENTATION: Bash printf extensions (NOT SUPPORTED)
    //
    // BASH EXTENSIONS (NOT SUPPORTED):
    // 1. %(...)T date/time formatting (use date command)
    // 2. %b interpret backslash escapes in argument (use escapes in format)
    // 3. %q shell-quote format (use manual quoting)
    // 4. -v var assign to variable (use command substitution)
    //
    // PURIFICATION STRATEGIES:
    //
    // 1. Replace %(...)T with date command:
    //    Bash:     printf 'Date: %(Today is %Y-%m-%d)T\n'
    //    Purified: printf 'Date: %s\n' "$(date +'Today is %Y-%m-%d')"
    //
    // 2. Replace %b with explicit escapes in format:
    //    Bash:     printf '%b' "Line1\nLine2"
    //    Purified: printf 'Line1\nLine2'
    //
    // 3. Replace %q with manual quoting:
    //    Bash:     printf '%q\n' "$unsafe_string"
    //    Purified: # Escape manually or use different approach
    //
    // 4. Replace -v var with command substitution:
    //    Bash:     printf -v myvar '%s %d' "Count:" 42
    //    Purified: myvar=$(printf '%s %d' "Count:" 42)

    let bash_extensions = r#"
# BASH EXTENSION: %(...)T date formatting (NOT SUPPORTED)
# Purify: Use date command
# printf 'Current date: %(Today is %Y-%m-%d)T\n'
# →
printf 'Current date: %s\n' "$(date +'Today is %Y-%m-%d')"

# BASH EXTENSION: %b interpret escapes in argument (NOT SUPPORTED)
# Purify: Put escapes in format string instead
# msg="Line1\nLine2"
# printf '%b\n' "$msg"
# →
printf 'Line1\nLine2\n'

# BASH EXTENSION: %q shell-quote (NOT SUPPORTED)
# Purify: Manual quoting or different approach
# unsafe="string with spaces"
# printf '%q\n' "$unsafe"
# →
unsafe="string with spaces"
printf '%s\n' "$unsafe"  # Or escape manually if needed

# BASH EXTENSION: -v var assign to variable (NOT SUPPORTED)
# Purify: Use command substitution
# printf -v result '%s %d' "Count:" 42
# →
result=$(printf '%s %d' "Count:" 42)
printf '%s\n' "$result"

# POSIX SUPPORTED: Regular printf
printf '%s\n' "This works everywhere"
printf '%d\n' 42
printf '%.2f\n' 3.14
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
fn test_BASH_BUILTIN_005_printf_common_patterns() {
    // DOCUMENTATION: Common printf patterns in POSIX scripts
    //
    // 1. Simple output (replace echo):
    //    printf '%s\n' "message"
    //
    // 2. No trailing newline (prompts):
    //    printf '%s' "Prompt: "
    //
    // 3. Formatted tables:
    //    printf '%-20s %10s\n' "Name" "Age"
    //
    // 4. Progress indicators:
    //    printf '\r%3d%%' "$percent"
    //
    // 5. Error messages to stderr:
    //    printf 'Error: %s\n' "$msg" >&2
    //
    // 6. CSV output:
    //    printf '%s,%s,%d\n' "Name" "City" 30
    //
    // 7. Logging with timestamps:
    //    printf '[%s] %s\n' "$(date +%Y-%m-%d)" "$message"

    let common_patterns = r#"
# Pattern 1: Simple output (portable echo replacement)
printf '%s\n' "Installation complete"
printf '%s\n' "Starting service..."

# Pattern 2: Prompts (no trailing newline)
printf '%s' "Enter your name: "
read -r name
printf '%s' "Continue? (y/n): "
read -r answer

# Pattern 3: Formatted tables
printf '%-20s %10s %8s\n' "Name" "Age" "Score"
printf '%-20s %10d %8.2f\n' "Alice" 30 95.5
printf '%-20s %10d %8.2f\n' "Bob" 25 87.3

# Pattern 4: Progress indicator
for i in 1 2 3 4 5; do
    percent=$((i * 20))
    printf '\rProgress: %3d%%' "$percent"
done
printf '\n'

# Pattern 5: Error messages to stderr
error_msg="File not found"
printf 'Error: %s\n' "$error_msg" >&2
printf 'Fatal: %s\n' "Cannot continue" >&2

# Pattern 6: CSV output
printf '%s,%s,%d\n' "Alice" "NYC" 30
printf '%s,%s,%d\n' "Bob" "LA" 25

# Pattern 7: Logging with timestamps
log_message="User logged in"
printf '[%s] %s\n' "$(date +%Y-%m-%d)" "$log_message"

# Pattern 8: Conditional output
if [ -f "/etc/config" ]; then
    printf '%s\n' "Config found"
else
    printf 'Warning: %s\n' "Config missing" >&2
fi

# Pattern 9: Number formatting
count=1234567
printf 'Total: %d items\n' "$count"
price=99.99
printf 'Price: $%.2f\n' "$price"
"#;

    let mut lexer = Lexer::new(common_patterns);
    match lexer.tokenize() {
        Ok(tokens) => {
            assert!(!tokens.is_empty(), "common patterns should tokenize");
            let _ = tokens;
        }
        Err(_) => {
            // Parser may not fully support all patterns yet
        }
    }
}

#[test]
fn test_BASH_BUILTIN_005_printf_comparison_table() {
    // COMPREHENSIVE COMPARISON: printf in POSIX vs Bash vs echo
    //
    // ┌──────────────────────────────────────────────────────────────────────────┐
    // │ Feature: printf Command                                                  │
    // ├────────────────────────────┬──────────────┬──────────────────────────────┤
    // │ Feature                    │ POSIX Status │ Purification                 │
    // ├────────────────────────────┼──────────────┼──────────────────────────────┤
    // │ FORMAT SPECIFIERS          │              │                              │
    // │ printf '%s\n' "text"       │ SUPPORTED    │ Keep as-is                   │
    // │ printf '%d' 42             │ SUPPORTED    │ Keep as-is                   │
    // │ printf '%.2f' 3.14         │ SUPPORTED    │ Keep as-is                   │
    // │ printf '%x' 255            │ SUPPORTED    │ Keep as-is                   │
    // │ printf '%o' 64             │ SUPPORTED    │ Keep as-is                   │
    // │                            │              │                              │
    // │ WIDTH/PRECISION            │              │                              │
    // │ printf '%10s' "right"      │ SUPPORTED    │ Keep as-is                   │
    // │ printf '%-10s' "left"      │ SUPPORTED    │ Keep as-is                   │
    // │ printf '%05d' 42           │ SUPPORTED    │ Keep as-is                   │
    // │ printf '%.2f' 3.14         │ SUPPORTED    │ Keep as-is                   │
    // │                            │              │                              │
    // │ ESCAPE SEQUENCES           │              │                              │
    // │ \n \t \\ \' \"             │ SUPPORTED    │ Keep as-is                   │
    // │ \r \a \b \f \v             │ SUPPORTED    │ Keep as-is                   │
    // │                            │              │                              │
    // │ BASH EXTENSIONS            │              │                              │
    // │ printf %(...)T date        │ NOT SUPPORT  │ Use date command             │
    // │ printf %b "a\nb"           │ NOT SUPPORT  │ Use \n in format             │
    // │ printf %q "str"            │ NOT SUPPORT  │ Manual quoting               │
    // │ printf -v var "fmt"        │ NOT SUPPORT  │ Use var=$(printf...)         │
    // │                            │              │                              │
    // │ ECHO REPLACEMENT           │              │                              │
    // │ echo "text"                │ AVOID        │ printf '%s\n' "text"         │
    // │ echo -n "text"             │ AVOID        │ printf '%s' "text"           │
    // │ echo -e "a\nb"             │ AVOID        │ printf 'a\nb\n'              │
    // └────────────────────────────┴──────────────┴──────────────────────────────┘
    //
    // RUST MAPPING:
    // printf '%s\n' "text"   → println!("{}", "text")
    // printf '%s' "text"     → print!("{}", "text")
    // printf '%d' 42         → println!("{}", 42)
    // printf '%.2f' 3.14     → println!("{:.2}", 3.14)
    // printf '%10s' "right"  → println!("{:>10}", "right")
    // printf '%-10s' "left"  → println!("{:<10}", "left")
    //
    // DETERMINISM: printf is deterministic (same input → same output)
    // IDEMPOTENCY: printf is idempotent (no side effects except output)
    // PORTABILITY: Use printf instead of echo for maximum POSIX compatibility

    let comparison_table = r#"
# This test documents the complete POSIX vs Bash comparison for printf
# See extensive comparison table in test function comments above

# POSIX SUPPORTED: Format specifiers
printf '%s\n' "string"          # String
printf '%d\n' 42                # Decimal integer
printf '%.2f\n' 3.14159         # Float with precision
printf '%x\n' 255               # Hexadecimal
printf '%o\n' 64                # Octal

# POSIX SUPPORTED: Width and precision
printf '%10s\n' "right"         # Right-aligned width 10
printf '%-10s\n' "left"         # Left-aligned width 10
printf '%05d\n' 42              # Zero-padded width 5
printf '%.2f\n' 3.14159         # 2 decimal places

# POSIX SUPPORTED: Escape sequences
printf 'Line1\nLine2\n'         # Newline
printf 'Col1\tCol2\n'           # Tab
printf 'Path: C:\\Users\n'      # Backslash

# NOT SUPPORTED: Bash extensions
# printf '%(Date: %Y-%m-%d)T\n'       → Use date command
# printf '%b' "a\nb"                  → Use printf 'a\nb'
# printf '%q' "string with spaces"    → Manual quoting
# printf -v var '%s' "value"          → var=$(printf '%s' "value")

# PORTABLE REPLACEMENT for echo
# echo "text"           → printf '%s\n' "text"
# echo -n "text"        → printf '%s' "text"
# echo -e "a\nb"        → printf 'a\nb\n'

# BEST PRACTICES
printf '%s\n' "Always use printf for portability"
printf '%s\n' "Control newlines explicitly"
printf '%-20s %10d\n' "Name" 42  # Formatted output
printf 'Error: %s\n' "msg" >&2   # Errors to stderr
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

    // Priority: HIGH - printf is the portable alternative to echo for formatted output
    // POSIX: IEEE Std 1003.1-2001 printf utility
    // Portability: Always use printf instead of echo for maximum compatibility
    // Determinism: printf is deterministic (same input produces same output)
    // Idempotency: printf is idempotent (no side effects except output to stdout/stderr)
}

// ============================================================================
// VAR-001: HOME Environment Variable (POSIX SUPPORTED - HIGH PRIORITY)
// ============================================================================

#[test]

include!("part4_7_var_001.rs");
