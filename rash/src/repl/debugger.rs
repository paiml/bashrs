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
        let call_stack = vec![StackFrame::new("<main>", 0)];

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
        let line = self.lines.get(self.current_line)?.clone();

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
    pub fn step_over(&mut self) -> Option<String> {
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

    /// Execute until the current function returns
    ///
    /// Continues execution until we exit the current stack frame.
    /// If already at the top level (main), this behaves like continue to end.
    ///
    /// Note: This is a simplified implementation that works with manual
    /// call stack tracking (push_frame/pop_frame). Future versions will
    /// automatically track function boundaries.
    ///
    /// # Returns
    ///
    /// - `ContinueResult::BreakpointHit(line)` if stopped at breakpoint
    /// - `ContinueResult::Finished` if execution completed or returned from function
    ///
    /// # Examples
    ///
    /// ```
    /// # use bashrs::repl::debugger::{DebugSession, ContinueResult};
    /// let script = "echo line1\necho line2\necho line3";
    /// let mut session = DebugSession::new(script);
    ///
    /// // Simulate entering a function
    /// session.push_frame("test_function", 1);
    /// assert_eq!(session.call_depth(), 2);
    ///
    /// // Finish will exit the function
    /// let result = session.finish();
    /// assert!(matches!(result, ContinueResult::Finished));
    /// ```
    pub fn finish(&mut self) -> ContinueResult {
        // Get current call stack depth
        let initial_depth = self.call_depth();

        // If already finished, return immediately
        if self.is_finished() {
            return ContinueResult::Finished;
        }

        // If at top level (<main> only), behave like continue_execution()
        if initial_depth <= 1 {
            return self.continue_execution();
        }

        // Execute until we return from current function
        loop {
            // Check if at breakpoint before executing
            if self.at_breakpoint() {
                return ContinueResult::BreakpointHit(self.current_line());
            }

            // Check if we've returned from the function (depth decreased)
            if self.call_depth() < initial_depth {
                return ContinueResult::Finished;
            }

            // Execute one step
            match self.step() {
                Some(_output) => {
                    // Line executed, check conditions again
                    if self.at_breakpoint() {
                        return ContinueResult::BreakpointHit(self.current_line());
                    }

                    if self.call_depth() < initial_depth {
                        return ContinueResult::Finished;
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

        let explanations = Self::detect_transformations(&comparison.original, &comparison.purified);

        Self::format_diff_with_explanations(
            &comparison.original,
            &comparison.purified,
            &explanations,
        )
    }

    /// Detect transformation patterns between original and purified bash
    fn detect_transformations(orig: &str, purified: &str) -> Vec<&'static str> {
        let mut explanations = Vec::new();

        // Check for idempotency flags
        if Self::flag_added(orig, purified, "mkdir", "mkdir -p") {
            explanations.push("added idempotency flag -p to mkdir");
        }

        if Self::flag_added(orig, purified, "rm", "rm -f") {
            explanations.push("added idempotency flag -f to rm");
        }

        if orig.contains("ln") {
            if Self::flag_added(orig, purified, "ln", "-sf") {
                explanations.push("added idempotency flag -f to ln");
            } else if purified.contains("-f") && !orig.contains("-f") {
                explanations.push("added idempotency flag -f");
            }
        }

        // Check for variable quoting (safety)
        if purified.contains("\"$") && !orig.contains("\"$") {
            explanations.push("added safety quoting around variables");
        }

        explanations
    }

    /// Check if a flag was added to a command
    #[inline]
    fn flag_added(orig: &str, purified: &str, cmd: &str, flag_pattern: &str) -> bool {
        orig.contains(cmd) && purified.contains(flag_pattern) && !orig.contains(flag_pattern)
    }

    /// Format diff output with explanations
    fn format_diff_with_explanations(orig: &str, purified: &str, explanations: &[&str]) -> String {
        let mut output = format!("- {}\n+ {}", orig, purified);

        if !explanations.is_empty() {
            output.push_str("\n(");
            output.push_str(&explanations.join(", "));
            output.push(')');
        }

        output
    }

    /// Explain transformations that will be applied at the current line
    ///
    /// Returns a human-readable explanation of what purification will change,
    /// or None if no transformations will be applied.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bashrs::repl::debugger::DebugSession;
    /// let script = "mkdir /tmp/foo";
    /// let session = DebugSession::new(script);
    ///
    /// // Check that explanation is available for transformable code
    /// let explanation = session.explain_current_line();
    /// assert!(explanation.is_some());
    /// ```
    ///
    /// # Returns
    ///
    /// - `Some(String)`: Explanation of transformations
    /// - `None`: No transformations (already purified)
    pub fn explain_current_line(&self) -> Option<String> {
        let comparison = self.compare_current_line()?;

        if !comparison.differs {
            return None; // No transformations needed
        }

        let explanations = Self::detect_transformations(&comparison.original, &comparison.purified);

        if explanations.is_empty() {
            // Lines differ but no specific transformations detected
            Some("Script will be transformed".to_string())
        } else {
            // Join explanations into readable sentence
            Some(explanations.join(", "))
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
#[path = "debugger_tests_ext.rs"]
mod tests_ext;
