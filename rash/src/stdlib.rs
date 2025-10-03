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
}
