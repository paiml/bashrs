// REPL Debugger Module
//
// Task: REPL-008-001 - Step execution (next line)
// Test Approach: RED → GREEN → REFACTOR → PROPERTY → MUTATION
//
// Quality targets:
// - Unit tests: 5+ scenarios
// - Property tests: Step never skips lines
// - Mutation score: ≥90%
// - Complexity: <10 per function

use crate::repl::{Breakpoint, BreakpointManager};
use std::collections::HashMap;

/// Debug session for step-by-step execution of bash scripts
///
/// Tracks the current execution state including:
/// - Script lines
/// - Current line number
/// - Breakpoints
/// - Variables
#[derive(Debug, Clone)]
pub struct DebugSession {
    /// Lines of the script being debugged
    lines: Vec<String>,

    /// Current line number (0-indexed)
    current_line: usize,

    /// Breakpoint manager
    breakpoints: BreakpointManager,

    /// Session variables
    variables: HashMap<String, String>,

    /// Whether execution is complete
    finished: bool,
}

impl DebugSession {
    /// Create a new debug session from script content
    pub fn new(script: &str) -> Self {
        let lines: Vec<String> = script.lines().map(|l| l.to_string()).collect();

        Self {
            lines,
            current_line: 0,
            breakpoints: BreakpointManager::new(),
            variables: HashMap::new(),
            finished: false,
        }
    }

    /// Get the current line number (1-indexed for user display)
    pub fn current_line(&self) -> usize {
        self.current_line + 1
    }

    /// Get the current line content
    pub fn current_line_content(&self) -> Option<&str> {
        self.lines.get(self.current_line).map(|s| s.as_str())
    }

    /// Check if execution is finished
    pub fn is_finished(&self) -> bool {
        self.finished
    }

    /// Get total number of lines
    pub fn total_lines(&self) -> usize {
        self.lines.len()
    }

    /// Execute one line (step)
    ///
    /// Executes the current line and advances to the next line.
    /// Returns the output from executing the line.
    ///
    /// # Returns
    /// - `Some(output)` if a line was executed
    /// - `None` if execution is finished
    pub fn step(&mut self) -> Option<String> {
        // Check if finished
        if self.finished || self.current_line >= self.lines.len() {
            self.finished = true;
            return None;
        }

        // Get current line
        let line = self.lines[self.current_line].clone();

        // Execute the line (simplified - just echo for now)
        let output = format!("Executed: {}", line);

        // Advance to next line
        self.current_line += 1;

        // Check if finished
        if self.current_line >= self.lines.len() {
            self.finished = true;
        }

        Some(output)
    }

    /// Set a breakpoint at the specified line (1-indexed)
    pub fn set_breakpoint(&mut self, line: usize) -> bool {
        if line == 0 || line > self.lines.len() {
            return false;
        }
        self.breakpoints.set_breakpoint(line)
    }

    /// Check if current line has a breakpoint
    pub fn at_breakpoint(&self) -> bool {
        self.breakpoints.is_breakpoint_hit(self.current_line())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== UNIT TESTS (RED PHASE) =====

    /// Test: REPL-008-001-001 - Create debug session from script
    #[test]
    fn test_REPL_008_001_create_session() {
        let script = "echo hello\necho world";
        let session = DebugSession::new(script);

        assert_eq!(session.current_line(), 1, "Should start at line 1");
        assert_eq!(session.total_lines(), 2, "Should have 2 lines");
        assert!(!session.is_finished(), "Should not be finished initially");
    }

    /// Test: REPL-008-001-002 - Step through single line
    #[test]
    fn test_REPL_008_001_step_single_line() {
        let script = "echo hello";
        let mut session = DebugSession::new(script);

        // Step once
        let output = session.step();
        assert!(output.is_some(), "Should execute the line");
        assert!(output.unwrap().contains("echo hello"), "Should show executed line");

        // Should be finished after one line
        assert!(session.is_finished(), "Should be finished after single line");

        // Stepping again should return None
        assert!(session.step().is_none(), "Should return None when finished");
    }

    /// Test: REPL-008-001-003 - Step through multiple lines
    #[test]
    fn test_REPL_008_001_step_multiple_lines() {
        let script = "echo line1\necho line2\necho line3";
        let mut session = DebugSession::new(script);

        // Step 1
        assert_eq!(session.current_line(), 1);
        session.step();

        // Step 2
        assert_eq!(session.current_line(), 2);
        session.step();

        // Step 3
        assert_eq!(session.current_line(), 3);
        session.step();

        // Should be finished
        assert!(session.is_finished());
    }

    /// Test: REPL-008-001-004 - Get current line content
    #[test]
    fn test_REPL_008_001_current_line_content() {
        let script = "first line\nsecond line";
        let session = DebugSession::new(script);

        assert_eq!(session.current_line_content(), Some("first line"));
    }

    /// Test: REPL-008-001-005 - Breakpoint at current line
    #[test]
    fn test_REPL_008_001_breakpoint_integration() {
        let script = "echo line1\necho line2\necho line3";
        let mut session = DebugSession::new(script);

        // Set breakpoint at line 2
        assert!(session.set_breakpoint(2), "Should set breakpoint at line 2");

        // Step to line 1 (no breakpoint)
        assert!(!session.at_breakpoint(), "Line 1 should not have breakpoint");
        session.step();

        // Now at line 2 (has breakpoint)
        assert!(session.at_breakpoint(), "Line 2 should have breakpoint");
    }

    /// Test: REPL-008-001-006 - Invalid breakpoint line
    #[test]
    fn test_REPL_008_001_invalid_breakpoint() {
        let script = "echo hello";
        let mut session = DebugSession::new(script);

        // Try to set breakpoint at line 0 (invalid)
        assert!(!session.set_breakpoint(0), "Should reject line 0");

        // Try to set breakpoint beyond script length
        assert!(!session.set_breakpoint(999), "Should reject line beyond script");
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    // ===== PROPERTY TESTS (PROPERTY PHASE) =====

    /// Property: Stepping never skips lines
    proptest! {
        #[test]
        fn prop_step_never_skips_lines(num_lines in 1usize..20) {
            // Create a script with N lines
            let script = (0..num_lines)
                .map(|i| format!("echo line{}", i))
                .collect::<Vec<_>>()
                .join("\n");

            let mut session = DebugSession::new(&script);

            // Step through all lines
            for expected_line in 1..=num_lines {
                prop_assert_eq!(
                    session.current_line(),
                    expected_line,
                    "Should be at line {} before step",
                    expected_line
                );

                if expected_line < num_lines {
                    prop_assert!(!session.is_finished(), "Should not be finished");
                }

                session.step();
            }

            // Should be finished
            prop_assert!(session.is_finished(), "Should be finished after all lines");
        }
    }

    /// Property: Current line is always valid
    proptest! {
        #[test]
        fn prop_current_line_always_valid(num_lines in 1usize..20, steps in 0usize..25) {
            let script = (0..num_lines)
                .map(|i| format!("line{}", i))
                .collect::<Vec<_>>()
                .join("\n");

            let mut session = DebugSession::new(&script);

            // Step N times
            for _ in 0..steps {
                let line = session.current_line();
                prop_assert!(line >= 1, "Line number should be >= 1");
                prop_assert!(line <= num_lines + 1, "Line number should be reasonable");

                if session.is_finished() {
                    break;
                }
                session.step();
            }
        }
    }

    /// Property: Total lines never changes
    proptest! {
        #[test]
        fn prop_total_lines_constant(num_lines in 1usize..20, steps in 0usize..25) {
            let script = (0..num_lines)
                .map(|i| format!("line{}", i))
                .collect::<Vec<_>>()
                .join("\n");

            let mut session = DebugSession::new(&script);
            let initial_total = session.total_lines();

            // Step multiple times
            for _ in 0..steps {
                prop_assert_eq!(
                    session.total_lines(),
                    initial_total,
                    "Total lines should never change"
                );

                if session.is_finished() {
                    break;
                }
                session.step();
            }
        }
    }
}
