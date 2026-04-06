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

include!("lib_oracle.rs");
