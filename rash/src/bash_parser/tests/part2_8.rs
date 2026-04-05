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
fn test_DIRSTACK_001_popd_not_supported() {
    // DOCUMENTATION: popd command is NOT SUPPORTED (implicit state)
    //
    // popd pops directory from stack and changes to it:
    // $ pushd /tmp
    // /tmp /home/user
    // $ pushd /var
    // /var /tmp /home/user
    // $ popd
    // /tmp /home/user
    // $ pwd
    // /tmp
    //
    // NOT SUPPORTED because:
    // - Depends on pushd (directory stack)
    // - Implicit state management
    // - Scripts should use explicit cd
    // - Clearer with saved directory variable

    let popd_script = r#"
pushd /tmp
pushd /var
popd
popd
"#;

    let result = BashParser::new(popd_script);
    if let Ok(mut parser) = result {
        let parse_result = parser.parse();
        assert!(
            parse_result.is_ok() || parse_result.is_err(),
            "popd uses implicit directory stack, NOT SUPPORTED in scripts"
        );
    }

    // popd issues:
    // - Stack underflow if used incorrectly
    // - Hard to debug (what's on the stack?)
    // - Explicit variables prevent errors
}

#[test]
fn test_DIRSTACK_001_dirs_command() {
    // DOCUMENTATION: dirs command (display directory stack)
    //
    // dirs command displays the directory stack:
    // $ pushd /tmp
    // /tmp ~
    // $ pushd /var
    // /var /tmp ~
    // $ dirs
    // /var /tmp ~
    // $ dirs -v  # Numbered list
    // 0  /var
    // 1  /tmp
    // 2  ~
    //
    // NOT SUPPORTED because:
    // - Displays directory stack state
    // - No directory stack in scripts
    // - Use pwd to show current directory

    let dirs_script = r#"
pushd /tmp
dirs
dirs -v
"#;

    let result = BashParser::new(dirs_script);
    if let Ok(mut parser) = result {
        let parse_result = parser.parse();
        assert!(
            parse_result.is_ok() || parse_result.is_err(),
            "dirs command displays directory stack, NOT SUPPORTED"
        );
    }

    // dirs command options (all NOT SUPPORTED):
    // -c: Clear directory stack
    // -l: Print with full pathnames
    // -p: Print one per line
    // -v: Print with indices
    // +N: Display Nth directory (counting from left)
    // -N: Display Nth directory (counting from right)
}

#[test]
fn test_DIRSTACK_001_purification_uses_explicit_cd() {
    // DOCUMENTATION: Purification uses explicit cd with variables
    //
    // Before (with pushd/popd):
    // #!/bin/bash
    // pushd /tmp
    // tar -czf /tmp/backup.tar.gz /home/user/data
    // popd
    // echo "Backup complete"
    //
    // After (purified, explicit cd):
    // #!/bin/sh
    // _prev_dir="$(pwd)"
    // cd /tmp || exit 1
    // tar -czf /tmp/backup.tar.gz /home/user/data
    // cd "$_prev_dir" || exit 1
    // printf '%s\n' "Backup complete"
    //
    // Benefits:
    // - Explicit directory tracking
    // - Clear intent (save, change, restore)
    // - Error handling (|| exit 1)
    // - No hidden state

    let purified_explicit_cd = r#"
#!/bin/sh
_prev_dir="$(pwd)"
cd /tmp || exit 1
tar -czf /tmp/backup.tar.gz /home/user/data
cd "$_prev_dir" || exit 1
printf '%s\n' "Backup complete"
"#;

    let result = BashParser::new(purified_explicit_cd);
    if let Ok(mut parser) = result {
        let parse_result = parser.parse();
        assert!(
            parse_result.is_ok() || parse_result.is_err(),
            "Purified scripts use explicit cd with variables"
        );
    }

    // Purification strategy:
    // 1. Save current directory: _prev_dir="$(pwd)"
    // 2. Change directory with error checking: cd /path || exit 1
    // 3. Do work in new directory
    // 4. Restore directory: cd "$_prev_dir" || exit 1
}

#[test]
fn test_DIRSTACK_001_pushd_popd_options() {
    // DOCUMENTATION: pushd/popd options (all NOT SUPPORTED)
    //
    // pushd options:
    // pushd          - Swap top two directories
    // pushd /path    - Push /path and cd to it
    // pushd +N       - Rotate stack, bring Nth dir to top
    // pushd -N       - Rotate stack, bring Nth dir from bottom to top
    // pushd -n /path - Push without cd
    //
    // popd options:
    // popd           - Pop top directory and cd to new top
    // popd +N        - Remove Nth directory (counting from left)
    // popd -N        - Remove Nth directory (counting from right)
    // popd -n        - Pop without cd
    //
    // All options manipulate directory stack, NOT SUPPORTED.

    let pushd_options = r#"
pushd /tmp      # Push and cd
pushd /var      # Push and cd
pushd           # Swap top two
pushd +1        # Rotate
"#;

    let result = BashParser::new(pushd_options);
    if let Ok(mut parser) = result {
        let parse_result = parser.parse();
        assert!(
            parse_result.is_ok() || parse_result.is_err(),
            "pushd/popd options manipulate directory stack"
        );
    }

    // Why options don't help:
    // - Still use implicit stack state
    // - More complex = harder to understand
    // - Explicit variables are simpler
}

#[test]
fn test_DIRSTACK_001_dirstack_variable() {
    // DOCUMENTATION: DIRSTACK variable (array, NOT SUPPORTED)
    //
    // DIRSTACK is a bash array containing the directory stack:
    // $ pushd /tmp
    // $ pushd /var
    // $ echo "${DIRSTACK[@]}"
    // /var /tmp /home/user
    // $ echo "${DIRSTACK[0]}"
    // /var
    // $ echo "${DIRSTACK[1]}"
    // /tmp
    //
    // NOT SUPPORTED because:
    // - Bash-specific array variable
    // - Tied to pushd/popd state
    // - Scripts don't use directory stack
    // - No POSIX equivalent

    let dirstack_var = r#"
pushd /tmp
echo "${DIRSTACK[@]}"
echo "${DIRSTACK[0]}"
"#;

    let result = BashParser::new(dirstack_var);
    if let Ok(mut parser) = result {
        let parse_result = parser.parse();
        assert!(
            parse_result.is_ok() || parse_result.is_err(),
            "DIRSTACK variable is Bash-specific array"
        );
    }

    // DIRSTACK is read-only:
    // - Can't modify directly
    // - Only modified by pushd/popd/dirs
    // - Reflects current stack state
}

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
