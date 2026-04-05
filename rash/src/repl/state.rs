// REPL State Module
//
// Task: REPL-003-002 - ReplState struct
// Test Approach: RED → GREEN → REFACTOR → PROPERTY → MUTATION
//
// Quality targets:
// - Unit tests: 10+ scenarios
// - Property tests: 3+ generators
// - Mutation score: ≥90%
// - Complexity: <10 per function

use crate::repl::ReplMode;
use std::collections::HashMap;
use std::path::PathBuf;

/// Mutable state for a REPL session.
///
/// `ReplState` manages all stateful aspects of an interactive REPL session:
/// - **Command history**: Navigable with up/down arrows
/// - **Session variables**: Persist values across commands
/// - **Exit management**: Clean shutdown signaling
/// - **Error tracking**: Statistics for debugging
/// - **Mode switching**: Different behaviors (Normal, Purify, Lint, Debug, Explain)
/// - **Script loading**: Track loaded files for `:reload` command
/// - **Function tracking**: Extract and manage functions from scripts
///
/// Inspired by Ruchy REPL state management patterns.
///
/// # Examples
///
/// ## Basic usage
///
/// ```
/// use bashrs::repl::ReplState;
///
/// let mut state = ReplState::new();
///
/// // Add commands to history
/// state.add_history("x=5".to_string());
/// state.add_history("echo $x".to_string());
/// assert_eq!(state.history_len(), 2);
///
/// // Set and retrieve variables
/// state.set_variable("user".to_string(), "alice".to_string());
/// assert_eq!(state.get_variable("user"), Some(&"alice".to_string()));
/// ```
///
/// ## Mode switching
///
/// ```
/// use bashrs::repl::{ReplState, ReplMode};
///
/// let mut state = ReplState::new();
/// assert_eq!(state.mode(), ReplMode::Normal);
///
/// state.set_mode(ReplMode::Purify);
/// assert_eq!(state.mode(), ReplMode::Purify);
/// ```
///
/// ## Error tracking
///
/// ```
/// use bashrs::repl::ReplState;
///
/// let mut state = ReplState::new();
/// assert_eq!(state.error_count(), 0);
///
/// state.record_error();
/// state.record_error();
/// assert_eq!(state.error_count(), 2);
///
/// state.reset_error_count();
/// assert_eq!(state.error_count(), 0);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ReplState {
    /// Command history (for up/down arrow navigation).
    history: Vec<String>,

    /// Session variables (persist across commands).
    variables: HashMap<String, String>,

    /// Exit requested flag (for clean shutdown).
    exit_requested: bool,

    /// Error count (for debugging and statistics).
    error_count: usize,

    /// Current REPL mode.
    mode: ReplMode,

    /// Last loaded script path (for `:reload` command).
    last_loaded_script: Option<PathBuf>,

    /// Functions extracted from loaded scripts.
    loaded_functions: Vec<String>,
}

impl ReplState {
    /// Creates a new `ReplState` with default values.
    ///
    /// Initializes an empty state with:
    /// - No command history
    /// - No session variables
    /// - Normal mode
    /// - Zero error count
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::repl::ReplState;
    ///
    /// let state = ReplState::new();
    /// assert_eq!(state.history_len(), 0);
    /// assert_eq!(state.variable_count(), 0);
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    // === Command History Management ===

    /// Adds a command to the history.
    ///
    /// Commands are stored in chronological order for navigation with up/down arrows.
    ///
    /// # Arguments
    ///
    /// * `command` - The command string to add to history
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::repl::ReplState;
    ///
    /// let mut state = ReplState::new();
    /// state.add_history("x=5".to_string());
    /// state.add_history("echo $x".to_string());
    ///
    /// assert_eq!(state.history_len(), 2);
    /// assert_eq!(state.history()[0], "x=5");
    /// ```
    pub fn add_history(&mut self, command: String) {
        self.history.push(command);
    }

    /// Returns a slice of all history entries.
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::repl::ReplState;
    ///
    /// let mut state = ReplState::new();
    /// state.add_history("cmd1".to_string());
    /// state.add_history("cmd2".to_string());
    ///
    /// let history = state.history();
    /// assert_eq!(history.len(), 2);
    /// assert_eq!(history[1], "cmd2");
    /// ```
    pub fn history(&self) -> &[String] {
        &self.history
    }

    /// Gets a specific history entry by index.
    ///
    /// # Arguments
    ///
    /// * `index` - Zero-based index into history
    ///
    /// # Returns
    ///
    /// * `Some(&String)` if index is valid
    /// * `None` if index is out of bounds
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::repl::ReplState;
    ///
    /// let mut state = ReplState::new();
    /// state.add_history("first".to_string());
    ///
    /// assert_eq!(state.get_history(0), Some(&"first".to_string()));
    /// assert_eq!(state.get_history(1), None);
    /// ```
    pub fn get_history(&self, index: usize) -> Option<&String> {
        self.history.get(index)
    }

    /// Clears all command history.
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::repl::ReplState;
    ///
    /// let mut state = ReplState::new();
    /// state.add_history("cmd".to_string());
    /// assert_eq!(state.history_len(), 1);
    ///
    /// state.clear_history();
    /// assert_eq!(state.history_len(), 0);
    /// ```
    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    /// Returns the number of history entries.
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::repl::ReplState;
    ///
    /// let mut state = ReplState::new();
    /// assert_eq!(state.history_len(), 0);
    ///
    /// state.add_history("cmd".to_string());
    /// assert_eq!(state.history_len(), 1);
    /// ```
    pub fn history_len(&self) -> usize {
        self.history.len()
    }

    // === Session Variable Management ===

    /// Sets a session variable.
    ///
    /// Variables persist across commands within a REPL session.
    ///
    /// # Arguments
    ///
    /// * `name` - Variable name
    /// * `value` - Variable value
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::repl::ReplState;
    ///
    /// let mut state = ReplState::new();
    /// state.set_variable("USER".to_string(), "alice".to_string());
    ///
    /// assert_eq!(state.get_variable("USER"), Some(&"alice".to_string()));
    /// ```
    pub fn set_variable(&mut self, name: String, value: String) {
        self.variables.insert(name, value);
    }

    /// Gets a session variable by name.
    ///
    /// # Arguments
    ///
    /// * `name` - Variable name to lookup
    ///
    /// # Returns
    ///
    /// * `Some(&String)` if variable exists
    /// * `None` if variable not found
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::repl::ReplState;
    ///
    /// let mut state = ReplState::new();
    /// state.set_variable("PATH".to_string(), "/usr/bin".to_string());
    ///
    /// assert_eq!(state.get_variable("PATH"), Some(&"/usr/bin".to_string()));
    /// assert_eq!(state.get_variable("MISSING"), None);
    /// ```
    pub fn get_variable(&self, name: &str) -> Option<&String> {
        self.variables.get(name)
    }

    /// Removes a session variable.
    ///
    /// # Arguments
    ///
    /// * `name` - Variable name to remove
    ///
    /// # Returns
    ///
    /// * `Some(String)` with the removed value
    /// * `None` if variable didn't exist
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::repl::ReplState;
    ///
    /// let mut state = ReplState::new();
    /// state.set_variable("TEMP".to_string(), "value".to_string());
    ///
    /// let removed = state.remove_variable("TEMP");
    /// assert_eq!(removed, Some("value".to_string()));
    /// assert_eq!(state.get_variable("TEMP"), None);
    /// ```
    pub fn remove_variable(&mut self, name: &str) -> Option<String> {
        self.variables.remove(name)
    }

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
