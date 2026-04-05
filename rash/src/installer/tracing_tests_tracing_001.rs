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
