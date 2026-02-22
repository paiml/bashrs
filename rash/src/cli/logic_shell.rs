// CLI Logic - Shell Detection and Processing
//
// Shell script detection, normalization, file type detection,
// and platform-related utility functions.

use std::path::Path;

// =============================================================================
// SHELL SCRIPT DETECTION
// =============================================================================

/// Detect if a file is a shell script based on extension and shebang (Issue #84)
///
/// Returns true if the file:
/// - Has a shell extension (.sh, .bash, .ksh, .zsh)
/// - Has a shell shebang (#!/bin/sh, #!/bin/bash, etc.)
pub fn is_shell_script_file(path: &Path, content: &str) -> bool {
    has_shell_extension(path) || has_shell_shebang(content)
}

/// Check if a file has a shell script extension
fn has_shell_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| {
            matches!(
                e.to_lowercase().as_str(),
                "sh" | "bash" | "ksh" | "zsh" | "ash"
            )
        })
        .unwrap_or(false)
}

/// Check if content starts with a shell shebang line
fn has_shell_shebang(content: &str) -> bool {
    const SHELL_PATTERNS: &[&str] = &[
        "/sh", "/bash", "/zsh", "/ksh", "/ash", "/dash", "env sh", "env bash",
    ];

    content
        .lines()
        .next()
        .filter(|line| line.starts_with("#!"))
        .map(|line| {
            let lower = line.to_lowercase();
            SHELL_PATTERNS.iter().any(|p| lower.contains(p))
        })
        .unwrap_or(false)
}

/// Normalize a shell script for comparison
/// Removes comments and normalizes whitespace
pub fn normalize_shell_script(script: &str) -> String {
    script
        .lines()
        .filter(|line| !line.trim().starts_with('#'))
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

// =============================================================================
// FILE TYPE DETECTION
// =============================================================================

/// Detect if a file is a Makefile
pub fn is_makefile(filename: &str) -> bool {
    filename == "Makefile"
        || filename == "makefile"
        || filename == "GNUmakefile"
        || filename.ends_with(".mk")
        || filename.ends_with(".make")
}

/// Detect if a file is a Dockerfile
pub fn is_dockerfile(filename: &str) -> bool {
    let filename_lower = filename.to_lowercase();
    filename_lower == "dockerfile"
        || filename_lower.starts_with("dockerfile.")
        || filename_lower.ends_with(".dockerfile")
}

// =============================================================================
// PLATFORM AND PATH UTILITIES
// =============================================================================

/// Detect the current platform (pure function)
pub fn detect_platform() -> &'static str {
    #[cfg(target_os = "linux")]
    {
        "linux"
    }
    #[cfg(target_os = "macos")]
    {
        "macos"
    }
    #[cfg(target_os = "windows")]
    {
        "windows"
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        "unknown"
    }
}

/// Check if output path indicates stdout (pure function)
pub fn should_output_to_stdout(output_path: &Path) -> bool {
    output_path == Path::new("-") || output_path == Path::new("/dev/null")
}

/// Check if path looks like stdin/stdout marker
pub fn is_stdio_path(path: &Path) -> bool {
    path == Path::new("-") || path == Path::new("/dev/stdin") || path == Path::new("/dev/stdout")
}

/// Parse shell dialect from string
pub fn parse_shell_dialect(s: &str) -> Option<&'static str> {
    match s.to_lowercase().as_str() {
        "posix" | "sh" => Some("posix"),
        "bash" => Some("bash"),
        "zsh" => Some("zsh"),
        "dash" => Some("dash"),
        _ => None,
    }
}

/// Count duplicate entries in a list of paths (pure function)
pub fn count_duplicate_path_entries(entries: &[String]) -> usize {
    use std::collections::HashSet;
    let unique: HashSet<&String> = entries.iter().collect();
    entries.len().saturating_sub(unique.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== SHELL SCRIPT DETECTION TESTS =====

    #[test]
    fn test_is_shell_script_by_extension() {
        assert!(is_shell_script_file(Path::new("script.sh"), "echo hello"));
        assert!(is_shell_script_file(Path::new("script.bash"), "echo hello"));
        assert!(is_shell_script_file(Path::new("script.zsh"), "echo hello"));
        assert!(is_shell_script_file(Path::new("script.ksh"), "echo hello"));
        assert!(is_shell_script_file(Path::new("script.ash"), "echo hello"));
        assert!(!is_shell_script_file(
            Path::new("script.rs"),
            "fn main() {}"
        ));
        assert!(!is_shell_script_file(
            Path::new("script.py"),
            "print('hello')"
        ));
    }

    #[test]
    fn test_is_shell_script_by_shebang() {
        assert!(is_shell_script_file(
            Path::new("script"),
            "#!/bin/sh\necho hello"
        ));
        assert!(is_shell_script_file(
            Path::new("script"),
            "#!/bin/bash\necho hello"
        ));
        assert!(is_shell_script_file(
            Path::new("script"),
            "#!/usr/bin/env bash\necho hello"
        ));
        assert!(is_shell_script_file(
            Path::new("script"),
            "#!/usr/bin/env sh\necho hello"
        ));
        assert!(!is_shell_script_file(
            Path::new("script"),
            "#!/usr/bin/python\nprint('hello')"
        ));
    }

    #[test]
    fn test_is_shell_script_case_insensitive() {
        assert!(is_shell_script_file(Path::new("script.SH"), "echo hello"));
        assert!(is_shell_script_file(Path::new("script.BASH"), "echo hello"));
    }

    // ===== NORMALIZE SHELL SCRIPT TESTS =====

    #[test]
    fn test_normalize_shell_script_removes_comments() {
        let script = "# comment\necho hello\n# another comment\necho world";
        let normalized = normalize_shell_script(script);
        assert_eq!(normalized, "echo hello\necho world");
    }

    #[test]
    fn test_normalize_shell_script_trims_whitespace() {
        let script = "  echo hello  \n  echo world  ";
        let normalized = normalize_shell_script(script);
        assert_eq!(normalized, "echo hello\necho world");
    }

    #[test]
    fn test_normalize_shell_script_removes_empty_lines() {
        let script = "echo hello\n\n\necho world";
        let normalized = normalize_shell_script(script);
        assert_eq!(normalized, "echo hello\necho world");
    }

    // ===== FILE TYPE DETECTION TESTS =====

    #[test]
    fn test_is_makefile() {
        assert!(is_makefile("Makefile"));
        assert!(is_makefile("makefile"));
        assert!(is_makefile("GNUmakefile"));
        assert!(is_makefile("rules.mk"));
        assert!(is_makefile("build.make"));
        assert!(!is_makefile("script.sh"));
        assert!(!is_makefile("Makefile.md"));
    }

    #[test]
    fn test_is_dockerfile() {
        assert!(is_dockerfile("Dockerfile"));
        assert!(is_dockerfile("dockerfile"));
        assert!(is_dockerfile("DOCKERFILE"));
        assert!(is_dockerfile("Dockerfile.dev"));
        assert!(is_dockerfile("app.dockerfile"));
        assert!(!is_dockerfile("Makefile"));
        assert!(!is_dockerfile("script.sh"));
    }

    // ===== DETECT PLATFORM TESTS =====

    #[test]
    fn test_detect_platform_returns_valid() {
        let platform = detect_platform();
        let valid_platforms = ["linux", "macos", "windows", "unknown"];
        assert!(valid_platforms.contains(&platform));
    }

    // ===== SHOULD OUTPUT TO STDOUT TESTS =====

    #[test]
    fn test_should_output_to_stdout_dash() {
        assert!(should_output_to_stdout(Path::new("-")));
    }

    #[test]
    fn test_should_output_to_stdout_devnull() {
        assert!(should_output_to_stdout(Path::new("/dev/null")));
    }

    #[test]
    fn test_should_output_to_stdout_regular_file() {
        assert!(!should_output_to_stdout(Path::new("output.txt")));
        assert!(!should_output_to_stdout(Path::new("/tmp/file.sh")));
    }

    // ===== STDIO PATH TESTS =====

    #[test]
    fn test_is_stdio_path_stdin_stdout() {
        assert!(is_stdio_path(Path::new("-")));
        assert!(is_stdio_path(Path::new("/dev/stdin")));
        assert!(is_stdio_path(Path::new("/dev/stdout")));
    }

    #[test]
    fn test_is_stdio_path_regular_files() {
        assert!(!is_stdio_path(Path::new("output.txt")));
        assert!(!is_stdio_path(Path::new("/tmp/file.sh")));
        assert!(!is_stdio_path(Path::new("/dev/null")));
    }

    // ===== SHELL DIALECT TESTS =====

    #[test]
    fn test_parse_shell_dialect_posix() {
        assert_eq!(parse_shell_dialect("posix"), Some("posix"));
        assert_eq!(parse_shell_dialect("sh"), Some("posix"));
        assert_eq!(parse_shell_dialect("POSIX"), Some("posix"));
        assert_eq!(parse_shell_dialect("SH"), Some("posix"));
    }

    #[test]
    fn test_parse_shell_dialect_bash() {
        assert_eq!(parse_shell_dialect("bash"), Some("bash"));
        assert_eq!(parse_shell_dialect("BASH"), Some("bash"));
    }

    #[test]
    fn test_parse_shell_dialect_zsh() {
        assert_eq!(parse_shell_dialect("zsh"), Some("zsh"));
        assert_eq!(parse_shell_dialect("ZSH"), Some("zsh"));
    }

    #[test]
    fn test_parse_shell_dialect_dash() {
        assert_eq!(parse_shell_dialect("dash"), Some("dash"));
    }

    #[test]
    fn test_parse_shell_dialect_unknown() {
        assert_eq!(parse_shell_dialect("fish"), None);
        assert_eq!(parse_shell_dialect("invalid"), None);
    }

    // ===== COUNT DUPLICATE PATH ENTRIES TESTS =====

    #[test]
    fn test_count_duplicate_path_entries_none() {
        let entries = vec![
            "/usr/bin".to_string(),
            "/usr/local/bin".to_string(),
            "/home/user/bin".to_string(),
        ];
        assert_eq!(count_duplicate_path_entries(&entries), 0);
    }

    #[test]
    fn test_count_duplicate_path_entries_some() {
        let entries = vec![
            "/usr/bin".to_string(),
            "/usr/local/bin".to_string(),
            "/usr/bin".to_string(), // duplicate
        ];
        assert_eq!(count_duplicate_path_entries(&entries), 1);
    }

    #[test]
    fn test_count_duplicate_path_entries_multiple() {
        let entries = vec![
            "/usr/bin".to_string(),
            "/usr/bin".to_string(),
            "/usr/bin".to_string(),
            "/home/user/bin".to_string(),
            "/home/user/bin".to_string(),
        ];
        assert_eq!(count_duplicate_path_entries(&entries), 3); // 2 extra /usr/bin + 1 extra /home/user/bin
    }

    #[test]
    fn test_count_duplicate_path_entries_empty() {
        let entries: Vec<String> = vec![];
        assert_eq!(count_duplicate_path_entries(&entries), 0);
    }
}
