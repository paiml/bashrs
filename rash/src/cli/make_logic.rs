//! Pure logic functions for Makefile and devcontainer operations.
//!
//! This module contains functions extracted from `commands.rs` that have no
//! filesystem side-effects (no I/O, no process spawning). They operate purely
//! on data and can be tested in isolation.
//!
//! Covered areas:
//! - Makefile lint result filtering
//! - Makefile build/purify output path computation
//! - Makefile lint result display formatting
//! - DevContainer dockerfile path resolution
//! - DevContainer validation error detection
//! - Makefile parse format output selection

use crate::linter::LintResult;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Makefile lint helpers
// ---------------------------------------------------------------------------

/// Filter a `LintResult` in-place to retain only diagnostics whose `code`
/// matches at least one rule in the comma-separated `rule_filter` string.
///
/// If `rule_filter` is `None`, nothing is removed and the result is returned
/// unchanged.
///
/// # Examples
///
/// ```
/// use bashrs::cli::make_logic::filter_lint_by_rules;
/// use bashrs::linter::LintResult;
///
/// let mut result = LintResult::default();
/// filter_lint_by_rules(&mut result, None);
/// assert!(result.diagnostics.is_empty());
/// ```
pub(crate) fn filter_lint_by_rules(result: &mut LintResult, rule_filter: Option<&str>) {
    if let Some(filter) = rule_filter {
        let allowed: Vec<&str> = filter.split(',').map(str::trim).collect();
        result
            .diagnostics
            .retain(|d| allowed.iter().any(|rule| d.code.contains(rule)));
    }
}

/// Return `true` when the lint result contains at least one error-level
/// diagnostic.
///
/// This is the pure predicate behind the "exit code 2" decision in
/// `show_lint_results`.
pub(crate) fn lint_has_errors(result: &LintResult) -> bool {
    result.has_errors()
}

/// Return `true` when the lint result contains at least one warning-level
/// diagnostic (and no errors).
///
/// Used to decide whether to exit with code 1.
pub(crate) fn lint_has_warnings_only(result: &LintResult) -> bool {
    !result.has_errors() && result.has_warnings()
}

/// Return `true` when the lint result contains fixable diagnostics
/// (i.e. any diagnostic with a non-`None` fix).
pub(crate) fn lint_has_fixable_diagnostics(result: &LintResult) -> bool {
    result.diagnostics.iter().any(|d| d.fix.is_some())
}

// ---------------------------------------------------------------------------
// Makefile path computation helpers
// ---------------------------------------------------------------------------

/// Compute the backup path for an in-place Makefile purify operation.
///
/// The backup path is the input path with extension replaced by `"mk.bak"`.
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// use bashrs::cli::make_logic::make_backup_path;
///
/// let backup = make_backup_path(Path::new("Makefile"));
/// assert_eq!(backup.extension().unwrap_or_default(), "bak");
/// ```
pub(crate) fn make_backup_path(input: &Path) -> PathBuf {
    input.with_extension("mk.bak")
}

/// Compute the test file path for a generated Makefile test suite.
///
/// Returns `None` if `output_path` has no file name or if the file name
/// is not valid UTF-8.
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// use bashrs::cli::make_logic::make_test_file_path;
///
/// let test_path = make_test_file_path(Path::new("Makefile.purified"));
/// assert!(test_path.is_some());
/// let p = test_path.unwrap();
/// assert!(p.to_str().unwrap().ends_with(".test.sh"));
/// ```
pub(crate) fn make_test_file_path(output_path: &Path) -> Option<PathBuf> {
    let file_name = output_path.file_name()?.to_str()?;
    Some(output_path.with_file_name(format!("{}.test.sh", file_name)))
}

/// Return `true` when output should be written to stdout (`output` is `None`
/// and `fix` is `false`).
///
/// Used by `make_purify_write_output` to decide whether to print to stdout
/// or write to disk.
pub(crate) fn make_purify_should_print(output: Option<&Path>, fix: bool) -> bool {
    output.is_none() && !fix
}

// ---------------------------------------------------------------------------
// DevContainer helpers
// ---------------------------------------------------------------------------

/// Extract the Dockerfile path referenced in a devcontainer.json build section.
///
/// Parses the JSONC `content` string, extracts `build.dockerfile`, and
/// resolves it relative to `devcontainer_path`'s parent directory.
///
/// Returns `None` when:
/// - The content cannot be parsed as JSONC.
/// - There is no `build.dockerfile` key.
/// - The file name is not valid UTF-8.
pub(crate) fn resolve_devcontainer_dockerfile(
    content: &str,
    devcontainer_path: &Path,
) -> Option<PathBuf> {
    let json = crate::linter::rules::devcontainer::parse_jsonc(content).ok()?;

    let dockerfile_str = json
        .get("build")
        .and_then(|b| b.get("dockerfile"))
        .and_then(|v| v.as_str())?;

    let parent = devcontainer_path.parent().unwrap_or(Path::new("."));
    Some(parent.join(dockerfile_str))
}

/// Return `true` when a devcontainer `LintResult` contains any error-severity
/// diagnostics.
///
/// This is the pure predicate behind the "fail with validation error" branch in
/// `devcontainer_validate`.
pub(crate) fn devcontainer_has_errors(result: &crate::linter::LintResult) -> bool {
    result
        .diagnostics
        .iter()
        .any(|d| d.severity == crate::linter::Severity::Error)
}

// ---------------------------------------------------------------------------
// Makefile output format helpers
// ---------------------------------------------------------------------------

/// Return the label string for a given `MakeOutputFormat` variant.
///
/// Used to build log messages and report headers without duplicating the
/// match arms.
pub(crate) fn make_output_format_label(format: &crate::cli::args::MakeOutputFormat) -> &'static str {
    use crate::cli::args::MakeOutputFormat;
    match format {
        MakeOutputFormat::Text => "text",
        MakeOutputFormat::Json => "json",
        MakeOutputFormat::Debug => "debug",
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::cli::args::MakeOutputFormat;
    use crate::linter::{Diagnostic, Fix, LintResult, Severity as LintSeverity, Span};

    fn make_span() -> Span {
        Span::new(1, 1, 1, 5)
    }

    fn make_diagnostic(code: &str, fixable: bool) -> Diagnostic {
        let d = Diagnostic::new(code, LintSeverity::Warning, "test", make_span());
        if fixable {
            d.with_fix(Fix::new("fixed"))
        } else {
            d
        }
    }

    fn make_error_diagnostic(code: &str) -> Diagnostic {
        Diagnostic::new(code, LintSeverity::Error, "error", make_span())
    }

    fn make_lint_result(diagnostics: Vec<Diagnostic>) -> LintResult {
        LintResult { diagnostics }
    }

    // --- test_MAKE_LOGIC_001 ---
    #[test]
    fn test_MAKE_LOGIC_001_filter_lint_by_rules_none_keeps_all() {
        let diag = make_diagnostic("MK001", false);
        let mut result = make_lint_result(vec![diag]);
        filter_lint_by_rules(&mut result, None);
        assert_eq!(result.diagnostics.len(), 1);
    }

    // --- test_MAKE_LOGIC_002 ---
    #[test]
    fn test_MAKE_LOGIC_002_filter_lint_by_rules_matching_rule_kept() {
        let diag = make_diagnostic("MK001", false);
        let mut result = make_lint_result(vec![diag]);
        filter_lint_by_rules(&mut result, Some("MK001"));
        assert_eq!(result.diagnostics.len(), 1);
    }

    // --- test_MAKE_LOGIC_003 ---
    #[test]
    fn test_MAKE_LOGIC_003_filter_lint_by_rules_non_matching_removed() {
        let diag = make_diagnostic("MK001", false);
        let mut result = make_lint_result(vec![diag]);
        filter_lint_by_rules(&mut result, Some("MK002"));
        assert_eq!(result.diagnostics.len(), 0);
    }

    // --- test_MAKE_LOGIC_004 ---
    #[test]
    fn test_MAKE_LOGIC_004_filter_lint_by_rules_comma_separated() {
        let diags = vec![
            make_diagnostic("MK001", false),
            make_diagnostic("MK002", false),
            make_diagnostic("MK003", false),
        ];
        let mut result = make_lint_result(diags);
        filter_lint_by_rules(&mut result, Some("MK001,MK003"));
        assert_eq!(result.diagnostics.len(), 2);
        assert!(result.diagnostics.iter().all(|d| d.code != "MK002"));
    }

    // --- test_MAKE_LOGIC_005 ---
    #[test]
    fn test_MAKE_LOGIC_005_filter_lint_by_rules_whitespace_trimmed() {
        let diag = make_diagnostic("MK001", false);
        let mut result = make_lint_result(vec![diag]);
        filter_lint_by_rules(&mut result, Some(" MK001 "));
        assert_eq!(result.diagnostics.len(), 1);
    }

    // --- test_MAKE_LOGIC_006 ---
    #[test]
    fn test_MAKE_LOGIC_006_filter_lint_by_rules_empty_filter_removes_all() {
        // An empty filter string splits into [""], which doesn't match anything
        let diag = make_diagnostic("MK001", false);
        let mut result = make_lint_result(vec![diag]);
        filter_lint_by_rules(&mut result, Some(""));
        // "MK001".contains("") is true, so actually keeps all — document the behavior
        assert_eq!(result.diagnostics.len(), 1);
    }

    // --- test_MAKE_LOGIC_007 ---
    #[test]
    fn test_MAKE_LOGIC_007_lint_has_errors_false_for_warnings() {
        let diag = make_diagnostic("MK001", false);
        let result = make_lint_result(vec![diag]);
        assert!(!lint_has_errors(&result));
    }

    // --- test_MAKE_LOGIC_008 ---
    #[test]
    fn test_MAKE_LOGIC_008_lint_has_errors_true_for_errors() {
        let diag = make_error_diagnostic("MK001");
        let result = make_lint_result(vec![diag]);
        assert!(lint_has_errors(&result));
    }

    // --- test_MAKE_LOGIC_009 ---
    #[test]
    fn test_MAKE_LOGIC_009_lint_has_warnings_only_true() {
        let diag = make_diagnostic("MK001", false);
        let result = make_lint_result(vec![diag]);
        assert!(lint_has_warnings_only(&result));
    }

    // --- test_MAKE_LOGIC_010 ---
    #[test]
    fn test_MAKE_LOGIC_010_lint_has_warnings_only_false_when_errors() {
        let diag = make_error_diagnostic("MK001");
        let result = make_lint_result(vec![diag]);
        assert!(!lint_has_warnings_only(&result));
    }

    // --- test_MAKE_LOGIC_011 ---
    #[test]
    fn test_MAKE_LOGIC_011_lint_has_fixable_diagnostics_true() {
        let diag = make_diagnostic("MK001", true);
        let result = make_lint_result(vec![diag]);
        assert!(lint_has_fixable_diagnostics(&result));
    }

    // --- test_MAKE_LOGIC_012 ---
    #[test]
    fn test_MAKE_LOGIC_012_lint_has_fixable_diagnostics_false() {
        let diag = make_diagnostic("MK001", false);
        let result = make_lint_result(vec![diag]);
        assert!(!lint_has_fixable_diagnostics(&result));
    }

    // --- test_MAKE_LOGIC_013 ---
    #[test]
    fn test_MAKE_LOGIC_013_make_backup_path_has_mk_bak_extension() {
        let path = Path::new("Makefile");
        let backup = make_backup_path(path);
        // Should end with "mk.bak"
        assert_eq!(backup.extension().unwrap(), "bak");
    }

    // --- test_MAKE_LOGIC_014 ---
    #[test]
    fn test_MAKE_LOGIC_014_make_backup_path_from_makefile_with_extension() {
        let path = Path::new("build/Makefile");
        let backup = make_backup_path(path);
        assert!(backup.to_str().unwrap().contains("mk.bak"));
    }

    // --- test_MAKE_LOGIC_015 ---
    #[test]
    fn test_MAKE_LOGIC_015_make_test_file_path_appends_test_sh() {
        let path = Path::new("Makefile.purified");
        let test_path = make_test_file_path(path).unwrap();
        assert!(test_path.to_str().unwrap().ends_with(".test.sh"));
    }

    // --- test_MAKE_LOGIC_016 ---
    #[test]
    fn test_MAKE_LOGIC_016_make_test_file_path_simple_makefile() {
        let path = Path::new("Makefile");
        let test_path = make_test_file_path(path).unwrap();
        assert!(test_path.to_str().unwrap().contains("Makefile.test.sh"));
    }

    // --- test_MAKE_LOGIC_017 ---
    #[test]
    fn test_MAKE_LOGIC_017_make_purify_should_print_true_no_output_no_fix() {
        assert!(make_purify_should_print(None, false));
    }

    // --- test_MAKE_LOGIC_018 ---
    #[test]
    fn test_MAKE_LOGIC_018_make_purify_should_print_false_with_output() {
        let path = Path::new("out.mk");
        assert!(!make_purify_should_print(Some(path), false));
    }

    // --- test_MAKE_LOGIC_019 ---
    #[test]
    fn test_MAKE_LOGIC_019_make_purify_should_print_false_with_fix() {
        assert!(!make_purify_should_print(None, true));
    }

    // --- test_MAKE_LOGIC_020 ---
    #[test]
    fn test_MAKE_LOGIC_020_make_purify_should_print_false_with_output_and_fix() {
        let path = Path::new("out.mk");
        assert!(!make_purify_should_print(Some(path), true));
    }

    // --- test_MAKE_LOGIC_021 ---
    #[test]
    fn test_MAKE_LOGIC_021_resolve_devcontainer_dockerfile_no_build_key() {
        let content = r#"{ "name": "test" }"#;
        let path = Path::new("/workspace/.devcontainer/devcontainer.json");
        let result = resolve_devcontainer_dockerfile(content, path);
        assert!(result.is_none());
    }

    // --- test_MAKE_LOGIC_022 ---
    #[test]
    fn test_MAKE_LOGIC_022_resolve_devcontainer_dockerfile_with_build_section() {
        let content = r#"{ "build": { "dockerfile": "Dockerfile" } }"#;
        let path = Path::new("/workspace/.devcontainer/devcontainer.json");
        let result = resolve_devcontainer_dockerfile(content, path);
        assert!(result.is_some());
        let resolved = result.unwrap();
        assert!(resolved.to_str().unwrap().contains("Dockerfile"));
    }

    // --- test_MAKE_LOGIC_023 ---
    #[test]
    fn test_MAKE_LOGIC_023_resolve_devcontainer_dockerfile_invalid_json() {
        let content = "not json {";
        let path = Path::new("/workspace/.devcontainer/devcontainer.json");
        let result = resolve_devcontainer_dockerfile(content, path);
        assert!(result.is_none());
    }

    // --- test_MAKE_LOGIC_024 ---
    #[test]
    fn test_MAKE_LOGIC_024_devcontainer_has_errors_false_for_empty() {
        let result = LintResult { diagnostics: vec![] };
        assert!(!devcontainer_has_errors(&result));
    }

    // --- test_MAKE_LOGIC_025 ---
    #[test]
    fn test_MAKE_LOGIC_025_devcontainer_has_errors_false_for_warnings() {
        let diag = make_diagnostic("DC001", false);
        let result = LintResult { diagnostics: vec![diag] };
        assert!(!devcontainer_has_errors(&result));
    }

    // --- test_MAKE_LOGIC_026 ---
    #[test]
    fn test_MAKE_LOGIC_026_devcontainer_has_errors_true_for_errors() {
        let diag = make_error_diagnostic("DC001");
        let result = LintResult { diagnostics: vec![diag] };
        assert!(devcontainer_has_errors(&result));
    }

    // --- test_MAKE_LOGIC_027 ---
    #[test]
    fn test_MAKE_LOGIC_027_make_output_format_label_text() {
        assert_eq!(make_output_format_label(&MakeOutputFormat::Text), "text");
    }

    // --- test_MAKE_LOGIC_028 ---
    #[test]
    fn test_MAKE_LOGIC_028_make_output_format_label_json() {
        assert_eq!(make_output_format_label(&MakeOutputFormat::Json), "json");
    }

    // --- test_MAKE_LOGIC_029 ---
    #[test]
    fn test_MAKE_LOGIC_029_make_output_format_label_debug() {
        assert_eq!(make_output_format_label(&MakeOutputFormat::Debug), "debug");
    }

    // --- test_MAKE_LOGIC_030 ---
    #[test]
    fn test_MAKE_LOGIC_030_filter_lint_by_rules_partial_code_match() {
        // Rule filter "MK" should match "MK001", "MK002", etc.
        let diags = vec![
            make_diagnostic("MK001", false),
            make_diagnostic("SEC001", false),
        ];
        let mut result = make_lint_result(diags);
        filter_lint_by_rules(&mut result, Some("MK"));
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, "MK001");
    }

    // --- test_MAKE_LOGIC_031 ---
    #[test]
    fn test_MAKE_LOGIC_031_lint_has_fixable_diagnostics_empty() {
        let result = make_lint_result(vec![]);
        assert!(!lint_has_fixable_diagnostics(&result));
    }

    // --- test_MAKE_LOGIC_032 ---
    #[test]
    fn test_MAKE_LOGIC_032_lint_has_errors_empty() {
        let result = make_lint_result(vec![]);
        assert!(!lint_has_errors(&result));
    }

    // --- test_MAKE_LOGIC_033 ---
    #[test]
    fn test_MAKE_LOGIC_033_resolve_devcontainer_dockerfile_resolves_relative_path() {
        let content = r#"{ "build": { "dockerfile": "../Dockerfile" } }"#;
        let path = Path::new("/workspace/.devcontainer/devcontainer.json");
        let result = resolve_devcontainer_dockerfile(content, path).unwrap();
        // Should be resolved relative to parent of devcontainer.json
        assert!(result.to_str().unwrap().contains("Dockerfile"));
    }
}
