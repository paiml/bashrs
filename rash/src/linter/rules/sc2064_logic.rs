//! SC2064 Pure Logic - Extracted for EXTREME TDD
//!
//! All testable logic for SC2064 trap quote checking.

use once_cell::sync::Lazy;
use regex::Regex;

#[allow(clippy::unwrap_used)] // Compile-time regex
static TRAP_VAR_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"\$([a-zA-Z_][a-zA-Z0-9_]*)"#).unwrap());

/// Extract variable names from a trap command
pub fn extract_trap_variables(trap_line: &str) -> Vec<&str> {
    TRAP_VAR_PATTERN
        .captures_iter(trap_line)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str()))
        .collect()
}

/// Check if line contains variable assignment
pub fn line_has_assignment(line: &str, var_name: &str) -> bool {
    let assign_pattern = format!("{}=", var_name);
    let readonly_pattern = format!("readonly {}=", var_name);
    let local_pattern = format!("local {}=", var_name);
    line.contains(&assign_pattern)
        || line.contains(&readonly_pattern)
        || line.contains(&local_pattern)
}

/// Check if trap uses intentional early expansion
/// Returns true if variables are assigned near the trap (intentional)
pub fn is_intentional_early_expansion(source: &str, trap_line_num: usize, trap_line: &str) -> bool {
    let trap_vars = extract_trap_variables(trap_line);
    if trap_vars.is_empty() {
        return false;
    }

    // Check same line assignment
    for var in &trap_vars {
        if line_has_assignment(trap_line, var) {
            return true;
        }
    }

    // Check previous 3 lines
    let lines: Vec<&str> = source.lines().collect();
    let start = trap_line_num.saturating_sub(3);

    for i in start..trap_line_num {
        if let Some(prev_line) = lines.get(i) {
            for var in &trap_vars {
                if line_has_assignment(prev_line, var) {
                    return true;
                }
            }
        }
    }

    false
}

/// Check if line is a comment
pub fn is_comment_line(line: &str) -> bool {
    line.trim_start().starts_with('#')
}

/// Check if line contains trap with double-quoted variable
pub fn has_trap_double_quoted_var(line: &str) -> bool {
    // Pattern: trap "...$var..." SIGNAL
    line.contains("trap") && line.contains('"') && line.contains('$')
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== EXTRACT TRAP VARIABLES =====

    #[test]
    fn test_extract_trap_variables_single() {
        let vars = extract_trap_variables(r#"trap "rm $tmpfile" EXIT"#);
        assert_eq!(vars, vec!["tmpfile"]);
    }

    #[test]
    fn test_extract_trap_variables_multiple() {
        let vars = extract_trap_variables(r#"trap "rm $file1 $file2" EXIT"#);
        assert_eq!(vars, vec!["file1", "file2"]);
    }

    #[test]
    fn test_extract_trap_variables_none() {
        let vars = extract_trap_variables(r#"trap "rm /tmp/file" EXIT"#);
        assert!(vars.is_empty());
    }

    #[test]
    fn test_extract_trap_variables_underscore() {
        let vars = extract_trap_variables(r#"trap "rm $my_temp_file" EXIT"#);
        assert_eq!(vars, vec!["my_temp_file"]);
    }

    #[test]
    fn test_extract_trap_variables_numbers() {
        let vars = extract_trap_variables(r#"trap "rm $file1 $var2" EXIT"#);
        assert_eq!(vars, vec!["file1", "var2"]);
    }

    // ===== LINE HAS ASSIGNMENT =====

    #[test]
    fn test_line_has_assignment_simple() {
        assert!(line_has_assignment("foo=bar", "foo"));
        assert!(!line_has_assignment("foo=bar", "bar"));
    }

    #[test]
    fn test_line_has_assignment_readonly() {
        assert!(line_has_assignment("readonly foo=bar", "foo"));
    }

    #[test]
    fn test_line_has_assignment_local() {
        assert!(line_has_assignment("local foo=bar", "foo"));
    }

    #[test]
    fn test_line_has_assignment_not_present() {
        assert!(!line_has_assignment("echo hello", "foo"));
    }

    #[test]
    fn test_line_has_assignment_partial_match() {
        // Should not match foobar when looking for foo
        assert!(!line_has_assignment("foobar=x", "foo"));
    }

    // ===== IS INTENTIONAL EARLY EXPANSION =====

    #[test]
    fn test_intentional_same_line() {
        let code = r#"v="value"; trap "echo $v" INT"#;
        assert!(is_intentional_early_expansion(code, 0, code));
    }

    #[test]
    fn test_intentional_prev_line() {
        let code = "v=\"value\"\ntrap \"echo $v\" INT";
        let lines: Vec<&str> = code.lines().collect();
        assert!(is_intentional_early_expansion(code, 1, lines[1]));
    }

    #[test]
    fn test_intentional_readonly_prev_line() {
        let code = "readonly v=\"value\"\ntrap \"echo $v\" INT";
        let lines: Vec<&str> = code.lines().collect();
        assert!(is_intentional_early_expansion(code, 1, lines[1]));
    }

    #[test]
    fn test_not_intentional_no_assignment() {
        let code = "trap \"echo $v\" INT";
        assert!(!is_intentional_early_expansion(code, 0, code));
    }

    #[test]
    fn test_not_intentional_distant_assignment() {
        let code = "v=\"value\"\necho a\necho b\necho c\necho d\ntrap \"echo $v\" INT";
        let lines: Vec<&str> = code.lines().collect();
        // Line 5 is the trap, assignment is on line 0 (too far)
        assert!(!is_intentional_early_expansion(code, 5, lines[5]));
    }

    #[test]
    fn test_intentional_no_variables() {
        let code = r#"trap "echo hello" INT"#;
        assert!(!is_intentional_early_expansion(code, 0, code));
    }

    // ===== IS COMMENT LINE =====

    #[test]
    fn test_is_comment_line_true() {
        assert!(is_comment_line("# this is a comment"));
        assert!(is_comment_line("  # indented comment"));
        assert!(is_comment_line("\t# tab comment"));
    }

    #[test]
    fn test_is_comment_line_false() {
        assert!(!is_comment_line("echo hello # inline comment"));
        assert!(!is_comment_line("trap 'rm' EXIT"));
        assert!(!is_comment_line(""));
    }

    // ===== HAS TRAP DOUBLE QUOTED VAR =====

    #[test]
    fn test_has_trap_double_quoted_var_true() {
        assert!(has_trap_double_quoted_var(r#"trap "rm $tmpfile" EXIT"#));
        assert!(has_trap_double_quoted_var(r#"trap "echo $var" INT"#));
    }

    #[test]
    fn test_has_trap_double_quoted_var_single_quotes() {
        // Single quotes don't count
        assert!(!has_trap_double_quoted_var(r#"trap 'rm $tmpfile' EXIT"#));
    }

    #[test]
    fn test_has_trap_double_quoted_var_no_var() {
        assert!(!has_trap_double_quoted_var(r#"trap "rm /tmp/file" EXIT"#));
    }

    #[test]
    fn test_has_trap_double_quoted_var_not_trap() {
        assert!(!has_trap_double_quoted_var(r#"echo "rm $tmpfile""#));
    }
}
