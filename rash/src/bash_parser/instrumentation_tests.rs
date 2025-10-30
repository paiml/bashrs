//! Instrumentation Tests - Verify tracing integration
//!
//! Task: TRACE-003 - Parser instrumentation tests
//! Tests that BashParser correctly emits trace events when tracer is enabled

#[cfg(test)]
mod tests {
    use crate::bash_parser::BashParser;
    use crate::tracing::{ParseEvent, TraceEvent, TraceManager, TraceSignificance};

    /// Test: TRACE-003-100 - Parser without tracer has zero overhead
    #[test]
    fn test_trace_003_100_parser_without_tracer() {
        let script = "x=5\necho $x";
        let mut parser = BashParser::new(script).expect("Parse failed");

        let ast = parser.parse().expect("Parse failed");
        assert_eq!(ast.statements.len(), 2);
    }

    /// Test: TRACE-003-101 - Parser with tracer emits ParseStart
    #[test]
    fn test_trace_003_101_parser_emits_parse_start() {
        let script = "x=5";
        let tracer = TraceManager::new();
        let mut parser = BashParser::new(script)
            .expect("Parse failed")
            .with_tracer(tracer.clone());

        let _ast = parser.parse().expect("Parse failed");

        let snapshot = tracer.snapshot();
        assert!(snapshot.len() > 0, "Expected trace events");

        // First event should be ParseStart
        let events: Vec<_> = snapshot.iter().collect();
        match events[0] {
            TraceEvent::Parse(ParseEvent::ParseStart { .. }) => {
                // Success
            }
            _ => panic!("Expected ParseStart event, got {:?}", events[0]),
        }
    }

    /// Test: TRACE-003-102 - Parser emits ParseNode for each statement
    #[test]
    fn test_trace_003_102_parser_emits_parse_nodes() {
        let script = "x=5\ny=10\necho $x";
        let tracer = TraceManager::new();
        let mut parser = BashParser::new(script)
            .expect("Parse failed")
            .with_tracer(tracer.clone());

        let ast = parser.parse().expect("Parse failed");
        assert_eq!(ast.statements.len(), 3);

        let snapshot = tracer.snapshot();

        // Count ParseNode events
        let parse_node_count = snapshot
            .iter()
            .filter(|e| matches!(e, TraceEvent::Parse(ParseEvent::ParseNode { .. })))
            .count();

        assert_eq!(
            parse_node_count, 3,
            "Expected 3 ParseNode events for 3 statements"
        );
    }

    /// Test: TRACE-003-103 - Parser emits ParseComplete with correct count
    #[test]
    fn test_trace_003_103_parser_emits_parse_complete() {
        let script = "x=5\ny=10";
        let tracer = TraceManager::new();
        let mut parser = BashParser::new(script)
            .expect("Parse failed")
            .with_tracer(tracer.clone());

        let _ast = parser.parse().expect("Parse failed");

        let snapshot = tracer.snapshot();
        let events: Vec<_> = snapshot.iter().collect();

        // Last event should be ParseComplete
        match events.last() {
            Some(TraceEvent::Parse(ParseEvent::ParseComplete { node_count, .. })) => {
                assert_eq!(*node_count, 2, "Expected 2 nodes parsed");
            }
            _ => panic!("Expected ParseComplete as last event"),
        }
    }

    /// Test: TRACE-003-104 - Parser emits ParseError on syntax error
    #[test]
    fn test_trace_003_104_parser_emits_parse_error() {
        let script = "if then fi"; // Malformed if statement
        let tracer = TraceManager::new();
        let mut parser = BashParser::new(script)
            .expect("Lexer succeeded")
            .with_tracer(tracer.clone());

        let result = parser.parse();
        assert!(result.is_err(), "Expected parse error");

        let snapshot = tracer.snapshot();

        // Should have ParseStart and ParseError
        let has_parse_start = snapshot
            .iter()
            .any(|e| matches!(e, TraceEvent::Parse(ParseEvent::ParseStart { .. })));
        let has_parse_error = snapshot
            .iter()
            .any(|e| matches!(e, TraceEvent::Parse(ParseEvent::ParseError { .. })));

        assert!(has_parse_start, "Expected ParseStart event");
        assert!(has_parse_error, "Expected ParseError event");
    }

    /// Test: TRACE-003-105 - ParseEvent significance levels
    #[test]
    fn test_trace_003_105_parse_event_significance() {
        let script = "x=5";
        let tracer = TraceManager::new();
        let mut parser = BashParser::new(script)
            .expect("Parse failed")
            .with_tracer(tracer.clone());

        let _ast = parser.parse().expect("Parse failed");

        let snapshot = tracer.snapshot();

        for event in snapshot.iter() {
            let sig = event.significance();
            match event {
                TraceEvent::Parse(ParseEvent::ParseStart { .. })
                | TraceEvent::Parse(ParseEvent::ParseComplete { .. }) => {
                    assert_eq!(
                        sig,
                        TraceSignificance::Trace,
                        "Start/Complete should be Trace"
                    );
                }
                TraceEvent::Parse(ParseEvent::ParseNode { .. }) => {
                    assert_eq!(sig, TraceSignificance::Low, "ParseNode should be Low");
                }
                TraceEvent::Parse(ParseEvent::ParseError { .. }) => {
                    assert_eq!(sig, TraceSignificance::High, "ParseError should be High");
                }
                _ => {}
            }
        }
    }

    /// Test: TRACE-003-106 - Filtered iteration by significance
    #[test]
    fn test_trace_003_106_filtered_iteration() {
        let script = "x=5\ny=10\necho $x";
        let tracer = TraceManager::new();
        let mut parser = BashParser::new(script)
            .expect("Parse failed")
            .with_tracer(tracer.clone());

        let _ast = parser.parse().expect("Parse failed");

        let snapshot = tracer.snapshot();

        // Filter for HIGH significance (should exclude ParseNode, ParseStart, ParseComplete)
        let high_events: Vec<_> = snapshot.iter_filtered(TraceSignificance::High).collect();

        // With no parse errors, should have 0 HIGH events
        assert_eq!(high_events.len(), 0, "No HIGH events without errors");

        // Filter for TRACE significance (should include everything)
        let all_events: Vec<_> = snapshot.iter_filtered(TraceSignificance::Trace).collect();

        assert!(all_events.len() >= 5, "Expected at least 5 total events");
    }

    /// Test: TRACE-003-107 - Complex script instrumentation
    #[test]
    fn test_trace_003_107_complex_script() {
        let script = r#"x=5
y=10
z=15
if [ "$x" -eq 5 ]; then
  echo "x is 5"
fi"#;
        let tracer = TraceManager::new();
        let mut parser = BashParser::new(script)
            .expect("Parse failed")
            .with_tracer(tracer.clone());

        let ast = parser.parse().expect("Parse failed");

        let snapshot = tracer.snapshot();

        // Should have:
        // - 1 ParseStart
        // - N ParseNode events (one per statement)
        // - 1 ParseComplete
        let parse_start_count = snapshot
            .iter()
            .filter(|e| matches!(e, TraceEvent::Parse(ParseEvent::ParseStart { .. })))
            .count();
        let parse_node_count = snapshot
            .iter()
            .filter(|e| matches!(e, TraceEvent::Parse(ParseEvent::ParseNode { .. })))
            .count();
        let parse_complete_count = snapshot
            .iter()
            .filter(|e| matches!(e, TraceEvent::Parse(ParseEvent::ParseComplete { .. })))
            .count();

        assert_eq!(parse_start_count, 1, "Expected 1 ParseStart");
        assert_eq!(
            parse_node_count,
            ast.statements.len(),
            "Expected ParseNode for each statement"
        );
        assert_eq!(parse_complete_count, 1, "Expected 1 ParseComplete");
    }

    /// Test: TRACE-003-108 - node_type() helper method
    #[test]
    fn test_trace_003_108_node_type_helper() {
        let script = "x=5\necho hello\nif [ -f test ]; then\n  echo ok\nfi";
        let mut parser = BashParser::new(script).expect("Parse failed");
        let ast = parser.parse().expect("Parse failed");

        assert_eq!(ast.statements[0].node_type(), "Assignment");
        assert_eq!(ast.statements[1].node_type(), "Command");
        assert_eq!(ast.statements[2].node_type(), "If");
    }

    /// Test: TRACE-003-109 - Tracer statistics
    #[test]
    fn test_trace_003_109_tracer_statistics() {
        let script = "x=5\ny=10";
        let tracer = TraceManager::new();
        let mut parser = BashParser::new(script)
            .expect("Parse failed")
            .with_tracer(tracer.clone());

        let _ast = parser.parse().expect("Parse failed");

        let stats = tracer.stats();
        assert!(stats.enabled, "Tracer should be enabled");
        assert!(stats.current_events > 0, "Should have events");
        assert_eq!(
            stats.total_events, stats.current_events as u64,
            "Total should equal current (no evictions)"
        );
        assert_eq!(stats.retention_rate, 1.0, "Should have 100% retention");
    }
}
