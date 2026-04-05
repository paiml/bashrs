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
