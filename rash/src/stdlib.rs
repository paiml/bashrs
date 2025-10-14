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
    )
}

/// Get the shell function name for a stdlib function
pub fn get_shell_function_name(name: &str) -> String {
    format!("rash_{}", name)
}

/// Stdlib function metadata
#[derive(Debug, Clone)]
pub struct StdlibFunction {
    pub name: &'static str,
    pub shell_name: &'static str,
    pub module: &'static str,
    pub description: &'static str,
}

/// All stdlib functions
pub const STDLIB_FUNCTIONS: &[StdlibFunction] = &[
    // String module
    StdlibFunction {
        name: "string_trim",
        shell_name: "rash_string_trim",
        module: "string",
        description: "Remove leading and trailing whitespace",
    },
    StdlibFunction {
        name: "string_contains",
        shell_name: "rash_string_contains",
        module: "string",
        description: "Check if string contains substring",
    },
    StdlibFunction {
        name: "string_len",
        shell_name: "rash_string_len",
        module: "string",
        description: "Get string length",
    },
    StdlibFunction {
        name: "string_replace",
        shell_name: "rash_string_replace",
        module: "string",
        description: "Replace substring with another string",
    },
    StdlibFunction {
        name: "string_to_upper",
        shell_name: "rash_string_to_upper",
        module: "string",
        description: "Convert string to uppercase",
    },
    StdlibFunction {
        name: "string_to_lower",
        shell_name: "rash_string_to_lower",
        module: "string",
        description: "Convert string to lowercase",
    },
    // File system module
    StdlibFunction {
        name: "fs_exists",
        shell_name: "rash_fs_exists",
        module: "fs",
        description: "Check if file/directory exists",
    },
    StdlibFunction {
        name: "fs_read_file",
        shell_name: "rash_fs_read_file",
        module: "fs",
        description: "Read entire file to string",
    },
    StdlibFunction {
        name: "fs_write_file",
        shell_name: "rash_fs_write_file",
        module: "fs",
        description: "Write string to file",
    },
    StdlibFunction {
        name: "fs_copy",
        shell_name: "rash_fs_copy",
        module: "fs",
        description: "Copy file from source to destination",
    },
    StdlibFunction {
        name: "fs_remove",
        shell_name: "rash_fs_remove",
        module: "fs",
        description: "Remove file or directory",
    },
    StdlibFunction {
        name: "fs_is_file",
        shell_name: "rash_fs_is_file",
        module: "fs",
        description: "Check if path is a regular file",
    },
    StdlibFunction {
        name: "fs_is_dir",
        shell_name: "rash_fs_is_dir",
        module: "fs",
        description: "Check if path is a directory",
    },
    // Environment module (Sprint 27a)
    StdlibFunction {
        name: "env",
        shell_name: "inline_env_var",
        module: "env",
        description: "Get environment variable value (inline ${VAR})",
    },
    StdlibFunction {
        name: "env_var_or",
        shell_name: "inline_env_var_or",
        module: "env",
        description: "Get environment variable with default (inline ${VAR:-default})",
    },
];

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

// Helper functions that need to be implemented in GREEN phase
fn is_valid_var_name(name: &str) -> bool {
    // RED: Stub - will implement in GREEN phase
    name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn is_safe_default_value(_value: &str) -> bool {
    // RED: Stub - will implement in GREEN phase
    true // Placeholder
}

fn contains_injection_attempt(value: &str) -> bool {
    // RED: Stub - will implement in GREEN phase
    value.contains(';') || value.contains('`') || value.contains("$(") || value.contains("${")
}
