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

use crate::repl::{purify_bash, BreakpointManager};
use std::collections::HashMap;

/// A stack frame in the call stack
///
/// Represents a function call or execution context with its name
/// and the line number where it was called from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StackFrame {
    /// Name of the function or context
    pub name: String,
    /// Line number where this frame was called (1-indexed)
    pub line: usize,
}

impl StackFrame {
    /// Create a new stack frame
    pub fn new(name: impl Into<String>, line: usize) -> Self {
        Self {
            name: name.into(),
            line,
        }
    }
}

/// Line comparison result for purification-aware debugging
///
/// Compares original bash line with its purified version,
/// enabling the debugger to show what transformations were applied.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineComparison {
    /// Original bash line
    pub original: String,
    /// Purified version of the line
    pub purified: String,
    /// Whether the lines differ
    pub differs: bool,
}

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

    /// Call stack for tracking function calls
    call_stack: Vec<StackFrame>,

    /// Purified version of the script lines (if purification succeeded)
    purified_lines: Option<Vec<String>>,

    /// Whether execution is complete
    finished: bool,
}

impl DebugSession {
    /// Create a new debug session from script content
    pub fn new(script: &str) -> Self {
        let lines: Vec<String> = script.lines().map(|l| l.to_string()).collect();

        // Initialize call stack with main frame
        let mut call_stack = Vec::new();
        call_stack.push(StackFrame::new("<main>", 0));

        // Attempt to purify the script for comparison
        let purified_lines = purify_bash(script)
            .ok()
            .map(|purified| purified.lines().map(|l| l.to_string()).collect());

        Self {
            lines,
            current_line: 0,
            breakpoints: BreakpointManager::new(),
            variables: HashMap::new(),
            call_stack,
            purified_lines,
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

    /// Get the current call depth (number of stack frames)
    ///
    /// Returns the size of the call stack. Depth 1 = main frame only.
    pub fn call_depth(&self) -> usize {
        self.call_stack.len()
    }

    /// Execute to next line at same call depth (skip over function calls)
    ///
    /// Similar to step(), but if a function call is encountered, it executes
    /// the entire function and stops at the next line at the same depth.
    ///
    /// For now, this is simplified to just call step() since we don't have
    /// full function call tracking yet. Future enhancement will track actual
    /// function boundaries.
    ///
    /// # Returns
    /// - `Some(output)` if a line was executed
    /// - `None` if execution is finished
    pub fn next(&mut self) -> Option<String> {
        // Simplified implementation: For now, just call step()
        // Future: Track call depth and skip over function calls
        self.step()
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

    // === Variable Inspection Methods (REPL-009-001) ===

    /// Set a variable value
    ///
    /// Updates or creates a variable in the session's variable store.
    ///
    /// # Arguments
    /// * `name` - Variable name
    /// * `value` - Variable value
    pub fn set_variable(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.variables.insert(name.into(), value.into());
    }

    /// Get a variable value
    ///
    /// Retrieves the value of a variable if it exists.
    ///
    /// # Arguments
    /// * `name` - Variable name to look up
    ///
    /// # Returns
    /// - `Some(&str)` if variable exists
    /// - `None` if variable does not exist
    pub fn get_variable(&self, name: &str) -> Option<&str> {
        self.variables.get(name).map(|s| s.as_str())
    }

    /// List all variables
    ///
    /// Returns a vector of (name, value) tuples for all variables.
    /// Variables are sorted by name for consistency.
    ///
    /// # Returns
    /// Vector of (variable_name, variable_value) tuples
    pub fn list_variables(&self) -> Vec<(&str, &str)> {
        let mut vars: Vec<(&str, &str)> = self
            .variables
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        vars.sort_by_key(|(name, _)| *name);
        vars
    }

    /// Get the count of variables
    ///
    /// # Returns
    /// Number of variables currently stored
    pub fn variable_count(&self) -> usize {
        self.variables.len()
    }

    /// Clear all variables
    ///
    /// Removes all variables from the session
    pub fn clear_variables(&mut self) {
        self.variables.clear();
    }

    // === Environment Display Methods (REPL-009-002) ===

    /// Get an environment variable value
    ///
    /// Retrieves the value of an environment variable from the process environment.
    ///
    /// # Arguments
    /// * `name` - Environment variable name to look up
    ///
    /// # Returns
    /// - `Some(String)` if environment variable exists
    /// - `None` if environment variable does not exist
    pub fn get_env(&self, name: &str) -> Option<String> {
        std::env::var(name).ok()
    }

    /// List all environment variables
    ///
    /// Returns a vector of (name, value) tuples for all environment variables.
    /// Variables are sorted by name for consistency.
    ///
    /// # Returns
    /// Vector of (variable_name, variable_value) tuples, sorted by name
    pub fn list_env(&self) -> Vec<(String, String)> {
        let mut env_vars: Vec<(String, String)> = std::env::vars().collect();
        env_vars.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));
        env_vars
    }

    /// Filter environment variables by prefix
    ///
    /// Returns environment variables whose names start with the given prefix.
    /// Results are sorted by name for consistency.
    ///
    /// # Arguments
    /// * `prefix` - Prefix to filter by (case-sensitive)
    ///
    /// # Returns
    /// Vector of (variable_name, variable_value) tuples matching the prefix
    pub fn filter_env(&self, prefix: &str) -> Vec<(String, String)> {
        let mut filtered: Vec<(String, String)> = std::env::vars()
            .filter(|(name, _)| name.starts_with(prefix))
            .collect();
        filtered.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));
        filtered
    }

    // === Call Stack Methods (REPL-009-003) ===

    /// Push a new frame onto the call stack
    ///
    /// Adds a new execution context (function call) to the call stack.
    ///
    /// # Arguments
    /// * `name` - Name of the function or context
    /// * `line` - Line number where this frame was called (1-indexed)
    pub fn push_frame(&mut self, name: impl Into<String>, line: usize) {
        self.call_stack.push(StackFrame::new(name, line));
    }

    /// Pop the most recent frame from the call stack
    ///
    /// Removes the top frame from the call stack (when returning from a function).
    /// Does nothing if only the main frame remains.
    pub fn pop_frame(&mut self) {
        // Keep at least the main frame
        if self.call_stack.len() > 1 {
            self.call_stack.pop();
        }
    }

    /// Get the current call stack
    ///
    /// Returns a reference to the call stack showing all active frames.
    /// The first frame is always <main>, and subsequent frames represent
    /// nested function calls.
    ///
    /// # Returns
    /// Vector of stack frames, ordered from oldest (bottom) to newest (top)
    pub fn call_stack(&self) -> &[StackFrame] {
        &self.call_stack
    }

    // ===== REPL-010: Purification-Aware Debugging Methods =====

    /// Compare current line with its purified version
    ///
    /// Returns a comparison showing the original line and its purified version,
    /// or None if purification failed or line is out of bounds.
    ///
    /// # Returns
    /// LineComparison with original, purified, and whether they differ
    pub fn compare_current_line(&self) -> Option<LineComparison> {
        // Get original line
        let original = self.lines.get(self.current_line)?.clone();

        // Get purified version if available
        let purified_lines = self.purified_lines.as_ref()?;
        let purified = purified_lines.get(self.current_line)?.clone();

        // Compare
        let differs = original != purified;

        Some(LineComparison {
            original,
            purified,
            differs,
        })
    }

    /// Format diff highlighting for a line comparison
    ///
    /// Creates a visual diff showing the differences between original
    /// and purified versions.
    ///
    /// # Arguments
    /// * `comparison` - The line comparison to format
    ///
    /// # Returns
    /// Formatted string showing the diff with highlighting
    /// Format diff with enhanced highlighting and transformation explanations
    ///
    /// Detects common bash purification patterns and explains what changed:
    /// - mkdir → mkdir -p (idempotency)
    /// - $var → "$var" (quoting)
    /// - ln -s → ln -sf (idempotency)
    /// - rm → rm -f (idempotency)
    pub fn format_diff_highlighting(&self, comparison: &LineComparison) -> String {
        if !comparison.differs {
            return format!("  {}\n(no changes)", comparison.original);
        }

        let orig = &comparison.original;
        let purified = &comparison.purified;

        // Detect transformation types
        let mut explanations = Vec::new();

        // Check for mkdir -p (idempotency flag)
        if orig.contains("mkdir") && purified.contains("mkdir -p") && !orig.contains("mkdir -p") {
            explanations.push("added idempotency flag -p to mkdir");
        }

        // Check for rm -f (idempotency flag)
        if orig.contains("rm") && purified.contains("rm -f") && !orig.contains("rm -f") {
            explanations.push("added idempotency flag -f to rm");
        }

        // Check for ln -sf (idempotency flags)
        if orig.contains("ln") {
            if purified.contains("-sf") && !orig.contains("-sf") {
                explanations.push("added idempotency flag -f to ln");
            } else if purified.contains("-f") && !orig.contains("-f") {
                explanations.push("added idempotency flag -f");
            }
        }

        // Check for variable quoting (safety)
        // Simple heuristic: look for $VAR → "$VAR" pattern
        if purified.contains("\"$") && !orig.contains("\"$") {
            explanations.push("added safety quoting around variables");
        }

        // Format output with explanations
        let mut output = format!("- {}\n+ {}", orig, purified);

        if !explanations.is_empty() {
            output.push_str("\n(");
            output.push_str(&explanations.join(", "));
            output.push(')');
        }

        output
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
        assert!(
            output.unwrap().contains("echo hello"),
            "Should show executed line"
        );

        // Should be finished after one line
        assert!(
            session.is_finished(),
            "Should be finished after single line"
        );

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
        assert!(
            !session.at_breakpoint(),
            "Line 1 should not have breakpoint"
        );
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
        assert!(
            !session.set_breakpoint(999),
            "Should reject line beyond script"
        );
    }

    // ===== REPL-008-002: NEXT COMMAND TESTS (SKIP OVER FUNCTIONS) =====

    /// Test: REPL-008-002-001 - Next at same level (simple statements)
    ///
    /// RED Phase: This test will FAIL because next() method doesn't exist yet
    #[test]
    fn test_REPL_008_002_next_same_level() {
        // ARRANGE: Script with simple statements (no functions)
        let script = "echo line1\necho line2\necho line3";
        let mut session = DebugSession::new(script);

        // ACT: Call next() from line 1
        assert_eq!(session.current_line(), 1, "Should start at line 1");
        session.next();

        // ASSERT: Should be at line 2 (next line at same depth)
        assert_eq!(
            session.current_line(),
            2,
            "Should be at line 2 after next()"
        );
        assert!(!session.is_finished(), "Should not be finished");
    }

    /// Test: REPL-008-002-002 - Next advances to completion
    ///
    /// RED Phase: This test will FAIL because next() method doesn't exist yet
    #[test]
    fn test_REPL_008_002_next_to_end() {
        // ARRANGE: Single line script
        let script = "echo hello";
        let mut session = DebugSession::new(script);

        // ACT: Call next() - should complete execution
        session.next();

        // ASSERT: Should be finished
        assert!(session.is_finished(), "Should be finished after next()");
    }

    /// Test: REPL-008-002-003 - Next multiple times
    ///
    /// RED Phase: This test will FAIL because next() method doesn't exist yet
    #[test]
    fn test_REPL_008_002_next_multiple() {
        // ARRANGE: Three line script
        let script = "echo line1\necho line2\necho line3";
        let mut session = DebugSession::new(script);

        // ACT: Next through all lines
        assert_eq!(session.current_line(), 1);
        session.next();

        assert_eq!(session.current_line(), 2);
        session.next();

        assert_eq!(session.current_line(), 3);
        session.next();

        // ASSERT: Should be finished
        assert!(session.is_finished(), "Should be finished after 3 next() calls");
    }

    /// Test: REPL-008-002-004 - Next when already finished
    ///
    /// RED Phase: This test will FAIL because next() method doesn't exist yet
    #[test]
    fn test_REPL_008_002_next_when_finished() {
        // ARRANGE: Single line script
        let script = "echo hello";
        let mut session = DebugSession::new(script);

        // ACT: Next to completion
        session.next();
        assert!(session.is_finished());

        // ACT: Try next() again when finished
        session.next();

        // ASSERT: Should still be finished
        assert!(
            session.is_finished(),
            "Should remain finished after next() on completed session"
        );
    }

    /// Test: REPL-008-002-005 - Next with call depth tracking
    ///
    /// RED Phase: This test will FAIL because next() method doesn't exist yet
    /// Note: Simplified version - just verify call_depth() accessor exists
    #[test]
    fn test_REPL_008_002_call_depth_accessor() {
        // ARRANGE
        let script = "echo test";
        let session = DebugSession::new(script);

        // ACT & ASSERT: Verify call_depth() method exists
        // Initial depth should be 1 (main frame)
        assert_eq!(
            session.call_depth(),
            1,
            "Initial call depth should be 1 (main frame)"
        );
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
        assert_eq!(
            session.continue_execution(),
            ContinueResult::BreakpointHit(2)
        );

        // Step past breakpoint
        session.step();

        // Second continue - run to end
        assert_eq!(session.continue_execution(), ContinueResult::Finished);
        assert!(session.is_finished());
    }

    // ===== REPL-009-001: VARIABLE INSPECTION TESTS =====

    /// Test: REPL-009-001-001 - Set and get variable
    #[test]
    fn test_REPL_009_001_print_variable() {
        let script = "echo hello";
        let mut session = DebugSession::new(script);

        // Set a variable
        session.set_variable("USER", "alice");
        session.set_variable("HOME", "/home/alice");

        // Get variable values
        assert_eq!(session.get_variable("USER"), Some("alice"));
        assert_eq!(session.get_variable("HOME"), Some("/home/alice"));

        // Variable count
        assert_eq!(session.variable_count(), 2);
    }

    /// Test: REPL-009-001-002 - Array-like variables (stored as comma-separated)
    #[test]
    fn test_REPL_009_001_print_array() {
        let script = "echo test";
        let mut session = DebugSession::new(script);

        // Store array as comma-separated string (simplified array handling)
        session.set_variable("ARRAY", "item1,item2,item3");

        // Retrieve array
        let array_value = session.get_variable("ARRAY");
        assert_eq!(array_value, Some("item1,item2,item3"));

        // Could be split by caller if needed
        let items: Vec<&str> = array_value.unwrap().split(',').collect();
        assert_eq!(items, vec!["item1", "item2", "item3"]);
    }

    /// Test: REPL-009-001-003 - Nonexistent variable returns None
    #[test]
    fn test_REPL_009_001_print_nonexistent() {
        let script = "echo test";
        let session = DebugSession::new(script);

        // Get nonexistent variable
        assert_eq!(session.get_variable("DOES_NOT_EXIST"), None);
        assert_eq!(session.get_variable(""), None);
    }

    /// Test: REPL-009-001-004 - List all variables
    #[test]
    fn test_REPL_009_001_list_variables() {
        let script = "echo test";
        let mut session = DebugSession::new(script);

        // Initially empty
        assert_eq!(session.list_variables(), vec![]);

        // Add variables
        session.set_variable("PATH", "/usr/bin");
        session.set_variable("USER", "bob");
        session.set_variable("HOME", "/home/bob");

        // List variables (sorted by name)
        let vars = session.list_variables();
        assert_eq!(vars.len(), 3);
        assert_eq!(vars[0], ("HOME", "/home/bob"));
        assert_eq!(vars[1], ("PATH", "/usr/bin"));
        assert_eq!(vars[2], ("USER", "bob"));
    }

    /// Test: REPL-009-001-005 - Variable update
    #[test]
    fn test_REPL_009_001_variable_update() {
        let script = "echo test";
        let mut session = DebugSession::new(script);

        // Set initial value
        session.set_variable("VERSION", "1.0");
        assert_eq!(session.get_variable("VERSION"), Some("1.0"));

        // Update value
        session.set_variable("VERSION", "2.0");
        assert_eq!(session.get_variable("VERSION"), Some("2.0"));

        // Count should still be 1
        assert_eq!(session.variable_count(), 1);
    }

    /// Test: REPL-009-001-006 - Clear variables
    #[test]
    fn test_REPL_009_001_clear_variables() {
        let script = "echo test";
        let mut session = DebugSession::new(script);

        // Add variables
        session.set_variable("A", "1");
        session.set_variable("B", "2");
        session.set_variable("C", "3");
        assert_eq!(session.variable_count(), 3);

        // Clear all
        session.clear_variables();
        assert_eq!(session.variable_count(), 0);
        assert_eq!(session.list_variables(), vec![]);
        assert_eq!(session.get_variable("A"), None);
    }

    /// Test: REPL-009-001-007 - Variables persist across steps
    #[test]
    fn test_REPL_009_001_variables_persist_across_steps() {
        let script = "echo line1\necho line2\necho line3";
        let mut session = DebugSession::new(script);

        // Set variable before stepping
        session.set_variable("COUNTER", "0");

        // Step through and verify variable persists
        session.step();
        assert_eq!(session.get_variable("COUNTER"), Some("0"));

        session.step();
        assert_eq!(session.get_variable("COUNTER"), Some("0"));

        // Update variable mid-execution
        session.set_variable("COUNTER", "2");
        session.step();
        assert_eq!(session.get_variable("COUNTER"), Some("2"));
    }

    // ===== REPL-009-002: Environment Display Tests =====

    #[test]
    fn test_REPL_009_002_env_display() {
        let script = "echo test";
        let session = DebugSession::new(script);

        // Get an environment variable that should exist (PATH always exists)
        let path = session.get_env("PATH");
        assert!(path.is_some(), "PATH environment variable should exist");

        // Get a variable that doesn't exist
        let nonexistent = session.get_env("BASHRS_NONEXISTENT_VAR_12345");
        assert_eq!(nonexistent, None);

        // List all environment variables
        let env_vars = session.list_env();
        assert!(!env_vars.is_empty(), "Should have at least one env var");

        // Verify sorted order
        let mut sorted = env_vars.clone();
        sorted.sort_by_key(|(name, _)| name.clone());
        assert_eq!(env_vars, sorted, "Environment variables should be sorted");
    }

    #[test]
    fn test_REPL_009_002_env_filter() {
        let script = "echo test";
        let session = DebugSession::new(script);

        // Filter by prefix (most systems have PATH-related variables)
        let path_vars = session.filter_env("PATH");
        assert!(
            !path_vars.is_empty(),
            "Should find at least one PATH-related variable"
        );

        // All filtered results should start with the prefix
        for (name, _) in &path_vars {
            assert!(
                name.starts_with("PATH"),
                "Filtered variable {} should start with PATH",
                name
            );
        }

        // Filter with non-matching prefix
        let empty_filter = session.filter_env("BASHRS_NONEXISTENT_PREFIX");
        assert_eq!(
            empty_filter.len(),
            0,
            "Filter with non-matching prefix should return empty"
        );

        // Verify sorted order
        let mut sorted = path_vars.clone();
        sorted.sort_by_key(|(name, _)| name.clone());
        assert_eq!(path_vars, sorted, "Filtered env vars should be sorted");
    }

    // ===== REPL-009-003: Call Stack Tracking Tests =====

    #[test]
    fn test_REPL_009_003_backtrace_single() {
        let script = "echo line1\necho line2\necho line3";
        let mut session = DebugSession::new(script);

        // Initially, call stack should have main frame
        let initial_len = session.call_stack().len();
        assert_eq!(initial_len, 1, "Should have just main frame initially");

        // Push a frame
        session.push_frame("function1", 1);

        // Get backtrace
        let stack = session.call_stack();
        assert_eq!(stack.len(), 2, "Should have main + function1");

        let frame = &stack[1];
        assert_eq!(frame.name, "function1");
        assert_eq!(frame.line, 1);

        // Pop frame
        session.pop_frame();

        // Should be back to initial
        let final_len = session.call_stack().len();
        assert_eq!(final_len, initial_len);
    }

    #[test]
    fn test_REPL_009_003_backtrace_nested() {
        let script = "echo test";
        let mut session = DebugSession::new(script);

        // Push nested frames
        session.push_frame("main", 1);
        session.push_frame("func_a", 5);
        session.push_frame("func_b", 10);

        // Get full stack
        let stack = session.call_stack();
        assert_eq!(
            stack.len(),
            4,
            "Should have <main> + main + func_a + func_b"
        );

        // Verify stack order (most recent last)
        assert_eq!(stack[1].name, "main");
        assert_eq!(stack[1].line, 1);
        assert_eq!(stack[2].name, "func_a");
        assert_eq!(stack[2].line, 5);
        assert_eq!(stack[3].name, "func_b");
        assert_eq!(stack[3].line, 10);

        // Pop frames
        session.pop_frame(); // func_b
        let stack2 = session.call_stack();
        assert_eq!(stack2.len(), 3);

        session.pop_frame(); // func_a
        let stack3 = session.call_stack();
        assert_eq!(stack3.len(), 2);

        session.pop_frame(); // main
        let stack4 = session.call_stack();
        assert_eq!(stack4.len(), 1, "Should be back to just <main>");
    }

    // ===== REPL-010-001: Compare Original vs Purified =====

    /// Test: REPL-010-001-001 - Compare at breakpoint shows original and purified
    #[test]
    fn test_REPL_010_001_compare_at_breakpoint() {
        // Script with non-idempotent command
        let script = "mkdir /tmp/test";
        let mut session = DebugSession::new(script);

        // Get comparison at line 1
        let comparison = session.compare_current_line();
        assert!(comparison.is_some(), "Should have comparison for line 1");

        let cmp = comparison.unwrap();
        assert_eq!(cmp.original, "mkdir /tmp/test");
        assert_eq!(cmp.purified, "mkdir -p /tmp/test"); // Purifier adds -p flag
        assert!(cmp.differs, "Original and purified should differ");
    }

    /// Test: REPL-010-001-002 - Compare diff highlighting marks changes
    #[test]
    fn test_REPL_010_001_compare_diff_highlighting() {
        // Script with missing quotes
        let script = "echo $HOME";
        let mut session = DebugSession::new(script);

        let comparison = session.compare_current_line();
        assert!(comparison.is_some());

        let cmp = comparison.unwrap();
        assert_eq!(cmp.original, "echo $HOME");
        assert_eq!(cmp.purified, "echo \"$HOME\"");
        assert!(cmp.differs);

        // Get diff highlighting
        let diff = session.format_diff_highlighting(&cmp);
        assert!(diff.contains("$HOME"), "Diff should show variable");
        assert!(
            diff.contains("\"$HOME\""),
            "Diff should show quoted version"
        );
    }

    // ===== REPL-010-002: ENHANCED HIGHLIGHTING TESTS (RED PHASE) =====

    /// Test: REPL-010-002-001 - Highlight mkdir -p idempotency flag
    ///
    /// RED phase: Write failing test for enhanced diff highlighting
    /// that specifically marks the added -p flag
    #[test]
    fn test_REPL_010_002_highlight_mkdir_p() {
        // ARRANGE: Script with non-idempotent mkdir
        let script = "mkdir /tmp/foo";
        let mut session = DebugSession::new(script);

        // ACT: Compare lines
        let comparison = session.compare_current_line();
        assert!(comparison.is_some(), "Should be able to compare");

        let cmp = comparison.unwrap();
        assert!(cmp.differs, "Lines should differ");

        // ACT: Get enhanced highlighting
        let highlighted = session.format_diff_highlighting(&cmp);

        // ASSERT: Should highlight mkdir command
        assert!(
            highlighted.contains("mkdir"),
            "Should show mkdir command"
        );

        // ASSERT: Should highlight -p flag addition
        assert!(
            highlighted.contains("-p"),
            "Should show -p flag"
        );

        // ASSERT: Should explain idempotency transformation
        assert!(
            highlighted.to_lowercase().contains("idempot")
                || highlighted.to_lowercase().contains("idem"),
            "Should explain idempotency: {}",
            highlighted
        );
    }

    /// Test: REPL-010-002-002 - Highlight variable quoting
    ///
    /// RED phase: Test should fail until we implement quote detection
    #[test]
    fn test_REPL_010_002_highlight_quote() {
        // ARRANGE: Script with unquoted variable
        let script = "echo $USER";
        let mut session = DebugSession::new(script);

        // ACT: Compare lines
        let comparison = session.compare_current_line();
        assert!(comparison.is_some(), "Should be able to compare");

        let cmp = comparison.unwrap();
        assert!(cmp.differs, "Lines should differ");

        // ACT: Get enhanced highlighting
        let highlighted = session.format_diff_highlighting(&cmp);

        // ASSERT: Should show quotes
        assert!(
            highlighted.contains("\""),
            "Should show quote addition"
        );

        // ASSERT: Should explain quoting transformation
        assert!(
            highlighted.to_lowercase().contains("quot")
                || highlighted.to_lowercase().contains("safe"),
            "Should explain quoting: {}",
            highlighted
        );
    }

    /// Test: REPL-010-002-003 - Highlight ln -sf safety flag
    ///
    /// RED phase: Test for ln command transformation highlighting
    #[test]
    fn test_REPL_010_002_highlight_ln_sf() {
        // ARRANGE: Script with non-idempotent ln
        let script = "ln -s /tmp/src /tmp/link";
        let mut session = DebugSession::new(script);

        // ACT: Compare lines
        let comparison = session.compare_current_line();
        assert!(comparison.is_some(), "Should be able to compare");

        let cmp = comparison.unwrap();

        // Only test if lines differ (purifier might add -f flag)
        if !cmp.differs {
            // Skip test if purifier doesn't transform this
            return;
        }

        // ACT: Get enhanced highlighting
        let highlighted = session.format_diff_highlighting(&cmp);

        // ASSERT: Should show ln command
        assert!(
            highlighted.contains("ln"),
            "Should show ln command"
        );

        // ASSERT: Should highlight flag addition
        assert!(
            highlighted.contains("-") && highlighted.contains("f"),
            "Should show flag addition"
        );

        // ASSERT: Should explain safety/idempotency
        assert!(
            highlighted.to_lowercase().contains("safe")
                || highlighted.to_lowercase().contains("idempot")
                || highlighted.to_lowercase().contains("idem"),
            "Should explain transformation: {}",
            highlighted
        );
    }

    /// Test: REPL-010-002-004 - Handle no changes case
    ///
    /// RED phase: Test for already-purified script
    #[test]
    fn test_REPL_010_002_highlight_no_change() {
        // ARRANGE: Script that's already purified
        let script = "mkdir -p /tmp/foo";
        let mut session = DebugSession::new(script);

        // ACT: Compare lines
        let comparison = session.compare_current_line();
        assert!(comparison.is_some(), "Should be able to compare");

        let cmp = comparison.unwrap();
        // May or may not differ depending on other transformations

        // ACT: Get highlighting
        let highlighted = session.format_diff_highlighting(&cmp);

        // ASSERT: Should handle no-change case gracefully
        if !cmp.differs {
            assert!(
                highlighted.to_lowercase().contains("no change")
                    || !highlighted.starts_with('-'),
                "Should indicate no changes: {}",
                highlighted
            );
        }
    }

    /// Test: REPL-010-002-005 - Handle multiple transformations
    ///
    /// RED phase: Test for line with multiple changes (rm + quoting)
    #[test]
    fn test_REPL_010_002_highlight_multiple_changes() {
        // ARRANGE: Script with multiple issues
        let script = "rm $FILE";
        let mut session = DebugSession::new(script);

        // ACT: Compare lines
        let comparison = session.compare_current_line();
        assert!(comparison.is_some(), "Should be able to compare");

        let cmp = comparison.unwrap();
        assert!(cmp.differs, "Lines should differ");

        // ACT: Get highlighting
        let highlighted = session.format_diff_highlighting(&cmp);

        // ASSERT: Should show at least one transformation
        assert!(
            highlighted.contains("-f") || highlighted.contains("\""),
            "Should show either -f flag or quoting: {}",
            highlighted
        );
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

    // ===== REPL-008-002: NEXT PROPERTY TESTS =====

    /// Property: next() never increases call depth
    ///
    /// Verifies the core invariant: next() should never go deeper into function calls,
    /// only stay at same level or return to shallower levels.
    proptest! {
        #[test]
        fn prop_next_never_goes_deeper(num_lines in 1usize..20) {
            let script = (0..num_lines)
                .map(|i| format!("echo line{}", i))
                .collect::<Vec<_>>()
                .join("\n");

            let mut session = DebugSession::new(&script);
            let initial_depth = session.call_depth();

            // Call next() multiple times
            for _ in 0..num_lines {
                if session.is_finished() {
                    break;
                }

                let depth_before = session.call_depth();
                session.next();
                let depth_after = session.call_depth();

                // Depth should never increase
                prop_assert!(
                    depth_after <= depth_before,
                    "next() should never increase call depth (was {}, now {})",
                    depth_before,
                    depth_after
                );

                // Depth should never exceed initial depth
                prop_assert!(
                    depth_after <= initial_depth,
                    "Call depth should never exceed initial depth"
                );
            }
        }
    }

    /// Property: next() eventually finishes execution
    ///
    /// Verifies that calling next() repeatedly will always eventually finish,
    /// preventing infinite loops in the debugger.
    proptest! {
        #[test]
        fn prop_next_eventually_finishes(num_lines in 1usize..100) {
            let script = (0..num_lines)
                .map(|i| format!("echo line{}", i))
                .collect::<Vec<_>>()
                .join("\n");

            let mut session = DebugSession::new(&script);

            // Call next() up to 2x the number of lines (generous bound)
            let max_iterations = num_lines * 2;
            for i in 0..max_iterations {
                if session.is_finished() {
                    // Success - finished execution
                    return Ok(());
                }
                session.next();
            }

            // If we get here, we didn't finish in reasonable time
            prop_assert!(
                session.is_finished(),
                "Session should finish after {} next() calls on {} line script",
                max_iterations,
                num_lines
            );
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

    // ===== REPL-009-001: VARIABLE INSPECTION PROPERTY TESTS =====

    /// Property: Set and get variable always matches
    proptest! {
        #[test]
        fn prop_variable_set_get_matches(
            var_name in "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
            var_value in ".*{0,50}"
        ) {
            let script = "echo test";
            let mut session = DebugSession::new(script);

            // Set variable
            session.set_variable(&var_name, &var_value);

            // Get should return exact value
            prop_assert_eq!(session.get_variable(&var_name), Some(var_value.as_str()));
        }
    }

    /// Property: Variable count equals number of set operations
    proptest! {
        #[test]
        fn prop_variable_count_correct(num_vars in 0usize..20) {
            let script = "echo test";
            let mut session = DebugSession::new(script);

            // Add N variables
            for i in 0..num_vars {
                session.set_variable(format!("VAR{}", i), format!("value{}", i));
            }

            prop_assert_eq!(session.variable_count(), num_vars);
        }
    }

    /// Property: List variables preserves all set variables
    proptest! {
        #[test]
        fn prop_list_variables_complete(num_vars in 1usize..10) {
            let script = "echo test";
            let mut session = DebugSession::new(script);

            // Add N variables
            for i in 0..num_vars {
                session.set_variable(format!("VAR{}", i), format!("value{}", i));
            }

            let vars = session.list_variables();
            prop_assert_eq!(vars.len(), num_vars, "List should contain all variables");

            // All variables should be present
            for i in 0..num_vars {
                let name = format!("VAR{}", i);
                let found = vars.iter().any(|(n, _)| *n == name);
                prop_assert!(found, "Variable {} should be in list", name);
            }
        }
    }

    /// Property: Clear variables removes all
    proptest! {
        #[test]
        fn prop_clear_removes_all(num_vars in 1usize..20) {
            let script = "echo test";
            let mut session = DebugSession::new(script);

            // Add N variables
            for i in 0..num_vars {
                session.set_variable(format!("VAR{}", i), format!("value{}", i));
            }

            prop_assert_eq!(session.variable_count(), num_vars);

            // Clear all
            session.clear_variables();

            prop_assert_eq!(session.variable_count(), 0, "Count should be 0 after clear");
            prop_assert_eq!(session.list_variables().len(), 0, "List should be empty after clear");
        }
    }

    /// Property: Variables persist across execution
    proptest! {
        #[test]
        fn prop_variables_persist_execution(
            num_lines in 1usize..10,
            num_vars in 1usize..5
        ) {
            let script = (0..num_lines)
                .map(|i| format!("echo line{}", i))
                .collect::<Vec<_>>()
                .join("\n");

            let mut session = DebugSession::new(&script);

            // Set variables
            for i in 0..num_vars {
                session.set_variable(format!("VAR{}", i), format!("value{}", i));
            }

            // Execute to completion
            while !session.is_finished() {
                session.step();
            }

            // Variables should still exist
            prop_assert_eq!(session.variable_count(), num_vars, "Variables should persist");

            for i in 0..num_vars {
                let name = format!("VAR{}", i);
                let value = format!("value{}", i);
                prop_assert_eq!(session.get_variable(&name), Some(value.as_str()));
            }
        }
    }

    // ===== REPL-009-002: Environment Display Property Tests =====

    /// Property: get_env is deterministic + filter results match prefix
    proptest! {
        #[test]
        fn prop_get_env_deterministic(
            var_name in "[A-Z_][A-Z0-9_]{0,20}"
        ) {
            let script = "echo test";
            let session = DebugSession::new(script);

            // Get env twice - should be identical
            let first = session.get_env(&var_name);
            let second = session.get_env(&var_name);
            prop_assert_eq!(first, second, "get_env should be deterministic");
        }

        #[test]
        fn prop_filter_env_matches_prefix(
            prefix in "[A-Z_]{1,5}"
        ) {
            let script = "echo test";
            let session = DebugSession::new(script);

            let filtered = session.filter_env(&prefix);

            // All results must start with prefix
            for (name, _) in &filtered {
                prop_assert!(
                    name.starts_with(&prefix),
                    "Variable {} should start with prefix {}",
                    name,
                    prefix
                );
            }

            // Verify sorted order
            let mut sorted = filtered.clone();
            sorted.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));
            prop_assert_eq!(filtered.clone(), sorted, "filter_env should return sorted results");

            // Verify determinism
            let second_filtered = session.filter_env(&prefix);
            prop_assert_eq!(filtered, second_filtered, "filter_env should be deterministic");
        }
    }

    // ===== REPL-009-003: Call Stack Tracking Property Tests =====

    /// Property: Call stack depth equals number of pushes minus pops
    proptest! {
        #[test]
        fn prop_call_stack_depth_correct(
            num_pushes in 0usize..10,
            num_pops in 0usize..10
        ) {
            let script = "echo test";
            let mut session = DebugSession::new(script);

            // Initially has 1 frame (<main>)
            prop_assert_eq!(session.call_stack().len(), 1);

            // Push N frames
            for i in 0..num_pushes {
                session.push_frame(format!("func{}", i), i);
            }

            // Stack depth should be 1 + num_pushes
            let depth_after_push = session.call_stack().len();
            prop_assert_eq!(depth_after_push, 1 + num_pushes);

            // Pop M times (min(num_pops, num_pushes))
            let actual_pops = std::cmp::min(num_pops, num_pushes);
            for _ in 0..actual_pops {
                session.pop_frame();
            }

            // Stack depth should be 1 + num_pushes - actual_pops
            let expected_depth = 1 + num_pushes - actual_pops;
            let final_depth = session.call_stack().len();
            prop_assert_eq!(final_depth, expected_depth);

            // Try to pop more than available - should never go below 1
            for _ in 0..100 {
                session.pop_frame();
            }
            prop_assert_eq!(session.call_stack().len(), 1, "Stack should never go below 1 (main frame)");
        }
    }

    // ===== REPL-010-001: Compare Original vs Purified Property Tests =====

    /// Property: Comparison is deterministic (same result every time)
    proptest! {
        #[test]
        fn prop_REPL_010_001_comparison_deterministic(
            cmd in "mkdir|rm|ln",
            path in "/tmp/[a-z]{1,10}"
        ) {
            let script = format!("{} {}", cmd, path);
            let session = DebugSession::new(&script);

            // Get comparison twice
            let first = session.compare_current_line();
            let second = session.compare_current_line();

            // Should be identical
            prop_assert_eq!(first, second, "Comparison should be deterministic");
        }
    }

    /// Property: Comparison correctly identifies differences
    proptest! {
        #[test]
        fn prop_REPL_010_001_differs_flag_correct(
            cmd in "mkdir|echo|rm"
        ) {
            let script = format!("{} /tmp/test", cmd);
            let session = DebugSession::new(&script);

            if let Some(comparison) = session.compare_current_line() {
                // differs flag should match actual string comparison
                let actual_differs = comparison.original != comparison.purified;
                prop_assert_eq!(
                    comparison.differs,
                    actual_differs,
                    "differs flag should match actual comparison"
                );
            }
        }
    }

    /// Property: Format diff highlighting never panics
    proptest! {
        #[test]
        fn prop_REPL_010_001_diff_highlighting_valid(
            cmd in "[a-z]{1,10}",
            arg in "[a-z/]{1,20}"
        ) {
            let script = format!("{} {}", cmd, arg);
            let session = DebugSession::new(&script);

            if let Some(comparison) = session.compare_current_line() {
                // Should not panic
                let diff = session.format_diff_highlighting(&comparison);

                // Should contain original and purified if differs
                if comparison.differs {
                    prop_assert!(diff.contains(&comparison.original) || diff.contains(&comparison.purified));
                }
            }
        }
    }
}
