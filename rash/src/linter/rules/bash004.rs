//! BASH004: Dangerous rm -rf Without Validation
//!
//! **Rule**: Detect `rm -rf` with variables that could expand to `/` or empty
//!
//! **Why this matters**:
//! `rm -rf "$DIR"` where DIR is empty expands to `rm -rf ""` which can be
//! catastrophic. `rm -rf $DIR/` where DIR is empty becomes `rm -rf /`.
//! Always validate variables before destructive operations.
//!
//! ## Examples
//!
//! Bad:
//! ```bash
//! rm -rf "$DIR"
//! rm -rf $BUILD_DIR/
//! rm -rf "${INSTALL_PREFIX}"/lib
//! ```
//!
//! Good:
//! ```bash
//! rm -rf "${DIR:?Variable not set}"
//! [ -n "$BUILD_DIR" ] && rm -rf "$BUILD_DIR/"
//! # Or use a safe default
//! rm -rf "${INSTALL_PREFIX:-/usr/local}"/lib
//! ```

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};

/// Check for dangerous rm -rf with unvalidated variables
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim();

        // Skip comments and empty lines
        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }

        // Look for rm with -rf or -fr flags
        if !has_rm_force_recursive(trimmed) {
            continue;
        }

        // Check if rm target uses a variable
        if has_unguarded_variable(trimmed) {
            let span = Span::new(line_num + 1, 1, line_num + 1, line.len());

            let var_name = extract_first_variable(trimmed).unwrap_or("VAR");

            let mut diag = Diagnostic::new(
                "BASH004",
                Severity::Warning,
                format!(
                    "Dangerous rm -rf with unguarded variable ${} - use ${{{}:?}} to fail if unset/empty",
                    var_name, var_name
                ),
                span,
            );

            diag.fix = Some(Fix::new(format!("${{{}:?\"Variable not set\"}}", var_name)));

            result.add(diag);
        }
    }

    result
}

/// Check if a flag word contains both -r and -f
fn is_rf_flag(word: &str) -> bool {
    word.starts_with('-') && word.contains('r') && word.contains('f')
}

/// Check if line contains rm with -rf or -fr
fn has_rm_force_recursive(line: &str) -> bool {
    if !line.contains("rm ") {
        return false;
    }

    let words: Vec<&str> = line.split_whitespace().collect();
    let rm_pos = words.iter().position(|w| *w == "rm");

    let Some(pos) = rm_pos else { return false };

    // Check flags after rm (stop at first non-flag argument)
    words[pos + 1..]
        .iter()
        .take_while(|w| w.starts_with('-'))
        .any(|w| is_rf_flag(w))
}

/// Check if the rm target uses an unguarded variable
fn has_unguarded_variable(line: &str) -> bool {
    // Find the rm -rf portion and look at targets
    let parts: Vec<&str> = line.splitn(2, "rm").collect();
    if parts.len() < 2 {
        return false;
    }

    let after_rm = parts[1];

    // Skip flags to get to targets
    let words: Vec<&str> = after_rm.split_whitespace().collect();
    let target_start = words.iter().position(|w| !w.starts_with('-'));

    if let Some(start) = target_start {
        let targets = &words[start..];
        for target in targets {
            let t = target.trim_matches('"').trim_matches('\'');
            // Has variable but NOT guarded with :? or :-
            if (t.contains('$') && !t.contains("\\$"))
                && !t.contains(":?")
                && !t.contains(":-")
            {
                // Also not a simple known-safe variable like $HOME
                return true;
            }
        }
    }

    false
}

/// Extract the first variable name from the line
fn extract_first_variable(line: &str) -> Option<&str> {
    // Find $VAR or ${VAR} pattern
    if let Some(pos) = line.find('$') {
        let rest = &line[pos + 1..];
        if rest.starts_with('{') {
            // ${VAR...}
            if let Some(end) = rest.find('}') {
                let inner = &rest[1..end];
                // Strip modifiers like :? :- etc.
                let name = inner.split(':').next().unwrap_or(inner);
                return Some(name);
            }
        } else {
            // $VAR
            let end = rest
                .find(|c: char| !c.is_alphanumeric() && c != '_')
                .unwrap_or(rest.len());
            if end > 0 {
                return Some(&rest[..end]);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bash004_detects_rm_rf_variable() {
        let script = r#"rm -rf "$DIR""#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "BASH004");
    }

    #[test]
    fn test_bash004_detects_rm_rf_unquoted() {
        let script = "rm -rf $BUILD_DIR/";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_bash004_detects_rm_fr() {
        let script = r#"rm -fr "$DIR""#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_bash004_safe_with_guard() {
        let script = r#"rm -rf "${DIR:?Variable not set}""#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_bash004_safe_with_default() {
        let script = r#"rm -rf "${DIR:-/tmp/safe}""#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_bash004_safe_literal_path() {
        let script = "rm -rf /tmp/build";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_bash004_ignores_comments() {
        let script = r#"# rm -rf "$DIR""#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_bash004_has_autofix() {
        let script = r#"rm -rf "$DIR""#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].fix.is_some());
    }

    #[test]
    fn test_bash004_empty() {
        let result = check("");
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_bash004_rm_without_rf() {
        let script = r#"rm "$FILE""#;
        let result = check(script);
        // rm without -rf is not as dangerous
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_bash004_braced_variable() {
        let script = r#"rm -rf "${INSTALL_PREFIX}"/lib"#;
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 1);
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(10))]

        #[test]
        fn prop_bash004_never_panics(s in ".*") {
            let _ = check(&s);
        }

        #[test]
        fn prop_bash004_guarded_is_safe(
            var in "[A-Z_]{1,10}",
            msg in "[a-z ]{1,15}",
        ) {
            let script = format!("rm -rf \"${{{}:?{}}}\"", var, msg);
            let result = check(&script);
            prop_assert_eq!(result.diagnostics.len(), 0);
        }
    }
}
