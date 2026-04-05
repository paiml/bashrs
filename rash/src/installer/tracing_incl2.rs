/// An event within a span
#[derive(Debug, Clone)]
pub struct SpanEvent {
    /// Event name
    pub name: String,
    /// Event timestamp
    pub timestamp: SystemTime,
    /// Event attributes
    pub attributes: HashMap<String, AttributeValue>,
}

impl SpanEvent {
    /// Create new event
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            timestamp: SystemTime::now(),
            attributes: HashMap::new(),
        }
    }

    /// Create event with attributes
    pub fn with_attributes(name: &str, attributes: HashMap<String, AttributeValue>) -> Self {
        Self {
            name: name.to_string(),
            timestamp: SystemTime::now(),
            attributes,
        }
    }

    /// Format as JSON
    pub fn to_json(&self) -> String {
        let mut json = format!(
            "{{\"name\": \"{}\", \"timeUnixNano\": {}",
            escape_json(&self.name),
            time_to_nanos(self.timestamp)
        );

        if !self.attributes.is_empty() {
            json.push_str(", \"attributes\": {");
            let attrs: Vec<String> = self
                .attributes
                .iter()
                .map(|(k, v)| format!("\"{}\": {}", escape_json(k), v.to_json()))
                .collect();
            json.push_str(&attrs.join(", "));
            json.push('}');
        }

        json.push('}');
        json
    }
}

/// Trace exporter configuration
#[derive(Debug, Clone, Default)]
pub enum TraceExporter {
    /// Export to stdout
    #[default]
    Stdout,
    /// Export to file
    File(PathBuf),
    /// Export to OTLP endpoint
    Otlp(String),
    /// In-memory (for testing)
    Memory,
}

/// Tracing context for an installer run
#[derive(Debug)]
pub struct TracingContext {
    /// Trace ID for this run
    trace_id: String,
    /// Root span
    root_span: Option<Span>,
    /// Active span stack
    span_stack: Vec<Span>,
    /// Completed spans
    completed_spans: Vec<Span>,
    /// Trace level
    level: TraceLevel,
    /// Exporter
    exporter: TraceExporter,
    /// Service name
    service_name: String,
    /// Service version
    service_version: String,
}

impl TracingContext {
    /// Create new tracing context
    pub fn new(service_name: &str, service_version: &str) -> Self {
        Self {
            trace_id: generate_trace_id(),
            root_span: None,
            span_stack: Vec::new(),
            completed_spans: Vec::new(),
            level: TraceLevel::Info,
            exporter: TraceExporter::default(),
            service_name: service_name.to_string(),
            service_version: service_version.to_string(),
        }
    }

    /// Set trace level
    pub fn with_level(mut self, level: TraceLevel) -> Self {
        self.level = level;
        self
    }

    /// Set exporter
    pub fn with_exporter(mut self, exporter: TraceExporter) -> Self {
        self.exporter = exporter;
        self
    }

    /// Get trace ID
    pub fn trace_id(&self) -> &str {
        &self.trace_id
    }

    /// Get service name
    pub fn service_name(&self) -> &str {
        &self.service_name
    }

    /// Start root span
    #[allow(clippy::expect_used)] // We just set root_span = Some, so it's guaranteed to be Some
    pub fn start_root(&mut self, name: &str) -> &Span {
        let mut span = Span::new(name, &self.trace_id);
        span.set_string("service.name", &self.service_name);
        span.set_string("service.version", &self.service_version);
        self.root_span = Some(span);
        self.root_span.as_ref().expect("just set")
    }

    /// Start a new span
    pub fn start_span(&mut self, name: &str) -> String {
        let parent_id = self.current_span_id();
        let mut span = Span::new(name, &self.trace_id);
        if let Some(pid) = parent_id {
            span.parent_id = Some(pid);
        }
        let span_id = span.span_id.clone();
        self.span_stack.push(span);
        span_id
    }

    /// Start a span for step execution
    pub fn start_step_span(&mut self, step_id: &str, step_name: &str) -> String {
        let span_id = self.start_span(&format!("step:{}", step_id));
        if let Some(span) = self.span_stack.last_mut() {
            span.set_string("step.id", step_id);
            span.set_string("step.name", step_name);
        }
        span_id
    }

    /// Start artifact verification span
    pub fn start_artifact_span(&mut self, artifact_id: &str) -> String {
        let span_id = self.start_span(&format!("artifact:{}", artifact_id));
        if let Some(span) = self.span_stack.last_mut() {
            span.set_string("artifact.id", artifact_id);
            span.kind = SpanKind::Client;
        }
        span_id
    }

    /// Start precondition check span
    pub fn start_precondition_span(&mut self, condition: &str) -> String {
        let span_id = self.start_span("precondition");
        if let Some(span) = self.span_stack.last_mut() {
            span.set_string("condition", condition);
        }
        span_id
    }

    /// Start postcondition check span
    pub fn start_postcondition_span(&mut self, condition: &str) -> String {
        let span_id = self.start_span("postcondition");
        if let Some(span) = self.span_stack.last_mut() {
            span.set_string("condition", condition);
        }
        span_id
    }

    /// Get current span ID
    pub fn current_span_id(&self) -> Option<String> {
        self.span_stack
            .last()
            .map(|s| s.span_id.clone())
            .or_else(|| self.root_span.as_ref().map(|s| s.span_id.clone()))
    }

    /// Set attribute on current span
    pub fn set_attribute(&mut self, key: &str, value: AttributeValue) {
        if let Some(span) = self.span_stack.last_mut() {
            span.set_attribute(key, value);
        } else if let Some(span) = self.root_span.as_mut() {
            span.set_attribute(key, value);
        }
    }

    /// Add event to current span
    pub fn add_event(&mut self, name: &str) {
        if let Some(span) = self.span_stack.last_mut() {
            span.add_event(name);
        } else if let Some(span) = self.root_span.as_mut() {
            span.add_event(name);
        }
    }

    /// End current span with OK status
    pub fn end_span_ok(&mut self) {
        if let Some(mut span) = self.span_stack.pop() {
            span.end_ok();
            self.completed_spans.push(span);
        }
    }

    /// End current span with error
    pub fn end_span_error(&mut self, message: &str) {
        if let Some(mut span) = self.span_stack.pop() {
            span.end_error(message);
            self.completed_spans.push(span);
        }
    }

    /// End root span
    pub fn end_root_ok(&mut self) {
        if let Some(ref mut span) = self.root_span {
            span.end_ok();
        }
    }

    /// End root span with error
    pub fn end_root_error(&mut self, message: &str) {
        if let Some(ref mut span) = self.root_span {
            span.end_error(message);
        }
    }

    /// Get completed span count
    pub fn completed_count(&self) -> usize {
        self.completed_spans.len()
    }

    /// Get all spans (completed + root)
    pub fn all_spans(&self) -> Vec<&Span> {
        let mut spans: Vec<&Span> = self.completed_spans.iter().collect();
        if let Some(ref root) = self.root_span {
            spans.push(root);
        }
        spans
    }

    /// Export traces
    pub fn export(&self) -> String {
        let spans = self.all_spans();

        let mut json = String::from("{\n");
        json.push_str("  \"resourceSpans\": [{\n");
        json.push_str("    \"resource\": {\n");
        json.push_str("      \"attributes\": {\n");
        json.push_str(&format!(
            "        \"service.name\": \"{}\",\n",
            escape_json(&self.service_name)
        ));
        json.push_str(&format!(
            "        \"service.version\": \"{}\"\n",
            escape_json(&self.service_version)
        ));
        json.push_str("      }\n");
        json.push_str("    },\n");
        json.push_str("    \"scopeSpans\": [{\n");
        json.push_str("      \"scope\": {\n");
        json.push_str("        \"name\": \"bashrs-installer\",\n");
        json.push_str("        \"version\": \"2.0.0\"\n");
        json.push_str("      },\n");
        json.push_str("      \"spans\": [\n");

        let span_jsons: Vec<String> = spans
            .iter()
            .map(|s| format!("        {}", s.to_json().replace('\n', "\n        ")))
            .collect();
        json.push_str(&span_jsons.join(",\n"));

        json.push_str("\n      ]\n");
        json.push_str("    }]\n");
        json.push_str("  }]\n");
        json.push_str("}\n");

        json
    }

    /// Generate trace summary
    pub fn summary(&self) -> TraceSummary {
        let spans = self.all_spans();
        let total = spans.len();
        let ok_count = spans.iter().filter(|s| s.status == SpanStatus::Ok).count();
        let error_count = spans
            .iter()
            .filter(|s| s.status == SpanStatus::Error)
            .count();

        let total_duration = self
            .root_span
            .as_ref()
            .and_then(|s| s.duration())
            .unwrap_or(Duration::ZERO);

        TraceSummary {
            trace_id: self.trace_id.clone(),
            total_spans: total,
            ok_spans: ok_count,
            error_spans: error_count,
            total_duration,
        }
    }
}

/// Summary of a trace
#[derive(Debug, Clone)]
pub struct TraceSummary {
    /// Trace ID
    pub trace_id: String,
    /// Total number of spans
    pub total_spans: usize,
    /// Spans with OK status
    pub ok_spans: usize,
    /// Spans with error status
    pub error_spans: usize,
    /// Total trace duration
    pub total_duration: Duration,
}

impl TraceSummary {
    /// Format as text
    pub fn format(&self) -> String {
        let duration = if self.total_duration.as_secs() >= 60 {
            format!(
                "{}m {:02}s",
                self.total_duration.as_secs() / 60,
                self.total_duration.as_secs() % 60
            )
        } else {
            format!("{:.2}s", self.total_duration.as_secs_f64())
        };

        format!(
            "Trace Summary\n\
             ─────────────────────────────────\n\
               Trace ID: {}\n\
               Spans: {} total, {} ok, {} error\n\
               Duration: {}\n",
            truncate(&self.trace_id, 16),
            self.total_spans,
            self.ok_spans,
            self.error_spans,
            duration
        )
    }
}

/// Log entry for structured logging
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// Timestamp
    pub timestamp: SystemTime,
    /// Log level
    pub level: TraceLevel,
    /// Log message
    pub message: String,
    /// Associated span ID
    pub span_id: Option<String>,
    /// Associated trace ID
    pub trace_id: Option<String>,
    /// Additional attributes
    pub attributes: HashMap<String, AttributeValue>,
}

impl LogEntry {
    /// Create new log entry
    pub fn new(level: TraceLevel, message: &str) -> Self {
        Self {
            timestamp: SystemTime::now(),
            level,
            message: message.to_string(),
            span_id: None,
            trace_id: None,
            attributes: HashMap::new(),
        }
    }

    /// Set span context
    pub fn with_span(mut self, span_id: &str, trace_id: &str) -> Self {
        self.span_id = Some(span_id.to_string());
        self.trace_id = Some(trace_id.to_string());
        self
    }

    /// Add attribute
    pub fn with_attr(mut self, key: &str, value: AttributeValue) -> Self {
        self.attributes.insert(key.to_string(), value);
        self
    }

    /// Format as JSON
    pub fn to_json(&self) -> String {
        let mut json = String::from("{");
        json.push_str(&format!(
            "\"timestamp\": \"{}\", \"level\": \"{}\", \"message\": \"{}\"",
            format_timestamp(self.timestamp),
            self.level.name(),
            escape_json(&self.message)
        ));

        if let Some(ref span_id) = self.span_id {
            json.push_str(&format!(", \"spanId\": \"{}\"", span_id));
        }

        if let Some(ref trace_id) = self.trace_id {
            json.push_str(&format!(", \"traceId\": \"{}\"", trace_id));
        }

        if !self.attributes.is_empty() {
            json.push_str(", \"attributes\": {");
            let attrs: Vec<String> = self
                .attributes
                .iter()
                .map(|(k, v)| format!("\"{}\": {}", escape_json(k), v.to_json()))
                .collect();
            json.push_str(&attrs.join(", "));
            json.push('}');
        }

        json.push('}');
        json
    }

    /// Format as text
    pub fn format(&self) -> String {
        format!(
            "{} [{}] {}",
            format_timestamp(self.timestamp),
            self.level.name(),
            self.message
        )
    }
}

/// Structured logger with trace context
#[derive(Debug)]
pub struct Logger {
    /// Trace context
    context: Option<TracingContext>,
    /// Log entries
    entries: Vec<LogEntry>,
    /// Minimum log level
    min_level: TraceLevel,
}

impl Logger {
    /// Create new logger
    pub fn new() -> Self {
        Self {
            context: None,
            entries: Vec::new(),
            min_level: TraceLevel::Info,
        }
    }

    /// Set trace context
    pub fn with_context(mut self, context: TracingContext) -> Self {
        self.context = Some(context);
        self
    }

    /// Set minimum log level
    pub fn with_level(mut self, level: TraceLevel) -> Self {
        self.min_level = level;
        self
    }

    /// Log at specified level
    pub fn log(&mut self, level: TraceLevel, message: &str) {
        if !self.min_level.should_log(level) {
            return;
        }

        let mut entry = LogEntry::new(level, message);

        if let Some(ref ctx) = self.context {
            if let Some(span_id) = ctx.current_span_id() {
                entry = entry.with_span(&span_id, ctx.trace_id());
            }
        }

        self.entries.push(entry);
    }

    /// Log error
    pub fn error(&mut self, message: &str) {
        self.log(TraceLevel::Error, message);
    }

    /// Log warning
    pub fn warn(&mut self, message: &str) {
        self.log(TraceLevel::Warn, message);
    }

    /// Log info
    pub fn info(&mut self, message: &str) {
        self.log(TraceLevel::Info, message);
    }

    /// Log debug
    pub fn debug(&mut self, message: &str) {
        self.log(TraceLevel::Debug, message);
    }

    /// Get log entries
    pub fn entries(&self) -> &[LogEntry] {
        &self.entries
    }

    /// Get context
    pub fn context(&self) -> Option<&TracingContext> {
        self.context.as_ref()
    }

    /// Get mutable context
    pub fn context_mut(&mut self) -> Option<&mut TracingContext> {
        self.context.as_mut()
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate a random ID (hex string)
fn generate_id() -> String {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{:016x}", now ^ 0xDEADBEEF_u128)
}

/// Generate a trace ID (32-char hex)
fn generate_trace_id() -> String {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{:032x}", now)
}

/// Convert SystemTime to nanoseconds since epoch
fn time_to_nanos(time: SystemTime) -> u128 {
    time.duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
}

/// Format timestamp for display
fn format_timestamp(time: SystemTime) -> String {
    let duration = time.duration_since(UNIX_EPOCH).unwrap_or(Duration::ZERO);
    let secs = duration.as_secs();
    let millis = duration.subsec_millis();
    format!("{}.{:03}", secs, millis)
}

/// Escape string for JSON
fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Truncate string with ellipsis
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

#[cfg(test)]
#[path = "tracing_tests_extracted.rs"]
mod tests_extracted;
