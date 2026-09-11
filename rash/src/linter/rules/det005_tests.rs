//! RED tests for DET005 (#232): time-dependent control flow, split from
//! DET002. Mirrors the four `#232` examples plus the duration exemption and
//! the DET002 true positive that must survive the split.

use super::check;
use crate::linter::rules::det002;
use crate::linter::Severity;

/// `if [ "$(date +%H)" -lt 6 ]; then ... fi` - #232 example 1.
#[test]
fn test_PMAT255_gh232_d1_if_hour_check_is_det005_warning_not_det002() {
    let script = "#!/bin/sh\nif [ \"$(date +%H)\" -lt 6 ]; then echo early; fi\n";

    let det005 = check(script);
    assert_eq!(det005.diagnostics.len(), 1, "expected one DET005 finding");
    assert_eq!(det005.diagnostics[0].code, "DET005");
    assert_eq!(det005.diagnostics[0].severity, Severity::Warning);

    let det002 = det002::check(script);
    assert_eq!(
        det002.diagnostics.len(),
        0,
        "DET002 must not double-report a DET005 line (#232)"
    );
}

/// `while [ "$(date +%s)" -lt "$deadline" ]; do ... done` - #232 example 2.
/// Decision: a timeout loop against a relative deadline still fires DET005 -
/// termination depends on wall-clock time, the exact hazard this rule names.
#[test]
fn test_PMAT255_gh232_d2_while_deadline_is_det005_warning_not_det002() {
    let script =
        "#!/bin/sh\ndeadline=5\nwhile [ \"$(date +%s)\" -lt \"$deadline\" ]; do sleep 1; done\n";

    let det005 = check(script);
    assert_eq!(det005.diagnostics.len(), 1, "expected one DET005 finding");
    assert_eq!(det005.diagnostics[0].severity, Severity::Warning);

    let det002 = det002::check(script);
    assert_eq!(
        det002.diagnostics.len(),
        0,
        "DET002 must stay silent (#232)"
    );
}

/// `case "$(date +%u)" in 6|7) echo weekend ;; esac` - #232 example 3.
#[test]
fn test_PMAT255_gh232_d3_case_weekday_is_det005_warning_not_det002() {
    let script = "#!/bin/sh\ncase \"$(date +%u)\" in 6|7) echo weekend ;; esac\n";

    let det005 = check(script);
    assert_eq!(det005.diagnostics.len(), 1, "expected one DET005 finding");
    assert_eq!(det005.diagnostics[0].severity, Severity::Warning);

    let det002 = det002::check(script);
    assert_eq!(
        det002.diagnostics.len(),
        0,
        "DET002 must stay silent (#232)"
    );
}

/// `[ "$(date +%s)" -gt "$expiry" ] && exit 1` - #232 example 4.
#[test]
fn test_PMAT255_gh232_d4_expiry_guard_is_det005_warning_not_det002() {
    let script = "#!/bin/sh\nexpiry=1\n[ \"$(date +%s)\" -gt \"$expiry\" ] && exit 1\n";

    let det005 = check(script);
    assert_eq!(det005.diagnostics.len(), 1, "expected one DET005 finding");
    assert_eq!(det005.diagnostics[0].severity, Severity::Warning);

    let det002 = det002::check(script);
    assert_eq!(
        det002.diagnostics.len(),
        0,
        "DET002 must stay silent (#232)"
    );
}

/// A duration - the arithmetic difference of two timestamp captures - fires
/// NEITHER rule: measuring elapsed time is the point of measuring elapsed
/// time (#232).
#[test]
fn test_PMAT255_gh232_d5_duration_neither_rule() {
    let script = "#!/bin/sh\nstart=$(date +%s)\nsleep 1\nend=$(date +%s)\n\
                  elapsed=$(( end - start ))\necho \"$elapsed\"\n";

    let det005 = check(script);
    assert_eq!(
        det005.diagnostics.len(),
        0,
        "a duration is not a branch condition (#232)"
    );

    let det002 = det002::check(script);
    assert_eq!(
        det002.diagnostics.len(),
        0,
        "a duration is not a reproducibility defect (#232)"
    );
}

/// `echo "$(date)" > VERSION` - the true positive that must survive the
/// DET002/DET005 split: this reaches an artifact, not a branch condition.
#[test]
fn test_PMAT255_gh232_d6_det002_still_fires_on_artifact_write() {
    let script = "#!/bin/sh\necho \"$(date)\" > VERSION\n";

    let det002 = det002::check(script);
    assert_eq!(
        det002.diagnostics.len(),
        1,
        "DET002 must still catch a timestamp reaching a build artifact (#232)"
    );
    assert_eq!(det002.diagnostics[0].code, "DET002");

    let det005 = check(script);
    assert_eq!(
        det005.diagnostics.len(),
        0,
        "DET005 must not double-report a DET002 line (#232)"
    );
}
