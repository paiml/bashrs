//! MAKE011: Dangerous pattern rules
//!
//! **Rule**: Detect overly broad pattern rules that can cause unexpected behavior
//!
//! **Why this matters**:
//! Pattern rules like `%: %.o` or `%:` match too broadly and can accidentally
//! apply to files they shouldn't. This causes confusing build failures and
//! unexpected rebuilds. More specific patterns are safer.
//!
//! **Auto-fix**: Suggest more specific pattern with file extensions
//!
//! ## Examples
//!
//! ❌ **BAD** (overly broad pattern):
//! ```makefile
//! %: %.o
//! \t$(CC) $< -o $@
//! ```
//!
//! ✅ **GOOD** (specific pattern):
//! ```makefile
//! %.out: %.o
//! \t$(CC) $< -o $@
//! ```

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};

/// Dangerous pattern rules (too broad)
const DANGEROUS_PATTERNS: &[&str] = &[
    "%:",  // Matches everything
    "% :", // Matches everything (with space)
];

/// Check for dangerous pattern rules
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        // Skip non-target lines (must contain ':')
        if !is_target_line(line) {
            continue;
        }

        // Check for dangerous patterns
        if let Some(diag) = check_line_for_dangerous_pattern(line, line_num) {
            result.add(diag);
        }
    }

    result
}

/// Check if line is a target line (contains ':' and not a recipe)
fn is_target_line(line: &str) -> bool {
    line.contains(':') && !line.starts_with('\t')
}

/// Check line for dangerous pattern and create diagnostic if found
fn check_line_for_dangerous_pattern(line: &str, line_num: usize) -> Option<Diagnostic> {
    for pattern in DANGEROUS_PATTERNS {
        if is_dangerous_pattern(line, pattern) {
            let span = Span::new(line_num + 1, 1, line_num + 1, line.len());
            let fix_replacement = create_fix(line);

            return Some(
                Diagnostic::new(
                    "MAKE011",
                    Severity::Warning,
                    "Dangerous pattern rule - too broad, matches everything (consider using specific extensions like %.out: %.o)",
                    span,
                )
                .with_fix(Fix::new(&fix_replacement))
            );
        }
    }
    None
}

/// Check if line contains a dangerous pattern
fn is_dangerous_pattern(line: &str, pattern: &str) -> bool {
    // Check if line starts with the dangerous pattern
    // Pattern includes the colon (e.g., "%:" or "% :")
    let trimmed_line = line.trim_start();
    trimmed_line.starts_with(pattern)
}

/// Create a fix by suggesting a more specific pattern
fn create_fix(line: &str) -> String {
    // Replace bare "%" or "% " with more specific "%.out"
    if let Some(colon_pos) = line.find(':') {
        let prefix = &line[..colon_pos];
        let suffix = &line[colon_pos..];

        let new_prefix = prefix.trim().replace('%', "%.out");
        format!("{}{}", new_prefix, suffix)
    } else {
        line.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // RED PHASE: Write failing tests first

    #[test]
    fn test_MAKE011_detects_percent_colon() {
        let makefile = "%: %.o\n\t$(CC) $< -o $@";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "MAKE011");
        assert_eq!(diag.severity, Severity::Warning);
        assert!(diag.message.to_lowercase().contains("pattern"));
    }

    #[test]
    fn test_MAKE011_detects_percent_colon_with_space() {
        let makefile = "% : %.o\n\t$(CC) $< -o $@";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_MAKE011_provides_fix() {
        let makefile = "%: %.o\n\t$(CC) $< -o $@";
        let result = check(makefile);

        assert!(result.diagnostics[0].fix.is_some());
        let fix = result.diagnostics[0].fix.as_ref().unwrap();
        // Fix should suggest more specific pattern
        assert!(fix.replacement.contains("%") && fix.replacement.contains("."));
    }

    #[test]
    fn test_MAKE011_no_warning_for_specific_pattern() {
        let makefile = "%.out: %.o\n\t$(CC) $< -o $@";
        let result = check(makefile);

        // Specific pattern is OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE011_no_warning_for_regular_targets() {
        let makefile = "all: main.o\n\t$(CC) main.o -o all";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE011_detects_multiple_dangerous_patterns() {
        let makefile = "%: %.c\n\t$(CC) $< -o $@\n\n%: %.cpp\n\t$(CXX) $< -o $@";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_MAKE011_no_warning_for_double_suffix() {
        let makefile = "%.o: %.c\n\t$(CC) -c $< -o $@";
        let result = check(makefile);

        // .o: .c pattern is standard and safe
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE011_empty_makefile() {
        let makefile = "";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 0);
    }
}
