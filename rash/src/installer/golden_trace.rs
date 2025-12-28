//! Golden Trace Regression Detection (§6)
//!
//! This module provides golden trace capture and comparison for installer regression detection.
//! It is designed for future integration with [renacer](https://github.com/paiml/renacer)
//! for syscall-level tracing.
//!
//! # Example
//!
//! ```ignore
//! use bashrs::installer::{GoldenTraceManager, GoldenTraceConfig};
//!
//! let manager = GoldenTraceManager::new(".golden-traces");
//! manager.capture("install-v1", &installer)?;
//! let comparison = manager.compare("install-v1", &installer)?;
//! ```

use crate::models::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Golden trace configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoldenTraceConfig {
    /// Enable golden trace capture/comparison
    #[serde(default)]
    pub enabled: bool,

    /// Directory for storing golden traces
    #[serde(default = "default_trace_dir")]
    pub trace_dir: String,

    /// Syscall categories to capture
    #[serde(default = "default_capture_categories")]
    pub capture: Vec<String>,

    /// Paths to ignore (noise reduction)
    #[serde(default = "default_ignore_paths")]
    pub ignore_paths: Vec<String>,
}

fn default_trace_dir() -> String {
    ".golden-traces".to_string()
}

fn default_capture_categories() -> Vec<String> {
    vec![
        "file".to_string(),
        "network".to_string(),
        "process".to_string(),
        "permission".to_string(),
    ]
}

fn default_ignore_paths() -> Vec<String> {
    vec![
        "/proc/*".to_string(),
        "/sys/*".to_string(),
        "/dev/null".to_string(),
        "/tmp/bashrs-*".to_string(),
    ]
}

impl Default for GoldenTraceConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            trace_dir: default_trace_dir(),
            capture: default_capture_categories(),
            ignore_paths: default_ignore_paths(),
        }
    }
}

/// Event type in a trace
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TraceEventType {
    /// File operation (open, read, write, close, unlink)
    File {
        operation: String,
        path: String,
        flags: Option<String>,
    },
    /// Network operation (connect, bind, listen)
    Network {
        operation: String,
        address: Option<String>,
        port: Option<u16>,
    },
    /// Process operation (fork, exec, wait)
    Process {
        operation: String,
        command: Option<String>,
        args: Option<Vec<String>>,
    },
    /// Permission change (chmod, chown)
    Permission {
        operation: String,
        path: String,
        mode: Option<u32>,
    },
}

/// A trace event with timestamp and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEvent {
    /// Event sequence number
    pub sequence: u64,
    /// Timestamp (nanoseconds since trace start)
    pub timestamp_ns: u64,
    /// Event type and data
    pub event_type: TraceEventType,
    /// Step ID that triggered this event
    pub step_id: Option<String>,
    /// Return value or status
    pub result: TraceResult,
}

/// Result of a trace event
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TraceResult {
    /// Operation succeeded
    Success,
    /// Operation failed with error code
    Error(i32),
    /// Operation result unknown
    Unknown,
}

impl TraceEvent {
    /// Create a summary string for display
    pub fn summary(&self) -> String {
        match &self.event_type {
            TraceEventType::File {
                operation, path, ..
            } => {
                format!("{}(\"{}\")", operation, path)
            }
            TraceEventType::Network {
                operation,
                address,
                port,
            } => {
                if let (Some(addr), Some(p)) = (address, port) {
                    format!("{}({}:{})", operation, addr, p)
                } else {
                    operation.clone()
                }
            }
            TraceEventType::Process {
                operation, command, ..
            } => {
                if let Some(cmd) = command {
                    format!("{}(\"{}\")", operation, cmd)
                } else {
                    operation.clone()
                }
            }
            TraceEventType::Permission {
                operation,
                path,
                mode,
            } => {
                if let Some(m) = mode {
                    format!("{}(\"{}\", {:o})", operation, path, m)
                } else {
                    format!("{}(\"{}\")", operation, path)
                }
            }
        }
    }

    /// Get the category of this event
    pub fn category(&self) -> &'static str {
        match &self.event_type {
            TraceEventType::File { .. } => "file",
            TraceEventType::Network { .. } => "network",
            TraceEventType::Process { .. } => "process",
            TraceEventType::Permission { .. } => "permission",
        }
    }
}

/// A captured golden trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoldenTrace {
    /// Trace name
    pub name: String,
    /// When the trace was captured
    pub captured_at: String,
    /// Installer version at capture time
    pub installer_version: String,
    /// Trace events
    pub events: Vec<TraceEvent>,
    /// Hash of the final result state
    pub result_hash: String,
    /// Number of steps executed
    pub steps_executed: usize,
    /// Total duration in milliseconds
    pub duration_ms: u64,
}

impl GoldenTrace {
    /// Save trace to file
    pub fn save(&self, path: &Path) -> Result<()> {
        let content = serde_json::to_string_pretty(self).map_err(|e| {
            Error::Validation(format!("Failed to serialize golden trace: {}", e))
        })?;
        std::fs::write(path, content).map_err(|e| {
            Error::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to write golden trace: {}", e),
            ))
        })?;
        Ok(())
    }

    /// Load trace from file
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            Error::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to read golden trace: {}", e),
            ))
        })?;
        serde_json::from_str(&content).map_err(|e| {
            Error::Validation(format!("Failed to parse golden trace: {}", e))
        })
    }
}

/// Comparison result between golden trace and current execution
#[derive(Debug, Clone, Default)]
pub struct TraceComparison {
    /// Events in current that weren't in golden
    pub added: Vec<TraceEvent>,
    /// Events in golden that aren't in current
    pub removed: Vec<TraceEvent>,
    /// Events that changed between golden and current
    pub changed: Vec<(TraceEvent, TraceEvent)>,
    /// Trace comparison metadata
    pub metadata: ComparisonMetadata,
}

/// Metadata about the comparison
#[derive(Debug, Clone, Default)]
pub struct ComparisonMetadata {
    /// Golden trace name
    pub golden_name: String,
    /// Golden trace timestamp
    pub golden_captured_at: String,
    /// Current execution timestamp
    pub current_captured_at: String,
    /// Number of events in golden
    pub golden_event_count: usize,
    /// Number of events in current
    pub current_event_count: usize,
}

impl TraceComparison {
    /// Check if traces are equivalent
    pub fn is_equivalent(&self) -> bool {
        self.added.is_empty() && self.removed.is_empty() && self.changed.is_empty()
    }

    /// Generate a human-readable report
    pub fn to_report(&self) -> String {
        let mut report = String::new();

        report.push_str(&format!(
            "Golden Trace Comparison: {}\n",
            self.metadata.golden_name
        ));
        report.push_str(&format!(
            "Golden captured: {}, Current: {}\n",
            self.metadata.golden_captured_at, self.metadata.current_captured_at
        ));
        report.push_str(&format!(
            "Events: {} (golden) vs {} (current)\n\n",
            self.metadata.golden_event_count, self.metadata.current_event_count
        ));

        if self.is_equivalent() {
            report.push_str("✅ Traces are EQUIVALENT - no regression detected\n");
            return report;
        }

        if !self.added.is_empty() {
            report.push_str("=== New events (potential security concern) ===\n");
            for event in &self.added {
                report.push_str(&format!(
                    "+ [{}] {} ({:?})\n",
                    event.category(),
                    event.summary(),
                    event.result
                ));
            }
            report.push('\n');
        }

        if !self.removed.is_empty() {
            report.push_str("=== Missing events (potential regression) ===\n");
            for event in &self.removed {
                report.push_str(&format!(
                    "- [{}] {} ({:?})\n",
                    event.category(),
                    event.summary(),
                    event.result
                ));
            }
            report.push('\n');
        }

        if !self.changed.is_empty() {
            report.push_str("=== Changed events ===\n");
            for (old, new) in &self.changed {
                report.push_str(&format!(
                    "~ [{}] {} -> {}\n",
                    old.category(),
                    old.summary(),
                    new.summary()
                ));
            }
        }

        report
    }

    /// Compare two traces
    pub fn compare(golden: &GoldenTrace, current: &GoldenTrace) -> Self {
        let mut comparison = TraceComparison {
            metadata: ComparisonMetadata {
                golden_name: golden.name.clone(),
                golden_captured_at: golden.captured_at.clone(),
                current_captured_at: current.captured_at.clone(),
                golden_event_count: golden.events.len(),
                current_event_count: current.events.len(),
            },
            ..Default::default()
        };

        // Build sets of event signatures for quick lookup
        let golden_sigs: HashSet<String> =
            golden.events.iter().map(|e| e.summary()).collect();
        let current_sigs: HashSet<String> =
            current.events.iter().map(|e| e.summary()).collect();

        // Find added events (in current but not in golden)
        for event in &current.events {
            if !golden_sigs.contains(&event.summary()) {
                comparison.added.push(event.clone());
            }
        }

        // Find removed events (in golden but not in current)
        for event in &golden.events {
            if !current_sigs.contains(&event.summary()) {
                comparison.removed.push(event.clone());
            }
        }

        comparison
    }
}

/// Manager for golden trace capture and comparison
#[derive(Debug, Clone)]
pub struct GoldenTraceManager {
    /// Directory for storing traces
    trace_dir: PathBuf,
    /// Configuration
    config: GoldenTraceConfig,
}

impl GoldenTraceManager {
    /// Create a new manager with the given trace directory
    pub fn new(trace_dir: impl Into<PathBuf>) -> Self {
        Self {
            trace_dir: trace_dir.into(),
            config: GoldenTraceConfig::default(),
        }
    }

    /// Create a new manager with configuration
    pub fn with_config(trace_dir: impl Into<PathBuf>, config: GoldenTraceConfig) -> Self {
        Self {
            trace_dir: trace_dir.into(),
            config,
        }
    }

    /// Get the path for a trace file
    pub fn trace_path(&self, trace_name: &str) -> PathBuf {
        self.trace_dir.join(format!("{}.trace.json", trace_name))
    }

    /// Check if a golden trace exists
    pub fn trace_exists(&self, trace_name: &str) -> bool {
        self.trace_path(trace_name).exists()
    }

    /// List all available golden traces
    pub fn list_traces(&self) -> Result<Vec<String>> {
        if !self.trace_dir.exists() {
            return Ok(Vec::new());
        }

        let mut traces = Vec::new();
        for entry in std::fs::read_dir(&self.trace_dir).map_err(|e| {
            Error::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to read trace directory: {}", e),
            ))
        })? {
            let entry = entry.map_err(|e| {
                Error::Io(std::io::Error::new(
                    e.kind(),
                    format!("Failed to read directory entry: {}", e),
                ))
            })?;
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "json") {
                if let Some(stem) = path.file_stem() {
                    if let Some(name) = stem.to_str() {
                        if let Some(trace_name) = name.strip_suffix(".trace") {
                            traces.push(trace_name.to_string());
                        }
                    }
                }
            }
        }
        Ok(traces)
    }

    /// Save a golden trace
    pub fn save_trace(&self, trace: &GoldenTrace) -> Result<PathBuf> {
        // Ensure trace directory exists
        std::fs::create_dir_all(&self.trace_dir).map_err(|e| {
            Error::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to create trace directory: {}", e),
            ))
        })?;

        let path = self.trace_path(&trace.name);
        trace.save(&path)?;
        Ok(path)
    }

    /// Load a golden trace
    pub fn load_trace(&self, trace_name: &str) -> Result<GoldenTrace> {
        let path = self.trace_path(trace_name);
        if !path.exists() {
            return Err(Error::Validation(format!(
                "Golden trace '{}' not found at {}",
                trace_name,
                path.display()
            )));
        }
        GoldenTrace::load(&path)
    }

    /// Compare a current trace against a golden trace
    pub fn compare(&self, golden_name: &str, current: &GoldenTrace) -> Result<TraceComparison> {
        let golden = self.load_trace(golden_name)?;
        Ok(TraceComparison::compare(&golden, current))
    }

    /// Check if a path should be ignored based on configuration
    pub fn should_ignore_path(&self, path: &str) -> bool {
        for pattern in &self.config.ignore_paths {
            if pattern.ends_with('*') {
                let prefix = &pattern[..pattern.len() - 1];
                if path.starts_with(prefix) {
                    return true;
                }
            } else if path == pattern {
                return true;
            }
        }
        false
    }

    /// Check if a category should be captured
    pub fn should_capture_category(&self, category: &str) -> bool {
        self.config.capture.iter().any(|c| c == category)
    }

    /// Get configuration
    pub fn config(&self) -> &GoldenTraceConfig {
        &self.config
    }
}

/// Simulated trace collector for testing (without renacer)
/// In production, this would integrate with renacer::Tracer
#[derive(Debug, Default)]
pub struct SimulatedTraceCollector {
    events: Vec<TraceEvent>,
    sequence: u64,
    start_time_ns: u64,
}

impl SimulatedTraceCollector {
    /// Create a new collector
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            sequence: 0,
            start_time_ns: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(0),
        }
    }

    /// Record a file event
    pub fn record_file_event(
        &mut self,
        operation: &str,
        path: &str,
        flags: Option<&str>,
        step_id: Option<&str>,
        result: TraceResult,
    ) {
        let event = TraceEvent {
            sequence: self.sequence,
            timestamp_ns: self.elapsed_ns(),
            event_type: TraceEventType::File {
                operation: operation.to_string(),
                path: path.to_string(),
                flags: flags.map(|s| s.to_string()),
            },
            step_id: step_id.map(|s| s.to_string()),
            result,
        };
        self.events.push(event);
        self.sequence += 1;
    }

    /// Record a process event
    pub fn record_process_event(
        &mut self,
        operation: &str,
        command: Option<&str>,
        args: Option<Vec<String>>,
        step_id: Option<&str>,
        result: TraceResult,
    ) {
        let event = TraceEvent {
            sequence: self.sequence,
            timestamp_ns: self.elapsed_ns(),
            event_type: TraceEventType::Process {
                operation: operation.to_string(),
                command: command.map(|s| s.to_string()),
                args,
            },
            step_id: step_id.map(|s| s.to_string()),
            result,
        };
        self.events.push(event);
        self.sequence += 1;
    }

    /// Record a permission event
    pub fn record_permission_event(
        &mut self,
        operation: &str,
        path: &str,
        mode: Option<u32>,
        step_id: Option<&str>,
        result: TraceResult,
    ) {
        let event = TraceEvent {
            sequence: self.sequence,
            timestamp_ns: self.elapsed_ns(),
            event_type: TraceEventType::Permission {
                operation: operation.to_string(),
                path: path.to_string(),
                mode,
            },
            step_id: step_id.map(|s| s.to_string()),
            result,
        };
        self.events.push(event);
        self.sequence += 1;
    }

    /// Get elapsed time in nanoseconds
    fn elapsed_ns(&self) -> u64 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);
        now.saturating_sub(self.start_time_ns)
    }

    /// Collect all events into a golden trace
    pub fn into_trace(self, name: &str, version: &str) -> GoldenTrace {
        let duration_ms = self.elapsed_ns() / 1_000_000;
        GoldenTrace {
            name: name.to_string(),
            captured_at: chrono::Utc::now().to_rfc3339(),
            installer_version: version.to_string(),
            events: self.events,
            result_hash: String::new(),
            steps_executed: 0,
            duration_ms,
        }
    }

    /// Get current event count
    pub fn event_count(&self) -> usize {
        self.events.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_GOLDEN_001_config_default() {
        let config = GoldenTraceConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.trace_dir, ".golden-traces");
        assert_eq!(config.capture.len(), 4);
        assert!(config.capture.contains(&"file".to_string()));
    }

    #[test]
    fn test_GOLDEN_002_trace_event_summary_file() {
        let event = TraceEvent {
            sequence: 0,
            timestamp_ns: 1000,
            event_type: TraceEventType::File {
                operation: "open".to_string(),
                path: "/etc/passwd".to_string(),
                flags: Some("O_RDONLY".to_string()),
            },
            step_id: Some("step-1".to_string()),
            result: TraceResult::Success,
        };

        assert_eq!(event.summary(), "open(\"/etc/passwd\")");
        assert_eq!(event.category(), "file");
    }

    #[test]
    fn test_GOLDEN_003_trace_event_summary_process() {
        let event = TraceEvent {
            sequence: 1,
            timestamp_ns: 2000,
            event_type: TraceEventType::Process {
                operation: "exec".to_string(),
                command: Some("apt-get".to_string()),
                args: Some(vec!["install".to_string(), "curl".to_string()]),
            },
            step_id: None,
            result: TraceResult::Success,
        };

        assert_eq!(event.summary(), "exec(\"apt-get\")");
        assert_eq!(event.category(), "process");
    }

    #[test]
    fn test_GOLDEN_004_trace_event_summary_permission() {
        let event = TraceEvent {
            sequence: 2,
            timestamp_ns: 3000,
            event_type: TraceEventType::Permission {
                operation: "chmod".to_string(),
                path: "/usr/local/bin/script".to_string(),
                mode: Some(0o755),
            },
            step_id: None,
            result: TraceResult::Success,
        };

        assert_eq!(event.summary(), "chmod(\"/usr/local/bin/script\", 755)");
        assert_eq!(event.category(), "permission");
    }

    #[test]
    fn test_GOLDEN_005_trace_comparison_equivalent() {
        let trace1 = GoldenTrace {
            name: "test".to_string(),
            captured_at: "2025-01-01T00:00:00Z".to_string(),
            installer_version: "1.0.0".to_string(),
            events: vec![TraceEvent {
                sequence: 0,
                timestamp_ns: 0,
                event_type: TraceEventType::File {
                    operation: "open".to_string(),
                    path: "/etc/test".to_string(),
                    flags: None,
                },
                step_id: None,
                result: TraceResult::Success,
            }],
            result_hash: "abc123".to_string(),
            steps_executed: 1,
            duration_ms: 100,
        };

        let trace2 = trace1.clone();
        let comparison = TraceComparison::compare(&trace1, &trace2);

        assert!(comparison.is_equivalent());
        assert!(comparison.added.is_empty());
        assert!(comparison.removed.is_empty());
    }

    #[test]
    fn test_GOLDEN_006_trace_comparison_added() {
        let trace1 = GoldenTrace {
            name: "test".to_string(),
            captured_at: "2025-01-01T00:00:00Z".to_string(),
            installer_version: "1.0.0".to_string(),
            events: vec![],
            result_hash: "abc123".to_string(),
            steps_executed: 0,
            duration_ms: 0,
        };

        let trace2 = GoldenTrace {
            name: "test".to_string(),
            captured_at: "2025-01-02T00:00:00Z".to_string(),
            installer_version: "1.0.0".to_string(),
            events: vec![TraceEvent {
                sequence: 0,
                timestamp_ns: 0,
                event_type: TraceEventType::File {
                    operation: "open".to_string(),
                    path: "/etc/new".to_string(),
                    flags: None,
                },
                step_id: None,
                result: TraceResult::Success,
            }],
            result_hash: "def456".to_string(),
            steps_executed: 1,
            duration_ms: 50,
        };

        let comparison = TraceComparison::compare(&trace1, &trace2);

        assert!(!comparison.is_equivalent());
        assert_eq!(comparison.added.len(), 1);
        assert!(comparison.removed.is_empty());
    }

    #[test]
    fn test_GOLDEN_007_trace_comparison_removed() {
        let trace1 = GoldenTrace {
            name: "test".to_string(),
            captured_at: "2025-01-01T00:00:00Z".to_string(),
            installer_version: "1.0.0".to_string(),
            events: vec![TraceEvent {
                sequence: 0,
                timestamp_ns: 0,
                event_type: TraceEventType::File {
                    operation: "open".to_string(),
                    path: "/etc/old".to_string(),
                    flags: None,
                },
                step_id: None,
                result: TraceResult::Success,
            }],
            result_hash: "abc123".to_string(),
            steps_executed: 1,
            duration_ms: 100,
        };

        let trace2 = GoldenTrace {
            name: "test".to_string(),
            captured_at: "2025-01-02T00:00:00Z".to_string(),
            installer_version: "1.0.0".to_string(),
            events: vec![],
            result_hash: "def456".to_string(),
            steps_executed: 0,
            duration_ms: 0,
        };

        let comparison = TraceComparison::compare(&trace1, &trace2);

        assert!(!comparison.is_equivalent());
        assert!(comparison.added.is_empty());
        assert_eq!(comparison.removed.len(), 1);
    }

    #[test]
    fn test_GOLDEN_008_manager_trace_path() {
        let manager = GoldenTraceManager::new("/tmp/traces");
        let path = manager.trace_path("install-v1");
        assert_eq!(
            path.to_str().expect("path should be valid"),
            "/tmp/traces/install-v1.trace.json"
        );
    }

    #[test]
    fn test_GOLDEN_009_manager_should_ignore_path() {
        let manager = GoldenTraceManager::new("/tmp/traces");

        assert!(manager.should_ignore_path("/proc/1/status"));
        assert!(manager.should_ignore_path("/sys/class/net"));
        assert!(manager.should_ignore_path("/dev/null"));
        assert!(!manager.should_ignore_path("/etc/passwd"));
    }

    #[test]
    fn test_GOLDEN_010_manager_should_capture_category() {
        let manager = GoldenTraceManager::new("/tmp/traces");

        assert!(manager.should_capture_category("file"));
        assert!(manager.should_capture_category("network"));
        assert!(manager.should_capture_category("process"));
        assert!(manager.should_capture_category("permission"));
        assert!(!manager.should_capture_category("unknown"));
    }

    #[test]
    fn test_GOLDEN_011_simulated_collector() {
        let mut collector = SimulatedTraceCollector::new();

        collector.record_file_event(
            "open",
            "/etc/passwd",
            Some("O_RDONLY"),
            Some("step-1"),
            TraceResult::Success,
        );

        collector.record_process_event(
            "exec",
            Some("ls"),
            Some(vec!["-la".to_string()]),
            Some("step-2"),
            TraceResult::Success,
        );

        assert_eq!(collector.event_count(), 2);

        let trace = collector.into_trace("test-trace", "1.0.0");
        assert_eq!(trace.name, "test-trace");
        assert_eq!(trace.events.len(), 2);
    }

    #[test]
    fn test_GOLDEN_012_trace_report_equivalent() {
        let comparison = TraceComparison::default();
        let report = comparison.to_report();
        assert!(report.contains("EQUIVALENT"));
    }

    #[test]
    fn test_GOLDEN_013_trace_report_regression() {
        let comparison = TraceComparison {
            added: vec![TraceEvent {
                sequence: 0,
                timestamp_ns: 0,
                event_type: TraceEventType::File {
                    operation: "open".to_string(),
                    path: "/etc/shadow".to_string(),
                    flags: None,
                },
                step_id: None,
                result: TraceResult::Success,
            }],
            removed: vec![TraceEvent {
                sequence: 1,
                timestamp_ns: 0,
                event_type: TraceEventType::File {
                    operation: "open".to_string(),
                    path: "/etc/passwd".to_string(),
                    flags: None,
                },
                step_id: None,
                result: TraceResult::Success,
            }],
            changed: vec![],
            metadata: ComparisonMetadata {
                golden_name: "test".to_string(),
                golden_captured_at: "2025-01-01".to_string(),
                current_captured_at: "2025-01-02".to_string(),
                golden_event_count: 1,
                current_event_count: 1,
            },
        };

        let report = comparison.to_report();
        assert!(report.contains("security concern"));
        assert!(report.contains("regression"));
        assert!(report.contains("/etc/shadow"));
        assert!(report.contains("/etc/passwd"));
    }

    #[test]
    fn test_GOLDEN_014_trace_save_load() {
        let temp_dir = std::env::temp_dir().join("bashrs-golden-test");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).expect("create temp dir");

        let trace = GoldenTrace {
            name: "save-load-test".to_string(),
            captured_at: "2025-01-01T00:00:00Z".to_string(),
            installer_version: "1.0.0".to_string(),
            events: vec![TraceEvent {
                sequence: 0,
                timestamp_ns: 1000,
                event_type: TraceEventType::File {
                    operation: "write".to_string(),
                    path: "/tmp/test".to_string(),
                    flags: Some("O_WRONLY".to_string()),
                },
                step_id: Some("step-1".to_string()),
                result: TraceResult::Success,
            }],
            result_hash: "test-hash".to_string(),
            steps_executed: 1,
            duration_ms: 100,
        };

        let path = temp_dir.join("test.trace.json");
        trace.save(&path).expect("save trace");

        let loaded = GoldenTrace::load(&path).expect("load trace");
        assert_eq!(loaded.name, trace.name);
        assert_eq!(loaded.events.len(), 1);
        assert_eq!(loaded.events[0].summary(), "write(\"/tmp/test\")");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_GOLDEN_015_manager_save_and_list() {
        let temp_dir = std::env::temp_dir().join("bashrs-golden-list-test");
        let _ = std::fs::remove_dir_all(&temp_dir);

        let manager = GoldenTraceManager::new(&temp_dir);

        let trace1 = GoldenTrace {
            name: "trace-a".to_string(),
            captured_at: "2025-01-01T00:00:00Z".to_string(),
            installer_version: "1.0.0".to_string(),
            events: vec![],
            result_hash: "".to_string(),
            steps_executed: 0,
            duration_ms: 0,
        };

        let trace2 = GoldenTrace {
            name: "trace-b".to_string(),
            captured_at: "2025-01-01T00:00:00Z".to_string(),
            installer_version: "1.0.0".to_string(),
            events: vec![],
            result_hash: "".to_string(),
            steps_executed: 0,
            duration_ms: 0,
        };

        manager.save_trace(&trace1).expect("save trace 1");
        manager.save_trace(&trace2).expect("save trace 2");

        let traces = manager.list_traces().expect("list traces");
        assert_eq!(traces.len(), 2);
        assert!(traces.contains(&"trace-a".to_string()));
        assert!(traces.contains(&"trace-b".to_string()));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_GOLDEN_016_trace_result_variants() {
        assert_eq!(TraceResult::Success, TraceResult::Success);
        assert_eq!(TraceResult::Error(1), TraceResult::Error(1));
        assert_ne!(TraceResult::Error(1), TraceResult::Error(2));
        assert_eq!(TraceResult::Unknown, TraceResult::Unknown);
    }

    #[test]
    fn test_GOLDEN_017_network_event_summary() {
        let event = TraceEvent {
            sequence: 0,
            timestamp_ns: 0,
            event_type: TraceEventType::Network {
                operation: "connect".to_string(),
                address: Some("192.168.1.1".to_string()),
                port: Some(443),
            },
            step_id: None,
            result: TraceResult::Success,
        };

        assert_eq!(event.summary(), "connect(192.168.1.1:443)");
        assert_eq!(event.category(), "network");
    }

    #[test]
    fn test_GOLDEN_018_network_event_no_address() {
        let event = TraceEvent {
            sequence: 0,
            timestamp_ns: 0,
            event_type: TraceEventType::Network {
                operation: "listen".to_string(),
                address: None,
                port: None,
            },
            step_id: None,
            result: TraceResult::Success,
        };

        assert_eq!(event.summary(), "listen");
    }

    #[test]
    fn test_GOLDEN_019_permission_event_no_mode() {
        let event = TraceEvent {
            sequence: 0,
            timestamp_ns: 0,
            event_type: TraceEventType::Permission {
                operation: "chown".to_string(),
                path: "/tmp/file".to_string(),
                mode: None,
            },
            step_id: None,
            result: TraceResult::Success,
        };

        assert_eq!(event.summary(), "chown(\"/tmp/file\")");
    }

    #[test]
    fn test_GOLDEN_020_process_event_no_command() {
        let event = TraceEvent {
            sequence: 0,
            timestamp_ns: 0,
            event_type: TraceEventType::Process {
                operation: "fork".to_string(),
                command: None,
                args: None,
            },
            step_id: None,
            result: TraceResult::Success,
        };

        assert_eq!(event.summary(), "fork");
    }
}
