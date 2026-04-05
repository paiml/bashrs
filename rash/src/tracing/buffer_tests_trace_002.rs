#[cfg(test)]
mod tests {
    use super::*;
    use crate::tracing::{ParseEvent, Span};

    // ===== UNIT TESTS (RED PHASE) =====

    /// Test: TRACE-002-001 - Buffer creation with default capacity
    #[test]
    fn test_trace_002_001_default_capacity() {
        let buffer = CircularTraceBuffer::new();
        assert_eq!(buffer.capacity(), 1024);
        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());
    }

    /// Test: TRACE-002-002 - Buffer creation with custom capacity
    #[test]
    fn test_trace_002_002_custom_capacity() {
        let buffer = CircularTraceBuffer::with_capacity(512);
        assert_eq!(buffer.capacity(), 512);
    }

    /// Test: TRACE-002-003 - Push event into buffer
    #[test]
    fn test_trace_002_003_push_event() {
        let mut buffer = CircularTraceBuffer::with_capacity(2);
        buffer.push(TraceEvent::Parse(ParseEvent::ParseStart {
            source: "test.sh".to_string(),
            line: 1,
            col: 1,
        }));
        assert_eq!(buffer.len(), 1);
        assert_eq!(buffer.total_events(), 1);
    }

    /// Test: TRACE-002-004 - Eviction when capacity reached
    #[test]
    fn test_trace_002_004_eviction() {
        let mut buffer = CircularTraceBuffer::with_capacity(2);

        // Push 3 events (capacity is 2, so oldest will be evicted)
        buffer.push(TraceEvent::Parse(ParseEvent::ParseStart {
            source: "test1.sh".to_string(),
            line: 1,
            col: 1,
        }));
        buffer.push(TraceEvent::Parse(ParseEvent::ParseStart {
            source: "test2.sh".to_string(),
            line: 1,
            col: 1,
        }));
        buffer.push(TraceEvent::Parse(ParseEvent::ParseStart {
            source: "test3.sh".to_string(),
            line: 1,
            col: 1,
        }));

        assert_eq!(buffer.len(), 2); // Still at capacity
        assert_eq!(buffer.total_events(), 3); // But 3 events recorded
        assert_eq!(buffer.evicted_count(), 1); // 1 evicted
    }

    /// Test: TRACE-002-005 - Retention rate calculation
    #[test]
    fn test_trace_002_005_retention_rate() {
        let mut buffer = CircularTraceBuffer::with_capacity(2);

        // Empty buffer: 100% retention
        assert_eq!(buffer.retention_rate(), 1.0);

        // Push 2 events: 100% retention (no evictions)
        buffer.push(TraceEvent::Parse(ParseEvent::ParseStart {
            source: "test1.sh".to_string(),
            line: 1,
            col: 1,
        }));
        buffer.push(TraceEvent::Parse(ParseEvent::ParseStart {
            source: "test2.sh".to_string(),
            line: 1,
            col: 1,
        }));
        assert_eq!(buffer.retention_rate(), 1.0);

        // Push 1 more: 66.7% retention (1 evicted out of 3 total)
        buffer.push(TraceEvent::Parse(ParseEvent::ParseStart {
            source: "test3.sh".to_string(),
            line: 1,
            col: 1,
        }));
        assert!((buffer.retention_rate() - 0.6667).abs() < 0.01);
    }

    /// Test: TRACE-002-006 - Buffer utilization
    #[test]
    fn test_trace_002_006_utilization() {
        let mut buffer = CircularTraceBuffer::with_capacity(4);
        assert_eq!(buffer.utilization(), 0.0);

        buffer.push(TraceEvent::Parse(ParseEvent::ParseStart {
            source: "test.sh".to_string(),
            line: 1,
            col: 1,
        }));
        assert_eq!(buffer.utilization(), 0.25);

        buffer.push(TraceEvent::Parse(ParseEvent::ParseStart {
            source: "test.sh".to_string(),
            line: 1,
            col: 1,
        }));
        assert_eq!(buffer.utilization(), 0.5);
    }

    /// Test: TRACE-002-007 - is_full check
    #[test]
    fn test_trace_002_007_is_full() {
        let mut buffer = CircularTraceBuffer::with_capacity(2);
        assert!(!buffer.is_full());

        buffer.push(TraceEvent::Parse(ParseEvent::ParseStart {
            source: "test.sh".to_string(),
            line: 1,
            col: 1,
        }));
        assert!(!buffer.is_full());

        buffer.push(TraceEvent::Parse(ParseEvent::ParseStart {
            source: "test.sh".to_string(),
            line: 1,
            col: 1,
        }));
        assert!(buffer.is_full());
    }

    /// Test: TRACE-002-008 - Clear buffer
    #[test]
    fn test_trace_002_008_clear() {
        let mut buffer = CircularTraceBuffer::with_capacity(2);
        buffer.push(TraceEvent::Parse(ParseEvent::ParseStart {
            source: "test.sh".to_string(),
            line: 1,
            col: 1,
        }));
        buffer.push(TraceEvent::Parse(ParseEvent::ParseStart {
            source: "test.sh".to_string(),
            line: 1,
            col: 1,
        }));

        assert_eq!(buffer.len(), 2);
        buffer.clear();
        assert_eq!(buffer.len(), 0);
        assert_eq!(buffer.total_events(), 0);
    }

    /// Test: TRACE-002-009 - Iterator over events
    #[test]
    fn test_trace_002_009_iterator() {
        let mut buffer = CircularTraceBuffer::with_capacity(3);
        buffer.push(TraceEvent::Parse(ParseEvent::ParseStart {
            source: "test1.sh".to_string(),
            line: 1,
            col: 1,
        }));
        buffer.push(TraceEvent::Parse(ParseEvent::ParseStart {
            source: "test2.sh".to_string(),
            line: 1,
            col: 1,
        }));

        let events: Vec<_> = buffer.iter().collect();
        assert_eq!(events.len(), 2);
    }

    /// Test: TRACE-002-010 - Filtered iterator by significance
    #[test]
    fn test_trace_002_010_filtered_iterator() {
        let mut buffer = CircularTraceBuffer::with_capacity(10);

        // Add events with different significances
        buffer.push(TraceEvent::Parse(ParseEvent::ParseStart {
            source: "test.sh".to_string(),
            line: 1,
            col: 1,
        })); // Trace significance

        buffer.push(TraceEvent::Parse(ParseEvent::ParseError {
            error: "Syntax error".to_string(),
            span: Span::single_line(1, 1, 10),
        })); // High significance

        // Filter for HIGH+ events
        let high_events: Vec<_> = buffer.iter_filtered(TraceSignificance::High).collect();
        assert_eq!(high_events.len(), 1);
    }

    /// Test: TRACE-002-011 - Buffer statistics
    #[test]
    fn test_trace_002_011_stats() {
        let mut buffer = CircularTraceBuffer::with_capacity(2);
        buffer.push(TraceEvent::Parse(ParseEvent::ParseStart {
            source: "test.sh".to_string(),
            line: 1,
            col: 1,
        }));

        let stats = buffer.stats();
        assert_eq!(stats.capacity, 2);
        assert_eq!(stats.current_size, 1);
        assert_eq!(stats.total_events, 1);
        assert_eq!(stats.utilization, 0.5);
    }

    /// Test: TRACE-002-012 - Serialization round-trip
    #[test]
    fn test_trace_002_012_serialization() {
        let mut buffer = CircularTraceBuffer::with_capacity(2);
        buffer.push(TraceEvent::Parse(ParseEvent::ParseStart {
            source: "test.sh".to_string(),
            line: 1,
            col: 1,
        }));

        let json = serde_json::to_string(&buffer).expect("Serialization failed");
        let deserialized: CircularTraceBuffer =
            serde_json::from_str(&json).expect("Deserialization failed");

        assert_eq!(buffer.len(), deserialized.len());
        assert_eq!(buffer.capacity(), deserialized.capacity());
    }

    // ===== PROPERTY TESTS =====

    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        // Property: Buffer never exceeds capacity
        proptest! {
            #[test]
            fn prop_trace_002_never_exceeds_capacity(
                capacity in 1usize..1000,
                push_count in 1usize..2000,
            ) {
                let mut buffer = CircularTraceBuffer::with_capacity(capacity);

                for i in 0..push_count {
                    buffer.push(TraceEvent::Parse(ParseEvent::ParseStart {
                        source: format!("test{}.sh", i),
                        line: 1,
                        col: 1,
                    }));
                }

                // Property: Buffer never exceeds capacity
                prop_assert!(buffer.len() <= capacity);
            }
        }

        // Property: Retention rate is always between 0.0 and 1.0
        proptest! {
            #[test]
            fn prop_trace_002_retention_rate_bounds(
                capacity in 1usize..100,
                push_count in 1usize..200,
            ) {
                let mut buffer = CircularTraceBuffer::with_capacity(capacity);

                for i in 0..push_count {
                    buffer.push(TraceEvent::Parse(ParseEvent::ParseStart {
                        source: format!("test{}.sh", i),
                        line: 1,
                        col: 1,
                    }));
                }

                let rate = buffer.retention_rate();
                // Property: Retention rate in [0.0, 1.0]
                prop_assert!((0.0..=1.0).contains(&rate));
            }
        }

        // Property: Total events always >= current size
        proptest! {
            #[test]
            fn prop_trace_002_total_vs_current(
                capacity in 1usize..100,
                push_count in 1usize..200,
            ) {
                let mut buffer = CircularTraceBuffer::with_capacity(capacity);

                for i in 0..push_count {
                    buffer.push(TraceEvent::Parse(ParseEvent::ParseStart {
                        source: format!("test{}.sh", i),
                        line: 1,
                        col: 1,
                    }));
                }

                // Property: Total events >= current size (monotonic)
                prop_assert!(buffer.total_events() >= buffer.len() as u64);
            }
        }
    }
}
