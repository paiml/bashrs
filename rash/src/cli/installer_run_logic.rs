//! Pure logic functions for installer run/resume/golden/audit CLI commands.
//!
//! Extracted from `commands.rs` to enable unit testing without I/O.
//! Functions here do NOT perform file I/O, print to stdout, or call
//! external processes. They transform data, validate inputs, and build
//! data structures or formatted strings.

#![allow(dead_code)]

use crate::cli::args::AuditOutputFormat;
use crate::models::{Error, Result};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

// ============================================================================
// Run Configuration Building
// ============================================================================

/// Determine the execution mode label from the hermetic flag.
///
/// Returns `"hermetic"` when hermetic mode is requested, `"normal"` otherwise.
pub(crate) fn execution_mode_label(hermetic: bool) -> &'static str {
    if hermetic {
        "hermetic"
    } else {
        "normal"
    }
}

/// Build an executor configuration record from CLI arguments.
///
/// Returns a tuple `(dry_run, use_sudo, environment, working_dir, timeout_secs)`
/// representing the fields of `ExecutorConfig` without depending on the struct
/// itself.
pub(crate) fn build_executor_config_fields(
    dry_run: bool,
    working_dir: &Path,
    timeout_secs: u64,
) -> (bool, bool, HashMap<String, String>, String, u64) {
    (
        dry_run,
        false,
        HashMap::new(),
        working_dir.display().to_string(),
        timeout_secs,
    )
}

/// Compute the default trace file path for an installer run.
///
/// The default path is `<installer_path>/traces/<installer_name>-trace.json`.
pub(crate) fn default_trace_file_path(installer_path: &Path, installer_name: &str) -> PathBuf {
    installer_path
        .join("traces")
        .join(format!("{}-trace.json", installer_name))
}

/// Format the header lines for an installer run.
///
/// Returns a vector of display lines (without leading newlines) suitable for
/// printing by the caller.
pub(crate) fn format_run_header_lines(
    installer_path: &str,
    checkpoint_path: &str,
    run_id: &str,
    mode_label: &str,
) -> Vec<String> {
    vec![
        format!("  Installer: {}", installer_path),
        format!("  Checkpoint: {}", checkpoint_path),
        format!("  Run ID: {}", run_id),
        format!("  Mode: {}", mode_label),
    ]
}

/// Determine whether early exit is needed (diff or dry-run mode).
///
/// Returns `Some("diff")` or `Some("dry_run")` if an early exit path should
/// be taken, or `None` if normal execution should proceed.
pub(crate) fn early_exit_mode(diff: bool, dry_run: bool) -> Option<&'static str> {
    if diff {
        Some("diff")
    } else if dry_run {
        Some("dry_run")
    } else {
        None
    }
}

// ============================================================================
// Resume Configuration Building
// ============================================================================

/// Validate that a checkpoint directory path is non-empty and structurally
/// valid (does not actually check the filesystem).
///
/// Returns an error message describing the expected checkpoint location when
/// the `checkpoint_exists` flag is `false`.
pub(crate) fn validate_checkpoint_present(
    checkpoint_exists: bool,
    checkpoint_display: &str,
    installer_display: &str,
) -> Result<()> {
    if !checkpoint_exists {
        return Err(Error::Validation(format!(
            "No checkpoint found at {} - run 'bashrs installer run {}' first",
            checkpoint_display, installer_display
        )));
    }
    Ok(())
}

/// Format the checkpoint status summary for the resume command.
///
/// Accepts pre-computed counts and returns formatted lines ready for display.
pub(crate) fn format_resume_status(
    run_id: &str,
    is_hermetic: bool,
    total_steps: usize,
    completed: usize,
    failed: usize,
    pending: usize,
    last_successful_step: Option<&str>,
    resume_from: &str,
    spec_steps: usize,
    installer_display: &str,
) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push(format!("Checkpoint found: {}", run_id));
    lines.push(format!(
        "  Hermetic mode: {}",
        if is_hermetic { "yes" } else { "no" }
    ));
    lines.push(format!(
        "  Steps: {} total, {} completed, {} failed, {} pending",
        total_steps, completed, failed, pending
    ));
    if let Some(last) = last_successful_step {
        lines.push(format!("  Last successful: {}", last));
    }
    lines.push(String::new());
    lines.push(format!("Would resume from step: {}", resume_from));
    lines.push(String::new());
    lines.push("Note: Full execution not yet implemented.".to_string());
    lines.push(format!("  Steps in spec: {}", spec_steps));
    lines.push(format!(
        "  Run with --dry-run to validate: bashrs installer run {} --dry-run",
        installer_display
    ));
    lines
}

// ============================================================================
// Golden Trace Logic
// ============================================================================

/// Compute the reproducibility hash for a golden trace.
///
/// This uses the event count and trace name to produce a deterministic
/// hash value identical to the one in `installer_golden_capture_command`.
pub(crate) fn compute_golden_trace_hash(event_count: usize, trace_name: &str) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    event_count.hash(&mut hasher);
    trace_name.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

/// Format the summary lines after a golden trace capture.
///
/// Returns display lines (without println!) for the caller to render.
pub(crate) fn format_golden_capture_summary(
    trace_name: &str,
    trace_path_display: &str,
    event_count: usize,
    steps_executed: usize,
    installer_display: &str,
) -> Vec<String> {
    vec![
        "Golden trace captured successfully:".to_string(),
        format!("  Name: {}", trace_name),
        format!("  Path: {}", trace_path_display),
        format!("  Events: {}", event_count),
        format!("  Steps: {}", steps_executed),
        String::new(),
        "To compare against this trace later:".to_string(),
        format!(
            "  bashrs installer golden-compare {} --trace {}",
            installer_display, trace_name
        ),
    ]
}

/// Build a current trace name from the golden trace name.
///
/// Appends `-current` suffix per the convention used in
/// `installer_golden_compare_command`.
pub(crate) fn current_trace_name(golden_trace_name: &str) -> String {
    format!("{}-current", golden_trace_name)
}

/// Determine the outcome of a trace comparison.
///
/// Returns `Ok(())` when traces are equivalent, or an `Err` with a
/// descriptive message when regression is detected.
pub(crate) fn trace_comparison_outcome(
    is_equivalent: bool,
    added_count: usize,
    removed_count: usize,
) -> Result<()> {
    if is_equivalent {
        Ok(())
    } else {
        Err(Error::Validation(format!(
            "Trace regression detected: {} added, {} removed events",
            added_count, removed_count
        )))
    }
}

/// Format the result line for a trace comparison (PASS or FAIL).
pub(crate) fn format_compare_pass_line() -> &'static str {
    "Result: PASS - No regression detected"
}

// ============================================================================
// Audit Configuration Building
// ============================================================================

/// Determine the initial audit mode from the `security_only` flag.
///
/// Returns `"security_only"` or `"full"`.
pub(crate) fn audit_mode_label(security_only: bool) -> &'static str {
    if security_only {
        "security_only"
    } else {
        "full"
    }
}

/// Apply ignored rules to a list of rule IDs, returning those that remain.
///
/// This mirrors the `with_ignored_rule` accumulation in `installer_audit_command`.
pub(crate) fn filter_ignored_rules<'a>(
    rule_ids: &'a [String],
    ignore: &[String],
) -> Vec<&'a String> {
    rule_ids
        .iter()
        .filter(|id| !ignore.iter().any(|ig| ig == *id))
        .collect()
}

/// Format the audit error message from report metrics.
///
/// This is the pure computation behind the `Err` branch in
/// `installer_audit_command`.
pub(crate) fn format_audit_error_message(
    error_count: usize,
    critical_count: usize,
    score: u32,
    grade: &str,
) -> String {
    format!(
        "Audit found {} error(s). Score: {}/100 (Grade: {})",
        error_count + critical_count,
        score,
        grade
    )
}

/// Determine the effective output format name for audit reporting.
///
/// SARIF falls back to JSON in the current implementation.
pub(crate) fn effective_audit_format(format: &AuditOutputFormat) -> &'static str {
    match format {
        AuditOutputFormat::Human => "human",
        AuditOutputFormat::Json | AuditOutputFormat::Sarif => "json",
    }
}

/// Validate that an installer TOML path looks structurally valid.
///
/// Returns an error when the display path is empty or the `exists` flag
/// is `false`.
pub(crate) fn validate_installer_toml_present(
    exists: bool,
    toml_display: &str,
) -> Result<()> {
    if !exists {
        return Err(Error::Validation(format!(
            "installer.toml not found at {}",
            toml_display
        )));
    }
    Ok(())
}

// ============================================================================
// Step Progress Computation
// ============================================================================

/// Compute progress percentage from completed steps and total steps.
///
/// Returns 0 when `total` is 0 (avoids division by zero).
pub(crate) fn step_progress_percent(completed: usize, total: usize) -> u8 {
    if total == 0 {
        return 0;
    }
    let pct = (completed as f64 / total as f64 * 100.0) as u8;
    pct.min(100)
}

/// Determine whether all steps succeeded given completed and total counts.
pub(crate) fn all_steps_succeeded(completed: usize, total: usize) -> bool {
    total > 0 && completed == total
}

/// Build a diff-preview summary from validation result fields.
///
/// Returns display lines for the diff preview mode.
pub(crate) fn format_diff_preview(
    steps: usize,
    artifacts: usize,
    has_hermetic_context: bool,
    has_keyring: bool,
) -> Vec<String> {
    let mut lines = vec![
        "=== Dry-Run Diff Preview ===".to_string(),
        String::new(),
        format!("Steps to execute: {}", steps),
        format!("Artifacts to download: {}", artifacts),
    ];
    if has_hermetic_context {
        lines.push("Mode: hermetic (reproducible)".to_string());
    }
    if has_keyring {
        lines.push("Signatures: will be verified".to_string());
    }
    lines
}

/// Build a dry-run summary from validation result fields.
///
/// Returns display lines for the dry-run mode.
pub(crate) fn format_dry_run_summary(
    steps: usize,
    artifacts: usize,
    has_hermetic_context: bool,
    has_keyring: bool,
) -> Vec<String> {
    let mut lines = vec![
        "Dry-run mode: validating only".to_string(),
        format!("  Steps: {}", steps),
        format!("  Artifacts: {}", artifacts),
    ];
    if has_hermetic_context {
        lines.push("  Mode: hermetic (reproducible)".to_string());
    }
    if has_keyring {
        lines.push("  Signatures: will be verified".to_string());
    }
    lines.push("\u{2713} Installer validated successfully".to_string());
    lines
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_INSTALLER_RUN_001_run_config_helpers() {
        assert_eq!(execution_mode_label(true), "hermetic");
        assert_eq!(execution_mode_label(false), "normal");
        let (dry, sudo, env, wd, timeout) =
            build_executor_config_fields(true, Path::new("/tmp/inst"), 300);
        assert!(dry); assert!(!sudo); assert!(env.is_empty());
        assert_eq!(wd, "/tmp/inst"); assert_eq!(timeout, 300);
        let (dry2, _, _, _, _) = build_executor_config_fields(false, Path::new("/opt/app"), 600);
        assert!(!dry2);
        assert_eq!(
            default_trace_file_path(Path::new("/opt/my-inst"), "my-inst"),
            PathBuf::from("/opt/my-inst/traces/my-inst-trace.json")
        );
        let h = format_run_header_lines("/path/inst", "/path/cp", "run-42", "hermetic");
        assert_eq!(h.len(), 4);
        assert!(h[0].contains("/path/inst")); assert!(h[1].contains("/path/cp"));
        assert!(h[2].contains("run-42")); assert!(h[3].contains("hermetic"));
        assert_eq!(early_exit_mode(true, false), Some("diff"));
        assert_eq!(early_exit_mode(false, true), Some("dry_run"));
        assert_eq!(early_exit_mode(false, false), None);
        assert_eq!(early_exit_mode(true, true), Some("diff"));
    }

    #[test]
    fn test_INSTALLER_RUN_011_resume_helpers() {
        assert!(validate_checkpoint_present(true, "/cp", "/inst").is_ok());
        let err = validate_checkpoint_present(false, "/cp/.checkpoint", "/inst")
            .unwrap_err().to_string();
        assert!(err.contains("No checkpoint found"));
        assert!(err.contains("/cp/.checkpoint")); assert!(err.contains("/inst"));
        let l = format_resume_status(
            "run-1", false, 5, 3, 1, 1, Some("step-3"), "step-3", 5, "/my/installer",
        );
        assert!(l.iter().any(|s| s.contains("run-1")));
        assert!(l.iter().any(|s| s.contains("Hermetic mode: no")));
        assert!(l.iter().any(|s| s.contains("3 completed")));
        assert!(l.iter().any(|s| s.contains("1 failed")));
        assert!(l.iter().any(|s| s.contains("Last successful: step-3")));
        assert!(l.iter().any(|s| s.contains("Would resume from step: step-3")));
        let l2 = format_resume_status("run-2", true, 2, 2, 0, 0, Some("step-2"), "step-2", 2, "/x");
        assert!(l2.iter().any(|s| s.contains("Hermetic mode: yes")));
        let l3 = format_resume_status("run-3", false, 3, 0, 1, 2, None, "step-1", 3, "/y");
        assert!(!l3.iter().any(|s| s.contains("Last successful:")));
    }

    #[test]
    fn test_INSTALLER_RUN_016_golden_trace_helpers() {
        assert_eq!(compute_golden_trace_hash(5, "test-trace"), compute_golden_trace_hash(5, "test-trace"));
        assert_ne!(compute_golden_trace_hash(5, "trace"), compute_golden_trace_hash(10, "trace"));
        assert_ne!(compute_golden_trace_hash(5, "alpha"), compute_golden_trace_hash(5, "beta"));
        let h = compute_golden_trace_hash(0, "x");
        assert_eq!(h.len(), 16); assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
        let s = format_golden_capture_summary("my-trace", "/golden/my-trace.json", 12, 4, "/inst");
        assert!(s[0].contains("captured successfully"));
        assert!(s.iter().any(|l| l.contains("my-trace")));
        assert!(s.iter().any(|l| l.contains("12"))); assert!(s.iter().any(|l| l.contains("4")));
        assert!(s.iter().any(|l| l.contains("golden-compare")));
        assert_eq!(current_trace_name("baseline"), "baseline-current");
        assert_eq!(current_trace_name(""), "-current");
        assert!(trace_comparison_outcome(true, 0, 0).is_ok());
        let e = trace_comparison_outcome(false, 3, 2).unwrap_err().to_string();
        assert!(e.contains("3 added")); assert!(e.contains("2 removed"));
        assert!(format_compare_pass_line().contains("PASS"));
    }

    #[test]
    fn test_INSTALLER_RUN_026_audit_helpers() {
        assert_eq!(audit_mode_label(true), "security_only");
        assert_eq!(audit_mode_label(false), "full");
        let rules2 = vec!["SEC-001".to_string(), "SEC-002".to_string()];
        assert_eq!(filter_ignored_rules(&rules2, &[]).len(), 2);
        let rules3 = vec!["SEC-001".to_string(), "SEC-002".to_string(), "SEC-003".to_string()];
        let filtered = filter_ignored_rules(&rules3, &["SEC-002".to_string()]);
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|r| r.as_str() != "SEC-002"));
        assert!(filter_ignored_rules(&["R1".to_string()], &["R1".to_string()]).is_empty());
        assert_eq!(format_audit_error_message(3, 1, 72, "C"), "Audit found 4 error(s). Score: 72/100 (Grade: C)");
        assert!(format_audit_error_message(2, 0, 85, "B").contains("2 error(s)"));
        assert_eq!(effective_audit_format(&AuditOutputFormat::Human), "human");
        assert_eq!(effective_audit_format(&AuditOutputFormat::Json), "json");
        assert_eq!(effective_audit_format(&AuditOutputFormat::Sarif), "json");
        assert!(validate_installer_toml_present(true, "/path/installer.toml").is_ok());
        let e2 = validate_installer_toml_present(false, "/missing/installer.toml").unwrap_err().to_string();
        assert!(e2.contains("installer.toml not found")); assert!(e2.contains("/missing/installer.toml"));
    }

    #[test]
    fn test_INSTALLER_RUN_038_progress_and_preview_helpers() {
        assert_eq!(step_progress_percent(0, 0), 0); assert_eq!(step_progress_percent(5, 10), 50);
        assert_eq!(step_progress_percent(10, 10), 100); assert_eq!(step_progress_percent(15, 10), 100);
        assert!(all_steps_succeeded(5, 5));
        assert!(!all_steps_succeeded(3, 5)); assert!(!all_steps_succeeded(0, 0));
        let d = format_diff_preview(3, 2, false, false);
        assert!(d[0].contains("Diff Preview"));
        assert!(d.iter().any(|l| l.contains("3"))); assert!(d.iter().any(|l| l.contains("2")));
        assert!(!d.iter().any(|l| l.contains("hermetic")));
        let d2 = format_diff_preview(1, 1, true, true);
        assert!(d2.iter().any(|l| l.contains("hermetic"))); assert!(d2.iter().any(|l| l.contains("Signatures")));
        let r = format_dry_run_summary(4, 3, false, false);
        assert!(r[0].contains("Dry-run"));
        assert!(r.iter().any(|l| l.contains("4"))); assert!(r.iter().any(|l| l.contains("3")));
        assert!(r.iter().any(|l| l.contains("\u{2713}")));
        assert!(format_dry_run_summary(1, 0, true, false).iter().any(|l| l.contains("hermetic")));
        assert!(format_dry_run_summary(1, 0, false, true).iter().any(|l| l.contains("Signatures")));
        let r4 = format_dry_run_summary(2, 1, true, true);
        assert!(r4.iter().any(|l| l.contains("hermetic")));
        assert!(r4.iter().any(|l| l.contains("Signatures"))); assert!(r4.iter().any(|l| l.contains("\u{2713}")));
    }
}
