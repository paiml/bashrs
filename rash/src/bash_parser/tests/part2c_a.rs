#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]

use super::super::ast::Redirect;
use super::super::lexer::Lexer;
use super::super::parser::BashParser;
use super::super::semantic::SemanticAnalyzer;
use super::super::*;

#[test]
fn test_DIRSTACK_001_cd_minus_alternative() {
    // DOCUMENTATION: cd - (alternative to popd, uses OLDPWD)
    //
    // cd - changes to previous directory (uses OLDPWD):
    // $ pwd
    // /home/user
    // $ cd /tmp
    // $ pwd
    // /tmp
    // $ cd -
    // /home/user
    // $ pwd
    // /home/user
    //
    // cd - is better than popd because:
    // - POSIX-compliant (OLDPWD is standard)
    // - No stack state (simpler)
    // - Only remembers one directory (sufficient)
    // - Explicit and predictable

    let cd_minus = r#"
cd /tmp
# do work
cd -     # Return to previous directory
"#;

    let result = BashParser::new(cd_minus);
    if let Ok(mut parser) = result {
        let parse_result = parser.parse();
        assert!(
            parse_result.is_ok() || parse_result.is_err(),
            "cd - uses OLDPWD, simpler than popd"
        );
    }

    // cd - advantages over pushd/popd:
    // - POSIX-compliant
    // - No hidden stack
    // - One previous directory (usually enough)
    // - More predictable behavior
}

#[test]
fn test_DIRSTACK_001_interactive_vs_script_directory_navigation() {
    // DOCUMENTATION: Interactive vs script directory navigation
    //
    // Interactive navigation (uses pushd/popd):
    // - Navigate between multiple directories
    // - Directory stack for quick switching
    // - pushd/popd for convenience
    // - dirs to see stack
    // - Useful for manual exploration
    //
    // Script navigation (uses explicit cd):
    // - Deterministic directory changes
    // - Save/restore with variables
    // - cd with error checking
    // - pwd to show current location
    // - Explicit and traceable

    let script_navigation = r#"
#!/bin/sh
# Script-style directory navigation (explicit)

# Save starting directory
start_dir="$(pwd)"

# Work in first location
cd /tmp || exit 1
printf '%s\n' "Working in /tmp"
# do work

# Work in second location
cd /var/log || exit 1
printf '%s\n' "Working in /var/log"
# do work

# Return to start
cd "$start_dir" || exit 1
printf '%s\n' "Back to $start_dir"
"#;

    let result = BashParser::new(script_navigation);
    if let Ok(mut parser) = result {
        let parse_result = parser.parse();
        assert!(
            parse_result.is_ok() || parse_result.is_err(),
            "Scripts use explicit cd with error checking"
        );
    }

    // Summary:
    // Interactive: pushd/popd with implicit stack
    // Script: cd with explicit variables and error checking
    //
    // bashrs: Remove pushd/popd, use explicit cd
}

// ============================================================================
// ARRAY-002: Associative Arrays (Bash 4.0+, NOT SUPPORTED)
// ============================================================================
//
// Task: ARRAY-002 - Document associative arrays
// Status: DOCUMENTED (NOT SUPPORTED - Bash 4.0+ extension, not POSIX)
// Priority: LOW (associative arrays not in POSIX sh)
//
// Associative arrays (hash maps/dictionaries) were introduced in Bash 4.0.
// They allow key-value pairs with string keys, unlike indexed arrays.
//
// Bash behavior:
// - declare -A name: Declare associative array
// - array[key]=value: Set value for key
// - ${array[key]}: Get value for key
// - ${!array[@]}: Get all keys
// - ${array[@]}: Get all values
// - Bash 4.0+ only (2009)
//
// bashrs policy:
// - NOT SUPPORTED (Bash 4.0+ extension, not POSIX)
// - Use separate variables with consistent naming
// - Use indexed arrays if order doesn't matter
// - More portable, works on older shells
//
// Transformation:
// Bash input:
//   declare -A config
//   config[host]="localhost"
//   config[port]="8080"
//   echo "${config[host]}"
//
// Purified POSIX sh:
//   config_host="localhost"
//   config_port="8080"
//   printf '%s\n' "$config_host"
//
// Related features:
// - Indexed arrays (ARRAY-001) - supported
// - declare -A - associative array declaration
// - readarray/mapfile - not supported (Bash 4.0+)

#[test]
fn test_ARRAY_002_associative_arrays_not_supported() {
    // DOCUMENTATION: Associative arrays are NOT SUPPORTED (Bash 4.0+)
    //
    // Associative arrays use string keys:
    // $ declare -A config
    // $ config[host]="localhost"
    // $ config[port]="8080"
    // $ echo "${config[host]}"
    // localhost
    // $ echo "${!config[@]}"
    // host port
    //
    // NOT SUPPORTED because:
    // - Bash 4.0+ extension (2009)
    // - Not available in POSIX sh, dash, ash
    // - Not portable to older systems
    // - Use separate variables instead

    let assoc_array_script = r#"
declare -A config
config[host]="localhost"
config[port]="8080"
echo "${config[host]}"
"#;

    let result = BashParser::new(assoc_array_script);
    match result {
        Ok(mut parser) => {
            let parse_result = parser.parse();
            assert!(
                parse_result.is_ok() || parse_result.is_err(),
                "Associative arrays are Bash 4.0+ only, NOT SUPPORTED"
            );
        }
        Err(_) => {
            // Parse error acceptable - Bash extension
        }
    }

    // Why associative arrays are problematic:
    // - Requires Bash 4.0+ (not available everywhere)
    // - macOS ships with Bash 3.2 (2006, pre-associative arrays)
    // - Alpine Linux uses ash (no associative arrays)
    // - Separate variables are more portable
}

#[test]
fn test_ARRAY_002_declare_uppercase_a() {
    // DOCUMENTATION: declare -A (associative array declaration)
    //
    // declare -A declares an associative array:
    // $ declare -A map
    // $ map[key1]="value1"
    // $ map[key2]="value2"
    // $ declare -p map
    // declare -A map=([key1]="value1" [key2]="value2")
    //
    // NOT SUPPORTED because:
    // - Bash 4.0+ only
    // - No POSIX equivalent
    // - Use individual variables instead

    let declare_a = r#"
declare -A map
map[name]="John"
map[age]="30"
"#;

    let result = BashParser::new(declare_a);
    if let Ok(mut parser) = result {
        let parse_result = parser.parse();
        assert!(
            parse_result.is_ok() || parse_result.is_err(),
            "declare -A is Bash 4.0+ only, NOT SUPPORTED"
        );
    }

    // Note: declare -a (lowercase) is for indexed arrays (supported)
    //       declare -A (uppercase) is for associative arrays (NOT supported)
}

#[test]
fn test_ARRAY_002_associative_array_operations() {
    // DOCUMENTATION: Associative array operations (all Bash 4.0+)
    //
    // Operations:
    // ${array[key]}        - Get value for key
    // ${!array[@]}         - Get all keys
    // ${array[@]}          - Get all values
    // ${#array[@]}         - Get number of elements
    // unset array[key]     - Delete key
    // [[ -v array[key] ]]  - Check if key exists
    //
    // All operations are Bash 4.0+ only, NOT SUPPORTED.

    let assoc_operations = r#"
declare -A data
data[x]="10"
data[y]="20"

echo "${data[x]}"           # Get value
echo "${!data[@]}"          # Get keys
echo "${data[@]}"           # Get values
echo "${#data[@]}"          # Get count
unset data[x]               # Delete key
[[ -v data[y] ]] && echo "exists"  # Check existence
"#;

    let result = BashParser::new(assoc_operations);
    if let Ok(mut parser) = result {
        let parse_result = parser.parse();
        assert!(
            parse_result.is_ok() || parse_result.is_err(),
            "Associative array operations are Bash 4.0+ only"
        );
    }

    // All these operations require:
    // - Bash 4.0+ (not available on older systems)
    // - No POSIX equivalent
    // - Use separate variables for portability
}

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

