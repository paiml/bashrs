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
            // File system module
            | "fs_exists"
            | "fs_read_file"
            | "fs_write_file"
            // Array module
            | "array_len"
            | "array_join"
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
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_stdlib_function() {
        assert!(is_stdlib_function("string_trim"));
        assert!(is_stdlib_function("fs_exists"));
        assert!(!is_stdlib_function("custom_function"));
        assert!(!is_stdlib_function("println"));
    }

    #[test]
    fn test_get_shell_function_name() {
        assert_eq!(get_shell_function_name("string_trim"), "rash_string_trim");
        assert_eq!(get_shell_function_name("fs_exists"), "rash_fs_exists");
    }
}
