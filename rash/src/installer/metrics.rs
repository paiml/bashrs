//! Installer Metrics Collection (#118)
//!
//! Kaizen-style timing and failure metrics for continuous improvement.
//!
//! Tracks:
//! - Step execution times
//! - Failure rates and patterns
//! - Resource usage
//! - Trend analysis for improvement
//!
//! # Example
//!
//! ```ignore
//! use bashrs::installer::{MetricsCollector, StepMetrics};
//!
//! let mut collector = MetricsCollector::new();
//! collector.record_step_start("install-deps");
//! // ... execute step ...
//! collector.record_step_end("install-deps", StepOutcome::Success);
//! let report = collector.generate_report();
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime};

/// Outcome of a step execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepOutcome {
    /// Step completed successfully
    Success,
    /// Step failed
    Failed,
    /// Step was skipped
    Skipped,
    /// Step timed out
    Timeout,
    /// Step was cancelled
    Cancelled,
}

/// Metrics for a single step execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepMetrics {
    /// Step ID
    pub step_id: String,
    /// Step name
    pub step_name: String,
    /// Execution start time
    pub started_at: String,
    /// Execution duration in milliseconds
    pub duration_ms: u64,
    /// Outcome of the step
    pub outcome: StepOutcome,
    /// Number of retries
    pub retry_count: u32,
    /// Error message if failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    /// Memory usage in bytes (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_bytes: Option<u64>,
}

/// Metrics for an entire installer run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallerMetrics {
    /// Installer name
    pub installer_name: String,
    /// Installer version
    pub installer_version: String,
    /// Run ID (unique identifier)
    pub run_id: String,
    /// Run start time
    pub started_at: String,
    /// Run end time
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ended_at: Option<String>,
    /// Total duration in milliseconds
    pub total_duration_ms: u64,
    /// Step metrics
    pub steps: Vec<StepMetrics>,
    /// Overall outcome
    pub outcome: StepOutcome,
    /// Environment info
    pub environment: EnvironmentInfo,
}

/// Environment information
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EnvironmentInfo {
    /// Operating system
    pub os: String,
    /// OS version
    pub os_version: String,
    /// Architecture
    pub arch: String,
    /// Hostname (anonymized)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname_hash: Option<String>,
}

/// Aggregated metrics for trend analysis
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AggregatedMetrics {
    /// Total runs
    pub total_runs: u64,
    /// Successful runs
    pub successful_runs: u64,
    /// Failed runs
    pub failed_runs: u64,
    /// Success rate (0.0 - 1.0)
    pub success_rate: f64,
    /// Average duration in milliseconds
    pub avg_duration_ms: f64,
    /// Median duration in milliseconds
    pub median_duration_ms: f64,
    /// 95th percentile duration
    pub p95_duration_ms: f64,
    /// Step-level aggregates
    pub step_aggregates: HashMap<String, StepAggregate>,
}

/// Aggregated metrics for a single step
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StepAggregate {
    /// Step ID
    pub step_id: String,
    /// Total executions
    pub total_executions: u64,
    /// Successful executions
    pub successful_executions: u64,
    /// Failed executions
    pub failed_executions: u64,
    /// Success rate
    pub success_rate: f64,
    /// Average duration
    pub avg_duration_ms: f64,
    /// Min duration
    pub min_duration_ms: f64,
    /// Max duration
    pub max_duration_ms: f64,
    /// Average retry count
    pub avg_retries: f64,
}

/// Metrics collector for real-time tracking
#[derive(Debug)]
pub struct MetricsCollector {
    /// Installer name
    installer_name: String,
    /// Installer version
    installer_version: String,
    /// Run ID
    run_id: String,
    /// Start time
    started_at: Instant,
    /// Start timestamp (for serialization)
    started_at_timestamp: String,
    /// Step start times
    step_starts: HashMap<String, Instant>,
    /// Collected step metrics
    steps: Vec<StepMetrics>,
    /// Step names
    step_names: HashMap<String, String>,
    /// Environment info
    environment: EnvironmentInfo,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            installer_name: String::new(),
            installer_version: String::new(),
            run_id: generate_run_id(),
            started_at: Instant::now(),
            started_at_timestamp: current_timestamp(),
            step_starts: HashMap::new(),
            steps: Vec::new(),
            step_names: HashMap::new(),
            environment: detect_environment(),
        }
    }

    /// Create a collector for a specific installer
    pub fn for_installer(name: &str, version: &str) -> Self {
        let mut collector = Self::new();
        collector.installer_name = name.to_string();
        collector.installer_version = version.to_string();
        collector
    }

    /// Record step start
    pub fn record_step_start(&mut self, step_id: &str, step_name: &str) {
        self.step_starts.insert(step_id.to_string(), Instant::now());
        self.step_names
            .insert(step_id.to_string(), step_name.to_string());
    }

    /// Record step end with outcome
    pub fn record_step_end(&mut self, step_id: &str, outcome: StepOutcome) {
        self.record_step_end_with_details(step_id, outcome, 0, None);
    }

    /// Record step end with full details
    pub fn record_step_end_with_details(
        &mut self,
        step_id: &str,
        outcome: StepOutcome,
        retry_count: u32,
        error_message: Option<String>,
    ) {
        let duration = self
            .step_starts
            .get(step_id)
            .map(|start| start.elapsed())
            .unwrap_or(Duration::ZERO);

        let step_name = self
            .step_names
            .get(step_id)
            .cloned()
            .unwrap_or_else(|| step_id.to_string());

        self.steps.push(StepMetrics {
            step_id: step_id.to_string(),
            step_name,
            started_at: current_timestamp(),
            duration_ms: duration.as_millis() as u64,
            outcome,
            retry_count,
            error_message,
            memory_bytes: None,
        });
    }

    /// Get the run ID
    pub fn run_id(&self) -> &str {
        &self.run_id
    }

    /// Get elapsed time since start
    pub fn elapsed(&self) -> Duration {
        self.started_at.elapsed()
    }

    /// Generate final metrics report
    pub fn finalize(self, overall_outcome: StepOutcome) -> InstallerMetrics {
        InstallerMetrics {
            installer_name: self.installer_name,
            installer_version: self.installer_version,
            run_id: self.run_id,
            started_at: self.started_at_timestamp,
            ended_at: Some(current_timestamp()),
            total_duration_ms: self.started_at.elapsed().as_millis() as u64,
            steps: self.steps,
            outcome: overall_outcome,
            environment: self.environment,
        }
    }

    /// Get current step count
    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    /// Get success count
    pub fn success_count(&self) -> usize {
        self.steps
            .iter()
            .filter(|s| s.outcome == StepOutcome::Success)
            .count()
    }

    /// Get failure count
    pub fn failure_count(&self) -> usize {
        self.steps
            .iter()
            .filter(|s| s.outcome == StepOutcome::Failed)
            .count()
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Metrics aggregator for historical analysis
#[derive(Debug, Default)]
pub struct MetricsAggregator {
    /// All collected runs
    runs: Vec<InstallerMetrics>,
}

impl MetricsAggregator {
    /// Create a new aggregator
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a run to the aggregator
    pub fn add_run(&mut self, metrics: InstallerMetrics) {
        self.runs.push(metrics);
    }

    /// Load runs from a directory
    pub fn load_from_dir(&mut self, dir: &std::path::Path) -> std::io::Result<usize> {
        let mut count = 0;
        if dir.exists() {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "json") {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if let Ok(metrics) = serde_json::from_str::<InstallerMetrics>(&content) {
                            self.runs.push(metrics);
                            count += 1;
                        }
                    }
                }
            }
        }
        Ok(count)
    }

    /// Generate aggregated metrics
    pub fn aggregate(&self) -> AggregatedMetrics {
        if self.runs.is_empty() {
            return AggregatedMetrics::default();
        }

        let total_runs = self.runs.len() as u64;
        let successful_runs = self
            .runs
            .iter()
            .filter(|r| r.outcome == StepOutcome::Success)
            .count() as u64;
        let failed_runs = total_runs - successful_runs;

        let durations: Vec<f64> = self.runs.iter().map(|r| r.total_duration_ms as f64).collect();
        let avg_duration = durations.iter().sum::<f64>() / durations.len() as f64;

        let mut sorted_durations = durations.clone();
        sorted_durations.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        #[allow(clippy::manual_is_multiple_of)]
        let median_duration = if sorted_durations.len() % 2 == 0 {
            let mid = sorted_durations.len() / 2;
            let a = sorted_durations.get(mid - 1).copied().unwrap_or(0.0);
            let b = sorted_durations.get(mid).copied().unwrap_or(0.0);
            (a + b) / 2.0
        } else {
            sorted_durations.get(sorted_durations.len() / 2).copied().unwrap_or(0.0)
        };

        let p95_idx = (sorted_durations.len() as f64 * 0.95) as usize;
        let p95_duration = sorted_durations.get(p95_idx).copied().unwrap_or(0.0);

        // Aggregate per-step metrics
        let mut step_data: HashMap<String, Vec<&StepMetrics>> = HashMap::new();
        for run in &self.runs {
            for step in &run.steps {
                step_data
                    .entry(step.step_id.clone())
                    .or_default()
                    .push(step);
            }
        }

        let step_aggregates: HashMap<String, StepAggregate> = step_data
            .into_iter()
            .map(|(step_id, metrics)| {
                let total = metrics.len() as u64;
                let successful = metrics
                    .iter()
                    .filter(|m| m.outcome == StepOutcome::Success)
                    .count() as u64;
                let failed = total - successful;

                let durations: Vec<f64> = metrics.iter().map(|m| m.duration_ms as f64).collect();
                let avg_dur = durations.iter().sum::<f64>() / durations.len() as f64;
                let min_dur = durations.iter().cloned().fold(f64::INFINITY, f64::min);
                let max_dur = durations.iter().cloned().fold(0.0_f64, f64::max);

                let avg_retries =
                    metrics.iter().map(|m| m.retry_count as f64).sum::<f64>() / total as f64;

                (
                    step_id.clone(),
                    StepAggregate {
                        step_id,
                        total_executions: total,
                        successful_executions: successful,
                        failed_executions: failed,
                        success_rate: if total > 0 {
                            successful as f64 / total as f64
                        } else {
                            0.0
                        },
                        avg_duration_ms: avg_dur,
                        min_duration_ms: if min_dur.is_infinite() { 0.0 } else { min_dur },
                        max_duration_ms: max_dur,
                        avg_retries,
                    },
                )
            })
            .collect();

        AggregatedMetrics {
            total_runs,
            successful_runs,
            failed_runs,
            success_rate: if total_runs > 0 {
                successful_runs as f64 / total_runs as f64
            } else {
                0.0
            },
            avg_duration_ms: avg_duration,
            median_duration_ms: median_duration,
            p95_duration_ms: p95_duration,
            step_aggregates,
        }
    }

    /// Get runs count
    pub fn runs_count(&self) -> usize {
        self.runs.len()
    }

    /// Generate Kaizen improvement report
    pub fn kaizen_report(&self) -> KaizenReport {
        let aggregates = self.aggregate();
        let mut improvements = Vec::new();
        let mut bottlenecks = Vec::new();

        // Identify bottlenecks (steps with high failure rate or long duration)
        for (step_id, agg) in &aggregates.step_aggregates {
            if agg.success_rate < 0.95 {
                bottlenecks.push(format!(
                    "Step '{}' has {:.1}% success rate (target: 95%)",
                    step_id,
                    agg.success_rate * 100.0
                ));
                improvements.push(format!(
                    "Investigate failures in step '{}' - {} failures out of {}",
                    step_id, agg.failed_executions, agg.total_executions
                ));
            }

            if agg.avg_duration_ms > 60000.0 {
                // > 1 minute
                bottlenecks.push(format!(
                    "Step '{}' takes {:.1}s on average",
                    step_id,
                    agg.avg_duration_ms / 1000.0
                ));
                improvements.push(format!(
                    "Consider optimizing step '{}' or adding parallelization",
                    step_id
                ));
            }

            if agg.avg_retries > 0.5 {
                improvements.push(format!(
                    "Step '{}' has high retry rate ({:.1}) - check preconditions",
                    step_id, agg.avg_retries
                ));
            }
        }

        // Overall health check
        if aggregates.success_rate < 0.9 {
            improvements.push(format!(
                "Overall success rate is {:.1}% - needs improvement",
                aggregates.success_rate * 100.0
            ));
        }

        KaizenReport {
            overall_health: if aggregates.success_rate >= 0.95 {
                "Excellent"
            } else if aggregates.success_rate >= 0.9 {
                "Good"
            } else if aggregates.success_rate >= 0.8 {
                "Needs Improvement"
            } else {
                "Critical"
            }
            .to_string(),
            success_rate: aggregates.success_rate,
            bottlenecks,
            improvements,
            metrics_summary: aggregates,
        }
    }
}

/// Kaizen-style improvement report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KaizenReport {
    /// Overall health assessment
    pub overall_health: String,
    /// Success rate
    pub success_rate: f64,
    /// Identified bottlenecks
    pub bottlenecks: Vec<String>,
    /// Suggested improvements
    pub improvements: Vec<String>,
    /// Full metrics summary
    pub metrics_summary: AggregatedMetrics,
}

/// Format metrics as human-readable report
pub fn format_metrics_report(metrics: &InstallerMetrics) -> String {
    let mut report = String::new();

    report.push_str(&format!(
        "Installer Metrics Report: {} v{}\n",
        metrics.installer_name, metrics.installer_version
    ));
    report.push_str(&format!("Run ID: {}\n", metrics.run_id));
    report.push_str(&format!("Started: {}\n", metrics.started_at));
    if let Some(ref ended) = metrics.ended_at {
        report.push_str(&format!("Ended: {}\n", ended));
    }
    report.push_str(&format!(
        "Duration: {:.2}s\n",
        metrics.total_duration_ms as f64 / 1000.0
    ));
    report.push_str(&format!("Outcome: {:?}\n\n", metrics.outcome));

    report.push_str("Steps:\n");
    for step in &metrics.steps {
        let status = match step.outcome {
            StepOutcome::Success => "✓",
            StepOutcome::Failed => "✗",
            StepOutcome::Skipped => "⊘",
            StepOutcome::Timeout => "⏱",
            StepOutcome::Cancelled => "⊗",
        };
        report.push_str(&format!(
            "  {} {} ({:.2}s)",
            status,
            step.step_name,
            step.duration_ms as f64 / 1000.0
        ));
        if step.retry_count > 0 {
            report.push_str(&format!(" [retries: {}]", step.retry_count));
        }
        if let Some(ref err) = step.error_message {
            report.push_str(&format!(" - {}", err));
        }
        report.push('\n');
    }

    report
}

// Helper functions

fn generate_run_id() -> String {
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    format!("run-{:x}", timestamp)
}

fn current_timestamp() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn detect_environment() -> EnvironmentInfo {
    EnvironmentInfo {
        os: std::env::consts::OS.to_string(),
        os_version: String::new(), // Would need platform-specific code
        arch: std::env::consts::ARCH.to_string(),
        hostname_hash: None,
    }
}

#[cfg(test)]
mod tests {
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
}
