//! Stdlib function metadata registry.
//!
//! Contains the `StdlibFunction` struct and `STDLIB_FUNCTIONS` const array.
//! Extracted from `stdlib.rs` to keep file sizes under 500 lines.

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
    StdlibFunction {
        name: "string_starts_with",
        shell_name: "rash_string_starts_with",
        module: "string",
        description: "Check if string starts with prefix",
    },
    StdlibFunction {
        name: "string_ends_with",
        shell_name: "rash_string_ends_with",
        module: "string",
        description: "Check if string ends with suffix",
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
    // Arguments module (Sprint 27b)
    StdlibFunction {
        name: "arg",
        shell_name: "inline_positional_arg",
        module: "args",
        description: "Get command-line argument by position (inline $n)",
    },
    StdlibFunction {
        name: "args",
        shell_name: "inline_all_args",
        module: "args",
        description: "Get all command-line arguments (inline $@)",
    },
    StdlibFunction {
        name: "arg_count",
        shell_name: "inline_arg_count",
        module: "args",
        description: "Get command-line argument count (inline $#)",
    },
    // Exit code module (Sprint 27c)
    StdlibFunction {
        name: "exit_code",
        shell_name: "inline_exit_code",
        module: "status",
        description: "Get exit code of last command (inline $?)",
    },
    // Sprint 28: Complete Missing Stdlib Functions
    StdlibFunction {
        name: "string_split",
        shell_name: "rash_string_split",
        module: "string",
        description: "Split string by delimiter into newline-separated output",
    },
    StdlibFunction {
        name: "array_len",
        shell_name: "rash_array_len",
        module: "array",
        description: "Count elements in newline-separated array",
    },
    StdlibFunction {
        name: "array_join",
        shell_name: "rash_array_join",
        module: "array",
        description: "Join newline-separated array elements with separator",
    },
    // Command execution module (GH-148)
    StdlibFunction {
        name: "capture",
        shell_name: "inline_command_subst",
        module: "command",
        description: "Command substitution: capture(\"cmd\") → $(cmd)",
    },
    StdlibFunction {
        name: "exec",
        shell_name: "inline_exec",
        module: "command",
        description: "Execute shell command: exec(\"cmd\") → cmd",
    },
    StdlibFunction {
        name: "exit",
        shell_name: "inline_exit",
        module: "command",
        description: "Exit process: exit(N) → exit N",
    },
    StdlibFunction {
        name: "sleep",
        shell_name: "inline_sleep",
        module: "command",
        description: "Pause execution: sleep(N) → sleep N",
    },
    // File iteration module (GH-148)
    StdlibFunction {
        name: "glob",
        shell_name: "inline_glob",
        module: "fs",
        description: "File glob pattern: glob(\"*.txt\") → *.txt (unquoted for shell expansion)",
    },
    // Directory/file management module (GH-148)
    StdlibFunction {
        name: "mkdir",
        shell_name: "inline_mkdir",
        module: "fs",
        description: "Create directory: mkdir(path) → mkdir -p path (idempotent)",
    },
    StdlibFunction {
        name: "mv",
        shell_name: "inline_mv",
        module: "fs",
        description: "Move/rename file: mv(src, dst) → mv src dst",
    },
    StdlibFunction {
        name: "chmod",
        shell_name: "inline_chmod",
        module: "fs",
        description: "Change permissions: chmod(mode, path) → chmod mode path",
    },
];
