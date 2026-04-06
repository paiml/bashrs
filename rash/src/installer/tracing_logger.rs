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
#[path = "tracing_tests_tracing_001.rs"]
mod tests_extracted;
