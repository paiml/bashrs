#![allow(clippy::unwrap_used)]

use super::*;
use crate::linter::{Diagnostic as BashDiag, Fix, FixSafetyLevel, Severity, Span};

// ---------------------------------------------------------------------------
// to_lsp_severity tests
// ---------------------------------------------------------------------------

#[test]
fn test_pmat211_severity_error() {
    assert_eq!(to_lsp_severity(Severity::Error), DiagnosticSeverity::ERROR);
}

#[test]
fn test_pmat211_severity_warning() {
    assert_eq!(
        to_lsp_severity(Severity::Warning),
        DiagnosticSeverity::WARNING
    );
}

#[test]
fn test_pmat211_severity_risk() {
    assert_eq!(to_lsp_severity(Severity::Risk), DiagnosticSeverity::WARNING);
}

#[test]
fn test_pmat211_severity_info() {
    assert_eq!(to_lsp_severity(Severity::Info), DiagnosticSeverity::HINT);
}

#[test]
fn test_pmat211_severity_note() {
    assert_eq!(
        to_lsp_severity(Severity::Note),
        DiagnosticSeverity::INFORMATION
    );
}

#[test]
fn test_pmat211_severity_perf() {
    assert_eq!(
        to_lsp_severity(Severity::Perf),
        DiagnosticSeverity::INFORMATION
    );
}

// ---------------------------------------------------------------------------
// to_lsp_range tests
// ---------------------------------------------------------------------------

#[test]
fn test_pmat211_range_single_line() {
    let span = Span::new(1, 5, 1, 10);
    let range = to_lsp_range(span);
    assert_eq!(range.start.line, 0);
    assert_eq!(range.start.character, 4);
    assert_eq!(range.end.line, 0);
    assert_eq!(range.end.character, 9);
}

#[test]
fn test_pmat211_range_multi_line() {
    let span = Span::new(3, 1, 5, 20);
    let range = to_lsp_range(span);
    assert_eq!(range.start.line, 2);
    assert_eq!(range.start.character, 0);
    assert_eq!(range.end.line, 4);
    assert_eq!(range.end.character, 19);
}

#[test]
fn test_pmat211_range_zero_width() {
    let span = Span::point(10, 5);
    let range = to_lsp_range(span);
    assert_eq!(range.start.line, 9);
    assert_eq!(range.start.character, 4);
    assert_eq!(range.end, range.start);
}

#[test]
fn test_pmat211_range_line_one_col_one() {
    let span = Span::new(1, 1, 1, 1);
    let range = to_lsp_range(span);
    assert_eq!(range.start.line, 0);
    assert_eq!(range.start.character, 0);
}

// ---------------------------------------------------------------------------
// to_lsp_diagnostic tests
// ---------------------------------------------------------------------------

#[test]
fn test_pmat211_diagnostic_conversion() {
    let span = Span::new(2, 6, 2, 13);
    let diag = BashDiag::new("SC2086", Severity::Warning, "Double quote variable", span);
    let lsp_diag = to_lsp_diagnostic(&diag);

    assert_eq!(lsp_diag.source, Some("bashrs".to_string()));
    assert_eq!(lsp_diag.code, Some(NumberOrString::String("SC2086".into())));
    assert_eq!(lsp_diag.severity, Some(DiagnosticSeverity::WARNING));
    assert_eq!(lsp_diag.message, "Double quote variable");
    assert_eq!(lsp_diag.range.start.line, 1);
    assert_eq!(lsp_diag.range.start.character, 5);
}

#[test]
fn test_pmat211_diagnostic_error_conversion() {
    let span = Span::new(1, 1, 1, 5);
    let diag = BashDiag::new("DET001", Severity::Error, "Non-deterministic", span);
    let lsp_diag = to_lsp_diagnostic(&diag);

    assert_eq!(lsp_diag.severity, Some(DiagnosticSeverity::ERROR));
    assert_eq!(lsp_diag.code, Some(NumberOrString::String("DET001".into())));
}

// ---------------------------------------------------------------------------
// lint_document integration tests
// ---------------------------------------------------------------------------

#[test]
fn test_pmat211_lint_shell_produces_diagnostics() {
    let url = Url::parse("file:///tmp/test.sh").unwrap();
    let diags = lint_document("#!/bin/bash\necho $RANDOM\n", &url);
    assert!(!diags.is_empty(), "Expected diagnostics for $RANDOM usage");
}

#[test]
fn test_pmat211_lint_clean_shell_no_errors() {
    let url = Url::parse("file:///tmp/clean.sh").unwrap();
    let diags = lint_document("#!/bin/sh\necho \"hello\"\n", &url);
    let serious: Vec<_> = diags
        .iter()
        .filter(|d| d.severity >= Severity::Warning)
        .collect();
    assert!(serious.is_empty(), "Clean script should have no warnings/errors");
}

#[test]
fn test_pmat211_lint_makefile_detected() {
    let url = Url::parse("file:///project/Makefile").unwrap();
    let diags = lint_document("all:\n\techo hello\n", &url);
    let _ = diags; // Just verify no panic
}

#[test]
fn test_pmat211_lint_dockerfile_detected() {
    let url = Url::parse("file:///project/Dockerfile").unwrap();
    let diags = lint_document("FROM ubuntu:latest\nRUN apt-get install curl\n", &url);
    let _ = diags;
}

// ---------------------------------------------------------------------------
// Code action tests (PMAT-214)
// ---------------------------------------------------------------------------

#[test]
fn test_pmat214_code_action_safe_fix() {
    let span = Span::new(2, 6, 2, 8);
    let fix = Fix::new("\"$x\"");
    let diag = BashDiag::new("SC2086", Severity::Warning, "Double quote", span).with_fix(fix);
    let url = Url::parse("file:///tmp/test.sh").unwrap();

    let action = to_code_action(&diag, &url);
    assert!(action.is_some(), "Should produce a code action for safe fix");

    if let Some(CodeActionOrCommand::CodeAction(ca)) = action {
        assert!(ca.title.contains("SC2086"));
        assert_eq!(ca.kind, Some(CodeActionKind::QUICKFIX));
        assert_eq!(ca.is_preferred, Some(true)); // Safe fixes are preferred
        let edit = ca.edit.unwrap();
        let changes = edit.changes.unwrap();
        let edits = changes.get(&url).unwrap();
        assert_eq!(edits[0].new_text, "\"$x\"");
    }
}

#[test]
fn test_pmat214_code_action_safe_with_assumptions() {
    let span = Span::new(3, 1, 3, 6);
    let fix = Fix::new_with_assumptions(
        "mkdir -p",
        vec!["Directory permissions are standard".into()],
    );
    let diag = BashDiag::new("IDEM001", Severity::Warning, "Non-idempotent mkdir", span)
        .with_fix(fix);
    let url = Url::parse("file:///tmp/test.sh").unwrap();

    let action = to_code_action(&diag, &url);
    assert!(action.is_some());

    if let Some(CodeActionOrCommand::CodeAction(ca)) = action {
        assert!(ca.title.contains("with assumptions"));
        assert_eq!(ca.is_preferred, Some(false)); // Not preferred (has assumptions)
    }
}

#[test]
fn test_pmat214_no_code_action_for_unsafe_fix() {
    let span = Span::new(1, 1, 1, 10);
    let fix = Fix::new_unsafe(vec!["Replace with safe alternative".into()]);
    let diag =
        BashDiag::new("DET001", Severity::Error, "Non-deterministic", span).with_fix(fix);
    let url = Url::parse("file:///tmp/test.sh").unwrap();

    let action = to_code_action(&diag, &url);
    assert!(action.is_none(), "Unsafe fixes should NOT produce code actions");
}

#[test]
fn test_pmat214_no_code_action_without_fix() {
    let span = Span::new(1, 1, 1, 5);
    let diag = BashDiag::new("SEM003", Severity::Warning, "Dead code", span);
    let url = Url::parse("file:///tmp/test.sh").unwrap();

    let action = to_code_action(&diag, &url);
    assert!(action.is_none(), "Diagnostics without fixes should not produce code actions");
}

// ---------------------------------------------------------------------------
// ranges_overlap tests
// ---------------------------------------------------------------------------

#[test]
fn test_pmat214_ranges_overlap_same() {
    let r = Range {
        start: Position { line: 1, character: 0 },
        end: Position { line: 1, character: 10 },
    };
    assert!(ranges_overlap(&r, &r));
}

#[test]
fn test_pmat214_ranges_overlap_partial() {
    let a = Range {
        start: Position { line: 1, character: 0 },
        end: Position { line: 1, character: 10 },
    };
    let b = Range {
        start: Position { line: 1, character: 5 },
        end: Position { line: 1, character: 15 },
    };
    assert!(ranges_overlap(&a, &b));
}

#[test]
fn test_pmat214_ranges_no_overlap() {
    let a = Range {
        start: Position { line: 1, character: 0 },
        end: Position { line: 1, character: 5 },
    };
    let b = Range {
        start: Position { line: 3, character: 0 },
        end: Position { line: 3, character: 10 },
    };
    assert!(!ranges_overlap(&a, &b));
}

// ---------------------------------------------------------------------------
// truncate tests
// ---------------------------------------------------------------------------

#[test]
fn test_pmat214_truncate_short() {
    assert_eq!(truncate("hello", 10), "hello");
}

#[test]
fn test_pmat214_truncate_long() {
    let long = "a".repeat(100);
    let result = truncate(&long, 10);
    assert!(result.len() <= 13); // 10 + "..."
    assert!(result.ends_with("..."));
}

// ---------------------------------------------------------------------------
// Hover tests (PMAT-221)
// ---------------------------------------------------------------------------

#[test]
fn test_pmat221_position_in_range_inside() {
    let range = Range {
        start: Position {
            line: 2,
            character: 0,
        },
        end: Position {
            line: 2,
            character: 10,
        },
    };
    let pos = Position {
        line: 2,
        character: 5,
    };
    assert!(position_in_range(&pos, &range));
}

#[test]
fn test_pmat221_position_in_range_start() {
    let range = Range {
        start: Position {
            line: 2,
            character: 0,
        },
        end: Position {
            line: 2,
            character: 10,
        },
    };
    let pos = Position {
        line: 2,
        character: 0,
    };
    assert!(position_in_range(&pos, &range));
}

#[test]
fn test_pmat221_position_in_range_outside_before() {
    let range = Range {
        start: Position {
            line: 2,
            character: 5,
        },
        end: Position {
            line: 2,
            character: 10,
        },
    };
    let pos = Position {
        line: 2,
        character: 3,
    };
    assert!(!position_in_range(&pos, &range));
}

#[test]
fn test_pmat221_position_in_range_outside_after() {
    let range = Range {
        start: Position {
            line: 2,
            character: 0,
        },
        end: Position {
            line: 2,
            character: 10,
        },
    };
    let pos = Position {
        line: 2,
        character: 15,
    };
    assert!(!position_in_range(&pos, &range));
}

#[test]
fn test_pmat221_position_in_range_different_line() {
    let range = Range {
        start: Position {
            line: 2,
            character: 0,
        },
        end: Position {
            line: 2,
            character: 10,
        },
    };
    let pos = Position {
        line: 5,
        character: 5,
    };
    assert!(!position_in_range(&pos, &range));
}

#[test]
fn test_pmat221_position_in_range_multiline() {
    let range = Range {
        start: Position {
            line: 2,
            character: 5,
        },
        end: Position {
            line: 4,
            character: 10,
        },
    };
    // Middle line should be in range
    let pos = Position {
        line: 3,
        character: 0,
    };
    assert!(position_in_range(&pos, &range));
}

#[test]
fn test_pmat221_hover_content_basic() {
    let diag = BashDiag {
        code: "SC2086".to_string(),
        severity: Severity::Warning,
        message: "Double quote to prevent globbing".to_string(),
        span: Span {
            start_line: 3,
            start_col: 1,
            end_line: 3,
            end_col: 10,
        },
        fix: None,
    };
    let content = format_hover_content(&diag);
    assert!(content.contains("SC2086"), "Should contain rule code");
    assert!(
        content.contains("Double quote"),
        "Should contain message"
    );
    assert!(content.contains("Warning"), "Should contain severity");
    assert!(
        content.contains("shellcheck disable=SC2086"),
        "Should contain disable hint"
    );
}

#[test]
fn test_pmat221_hover_content_with_fix() {
    let diag = BashDiag {
        code: "DET001".to_string(),
        severity: Severity::Warning,
        message: "Non-deterministic variable".to_string(),
        span: Span {
            start_line: 1,
            start_col: 1,
            end_line: 1,
            end_col: 10,
        },
        fix: Some(Fix {
            replacement: "FIXED_SEED=42".to_string(),
            safety_level: FixSafetyLevel::Safe,
        }),
    };
    let content = format_hover_content(&diag);
    assert!(content.contains("Fix"), "Should contain fix section");
    assert!(
        content.contains("FIXED_SEED=42"),
        "Should contain replacement"
    );
    assert!(content.contains("Safe"), "Should contain safety level");
}

#[test]
fn test_pmat221_hover_content_with_registry_metadata() {
    // SEC001 should be in the rule registry
    let diag = BashDiag {
        code: "SEC001".to_string(),
        severity: Severity::Error,
        message: "Command injection risk".to_string(),
        span: Span {
            start_line: 1,
            start_col: 1,
            end_line: 1,
            end_col: 10,
        },
        fix: None,
    };
    let content = format_hover_content(&diag);
    assert!(content.contains("Rule"), "Should contain Rule metadata");
    assert!(
        content.contains("Compatibility"),
        "Should contain compatibility info"
    );
}
