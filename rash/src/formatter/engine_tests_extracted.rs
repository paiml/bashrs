#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = NormalizationEngine::new();
        assert_eq!(engine.ws_stack.len(), 1);
        assert!(matches!(engine.ws_stack[0], WhitespaceContext::Command));
    }

    #[test]
    fn test_engine_with_config() {
        let config = EngineConfig {
            enable_fast_path: false,
            max_nesting_depth: 512,
            preserve_comments: false,
            generate_proofs: true,
        };

        let engine = NormalizationEngine::with_config(config.clone());
        assert!(!engine.config.enable_fast_path);
        assert_eq!(engine.config.max_nesting_depth, 512);
    }

    #[test]
    fn test_is_canonical_simple() {
        let engine = NormalizationEngine::new();

        assert!(engine.is_canonical(b"echo hello"));
        assert!(!engine.is_canonical(b"echo  hello")); // Multiple spaces
        assert!(!engine.is_canonical(b" echo hello")); // Leading space
        assert!(!engine.is_canonical(b"echo hello ")); // Trailing space
        assert!(!engine.is_canonical(b"echo\thello")); // Tab
    }

    #[test]
    fn test_is_canonical_quoting() {
        let engine = NormalizationEngine::new();

        assert!(engine.is_canonical(b"echo \"$var\""));
        assert!(!engine.is_canonical(b"echo $var")); // Unquoted variable
    }

    #[test]
    fn test_normalize_identity() {
        let mut engine = NormalizationEngine::new();
        let input = b"echo hello";
        let config = FormatConfig::default();

        let result = engine.normalize(input, ShellDialect::Posix, config);
        assert!(result.is_ok());

        let formatted = result.unwrap();
        assert_eq!(formatted.text.as_ref(), "echo hello");
    }

    #[test]
    fn test_normalize_whitespace() {
        let mut engine = NormalizationEngine::new();
        let input = b"echo  hello   world";
        let config = FormatConfig::default();

        let result = engine.normalize(input, ShellDialect::Posix, config);
        assert!(result.is_ok());

        let formatted = result.unwrap();
        assert_eq!(formatted.text.as_ref(), "echo hello world");
        assert!(!formatted.transforms.entries.is_empty());
    }

    #[test]
    fn test_normalize_variable_quoting() {
        let mut engine = NormalizationEngine::new();
        let input = b"echo $var";
        let config = FormatConfig::default();

        let result = engine.normalize(input, ShellDialect::Posix, config);
        assert!(result.is_ok());

        let formatted = result.unwrap();
        assert_eq!(formatted.text.as_ref(), "echo \"$var\"");

        // Should have a quote expansion transform
        let has_quote_transform = formatted
            .transforms
            .entries
            .iter()
            .any(|entry| matches!(entry.transform, Transform::QuoteExpansion { .. }));
        assert!(has_quote_transform);
    }

    #[test]
    fn test_normalize_quoted_strings() {
        let mut engine = NormalizationEngine::new();
        let input = b"echo 'hello  world'";
        let config = FormatConfig::default();

        let result = engine.normalize(input, ShellDialect::Posix, config);
        assert!(result.is_ok());

        let formatted = result.unwrap();
        // Whitespace inside quotes should be preserved
        assert_eq!(formatted.text.as_ref(), "echo 'hello  world'");
    }

    #[test]
    fn test_normalize_comments() {
        let mut engine = NormalizationEngine::new();
        let input = b"echo hello # this is a comment";
        let config = FormatConfig::default();

        let result = engine.normalize(input, ShellDialect::Posix, config);
        assert!(result.is_ok());

        let formatted = result.unwrap();
        assert_eq!(formatted.text.as_ref(), "echo hello # this is a comment");
        assert_eq!(formatted.metadata.comments.len(), 1);
        assert_eq!(
            formatted.metadata.comments[0].content,
            "# this is a comment"
        );
    }

    #[test]
    fn test_normalize_multiline() {
        let mut engine = NormalizationEngine::new();
        let input = b"echo  hello\necho   world";
        let config = FormatConfig::default();

        let result = engine.normalize(input, ShellDialect::Posix, config);
        assert!(result.is_ok());

        let formatted = result.unwrap();
        assert_eq!(formatted.text.as_ref(), "echo hello\necho world");
    }

    #[test]
    fn test_has_proper_quoting() {
        let engine = NormalizationEngine::new();

        assert!(engine.has_proper_quoting("echo \"$var\""));
        assert!(engine.has_proper_quoting("echo '$var'"));
        assert!(!engine.has_proper_quoting("echo $var"));
        assert!(engine.has_proper_quoting("echo hello")); // No variables
    }

    #[test]
    fn test_config_effects() {
        let config = EngineConfig {
            enable_fast_path: false,
            preserve_comments: false,
            ..Default::default()
        };

        let mut engine = NormalizationEngine::with_config(config);

        // Fast path should be disabled
        assert!(!engine.is_canonical(b"echo hello"));

        // Comments should not be preserved (this would need full implementation)
        let input = b"echo hello # comment";
        let format_config = FormatConfig::default();

        let result = engine.normalize(input, ShellDialect::Posix, format_config);
        assert!(result.is_ok());
    }
}
