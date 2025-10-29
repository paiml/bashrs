// formatter_config.rs - Configuration for bash script formatter
// Following ruchy design patterns for configuration
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Configuration for bash script formatting
///
/// # Examples
///
/// ```
/// use bashrs::bash_quality::FormatterConfig;
///
/// let config = FormatterConfig::default();
/// assert_eq!(config.indent_width, 2);
/// assert!(!config.use_tabs);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct FormatterConfig {
    /// Number of spaces per indentation level (default: 2, bash standard)
    pub indent_width: usize,

    /// Use tabs instead of spaces for indentation (default: false)
    pub use_tabs: bool,

    /// Quote all variable expansions (default: true)
    pub quote_variables: bool,

    /// Use [[ ]] instead of [ ] for tests (default: true)
    pub use_double_brackets: bool,

    /// Normalize function syntax to name() { } (default: true)
    pub normalize_functions: bool,

    /// Put 'then' on same line as 'if' (default: true)
    pub inline_then: bool,

    /// Add space before function braces (default: true)
    pub space_before_brace: bool,

    /// Preserve existing blank lines (default: true)
    pub preserve_blank_lines: bool,

    /// Maximum consecutive blank lines (default: 2)
    pub max_blank_lines: usize,

    /// Ignore files matching these patterns (default: empty)
    pub ignore_patterns: Vec<String>,
}

impl Default for FormatterConfig {
    fn default() -> Self {
        Self {
            indent_width: 2,
            use_tabs: false,
            quote_variables: true,
            use_double_brackets: true,
            normalize_functions: true,
            inline_then: true,
            space_before_brace: true,
            preserve_blank_lines: true,
            max_blank_lines: 2,
            ignore_patterns: vec![],
        }
    }
}

impl FormatterConfig {
    /// Create a new configuration with default values
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::bash_quality::FormatterConfig;
    ///
    /// let config = FormatterConfig::new();
    /// assert_eq!(config.indent_width, 2);
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Load configuration from a TOML file
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file cannot be read
    /// - The TOML is invalid
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use bashrs::bash_quality::FormatterConfig;
    ///
    /// let config = FormatterConfig::from_file(".bashrs-fmt.toml").unwrap();
    /// println!("Indent width: {}", config.indent_width);
    /// ```
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let contents = std::fs::read_to_string(path.as_ref())
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        Self::from_toml(&contents)
    }

    /// Load configuration from TOML string
    ///
    /// # Errors
    ///
    /// Returns an error if the TOML is invalid
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::bash_quality::FormatterConfig;
    ///
    /// let toml = r#"
    /// indent_width = 4
    /// use_tabs = true
    /// "#;
    ///
    /// let config = FormatterConfig::from_toml(toml).unwrap();
    /// assert_eq!(config.indent_width, 4);
    /// assert!(config.use_tabs);
    /// ```
    pub fn from_toml(toml_str: &str) -> Result<Self, String> {
        toml::from_str(toml_str.trim())
            .map_err(|e| format!("Failed to parse config TOML: {}", e))
    }

    /// Save configuration to a TOML file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use bashrs::bash_quality::FormatterConfig;
    ///
    /// let config = FormatterConfig::default();
    /// config.to_file(".bashrs-fmt.toml").unwrap();
    /// ```
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let toml_str = self.to_toml()?;
        std::fs::write(path.as_ref(), toml_str)
            .map_err(|e| format!("Failed to write config file: {}", e))
    }

    /// Convert configuration to TOML string
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::bash_quality::FormatterConfig;
    ///
    /// let config = FormatterConfig::default();
    /// let toml = config.to_toml().unwrap();
    /// assert!(toml.contains("indent_width"));
    /// ```
    pub fn to_toml(&self) -> Result<String, String> {
        toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))
    }

    /// Check if a file path should be ignored based on patterns
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::bash_quality::FormatterConfig;
    ///
    /// let mut config = FormatterConfig::default();
    /// config.ignore_patterns = vec!["**/test/**".to_string()];
    ///
    /// assert!(config.should_ignore("src/test/example.sh"));
    /// assert!(!config.should_ignore("src/main.sh"));
    /// ```
    pub fn should_ignore(&self, path: &str) -> bool {
        for pattern in &self.ignore_patterns {
            if path.contains(pattern.trim_start_matches("**/").trim_end_matches("/**")) {
                return true;
            }
        }
        false
    }

    /// Merge with another configuration, preferring non-default values
    ///
    /// # Examples
    ///
    /// ```
    /// use bashrs::bash_quality::FormatterConfig;
    ///
    /// let mut base = FormatterConfig::default();
    /// let mut override_config = FormatterConfig::default();
    /// override_config.indent_width = 4;
    ///
    /// base.merge(override_config);
    /// assert_eq!(base.indent_width, 4);
    /// ```
    pub fn merge(&mut self, other: Self) {
        let default = Self::default();

        if other.indent_width != default.indent_width {
            self.indent_width = other.indent_width;
        }
        if other.use_tabs != default.use_tabs {
            self.use_tabs = other.use_tabs;
        }
        if other.quote_variables != default.quote_variables {
            self.quote_variables = other.quote_variables;
        }
        if other.use_double_brackets != default.use_double_brackets {
            self.use_double_brackets = other.use_double_brackets;
        }
        if other.normalize_functions != default.normalize_functions {
            self.normalize_functions = other.normalize_functions;
        }
        if other.inline_then != default.inline_then {
            self.inline_then = other.inline_then;
        }
        if other.space_before_brace != default.space_before_brace {
            self.space_before_brace = other.space_before_brace;
        }
        if other.preserve_blank_lines != default.preserve_blank_lines {
            self.preserve_blank_lines = other.preserve_blank_lines;
        }
        if other.max_blank_lines != default.max_blank_lines {
            self.max_blank_lines = other.max_blank_lines;
        }
        if !other.ignore_patterns.is_empty() {
            self.ignore_patterns.extend(other.ignore_patterns);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = FormatterConfig::default();
        assert_eq!(config.indent_width, 2);
        assert!(!config.use_tabs);
        assert!(config.quote_variables);
        assert!(config.use_double_brackets);
        assert!(config.normalize_functions);
    }

    #[test]
    fn test_from_toml() {
        let toml = r#"
        indent_width = 4
        use_tabs = true
        quote_variables = false
        use_double_brackets = false
        normalize_functions = false
        inline_then = false
        space_before_brace = false
        preserve_blank_lines = false
        max_blank_lines = 1
        ignore_patterns = ["*.test.sh"]
        "#;

        let config = FormatterConfig::from_toml(toml).unwrap();
        assert_eq!(config.indent_width, 4);
        assert!(config.use_tabs);
        assert!(!config.quote_variables);
        assert!(!config.use_double_brackets);
        assert!(!config.normalize_functions);
    }

    #[test]
    fn test_to_toml() {
        let config = FormatterConfig::default();
        let toml = config.to_toml().unwrap();

        assert!(toml.contains("indent_width = 2"));
        assert!(toml.contains("use_tabs = false"));
        assert!(toml.contains("quote_variables = true"));
    }

    #[test]
    fn test_should_ignore() {
        let mut config = FormatterConfig::default();
        config.ignore_patterns = vec![
            "**/target/**".to_string(),
            "**/test/**".to_string(),
        ];

        assert!(config.should_ignore("src/target/debug/script.sh"));
        assert!(config.should_ignore("src/test/integration.sh"));
        assert!(!config.should_ignore("src/main.sh"));
    }

    #[test]
    fn test_merge() {
        let mut base = FormatterConfig::default();
        let mut override_config = FormatterConfig::default();
        override_config.indent_width = 4;
        override_config.use_tabs = true;

        base.merge(override_config);

        assert_eq!(base.indent_width, 4);
        assert!(base.use_tabs);
        assert_eq!(base.max_blank_lines, 2); // unchanged
    }

    #[test]
    fn test_config_round_trip() {
        let original = FormatterConfig {
            indent_width: 4,
            use_tabs: true,
            quote_variables: false,
            ..Default::default()
        };

        let toml = original.to_toml().unwrap();
        let loaded = FormatterConfig::from_toml(&toml).unwrap();

        assert_eq!(loaded.indent_width, original.indent_width);
        assert_eq!(loaded.use_tabs, original.use_tabs);
        assert_eq!(loaded.quote_variables, original.quote_variables);
    }
}
