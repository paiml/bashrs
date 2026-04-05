//! Coverage tests for installer/progress.rs — targets uncovered branches in
//! StepState text/progress, StepInfo, ProgressStyle, ExecutionMode, builder
//! methods, estimated_remaining, format_duration, progress_bar edge cases,
//! TerminalRenderer all states, JsonRenderer all states, InstallationSummary
//! format/to_json, generate_summary, and helper functions.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;
use std::time::{Duration, Instant};

#[test]
fn test_PROGRESS_COV_001_step_state_text_all() {
    assert_eq!(StepState::Pending.text(), "PENDING");
    assert_eq!(
        StepState::Running {
            progress: 0,
            message: String::new(),
            started_at: Instant::now()
        }
        .text(),
        "RUNNING"
    );
    assert_eq!(
        StepState::Completed {
            duration: Duration::ZERO
        }
        .text(),
        "COMPLETE"
    );
    assert_eq!(
        StepState::Failed {
            error: String::new(),
            duration: Duration::ZERO
        }
        .text(),
        "FAILED"
    );
    assert_eq!(
        StepState::Skipped {
            reason: String::new()
        }
        .text(),
        "SKIPPED"
    );
}

#[test]
fn test_PROGRESS_COV_002_step_state_progress_all() {
    assert_eq!(StepState::Pending.progress(), 0);
    assert_eq!(
        StepState::Running {
            progress: 75,
            message: String::new(),
            started_at: Instant::now()
        }
        .progress(),
        75
    );
    assert_eq!(
        StepState::Completed {
            duration: Duration::ZERO
        }
        .progress(),
        100
    );
    assert_eq!(
        StepState::Failed {
            error: String::new(),
            duration: Duration::ZERO
        }
        .progress(),
        0
    );
    assert_eq!(
        StepState::Skipped {
            reason: String::new()
        }
        .progress(),
        0
    );
}

#[test]
fn test_PROGRESS_COV_003_step_info_new() {
    let info = StepInfo::new("pkg-install", "Install Package", 3);
    assert_eq!(info.id, "pkg-install");
    assert_eq!(info.name, "Install Package");
    assert_eq!(info.index, 3);
    assert_eq!(info.state, StepState::Pending);
}

#[test]
fn test_PROGRESS_COV_004_style_and_mode_defaults() {
    assert_eq!(ProgressStyle::default(), ProgressStyle::Standard);
    assert_eq!(ExecutionMode::default(), ExecutionMode::Normal);
    let _min = ProgressStyle::Minimal;
    let _verb = ProgressStyle::Verbose;
    let _quiet = ProgressStyle::Quiet;
}

#[test]
fn test_PROGRESS_COV_005_progress_builder_methods() {
    let p = InstallerProgress::new("app", "1.0")
        .with_style(ProgressStyle::Verbose)
        .with_mode(ExecutionMode::DryRun)
        .with_artifacts(3, 5)
        .with_signatures(true)
        .with_trace(true);
    assert_eq!(p.total_steps(), 0);
    let renderer = TerminalRenderer::new();
    let footer = renderer.render_footer(&p);
    assert!(footer.contains("3/5 verified"));
    assert!(footer.contains("Mode: DRY-RUN"));
    assert!(footer.contains("Trace: recording"));
}

#[test]
fn test_PROGRESS_COV_006_estimated_remaining_branches() {
    // No completed => None
    let mut p = InstallerProgress::new("app", "1.0");
    p.add_step("s1", "S1");
    assert!(p.estimated_remaining().is_none());
    // All done => Some(ZERO)
    let mut p2 = InstallerProgress::new("app", "1.0");
    p2.add_step("s1", "S1");
    p2.start_step("s1", "go");
    p2.complete_step("s1");
    assert_eq!(p2.estimated_remaining(), Some(Duration::ZERO));
    // Partial
    let mut p3 = InstallerProgress::new("app", "1.0");
    p3.add_step("s1", "S1");
    p3.add_step("s2", "S2");
    p3.add_step("s3", "S3");
    p3.start_step("s1", "go");
    p3.complete_step("s1");
    assert!(p3.estimated_remaining().is_some());
    // With skipped
    let mut p4 = InstallerProgress::new("app", "1.0");
    p4.add_step("s1", "S1");
    p4.add_step("s2", "S2");
    p4.add_step("s3", "S3");
    p4.start_step("s1", "go");
    p4.complete_step("s1");
    p4.skip_step("s2", "not needed");
    assert!(p4.estimated_remaining().is_some());
}

#[test]
fn test_PROGRESS_COV_007_format_duration_all_branches() {
    assert_eq!(TerminalRenderer::format_duration(Duration::ZERO), "0ms");
    assert_eq!(
        TerminalRenderer::format_duration(Duration::from_millis(450)),
        "450ms"
    );
    assert_eq!(
        TerminalRenderer::format_duration(Duration::from_millis(5230)),
        "5.23s"
    );
    assert_eq!(
        TerminalRenderer::format_duration(Duration::from_secs(60)),
        "1m 00s"
    );
    assert_eq!(
        TerminalRenderer::format_duration(Duration::from_secs(125)),
        "2m 05s"
    );
}

#[test]
fn test_PROGRESS_COV_008_progress_bar_edge_cases() {
    let r = TerminalRenderer::with_width(80);
    assert_eq!(r.progress_bar(0, 20).chars().count(), 20);
    assert_eq!(r.progress_bar(1, 20).chars().count(), 20);
    assert_eq!(r.progress_bar(99, 20).chars().count(), 20);
    assert_eq!(r.progress_bar(100, 20).chars().count(), 20);
    assert!(r.progress_bar(50, 0).is_empty());
}

#[test]
fn test_PROGRESS_COV_009_render_step_all_states() {
    let r = TerminalRenderer::new();
    // Pending
    let pending = StepInfo::new("setup", "Setup", 1);
    let out_p = r.render_step(&pending, 3);
    assert!(out_p.contains("Setup") && out_p.contains("Pending") && out_p.contains("0%"));
    // Running
    let running = StepInfo {
        id: "install".into(),
        name: "Install".into(),
        index: 2,
        state: StepState::Running {
            progress: 60,
            message: "Downloading".into(),
            started_at: Instant::now(),
        },
    };
    let out_r = r.render_step(&running, 3);
    assert!(out_r.contains("Downloading") && out_r.contains("60%"));
    // Failed
    let failed = StepInfo {
        id: "verify".into(),
        name: "Verify".into(),
        index: 3,
        state: StepState::Failed {
            error: "checksum mismatch on artifact".into(),
            duration: Duration::from_secs(2),
        },
    };
    assert!(r.render_step(&failed, 3).contains("checksum mismatch"));
    // Skipped
    let skipped = StepInfo {
        id: "opt".into(),
        name: "Optional".into(),
        index: 2,
        state: StepState::Skipped {
            reason: "already done".into(),
        },
    };
    assert!(r.render_step(&skipped, 4).contains("already done"));
    // Completed
    let completed = StepInfo {
        id: "fin".into(),
        name: "Final".into(),
        index: 1,
        state: StepState::Completed {
            duration: Duration::from_secs(5),
        },
    };
    let out_c = r.render_step(&completed, 1);
    assert!(out_c.contains("100%"));
}

#[test]
fn test_PROGRESS_COV_010_render_footer_variants() {
    let r = TerminalRenderer::with_width(60);
    // No extras
    let p1 = InstallerProgress::new("app", "1.0");
    let f1 = r.render_footer(&p1);
    assert!(f1.contains("Checkpoint: none") && f1.contains("Mode: NORMAL"));
    assert!(!f1.contains("Artifacts:"));
    // With checkpoint
    let mut p2 = InstallerProgress::new("app", "1.0");
    p2.add_step("s1", "First");
    p2.start_step("s1", "go");
    p2.complete_step("s1");
    assert!(r.render_footer(&p2).contains("Checkpoint: s1"));
    // All modes
    assert!(r
        .render_footer(&InstallerProgress::new("a", "1").with_mode(ExecutionMode::DryRun))
        .contains("DRY-RUN"));
    assert!(r
        .render_footer(&InstallerProgress::new("a", "1").with_mode(ExecutionMode::Hermetic))
        .contains("HERMETIC"));
    assert!(r
        .render_footer(&InstallerProgress::new("a", "1").with_mode(ExecutionMode::Test))
        .contains("TEST"));
    // With all extras
    let p3 = InstallerProgress::new("a", "1")
        .with_artifacts(5, 5)
        .with_signatures(true)
        .with_trace(true);
    let f3 = r.render_footer(&p3);
    assert!(f3.contains("5/5 verified") && f3.contains("Trace: recording"));
    // Sigs not verified
    let p4 = InstallerProgress::new("a", "1")
        .with_artifacts(2, 4)
        .with_signatures(false);
    assert!(r.render_footer(&p4).contains("2/4 verified"));
}

#[test]
fn test_PROGRESS_COV_011_full_terminal_render() {
    let mut p = InstallerProgress::new("my-installer", "2.0.0")
        .with_mode(ExecutionMode::Normal)
        .with_artifacts(1, 2)
        .with_trace(true);
    p.add_step("prepare", "Prepare");
    p.add_step("install", "Install");
    p.start_step("prepare", "Initializing");
    p.complete_step("prepare");
    let out = TerminalRenderer::with_width(60).render(&p);
    assert!(
        out.contains("my-installer v2.0.0") && out.contains("Prepare") && out.contains("Install")
    );
}

#[test]
fn test_PROGRESS_COV_012_json_renderer_all_states() {
    let jr = JsonRenderer::new();
    // Pending with comma (not last)
    let s1 = StepInfo::new("s1", "Step One", 1);
    let j1 = jr.render_step(&s1, 2);
    assert!(j1.contains("\"status\": \"pending\"") && j1.contains("\"progress\": 0"));
    // Running
    let s2 = StepInfo {
        id: "s1".into(),
        name: "Step One".into(),
        index: 1,
        state: StepState::Running {
            progress: 40,
            message: "Working".into(),
            started_at: Instant::now(),
        },
    };
    let j2 = jr.render_step(&s2, 1);
    assert!(
        j2.contains("\"status\": \"running\"")
            && j2.contains("\"duration_ms\"")
            && j2.contains("\"message\": \"Working\"")
    );
    // Failed
    let s3 = StepInfo {
        id: "s2".into(),
        name: "S2".into(),
        index: 2,
        state: StepState::Failed {
            error: "timeout".into(),
            duration: Duration::from_millis(3000),
        },
    };
    let j3 = jr.render_step(&s3, 2);
    assert!(j3.contains("\"status\": \"failed\"") && j3.contains("\"duration_ms\": 3000"));
    // Skipped (no duration)
    let s4 = StepInfo {
        id: "s3".into(),
        name: "S3".into(),
        index: 1,
        state: StepState::Skipped {
            reason: "precondition failed".into(),
        },
    };
    let j4 = jr.render_step(&s4, 1);
    assert!(j4.contains("\"status\": \"skipped\"") && !j4.contains("\"duration_ms\""));
    // Last step has no trailing comma
    let s5 = StepInfo::new("final", "Final", 3);
    assert!(!jr.render_step(&s5, 3).trim_end().ends_with(','));
}

#[test]
fn test_PROGRESS_COV_013_json_full_render() {
    let mut p = InstallerProgress::new("test", "1.0");
    p.add_step("s1", "First");
    p.add_step("s2", "Second");
    p.start_step("s1", "go");
    p.complete_step("s1");
    p.start_step("s2", "go");
    p.fail_step("s2", "bad");
    let out = JsonRenderer::new().render(&p);
    assert!(out.contains("\"steps\"") && out.contains("\"summary\""));
    assert!(out.contains("\"completed\": 1") && out.contains("\"failed\": 1"));
}

#[test]

include!("progress_tests_tests_PROGRESS.rs");
