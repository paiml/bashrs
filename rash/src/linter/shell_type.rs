//! Shell type detection for correct linting.
//!
//! Automatically detects which shell (bash, zsh, sh, ksh) a script is written for,
//! allowing bashrs to apply appropriate linting rules based on shell compatibility.
//!
//! # Examples
//!
//! ## Detecting from shebang
//!
//! ```
//! use bashrs::linter::shell_type::{detect_shell_type, ShellType};
//! use std::path::Path;
//!
//! let content = "#!/bin/zsh\necho hello";
//! let path = Path::new("script.sh");
//! let shell = detect_shell_type(path, content);
//! assert_eq!(shell, ShellType::Zsh);
//! ```
//!
//! ## Detecting from file extension
//!
//! ```
//! use bashrs::linter::shell_type::{detect_shell_type, ShellType};
//! use std::path::Path;
//!
//! let content = "echo hello";
//! let path = Path::new(".zshrc");
//! let shell = detect_shell_type(path, content);
//! assert_eq!(shell, ShellType::Zsh);
//! ```
//!
//! ## Detection priority
//!
//! ```
//! use bashrs::linter::shell_type::{detect_shell_type, ShellType};
//! use std::path::Path;
//!
//! // ShellCheck directive overrides everything
//! let content = "#!/bin/bash\n# shellcheck shell=zsh\necho hello";
//! let path = Path::new("script.sh");
//! assert_eq!(detect_shell_type(path, content), ShellType::Zsh);
//! ```

use std::path::Path;

/// Supported shell types for linting.
///
/// Each shell type has different syntax features and compatibility requirements.
/// bashrs uses this to apply appropriate linting rules.
///
/// # Examples
///
/// ## Converting to ShellCheck names
///
/// ```
/// use bashrs::linter::ShellType;
///
/// assert_eq!(ShellType::Bash.to_shellcheck_name(), "bash");
/// assert_eq!(ShellType::Zsh.to_shellcheck_name(), "zsh");
/// assert_eq!(ShellType::Sh.to_shellcheck_name(), "sh");
/// ```
///
/// ## Using with rule compatibility
///
/// ```
/// use bashrs::linter::{rule_registry, ShellType};
///
/// // Check if a rule applies to a specific shell
/// let applies_to_zsh = rule_registry::should_apply_rule("SEC001", ShellType::Zsh);
/// assert!(applies_to_zsh); // Security rules apply to all shells
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellType {
    /// Bash shell (default if detection fails).
    ///
    /// Bash-specific features include arrays, `[[` test operator, process substitution.
    Bash,

    /// Zsh shell.
    ///
    /// Zsh has unique features like enhanced globbing, associative arrays.
    Zsh,

    /// POSIX sh (most restrictive).
    ///
    /// POSIX sh supports only standard shell features, no bash/zsh extensions.
    Sh,

    /// Korn shell.
    ///
    /// Ksh supports some advanced features but differs from bash/zsh.
    Ksh,

    /// Auto-detect (let ShellCheck decide).
    ///
    /// Defers shell type detection to external tools.
    Auto,
}

impl ShellType {
    /// Converts to ShellCheck-compatible shell name.
    ///
    /// Returns the lowercase shell name used by ShellCheck and other tools.
    ///
    /// # Returns
    ///
    /// * `"bash"` - For `ShellType::Bash`
    /// * `"zsh"` - For `ShellType::Zsh`
    /// * `"sh"` - For `ShellType::Sh`
    /// * `"ksh"` - For `ShellType::Ksh`
    /// * `"auto"` - For `ShellType::Auto`
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::linter::ShellType;
    ///
    /// assert_eq!(ShellType::Bash.to_shellcheck_name(), "bash");
    /// assert_eq!(ShellType::Zsh.to_shellcheck_name(), "zsh");
    /// assert_eq!(ShellType::Sh.to_shellcheck_name(), "sh");
    /// assert_eq!(ShellType::Ksh.to_shellcheck_name(), "ksh");
    /// assert_eq!(ShellType::Auto.to_shellcheck_name(), "auto");
    /// ```
    pub fn to_shellcheck_name(&self) -> &'static str {
        match self {
            ShellType::Bash => "bash",
            ShellType::Zsh => "zsh",
            ShellType::Sh => "sh",
            ShellType::Ksh => "ksh",
            ShellType::Auto => "auto",
        }
    }
}

/// Detects shell type from file path and content.
///
/// Uses a priority-based detection system to determine which shell a script is written for.
///
/// # Priority Order
///
/// 1. **ShellCheck directive** - `# shellcheck shell=zsh` (highest priority)
/// 2. **Shebang line** - `#!/usr/bin/env zsh`
/// 3. **File extension** - `.zsh`, `.bash`, `.ksh`
/// 4. **File name** - `.zshrc`, `.bashrc`, `.bash_profile`
/// 5. **Default** - `Bash` if no other indicator found
///
/// # Arguments
///
/// * `path` - File path for extension/name detection
/// * `content` - File content for shebang/directive detection
///
/// # Returns
///
/// Detected shell type, defaulting to `Bash` if detection fails.
///
/// # Examples
///
/// ## Detection from shebang
///
/// ```
/// use bashrs::linter::shell_type::{detect_shell_type, ShellType};
/// use std::path::Path;
///
/// let content = "#!/usr/bin/env zsh\necho hello";
/// let path = Path::new("script.sh");
/// assert_eq!(detect_shell_type(path, content), ShellType::Zsh);
/// ```
///
/// ## Detection from file extension
///
/// ```
/// use bashrs::linter::shell_type::{detect_shell_type, ShellType};
/// use std::path::Path;
///
/// let content = "echo hello";
/// let path = Path::new("script.zsh");
/// assert_eq!(detect_shell_type(path, content), ShellType::Zsh);
/// ```
///
/// ## Detection from dotfile name
///
/// ```
/// use bashrs::linter::shell_type::{detect_shell_type, ShellType};
/// use std::path::Path;
///
/// let content = "echo hello";
/// let path = Path::new(".bashrc");
/// assert_eq!(detect_shell_type(path, content), ShellType::Bash);
/// ```
///
/// ## ShellCheck directive overrides shebang
///
/// ```
/// use bashrs::linter::shell_type::{detect_shell_type, ShellType};
/// use std::path::Path;
///
/// // Directive has highest priority
/// let content = "#!/bin/bash\n# shellcheck shell=zsh\necho hello";
/// let path = Path::new("script.sh");
/// assert_eq!(detect_shell_type(path, content), ShellType::Zsh);
/// ```
///
/// ## Default to Bash
///
/// ```
/// use bashrs::linter::shell_type::{detect_shell_type, ShellType};
/// use std::path::Path;
///
/// // No indicators → defaults to Bash
/// let content = "echo hello";
/// let path = Path::new("script.sh");
/// assert_eq!(detect_shell_type(path, content), ShellType::Bash);
/// ```
pub fn detect_shell_type(path: &Path, content: &str) -> ShellType {
    // Priority 1: ShellCheck directive (overrides everything)
    if let Some(shell) = detect_from_shellcheck_directive(content) {
        return shell;
    }

    // Priority 2: Shebang
    if let Some(shell) = detect_from_shebang(content) {
        return shell;
    }

    // Priority 3: File extension/name
    if let Some(shell) = detect_from_path(path) {
        return shell;
    }

    // Priority 4: Default
    ShellType::Bash
}

/// Detect shell from shebang line
fn detect_from_shebang(content: &str) -> Option<ShellType> {
    let first_line = content.lines().next()?;

    // Must start with #!
    if !first_line.trim_start().starts_with("#!") {
        return None;
    }

    // Extract shell name from shebang
    // Examples: #!/bin/bash, #!/usr/bin/env zsh, #! /bin/sh
    let shebang = first_line.trim_start().trim_start_matches("#!").trim();

    if shebang.contains("zsh") {
        Some(ShellType::Zsh)
    } else if shebang.contains("bash") {
        Some(ShellType::Bash)
    } else if shebang.ends_with("/sh") || shebang.ends_with(" sh") || shebang == "sh" {
        Some(ShellType::Sh)
    } else if shebang.contains("ksh") {
        Some(ShellType::Ksh)
    } else {
        None
    }
}

/// Detect shell from ShellCheck directive
fn detect_from_shellcheck_directive(content: &str) -> Option<ShellType> {
    // Look for # shellcheck shell=<type> in first few lines
    for line in content.lines().take(10) {
        let line = line.trim();
        if line.starts_with('#') && line.contains("shellcheck") && line.contains("shell=") {
            // Extract shell name after shell=
            if let Some(after_shell) = line.split("shell=").nth(1) {
                let shell_name = after_shell.split_whitespace().next()?;

                return match shell_name {
                    "zsh" => Some(ShellType::Zsh),
                    "bash" => Some(ShellType::Bash),
                    "sh" => Some(ShellType::Sh),
                    "ksh" => Some(ShellType::Ksh),
                    "auto" => Some(ShellType::Auto),
                    _ => None,
                };
            }
        }
    }

    None
}

/// Detect shell from file path (extension or name)
fn detect_from_path(path: &Path) -> Option<ShellType> {
    // Check file name first (for dotfiles like .zshrc)
    if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
        match file_name {
            // Zsh files
            ".zshrc" | ".zshenv" | ".zprofile" | ".zlogin" | ".zlogout" => {
                return Some(ShellType::Zsh);
            }
            // Bash files
            ".bashrc" | ".bash_profile" | ".bash_login" | ".bash_logout" => {
                return Some(ShellType::Bash);
            }
            _ => {}
        }
    }

    // Check file extension
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        match ext {
            "zsh" => return Some(ShellType::Zsh),
            "bash" => return Some(ShellType::Bash),
            "ksh" => return Some(ShellType::Ksh),
            _ => {}
        }
    }

    None
}

#[cfg(test)]
#[path = "shell_type_tests_extracted.rs"]
mod tests_extracted;
