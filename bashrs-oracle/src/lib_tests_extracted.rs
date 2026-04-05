#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]
    use super::*;

    #[test]
    fn test_oracle_creation() {
        let oracle = Oracle::new();
        assert_eq!(oracle.categories.len(), ErrorCategory::all().len());
        assert!(!oracle.is_trained());
    }

    #[test]
    fn test_fix_templates_coverage() {
        let oracle = Oracle::new();
        for category in ErrorCategory::all() {
            assert!(
                oracle.fix_templates.contains_key(category),
                "Missing fix template for {category:?}"
            );
        }
    }

    #[test]
    fn test_drift_detection_insufficient_data() {
        let mut oracle = Oracle::new();
        let status = oracle.check_drift(0.95);
        assert!(matches!(status, DriftStatus::NoDrift));
    }

    #[test]
    fn test_default_model_path() {
        let path = Oracle::default_model_path();
        assert!(path.to_string_lossy().contains("bashrs_oracle.apr"));
    }

    #[test]
    fn test_suggest_fix_fallback() {
        let oracle = Oracle::new();
        // Without training, should fall back to keyword classifier
        let suggestion = oracle.suggest_fix(127, "bash: foo: command not found", None);
        assert!(
            suggestion.contains("command") || suggestion.contains("Command"),
            "Got: {suggestion}"
        );
    }

    #[test]
    fn test_train_from_corpus() {
        let corpus = Corpus::generate_synthetic(100);
        let oracle = Oracle::train_from_corpus(&corpus, OracleConfig::default())
            .expect("Training should succeed");

        assert!(oracle.is_trained());

        // Should be able to classify after training
        let features = ErrorFeatures::extract(127, "command not found", None);
        let result = oracle.classify(&features);
        assert!(result.is_ok());
    }

    #[test]
    fn test_classify_error_convenience() {
        let corpus = Corpus::generate_synthetic(100);
        let oracle = Oracle::train_from_corpus(&corpus, OracleConfig::default())
            .expect("Training should succeed");

        let result = oracle
            .classify_error(127, "bash: foo: command not found", None)
            .expect("Classification should succeed");

        assert!(result.confidence > 0.0);
        assert!(result.suggested_fix.is_some());
    }

    #[test]
    fn test_save_and_load() {
        let corpus = Corpus::generate_synthetic(100);
        let oracle = Oracle::train_from_corpus(&corpus, OracleConfig::default())
            .expect("Training should succeed");

        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let path = temp_dir.path().join("test_model.apr");

        oracle.save(&path).expect("Save should succeed");
        assert!(path.exists());

        let loaded = Oracle::load(&path).expect("Load should succeed");
        assert_eq!(loaded.categories.len(), oracle.categories.len());
        assert!(loaded.is_trained());
    }

    #[test]
    fn test_oracle_config_default() {
        let config = OracleConfig::default();
        assert_eq!(config.n_estimators, 100);
        assert_eq!(config.max_depth, 10);
        assert_eq!(config.random_state, Some(42));
    }
}
