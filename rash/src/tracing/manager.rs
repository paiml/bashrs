// Trace Manager - Coordinates tracing across parse/purify/lint/generate phases
//
// Task: TRACE-003 - Instrumentation coordinator
// Design: Opt-in tracing to minimize overhead when not needed
//
// Philosophy:
// - Zero overhead when tracing disabled (static dispatch, no allocations)
// - Thread-safe for future parallel processing
// - Clean separation: modules emit events, manager coordinates storage

use super::{CircularTraceBuffer, ParseEvent, TraceEvent};
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Trace manager coordinates event collection across all phases
///
/// Design goals:
/// - Opt-in: No overhead when tracing disabled
/// - Thread-safe: Use Arc<Mutex<>> for future parallel processing
/// - Clean API: emit_parse(), emit_purify(), emit_lint(), emit_generate()
#[derive(Clone)]
pub struct TraceManager {
    buffer: Arc<Mutex<CircularTraceBuffer>>,
    enabled: bool,
    start_time: Instant,
}

impl TraceManager {
    /// Create a new trace manager with default buffer size (1024 events)
    pub fn new() -> Self {
        Self {
            buffer: Arc::new(Mutex::new(CircularTraceBuffer::new())),
            enabled: true,
            start_time: Instant::now(),
        }
    }

    /// Create a trace manager with custom buffer capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: Arc::new(Mutex::new(CircularTraceBuffer::with_capacity(capacity))),
            enabled: true,
            start_time: Instant::now(),
        }
    }

    /// Create a disabled trace manager (zero overhead)
    pub fn disabled() -> Self {
        Self {
            buffer: Arc::new(Mutex::new(CircularTraceBuffer::new())),
            enabled: false,
            start_time: Instant::now(),
        }
    }

    /// Check if tracing is enabled
    #[must_use]
    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Emit a parse event
    ///
    /// Only records if tracing is enabled. Zero overhead when disabled.
    pub fn emit_parse(&self, event: ParseEvent) {
        if !self.enabled {
            return;
        }

        if let Ok(mut buffer) = self.buffer.lock() {
            buffer.push(TraceEvent::Parse(event));
        }
    }

    /// Get a snapshot of the current trace buffer
    ///
    /// Returns a cloned buffer for analysis without holding the lock
    #[must_use]
    pub fn snapshot(&self) -> CircularTraceBuffer {
        self.buffer
            .lock()
            .ok()
            .map(|b| b.clone())
            .unwrap_or_else(CircularTraceBuffer::new)
    }

    /// Get the number of events currently in the buffer
    #[must_use]
    pub fn event_count(&self) -> usize {
        self.buffer
            .lock()
            .ok()
            .map_or(0, |b| b.len())
    }

    /// Get elapsed time since trace manager was created
    #[must_use]
    pub fn elapsed(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }

    /// Clear all events in the buffer
    pub fn clear(&self) {
        if let Ok(mut buffer) = self.buffer.lock() {
            buffer.clear();
        }
    }

    /// Get buffer statistics (retention rate, capacity, etc.)
    #[must_use]
    pub fn stats(&self) -> TraceStats {
        self.buffer
            .lock()
            .ok()
            .map(|b| TraceStats {
                current_events: b.len(),
                total_events: b.total_events(),
                evicted_events: b.evicted_count(),
                capacity: b.capacity(),
                retention_rate: b.retention_rate(),
                enabled: self.enabled,
                elapsed_ms: self.elapsed().as_millis() as u64,
            })
            .unwrap_or_default()
    }
}

impl Default for TraceManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about the trace buffer
#[derive(Debug, Clone, PartialEq)]
pub struct TraceStats {
    pub current_events: usize,
    pub total_events: u64,
    pub evicted_events: u64,
    pub capacity: usize,
    pub retention_rate: f64,
    pub enabled: bool,
    pub elapsed_ms: u64,
}

impl Default for TraceStats {
    fn default() -> Self {
        Self {
            current_events: 0,
            total_events: 0,
            evicted_events: 0,
            capacity: 1024,
            retention_rate: 1.0,
            enabled: false,
            elapsed_ms: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tracing::Span;

    /// Test: TRACE-003-001 - TraceManager creation
    #[test]
    fn test_trace_003_001_manager_creation() {
        let manager = TraceManager::new();
        assert!(manager.is_enabled());
        assert_eq!(manager.event_count(), 0);
    }

    /// Test: TRACE-003-002 - TraceManager with custom capacity
    #[test]
    fn test_trace_003_002_custom_capacity() {
        let manager = TraceManager::with_capacity(512);
        assert!(manager.is_enabled());

        let stats = manager.stats();
        assert_eq!(stats.capacity, 512);
    }

    /// Test: TRACE-003-003 - Disabled trace manager (zero overhead)
    #[test]
    fn test_trace_003_003_disabled_manager() {
        let manager = TraceManager::disabled();
        assert!(!manager.is_enabled());

        // Emit event should be no-op
        manager.emit_parse(ParseEvent::ParseStart {
            source: "test.sh".to_string(),
            line: 1,
            col: 1,
        });

        assert_eq!(manager.event_count(), 0);
    }

    /// Test: TRACE-003-004 - Emit parse events
    #[test]
    fn test_trace_003_004_emit_parse_events() {
        let manager = TraceManager::new();

        manager.emit_parse(ParseEvent::ParseStart {
            source: "test.sh".to_string(),
            line: 1,
            col: 1,
        });

        manager.emit_parse(ParseEvent::ParseNode {
            node_type: "Command".to_string(),
            span: Span::single_line(1, 1, 10),
        });

        assert_eq!(manager.event_count(), 2);
    }

    /// Test: TRACE-003-005 - Snapshot returns cloned buffer
    #[test]
    fn test_trace_003_005_snapshot() {
        let manager = TraceManager::new();

        manager.emit_parse(ParseEvent::ParseStart {
            source: "test.sh".to_string(),
            line: 1,
            col: 1,
        });

        let snapshot = manager.snapshot();
        assert_eq!(snapshot.len(), 1);

        // Adding more events doesn't affect snapshot
        manager.emit_parse(ParseEvent::ParseNode {
            node_type: "Command".to_string(),
            span: Span::single_line(1, 1, 10),
        });

        assert_eq!(snapshot.len(), 1); // Snapshot unchanged
        assert_eq!(manager.event_count(), 2); // Manager updated
    }

    /// Test: TRACE-003-006 - Clear events
    #[test]
    fn test_trace_003_006_clear() {
        let manager = TraceManager::new();

        manager.emit_parse(ParseEvent::ParseStart {
            source: "test.sh".to_string(),
            line: 1,
            col: 1,
        });

        assert_eq!(manager.event_count(), 1);

        manager.clear();
        assert_eq!(manager.event_count(), 0);
    }

    /// Test: TRACE-003-007 - Statistics
    #[test]
    fn test_trace_003_007_statistics() {
        let manager = TraceManager::with_capacity(2);

        manager.emit_parse(ParseEvent::ParseStart {
            source: "test.sh".to_string(),
            line: 1,
            col: 1,
        });

        manager.emit_parse(ParseEvent::ParseNode {
            node_type: "Command".to_string(),
            span: Span::single_line(1, 1, 10),
        });

        manager.emit_parse(ParseEvent::ParseNode {
            node_type: "If".to_string(),
            span: Span::single_line(2, 1, 5),
        });

        let stats = manager.stats();
        assert_eq!(stats.capacity, 2);
        assert_eq!(stats.current_events, 2);
        assert_eq!(stats.total_events, 3);
        assert_eq!(stats.evicted_events, 1);
        assert!(stats.enabled);
    }

    /// Test: TRACE-003-008 - Thread safety (concurrent access)
    #[test]
    fn test_trace_003_008_thread_safety() {
        use std::thread;

        let manager = TraceManager::new();
        let manager_clone = manager.clone();

        let handle = thread::spawn(move || {
            for i in 0..10 {
                manager_clone.emit_parse(ParseEvent::ParseStart {
                    source: format!("test{}.sh", i),
                    line: 1,
                    col: 1,
                });
            }
        });

        for i in 0..10 {
            manager.emit_parse(ParseEvent::ParseStart {
                source: format!("main{}.sh", i),
                line: 1,
                col: 1,
            });
        }

        handle.join().unwrap();

        // Should have 20 events from both threads
        assert_eq!(manager.event_count(), 20);
    }
}
