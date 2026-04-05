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
