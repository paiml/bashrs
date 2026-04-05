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
#[path = "golden_trace_tests_extracted.rs"]
mod tests_extracted;
