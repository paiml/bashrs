// Circular Trace Buffer
//
// Task: TRACE-002 - Circular buffer implementation
// Source: bashrs-tracing-while-purify-lint-spec.md (Section 2.3)
// Research: Science of Computer Programming 2024 (Near-Omniscient Debugging)
//
// Design:
// - Fixed-size circular buffer (default: 1024 events)
// - Oldest events evicted when full
// - Target: 80%+ bug coverage with 1024 events
// - Performance: <10% overhead
//
// Trade-off: Memory vs coverage
// - 256 events: ~50% coverage, 2 MB memory
// - 512 events: ~65% coverage, 4 MB memory
// - 1024 events: ~80% coverage, 8 MB memory (default)
// - 2048 events: ~90% coverage, 16 MB memory
// - 4096 events: ~95% coverage, 32 MB memory

use crate::tracing::{TraceEvent, TraceSignificance};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Circular trace buffer with fixed capacity
///
/// When capacity is reached, oldest events are automatically evicted (FIFO).
/// This maintains constant memory usage while providing recent event history.
///
/// Inspired by: Science of Computer Programming 2024
/// - Circular buffers achieve 80%+ coverage with 1024 events
/// - Minimal memory footprint (<10 MB)
/// - Constant-time operations (O(1) push/pop)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircularTraceBuffer {
    /// Event storage (circular FIFO queue)
    events: VecDeque<TraceEvent>,

    /// Maximum capacity (default: 1024)
    capacity: usize,

    /// Total events recorded (including evicted)
    total_events: u64,

    /// Number of events evicted due to capacity limit
    evicted_count: u64,
}

impl CircularTraceBuffer {
    /// Create a new circular trace buffer with default capacity (1024)
    ///
    /// # Example
    /// ```
    /// use rash::tracing::CircularTraceBuffer;
    ///
    /// let buffer = CircularTraceBuffer::new();
    /// assert_eq!(buffer.capacity(), 1024);
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::with_capacity(1024)
    }

    /// Create a new circular trace buffer with custom capacity
    ///
    /// # Recommended capacities
    /// - 256: Low memory (~2 MB), ~50% coverage
    /// - 512: Medium memory (~4 MB), ~65% coverage
    /// - 1024: Balanced (~8 MB), ~80% coverage (default)
    /// - 2048: High memory (~16 MB), ~90% coverage
    /// - 4096: Very high memory (~32 MB), ~95% coverage
    ///
    /// # Example
    /// ```
    /// use rash::tracing::CircularTraceBuffer;
    ///
    /// let buffer = CircularTraceBuffer::with_capacity(2048);
    /// assert_eq!(buffer.capacity(), 2048);
    /// ```
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            events: VecDeque::with_capacity(capacity),
            capacity,
            total_events: 0,
            evicted_count: 0,
        }
    }

    /// Push a new event into the buffer
    ///
    /// If buffer is at capacity, the oldest event is evicted (FIFO).
    ///
    /// # Example
    /// ```
    /// use rash::tracing::{CircularTraceBuffer, TraceEvent, ParseEvent, Span};
    ///
    /// let mut buffer = CircularTraceBuffer::with_capacity(2);
    /// buffer.push(TraceEvent::Parse(ParseEvent::ParseStart {
    ///     source: "test.sh".to_string(),
    ///     line: 1,
    ///     col: 1,
    /// }));
    /// assert_eq!(buffer.len(), 1);
    /// ```
    pub fn push(&mut self, event: TraceEvent) {
        if self.events.len() == self.capacity {
            self.events.pop_front(); // Evict oldest
            self.evicted_count += 1;
        }
        self.events.push_back(event);
        self.total_events += 1;
    }

    /// Get the number of events currently in the buffer
    #[must_use]
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Check if the buffer is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Get the maximum capacity of this buffer
    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.capacity
    }

    /// Get total events recorded (including evicted)
    #[must_use]
    pub const fn total_events(&self) -> u64 {
        self.total_events
    }

    /// Get number of events evicted due to capacity limit
    #[must_use]
    pub const fn evicted_count(&self) -> u64 {
        self.evicted_count
    }

    /// Calculate retention rate (% of events retained)
    ///
    /// Returns 1.0 (100%) if no events evicted, otherwise ratio of
    /// retained/total events.
    ///
    /// # Example
    /// ```
    /// use rash::tracing::CircularTraceBuffer;
    ///
    /// let buffer = CircularTraceBuffer::new();
    /// assert_eq!(buffer.retention_rate(), 1.0); // Empty buffer, 100%
    /// ```
    #[must_use]
    pub fn retention_rate(&self) -> f64 {
        if self.total_events == 0 {
            return 1.0;
        }
        (self.total_events - self.evicted_count) as f64 / self.total_events as f64
    }

    /// Get an iterator over events in chronological order (oldest first)
    pub fn iter(&self) -> impl Iterator<Item = &TraceEvent> {
        self.events.iter()
    }

    /// Get events filtered by minimum significance level
    ///
    /// # Example
    /// ```
    /// use rash::tracing::{CircularTraceBuffer, TraceSignificance};
    ///
    /// let buffer = CircularTraceBuffer::new();
    /// let high_events: Vec<_> = buffer
    ///     .iter_filtered(TraceSignificance::High)
    ///     .collect();
    /// // Returns only HIGH + CRITICAL events
    /// ```
    pub fn iter_filtered(
        &self,
        min_significance: TraceSignificance,
    ) -> impl Iterator<Item = &TraceEvent> + '_ {
        self.events
            .iter()
            .filter(move |e| e.significance() >= min_significance)
    }

    /// Clear all events from the buffer
    ///
    /// Resets total_events and evicted_count to 0.
    pub fn clear(&mut self) {
        self.events.clear();
        self.total_events = 0;
        self.evicted_count = 0;
    }

    /// Get buffer utilization (% full)
    ///
    /// Returns value between 0.0 and 1.0.
    #[must_use]
    pub fn utilization(&self) -> f64 {
        self.events.len() as f64 / self.capacity as f64
    }

    /// Check if buffer is at capacity
    #[must_use]
    pub fn is_full(&self) -> bool {
        self.events.len() == self.capacity
    }

    /// Get buffer statistics
    #[must_use]
    pub fn stats(&self) -> BufferStats {
        BufferStats {
            capacity: self.capacity,
            current_size: self.events.len(),
            total_events: self.total_events,
            evicted_count: self.evicted_count,
            retention_rate: self.retention_rate(),
            utilization: self.utilization(),
        }
    }
}

impl Default for CircularTraceBuffer {
    fn default() -> Self {
        Self::new()
    }
}

/// Buffer statistics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BufferStats {
    pub capacity: usize,
    pub current_size: usize,
    pub total_events: u64,
    pub evicted_count: u64,
    pub retention_rate: f64,
    pub utilization: f64,
}

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
