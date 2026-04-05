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
}

include!("debugger_set_variable.rs");
