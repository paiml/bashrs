// REPL Modes Module
//
// Task: REPL-003-004 - Mode switching (Normal, Purify, Lint, Debug, Explain)
// Test Approach: RED → GREEN → REFACTOR → PROPERTY → MUTATION
//
// Quality targets:
// - Unit tests: 6+ scenarios
// - Integration tests: CLI mode switching
// - Mutation score: ≥90%
// - Complexity: <10 per function

/// REPL execution modes for different command processing behaviors.
///
/// `ReplMode` determines how the REPL interprets and processes bash commands.
/// Each mode provides a specialized view or transformation of the input.
///
/// # Modes
///
/// - **Normal**: Execute bash commands directly (default)
/// - **Purify**: Show purified, deterministic version of bash commands
/// - **Lint**: Display linting results and safety warnings
/// - **Debug**: Step-by-step execution with breakpoints
/// - **Explain**: Educational explanations of bash constructs
///
/// # Examples
///
/// ## Using modes with ReplState
///
/// ```
/// use bashrs::repl::{ReplState, ReplMode};
///
/// let mut state = ReplState::new();
///
/// // Start in normal mode
/// assert_eq!(state.mode(), ReplMode::Normal);
///
/// // Switch to purify mode
/// state.set_mode(ReplMode::Purify);
/// assert_eq!(state.mode(), ReplMode::Purify);
/// ```
///
/// ## Parsing mode from user input
///
/// ```
/// use bashrs::repl::ReplMode;
///
/// let mode = ReplMode::parse("lint").unwrap();
/// assert_eq!(mode, ReplMode::Lint);
///
/// // Case-insensitive parsing
/// let mode2 = ReplMode::parse("PURIFY").unwrap();
/// assert_eq!(mode2, ReplMode::Purify);
/// ```
///
/// ## Converting mode to string
///
/// ```
/// use bashrs::repl::ReplMode;
///
/// let mode = ReplMode::Debug;
/// assert_eq!(mode.as_str(), "debug");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ReplMode {
    /// Normal mode - execute bash commands directly (default).
    ///
    /// In Normal mode, commands are executed as-is without transformation.
    /// This is the standard interactive shell behavior.
    #[default]
    Normal,

    /// Purify mode - show purified, deterministic version of bash commands.
    ///
    /// In Purify mode, commands are transformed to remove non-deterministic
    /// constructs ($RANDOM, timestamps, etc.) and ensure idempotency.
    Purify,

    /// Lint mode - show linting results and safety warnings.
    ///
    /// In Lint mode, commands are analyzed for potential security issues,
    /// POSIX compliance, and best practices violations.
    Lint,

    /// Debug mode - step-by-step execution with breakpoints.
    ///
    /// In Debug mode, commands can be executed line-by-line with the ability
    /// to inspect variables and control flow.
    Debug,

    /// Explain mode - educational explanations of bash constructs.
    ///
    /// In Explain mode, commands are analyzed and explained in plain language,
    /// useful for learning bash syntax and semantics.
    Explain,
}

impl ReplMode {
    /// Parse mode from string (case-insensitive)
    ///
    /// This is a convenience method. You can also use `str::parse()` or `from_str()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::repl::modes::ReplMode;
    ///
    /// assert_eq!(ReplMode::parse("normal").unwrap(), ReplMode::Normal);
    /// assert_eq!(ReplMode::parse("purify").unwrap(), ReplMode::Purify);
    /// assert!(ReplMode::parse("invalid").is_err());
    /// ```
    pub fn parse(s: &str) -> Result<Self, &'static str> {
        match s.to_lowercase().as_str() {
            "normal" => Ok(ReplMode::Normal),
            "purify" => Ok(ReplMode::Purify),
            "lint" => Ok(ReplMode::Lint),
            "debug" => Ok(ReplMode::Debug),
            "explain" => Ok(ReplMode::Explain),
            _ => Err("Unknown mode: valid modes are normal, purify, lint, debug, explain"),
        }
    }

    /// Get mode name as string
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::repl::modes::ReplMode;
    ///
    /// assert_eq!(ReplMode::Normal.as_str(), "normal");
    /// assert_eq!(ReplMode::Purify.as_str(), "purify");
    /// ```
    pub fn as_str(&self) -> &'static str {
        match self {
            ReplMode::Normal => "normal",
            ReplMode::Purify => "purify",
            ReplMode::Lint => "lint",
            ReplMode::Debug => "debug",
            ReplMode::Explain => "explain",
        }
    }

    /// Get mode description
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::repl::modes::ReplMode;
    ///
    /// assert_eq!(ReplMode::Normal.description(), "Execute bash commands directly");
    /// ```
    pub fn description(&self) -> &'static str {
        match self {
            ReplMode::Normal => "Execute bash commands directly",
            ReplMode::Purify => "Show purified version of bash commands",
            ReplMode::Lint => "Show linting results for bash commands",
            ReplMode::Debug => "Debug bash commands with step-by-step execution",
            ReplMode::Explain => "Explain bash constructs and syntax",
        }
    }
}

impl std::fmt::Display for ReplMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for ReplMode {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== UNIT TESTS (RED PHASE) =====

    /// Test: REPL-003-004-001 - Normal mode is default
    #[test]
    fn test_REPL_003_004_mode_normal() {
        let mode = ReplMode::default();
        assert_eq!(mode, ReplMode::Normal);
        assert_eq!(mode.as_str(), "normal");
        assert_eq!(mode.description(), "Execute bash commands directly");
    }

    /// Test: REPL-003-004-002 - Purify mode
    #[test]
    fn test_REPL_003_004_mode_purify() {
        let mode = ReplMode::parse("purify").unwrap();
        assert_eq!(mode, ReplMode::Purify);
        assert_eq!(mode.as_str(), "purify");
        assert!(mode.description().contains("purified"));
    }

    /// Test: REPL-003-004-003 - Lint mode
    #[test]
    fn test_REPL_003_004_mode_lint() {
        let mode = ReplMode::parse("lint").unwrap();
        assert_eq!(mode, ReplMode::Lint);
        assert_eq!(mode.as_str(), "lint");
        assert!(mode.description().contains("linting"));
    }

    /// Test: REPL-003-004-004 - Debug mode
    #[test]
    fn test_REPL_003_004_mode_debug() {
        let mode = ReplMode::parse("debug").unwrap();
        assert_eq!(mode, ReplMode::Debug);
        assert_eq!(mode.as_str(), "debug");
        assert!(mode.description().contains("Debug"));
    }

    /// Test: REPL-003-004-005 - Explain mode
    #[test]
    fn test_REPL_003_004_mode_explain() {
        let mode = ReplMode::parse("explain").unwrap();
        assert_eq!(mode, ReplMode::Explain);
        assert_eq!(mode.as_str(), "explain");
        assert!(mode.description().contains("Explain"));
    }

    /// Test: REPL-003-004-006 - Invalid mode returns error
    #[test]
    fn test_REPL_003_004_mode_invalid() {
        let result = ReplMode::parse("invalid");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown mode"));
    }

    /// Test: REPL-003-004-007 - Mode parsing is case-insensitive
    #[test]
    fn test_REPL_003_004_mode_case_insensitive() {
        assert_eq!(ReplMode::parse("NORMAL").unwrap(), ReplMode::Normal);
        assert_eq!(ReplMode::parse("Purify").unwrap(), ReplMode::Purify);
        assert_eq!(ReplMode::parse("LINT").unwrap(), ReplMode::Lint);
    }

    /// Test: REPL-003-004-008 - Display trait formats correctly
    #[test]
    fn test_REPL_003_004_mode_display() {
        assert_eq!(format!("{}", ReplMode::Normal), "normal");
        assert_eq!(format!("{}", ReplMode::Purify), "purify");
        assert_eq!(format!("{}", ReplMode::Lint), "lint");
    }

    /// Test: REPL-003-004-009 - All modes are clonable
    #[test]
    fn test_REPL_003_004_mode_clone() {
        let mode1 = ReplMode::Purify;
        let mode2 = mode1;
        assert_eq!(mode1, mode2);
    }

    /// Test: REPL-003-004-010 - Mode equality works correctly
    #[test]
    fn test_REPL_003_004_mode_equality() {
        assert_eq!(ReplMode::Normal, ReplMode::Normal);
        assert_ne!(ReplMode::Normal, ReplMode::Purify);
        assert_eq!(ReplMode::parse("lint").unwrap(), ReplMode::Lint);
    }
}
