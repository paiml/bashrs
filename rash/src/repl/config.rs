// REPL Configuration Module
//
// Task: REPL-003-001 - ReplConfig struct
// Test Approach: RED → GREEN → REFACTOR → PROPERTY → MUTATION
//
// Quality targets:
// - Unit tests: 3+ scenarios
// - Property tests: 2+ generators
// - Mutation score: ≥90%
// - Complexity: <10 per function

use std::path::PathBuf;
use std::time::Duration;

/// Configuration for the bashrs REPL (Read-Eval-Print Loop).
///
/// `ReplConfig` controls resource limits, history management, and execution modes
/// for the interactive bash purification shell. It provides sensible defaults while
/// allowing fine-grained control for specialized use cases.
///
/// Inspired by Ruchy REPL configuration pattern:
/// - Resource limits for safety (memory, timeout, recursion depth)
/// - Sandboxed mode for untrusted input
/// - Configurable constraints prevent runaway execution
///
/// # Examples
///
/// ## Basic usage with defaults
///
/// ```
/// use bashrs::repl::ReplConfig;
///
/// let config = ReplConfig::default();
/// assert_eq!(config.max_memory, 100_000_000); // 100MB
/// assert_eq!(config.timeout.as_secs(), 30);
/// assert!(config.validate().is_ok());
/// ```
///
/// ## Custom configuration with builder pattern
///
/// ```
/// use bashrs::repl::ReplConfig;
/// use std::time::Duration;
///
/// let config = ReplConfig::default()
///     .with_max_memory(200_000_000)  // 200MB
///     .with_timeout(Duration::from_secs(60))
///     .with_debug();
///
/// assert_eq!(config.max_memory, 200_000_000);
/// assert!(config.debug);
/// ```
///
/// ## Sandboxed mode for untrusted input
///
/// ```
/// use bashrs::repl::ReplConfig;
///
/// let config = ReplConfig::sandboxed();
/// assert!(config.sandboxed);
/// assert_eq!(config.max_memory, 10_000_000);  // More restrictive
/// assert_eq!(config.timeout.as_secs(), 5);    // Shorter timeout
/// ```
///
/// # Safety
///
/// This configuration does not directly handle unsafe operations, but the resource
/// limits it defines help prevent:
/// - Memory exhaustion from unbounded allocations
/// - Infinite loops or excessive recursion
/// - Denial of service from malicious input
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplConfig {
    /// Maximum memory usage in bytes (default: 100MB).
    ///
    /// Prevents runaway memory consumption from malformed or malicious scripts.
    pub max_memory: usize,

    /// Execution timeout for commands (default: 30 seconds).
    ///
    /// Commands exceeding this duration will be terminated to prevent
    /// infinite loops or excessive processing.
    pub timeout: Duration,

    /// Maximum recursion depth (default: 100).
    ///
    /// Prevents stack overflow from deeply nested function calls or
    /// recursive scripts.
    pub max_depth: usize,

    /// Enable debug mode (default: false).
    ///
    /// When enabled, prints detailed execution traces and intermediate results
    /// for troubleshooting and development.
    pub debug: bool,

    /// Enable sandboxed execution (default: false).
    ///
    /// When enabled, applies more restrictive resource limits suitable for
    /// untrusted input from external sources.
    pub sandboxed: bool,

    /// Maximum history entries (default: 1000).
    ///
    /// Number of command history entries to retain across sessions.
    pub max_history: usize,

    /// Ignore duplicate commands in history (default: true).
    ///
    /// When true, consecutive duplicate commands are collapsed into a single
    /// history entry.
    pub history_ignore_dups: bool,

    /// Ignore commands starting with space (default: true).
    ///
    /// When true, commands prefixed with whitespace are not saved to history.
    /// Useful for sensitive commands that should not be logged.
    pub history_ignore_space: bool,

    /// Custom history file path (default: None, uses ~/.bashrs_history).
    ///
    /// When specified, overrides the default history file location.
    pub history_path: Option<PathBuf>,
}

impl Default for ReplConfig {
    fn default() -> Self {
        Self {
            max_memory: 100_000_000, // 100MB
            timeout: Duration::from_secs(30),
            max_depth: 100,
            debug: false,
            sandboxed: false,
            max_history: 1000,
            history_ignore_dups: true,
            history_ignore_space: true,
            history_path: None,
        }
    }
}

impl ReplConfig {
    /// Creates a new `ReplConfig` with custom resource limits.
    ///
    /// This constructor allows direct specification of the three core resource limits
    /// while using default values for other settings. For more fine-grained control,
    /// use the builder pattern with [`ReplConfig::default()`] and `with_*` methods.
    ///
    /// # Arguments
    ///
    /// * `max_memory` - Maximum memory usage in bytes
    /// * `timeout` - Execution timeout for commands
    /// * `max_depth` - Maximum recursion depth
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::repl::ReplConfig;
    /// use std::time::Duration;
    ///
    /// let config = ReplConfig::new(
    ///     50_000_000,              // 50MB memory limit
    ///     Duration::from_secs(10), // 10 second timeout
    ///     50,                      // 50 levels of recursion
    /// );
    ///
    /// assert_eq!(config.max_memory, 50_000_000);
    /// assert_eq!(config.timeout.as_secs(), 10);
    /// assert_eq!(config.max_depth, 50);
    /// ```
    pub fn new(max_memory: usize, timeout: Duration, max_depth: usize) -> Self {
        Self {
            max_memory,
            timeout,
            max_depth,
            debug: false,
            sandboxed: false,
            max_history: 1000,
            history_ignore_dups: true,
            history_ignore_space: true,
            history_path: None,
        }
    }

    /// Creates a sandboxed configuration with restrictive limits for untrusted input.
    ///
    /// Sandboxed mode applies much tighter resource constraints compared to default
    /// configuration, making it suitable for executing user-provided scripts or
    /// handling input from untrusted sources.
    ///
    /// **Sandboxed limits:**
    /// - Memory: 10MB (vs 100MB default)
    /// - Timeout: 5 seconds (vs 30 seconds default)
    /// - Recursion depth: 10 (vs 100 default)
    /// - History: 100 entries (vs 1000 default)
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::repl::ReplConfig;
    ///
    /// let config = ReplConfig::sandboxed();
    ///
    /// assert!(config.sandboxed);
    /// assert_eq!(config.max_memory, 10_000_000);
    /// assert_eq!(config.timeout.as_secs(), 5);
    /// assert_eq!(config.max_depth, 10);
    /// ```
    ///
    /// # Use Cases
    ///
    /// - Processing scripts from external APIs
    /// - Educational environments with untrusted user code
    /// - CI/CD pipelines running third-party scripts
    /// - Interactive demos with public access
    pub fn sandboxed() -> Self {
        Self {
            max_memory: 10_000_000,          // 10MB (more restrictive)
            timeout: Duration::from_secs(5), // 5s (shorter timeout)
            max_depth: 10,                   // Shallow recursion
            debug: false,
            sandboxed: true,
            max_history: 100, // Limited history in sandboxed mode
            history_ignore_dups: true,
            history_ignore_space: true,
            history_path: None,
        }
    }

    /// Enables debug mode for detailed execution traces.
    ///
    /// When debug mode is enabled, the REPL will print:
    /// - Detailed execution traces
    /// - Intermediate parsing results
    /// - Variable assignments and state changes
    /// - Timing information for operations
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::repl::ReplConfig;
    ///
    /// let config = ReplConfig::default().with_debug();
    /// assert!(config.debug);
    /// ```
    pub fn with_debug(mut self) -> Self {
        self.debug = true;
        self
    }

    /// Sets the maximum memory usage in bytes.
    ///
    /// # Arguments
    ///
    /// * `max_memory` - Maximum memory usage in bytes (must be > 0)
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::repl::ReplConfig;
    ///
    /// let config = ReplConfig::default()
    ///     .with_max_memory(200_000_000); // 200MB
    ///
    /// assert_eq!(config.max_memory, 200_000_000);
    /// ```
    pub fn with_max_memory(mut self, max_memory: usize) -> Self {
        self.max_memory = max_memory;
        self
    }

    /// Sets the execution timeout for commands.
    ///
    /// Commands exceeding this duration will be terminated.
    ///
    /// # Arguments
    ///
    /// * `timeout` - Execution timeout (must be > 0)
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::repl::ReplConfig;
    /// use std::time::Duration;
    ///
    /// let config = ReplConfig::default()
    ///     .with_timeout(Duration::from_secs(60)); // 1 minute timeout
    ///
    /// assert_eq!(config.timeout.as_secs(), 60);
    /// ```
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Sets the maximum recursion depth.
    ///
    /// # Arguments
    ///
    /// * `max_depth` - Maximum recursion depth (must be > 0)
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::repl::ReplConfig;
    ///
    /// let config = ReplConfig::default()
    ///     .with_max_depth(200);
    ///
    /// assert_eq!(config.max_depth, 200);
    /// ```
    pub fn with_max_depth(mut self, max_depth: usize) -> Self {
        self.max_depth = max_depth;
        self
    }

    /// Sets a custom history file path.
    ///
    /// By default, the REPL uses `~/.bashrs_history`. This method allows
    /// overriding the history file location.
    ///
    /// # Arguments
    ///
    /// * `history_path` - Path to the history file
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::repl::ReplConfig;
    /// use std::path::PathBuf;
    ///
    /// let config = ReplConfig::default()
    ///     .with_history_path(PathBuf::from("/tmp/my_history"));
    ///
    /// assert_eq!(config.history_path, Some(PathBuf::from("/tmp/my_history")));
    /// ```
    pub fn with_history_path(mut self, history_path: PathBuf) -> Self {
        self.history_path = Some(history_path);
        self
    }

    /// Validates the configuration and returns an error if invalid.
    ///
    /// Checks that all resource limits are positive (> 0):
    /// - `max_memory` must be > 0
    /// - `max_depth` must be > 0
    /// - `timeout` must be > 0
    ///
    /// # Returns
    ///
    /// * `Ok(())` if configuration is valid
    /// * `Err(String)` with error message if configuration is invalid
    ///
    /// # Examples
    ///
    /// ## Valid configuration
    ///
    /// ```
    /// use bashrs::repl::ReplConfig;
    ///
    /// let config = ReplConfig::default();
    /// assert!(config.validate().is_ok());
    /// ```
    ///
    /// ## Invalid configuration (zero memory)
    ///
    /// ```
    /// use bashrs::repl::ReplConfig;
    /// use std::time::Duration;
    ///
    /// let config = ReplConfig::new(0, Duration::from_secs(30), 100);
    /// let result = config.validate();
    ///
    /// assert!(result.is_err());
    /// assert!(result.unwrap_err().contains("max_memory"));
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error string describing the validation failure:
    /// - `"max_memory must be greater than 0"` - Memory limit is zero
    /// - `"max_depth must be greater than 0"` - Recursion depth is zero
    /// - `"timeout must be greater than 0"` - Timeout is zero
    pub fn validate(&self) -> Result<(), String> {
        if self.max_memory == 0 {
            return Err("max_memory must be greater than 0".to_string());
        }
        if self.max_depth == 0 {
            return Err("max_depth must be greater than 0".to_string());
        }
        if self.timeout.as_millis() == 0 {
            return Err("timeout must be greater than 0".to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // ===== UNIT TESTS (RED PHASE) =====

    /// Test: REPL-003-001-001 - Default configuration has reasonable defaults
    #[test]
    fn test_repl_003_001_config_defaults() {
        let config = ReplConfig::default();
        assert_eq!(config.max_memory, 100_000_000); // 100MB
        assert_eq!(config.timeout.as_secs(), 30);
        assert_eq!(config.max_depth, 100);
        assert!(!config.debug);
        assert!(!config.sandboxed);
    }

    /// Test: REPL-003-001-002 - Custom configuration with new()
    #[test]
    fn test_repl_003_001_config_custom_limits() {
        let config = ReplConfig::new(
            50_000_000, // 50MB
            Duration::from_secs(10),
            50,
        );
        assert_eq!(config.max_memory, 50_000_000);
        assert_eq!(config.timeout.as_secs(), 10);
        assert_eq!(config.max_depth, 50);
    }

    /// Test: REPL-003-001-003 - Sandboxed configuration is more restrictive
    #[test]
    fn test_repl_003_001_config_sandboxed() {
        let config = ReplConfig::sandboxed();
        assert!(config.sandboxed);
        assert_eq!(config.max_memory, 10_000_000); // 10MB (less than default)
        assert_eq!(config.timeout.as_secs(), 5); // 5s (less than default)
        assert_eq!(config.max_depth, 10); // Shallow (less than default)
    }

    /// Test: REPL-003-001-004 - Builder pattern with_debug()
    #[test]
    fn test_repl_003_001_config_with_debug() {
        let config = ReplConfig::default().with_debug();
        assert!(config.debug);
    }

    /// Test: REPL-003-001-005 - Builder pattern with_max_memory()
    #[test]
    fn test_repl_003_001_config_with_max_memory() {
        let config = ReplConfig::default().with_max_memory(200_000_000);
        assert_eq!(config.max_memory, 200_000_000);
    }

    /// Test: REPL-003-001-006 - Builder pattern with_timeout()
    #[test]
    fn test_repl_003_001_config_with_timeout() {
        let config = ReplConfig::default().with_timeout(Duration::from_secs(60));
        assert_eq!(config.timeout.as_secs(), 60);
    }

    /// Test: REPL-003-001-007 - Builder pattern with_max_depth()
    #[test]
    fn test_repl_003_001_config_with_max_depth() {
        let config = ReplConfig::default().with_max_depth(200);
        assert_eq!(config.max_depth, 200);
    }

    /// Test: REPL-003-001-008 - Validation succeeds for valid config
    #[test]
    fn test_repl_003_001_config_validation_valid() {
        let config = ReplConfig::default();
        assert!(config.validate().is_ok());
    }

    /// Test: REPL-003-001-009 - Validation fails for zero max_memory
    #[test]
    fn test_repl_003_001_config_validation_zero_memory() {
        let config = ReplConfig::new(0, Duration::from_secs(30), 100);
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("max_memory"));
    }

    /// Test: REPL-003-001-010 - Validation fails for zero max_depth
    #[test]
    fn test_repl_003_001_config_validation_zero_depth() {
        let config = ReplConfig::new(100_000_000, Duration::from_secs(30), 0);
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("max_depth"));
    }

    /// Test: REPL-003-001-011 - Validation fails for zero timeout
    #[test]
    fn test_repl_003_001_config_validation_zero_timeout() {
        let config = ReplConfig::new(100_000_000, Duration::from_millis(0), 100);
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("timeout"));
    }

    // ===== PROPERTY TESTS =====

    // Property test: Resource limits are always positive
    proptest! {
        #[test]
        fn prop_repl_003_001_resource_limits_always_positive(
            mem in 1usize..1_000_000_000,
            timeout_secs in 1u64..3600,
            depth in 1usize..1000,
        ) {
            let config = ReplConfig::new(
                mem,
                Duration::from_secs(timeout_secs),
                depth,
            );

            // Property: All resource limits must be > 0
            prop_assert!(config.max_memory > 0);
            prop_assert!(config.timeout.as_millis() > 0);
            prop_assert!(config.max_depth > 0);

            // Property: Valid configs pass validation
            prop_assert!(config.validate().is_ok());
        }
    }

    // Property test: Builder pattern preserves values
    proptest! {
        #[test]
        fn prop_repl_003_001_builder_preserves_values(
            mem in 1usize..1_000_000_000,
            timeout_secs in 1u64..3600,
            depth in 1usize..1000,
        ) {
            let config = ReplConfig::default()
                .with_max_memory(mem)
                .with_timeout(Duration::from_secs(timeout_secs))
                .with_max_depth(depth);

            // Property: Builder methods set correct values
            prop_assert_eq!(config.max_memory, mem);
            prop_assert_eq!(config.timeout.as_secs(), timeout_secs);
            prop_assert_eq!(config.max_depth, depth);
        }
    }

    // Property test: Sandboxed config is always more restrictive than default
    proptest! {
        #[test]
        fn prop_repl_003_001_sandboxed_more_restrictive(
            _x in 0..100,  // Dummy property to run multiple times
        ) {
            let default_config = ReplConfig::default();
            let sandboxed_config = ReplConfig::sandboxed();

            // Property: Sandboxed is more restrictive
            prop_assert!(sandboxed_config.max_memory < default_config.max_memory);
            prop_assert!(sandboxed_config.timeout < default_config.timeout);
            prop_assert!(sandboxed_config.max_depth < default_config.max_depth);
            prop_assert!(sandboxed_config.sandboxed);
        }
    }
}
