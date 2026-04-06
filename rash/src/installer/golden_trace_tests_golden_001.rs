
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
