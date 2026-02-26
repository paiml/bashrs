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
fn test_PROGRESS_COV_014_summary_format_success() {
    let s = InstallationSummary {
        name: "my-app".into(),
        version: "3.0.0".into(),
        total_steps: 4,
        completed: 4,
        failed: 0,
        skipped: 0,
        total_duration: Duration::from_secs(12),
        success: true,
        step_results: vec![],
    };
    let t = s.format();
    assert!(t.contains("SUCCESS") && t.contains("4/4") && !t.contains("Failed steps:"));
}

#[test]
fn test_PROGRESS_COV_015_summary_format_failures() {
    let s = InstallationSummary {
        name: "broken".into(),
        version: "0.1.0".into(),
        total_steps: 3,
        completed: 1,
        failed: 2,
        skipped: 0,
        total_duration: Duration::from_secs(5),
        success: false,
        step_results: vec![
            StepResult {
                id: "s1".into(),
                name: "S1".into(),
                status: "passed".into(),
                duration: Some(Duration::from_secs(1)),
                message: None,
            },
            StepResult {
                id: "s2".into(),
                name: "S2".into(),
                status: "failed".into(),
                duration: Some(Duration::from_secs(2)),
                message: Some("network error".into()),
            },
            StepResult {
                id: "s3".into(),
                name: "S3".into(),
                status: "failed".into(),
                duration: Some(Duration::from_secs(1)),
                message: None,
            },
        ],
    };
    let t = s.format();
    assert!(
        t.contains("FAILED") && t.contains("s2: network error") && t.contains("s3: unknown error")
    );
}

#[test]
fn test_PROGRESS_COV_016_summary_format_minutes() {
    let s = InstallationSummary {
        name: "long".into(),
        version: "1.0.0".into(),
        total_steps: 1,
        completed: 1,
        failed: 0,
        skipped: 0,
        total_duration: Duration::from_secs(90),
        success: true,
        step_results: vec![],
    };
    assert!(s.format().contains("1m 30s"));
}

#[test]
fn test_PROGRESS_COV_017_summary_to_json_with_steps() {
    let s = InstallationSummary {
        name: "app".into(),
        version: "2.0.0".into(),
        total_steps: 2,
        completed: 1,
        failed: 1,
        skipped: 0,
        total_duration: Duration::from_millis(5000),
        success: false,
        step_results: vec![
            StepResult {
                id: "s1".into(),
                name: "S1".into(),
                status: "passed".into(),
                duration: Some(Duration::from_secs(2)),
                message: None,
            },
            StepResult {
                id: "s2".into(),
                name: "S2".into(),
                status: "failed".into(),
                duration: Some(Duration::from_secs(3)),
                message: Some("error".into()),
            },
        ],
    };
    let j = s.to_json();
    assert!(
        j.contains("\"success\": false")
            && j.contains("\"id\": \"s1\"")
            && j.contains("\"id\": \"s2\"")
    );
}

#[test]
fn test_PROGRESS_COV_018_generate_summary_with_failures_and_pending() {
    let mut p = InstallerProgress::new("test", "1.0");
    p.add_step("s1", "Pass");
    p.add_step("s2", "Fail");
    p.add_step("s3", "Pending");
    p.start_step("s1", "go");
    p.complete_step("s1");
    p.start_step("s2", "go");
    p.fail_step("s2", "kaboom");
    let summary = generate_summary(&p);
    assert_eq!(summary.total_steps, 3);
    assert_eq!(summary.completed, 1);
    assert_eq!(summary.failed, 1);
    assert!(!summary.success);
    assert_eq!(summary.step_results[1].message.as_deref(), Some("kaboom"));
    assert_eq!(summary.step_results[2].status, "pending");
    assert!(summary.step_results[2].duration.is_none());
}

#[test]
fn test_PROGRESS_COV_019_truncate_and_escape_json() {
    assert_eq!(truncate("", 0), "");
    assert_eq!(truncate("a", 1), "a");
    assert_eq!(truncate("ab", 1), "...");
    assert_eq!(truncate("abcde", 5), "abcde");
    assert_eq!(truncate("abcdef", 5), "ab...");
    assert_eq!(escape_json("a\\b"), "a\\\\b");
    assert_eq!(escape_json("a\"b"), "a\\\"b");
    assert_eq!(escape_json("a\nb"), "a\\nb");
    assert_eq!(escape_json("a\rb"), "a\\rb");
    assert_eq!(escape_json("a\tb"), "a\\tb");
}

#[test]
fn test_PROGRESS_COV_020_state_transitions_edge_cases() {
    // Complete from pending (no Running state) => duration ZERO
    let mut p = InstallerProgress::new("test", "1.0");
    p.add_step("s1", "S1");
    p.complete_step("s1");
    if let StepState::Completed { duration } = &p.get_step("s1").unwrap().state {
        assert_eq!(*duration, Duration::ZERO);
    } else {
        panic!("Expected Completed");
    }
    // Fail from pending
    let mut p2 = InstallerProgress::new("test", "1.0");
    p2.add_step("s1", "S1");
    p2.fail_step("s1", "err");
    assert!(matches!(
        p2.get_step("s1").unwrap().state,
        StepState::Failed { .. }
    ));
    // Update while pending = no-op
    let mut p3 = InstallerProgress::new("test", "1.0");
    p3.add_step("s1", "S1");
    p3.update_step("s1", 50, "msg");
    assert_eq!(p3.get_step("s1").unwrap().state, StepState::Pending);
    // Operations on nonexistent step = safe no-op
    let mut p4 = InstallerProgress::new("test", "1.0");
    p4.start_step("ghost", "go");
    p4.update_step("ghost", 50, "m");
    p4.complete_step("ghost");
    p4.fail_step("ghost", "e");
    p4.skip_step("ghost", "r");
    assert!(p4.get_step("ghost").is_none());
    // Progress clamp
    let mut p5 = InstallerProgress::new("test", "1.0");
    p5.add_step("s1", "S1");
    p5.start_step("s1", "go");
    p5.update_step("s1", 200, "overflow");
    assert_eq!(p5.get_step("s1").unwrap().state.progress(), 100);
}

#[test]
fn test_PROGRESS_COV_021_json_render_special_chars() {
    let step = StepInfo {
        id: "step \"quoted\"".into(),
        name: "Step\nNewline".into(),
        index: 1,
        state: StepState::Completed {
            duration: Duration::from_millis(100),
        },
    };
    let out = JsonRenderer::new().render_step(&step, 1);
    assert!(out.contains("step \\\"quoted\\\"") && out.contains("Step\\nNewline"));
}
