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
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| Error::Validation(format!("Failed to serialize golden trace: {}", e)))?;
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
        serde_json::from_str(&content)
            .map_err(|e| Error::Validation(format!("Failed to parse golden trace: {}", e)))
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
        let golden_sigs: HashSet<String> = golden.events.iter().map(|e| e.summary()).collect();
        let current_sigs: HashSet<String> = current.events.iter().map(|e| e.summary()).collect();

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

include!("golden_trace_goldentracemanager.rs");
