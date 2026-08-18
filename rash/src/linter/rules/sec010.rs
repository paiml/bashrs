//! SEC010: Path Traversal Vulnerabilities
//!
//! **Rule**: Detect path traversal risks in file operations
//!
//! **Why this matters**:
//! Path traversal vulnerabilities allow attackers to access files outside intended directories
//! by using sequences like `../` or absolute paths. This can lead to unauthorized file access,
//! data theft, or system compromise.
//!
//! **Auto-fix**: Manual review required (context-dependent validation needed)
//!
//! ## Examples
//!
//! ❌ **CRITICAL VULNERABILITY**:
//! ```bash
//! # Dangerous - user could provide "../../../../etc/passwd"
//! cp "$USER_FILE" /destination/
//! cat "$INPUT_PATH"
//! tar -xf "$ARCHIVE"  # Could extract outside intended directory
//!
//! # Dangerous - no validation of path
//! mkdir -p "$USER_DIR"
//! cd "$USER_PATH"
//! ```
//!
//! ✅ **SAFE ALTERNATIVES**:
//! ```bash
//! # Validate path doesn't contain ../
//! if [[ "$USER_FILE" == *".."* ]] || [[ "$USER_FILE" == /* ]]; then
//!     echo "Invalid path" >&2
//!     exit 1
//! fi
//! cp "$USER_FILE" /destination/
//!
//! # Use realpath to resolve and validate
//! REAL_PATH=$(realpath -m "$USER_FILE")
//! if [[ "$REAL_PATH" != /safe/base/path/* ]]; then
//!     echo "Path outside allowed directory" >&2
//!     exit 1
//! fi
//! ```

use crate::linter::taint::{self, TaintKind, TaintMap};
use crate::linter::{Diagnostic, LintResult, Severity, Span};

/// File operation commands that are path traversal vectors
const FILE_OPS: &[&str] = &["cp", "mv", "cat", "tar", "unzip", "rm", "mkdir", "cd", "ln"];

/// Patterns that indicate potential path traversal
const TRAVERSAL_PATTERNS: &[&str] = &[
    "..",  // Parent directory reference
    "../", // Parent directory path
    "/..", // Absolute parent reference
];

/// Check file operations for path traversal risks.
///
/// GH-227: a file operation is a traversal risk only when the path expression
/// can be influenced from outside the script. The old implementation asked
/// `contains_unvalidated_variable` alone, which matched `mkdir -p "$OUT_DIR"`
/// on the substring `DIR` even for a literal path.
fn check_file_ops(line: &str, line_num: usize, taint: &TaintMap, result: &mut LintResult) {
    let kind = taint.line_taint(line_num, line);
    if kind == TaintKind::Clean {
        return;
    }
    for file_op in FILE_OPS {
        let Some(cmd_col) = find_command(line, file_op) else {
            continue;
        };
        if !contains_unvalidated_variable(line, file_op) {
            continue;
        }
        let span = Span::new(line_num + 1, cmd_col + 1, line_num + 1, line.len());
        let diag = Diagnostic::new(
            "SEC010",
            severity_for(kind),
            format!("Path traversal risk in {} - validate paths don't contain '..' or start with '/'", file_op),
            span,
        );
        result.add(diag);
        break;
    }
}

/// GH-227: grade the severity by provenance.
///
/// Proven external input reaching an unguarded path is a vulnerability and
/// keeps exit code 2. A variable this file never assigns is a *guess* about
/// the environment, and a guess must not break a build.
fn severity_for(kind: TaintKind) -> Severity {
    match kind {
        TaintKind::External => Severity::Error,
        _ => Severity::Warning,
    }
}

/// Check for explicit traversal patterns (e.g. ".." in literal paths)
fn check_traversal_patterns(line: &str, line_num: usize, result: &mut LintResult) {
    for pattern in TRAVERSAL_PATTERNS {
        if line.contains(pattern) && contains_file_operation(line) {
            if line.contains("BASH_SOURCE") || line.contains("dirname") {
                continue;
            }
            if !is_validation_context(line) {
                if let Some(pos) = line.find(pattern) {
                    let span = Span::new(line_num + 1, pos + 1, line_num + 1, line.len());
                    let diag = Diagnostic::new(
                        "SEC010",
                        Severity::Warning,
                        "Path contains traversal sequence '..' - ensure this is intentional and validated",
                        span,
                    );
                    result.add(diag);
                    break;
                }
            }
        }
    }
}

/// Check for path traversal vulnerabilities
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    let taint = taint::analyze(source);
    // GH-227: the body of a quoted heredoc is data, not shell. The taint pass
    // deliberately ignores it, so the rule must ignore it too — otherwise a
    // variable "assigned" inside the body reads as unknown-provenance and the
    // rule fires on a line the script never executes.
    let heredoc_body = crate::linter::heredoc::quoted_heredoc_lines(source);

    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with('#')
            || is_heredoc_pattern(line)
            || heredoc_body.contains(&(line_num + 1))
        {
            continue;
        }

        check_file_ops(line, line_num, &taint, &mut result);
        check_traversal_patterns(line, line_num, &mut result);
    }

    result
}

/// Find a command in a line (word boundary detection)
fn find_command(line: &str, cmd: &str) -> Option<usize> {
    if let Some(pos) = line.find(cmd) {
        let before_ok = if pos == 0 {
            true
        } else {
            let char_before = line.chars().nth(pos - 1);
            matches!(char_before, Some(' ' | '\t' | ';' | '&' | '|' | '(' | '\n'))
        };

        let after_idx = pos + cmd.len();
        let after_ok = if after_idx >= line.len() {
            true
        } else {
            let char_after = line.chars().nth(after_idx);
            matches!(char_after, Some(' ' | '\t' | ';' | '&' | '|' | ')'))
        };

        if before_ok && after_ok {
            return Some(pos);
        }
    }
    None
}

/// Issue #73: Known-safe patterns that should not trigger SEC010
// These are checked as exact variable names (with $ or ${} wrapper)
const SAFE_VAR_PATTERNS: &[&str] = &[
    "$PWD",        // Current directory is intentional
    "${PWD}",      // Current directory is intentional
    "$HOME",       // User's home directory is safe
    "${HOME}",     // User's home directory is safe
    "$TMPDIR",     // Temp directory is safe
    "${TMPDIR}",   // Temp directory is safe
    "BASH_SOURCE", // Script's own directory is safe
    "dirname",     // dirname of script is safe
    "XDG_",        // XDG directories are safe
];

/// Substrings that suggest untrusted or user-provided input.
///
/// GH-227 note: this is a NAME heuristic over the whole line, not a dataflow
/// fact. It is now only one of two necessary conditions — `check_file_ops`
/// also requires `crate::linter::taint` to say the line can be influenced from
/// outside the script.
const USER_INPUT_PATTERNS: &[&str] = &[
    "USER",      // USER_FILE, USER_PATH, etc.
    "INPUT",     // INPUT_PATH, INPUT_FILE, etc.
    "UPLOAD",    // Uploaded files
    "ARCHIVE",   // Archive files (could be user-provided)
    "UNTRUSTED", // Explicitly untrusted
    "EXTERNAL",  // External input
    "REMOTE",    // Remote data
    "ARG",       // Command line arguments
    "NAME",      // Could be user-provided name
    "FILE",      // Generic file variables
    "PATH",      // Generic path variables (but not PATH env var)
    "DIR",       // Generic directory variables
];

/// Issue #73: the line uses only patterns known to be safe.
fn is_known_safe_line(line: &str) -> bool {
    if SAFE_VAR_PATTERNS.iter().any(|p| line.contains(p)) {
        return true;
    }
    // Script directory parent (..) with BASH_SOURCE is intentional:
    // `cd "$(dirname "${BASH_SOURCE[0]}")/.."`
    line.contains("dirname") && line.contains("..")
}

/// Every `PATH` mention on the line is the `PATH` environment variable.
fn only_path_env_var(line: &str) -> bool {
    if !line.contains("$PATH") && !line.contains("${PATH}") {
        return false;
    }
    let path_count = line.matches("PATH").count();
    let dollar_path_count = line.matches("$PATH").count() + line.matches("${PATH}").count();
    path_count == dollar_path_count
}

/// Check if line contains unvalidated variable in file operation
fn contains_unvalidated_variable(line: &str, _cmd: &str) -> bool {
    // Look for variable usage: $VAR, ${VAR}, "$VAR"
    if !line.contains('$') {
        return false;
    }

    if is_known_safe_line(line) || only_path_env_var(line) {
        return false;
    }

    // If no suspicious pattern is found, assume it's safe. This reduces false
    // positives for common scripts.
    let line_upper = line.to_uppercase();
    USER_INPUT_PATTERNS
        .iter()
        .any(|pattern| line_upper.contains(pattern))
}

/// Check if line contains any file operation
fn contains_file_operation(line: &str) -> bool {
    FILE_OPS.iter().any(|op| find_command(line, op).is_some())
}

/// Check if this is a validation context (checking for ..)
fn is_validation_context(line: &str) -> bool {
    // Common validation patterns
    let validation_keywords = ["if", "case", "grep", "=~", "==", "!="];

    validation_keywords.iter().any(|kw| line.contains(kw))
}

/// Issue #106: Check if this is a heredoc pattern
/// Heredocs like `cat <<EOF` or `cat <<'EOF'` are not file reads
fn is_heredoc_pattern(line: &str) -> bool {
    // Check for heredoc operators: << or <<<
    if line.contains("<<") {
        // Common heredoc patterns with file commands
        // cat <<EOF, cat <<'EOF', cat <<"EOF", cat <<-EOF
        // Also handles here-string: cat <<<
        let heredoc_patterns = [
            "cat <<", "cat<<<", "cat <<-", "echo <<", "read <<", "tee <<",
        ];

        for pattern in &heredoc_patterns {
            if line.contains(pattern) {
                return true;
            }
        }

        // Also check for $(...) containing heredoc
        // e.g., content=$(cat <<EOF ... EOF)
        if line.contains("$(cat <<") || line.contains("$(cat<<") {
            return true;
        }
    }

    false
}

#[cfg(test)]
#[path = "sec010_tests_sec010_detec.rs"]
mod tests_ext;
