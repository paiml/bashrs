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
