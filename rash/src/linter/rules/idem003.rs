//! IDEM003: Non-idempotent ln
//!
//! **Rule**: Detect `ln -s` without removing existing symlink first
//!
//! **Why this matters**:
//! `ln -s` fails if symlink exists, making scripts non-idempotent.
//! Re-running the script will fail instead of succeeding.
//!
//! **Auto-fix**: Suggest prepending `rm -f`
//!
//! ## Examples
//!
//! ❌ **BAD** (non-idempotent):
//! ```bash
//! ln -s /app/releases/v1.0 /app/current
//! ```
//!
//! ✅ **GOOD** (idempotent):
//! ```bash
//! rm -f /app/current && ln -s /app/releases/v1.0 /app/current
//! # OR use -f flag (force):
//! ln -sf /app/releases/v1.0 /app/current
//! ln -sfn /app/releases/v1.0 /app/current
//! ```

use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};
use regex::Regex;

/// Check for ln -s without rm -f first
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();

    // Match ln command with -s flag but WITHOUT -f flag
    // This regex captures ln with -s but excludes idempotent variants:
    // - ln -sf, ln -sfn (combined flags with f)
    // - ln -fs, ln -fns (combined flags with f first)
    // - ln -s ... -f (separate -f flag)
    let ln_pattern = Regex::new(r"\bln\s+(-[a-z]*s[a-z]*)\s").unwrap();

    for (line_num, line) in source.lines().enumerate() {
        // Skip if line has rm -f (already safe)
        if line.contains("rm -f") {
            continue;
        }

        // Check for ln -s pattern
        if let Some(caps) = ln_pattern.captures(line) {
            let flags = caps.get(1).map(|m| m.as_str()).unwrap_or("");

            // Skip if -f flag is present (makes it idempotent)
            // -f can be in combined flags like -sf, -sfn, -fs, -fns
            // or as a separate flag later in the command
            if flags.contains('f') || line.contains(" -f") {
                continue;
            }

            if let Some(col) = line.find("ln ") {
                let span = Span::new(line_num + 1, col + 1, line_num + 1, col + 6);

                let fix = Fix::new_unsafe(vec![
                    "Option 1: ln -sfn /source /target (force + no-dereference, most portable)"
                        .to_string(),
                    "Option 2: ln -sf /source /target (force, may follow existing symlinks)"
                        .to_string(),
                    "Option 3: rm -f /target && ln -s /source /target".to_string(),
                ]);

                let diag = Diagnostic::new(
                    "IDEM003",
                    Severity::Warning,
                    "Non-idempotent ln - requires manual fix (UNSAFE)",
                    span,
                )
                .with_fix(fix);

                result.add(diag);
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_IDEM003_detects_ln_without_rm() {
        let script = "ln -s /app/releases/v1.0 /app/current";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "IDEM003");
        assert_eq!(diag.severity, Severity::Warning);
    }

    #[test]
    fn test_IDEM003_no_warning_with_rm() {
        let script = "rm -f /app/current && ln -s /app/releases/v1.0 /app/current";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_IDEM003_no_warning_with_force_flag() {
        // ln -sf is idempotent (force flag removes existing)
        let script = "ln -sf /app/releases/v1.0 /app/current";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0, "ln -sf should be idempotent");

        // ln -sfn is also idempotent (force + no-dereference)
        let script = "ln -sfn /raid/target /src/target";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0, "ln -sfn should be idempotent");

        // ln -fs (f before s) is also idempotent
        let script = "ln -fs /app/releases/v1.0 /app/current";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0, "ln -fs should be idempotent");

        // ln -nfs (multiple flags with f) is also idempotent
        let script = "ln -nfs /app/releases/v1.0 /app/current";
        let result = check(script);
        assert_eq!(result.diagnostics.len(), 0, "ln -nfs should be idempotent");
    }

    #[test]
    fn test_IDEM003_no_warning_with_separate_force_flag() {
        // ln -s ... -f (separate -f flag)
        let script = "ln -s /app/releases/v1.0 /app/current -f";
        let result = check(script);
        assert_eq!(
            result.diagnostics.len(),
            0,
            "ln -s with separate -f should be idempotent"
        );
    }

    #[test]
    fn test_IDEM003_provides_fix() {
        let script = "ln -s /src /dst";
        let result = check(script);

        assert!(result.diagnostics[0].fix.is_some());
        let fix = result.diagnostics[0].fix.as_ref().unwrap();
        // UNSAFE fix: no automatic replacement, provides suggestions
        assert_eq!(fix.replacement, "");
        assert!(fix.is_unsafe());
        assert!(!fix.suggested_alternatives.is_empty());
        // Verify suggestions mention ln -sfn as the preferred option
        assert!(fix
            .suggested_alternatives
            .iter()
            .any(|s| s.contains("-sfn")));
    }

    #[test]
    fn test_IDEM003_detects_ln_sn_without_force() {
        // ln -sn (symbolic + no-dereference but NO force) is NOT idempotent
        let script = "ln -sn /app/releases/v1.0 /app/current";
        let result = check(script);
        assert_eq!(
            result.diagnostics.len(),
            1,
            "ln -sn without -f should trigger warning"
        );
    }
}
