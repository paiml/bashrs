//! DET002: Non-deterministic timestamp usage that reaches a build artifact
//!
//! **Rule**: A timestamp is a reproducibility defect when it reaches the *name
//! or contents of a build artifact*. A timestamp on a log line is the point of
//! a log line, and a timestamp derived from `SOURCE_DATE_EPOCH` is this rule's
//! own remedy - GH-230: both used to be reported identically to real
//! non-determinism, which made the rule unactionable.
//!
//! The destination analysis lives in [`crate::linter::timestamp_flow`].
//!
//! **Auto-fix**: UNSAFE - the remedy needs human judgement, so suggestions only.
//!
//! ## Examples
//!
//! BAD (the artifact's name changes on every run):
//! ```bash
//! RELEASE="release-$(date +%s)"
//! TS=$(date +%Y%m%d); cp build.log "out/report_$TS.log"
//! ```
//!
//! GOOD (reproducible, or not an artifact at all):
//! ```bash
//! BUILD_DATE=$(date -u -d "@${SOURCE_DATE_EPOCH:-$(git log -1 --format=%ct)}" +%Y%m%d)
//! echo "[$(date)] started" >> "$LOG_FILE"
//! ```

use crate::linter::timestamp_flow::{analyze, SinkClass, TimestampUse};
use crate::linter::{Diagnostic, Fix, LintResult, Severity, Span};

/// The remedy DET002 hands the user. It reads `SOURCE_DATE_EPOCH`, so pasting
/// it into a script does **not** re-trigger DET002 - that self-contradiction is
/// the core of GH-230 and `test_GH230_remedy_text_does_not_self_flag` pins it.
const REMEDY: &str = "Derive it from SOURCE_DATE_EPOCH: \
     BUILD_DATE=$(date -u -d \"@${SOURCE_DATE_EPOCH:-$(git log -1 --format=%ct)}\" +%Y%m%d)";

/// Advice for the case where the timestamp is only ever logged.
const LOG_ADVICE: &str = "If it is only for logging, send it to an append-only sink \
     (>>, tee -a, logger), or suppress with `# bashrs disable-line=DET002`.";

/// Check for timestamp usage that reaches a reproducible sink.
pub fn check(source: &str) -> LintResult {
    let mut result = LintResult::new();
    for u in analyze(source) {
        if u.class != SinkClass::Benign {
            result.add(build_diagnostic(&u));
        }
    }
    result
}

/// Build the diagnostic for one reportable timestamp.
///
/// The span deliberately stays on the `date` occurrence rather than moving to
/// the sink: `# bashrs disable-line=DET002` and `.bashrsignore` line scopes are
/// both keyed on `span.start_line`, so moving the anchor would silently
/// invalidate every existing user suppression. The sink is named in the message
/// instead.
fn build_diagnostic(u: &TimestampUse) -> Diagnostic {
    let span = Span::new(u.line, u.col, u.line, u.col + u.len);
    Diagnostic::new("DET002", Severity::Error, message_for(u), span).with_fix(det002_fix())
}

/// Message text, naming the sink line when we proved one.
fn message_for(u: &TimestampUse) -> String {
    match (u.class, u.sink_line, u.sink_text.as_deref(), u.var.as_deref()) {
        (SinkClass::Reproducible, Some(l), Some(t), _) => format!(
            "Timestamp reaches reproducible output at line {l}: `{}` - the artifact's name or \
             contents change on every run. {REMEDY}",
            elide(t, 60)
        ),
        (_, _, _, Some(v)) => format!(
            "Timestamp captured in `${v}`; its destination is not provably reproducible. \
             If it names or fills a build artifact: {REMEDY}. {LOG_ADVICE}"
        ),
        _ => format!(
            "Timestamp used here; its destination is not provably reproducible. \
             {REMEDY} {LOG_ADVICE}"
        ),
    }
}

/// Shorten a quoted source line for the message.
fn elide(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let head: String = s.chars().take(max.saturating_sub(3)).collect();
    format!("{head}...")
}

/// UNSAFE fix: five alternatives, every one of them DET002-clean itself.
fn det002_fix() -> Fix {
    Fix::new_unsafe(vec![
        format!("Reproducible builds: {REMEDY} - reading SOURCE_DATE_EPOCH clears DET002"),
        "Use the release version: RELEASE=\"release-${VERSION}\"".to_string(),
        "Use the commit: RELEASE=\"release-$(git rev-parse --short HEAD)\"".to_string(),
        "Send the timestamp to a log, not an artifact: `... >> \"$LOG_FILE\"`, \
         `... | tee -a \"$LOG_FILE\"`, or `logger \"...\"`"
            .to_string(),
        "Suppress with rationale: # bashrs disable-line=DET002".to_string(),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_DET002_detects_date_epoch() {
        let script = "RELEASE=\"release-$(date +%s)\"";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        assert_eq!(diag.code, "DET002");
        assert_eq!(diag.severity, Severity::Error);
    }

    #[test]
    fn test_DET002_detects_date_command_sub() {
        let script = "BUILD_ID=$(date +%Y%m%d)";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_DET002_detects_backtick_date() {
        let script = "TIMESTAMP=`date`";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_DET002_provides_fix() {
        let script = "ID=$(date +%s)";
        let result = check(script);

        assert!(result.diagnostics[0].fix.is_some());
        let fix = result.diagnostics[0].fix.as_ref().unwrap();
        // UNSAFE fix: no automatic replacement, provides suggestions
        assert_eq!(fix.replacement, "");
        assert!(fix.is_unsafe());
        assert!(!fix.suggested_alternatives.is_empty());
        assert!(fix.suggested_alternatives.len() >= 3);
    }

    #[test]
    fn test_DET002_no_false_positive() {
        let script = "RELEASE=\"release-${VERSION}\"";
        let result = check(script);

        assert_eq!(result.diagnostics.len(), 0);
    }

    // RED TEST: Issue #43 - Allow timestamps for benchmark result tracking
    #[test]
    fn test_DET002_allows_intentional_timestamp_for_benchmarks() {
        let script = r#"#!/bin/bash
# Intentional: timestamp for result tracking
TIMESTAMP=$(date +%Y-%m-%d_%H-%M-%S)
RESULT_FILE="benchmarks/results/baseline_$TIMESTAMP.md"
"#;
        let result = check(script);

        // Should NOT flag as error when marked intentional for tracking
        assert_eq!(
            result.diagnostics.len(),
            0,
            "Intentionally marked timestamp for benchmark tracking should not be flagged"
        );
    }

    #[test]
    fn test_DET002_allows_benchmark_result_comment() {
        // Alternative marker: "benchmark result" in comment
        let script = r#"#!/bin/bash
# Generate benchmark result file
RESULT_FILE="results/baseline_$(date +%s).md"
"#;
        let result = check(script);

        assert_eq!(
            result.diagnostics.len(),
            0,
            "Timestamp for benchmark/logging should be allowed with marker comment"
        );
    }

    // GH-230: this assertion encoded the bug. A timestamp that is only
    // *compared* never reaches a build artifact, so it is not a reproducibility
    // defect and DET002 must not report it. Time-dependent control flow is a
    // real but different concern; it belongs in its own rule, not in DET002.
    #[test]
    fn test_GH230_det002_comparison_sink_not_flagged() {
        let script = r#"#!/bin/bash
# Intentional: timestamp for result tracking
if [ $(date +%s) -gt 1000 ]; then
    echo "error"
fi
"#;
        let result = check(script);

        assert_eq!(
            result.diagnostics.len(),
            0,
            "Timestamp in a comparison is not a reproducible-output defect (GH-230)"
        );
    }

    // Issue #58: Metrics recording scripts should not be flagged
    #[test]
    fn test_DET002_allows_metrics_recording_marker() {
        let script = r#"#!/bin/bash
# Metrics recording script - timestamps are THE PURPOSE
TIMESTAMP=$(date +%Y-%m-%d_%H-%M-%S)
METRIC_FILE="metrics_$TIMESTAMP.json"
"#;
        let result = check(script);

        assert_eq!(
            result.diagnostics.len(),
            0,
            "Metrics recording script should not flag DET002 (Issue #58)"
        );
    }

    #[test]
    fn test_DET002_allows_record_metric_marker() {
        let script = r#"#!/bin/bash
# Record metric to pmat database
TIMESTAMP=$(date +%s)
echo "$TIMESTAMP,$VALUE" >> metrics.csv
"#;
        let result = check(script);

        assert_eq!(
            result.diagnostics.len(),
            0,
            "Record-metric script should not flag DET002 (Issue #58)"
        );
    }

    #[test]
    fn test_DET002_allows_telemetry_marker() {
        let script = r#"#!/bin/bash
# Telemetry collection for observability
TIMESTAMP=$(date +%s)
send_metric "$TIMESTAMP"
"#;
        let result = check(script);

        assert_eq!(
            result.diagnostics.len(),
            0,
            "Telemetry script should not flag DET002 (Issue #58)"
        );
    }
}
