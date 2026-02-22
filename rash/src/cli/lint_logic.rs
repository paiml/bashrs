//! Pure logic functions extracted from `lint_command` and related CLI functions.
//!
//! No I/O side-effects: no filesystem reads/writes, no `std::process::exit`,
//! no `println!`/`eprintln!`. Every function operates on data and returns data.
//!
//! **See also**: `gate_logic` (ignored rules, severity, format conversion),
//! `make_logic` (makefile lint predicates), `logic/lint.rs` (process_lint,
//! FileType, LintOptions).

use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

use crate::linter::{Diagnostic, LintResult, Severity};

/// Filter a `LintResult` by minimum severity and a set of ignored rule codes.
pub(crate) fn filter_diagnostics_by_severity_and_rules(
    result: LintResult, min_severity: Severity, ignored_rules: &HashSet<String>,
) -> LintResult {
    let filtered = result.diagnostics.into_iter()
        .filter(|d| d.severity >= min_severity)
        .filter(|d| !ignored_rules.contains(&d.code.to_uppercase()))
        .collect();
    LintResult { diagnostics: filtered }
}

/// The exit code a lint invocation should return based on diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LintExitCode {
    Clean,    // exit 0
    Warnings, // exit 1
    Errors,   // exit 2
}

impl LintExitCode {
    pub(crate) fn code(self) -> i32 {
        match self { Self::Clean => 0, Self::Warnings => 1, Self::Errors => 2 }
    }
}

/// Determine the exit code from a `LintResult`.
pub(crate) fn determine_exit_code(result: &LintResult) -> LintExitCode {
    if result.has_errors() { LintExitCode::Errors }
    else if result.has_warnings() { LintExitCode::Warnings }
    else { LintExitCode::Clean }
}

/// Aggregated severity counts for a set of diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct LintSummary {
    pub(crate) errors: usize,
    pub(crate) warnings: usize,
    pub(crate) info: usize,
    pub(crate) fixable: usize,
    pub(crate) total: usize,
}

impl LintSummary {
    /// Return `true` when the overall lint result is a pass (no errors or warnings).
    pub(crate) fn passed(&self) -> bool { self.errors == 0 && self.warnings == 0 }
}

/// Compute summary statistics from a `LintResult`.
pub(crate) fn compute_lint_summary(result: &LintResult) -> LintSummary {
    let mut s = LintSummary::default();
    for d in &result.diagnostics {
        match d.severity {
            Severity::Error => s.errors += 1,
            Severity::Warning => s.warnings += 1,
            Severity::Info | Severity::Note | Severity::Perf | Severity::Risk => s.info += 1,
        }
        if d.fix.is_some() { s.fixable += 1; }
    }
    s.total = result.diagnostics.len();
    s
}

/// Combined lint outcome: exit code + summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LintOutcome {
    pub(crate) exit_code: LintExitCode,
    pub(crate) summary: LintSummary,
}

/// Compute a full `LintOutcome` (exit code + summary) from a `LintResult`.
pub(crate) fn compute_lint_outcome(result: &LintResult) -> LintOutcome {
    LintOutcome { exit_code: determine_exit_code(result), summary: compute_lint_summary(result) }
}

/// Format a `LintSummary` as a human-readable one-line string.
pub(crate) fn format_lint_summary(s: &LintSummary) -> String {
    format!("{} issue(s): {} error, {} warning, {} info ({} fixable)",
            s.total, s.errors, s.warnings, s.info, s.fixable)
}

/// Return `true` when `--fix` was requested AND the result has fixable diagnostics.
pub(crate) fn should_apply_fixes(fix_requested: bool, result: &LintResult) -> bool {
    fix_requested && result.diagnostics.iter().any(|d| d.fix.is_some())
}

/// Count the number of fixable diagnostics.
pub(crate) fn count_fixable_diagnostics(result: &LintResult) -> usize {
    result.diagnostics.iter().filter(|d| d.fix.is_some()).count()
}

/// Count fixable diagnostics grouped by safety level: `(safe, assumptions, unsafe)`.
pub(crate) fn count_fixable_by_safety(result: &LintResult) -> (usize, usize, usize) {
    use crate::linter::FixSafetyLevel;
    let (mut safe, mut assumptions, mut unsafe_c) = (0, 0, 0);
    for d in &result.diagnostics {
        if let Some(ref fix) = d.fix {
            match fix.safety_level {
                FixSafetyLevel::Safe => safe += 1,
                FixSafetyLevel::SafeWithAssumptions => assumptions += 1,
                FixSafetyLevel::Unsafe => unsafe_c += 1,
            }
        }
    }
    (safe, assumptions, unsafe_c)
}

/// Build `FixOptions` from CLI parameters (extracted from `handle_lint_fixes`).
pub(crate) fn build_fix_options(
    fix_assumptions: bool, output: Option<&Path>,
) -> crate::linter::autofix::FixOptions {
    crate::linter::autofix::FixOptions {
        create_backup: true, dry_run: false,
        backup_suffix: ".bak".to_string(),
        apply_assumptions: fix_assumptions,
        output_path: output.map(|p| p.to_path_buf()),
    }
}

/// Build `FixOptions` for output-file (non-inplace) scenario.
pub(crate) fn build_fix_options_for_output() -> crate::linter::autofix::FixOptions {
    crate::linter::autofix::FixOptions {
        create_backup: false, dry_run: false, backup_suffix: String::new(),
        apply_assumptions: false, output_path: None,
    }
}

/// Map `Severity` to a single-character icon string for human output.
pub(crate) fn severity_to_icon(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "\u{274c}", Severity::Warning => "\u{26a0}", _ => "\u{2139}",
    }
}

/// Map `Severity` to its SARIF-level string.
pub(crate) fn severity_to_sarif_level(severity: Severity) -> &'static str {
    match severity { Severity::Error => "error", Severity::Warning => "warning", _ => "note" }
}

/// Format a single diagnostic as a human-readable line.
pub(crate) fn format_diagnostic_human(d: &Diagnostic) -> String {
    let icon = severity_to_icon(d.severity);
    let mut line = format!("{} Line {}: [{}] {}", icon, d.span.start_line, d.code, d.message);
    if let Some(ref fix) = d.fix { line.push_str(&format!("\n   Fix: {}", fix.replacement)); }
    line
}

/// Format a diagnostic as a compact single-line string (for `--quiet` or summary modes).
pub(crate) fn format_diagnostic_compact(d: &Diagnostic) -> String {
    format!("{}:{}: [{}] {}", d.span.start_line, severity_to_sarif_level(d.severity),
            d.code, d.message)
}

/// Group diagnostics by their rule code.  Returns a sorted map from code to
/// the list of diagnostics matching that code, useful for per-rule summaries.
pub(crate) fn group_diagnostics_by_code(result: &LintResult) -> BTreeMap<String, Vec<&Diagnostic>> {
    let mut groups: BTreeMap<String, Vec<&Diagnostic>> = BTreeMap::new();
    for d in &result.diagnostics { groups.entry(d.code.clone()).or_default().push(d); }
    groups
}

/// Build a SARIF 2.1.0 result entry for a single diagnostic.
pub(crate) fn build_sarif_result_entry(d: &Diagnostic, file_path: &str) -> serde_json::Value {
    serde_json::json!({
        "ruleId": d.code,
        "level": severity_to_sarif_level(d.severity),
        "message": { "text": d.message },
        "locations": [{ "physicalLocation": {
            "artifactLocation": { "uri": file_path },
            "region": {
                "startLine": d.span.start_line, "startColumn": d.span.start_col,
                "endLine": d.span.end_line, "endColumn": d.span.end_col
            }
        }}]
    })
}

/// Return `true` when CITL export should be attempted.
pub(crate) fn should_export_citl(citl_export_path: Option<&Path>) -> bool {
    citl_export_path.is_some()
}

/// Return `true` when a non-standard Dockerfile lint profile is in use.
pub(crate) fn should_log_lint_profile(
    is_dockerfile: bool, profile: crate::linter::rules::LintProfile,
) -> bool {
    is_dockerfile && profile != crate::linter::rules::LintProfile::Standard
}

/// Summarise fix application results as a human-readable string.
pub(crate) fn format_fix_summary(
    fixes_applied: usize, backup_path: Option<&str>, input_display: &str,
) -> String {
    let mut msg = format!("Applied {} fix(es) to {}", fixes_applied, input_display);
    if let Some(bp) = backup_path { msg.push_str(&format!("\nBackup created at {}", bp)); }
    msg
}

/// Build the backup `PathBuf` for inplace shell lint fixes (append `.bak`).
pub(crate) fn lint_backup_path(input: &Path) -> PathBuf {
    let mut p = input.as_os_str().to_owned();
    p.push(".bak");
    PathBuf::from(p)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::linter::{Fix, Span};
    fn sp() -> Span { Span::new(1, 1, 1, 10) }
    fn sp2(l: usize) -> Span { Span::new(l, 1, l, 20) }
    fn de(c: &str) -> Diagnostic { Diagnostic::new(c, Severity::Error, "err", sp()) }
    fn dw(c: &str) -> Diagnostic { Diagnostic::new(c, Severity::Warning, "warn", sp()) }
    fn di(c: &str) -> Diagnostic { Diagnostic::new(c, Severity::Info, "info", sp()) }
    fn dwf(c: &str) -> Diagnostic {
        Diagnostic::new(c, Severity::Warning, "fixable", sp()).with_fix(Fix::new("rep"))
    }
    fn r(d: Vec<Diagnostic>) -> LintResult { LintResult { diagnostics: d } }
    fn ign(c: &[&str]) -> HashSet<String> { c.iter().map(|s| s.to_string()).collect() }

    #[test]
    fn test_LINT_LOGIC_001_filter_severity() {
        let f = filter_diagnostics_by_severity_and_rules(
            r(vec![di("I1"), dw("W1"), de("E1")]), Severity::Warning, &ign(&[]));
        assert_eq!(f.diagnostics.len(), 2);
        assert!(f.diagnostics.iter().all(|d| d.severity >= Severity::Warning));
    }
    #[test]
    fn test_LINT_LOGIC_002_filter_ignored_rules() {
        let f = filter_diagnostics_by_severity_and_rules(
            r(vec![dw("SEC001"), dw("DET002")]), Severity::Info, &ign(&["SEC001"]));
        assert_eq!(f.diagnostics.len(), 1);
        assert_eq!(f.diagnostics[0].code, "DET002");
    }
    #[test]
    fn test_LINT_LOGIC_003_filter_case_insensitive() {
        let f = filter_diagnostics_by_severity_and_rules(
            r(vec![dw("sec001")]), Severity::Info, &ign(&["SEC001"]));
        assert!(f.diagnostics.is_empty());
    }
    #[test]
    fn test_LINT_LOGIC_004_filter_combined() {
        assert!(filter_diagnostics_by_severity_and_rules(
            r(vec![]), Severity::Info, &ign(&[])).diagnostics.is_empty());
        let f = filter_diagnostics_by_severity_and_rules(
            r(vec![di("S1"), dw("S2"), dw("D1"), de("S3")]), Severity::Warning, &ign(&["S2"]));
        assert_eq!(f.diagnostics.len(), 2);
    }
    #[test]
    fn test_LINT_LOGIC_005_filter_error_min_multi_ignore() {
        let f1 = filter_diagnostics_by_severity_and_rules(
            r(vec![di("I"), dw("W"), de("E")]), Severity::Error, &ign(&[]));
        assert_eq!(f1.diagnostics.len(), 1);
        let f2 = filter_diagnostics_by_severity_and_rules(
            r(vec![dw("a"), dw("b"), dw("d")]), Severity::Info, &ign(&["A", "B", "C"]));
        assert_eq!(f2.diagnostics.len(), 1);
    }
    #[test]
    fn test_LINT_LOGIC_006_exit_code_all_variants() {
        assert_eq!(determine_exit_code(&r(vec![])), LintExitCode::Clean);
        assert_eq!(determine_exit_code(&r(vec![di("I")])), LintExitCode::Clean);
        assert_eq!(determine_exit_code(&r(vec![dw("W")])), LintExitCode::Warnings);
        assert_eq!(determine_exit_code(&r(vec![de("E")])), LintExitCode::Errors);
        assert_eq!(determine_exit_code(&r(vec![dw("W"), de("E")])), LintExitCode::Errors);
        assert_eq!(LintExitCode::Clean.code(), 0);
        assert_eq!(LintExitCode::Warnings.code(), 1);
        assert_eq!(LintExitCode::Errors.code(), 2);
    }
    #[test]
    fn test_LINT_LOGIC_007_summary_empty_and_mixed() {
        let s = compute_lint_summary(&r(vec![]));
        assert_eq!((s.total, s.errors, s.warnings, s.info, s.fixable), (0, 0, 0, 0, 0));
        let s2 = compute_lint_summary(&r(vec![de("E"), dw("W1"), dw("W2"), di("I")]));
        assert_eq!((s2.total, s2.errors, s2.warnings, s2.info), (4, 1, 2, 1));
    }
    #[test]
    fn test_LINT_LOGIC_008_summary_fixable_count() {
        let s = compute_lint_summary(&r(vec![dwf("W1"), dw("W2"),
            Diagnostic::new("E1", Severity::Error, "f", sp()).with_fix(Fix::new("x"))]));
        assert_eq!(s.fixable, 2);
    }
    #[test]
    fn test_LINT_LOGIC_009_format_summary() {
        let s = LintSummary { errors: 1, warnings: 2, info: 3, fixable: 1, total: 6 };
        let t = format_lint_summary(&s);
        assert!(t.contains("6 issue(s)") && t.contains("1 error") && t.contains("1 fixable"));
        assert!(format_lint_summary(&LintSummary::default()).starts_with("0 issue(s)"));
    }
    #[test]
    fn test_LINT_LOGIC_010_summary_passed() {
        assert!(compute_lint_summary(&r(vec![])).passed());
        assert!(compute_lint_summary(&r(vec![di("I1"), di("I2")])).passed());
        assert!(!compute_lint_summary(&r(vec![dw("W")])).passed());
        assert!(!compute_lint_summary(&r(vec![de("E")])).passed());
    }
    #[test]
    fn test_LINT_LOGIC_011_outcome_clean() {
        let o = compute_lint_outcome(&r(vec![]));
        assert_eq!(o.exit_code, LintExitCode::Clean);
        assert!(o.summary.passed());
    }
    #[test]
    fn test_LINT_LOGIC_012_outcome_errors() {
        let o = compute_lint_outcome(&r(vec![de("E"), dw("W")]));
        assert_eq!(o.exit_code, LintExitCode::Errors);
        assert!(!o.summary.passed());
    }
    #[test]
    fn test_LINT_LOGIC_013_outcome_warnings() {
        let o = compute_lint_outcome(&r(vec![dw("W"), di("I")]));
        assert_eq!(o.exit_code, LintExitCode::Warnings);
    }
    #[test]
    fn test_LINT_LOGIC_014_apply_fixes() {
        assert!(should_apply_fixes(true, &r(vec![dwf("W")])));
        assert!(!should_apply_fixes(false, &r(vec![dwf("W")])));
        assert!(!should_apply_fixes(true, &r(vec![dw("W")])));
        assert!(!should_apply_fixes(true, &r(vec![])));
    }
    #[test]
    fn test_LINT_LOGIC_015_count_fixable() {
        assert_eq!(count_fixable_diagnostics(&r(vec![dwf("W1"), dw("W2"), dwf("W3")])), 2);
        assert_eq!(count_fixable_diagnostics(&r(vec![dw("W"), de("E")])), 0);
    }
    #[test]
    fn test_LINT_LOGIC_016_fixable_by_safety() {
        assert_eq!(count_fixable_by_safety(&r(vec![dwf("W1"), dwf("W2")])), (2, 0, 0));
        let da = Diagnostic::new("W", Severity::Warning, "f", sp())
            .with_fix(Fix::new_with_assumptions("x", vec!["a".into()]));
        assert_eq!(count_fixable_by_safety(&r(vec![da])), (0, 1, 0));
        let du = Diagnostic::new("W", Severity::Warning, "f", sp())
            .with_fix(Fix::new_unsafe(vec!["o".into()]));
        assert_eq!(count_fixable_by_safety(&r(vec![du])), (0, 0, 1));
    }
    #[test]
    fn test_LINT_LOGIC_017_fix_options() {
        let o = build_fix_options(false, None);
        assert!(o.create_backup && !o.dry_run && !o.apply_assumptions && o.output_path.is_none());
        assert!(build_fix_options(true, None).apply_assumptions);
        let o2 = build_fix_options(false, Some(Path::new("/tmp/f.sh")));
        assert_eq!(o2.output_path.as_deref(), Some(Path::new("/tmp/f.sh")));
        let o3 = build_fix_options_for_output();
        assert!(!o3.create_backup && o3.backup_suffix.is_empty());
    }
    #[test]
    fn test_LINT_LOGIC_018_severity_icons_and_sarif() {
        assert_eq!(severity_to_icon(Severity::Error), "\u{274c}");
        assert_eq!(severity_to_icon(Severity::Warning), "\u{26a0}");
        assert_eq!(severity_to_icon(Severity::Info), "\u{2139}");
        assert_eq!(severity_to_icon(Severity::Note), "\u{2139}");
        assert_eq!(severity_to_sarif_level(Severity::Error), "error");
        assert_eq!(severity_to_sarif_level(Severity::Warning), "warning");
        assert_eq!(severity_to_sarif_level(Severity::Info), "note");
    }
    #[test]
    fn test_LINT_LOGIC_019_format_human() {
        let t = format_diagnostic_human(&dw("SEC001"));
        assert!(t.contains("Line 1") && t.contains("[SEC001]") && !t.contains("Fix:"));
        let t2 = format_diagnostic_human(&dwf("SEC002"));
        assert!(t2.contains("[SEC002]") && t2.contains("Fix: rep"));
    }
    #[test]
    fn test_LINT_LOGIC_020_format_compact() {
        assert!(format_diagnostic_compact(&de("SEC001")).starts_with("1:error:"));
        assert!(format_diagnostic_compact(&dw("DET002")).contains(":warning:"));
        assert!(format_diagnostic_compact(&di("INFO1")).contains(":note:"));
    }
    #[test]
    fn test_LINT_LOGIC_021_citl_and_profile() {
        assert!(should_export_citl(Some(Path::new("/tmp/c.json"))));
        assert!(!should_export_citl(None));
        use crate::linter::rules::LintProfile;
        assert!(should_log_lint_profile(true, LintProfile::Coursera));
        assert!(!should_log_lint_profile(true, LintProfile::Standard));
        assert!(!should_log_lint_profile(false, LintProfile::Coursera));
    }
    #[test]
    fn test_LINT_LOGIC_022_fix_summary_and_backup() {
        let t1 = format_fix_summary(3, None, "s.sh");
        assert!(t1.contains("Applied 3 fix(es)") && !t1.contains("Backup"));
        let t2 = format_fix_summary(2, Some("s.sh.bak"), "s.sh");
        assert!(t2.contains("Backup created at s.sh.bak"));
        assert_eq!(lint_backup_path(Path::new("d.sh")), PathBuf::from("d.sh.bak"));
        assert_eq!(lint_backup_path(Path::new("/a/b.sh")), PathBuf::from("/a/b.sh.bak"));
    }
    #[test]
    fn test_LINT_LOGIC_023_group_diagnostics() {
        let empty = r(vec![]);
        assert!(group_diagnostics_by_code(&empty).is_empty());
        let res = r(vec![dw("SEC001"), dw("DET002"), de("SEC001")]);
        let g = group_diagnostics_by_code(&res);
        assert_eq!(g.len(), 2);
        assert_eq!(g["SEC001"].len(), 2);
        assert_eq!(g["DET002"].len(), 1);
    }
    #[test]
    fn test_LINT_LOGIC_024_group_sorted() {
        let res = r(vec![dw("ZZZ"), dw("AAA"), dw("MMM")]);
        let grouped = group_diagnostics_by_code(&res);
        let keys: Vec<&String> = grouped.keys().collect();
        assert_eq!(keys, vec!["AAA", "MMM", "ZZZ"]);
    }
    #[test]
    fn test_LINT_LOGIC_025_sarif_entry_full() {
        let e = build_sarif_result_entry(&de("SEC001"), "script.sh");
        assert_eq!(e["ruleId"], "SEC001");
        assert_eq!(e["level"], "error");
        assert_eq!(e["message"]["text"], "err");
        let loc = &e["locations"][0]["physicalLocation"];
        assert_eq!(loc["artifactLocation"]["uri"], "script.sh");
        assert_eq!(loc["region"]["startLine"], 1);
    }
    #[test]
    fn test_LINT_LOGIC_026_sarif_entry_levels() {
        assert_eq!(build_sarif_result_entry(&dw("D"), "a.sh")["level"], "warning");
        assert_eq!(build_sarif_result_entry(&di("I"), "b.sh")["level"], "note");
    }
    #[test]
    fn test_LINT_LOGIC_027_sarif_entry_region_and_path() {
        let d = Diagnostic::new("R", Severity::Error, "msg", sp2(42));
        let region = &build_sarif_result_entry(&d, "x.sh")["locations"][0]
            ["physicalLocation"]["region"];
        assert_eq!(region["startLine"], 42);
        assert_eq!(region["endLine"], 42);
        let e = build_sarif_result_entry(&de("E"), "path/with spaces/f.sh");
        assert_eq!(e["locations"][0]["physicalLocation"]["artifactLocation"]["uri"],
                   "path/with spaces/f.sh");
    }
}
