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

    /// Continue execution until a breakpoint is hit or execution finishes
    ///
    /// Executes lines repeatedly using step() until:
    /// - A breakpoint is encountered (returns BreakpointHit with line number)
    /// - Execution completes (returns Finished)
    ///
    /// # Returns
    /// - `ContinueResult::BreakpointHit(line)` if stopped at breakpoint
    /// - `ContinueResult::Finished` if execution completed
    pub fn continue_execution(&mut self) -> ContinueResult {
        loop {
            // Check if at breakpoint before executing
            if self.at_breakpoint() {
                return ContinueResult::BreakpointHit(self.current_line());
            }

            // Execute one step
            match self.step() {
                Some(_output) => {
                    // Line executed, check if we hit a breakpoint
                    if self.at_breakpoint() {
                        return ContinueResult::BreakpointHit(self.current_line());
                    }
                }
                None => {
                    // Execution finished
                    return ContinueResult::Finished;
                }
            }
        }
    }
}

/// Result of continue execution
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContinueResult {
    /// Stopped at a breakpoint on the specified line (1-indexed)
    BreakpointHit(usize),
    /// Execution completed without hitting breakpoint
    Finished,
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

    // ===== REPL-008-003: CONTINUE EXECUTION TESTS =====

    /// Test: REPL-008-003-001 - Continue to breakpoint
    #[test]
    fn test_REPL_008_003_continue_to_breakpoint() {
        let script = "echo line1\necho line2\necho line3\necho line4";
        let mut session = DebugSession::new(script);

        // Set breakpoint at line 3
        session.set_breakpoint(3);

        // Continue execution - should stop at line 3
        let result = session.continue_execution();
        assert_eq!(
            result,
            ContinueResult::BreakpointHit(3),
            "Should stop at breakpoint on line 3"
        );
        assert_eq!(session.current_line(), 3, "Current line should be 3");
    }

    /// Test: REPL-008-003-002 - Continue to end (no breakpoints)
    #[test]
    fn test_REPL_008_003_continue_to_end() {
        let script = "echo line1\necho line2\necho line3";
        let mut session = DebugSession::new(script);

        // No breakpoints - should run to completion
        let result = session.continue_execution();
        assert_eq!(
            result,
            ContinueResult::Finished,
            "Should finish execution without breakpoints"
        );
        assert!(session.is_finished(), "Session should be finished");
    }

    /// Test: REPL-008-003-003 - Continue past first breakpoint
    #[test]
    fn test_REPL_008_003_continue_multiple_breakpoints() {
        let script = "echo line1\necho line2\necho line3\necho line4\necho line5";
        let mut session = DebugSession::new(script);

        // Set breakpoints at lines 2 and 4
        session.set_breakpoint(2);
        session.set_breakpoint(4);

        // First continue - stop at line 2
        let result1 = session.continue_execution();
        assert_eq!(result1, ContinueResult::BreakpointHit(2));
        assert_eq!(session.current_line(), 2);

        // Step over the breakpoint
        session.step();

        // Second continue - stop at line 4
        let result2 = session.continue_execution();
        assert_eq!(result2, ContinueResult::BreakpointHit(4));
        assert_eq!(session.current_line(), 4);
    }

    /// Test: REPL-008-003-004 - Continue when already at breakpoint
    #[test]
    fn test_REPL_008_003_continue_at_breakpoint() {
        let script = "echo line1\necho line2\necho line3";
        let mut session = DebugSession::new(script);

        // Set breakpoint at line 1 (starting position)
        session.set_breakpoint(1);

        // Continue should immediately return (already at breakpoint)
        let result = session.continue_execution();
        assert_eq!(
            result,
            ContinueResult::BreakpointHit(1),
            "Should detect we're already at breakpoint"
        );
        assert_eq!(session.current_line(), 1);
    }

    /// Test: REPL-008-003-005 - Continue from middle of script
    #[test]
    fn test_REPL_008_003_continue_from_middle() {
        let script = "echo line1\necho line2\necho line3\necho line4";
        let mut session = DebugSession::new(script);

        // Step to line 2
        session.step();
        assert_eq!(session.current_line(), 2);

        // Set breakpoint at line 4
        session.set_breakpoint(4);

        // Continue from line 2 to line 4
        let result = session.continue_execution();
        assert_eq!(result, ContinueResult::BreakpointHit(4));
        assert_eq!(session.current_line(), 4);
    }

    /// Test: REPL-008-003-006 - Continue past last breakpoint to end
    #[test]
    fn test_REPL_008_003_continue_past_breakpoint_to_end() {
        let script = "echo line1\necho line2\necho line3";
        let mut session = DebugSession::new(script);

        // Set breakpoint at line 2
        session.set_breakpoint(2);

        // First continue - stop at breakpoint
        assert_eq!(session.continue_execution(), ContinueResult::BreakpointHit(2));

        // Step past breakpoint
        session.step();

        // Second continue - run to end
        assert_eq!(session.continue_execution(), ContinueResult::Finished);
        assert!(session.is_finished());
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

    // ===== REPL-008-003: CONTINUE PROPERTY TESTS =====

    /// Property: Continue without breakpoints always finishes
    proptest! {
        #[test]
        fn prop_continue_no_breakpoints_finishes(num_lines in 1usize..20) {
            let script = (0..num_lines)
                .map(|i| format!("echo line{}", i))
                .collect::<Vec<_>>()
                .join("\n");

            let mut session = DebugSession::new(&script);

            // Continue without breakpoints should always finish
            let result = session.continue_execution();
            prop_assert_eq!(result, ContinueResult::Finished, "Should finish without breakpoints");
            prop_assert!(session.is_finished(), "Session should be finished");
        }
    }

    /// Property: Continue always stops at breakpoint
    proptest! {
        #[test]
        fn prop_continue_stops_at_breakpoint(
            num_lines in 2usize..20,
            breakpoint_line in 1usize..19
        ) {
            let script = (0..num_lines)
                .map(|i| format!("echo line{}", i))
                .collect::<Vec<_>>()
                .join("\n");

            let mut session = DebugSession::new(&script);

            // Only test if breakpoint is within script
            if breakpoint_line <= num_lines {
                session.set_breakpoint(breakpoint_line);
                let result = session.continue_execution();

                match result {
                    ContinueResult::BreakpointHit(line) => {
                        prop_assert_eq!(line, breakpoint_line, "Should stop at correct breakpoint");
                    }
                    ContinueResult::Finished => {
                        // This should not happen if breakpoint is valid
                        prop_assert!(false, "Should not finish if breakpoint exists");
                    }
                }
            }
        }
    }

    /// Property: Continue result is deterministic
    proptest! {
        #[test]
        fn prop_continue_deterministic(
            num_lines in 1usize..20,
            has_breakpoint in proptest::bool::ANY,
            breakpoint_line in 1usize..19
        ) {
            let script = (0..num_lines)
                .map(|i| format!("echo line{}", i))
                .collect::<Vec<_>>()
                .join("\n");

            // Run twice with same setup
            let mut session1 = DebugSession::new(&script);
            let mut session2 = DebugSession::new(&script);

            if has_breakpoint && breakpoint_line <= num_lines {
                session1.set_breakpoint(breakpoint_line);
                session2.set_breakpoint(breakpoint_line);
            }

            let result1 = session1.continue_execution();
            let result2 = session2.continue_execution();

            prop_assert_eq!(result1, result2, "Same setup should produce same result");
        }
    }

    /// Property: Multiple continues eventually finish
    proptest! {
        #[test]
        fn prop_multiple_continues_finish(num_lines in 1usize..10) {
            let script = (0..num_lines)
                .map(|i| format!("echo line{}", i))
                .collect::<Vec<_>>()
                .join("\n");

            let mut session = DebugSession::new(&script);

            // Set breakpoints at every line
            for line in 1..=num_lines {
                session.set_breakpoint(line);
            }

            // Continue multiple times, eventually should finish
            let mut iterations = 0;
            let max_iterations = num_lines + 5;  // Safety limit

            loop {
                let result = session.continue_execution();
                match result {
                    ContinueResult::Finished => break,
                    ContinueResult::BreakpointHit(_) => {
                        // Step past breakpoint and continue
                        session.step();
                    }
                }

                iterations += 1;
                if iterations > max_iterations {
                    prop_assert!(false, "Too many iterations, should have finished");
                    break;
                }
            }

            prop_assert!(session.is_finished(), "Should eventually finish");
        }
    }
}
