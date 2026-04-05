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

include!("tracing_incl2.rs");
