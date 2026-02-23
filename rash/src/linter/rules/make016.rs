//! MAKE016: Unquoted variable in prerequisites
//!
//! **Rule**: Detect unquoted variables in target prerequisites
//!
//! **Why this matters**:
//! Variables in prerequisites should be quoted to handle filenames with spaces.
//! Unquoted variables like `$(FILES)` will break if any filename contains spaces.
//! GNU Make doesn't automatically quote variable expansions, so this must be
//! done explicitly. This is especially important for `$(wildcard)` results.
//!
//! **Auto-fix**: Add quotes around variable references in prerequisites
//!
//! ## Examples
//!
//! ❌ **BAD** (unquoted variable - breaks with spaces):
//! ```makefile
//! app: $(FILES)
//! \t$(CC) $(FILES) -o app
//! ```
//!
//! ✅ **GOOD** (quoted variable - handles spaces):
//! ```makefile
//! app: "$(FILES)"
//! \t$(CC) "$(FILES)" -o app
//! ```

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};

/// Check for unquoted variables in prerequisites
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        // Skip comment lines
        if line.trim_start().starts_with('#') {
            continue;
        }

        // Skip non-target lines (must contain ':')
        if !is_target_line(line) {
            continue;
        }

        // Extract prerequisites part (after ':')
        if let Some(prerequisites) = extract_prerequisites(line) {
            // Find all unquoted variables in prerequisites
            let unquoted_vars = find_unquoted_variables(&prerequisites);

            for var in unquoted_vars {
                let span = Span::new(line_num + 1, 1, line_num + 1, line.len());
                let fix_replacement = create_fix(line, &var);

                let diag = Diagnostic::new(
                    "MAKE016",
                    Severity::Warning,
                    format!("Unquoted variable '{}' in prerequisites - may break with spaces in filenames", var),
                    span,
                )
                .with_fix(Fix::new(&fix_replacement));

                result.add(diag);
            }
        }
    }

    result
}

/// Check if line is a target line (contains ':' and not a recipe)
fn is_target_line(line: &str) -> bool {
    line.contains(':') && !line.starts_with('\t')
}

/// Extract prerequisites part from target line (everything after ':')
fn extract_prerequisites(line: &str) -> Option<String> {
    if let Some(colon_pos) = line.find(':') {
        let prereqs = line[colon_pos + 1..].trim();
        if !prereqs.is_empty() {
            return Some(prereqs.to_string());
        }
    }
    None
}

/// Find all unquoted variables in prerequisites
/// Returns variable references like "$(FILES)" that are not already quoted
fn find_unquoted_variables(prerequisites: &str) -> Vec<String> {
    let mut vars = Vec::new();
    let mut chars = prerequisites.chars().peekable();
    let mut in_quote = false;
    let mut pos = 0;

    while let Some(ch) = chars.next() {
        match ch {
            '"' => in_quote = !in_quote,
            '$' if !in_quote => {
                collect_unquoted_var_at(prerequisites, pos, &mut chars, &mut vars);
            }
            _ => {}
        }
        pos += ch.len_utf8();
    }

    vars
}

/// Check if the current position starts a variable reference and collect it
fn collect_unquoted_var_at(
    source: &str,
    pos: usize,
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    vars: &mut Vec<String>,
) {
    if let Some(&next_ch) = chars.peek() {
        if next_ch == '(' || next_ch == '{' {
            if let Some(var) = extract_variable_ref(&source[pos..]) {
                if !is_automatic_variable(&var) {
                    vars.push(var);
                }
            }
        }
    }
}

/// Extract a variable reference starting at position (e.g., "$(FILES)")
fn extract_variable_ref(s: &str) -> Option<String> {
    if !s.starts_with("$(") && !s.starts_with("${") {
        return None;
    }

    let close_char = if s.starts_with("$(") { ')' } else { '}' };
    if let Some(close_pos) = s.find(close_char) {
        return Some(s[..=close_pos].to_string());
    }

    None
}

/// Check if a variable is an automatic variable ($@, $<, $^, $?, $*, $+)
fn is_automatic_variable(var: &str) -> bool {
    let content = var
        .trim_start_matches("$(")
        .trim_start_matches("${")
        .trim_end_matches(')')
        .trim_end_matches('}');

    // Automatic variables are single character
    content.len() == 1
        && matches!(
            content.chars().next(),
            Some('@' | '<' | '^' | '?' | '*' | '+')
        )
}

/// Create a fix by adding quotes around the unquoted variable
fn create_fix(line: &str, unquoted_var: &str) -> String {
    // Replace first occurrence of unquoted variable with quoted version
    line.replacen(unquoted_var, &format!("\"{}\"", unquoted_var), 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    // RED PHASE: Write failing tests first

    #[test]
    fn test_MAKE016_detects_unquoted_variable() {
        let makefile = "app: $(FILES)\n\t$(CC) $(FILES) -o app";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "MAKE016");
        assert_eq!(diag.severity, Severity::Warning);
        assert!(
            diag.message.to_lowercase().contains("variable")
                || diag.message.to_lowercase().contains("quote")
        );
    }

    #[test]
    fn test_MAKE016_detects_wildcard_variable() {
        let makefile = "app: $(wildcard *.c)\n\t$(CC) $^ -o app";
        let result = check(makefile);

        // $(wildcard) in prerequisites should be quoted
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_MAKE016_detects_multiple_variables() {
        let makefile = "app: $(SOURCES) $(HEADERS)\n\t$(CC) $^ -o app";
        let result = check(makefile);

        // Two unquoted variables
        assert_eq!(result.diagnostics.len(), 2);
    }

    #[test]
    fn test_MAKE016_provides_fix() {
        let makefile = "app: $(FILES)\n\t$(CC) $(FILES) -o app";
        let result = check(makefile);

        assert!(result.diagnostics[0].fix.is_some());
        let fix = result.diagnostics[0].fix.as_ref().unwrap();
        // Fix should add quotes
        assert!(fix.replacement.contains("\"$(FILES)\""));
    }

    #[test]
    fn test_MAKE016_no_warning_for_quoted_variables() {
        let makefile = "app: \"$(FILES)\"\n\t$(CC) \"$(FILES)\" -o app";
        let result = check(makefile);

        // Quoted variables are OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE016_no_warning_for_simple_targets() {
        let makefile = "app: main.c utils.c\n\t$(CC) $^ -o app";
        let result = check(makefile);

        // No variables in prerequisites - OK
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE016_no_warning_for_automatic_variables() {
        let makefile = "%.o: %.c\n\t$(CC) -c $< -o $@";
        let result = check(makefile);

        // Automatic variables ($<, $@, $^) don't need quotes in prerequisites
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE016_empty_makefile() {
        let makefile = "";
        let result = check(makefile);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_MAKE016_no_warning_for_variables_in_comments() {
        // Comments should be completely ignored, even if they contain $(VAR) patterns
        let makefile = r#"# MAKE016 (2 warnings): Unquoted $(MAKE) variable in comments - not applicable
# Cannot quote variables in comments
app: $(FILES)
	$(CC) $^ -o app"#;
        let result = check(makefile);

        // Should only warn about $(FILES) in actual target line, not in comments
        assert_eq!(
            result.diagnostics.len(),
            1,
            "Should ignore variables in comments"
        );
        assert!(result.diagnostics[0].message.contains("$(FILES)"));
    }

    #[test]
    fn test_MAKE016_ignores_comment_only_lines() {
        let makefile = "# target: $(DEPS)\n# This is documentation\napp: actual_file\n\tgcc app.c";
        let result = check(makefile);

        // No warnings - comments are ignored
        assert_eq!(result.diagnostics.len(), 0);
    }
}
