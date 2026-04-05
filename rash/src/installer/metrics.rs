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
            .map_or(Duration::ZERO, |start| start.elapsed());

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

include!("metrics_metricsaggregator.rs");
