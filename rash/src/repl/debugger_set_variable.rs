impl DebugSession {

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










include!("debugger_continueresult.rs");
