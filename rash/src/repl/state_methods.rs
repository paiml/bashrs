impl ReplState {

    /// Returns a reference to all session variables.
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::repl::ReplState;
    ///
    /// let mut state = ReplState::new();
    /// state.set_variable("A".to_string(), "1".to_string());
    /// state.set_variable("B".to_string(), "2".to_string());
    ///
    /// assert_eq!(state.variables().len(), 2);
    /// ```
    pub fn variables(&self) -> &HashMap<String, String> {
        &self.variables
    }

    /// Clears all session variables.
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::repl::ReplState;
    ///
    /// let mut state = ReplState::new();
    /// state.set_variable("X".to_string(), "1".to_string());
    /// assert_eq!(state.variable_count(), 1);
    ///
    /// state.clear_variables();
    /// assert_eq!(state.variable_count(), 0);
    /// ```
    pub fn clear_variables(&mut self) {
        self.variables.clear();
    }

    /// Returns the number of session variables.
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::repl::ReplState;
    ///
    /// let mut state = ReplState::new();
    /// assert_eq!(state.variable_count(), 0);
    ///
    /// state.set_variable("VAR".to_string(), "val".to_string());
    /// assert_eq!(state.variable_count(), 1);
    /// ```
    pub fn variable_count(&self) -> usize {
        self.variables.len()
    }

    // === Exit Management ===

    /// Requests the REPL to exit.
    ///
    /// Sets the exit flag, signaling the main loop to terminate cleanly.
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::repl::ReplState;
    ///
    /// let mut state = ReplState::new();
    /// assert!(!state.should_exit());
    ///
    /// state.request_exit();
    /// assert!(state.should_exit());
    /// ```
    pub fn request_exit(&mut self) {
        self.exit_requested = true;
    }

    /// Checks if exit was requested.
    ///
    /// # Returns
    ///
    /// * `true` if exit was requested
    /// * `false` otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::repl::ReplState;
    ///
    /// let mut state = ReplState::new();
    /// assert!(!state.should_exit());
    /// ```
    pub fn should_exit(&self) -> bool {
        self.exit_requested
    }

    // === Error Tracking ===

    /// Records an error occurrence.
    ///
    /// Increments the error counter for statistics and debugging.
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::repl::ReplState;
    ///
    /// let mut state = ReplState::new();
    /// state.record_error();
    /// state.record_error();
    ///
    /// assert_eq!(state.error_count(), 2);
    /// ```
    pub fn record_error(&mut self) {
        self.error_count += 1;
    }

    /// Returns the total error count.
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::repl::ReplState;
    ///
    /// let mut state = ReplState::new();
    /// assert_eq!(state.error_count(), 0);
    ///
    /// state.record_error();
    /// assert_eq!(state.error_count(), 1);
    /// ```
    pub fn error_count(&self) -> usize {
        self.error_count
    }

    /// Resets the error count to zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::repl::ReplState;
    ///
    /// let mut state = ReplState::new();
    /// state.record_error();
    /// state.record_error();
    /// assert_eq!(state.error_count(), 2);
    ///
    /// state.reset_error_count();
    /// assert_eq!(state.error_count(), 0);
    /// ```
    pub fn reset_error_count(&mut self) {
        self.error_count = 0;
    }

    // === Mode Management ===

    /// Returns the current REPL mode.
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::repl::{ReplState, ReplMode};
    ///
    /// let state = ReplState::new();
    /// assert_eq!(state.mode(), ReplMode::Normal);
    /// ```
    pub fn mode(&self) -> ReplMode {
        self.mode
    }

    /// Sets the REPL mode.
    ///
    /// # Arguments
    ///
    /// * `mode` - The new mode to set
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::repl::{ReplState, ReplMode};
    ///
    /// let mut state = ReplState::new();
    /// state.set_mode(ReplMode::Purify);
    ///
    /// assert_eq!(state.mode(), ReplMode::Purify);
    /// ```
    pub fn set_mode(&mut self, mode: ReplMode) {
        self.mode = mode;
    }

    /// Set last loaded script
    pub fn set_last_loaded_script(&mut self, path: PathBuf) {
        self.last_loaded_script = Some(path);
    }

    /// Get last loaded script
    pub fn last_loaded_script(&self) -> Option<&PathBuf> {
        self.last_loaded_script.as_ref()
    }

    /// Add a function to loaded functions
    pub fn add_function(&mut self, name: String) {
        if !self.loaded_functions.contains(&name) {
            self.loaded_functions.push(name);
        }
    }

    /// Get loaded functions
    pub fn loaded_functions(&self) -> &[String] {
        &self.loaded_functions
    }

    /// Clear all loaded functions
    pub fn clear_functions(&mut self) {
        self.loaded_functions.clear();
    }

    /// Get function count
    pub fn function_count(&self) -> usize {
        self.loaded_functions.len()
    }
}










include!("state_cont.rs");
