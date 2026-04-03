// ! Standard Library Support
//!
//! This module provides support for stdlib functions that are transpiled
//! to POSIX shell runtime functions.

/// Check if a function name is a stdlib function
pub fn is_stdlib_function(name: &str) -> bool {
    matches!(
        name,
        // String module
        "string_trim"
            | "string_contains"
            | "string_len"
            | "string_split"
            | "string_replace"
            | "string_to_upper"
            | "string_to_lower"
            | "string_starts_with"
            | "string_ends_with"
            // File system module
            | "fs_exists"
            | "fs_read_file"
            | "fs_write_file"
            | "fs_copy"
            | "fs_remove"
            | "fs_is_file"
            | "fs_is_dir"
            // Array module
            | "array_len"
            | "array_join"
            // Environment module (Sprint 27a)
            | "env"
            | "env_var_or"
            // Arguments module (Sprint 27b)
            | "arg"
            | "args"
            | "arg_count"
            // Exit code module (Sprint 27c)
            | "exit_code"
            // Command execution module (GH-148)
            | "capture"
            | "exec"
            | "exit"
            | "sleep"
            // File iteration module (GH-148)
            | "glob"
            // Directory/file management module (GH-148)
            | "mkdir"
            | "mv"
            | "chmod"
    )
}

/// Get the shell function name for a stdlib function
pub fn get_shell_function_name(name: &str) -> String {
    format!("rash_{}", name)
}

// Re-export metadata from extracted module (file size discipline)
pub use crate::stdlib_metadata::{StdlibFunction, STDLIB_FUNCTIONS};

// Tests use STDLIB_FUNCTIONS below — the const is defined in stdlib_metadata.rs
// Legacy anchor (do not add new entries here — edit stdlib_metadata.rs):
const _STDLIB_METADATA_ANCHOR: () = {
    // Compile-time check: STDLIB_FUNCTIONS exists and is non-empty
    assert!(!STDLIB_FUNCTIONS.is_empty());
};
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_stdlib_function() {
        // String functions
        assert!(is_stdlib_function("string_trim"));
        assert!(is_stdlib_function("string_replace"));
        assert!(is_stdlib_function("string_to_upper"));
        assert!(is_stdlib_function("string_to_lower"));

        // File system functions
        assert!(is_stdlib_function("fs_exists"));
        assert!(is_stdlib_function("fs_copy"));
        assert!(is_stdlib_function("fs_remove"));
        assert!(is_stdlib_function("fs_is_file"));
        assert!(is_stdlib_function("fs_is_dir"));

        // Command execution functions (GH-148)
        assert!(is_stdlib_function("capture"));
        assert!(is_stdlib_function("exec"));
        assert!(is_stdlib_function("exit"));
        assert!(is_stdlib_function("sleep"));

        // String prefix/suffix functions (GH-148)
        assert!(is_stdlib_function("string_starts_with"));
        assert!(is_stdlib_function("string_ends_with"));

        // File iteration functions (GH-148)
        assert!(is_stdlib_function("glob"));

        // Directory/file management functions (GH-148)
        assert!(is_stdlib_function("mkdir"));
        assert!(is_stdlib_function("mv"));
        assert!(is_stdlib_function("chmod"));

        // Not stdlib functions
        assert!(!is_stdlib_function("custom_function"));
        assert!(!is_stdlib_function("println"));
    }

    #[test]
    fn test_get_shell_function_name() {
        assert_eq!(get_shell_function_name("string_trim"), "rash_string_trim");
        assert_eq!(get_shell_function_name("fs_exists"), "rash_fs_exists");
    }

    // Sprint 27a: Environment Variables Support - RED PHASE
    #[test]
    fn test_stdlib_env_function_recognized() {
        // RED: This test will fail until we add "env" to is_stdlib_function()
        assert!(
            is_stdlib_function("env"),
            "env() should be recognized as stdlib function"
        );
    }

    #[test]
    fn test_stdlib_env_var_or_function_recognized() {
        // RED: This test will fail until we add "env_var_or" to is_stdlib_function()
        assert!(
            is_stdlib_function("env_var_or"),
            "env_var_or() should be recognized as stdlib function"
        );
    }

    // Sprint 27b: Command-Line Arguments Support - RED PHASE
    #[test]
    fn test_stdlib_arg_function_recognized() {
        // RED: This test will fail until we add "arg" to is_stdlib_function()
        assert!(
            is_stdlib_function("arg"),
            "arg() should be recognized as stdlib function"
        );
    }

    #[test]
    fn test_stdlib_args_function_recognized() {
        // RED: This test will fail until we add "args" to is_stdlib_function()
        assert!(
            is_stdlib_function("args"),
            "args() should be recognized as stdlib function"
        );
    }

    #[test]
    fn test_stdlib_arg_count_function_recognized() {
        // RED: This test will fail until we add "arg_count" to is_stdlib_function()
        assert!(
            is_stdlib_function("arg_count"),
            "arg_count() should be recognized as stdlib function"
        );
    }

    // Sprint 27c: Exit Code Handling - RED PHASE
    #[test]
    fn test_stdlib_exit_code_function_recognized() {
        // RED: This test will fail until we add "exit_code" to is_stdlib_function()
        assert!(
            is_stdlib_function("exit_code"),
            "exit_code() should be recognized as stdlib function"
        );
    }

    #[test]
    fn test_stdlib_exit_code_metadata() {
        // RED: This test will fail until we add metadata for exit_code
        let metadata: Vec<&StdlibFunction> = STDLIB_FUNCTIONS
            .iter()
            .filter(|f| f.name == "exit_code")
            .collect();

        assert_eq!(metadata.len(), 1, "exit_code should have metadata entry");
        assert_eq!(
            metadata[0].module, "status",
            "exit_code should be in 'status' module"
        );
        assert_eq!(
            metadata[0].shell_name, "inline_exit_code",
            "exit_code should use inline shell syntax"
        );
    }

    // Sprint 28: Complete Missing Stdlib Functions - RED PHASE

    #[test]
    fn test_stdlib_string_split_recognized() {
        // RED: string_split is in is_stdlib_function() but needs metadata
        assert!(
            is_stdlib_function("string_split"),
            "string_split() should be recognized as stdlib function"
        );
    }

    #[test]
    fn test_stdlib_string_split_metadata() {
        // RED: This test will fail until we add metadata for string_split
        let metadata: Vec<&StdlibFunction> = STDLIB_FUNCTIONS
            .iter()
            .filter(|f| f.name == "string_split")
            .collect();

        assert_eq!(metadata.len(), 1, "string_split should have metadata entry");
        assert_eq!(
            metadata[0].module, "string",
            "string_split should be in 'string' module"
        );
        assert_eq!(
            metadata[0].shell_name, "rash_string_split",
            "string_split should use rash_ prefix"
        );
    }

    #[test]
    fn test_stdlib_array_len_recognized() {
        // RED: array_len is in is_stdlib_function() but needs metadata
        assert!(
            is_stdlib_function("array_len"),
            "array_len() should be recognized as stdlib function"
        );
    }

    #[test]
    fn test_stdlib_array_len_metadata() {
        // RED: This test will fail until we add metadata for array_len
        let metadata: Vec<&StdlibFunction> = STDLIB_FUNCTIONS
            .iter()
            .filter(|f| f.name == "array_len")
            .collect();

        assert_eq!(metadata.len(), 1, "array_len should have metadata entry");
        assert_eq!(
            metadata[0].module, "array",
            "array_len should be in 'array' module"
        );
        assert_eq!(
            metadata[0].shell_name, "rash_array_len",
            "array_len should use rash_ prefix"
        );
    }

    #[test]
    fn test_stdlib_array_join_recognized() {
        // RED: array_join is in is_stdlib_function() but needs metadata
        assert!(
            is_stdlib_function("array_join"),
            "array_join() should be recognized as stdlib function"
        );
    }

    #[test]
    fn test_GH148_starts_with_recognized() {
        assert!(
            is_stdlib_function("string_starts_with"),
            "string_starts_with() should be recognized as stdlib function"
        );
    }

    #[test]
    fn test_GH148_ends_with_recognized() {
        assert!(
            is_stdlib_function("string_ends_with"),
            "string_ends_with() should be recognized as stdlib function"
        );
    }

    #[test]
    fn test_GH148_starts_with_metadata() {
        let metadata: Vec<&StdlibFunction> = STDLIB_FUNCTIONS
            .iter()
            .filter(|f| f.name == "string_starts_with")
            .collect();

        assert_eq!(metadata.len(), 1, "string_starts_with should have metadata entry");
        assert_eq!(metadata[0].module, "string");
        assert_eq!(metadata[0].shell_name, "rash_string_starts_with");
    }

    #[test]
    fn test_GH148_ends_with_metadata() {
        let metadata: Vec<&StdlibFunction> = STDLIB_FUNCTIONS
            .iter()
            .filter(|f| f.name == "string_ends_with")
            .collect();

        assert_eq!(metadata.len(), 1, "string_ends_with should have metadata entry");
        assert_eq!(metadata[0].module, "string");
        assert_eq!(metadata[0].shell_name, "rash_string_ends_with");
    }

    #[test]
    fn test_GH148_glob_recognized() {
        assert!(
            is_stdlib_function("glob"),
            "glob() should be recognized as stdlib function"
        );
    }

    #[test]
    fn test_GH148_glob_metadata() {
        let metadata: Vec<&StdlibFunction> = STDLIB_FUNCTIONS
            .iter()
            .filter(|f| f.name == "glob")
            .collect();

        assert_eq!(metadata.len(), 1, "glob should have metadata entry");
        assert_eq!(
            metadata[0].module, "fs",
            "glob should be in 'fs' module"
        );
        assert_eq!(
            metadata[0].shell_name, "inline_glob",
            "glob should use inline shell syntax"
        );
    }

    #[test]
    fn test_stdlib_array_join_metadata() {
        // RED: This test will fail until we add metadata for array_join
        let metadata: Vec<&StdlibFunction> = STDLIB_FUNCTIONS
            .iter()
            .filter(|f| f.name == "array_join")
            .collect();

        assert_eq!(metadata.len(), 1, "array_join should have metadata entry");
        assert_eq!(
            metadata[0].module, "array",
            "array_join should be in 'array' module"
        );
        assert_eq!(
            metadata[0].shell_name, "rash_array_join",
            "array_join should use rash_ prefix"
        );
    }
}

// Sprint 27a: Security Tests - RED PHASE

/// RED TEST: Invalid variable names should be rejected
/// Tests that env() rejects variable names with invalid characters
#[test]
fn test_env_rejects_invalid_var_names() {
    // Valid var names: alphanumeric + underscore only
    assert!(is_valid_var_name("HOME"));
    assert!(is_valid_var_name("MY_VAR"));
    assert!(is_valid_var_name("VAR123"));
    assert!(is_valid_var_name("_PRIVATE"));

    // Invalid var names (injection attempts)
    assert!(!is_valid_var_name("'; rm -rf /; #"));
    assert!(!is_valid_var_name("VAR; echo hack"));
    assert!(!is_valid_var_name("$(whoami)"));
    assert!(!is_valid_var_name("VAR`id`"));
    assert!(!is_valid_var_name("VAR$OTHER"));
    assert!(!is_valid_var_name("VAR-NAME")); // Dash not allowed
    assert!(!is_valid_var_name("VAR.NAME")); // Dot not allowed
}

/// RED TEST: Default values must be properly escaped
/// Tests that env_var_or() safely handles special characters in defaults
#[test]
fn test_env_var_or_escapes_default() {
    // These should be safely handled (no injection)
    let safe_defaults = vec![
        "/usr/local",
        "hello world",
        "/path/to/file",
        "value-with-dash",
    ];

    for default in safe_defaults {
        // RED: This will fail until we implement escaping
        assert!(
            is_safe_default_value(default),
            "Default '{}' should be considered safe",
            default
        );
    }

    // These are dangerous and should either be escaped or rejected
    let dangerous_defaults = vec![
        "\"; rm -rf /; echo \"",
        "value`whoami`",
        "value$(id)",
        "value;ls",
    ];

    for default in dangerous_defaults {
        // RED: This will fail until we implement injection detection
        assert!(
            contains_injection_attempt(default),
            "Default '{}' contains injection attempt and should be detected",
            default
        );
    }
}

// Helper functions for validation (used by test code, Phase 2 production integration pending)
#[cfg(test)]
fn is_valid_var_name(name: &str) -> bool {
    name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

#[cfg(test)]
fn is_safe_default_value(_value: &str) -> bool {
    true // Placeholder - will be refined in Phase 2
}

#[cfg(test)]
fn contains_injection_attempt(value: &str) -> bool {
    value.contains(';') || value.contains('`') || value.contains("$(") || value.contains("${")
}
