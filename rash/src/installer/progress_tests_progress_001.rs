
use super::*;

#[test]
fn test_PROGRESS_001_step_state_symbols() {
    assert_eq!(StepState::Pending.symbol(), "⏳");
    assert_eq!(
        StepState::Running {
            progress: 50,
            message: String::new(),
            started_at: Instant::now()
        }
        .symbol(),
        "▶"
    );
    assert_eq!(
        StepState::Completed {
            duration: Duration::ZERO
        }
        .symbol(),
        "✓"
    );
    assert_eq!(
        StepState::Failed {
            error: String::new(),
            duration: Duration::ZERO
        }
        .symbol(),
        "✗"
    );
    assert_eq!(
        StepState::Skipped {
            reason: String::new()
        }
        .symbol(),
        "⊘"
    );
}

#[test]
fn test_PROGRESS_002_step_state_terminal() {
    assert!(!StepState::Pending.is_terminal());
    assert!(!StepState::Running {
        progress: 50,
        message: String::new(),
        started_at: Instant::now()
    }
    .is_terminal());
    assert!(StepState::Completed {
        duration: Duration::ZERO
    }
    .is_terminal());
    assert!(StepState::Failed {
        error: String::new(),
        duration: Duration::ZERO
    }
    .is_terminal());
    assert!(StepState::Skipped {
        reason: String::new()
    }
    .is_terminal());
}

#[test]
fn test_PROGRESS_003_progress_tracker_add_step() {
    let mut progress = InstallerProgress::new("test", "1.0.0");
    progress.add_step("step-1", "First Step");
    progress.add_step("step-2", "Second Step");

    assert_eq!(progress.total_steps(), 2);
    assert!(progress.get_step("step-1").is_some());
    assert!(progress.get_step("step-2").is_some());
    assert!(progress.get_step("step-3").is_none());
}

#[test]
fn test_PROGRESS_004_progress_tracker_start_step() {
    let mut progress = InstallerProgress::new("test", "1.0.0");
    progress.add_step("step-1", "First Step");
    progress.start_step("step-1", "Starting...");

    let step = progress.get_step("step-1").unwrap();
    assert!(matches!(step.state, StepState::Running { .. }));
}

#[test]
fn test_PROGRESS_005_progress_tracker_update_step() {
    let mut progress = InstallerProgress::new("test", "1.0.0");
    progress.add_step("step-1", "First Step");
    progress.start_step("step-1", "Starting...");
    progress.update_step("step-1", 50, "Halfway");

    let step = progress.get_step("step-1").unwrap();
    if let StepState::Running {
        progress, message, ..
    } = &step.state
    {
        assert_eq!(*progress, 50);
        assert_eq!(message, "Halfway");
    } else {
        panic!("Expected Running state");
    }
}

#[test]
fn test_PROGRESS_006_progress_tracker_complete_step() {
    let mut progress = InstallerProgress::new("test", "1.0.0");
    progress.add_step("step-1", "First Step");
    progress.start_step("step-1", "Starting...");
    progress.complete_step("step-1");

    let step = progress.get_step("step-1").unwrap();
    assert!(matches!(step.state, StepState::Completed { .. }));
    assert_eq!(progress.completed_count(), 1);
}

#[test]
fn test_PROGRESS_007_progress_tracker_fail_step() {
    let mut progress = InstallerProgress::new("test", "1.0.0");
    progress.add_step("step-1", "First Step");
    progress.start_step("step-1", "Starting...");
    progress.fail_step("step-1", "Something went wrong");

    let step = progress.get_step("step-1").unwrap();
    if let StepState::Failed { error, .. } = &step.state {
        assert_eq!(error, "Something went wrong");
    } else {
        panic!("Expected Failed state");
    }
    assert_eq!(progress.failed_count(), 1);
}

#[test]
fn test_PROGRESS_008_progress_tracker_skip_step() {
    let mut progress = InstallerProgress::new("test", "1.0.0");
    progress.add_step("step-1", "First Step");
    progress.skip_step("step-1", "Not needed");

    let step = progress.get_step("step-1").unwrap();
    if let StepState::Skipped { reason } = &step.state {
        assert_eq!(reason, "Not needed");
    } else {
        panic!("Expected Skipped state");
    }
    assert_eq!(progress.skipped_count(), 1);
}

#[test]
fn test_PROGRESS_009_progress_tracker_is_complete() {
    let mut progress = InstallerProgress::new("test", "1.0.0");
    progress.add_step("step-1", "First Step");
    progress.add_step("step-2", "Second Step");

    assert!(!progress.is_complete());

    progress.start_step("step-1", "Starting...");
    progress.complete_step("step-1");
    assert!(!progress.is_complete());

    progress.skip_step("step-2", "Skipped");
    assert!(progress.is_complete());
}

#[test]
fn test_PROGRESS_010_progress_tracker_has_failures() {
    let mut progress = InstallerProgress::new("test", "1.0.0");
    progress.add_step("step-1", "First Step");

    assert!(!progress.has_failures());

    progress.start_step("step-1", "Starting...");
    progress.fail_step("step-1", "Error");

    assert!(progress.has_failures());
}

#[test]
fn test_PROGRESS_011_execution_mode_labels() {
    assert_eq!(ExecutionMode::Normal.label(), "NORMAL");
    assert_eq!(ExecutionMode::DryRun.label(), "DRY-RUN");
    assert_eq!(ExecutionMode::Hermetic.label(), "HERMETIC");
    assert_eq!(ExecutionMode::Test.label(), "TEST");
}

#[test]
fn test_PROGRESS_012_terminal_renderer_header() {
    let progress = InstallerProgress::new("docker-ce", "1.0.0");
    let renderer = TerminalRenderer::with_width(40);
    let header = renderer.render_header(&progress);

    assert!(header.contains("docker-ce"));
    assert!(header.contains("v1.0.0"));
}

#[test]
fn test_PROGRESS_013_terminal_renderer_step() {
    let step = StepInfo {
        id: "install".to_string(),
        name: "Install Package".to_string(),
        state: StepState::Completed {
            duration: Duration::from_secs(5),
        },
        index: 1,
    };

    let renderer = TerminalRenderer::new();
    let output = renderer.render_step(&step, 3);

    assert!(output.contains("Install Package"));
    assert!(output.contains("✓"));
    assert!(output.contains("100%"));
}

#[test]
fn test_PROGRESS_014_json_renderer() {
    let mut progress = InstallerProgress::new("test", "1.0.0");
    progress.add_step("step-1", "First Step");
    progress.start_step("step-1", "Running");
    progress.complete_step("step-1");

    let renderer = JsonRenderer::new();
    let output = renderer.render(&progress);

    assert!(output.contains("\"steps\""));
    assert!(output.contains("\"id\": \"step-1\""));
    assert!(output.contains("\"status\": \"completed\""));
}

#[test]
fn test_PROGRESS_015_generate_summary() {
    let mut progress = InstallerProgress::new("test", "1.0.0");
    progress.add_step("step-1", "First");
    progress.add_step("step-2", "Second");
    progress.start_step("step-1", "Running");
    progress.complete_step("step-1");
    progress.skip_step("step-2", "Not needed");

    let summary = generate_summary(&progress);

    assert_eq!(summary.total_steps, 2);
    assert_eq!(summary.completed, 1);
    assert_eq!(summary.skipped, 1);
    assert!(summary.success);
}

#[test]
fn test_PROGRESS_016_summary_format() {
    let summary = InstallationSummary {
        name: "test".to_string(),
        version: "1.0.0".to_string(),
        total_steps: 3,
        completed: 2,
        failed: 1,
        skipped: 0,
        total_duration: Duration::from_secs(65),
        success: false,
        step_results: vec![StepResult {
            id: "failed-step".to_string(),
            name: "Failed Step".to_string(),
            status: "failed".to_string(),
            duration: Some(Duration::from_secs(5)),
            message: Some("Error occurred".to_string()),
        }],
    };

    let output = summary.format();
    assert!(output.contains("FAILED"));
    assert!(output.contains("1m 05s"));
    assert!(output.contains("failed-step"));
}

#[test]
fn test_PROGRESS_017_summary_to_json() {
    let summary = InstallationSummary {
        name: "test".to_string(),
        version: "1.0.0".to_string(),
        total_steps: 2,
        completed: 2,
        failed: 0,
        skipped: 0,
        total_duration: Duration::from_secs(10),
        success: true,
        step_results: vec![],
    };

    let json = summary.to_json();
    assert!(json.contains("\"success\": true"));
    assert!(json.contains("\"completed\": 2"));
}

#[test]
fn test_PROGRESS_018_truncate() {
    assert_eq!(truncate("short", 10), "short");
    assert_eq!(truncate("this is a long string", 10), "this is...");
}

#[test]
fn test_PROGRESS_019_escape_json() {
    assert_eq!(escape_json("hello"), "hello");
    assert_eq!(escape_json("line1\nline2"), "line1\\nline2");
    assert_eq!(escape_json("say \"hi\""), "say \\\"hi\\\"");
}

#[test]
fn test_PROGRESS_020_progress_bar() {
    let renderer = TerminalRenderer::with_width(80);

    let bar_0 = renderer.progress_bar(0, 10);
    assert_eq!(bar_0.chars().count(), 10);

    let bar_50 = renderer.progress_bar(50, 10);
    assert_eq!(bar_50.chars().count(), 10);

    let bar_100 = renderer.progress_bar(100, 10);
    assert_eq!(bar_100.chars().count(), 10);
}
