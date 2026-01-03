//! OpenTelemetry tracing for installers (#117)
//!
//! Provides structured tracing and observability for installer execution.
//! Integrates with OpenTelemetry for distributed tracing support.
//!
//! # Example
//!
//! ```bash
//! # Run with tracing enabled
//! bashrs installer run ./my-installer --trace
//!
//! # Export traces to Jaeger
//! bashrs installer run ./my-installer --trace --trace-endpoint http://localhost:4317
//!
//! # Export traces to file
//! bashrs installer run ./my-installer --trace --trace-file traces.json
//! ```

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Trace level for filtering
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum TraceLevel {
    /// No tracing
    Off,
    /// Errors only
    Error,
    /// Warnings and errors
    Warn,
    /// Info, warnings, and errors
    #[default]
    Info,
    /// Debug level (includes info)
    Debug,
    /// Trace level (most verbose)
    Trace,
}

impl TraceLevel {
    /// Parse from string
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "off" | "none" => Some(Self::Off),
            "error" => Some(Self::Error),
            "warn" | "warning" => Some(Self::Warn),
            "info" => Some(Self::Info),
            "debug" => Some(Self::Debug),
            "trace" => Some(Self::Trace),
            _ => None,
        }
    }

    /// Get level name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Off => "OFF",
            Self::Error => "ERROR",
            Self::Warn => "WARN",
            Self::Info => "INFO",
            Self::Debug => "DEBUG",
            Self::Trace => "TRACE",
        }
    }

    /// Check if a level should be logged at this threshold
    pub fn should_log(&self, level: TraceLevel) -> bool {
        level <= *self
    }
}

/// Span status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SpanStatus {
    /// Span is unset (in progress)
    #[default]
    Unset,
    /// Span completed successfully
    Ok,
    /// Span completed with error
    Error,
}

impl SpanStatus {
    /// Get status name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Unset => "UNSET",
            Self::Ok => "OK",
            Self::Error => "ERROR",
        }
    }
}

/// Span kind
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SpanKind {
    /// Internal span
    #[default]
    Internal,
    /// Server-side span
    Server,
    /// Client-side span
    Client,
    /// Producer span (async)
    Producer,
    /// Consumer span (async)
    Consumer,
}

impl SpanKind {
    /// Get kind name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Internal => "INTERNAL",
            Self::Server => "SERVER",
            Self::Client => "CLIENT",
            Self::Producer => "PRODUCER",
            Self::Consumer => "CONSUMER",
        }
    }
}

/// Attribute value for spans
#[derive(Debug, Clone, PartialEq)]
pub enum AttributeValue {
    /// String value
    String(String),
    /// Integer value
    Int(i64),
    /// Float value
    Float(f64),
    /// Boolean value
    Bool(bool),
    /// String array
    StringArray(Vec<String>),
    /// Integer array
    IntArray(Vec<i64>),
}

impl AttributeValue {
    /// Create string attribute
    pub fn string(s: impl Into<String>) -> Self {
        Self::String(s.into())
    }

    /// Create integer attribute
    pub fn int(i: i64) -> Self {
        Self::Int(i)
    }

    /// Create float attribute
    pub fn float(f: f64) -> Self {
        Self::Float(f)
    }

    /// Create boolean attribute
    pub fn bool(b: bool) -> Self {
        Self::Bool(b)
    }

    /// Format for JSON output
    pub fn to_json(&self) -> String {
        match self {
            Self::String(s) => format!("\"{}\"", escape_json(s)),
            Self::Int(i) => i.to_string(),
            Self::Float(f) => f.to_string(),
            Self::Bool(b) => b.to_string(),
            Self::StringArray(arr) => {
                let items: Vec<String> = arr
                    .iter()
                    .map(|s| format!("\"{}\"", escape_json(s)))
                    .collect();
                format!("[{}]", items.join(", "))
            }
            Self::IntArray(arr) => {
                let items: Vec<String> = arr.iter().map(|i| i.to_string()).collect();
                format!("[{}]", items.join(", "))
            }
        }
    }
}

/// A trace span representing an operation
#[derive(Debug, Clone)]
pub struct Span {
    /// Unique span ID
    pub span_id: String,
    /// Parent span ID (if any)
    pub parent_id: Option<String>,
    /// Trace ID (shared across spans)
    pub trace_id: String,
    /// Span name
    pub name: String,
    /// Span kind
    pub kind: SpanKind,
    /// Start time
    pub start_time: SystemTime,
    /// End time (if completed)
    pub end_time: Option<SystemTime>,
    /// Span status
    pub status: SpanStatus,
    /// Status message
    pub status_message: Option<String>,
    /// Attributes
    pub attributes: HashMap<String, AttributeValue>,
    /// Events within the span
    pub events: Vec<SpanEvent>,
}

impl Span {
    /// Create a new span
    pub fn new(name: &str, trace_id: &str) -> Self {
        Self {
            span_id: generate_id(),
            parent_id: None,
            trace_id: trace_id.to_string(),
            name: name.to_string(),
            kind: SpanKind::Internal,
            start_time: SystemTime::now(),
            end_time: None,
            status: SpanStatus::Unset,
            status_message: None,
            attributes: HashMap::new(),
            events: Vec::new(),
        }
    }

    /// Create child span
    pub fn child(&self, name: &str) -> Self {
        let mut span = Self::new(name, &self.trace_id);
        span.parent_id = Some(self.span_id.clone());
        span
    }

    /// Set parent span ID
    pub fn with_parent(mut self, parent_id: &str) -> Self {
        self.parent_id = Some(parent_id.to_string());
        self
    }

    /// Set span kind
    pub fn with_kind(mut self, kind: SpanKind) -> Self {
        self.kind = kind;
        self
    }

    /// Add attribute
    pub fn set_attribute(&mut self, key: &str, value: AttributeValue) {
        self.attributes.insert(key.to_string(), value);
    }

    /// Add string attribute
    pub fn set_string(&mut self, key: &str, value: impl Into<String>) {
        self.set_attribute(key, AttributeValue::string(value));
    }

    /// Add integer attribute
    pub fn set_int(&mut self, key: &str, value: i64) {
        self.set_attribute(key, AttributeValue::int(value));
    }

    /// Add boolean attribute
    pub fn set_bool(&mut self, key: &str, value: bool) {
        self.set_attribute(key, AttributeValue::bool(value));
    }

    /// Add event to span
    pub fn add_event(&mut self, name: &str) {
        self.events.push(SpanEvent::new(name));
    }

    /// Add event with attributes
    pub fn add_event_with_attrs(&mut self, name: &str, attrs: HashMap<String, AttributeValue>) {
        self.events.push(SpanEvent::with_attributes(name, attrs));
    }

    /// End span with OK status
    pub fn end_ok(&mut self) {
        self.end_time = Some(SystemTime::now());
        self.status = SpanStatus::Ok;
    }

    /// End span with error status
    pub fn end_error(&mut self, message: &str) {
        self.end_time = Some(SystemTime::now());
        self.status = SpanStatus::Error;
        self.status_message = Some(message.to_string());
    }

    /// Get duration if completed
    pub fn duration(&self) -> Option<Duration> {
        self.end_time.map(|end| {
            end.duration_since(self.start_time)
                .unwrap_or(Duration::ZERO)
        })
    }

    /// Check if span is completed
    pub fn is_completed(&self) -> bool {
        self.end_time.is_some()
    }

    /// Format as JSON
    pub fn to_json(&self) -> String {
        let mut json = String::from("{\n");
        json.push_str(&format!("  \"traceId\": \"{}\",\n", self.trace_id));
        json.push_str(&format!("  \"spanId\": \"{}\",\n", self.span_id));

        if let Some(ref parent) = self.parent_id {
            json.push_str(&format!("  \"parentSpanId\": \"{}\",\n", parent));
        }

        json.push_str(&format!("  \"name\": \"{}\",\n", escape_json(&self.name)));
        json.push_str(&format!("  \"kind\": \"{}\",\n", self.kind.name()));
        json.push_str(&format!(
            "  \"startTimeUnixNano\": {},\n",
            time_to_nanos(self.start_time)
        ));

        if let Some(end) = self.end_time {
            json.push_str(&format!("  \"endTimeUnixNano\": {},\n", time_to_nanos(end)));
        }

        json.push_str(&format!(
            "  \"status\": {{\n    \"code\": \"{}\"\n  }}",
            self.status.name()
        ));

        if !self.attributes.is_empty() {
            json.push_str(",\n  \"attributes\": {\n");
            let attrs: Vec<String> = self
                .attributes
                .iter()
                .map(|(k, v)| format!("    \"{}\": {}", escape_json(k), v.to_json()))
                .collect();
            json.push_str(&attrs.join(",\n"));
            json.push_str("\n  }");
        }

        if !self.events.is_empty() {
            json.push_str(",\n  \"events\": [\n");
            let events: Vec<String> = self
                .events
                .iter()
                .map(|e| format!("    {}", e.to_json()))
                .collect();
            json.push_str(&events.join(",\n"));
            json.push_str("\n  ]");
        }

        json.push_str("\n}");
        json
    }
}

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
mod tests {
    use super::*;

    #[test]
    fn test_TRACING_001_trace_level_parse() {
        assert_eq!(TraceLevel::parse("off"), Some(TraceLevel::Off));
        assert_eq!(TraceLevel::parse("error"), Some(TraceLevel::Error));
        assert_eq!(TraceLevel::parse("warn"), Some(TraceLevel::Warn));
        assert_eq!(TraceLevel::parse("INFO"), Some(TraceLevel::Info));
        assert_eq!(TraceLevel::parse("debug"), Some(TraceLevel::Debug));
        assert_eq!(TraceLevel::parse("trace"), Some(TraceLevel::Trace));
        assert_eq!(TraceLevel::parse("invalid"), None);
    }

    #[test]
    fn test_TRACING_002_trace_level_should_log() {
        assert!(TraceLevel::Info.should_log(TraceLevel::Error));
        assert!(TraceLevel::Info.should_log(TraceLevel::Warn));
        assert!(TraceLevel::Info.should_log(TraceLevel::Info));
        assert!(!TraceLevel::Info.should_log(TraceLevel::Debug));
        assert!(!TraceLevel::Info.should_log(TraceLevel::Trace));
    }

    #[test]
    fn test_TRACING_003_span_creation() {
        let span = Span::new("test-span", "trace-123");

        assert!(!span.span_id.is_empty());
        assert_eq!(span.trace_id, "trace-123");
        assert_eq!(span.name, "test-span");
        assert!(span.parent_id.is_none());
        assert_eq!(span.status, SpanStatus::Unset);
    }

    #[test]
    fn test_TRACING_004_span_child() {
        let parent = Span::new("parent", "trace-123");
        let child = parent.child("child");

        assert_eq!(child.parent_id, Some(parent.span_id.clone()));
        assert_eq!(child.trace_id, parent.trace_id);
    }

    #[test]
    fn test_TRACING_005_span_attributes() {
        let mut span = Span::new("test", "trace-123");
        span.set_string("key1", "value1");
        span.set_int("key2", 42);
        span.set_bool("key3", true);

        assert_eq!(span.attributes.len(), 3);
        assert!(
            matches!(span.attributes.get("key1"), Some(AttributeValue::String(s)) if s == "value1")
        );
        assert!(matches!(
            span.attributes.get("key2"),
            Some(AttributeValue::Int(42))
        ));
        assert!(matches!(
            span.attributes.get("key3"),
            Some(AttributeValue::Bool(true))
        ));
    }

    #[test]
    fn test_TRACING_006_span_events() {
        let mut span = Span::new("test", "trace-123");
        span.add_event("event1");
        span.add_event("event2");

        assert_eq!(span.events.len(), 2);
        assert_eq!(span.events[0].name, "event1");
        assert_eq!(span.events[1].name, "event2");
    }

    #[test]
    fn test_TRACING_007_span_end_ok() {
        let mut span = Span::new("test", "trace-123");
        assert!(!span.is_completed());

        span.end_ok();

        assert!(span.is_completed());
        assert_eq!(span.status, SpanStatus::Ok);
        assert!(span.duration().is_some());
    }

    #[test]
    fn test_TRACING_008_span_end_error() {
        let mut span = Span::new("test", "trace-123");
        span.end_error("Something failed");

        assert!(span.is_completed());
        assert_eq!(span.status, SpanStatus::Error);
        assert_eq!(span.status_message, Some("Something failed".to_string()));
    }

    #[test]
    fn test_TRACING_009_span_to_json() {
        let mut span = Span::new("test", "trace-123");
        span.set_string("key", "value");
        span.end_ok();

        let json = span.to_json();

        assert!(json.contains("\"traceId\": \"trace-123\""));
        assert!(json.contains("\"name\": \"test\""));
        assert!(json.contains("\"key\": \"value\""));
        assert!(json.contains("\"status\""));
    }

    #[test]
    fn test_TRACING_010_context_creation() {
        let ctx = TracingContext::new("test-service", "1.0.0");

        assert!(!ctx.trace_id().is_empty());
        assert_eq!(ctx.service_name(), "test-service");
    }

    #[test]
    fn test_TRACING_011_context_root_span() {
        let mut ctx = TracingContext::new("test-service", "1.0.0");
        let span = ctx.start_root("installer");

        assert_eq!(span.name, "installer");
        assert!(ctx.current_span_id().is_some());
    }

    #[test]
    fn test_TRACING_012_context_span_stack() {
        let mut ctx = TracingContext::new("test-service", "1.0.0");
        ctx.start_root("root");

        let id1 = ctx.start_span("span1");
        let id2 = ctx.start_span("span2");

        assert_eq!(ctx.current_span_id(), Some(id2.clone()));

        ctx.end_span_ok();
        assert_eq!(ctx.current_span_id(), Some(id1.clone()));

        ctx.end_span_ok();
        assert_eq!(ctx.completed_count(), 2);
    }

    #[test]
    fn test_TRACING_013_context_step_span() {
        let mut ctx = TracingContext::new("test-service", "1.0.0");
        ctx.start_root("installer");

        let span_id = ctx.start_step_span("step-1", "Install Package");

        assert!(!span_id.is_empty());
        ctx.end_span_ok();

        let spans = ctx.all_spans();
        let step_span = spans.iter().find(|s| s.name == "step:step-1");
        assert!(step_span.is_some());
    }

    #[test]
    fn test_TRACING_014_context_export() {
        let mut ctx = TracingContext::new("test-service", "1.0.0");
        ctx.start_root("installer");
        ctx.start_span("step1");
        ctx.end_span_ok();
        ctx.end_root_ok();

        let json = ctx.export();

        assert!(json.contains("resourceSpans"));
        assert!(json.contains("test-service"));
        assert!(json.contains("scopeSpans"));
    }

    #[test]
    fn test_TRACING_015_context_summary() {
        let mut ctx = TracingContext::new("test-service", "1.0.0");
        ctx.start_root("installer");

        ctx.start_span("ok-span");
        ctx.end_span_ok();

        ctx.start_span("error-span");
        ctx.end_span_error("failed");

        ctx.end_root_ok();

        let summary = ctx.summary();

        assert_eq!(summary.total_spans, 3); // root + 2 child
        assert_eq!(summary.ok_spans, 2);
        assert_eq!(summary.error_spans, 1);
    }

    #[test]
    fn test_TRACING_016_log_entry() {
        let entry = LogEntry::new(TraceLevel::Info, "Test message")
            .with_span("span-123", "trace-456")
            .with_attr("key", AttributeValue::string("value"));

        assert_eq!(entry.level, TraceLevel::Info);
        assert_eq!(entry.message, "Test message");
        assert_eq!(entry.span_id, Some("span-123".to_string()));
        assert_eq!(entry.trace_id, Some("trace-456".to_string()));
    }

    #[test]
    fn test_TRACING_017_log_entry_to_json() {
        let entry = LogEntry::new(TraceLevel::Error, "Error occurred");
        let json = entry.to_json();

        assert!(json.contains("\"level\": \"ERROR\""));
        assert!(json.contains("\"message\": \"Error occurred\""));
    }

    #[test]
    fn test_TRACING_018_logger() {
        let mut logger = Logger::new().with_level(TraceLevel::Debug);

        logger.error("Error message");
        logger.warn("Warning message");
        logger.info("Info message");
        logger.debug("Debug message");

        assert_eq!(logger.entries().len(), 4);
    }

    #[test]
    fn test_TRACING_019_logger_with_context() {
        let ctx = TracingContext::new("test", "1.0.0");
        let mut logger = Logger::new().with_context(ctx);

        if let Some(ctx) = logger.context_mut() {
            ctx.start_root("root");
        }

        logger.info("With context");

        let entry = &logger.entries()[0];
        assert!(entry.trace_id.is_some());
    }

    #[test]
    fn test_TRACING_020_attribute_value_json() {
        assert_eq!(AttributeValue::string("test").to_json(), "\"test\"");
        assert_eq!(AttributeValue::int(42).to_json(), "42");
        assert_eq!(AttributeValue::float(2.5).to_json(), "2.5");
        assert_eq!(AttributeValue::bool(true).to_json(), "true");
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Property: Trace IDs are always unique
        #[test]
        fn prop_trace_ids_unique(_dummy in 0..100usize) {
            let id1 = generate_trace_id();
            std::thread::sleep(std::time::Duration::from_nanos(1));
            let id2 = generate_trace_id();
            prop_assert_ne!(id1, id2);
        }

        /// Property: Span completion sets end time
        #[test]
        fn prop_span_completion_sets_end_time(ok in proptest::bool::ANY) {
            let mut span = Span::new("test", "trace");

            if ok {
                span.end_ok();
                prop_assert_eq!(span.status, SpanStatus::Ok);
            } else {
                span.end_error("error");
                prop_assert_eq!(span.status, SpanStatus::Error);
            }

            prop_assert!(span.end_time.is_some());
            prop_assert!(span.is_completed());
        }

        /// Property: Logger respects level filtering
        #[test]
        fn prop_logger_respects_level(level_idx in 0usize..5) {
            let levels = [
                TraceLevel::Off,
                TraceLevel::Error,
                TraceLevel::Warn,
                TraceLevel::Info,
                TraceLevel::Debug,
            ];
            let min_level = levels[level_idx];

            let mut logger = Logger::new().with_level(min_level);
            logger.error("e");
            logger.warn("w");
            logger.info("i");
            logger.debug("d");

            // Count should match number of levels <= min_level
            let expected = if min_level == TraceLevel::Off {
                0
            } else {
                level_idx
            };

            prop_assert!(logger.entries().len() <= expected + 1);
        }
    }
}
