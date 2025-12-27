// Allow multiple crate versions from transitive dependencies (aprender -> wgpu -> foldhash)
#![allow(clippy::multiple_crate_versions)]

//! ML-powered error classification oracle for bashrs.
//!
//! Uses aprender Random Forest classifier (GPU-accelerated via trueno/wgpu) to:
//! - Classify shell errors into actionable categories (24 categories)
//! - Suggest fixes based on error patterns
//! - Detect error drift requiring model retraining
//!
//! ## GPU Acceleration
//!
//! Enable GPU feature for RTX 4090 acceleration via wgpu/trueno:
//! ```toml
//! bashrs-oracle = { version = "*", features = ["gpu"] }
//! ```
//!
//! ## Performance Targets (from depyler-oracle)
//! - Accuracy: >90% (depyler achieved 97.73%)
//! - Training time: <1s
//! - Predictions/sec: >1000
//! - Model size: <1MB (with zstd compression)

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use aprender::format::{self, Compression, ModelType, SaveOptions};
use aprender::metrics::drift::{DriftConfig, DriftDetector, DriftStatus};
use aprender::primitives::Matrix;
use aprender::tree::RandomForestClassifier;
use serde::{Deserialize, Serialize};

pub mod categories;
pub mod classifier;
pub mod corpus;
pub mod features;

pub use categories::ErrorCategory;
pub use classifier::ErrorClassifier;
pub use corpus::{Corpus, TrainingExample};
pub use features::ErrorFeatures;

/// Error types for the oracle.
#[derive(Debug, thiserror::Error)]
pub enum OracleError {
    /// Model loading/saving error.
    #[error("Model error: {0}")]
    Model(String),
    /// Feature extraction error.
    #[error("Feature extraction error: {0}")]
    Feature(String),
    /// Training error.
    #[error("Training error: {0}")]
    Training(String),
    /// Classification error.
    #[error("Classification error: {0}")]
    Classification(String),
    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for oracle operations.
pub type Result<T> = std::result::Result<T, OracleError>;

/// Classification result with confidence and suggested fix.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClassificationResult {
    /// Predicted error category.
    pub category: ErrorCategory,
    /// Confidence score (0.0 - 1.0).
    pub confidence: f32,
    /// Suggested fix template.
    pub suggested_fix: Option<String>,
    /// Related fix patterns.
    pub related_patterns: Vec<String>,
}

/// Configuration for the Random Forest classifier.
#[derive(Clone, Debug)]
pub struct OracleConfig {
    /// Number of trees in the forest (default: 100).
    /// IMPORTANT: 100 is sufficient. 10,000 causes 15+ min training!
    pub n_estimators: usize,
    /// Maximum tree depth (default: 10).
    pub max_depth: usize,
    /// Random seed for reproducibility.
    pub random_state: Option<u64>,
}

impl Default for OracleConfig {
    fn default() -> Self {
        Self {
            n_estimators: 100,
            max_depth: 10,
            random_state: Some(42),
        }
    }
}

/// Default model filename.
const DEFAULT_MODEL_NAME: &str = "bashrs_oracle.apr";

/// ML-powered shell error classification oracle.
pub struct Oracle {
    /// Random Forest classifier (GPU-accelerated via aprender).
    classifier: RandomForestClassifier,
    /// Configuration used to create the classifier.
    #[allow(dead_code)]
    config: OracleConfig,
    /// Category list for index mapping (kept for model introspection).
    #[allow(dead_code)]
    categories: Vec<ErrorCategory>,
    /// Fix templates per category.
    fix_templates: HashMap<ErrorCategory, Vec<String>>,
    /// Drift detector for retraining triggers.
    drift_detector: DriftDetector,
    /// Historical performance scores.
    performance_history: Vec<f32>,
    /// Whether model has been trained.
    is_trained: bool,
}

impl Default for Oracle {
    fn default() -> Self {
        Self::new()
    }
}

impl Oracle {
    /// Get the default model path.
    #[must_use]
    pub fn default_model_path() -> PathBuf {
        // Try to find project root via Cargo.toml
        let mut path = std::env::current_dir().unwrap_or_default();
        for _ in 0..5 {
            if path.join("Cargo.toml").exists() {
                return path.join(DEFAULT_MODEL_NAME);
            }
            if !path.pop() {
                break;
            }
        }
        PathBuf::from(DEFAULT_MODEL_NAME)
    }

    /// Load model from default path, or train and save if not found.
    ///
    /// # Errors
    /// Returns error if training fails.
    pub fn load_or_train() -> Result<Self> {
        let path = Self::default_model_path();

        if path.exists() {
            match Self::load(&path) {
                Ok(oracle) => return Ok(oracle),
                Err(e) => {
                    tracing::warn!("Failed to load cached model: {e}. Retraining...");
                }
            }
        }

        // Train using synthetic data (5000 samples for good accuracy)
        let corpus = Corpus::generate_synthetic(5000);
        let oracle = Self::train_from_corpus(&corpus, OracleConfig::default())?;

        // Save for next time
        if let Err(e) = oracle.save(&path) {
            tracing::warn!("Failed to cache model to {}: {e}", path.display());
        }

        Ok(oracle)
    }

    /// Create a new oracle with default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(OracleConfig::default())
    }

    /// Create a new oracle with custom configuration.
    #[must_use]
    pub fn with_config(config: OracleConfig) -> Self {
        let mut classifier =
            RandomForestClassifier::new(config.n_estimators).with_max_depth(config.max_depth);
        if let Some(seed) = config.random_state {
            classifier = classifier.with_random_state(seed);
        }

        Self {
            classifier,
            config,
            categories: ErrorCategory::all().to_vec(),
            fix_templates: Self::default_fix_templates(),
            drift_detector: DriftDetector::new(
                DriftConfig::default()
                    .with_min_samples(10)
                    .with_window_size(50),
            ),
            performance_history: Vec::new(),
            is_trained: false,
        }
    }

    /// Train oracle from a corpus.
    ///
    /// # Errors
    /// Returns error if training fails.
    pub fn train_from_corpus(corpus: &Corpus, config: OracleConfig) -> Result<Self> {
        let (x, y) = corpus.to_training_data();

        // Convert to Matrix for aprender
        let n_samples = x.len();
        let n_features = x.first().map(|row| row.len()).unwrap_or(0);
        let flat: Vec<f32> = x.into_iter().flatten().collect();
        let features = Matrix::from_vec(n_samples, n_features, flat)
            .map_err(|e| OracleError::Training(format!("Failed to create feature matrix: {e}")))?;
        let labels: Vec<usize> = y.into_iter().map(|l| l as usize).collect();

        let mut oracle = Self::with_config(config);
        oracle.train(&features, &labels)?;

        Ok(oracle)
    }

    /// Train the oracle on labeled error data.
    ///
    /// # Errors
    /// Returns error if training fails.
    pub fn train(&mut self, features: &Matrix<f32>, labels: &[usize]) -> Result<()> {
        self.classifier
            .fit(features, labels)
            .map_err(|e| OracleError::Training(e.to_string()))?;
        self.is_trained = true;

        Ok(())
    }

    /// Classify an error and return category with confidence.
    pub fn classify(&self, features: &ErrorFeatures) -> Result<ClassificationResult> {
        if !self.is_trained {
            // Fallback to keyword-based classification
            let kw_classifier = ErrorClassifier::new();
            let category = kw_classifier.classify_by_keywords(
                &features
                    .features
                    .iter()
                    .map(|f| f.to_string())
                    .collect::<Vec<_>>()
                    .join(" "),
            );
            return Ok(ClassificationResult {
                category,
                confidence: 0.5,
                suggested_fix: Some(category.fix_suggestion().to_string()),
                related_patterns: vec![],
            });
        }

        let feature_matrix = Matrix::from_vec(1, ErrorFeatures::SIZE, features.as_slice().to_vec())
            .map_err(|e| {
                OracleError::Classification(format!("Failed to create feature matrix: {e}"))
            })?;
        let predictions = self.classifier.predict(&feature_matrix);

        let pred_idx = predictions
            .as_slice()
            .first()
            .copied()
            .ok_or_else(|| OracleError::Classification("No prediction produced".to_string()))?;
        let category = ErrorCategory::from_label_index(pred_idx);

        let suggested_fix = self
            .fix_templates
            .get(&category)
            .and_then(|fixes| fixes.first().cloned());

        let related = self
            .fix_templates
            .get(&category)
            .map(|fixes| fixes.iter().skip(1).cloned().collect())
            .unwrap_or_default();

        Ok(ClassificationResult {
            category,
            confidence: 0.85, // TODO: Extract from tree probabilities
            suggested_fix,
            related_patterns: related,
        })
    }

    /// Classify an error from raw inputs.
    pub fn classify_error(
        &self,
        exit_code: i32,
        stderr: &str,
        command: Option<&str>,
    ) -> Result<ClassificationResult> {
        let features = ErrorFeatures::extract(exit_code, stderr, command);
        self.classify(&features)
    }

    /// Get fix suggestion for an error.
    #[must_use]
    pub fn suggest_fix(&self, exit_code: i32, stderr: &str, command: Option<&str>) -> String {
        // If not trained, use keyword classifier directly on the stderr message
        if !self.is_trained {
            let kw_classifier = ErrorClassifier::new();
            let category = kw_classifier.classify_by_keywords(stderr);
            let confidence = kw_classifier.confidence(stderr, category);
            return format!(
                "[{:.0}% confident] {}: {}",
                confidence * 100.0,
                category.name(),
                category.fix_suggestion()
            );
        }

        match self.classify_error(exit_code, stderr, command) {
            Ok(result) => {
                format!(
                    "[{:.0}% confident] {}: {}",
                    result.confidence * 100.0,
                    result.category.name(),
                    result
                        .suggested_fix
                        .unwrap_or_else(|| result.category.fix_suggestion().to_string())
                )
            }
            Err(_) => {
                // Fallback to keyword classifier
                let kw_classifier = ErrorClassifier::new();
                let category = kw_classifier.classify_by_keywords(stderr);
                format!(
                    "[keyword] {}: {}",
                    category.name(),
                    category.fix_suggestion()
                )
            }
        }
    }

    /// Check if the model needs retraining based on performance drift.
    pub fn check_drift(&mut self, recent_accuracy: f32) -> DriftStatus {
        self.performance_history.push(recent_accuracy);

        if self.performance_history.len() < 10 {
            return DriftStatus::NoDrift;
        }

        let mid = self.performance_history.len() / 2;
        let baseline: Vec<f32> = self
            .performance_history
            .get(..mid)
            .map(|s| s.to_vec())
            .unwrap_or_default();
        let current: Vec<f32> = self
            .performance_history
            .get(mid..)
            .map(|s| s.to_vec())
            .unwrap_or_default();

        self.drift_detector
            .detect_performance_drift(&baseline, &current)
    }

    /// Save the oracle model to a file (with zstd compression).
    ///
    /// # Errors
    /// Returns error if saving fails.
    pub fn save(&self, path: &Path) -> Result<()> {
        let options = SaveOptions::default()
            .with_name("bashrs-oracle")
            .with_description("RandomForest error classification model for bashrs shell linter")
            .with_compression(Compression::ZstdDefault); // 14x smaller!

        format::save(&self.classifier, ModelType::RandomForest, path, options)
            .map_err(|e| OracleError::Model(e.to_string()))?;

        Ok(())
    }

    /// Load an oracle model from a file.
    ///
    /// # Errors
    /// Returns error if loading fails.
    pub fn load(path: &Path) -> Result<Self> {
        let classifier: RandomForestClassifier = format::load(path, ModelType::RandomForest)
            .map_err(|e| OracleError::Model(e.to_string()))?;

        let config = OracleConfig::default();
        Ok(Self {
            classifier,
            config,
            categories: ErrorCategory::all().to_vec(),
            fix_templates: Self::default_fix_templates(),
            drift_detector: DriftDetector::new(
                DriftConfig::default()
                    .with_min_samples(10)
                    .with_window_size(50),
            ),
            performance_history: Vec::new(),
            is_trained: true,
        })
    }

    /// Check if the oracle has been trained.
    #[must_use]
    pub fn is_trained(&self) -> bool {
        self.is_trained
    }

    /// Default fix templates for each category.
    fn default_fix_templates() -> HashMap<ErrorCategory, Vec<String>> {
        let mut templates = HashMap::new();

        // Syntax errors
        templates.insert(
            ErrorCategory::SyntaxQuoteMismatch,
            vec![
                "Check for unmatched quotes (' or \")".to_string(),
                "Use shellcheck to identify the exact location".to_string(),
            ],
        );
        templates.insert(
            ErrorCategory::SyntaxBracketMismatch,
            vec![
                "Check for unmatched brackets ([], {}, ())".to_string(),
                "Ensure conditionals have proper [ ] or [[ ]] syntax".to_string(),
            ],
        );
        templates.insert(
            ErrorCategory::SyntaxUnexpectedToken,
            vec![
                "Review syntax near the reported token".to_string(),
                "Check for missing 'then', 'do', or 'fi'".to_string(),
            ],
        );
        templates.insert(
            ErrorCategory::SyntaxMissingOperand,
            vec![
                "Add missing operand to the expression".to_string(),
                "Check arithmetic expressions for completeness".to_string(),
            ],
        );

        // Command errors
        templates.insert(
            ErrorCategory::CommandNotFound,
            vec![
                "Check PATH or install the missing command".to_string(),
                "Verify the command name spelling".to_string(),
                "Try 'which <command>' or 'type <command>'".to_string(),
            ],
        );
        templates.insert(
            ErrorCategory::CommandPermissionDenied,
            vec![
                "Use chmod +x to make the script executable".to_string(),
                "Run with sudo if elevated privileges needed".to_string(),
            ],
        );
        templates.insert(
            ErrorCategory::CommandInvalidOption,
            vec![
                "Check command documentation with --help or man page".to_string(),
                "Verify option syntax (single dash vs double dash)".to_string(),
            ],
        );
        templates.insert(
            ErrorCategory::CommandMissingArgument,
            vec![
                "Provide required argument to the command".to_string(),
                "Check command usage with --help".to_string(),
            ],
        );

        // File errors
        templates.insert(
            ErrorCategory::FileNotFound,
            vec![
                "Verify the file path exists".to_string(),
                "Check for typos in the path".to_string(),
                "Use 'ls' to list directory contents".to_string(),
            ],
        );
        templates.insert(
            ErrorCategory::FilePermissionDenied,
            vec![
                "Check file permissions with ls -la".to_string(),
                "Use sudo if needed for system files".to_string(),
            ],
        );
        templates.insert(
            ErrorCategory::FileIsDirectory,
            vec![
                "Use a file path, not a directory".to_string(),
                "Add /* to operate on directory contents".to_string(),
            ],
        );
        templates.insert(
            ErrorCategory::FileNotDirectory,
            vec![
                "Use a directory path, not a file".to_string(),
                "Check parent directories exist".to_string(),
            ],
        );
        templates.insert(
            ErrorCategory::FileTooManyOpen,
            vec![
                "Close unused file descriptors".to_string(),
                "Increase ulimit -n value".to_string(),
            ],
        );

        // Variable errors
        templates.insert(
            ErrorCategory::VariableUnbound,
            vec![
                "Initialize variable before use".to_string(),
                "Use ${VAR:-default} for default values".to_string(),
                "Check for typos in variable name".to_string(),
            ],
        );
        templates.insert(
            ErrorCategory::VariableReadonly,
            vec![
                "Cannot modify readonly variable".to_string(),
                "Use a different variable name".to_string(),
            ],
        );
        templates.insert(
            ErrorCategory::VariableBadSubstitution,
            vec![
                "Fix parameter expansion syntax".to_string(),
                "Check for proper ${} brace matching".to_string(),
            ],
        );

        // Process errors
        templates.insert(
            ErrorCategory::ProcessSignaled,
            vec![
                "Process was killed by signal".to_string(),
                "Check for memory issues (OOM killer)".to_string(),
            ],
        );
        templates.insert(
            ErrorCategory::ProcessExitNonZero,
            vec![
                "Check command exit status with echo $?".to_string(),
                "Add error handling with || or set -e".to_string(),
            ],
        );
        templates.insert(
            ErrorCategory::ProcessTimeout,
            vec![
                "Increase timeout value".to_string(),
                "Optimize the command for better performance".to_string(),
            ],
        );

        // Pipe/redirect errors
        templates.insert(
            ErrorCategory::PipeBroken,
            vec![
                "Check if downstream process exited early".to_string(),
                "Use || true to ignore SIGPIPE".to_string(),
            ],
        );
        templates.insert(
            ErrorCategory::RedirectFailed,
            vec![
                "Verify target path is writable".to_string(),
                "Check disk space availability".to_string(),
            ],
        );
        templates.insert(
            ErrorCategory::HereDocUnterminated,
            vec![
                "Add terminating delimiter for here-doc".to_string(),
                "Ensure delimiter is at start of line with no trailing spaces".to_string(),
            ],
        );

        // Unknown
        templates.insert(
            ErrorCategory::Unknown,
            vec!["Review the full error message for details".to_string()],
        );

        templates
    }
}

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
