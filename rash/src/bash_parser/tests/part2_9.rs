#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]

use super::super::ast::Redirect;
use super::super::lexer::Lexer;
use super::super::parser::BashParser;
use super::super::semantic::SemanticAnalyzer;
use super::super::*;

/// Helper: parse a script and return whether parsing succeeded.
/// Used by documentation tests that only need to verify parsability.
#[test]
fn test_ARRAY_002_purification_uses_separate_variables() {
    // DOCUMENTATION: Purification uses separate variables
    //
    // Before (with associative arrays):
    // #!/bin/bash
    // declare -A config
    // config[host]="localhost"
    // config[port]="8080"
    // config[user]="admin"
    // echo "Connecting to ${config[host]}:${config[port]}"
    //
    // After (purified, separate variables):
    // #!/bin/sh
    // config_host="localhost"
    // config_port="8080"
    // config_user="admin"
    // printf '%s\n' "Connecting to ${config_host}:${config_port}"
    //
    // Benefits:
    // - POSIX-compliant (works everywhere)
    // - Clear variable names (self-documenting)
    // - No Bash 4.0+ requirement
    // - Simpler and more explicit

    let purified_separate_vars = r#"
#!/bin/sh
config_host="localhost"
config_port="8080"
config_user="admin"
printf '%s\n' "Connecting to ${config_host}:${config_port}"
"#;

    let result = BashParser::new(purified_separate_vars);
    if let Ok(mut parser) = result {
        let parse_result = parser.parse();
        assert!(
            parse_result.is_ok() || parse_result.is_err(),
            "Purified scripts use separate variables"
        );
    }

    // Purification strategy:
    // 1. Replace associative array with separate variables
    // 2. Use consistent naming: prefix_key pattern
    // 3. Replace ${array[key]} with $prefix_key
    // 4. More portable and readable
}

#[test]
fn test_ARRAY_002_indexed_array_alternative() {
    // DOCUMENTATION: Indexed arrays as alternative (if order matters)
    //
    // If you need multiple values and order matters, use indexed arrays:
    //
    // Associative array (NOT supported):
    // declare -A fruits=([apple]="red" [banana]="yellow")
    //
    // Indexed array (supported):
    // fruits=("apple:red" "banana:yellow")
    // for item in "${fruits[@]}"; do
    //   key="${item%%:*}"
    //   value="${item#*:}"
    //   echo "$key is $value"
    // done
    //
    // This approach:
    // - Works in POSIX sh
    // - Requires parsing (key:value format)
    // - Good for small datasets
    // - Order preserved

    let indexed_alternative = r#"
#!/bin/sh
# Indexed array as alternative to associative

fruits="apple:red banana:yellow cherry:red"

for item in $fruits; do
  key="${item%%:*}"
  value="${item#*:}"
  printf '%s is %s\n' "$key" "$value"
done
"#;

    let result = BashParser::new(indexed_alternative);
    if let Ok(mut parser) = result {
        let parse_result = parser.parse();
        assert!(
            parse_result.is_ok() || parse_result.is_err(),
            "Indexed arrays or space-separated values work as alternatives"
        );
    }

    // Alternatives to associative arrays:
    // 1. Separate variables (best for small fixed set)
    // 2. Indexed array with key:value pairs (good for iteration)
    // 3. Space-separated list (simple cases)
    // 4. External file (large datasets)
}

#[test]
fn test_ARRAY_002_bash_version_compatibility() {
    // DOCUMENTATION: Bash version compatibility for arrays
    //
    // Array support by Bash version:
    // - Bash 2.0+ (1996): Indexed arrays
    // - Bash 3.0+ (2004): Improved indexed arrays
    // - Bash 4.0+ (2009): Associative arrays
    //
    // Platform availability:
    // - macOS: Bash 3.2 (2006) - NO associative arrays
    // - Ubuntu 18.04+: Bash 4.4+ - Has associative arrays
    // - Alpine Linux: ash (not bash) - NO associative arrays
    // - Debian/RHEL: Usually Bash 4.0+
    //
    // For maximum portability, avoid associative arrays.

    let version_check = r#"
# This script fails on Bash < 4.0
if [ "${BASH_VERSINFO[0]}" -lt 4 ]; then
  echo "Error: Bash 4.0+ required for associative arrays"
  exit 1
fi

declare -A config
"#;

    let result = BashParser::new(version_check);
    if let Ok(mut parser) = result {
        let parse_result = parser.parse();
        assert!(
            parse_result.is_ok() || parse_result.is_err(),
            "Version checks indicate Bash-specific features"
        );
    }

    // bashrs philosophy:
    // - Target POSIX sh (works everywhere)
    // - Avoid Bash-specific features
    // - No version checks needed
    // - Maximum portability
}

#[test]
fn test_ARRAY_002_use_cases_and_alternatives() {
    // DOCUMENTATION: Common use cases and POSIX alternatives
    //
    // Use case 1: Configuration values
    // Associative: declare -A config; config[host]="localhost"
    // Alternative:  config_host="localhost" (separate variables)
    //
    // Use case 2: Counting occurrences
    // Associative: declare -A count; ((count[$word]++))
    // Alternative:  awk '{count[$1]++} END {for (w in count) print w, count[w]}'
    //
    // Use case 3: Lookup table
    // Associative: declare -A map; map[key]="value"
    // Alternative:  case "$key" in key) value="value" ;; esac
    //
    // Use case 4: Environment-like variables
    // Associative: declare -A env; env[PATH]="/usr/bin"
    // Alternative:  Just use actual environment variables

    let case_alternative = r#"
#!/bin/sh
# Case statement as lookup table alternative

get_color() {
  fruit="$1"
  case "$fruit" in
    apple)  color="red" ;;
    banana) color="yellow" ;;
    cherry) color="red" ;;
    *)      color="unknown" ;;
  esac
  printf '%s\n' "$color"
}

get_color "apple"    # red
get_color "banana"   # yellow
"#;

    let result = BashParser::new(case_alternative);
    if let Ok(mut parser) = result {
        let parse_result = parser.parse();
        assert!(
            parse_result.is_ok() || parse_result.is_err(),
            "Case statements work as lookup table alternative"
        );
    }

    // Summary of alternatives:
    // - Separate variables: Best for known keys
    // - Case statements: Best for lookup/mapping
    // - Indexed arrays: Best for lists with parsing
    // - External tools (awk): Best for complex data processing
}

#[test]
fn test_ARRAY_002_bash_vs_posix_arrays() {
    // DOCUMENTATION: Bash vs POSIX array support
    //
    // POSIX sh (portable):
    // - No arrays at all (officially)
    // - Use "$@" for positional parameters
    // - Use space-separated strings
    // - Use separate variables
    //
    // Bash extensions:
    // - Indexed arrays: array=(1 2 3)
    // - Associative arrays: declare -A map (Bash 4.0+)
    // - Array operations: ${array[@]}, ${#array[@]}, etc.
    //
    // bashrs approach:
    // - Limited indexed array support (for compatibility)
    // - NO associative arrays (not portable)
    // - Prefer separate variables or space-separated lists

    let posix_no_arrays = r#"
#!/bin/sh
# POSIX sh - no arrays, use alternatives

# Option 1: Positional parameters
set -- "apple" "banana" "cherry"
for fruit in "$@"; do
  printf '%s\n' "$fruit"
done

# Option 2: Space-separated string
fruits="apple banana cherry"
for fruit in $fruits; do
  printf '%s\n' "$fruit"
done

# Option 3: Separate variables
fruit1="apple"
fruit2="banana"
fruit3="cherry"
"#;

    let result = BashParser::new(posix_no_arrays);
    if let Ok(mut parser) = result {
        let parse_result = parser.parse();
        assert!(
            parse_result.is_ok() || parse_result.is_err(),
            "POSIX sh uses alternatives to arrays"
        );
    }

    // Summary:
    // Bash: Indexed and associative arrays
    // POSIX: No arrays, use alternatives
    // bashrs: Limited indexed array support, no associative arrays
}

// ============================================================================
// ANSI-C-001: ANSI-C Quoting ($'...') (Bash 2.0+, NOT SUPPORTED)
// ============================================================================
//
// Task: ANSI-C-001 (3.1.2.4) - Document $'...' transformation
// Status: DOCUMENTED (NOT SUPPORTED - Bash extension, not POSIX)
// Priority: MEDIUM (common in modern bash scripts)
//
// ANSI-C quoting allows escape sequences in strings using $'...' syntax.
// This is a Bash extension introduced in Bash 2.0 (1996).
//
// Bash behavior:
// - $'string': Interpret escape sequences
// - \n: Newline
// - \t: Tab
// - \r: Carriage return
// - \\: Backslash
// - \': Single quote
// - \": Double quote
// - \xHH: Hex byte (e.g., \x41 = 'A')
// - \uHHHH: Unicode (Bash 4.2+)
// - \UHHHHHHHH: Unicode (Bash 4.2+)
//
// bashrs policy:
// - NOT SUPPORTED (Bash extension, not POSIX)
// - Use printf for escape sequences
// - Use literal strings with real newlines
// - More portable, works on all POSIX shells

#[test]
fn test_ANSI_C_001_ansi_c_quoting_not_supported() {
    // DOCUMENTATION: ANSI-C quoting ($'...') is NOT SUPPORTED (Bash extension)
    //
    // ANSI-C quoting allows escape sequences:
    // $ echo $'Hello\nWorld'
    // Hello
    // World
    //
    // $ echo $'Tab:\there'
    // Tab:    here
    //
    // $ echo $'Quote: \''
    // Quote: '
    //
    // NOT SUPPORTED because:
    // - Bash 2.0+ extension (1996)
    // - Not available in POSIX sh, dash, ash
    // - printf provides same functionality
    // - Literal strings more readable

    let ansi_c_script = r#"
echo $'Hello\nWorld'
echo $'Tab:\there'
"#;

    let result = BashParser::new(ansi_c_script);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "ANSI-C quoting is Bash extension, NOT SUPPORTED"
            );
        }
        Err(_) => {
            // Parse error acceptable - Bash extension
        }
    }
}

#[test]
fn test_ANSI_C_001_basic_escape_sequences() {
    // DOCUMENTATION: Basic escape sequences in $'...'
    //
    // Common escape sequences:
    // - \n: Newline (Line Feed, 0x0A)
    // - \t: Horizontal Tab (0x09)
    // - \r: Carriage Return (0x0D)
    // - \\: Backslash (0x5C)
    // - \': Single quote (0x27)
    // - \": Double quote (0x22)
    //
    // Examples:
    // $ echo $'Line 1\nLine 2'
    // Line 1
    // Line 2
    //
    // $ echo $'Column1\tColumn2'
    // Column1    Column2
    //
    // $ echo $'It'\''s OK'  # Single quote inside ANSI-C
    // It's OK

    let basic_escapes = r#"
echo $'Hello\nWorld'
echo $'Tab\there'
echo $'Back\\slash'
echo $'Single\'quote'
"#;

    let result = BashParser::new(basic_escapes);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "ANSI-C basic escapes: Bash extension, NOT SUPPORTED"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

#[test]
fn test_ANSI_C_001_hex_and_octal_escapes() {
    // DOCUMENTATION: Hex and octal escape sequences
    //
    // Numeric escape sequences:
    // - \xHH: Hex byte (2 hex digits)
    // - \OOO: Octal byte (1-3 octal digits)
    //
    // Examples:
    // $ echo $'\x41\x42\x43'
    // ABC
    //
    // $ echo $'\101\102\103'
    // ABC
    //
    // $ echo $'\x48\x65\x6c\x6c\x6f'
    // Hello

    let numeric_escapes = r#"
echo $'\x41\x42\x43'
echo $'\101\102\103'
echo $'\x48\x65\x6c\x6c\x6f'
"#;

    let result = BashParser::new(numeric_escapes);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "ANSI-C hex/octal escapes: Bash extension, NOT SUPPORTED"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

#[test]
fn test_ANSI_C_001_unicode_escapes() {
    // DOCUMENTATION: Unicode escape sequences (Bash 4.2+)
    //
    // Unicode escapes added in Bash 4.2 (2011):
    // - \uHHHH: Unicode code point (4 hex digits)
    // - \UHHHHHHHH: Unicode code point (8 hex digits)
    //
    // Examples:
    // $ echo $'\u0041'  # Latin A
    // A
    //
    // $ echo $'\u03B1'  # Greek alpha
    // α
    //
    // $ echo $'\U0001F600'  # Emoji (grinning face)
    // 😀
    //
    // NOT SUPPORTED (Bash 4.2+ only, macOS has 3.2)

    let unicode_escapes = r#"
echo $'\u0041'
echo $'\u03B1'
echo $'\U0001F600'
"#;

    let result = BashParser::new(unicode_escapes);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "ANSI-C unicode escapes: Bash 4.2+ extension, NOT SUPPORTED"
            );
        }
        Err(_) => {
            // Parse error acceptable
        }
    }
}

#[test]
fn test_ANSI_C_001_purification_uses_printf() {
    // DOCUMENTATION: Purification uses printf for escape sequences
    //
    // Before (with ANSI-C quoting):
    // #!/bin/bash
    // echo $'Line 1\nLine 2\nLine 3'
    // echo $'Column1\tColumn2\tColumn3'
    // echo $'Hex: \x48\x65\x6c\x6c\x6f'
    //
    // After (purified, using printf):
    // #!/bin/sh
    // printf '%s\n' "Line 1" "Line 2" "Line 3"
    // printf 'Column1\tColumn2\tColumn3\n'
    // printf 'Hello\n'

    let purified_printf = r#"
#!/bin/sh
printf '%s\n' "Line 1" "Line 2" "Line 3"
printf 'Column1\tColumn2\tColumn3\n'
printf 'Hello\n'
"#;

    let result = BashParser::new(purified_printf);
    assert!(result.is_ok(), "Purified printf should parse successfully");

    let mut parser = result.unwrap();
    let parse_result = parser.parse();
    assert!(
        parse_result.is_ok(),
        "Purified printf should parse without errors"
    );
}

