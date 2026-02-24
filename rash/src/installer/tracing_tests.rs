//! Coverage tests for installer/tracing.rs — targets uncovered branches in
//! TraceLevel, SpanStatus, SpanKind, AttributeValue arrays, Span builders,
//! SpanEvent JSON, TracingContext specialized spans, TraceSummary formatting,
//! LogEntry formatting, Logger filtering, and helper functions.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use super::*;

#[test]
fn test_TRACING_COV_001_trace_level_name_all_variants() {
    assert_eq!(TraceLevel::Off.name(), "OFF");
    assert_eq!(TraceLevel::Error.name(), "ERROR");
    assert_eq!(TraceLevel::Warn.name(), "WARN");
    assert_eq!(TraceLevel::Info.name(), "INFO");
    assert_eq!(TraceLevel::Debug.name(), "DEBUG");
    assert_eq!(TraceLevel::Trace.name(), "TRACE");
}

#[test]
fn test_TRACING_COV_002_trace_level_parse_alternates_and_edge() {
    assert_eq!(TraceLevel::parse("none"), Some(TraceLevel::Off));
    assert_eq!(TraceLevel::parse("warning"), Some(TraceLevel::Warn));
    assert_eq!(TraceLevel::parse("TRACE"), Some(TraceLevel::Trace));
    assert_eq!(TraceLevel::parse("DEBUG"), Some(TraceLevel::Debug));
    assert_eq!(TraceLevel::parse(""), None);
    assert_eq!(TraceLevel::parse("verbose"), None);
}

#[test]
fn test_TRACING_COV_003_trace_level_default_and_ord() {
    let level: TraceLevel = Default::default();
    assert_eq!(level, TraceLevel::Info);
    assert!(TraceLevel::Off < TraceLevel::Error);
    assert!(TraceLevel::Error < TraceLevel::Warn);
    assert!(TraceLevel::Warn < TraceLevel::Info);
    assert!(TraceLevel::Info < TraceLevel::Debug);
    assert!(TraceLevel::Debug < TraceLevel::Trace);
}

#[test]
fn test_TRACING_COV_004_trace_level_should_log_boundaries() {
    assert!(TraceLevel::Off.should_log(TraceLevel::Off));
    assert!(!TraceLevel::Off.should_log(TraceLevel::Error));
    assert!(TraceLevel::Error.should_log(TraceLevel::Error));
    assert!(!TraceLevel::Error.should_log(TraceLevel::Warn));
    assert!(TraceLevel::Trace.should_log(TraceLevel::Trace));
    assert!(!TraceLevel::Debug.should_log(TraceLevel::Trace));
}

#[test]
fn test_TRACING_COV_005_span_status_and_kind_names() {
    assert_eq!(SpanStatus::Unset.name(), "UNSET");
    assert_eq!(SpanStatus::Ok.name(), "OK");
    assert_eq!(SpanStatus::Error.name(), "ERROR");
    assert_eq!(SpanStatus::default(), SpanStatus::Unset);
    assert_eq!(SpanKind::Internal.name(), "INTERNAL");
    assert_eq!(SpanKind::Server.name(), "SERVER");
    assert_eq!(SpanKind::Client.name(), "CLIENT");
    assert_eq!(SpanKind::Producer.name(), "PRODUCER");
    assert_eq!(SpanKind::Consumer.name(), "CONSUMER");
    assert_eq!(SpanKind::default(), SpanKind::Internal);
}

#[test]
fn test_TRACING_COV_006_attribute_value_array_json() {
    let sa = AttributeValue::StringArray(vec!["hello".into(), "world".into()]);
    assert_eq!(sa.to_json(), r#"["hello", "world"]"#);
    let ia = AttributeValue::IntArray(vec![1, 2, 3]);
    assert_eq!(ia.to_json(), "[1, 2, 3]");
    assert_eq!(AttributeValue::StringArray(vec![]).to_json(), "[]");
    assert_eq!(AttributeValue::IntArray(vec![]).to_json(), "[]");
}

#[test]
fn test_TRACING_COV_007_attribute_value_special_chars_and_constructors() {
    let val = AttributeValue::string("line1\nline2\t\"quoted\"");
    let json = val.to_json();
    assert!(json.contains("\\n") && json.contains("\\t") && json.contains("\\\""));
    assert_eq!(AttributeValue::float(3.14).to_json(), "3.14");
    assert_eq!(AttributeValue::bool(false).to_json(), "false");
    assert!(matches!(AttributeValue::string("x"), AttributeValue::String(s) if s == "x"));
    assert!(matches!(AttributeValue::int(99), AttributeValue::Int(99)));
}

#[test]
fn test_TRACING_COV_008_span_builders() {
    let span = Span::new("child", "t1").with_parent("p99");
    assert_eq!(span.parent_id, Some("p99".to_string()));
    let span2 = Span::new("srv", "t1").with_kind(SpanKind::Server);
    assert_eq!(span2.kind, SpanKind::Server);
    let span3 = Span::new("test", "t1");
    assert!(span3.duration().is_none());
    assert!(!span3.is_completed());
}

#[test]
fn test_TRACING_COV_009_span_add_event_with_attrs() {
    let mut span = Span::new("test", "t1");
    let mut attrs = HashMap::new();
    attrs.insert("key".to_string(), AttributeValue::string("val"));
    span.add_event_with_attrs("my_event", attrs);
    assert_eq!(span.events.len(), 1);
    assert_eq!(span.events[0].name, "my_event");
    assert!(span.events[0].attributes.contains_key("key"));
}

#[test]
fn test_TRACING_COV_010_span_to_json_branches() {
    // With parent_id
    let s1 = Span::new("child", "t1").with_parent("p42");
    assert!(s1.to_json().contains("\"parentSpanId\": \"p42\""));
    // With events
    let mut s2 = Span::new("test", "t1");
    s2.add_event("evt1");
    s2.add_event("evt2");
    let json2 = s2.to_json();
    assert!(json2.contains("\"events\"") && json2.contains("evt1") && json2.contains("evt2"));
    // No attributes, no events
    let s3 = Span::new("bare", "t1");
    let json3 = s3.to_json();
    assert!(!json3.contains("\"attributes\"") && !json3.contains("\"events\""));
    // With end time
    let mut s4 = Span::new("test", "t1");
    s4.end_ok();
    assert!(s4.to_json().contains("\"endTimeUnixNano\""));
}

#[test]
fn test_TRACING_COV_011_span_event_to_json() {
    let evt1 = SpanEvent::new("simple");
    let j1 = evt1.to_json();
    assert!(j1.contains("\"name\": \"simple\"") && !j1.contains("\"attributes\""));
    let mut attrs = HashMap::new();
    attrs.insert("k1".to_string(), AttributeValue::int(10));
    let evt2 = SpanEvent::with_attributes("detailed", attrs);
    let j2 = evt2.to_json();
    assert!(j2.contains("\"attributes\"") && j2.contains("\"k1\""));
    let evt3 = SpanEvent::new("event\twith\"special");
    assert!(evt3.to_json().contains("event\\twith\\\"special"));
}

#[test]
fn test_TRACING_COV_012_trace_exporter_default_and_variants() {
    assert!(matches!(TraceExporter::default(), TraceExporter::Stdout));
    let _f = TraceExporter::File(std::path::PathBuf::from("/tmp/t.json"));
    let _o = TraceExporter::Otlp("http://localhost:4317".into());
    let _m = TraceExporter::Memory;
}

#[test]
fn test_TRACING_COV_013_context_builder_methods() {
    let ctx = TracingContext::new("svc", "1.0")
        .with_level(TraceLevel::Debug)
        .with_exporter(TraceExporter::Memory);
    assert_eq!(ctx.service_name(), "svc");
    assert!(!ctx.trace_id().is_empty());
}

#[test]
fn test_TRACING_COV_014_context_artifact_span() {
    let mut ctx = TracingContext::new("svc", "1.0");
    ctx.start_root("installer");
    let id = ctx.start_artifact_span("art-abc");
    assert!(!id.is_empty());
    ctx.end_span_ok();
    let art = ctx.all_spans().into_iter().find(|s| s.name == "artifact:art-abc").unwrap();
    assert_eq!(art.kind, SpanKind::Client);
}

#[test]
fn test_TRACING_COV_015_context_pre_post_condition_spans() {
    let mut ctx = TracingContext::new("svc", "1.0");
    ctx.start_root("installer");
    let pre_id = ctx.start_precondition_span("os == linux");
    assert!(!pre_id.is_empty());
    ctx.end_span_ok();
    let post_id = ctx.start_postcondition_span("file exists");
    assert!(!post_id.is_empty());
    ctx.end_span_ok();
    let names: Vec<&str> = ctx.all_spans().iter().map(|s| s.name.as_str()).collect();
    assert!(names.contains(&"precondition"));
    assert!(names.contains(&"postcondition"));
}

#[test]
fn test_TRACING_COV_016_context_set_attribute_stack_vs_root_vs_none() {
    // On stack child
    let mut ctx = TracingContext::new("svc", "1.0");
    ctx.start_root("root");
    ctx.start_span("child");
    ctx.set_attribute("attr_key", AttributeValue::int(42));
    ctx.end_span_ok();
    let child = ctx.all_spans().into_iter().find(|s| s.name == "child").unwrap();
    assert!(child.attributes.contains_key("attr_key"));
    // On root (no child on stack)
    let mut ctx2 = TracingContext::new("svc", "1.0");
    ctx2.start_root("root");
    ctx2.set_attribute("root_attr", AttributeValue::string("val"));
    let root = ctx2.all_spans().into_iter().find(|s| s.name == "root").unwrap();
    assert!(root.attributes.contains_key("root_attr"));
    // No spans at all
    let mut ctx3 = TracingContext::new("svc", "1.0");
    ctx3.set_attribute("orphan", AttributeValue::bool(true));
    assert_eq!(ctx3.completed_count(), 0);
}

#[test]
fn test_TRACING_COV_017_context_add_event_stack_vs_root_vs_none() {
    let mut ctx = TracingContext::new("svc", "1.0");
    ctx.start_root("root");
    ctx.start_span("child");
    ctx.add_event("child_event");
    ctx.end_span_ok();
    let child = ctx.all_spans().into_iter().find(|s| s.name == "child").unwrap();
    assert_eq!(child.events.len(), 1);
    // On root
    let mut ctx2 = TracingContext::new("svc", "1.0");
    ctx2.start_root("root");
    ctx2.add_event("root_event");
    let root = ctx2.all_spans().into_iter().find(|s| s.name == "root").unwrap();
    assert_eq!(root.events.len(), 1);
    // No spans
    let mut ctx3 = TracingContext::new("svc", "1.0");
    ctx3.add_event("orphan");
    assert_eq!(ctx3.completed_count(), 0);
}

#[test]
fn test_TRACING_COV_018_context_end_root_error() {
    let mut ctx = TracingContext::new("svc", "1.0");
    ctx.start_root("root");
    ctx.end_root_error("fatal failure");
    let root = ctx.all_spans().into_iter().find(|s| s.name == "root").unwrap();
    assert_eq!(root.status, SpanStatus::Error);
    assert_eq!(root.status_message.as_deref(), Some("fatal failure"));
}

#[test]
fn test_TRACING_COV_019_context_end_span_error_and_no_stack() {
    let mut ctx = TracingContext::new("svc", "1.0");
    ctx.start_root("root");
    ctx.start_span("failing");
    ctx.end_span_error("step failed");
    let failing = ctx.all_spans().into_iter().find(|s| s.name == "failing").unwrap();
    assert_eq!(failing.status, SpanStatus::Error);
    // Empty stack no-ops
    let mut ctx2 = TracingContext::new("svc", "1.0");
    ctx2.end_span_ok();
    ctx2.end_span_error("no stack");
    assert_eq!(ctx2.completed_count(), 0);
}

#[test]
fn test_TRACING_COV_020_context_empty_state() {
    let ctx = TracingContext::new("svc", "1.0");
    assert!(ctx.current_span_id().is_none());
    assert!(ctx.all_spans().is_empty());
}

#[test]
fn test_TRACING_COV_021_context_end_root_no_root() {
    let mut ctx = TracingContext::new("svc", "1.0");
    ctx.end_root_ok();
    ctx.end_root_error("no root");
    assert!(ctx.all_spans().is_empty());
}

#[test]
fn test_TRACING_COV_022_trace_summary_format_seconds_and_minutes() {
    let s1 = TraceSummary {
        trace_id: "abc123def456".into(), total_spans: 5, ok_spans: 4,
        error_spans: 1, total_duration: Duration::from_secs(30),
    };
    let t1 = s1.format();
    assert!(t1.contains("30.00s") && t1.contains("5 total") && t1.contains("4 ok"));
    let s2 = TraceSummary {
        trace_id: "abc".into(), total_spans: 10, ok_spans: 10,
        error_spans: 0, total_duration: Duration::from_secs(125),
    };
    assert!(s2.format().contains("2m 05s"));
}

#[test]
fn test_TRACING_COV_023_trace_summary_truncates_id() {
    let s = TraceSummary {
        trace_id: "0123456789abcdef0123456789abcdef".into(),
        total_spans: 1, ok_spans: 1, error_spans: 0,
        total_duration: Duration::from_millis(500),
    };
    let t = s.format();
    assert!(!t.contains("0123456789abcdef0123456789abcdef"));
    assert!(t.contains("0123456789abc..."));
}

#[test]
fn test_TRACING_COV_024_log_entry_format_and_json() {
    let e1 = LogEntry::new(TraceLevel::Warn, "disk low");
    assert!(e1.format().contains("[WARN]") && e1.format().contains("disk low"));
    let e2 = LogEntry::new(TraceLevel::Error, "crash")
        .with_span("s1", "t1")
        .with_attr("code", AttributeValue::int(500));
    let j = e2.to_json();
    assert!(j.contains("\"spanId\": \"s1\"") && j.contains("\"traceId\": \"t1\""));
    assert!(j.contains("\"attributes\"") && j.contains("\"code\""));
    let e3 = LogEntry::new(TraceLevel::Info, "hello");
    assert!(!e3.to_json().contains("spanId"));
}

#[test]
fn test_TRACING_COV_025_logger_filtering_and_context() {
    let logger = Logger::default();
    assert!(logger.entries().is_empty() && logger.context().is_none());
    let mut l_off = Logger::new().with_level(TraceLevel::Off);
    l_off.error("e"); l_off.warn("w"); l_off.info("i"); l_off.debug("d");
    assert_eq!(l_off.entries().len(), 0);
    let mut l_err = Logger::new().with_level(TraceLevel::Error);
    l_err.error("e"); l_err.warn("w"); l_err.info("i");
    assert_eq!(l_err.entries().len(), 1);
    // With context wiring
    let ctx = TracingContext::new("svc", "1.0");
    let mut l_ctx = Logger::new().with_context(ctx);
    if let Some(c) = l_ctx.context_mut() { c.start_root("root"); c.start_span("active"); }
    l_ctx.info("traced");
    assert!(l_ctx.entries()[0].span_id.is_some() && l_ctx.entries()[0].trace_id.is_some());
}

#[test]
fn test_TRACING_COV_026_escape_json_and_truncate() {
    assert_eq!(escape_json(r#"a\b"#), r#"a\\b"#);
    assert_eq!(escape_json("a\"b"), "a\\\"b");
    assert_eq!(escape_json("a\nb"), "a\\nb");
    assert_eq!(escape_json("a\rb"), "a\\rb");
    assert_eq!(escape_json("a\tb"), "a\\tb");
    assert_eq!(escape_json(""), "");
    assert_eq!(truncate("", 5), "");
    assert_eq!(truncate("abc", 3), "abc");
    assert_eq!(truncate("abcd", 3), "...");
    assert_eq!(truncate("abcdefg", 6), "abc...");
}

#[test]
fn test_TRACING_COV_027_generate_id_and_trace_id() {
    let id = generate_id();
    assert!(!id.is_empty() && id.chars().all(|c| c.is_ascii_hexdigit()));
    let tid = generate_trace_id();
    assert_eq!(tid.len(), 32);
    assert!(tid.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_TRACING_COV_028_time_helpers() {
    assert!(time_to_nanos(SystemTime::now()) > 0);
    let ts = format_timestamp(SystemTime::now());
    let parts: Vec<&str> = ts.split('.').collect();
    assert_eq!(parts.len(), 2);
    assert_eq!(parts[1].len(), 3);
}

#[test]
fn test_TRACING_COV_029_context_full_export() {
    let mut ctx = TracingContext::new("my-installer", "2.0.0");
    ctx.start_root("full-install");
    ctx.start_artifact_span("binary-v1"); ctx.end_span_ok();
    ctx.start_precondition_span("os == linux"); ctx.end_span_ok();
    ctx.start_step_span("install-pkg", "Install Package"); ctx.end_span_ok();
    ctx.start_postcondition_span("binary exists"); ctx.end_span_ok();
    ctx.end_root_ok();
    let json = ctx.export();
    assert!(json.contains("my-installer") && json.contains("resourceSpans"));
    let summary = ctx.summary();
    assert_eq!(summary.total_spans, 5);
    assert_eq!(summary.ok_spans, 5);
}

#[test]
fn test_TRACING_COV_030_context_summary_no_root() {
    let ctx = TracingContext::new("svc", "1.0");
    let s = ctx.summary();
    assert_eq!(s.total_spans, 0);
    assert_eq!(s.total_duration, Duration::ZERO);
}

#[test]
fn test_TRACING_COV_031_string_array_special_chars() {
    let val = AttributeValue::StringArray(vec!["has\"quote".into(), "has\nnewline".into()]);
    let j = val.to_json();
    assert!(j.contains("has\\\"quote") && j.contains("has\\nnewline"));
}

#[test]
fn test_TRACING_COV_032_context_orphan_span() {
    let mut ctx = TracingContext::new("svc", "1.0");
    let id = ctx.start_span("orphan");
    assert!(!id.is_empty());
    ctx.end_span_ok();
    assert_eq!(ctx.completed_count(), 1);
}
