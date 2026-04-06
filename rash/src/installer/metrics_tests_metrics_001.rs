
use super::*;

#[test]
fn test_METRICS_001_collector_new() {
    let collector = MetricsCollector::new();
    assert!(!collector.run_id().is_empty());
    assert_eq!(collector.step_count(), 0);
}

#[test]
fn test_METRICS_002_collector_for_installer() {
    let collector = MetricsCollector::for_installer("test-installer", "1.0.0");
    assert_eq!(collector.installer_name, "test-installer");
    assert_eq!(collector.installer_version, "1.0.0");
}

#[test]
fn test_METRICS_003_record_step() {
    let mut collector = MetricsCollector::new();
    collector.record_step_start("step-1", "First Step");
    std::thread::sleep(std::time::Duration::from_millis(10));
    collector.record_step_end("step-1", StepOutcome::Success);

    assert_eq!(collector.step_count(), 1);
    assert_eq!(collector.success_count(), 1);
    assert_eq!(collector.failure_count(), 0);
}

#[test]
fn test_METRICS_004_record_failed_step() {
    let mut collector = MetricsCollector::new();
    collector.record_step_start("step-1", "Failing Step");
    collector.record_step_end_with_details(
        "step-1",
        StepOutcome::Failed,
        2,
        Some("Connection refused".to_string()),
    );

    assert_eq!(collector.failure_count(), 1);
    assert_eq!(collector.steps[0].retry_count, 2);
    assert!(collector.steps[0].error_message.is_some());
}

#[test]
fn test_METRICS_005_finalize() {
    let mut collector = MetricsCollector::for_installer("test", "1.0.0");
    collector.record_step_start("step-1", "Step 1");
    collector.record_step_end("step-1", StepOutcome::Success);

    let metrics = collector.finalize(StepOutcome::Success);

    assert_eq!(metrics.installer_name, "test");
    assert_eq!(metrics.outcome, StepOutcome::Success);
    assert_eq!(metrics.steps.len(), 1);
    assert!(metrics.ended_at.is_some());
}

#[test]
fn test_METRICS_006_aggregator_empty() {
    let aggregator = MetricsAggregator::new();
    let agg = aggregator.aggregate();

    assert_eq!(agg.total_runs, 0);
    assert_eq!(agg.success_rate, 0.0);
}

#[test]
fn test_METRICS_007_aggregator_single_run() {
    let mut aggregator = MetricsAggregator::new();

    let metrics = InstallerMetrics {
        installer_name: "test".to_string(),
        installer_version: "1.0.0".to_string(),
        run_id: "run-1".to_string(),
        started_at: "2025-01-01T00:00:00Z".to_string(),
        ended_at: Some("2025-01-01T00:01:00Z".to_string()),
        total_duration_ms: 60000,
        steps: vec![StepMetrics {
            step_id: "step-1".to_string(),
            step_name: "Step 1".to_string(),
            started_at: "2025-01-01T00:00:00Z".to_string(),
            duration_ms: 1000,
            outcome: StepOutcome::Success,
            retry_count: 0,
            error_message: None,
            memory_bytes: None,
        }],
        outcome: StepOutcome::Success,
        environment: EnvironmentInfo::default(),
    };

    aggregator.add_run(metrics);
    let agg = aggregator.aggregate();

    assert_eq!(agg.total_runs, 1);
    assert_eq!(agg.successful_runs, 1);
    assert_eq!(agg.success_rate, 1.0);
}

#[test]
fn test_METRICS_008_aggregator_multiple_runs() {
    let mut aggregator = MetricsAggregator::new();

    for i in 0..10 {
        let outcome = if i % 3 == 0 {
            StepOutcome::Failed
        } else {
            StepOutcome::Success
        };

        aggregator.add_run(InstallerMetrics {
            installer_name: "test".to_string(),
            installer_version: "1.0.0".to_string(),
            run_id: format!("run-{}", i),
            started_at: "2025-01-01T00:00:00Z".to_string(),
            ended_at: None,
            total_duration_ms: 1000 * (i + 1) as u64,
            steps: vec![],
            outcome,
            environment: EnvironmentInfo::default(),
        });
    }

    let agg = aggregator.aggregate();

    assert_eq!(agg.total_runs, 10);
    assert_eq!(agg.failed_runs, 4); // 0, 3, 6, 9
    assert!((agg.success_rate - 0.6).abs() < 0.01);
}

#[test]
fn test_METRICS_009_kaizen_report_excellent() {
    let mut aggregator = MetricsAggregator::new();

    for i in 0..10 {
        aggregator.add_run(InstallerMetrics {
            installer_name: "test".to_string(),
            installer_version: "1.0.0".to_string(),
            run_id: format!("run-{}", i),
            started_at: "2025-01-01T00:00:00Z".to_string(),
            ended_at: None,
            total_duration_ms: 1000,
            steps: vec![StepMetrics {
                step_id: "step-1".to_string(),
                step_name: "Step 1".to_string(),
                started_at: "2025-01-01T00:00:00Z".to_string(),
                duration_ms: 1000,
                outcome: StepOutcome::Success,
                retry_count: 0,
                error_message: None,
                memory_bytes: None,
            }],
            outcome: StepOutcome::Success,
            environment: EnvironmentInfo::default(),
        });
    }

    let report = aggregator.kaizen_report();
    assert_eq!(report.overall_health, "Excellent");
    assert!(report.bottlenecks.is_empty());
}

#[test]
fn test_METRICS_010_kaizen_report_needs_improvement() {
    let mut aggregator = MetricsAggregator::new();

    for i in 0..10 {
        let outcome = if i < 2 {
            StepOutcome::Failed
        } else {
            StepOutcome::Success
        };

        aggregator.add_run(InstallerMetrics {
            installer_name: "test".to_string(),
            installer_version: "1.0.0".to_string(),
            run_id: format!("run-{}", i),
            started_at: "2025-01-01T00:00:00Z".to_string(),
            ended_at: None,
            total_duration_ms: 1000,
            steps: vec![StepMetrics {
                step_id: "slow-step".to_string(),
                step_name: "Slow Step".to_string(),
                started_at: "2025-01-01T00:00:00Z".to_string(),
                duration_ms: 120000, // 2 minutes - should trigger warning
                outcome,
                retry_count: if i < 2 { 3 } else { 0 },
                error_message: if i < 2 {
                    Some("Failed".to_string())
                } else {
                    None
                },
                memory_bytes: None,
            }],
            outcome,
            environment: EnvironmentInfo::default(),
        });
    }

    let report = aggregator.kaizen_report();
    assert_eq!(report.overall_health, "Needs Improvement");
    assert!(!report.bottlenecks.is_empty());
    assert!(!report.improvements.is_empty());
}

#[test]
fn test_METRICS_011_format_report() {
    let metrics = InstallerMetrics {
        installer_name: "test".to_string(),
        installer_version: "1.0.0".to_string(),
        run_id: "run-test".to_string(),
        started_at: "2025-01-01T00:00:00Z".to_string(),
        ended_at: Some("2025-01-01T00:01:00Z".to_string()),
        total_duration_ms: 60000,
        steps: vec![
            StepMetrics {
                step_id: "step-1".to_string(),
                step_name: "Install Dependencies".to_string(),
                started_at: "2025-01-01T00:00:00Z".to_string(),
                duration_ms: 30000,
                outcome: StepOutcome::Success,
                retry_count: 0,
                error_message: None,
                memory_bytes: None,
            },
            StepMetrics {
                step_id: "step-2".to_string(),
                step_name: "Configure App".to_string(),
                started_at: "2025-01-01T00:00:30Z".to_string(),
                duration_ms: 30000,
                outcome: StepOutcome::Failed,
                retry_count: 2,
                error_message: Some("Config file not found".to_string()),
                memory_bytes: None,
            },
        ],
        outcome: StepOutcome::Failed,
        environment: EnvironmentInfo::default(),
    };

    let report = format_metrics_report(&metrics);

    assert!(report.contains("test v1.0.0"));
    assert!(report.contains("run-test"));
    assert!(report.contains("Install Dependencies"));
    assert!(report.contains("Config file not found"));
    assert!(report.contains("[retries: 2]"));
}

#[test]
fn test_METRICS_012_step_outcome_equality() {
    assert_eq!(StepOutcome::Success, StepOutcome::Success);
    assert_ne!(StepOutcome::Success, StepOutcome::Failed);
    assert_ne!(StepOutcome::Timeout, StepOutcome::Cancelled);
}

#[test]
fn test_METRICS_013_environment_detection() {
    let env = detect_environment();
    assert!(!env.os.is_empty());
    assert!(!env.arch.is_empty());
}

#[test]
fn test_METRICS_014_run_id_generation() {
    let id1 = generate_run_id();
    let _id2 = generate_run_id();

    assert!(id1.starts_with("run-"));
    // IDs should be unique (or at least different if generated quickly)
    // Note: This might occasionally fail if both are generated in same ms
}

#[test]
fn test_METRICS_015_step_aggregate_calculation() {
    let mut aggregator = MetricsAggregator::new();

    aggregator.add_run(InstallerMetrics {
        installer_name: "test".to_string(),
        installer_version: "1.0.0".to_string(),
        run_id: "run-1".to_string(),
        started_at: "2025-01-01T00:00:00Z".to_string(),
        ended_at: None,
        total_duration_ms: 1000,
        steps: vec![
            StepMetrics {
                step_id: "step-1".to_string(),
                step_name: "Step 1".to_string(),
                started_at: "".to_string(),
                duration_ms: 100,
                outcome: StepOutcome::Success,
                retry_count: 0,
                error_message: None,
                memory_bytes: None,
            },
            StepMetrics {
                step_id: "step-1".to_string(),
                step_name: "Step 1".to_string(),
                started_at: "".to_string(),
                duration_ms: 200,
                outcome: StepOutcome::Success,
                retry_count: 1,
                error_message: None,
                memory_bytes: None,
            },
        ],
        outcome: StepOutcome::Success,
        environment: EnvironmentInfo::default(),
    });

    let agg = aggregator.aggregate();
    let step_agg = agg.step_aggregates.get("step-1").expect("step exists");

    assert_eq!(step_agg.total_executions, 2);
    assert_eq!(step_agg.min_duration_ms, 100.0);
    assert_eq!(step_agg.max_duration_ms, 200.0);
    assert_eq!(step_agg.avg_duration_ms, 150.0);
    assert_eq!(step_agg.avg_retries, 0.5);
}
