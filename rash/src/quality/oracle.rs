//! Oracle ML-Powered Error Classifier (ML-007, ML-008, ML-009, ML-010)
//!
//! Implements ML-based error classification using k-NN and pattern matching
//! following Toyota Way principles of continuous improvement (Kaizen).
//!
//! # Toyota Way Principles
//!
//! - **Kaizen** (Continuous improvement): Learn from user fix acceptance
//! - **Jidoka** (Automation with human touch): ML classifies, human approves
//! - **Poka-yoke** (Error-proofing): Confidence scores prevent bad auto-fixes
//!
//! # References
//!
//! - BASHRS-SPEC-ML-007: Feature Extraction
//! - BASHRS-SPEC-ML-008: k-NN Classifier
//! - BASHRS-SPEC-ML-009: Pattern Library
//! - Kim et al. (2013): Bug localization with learning-to-rank
//! - Le et al. (2016): Learning-to-rank fault localization

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::linter::{Diagnostic, Severity};

/// ML-classified error categories for shell scripts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShellErrorCategory {
    // Security (SEC rules)
    CommandInjection,
    PathTraversal,
    UnsafeExpansion,

    // Determinism (DET rules)
    NonDeterministicRandom,
    TimestampUsage,
    ProcessIdDependency,

    // Idempotency (IDEM rules)
    NonIdempotentOperation,
    MissingGuard,
    UnsafeOverwrite,

    // Quoting (SC2xxx)
    MissingQuotes,
    GlobbingRisk,
    WordSplitting,

    // Other
    SyntaxError,
    StyleViolation,
    Unknown,
}

impl ShellErrorCategory {
    /// Get human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            Self::CommandInjection => "Command Injection",
            Self::PathTraversal => "Path Traversal",
            Self::UnsafeExpansion => "Unsafe Expansion",
            Self::NonDeterministicRandom => "Non-Deterministic Random",
            Self::TimestampUsage => "Timestamp Usage",
            Self::ProcessIdDependency => "Process ID Dependency",
            Self::NonIdempotentOperation => "Non-Idempotent Operation",
            Self::MissingGuard => "Missing Guard",
            Self::UnsafeOverwrite => "Unsafe Overwrite",
            Self::MissingQuotes => "Missing Quotes",
            Self::GlobbingRisk => "Globbing Risk",
            Self::WordSplitting => "Word Splitting",
            Self::SyntaxError => "Syntax Error",
            Self::StyleViolation => "Style Violation",
            Self::Unknown => "Unknown",
        }
    }

    /// Get severity level for this category
    pub fn default_severity(&self) -> Severity {
        match self {
            Self::CommandInjection | Self::PathTraversal => Severity::Error,
            Self::UnsafeExpansion | Self::NonDeterministicRandom => Severity::Warning,
            Self::TimestampUsage | Self::ProcessIdDependency => Severity::Warning,
            Self::NonIdempotentOperation | Self::MissingGuard => Severity::Warning,
            Self::UnsafeOverwrite => Severity::Error,
            Self::MissingQuotes | Self::GlobbingRisk | Self::WordSplitting => Severity::Warning,
            Self::SyntaxError => Severity::Error,
            Self::StyleViolation => Severity::Info,
            Self::Unknown => Severity::Info,
        }
    }
}

/// 73-feature vector for ML classification (ML-007)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FeatureVector {
    // Lexical features (20)
    pub code_prefix: String,
    pub code_numeric: u32,
    pub message_length: usize,
    pub has_variable_reference: bool,
    pub has_path_reference: bool,
    pub has_command_reference: bool,
    pub has_array_reference: bool,
    pub has_arithmetic: bool,
    pub has_subshell: bool,
    pub has_pipe: bool,
    pub has_redirect: bool,
    pub has_glob: bool,
    pub has_quote_chars: bool,
    pub word_count: usize,
    pub special_char_count: usize,
    pub uppercase_ratio: f64,
    pub digit_ratio: f64,
    pub has_file_extension: bool,
    pub has_url: bool,
    pub has_env_var: bool,

    // Structural features (25)
    pub span_length: usize,
    pub start_line: usize,
    pub start_column: usize,
    pub is_multiline: bool,
    pub nesting_depth: usize,
    pub in_function: bool,
    pub in_loop: bool,
    pub in_conditional: bool,
    pub in_case: bool,
    pub in_subshell: bool,
    pub line_length: usize,
    pub indentation_level: usize,
    pub preceding_blank_lines: usize,
    pub following_blank_lines: usize,
    pub has_continuation: bool,
    pub statement_count_in_line: usize,
    pub is_first_in_block: bool,
    pub is_last_in_block: bool,
    pub block_size: usize,
    pub distance_from_function_start: usize,
    pub distance_to_function_end: usize,
    pub sibling_count: usize,
    pub child_count: usize,
    pub parent_type: String,
    pub grandparent_type: String,

    // Semantic features (28)
    pub affected_variable: Option<String>,
    pub operation_type: String,
    pub is_read_operation: bool,
    pub is_write_operation: bool,
    pub is_exec_operation: bool,
    pub is_test_operation: bool,
    pub has_side_effects: bool,
    pub is_deterministic: bool,
    pub is_idempotent: bool,
    pub variable_scope: String,
    pub data_flow_in_count: usize,
    pub data_flow_out_count: usize,
    pub control_flow_in_count: usize,
    pub control_flow_out_count: usize,
    pub uses_external_command: bool,
    pub external_command_name: Option<String>,
    pub file_operation_type: Option<String>,
    pub network_operation: bool,
    pub process_operation: bool,
    pub signal_operation: bool,
    pub arithmetic_operation: bool,
    pub string_operation: bool,
    pub array_operation: bool,
    pub associative_array_operation: bool,
    pub regex_operation: bool,
    pub date_time_operation: bool,
    pub random_operation: bool,
    pub environment_operation: bool,
}

/// Convert a boolean to f64 (1.0 for true, 0.0 for false)
#[inline]
fn bool_f64(b: bool) -> f64 {
    if b {
        1.0
    } else {
        0.0
    }
}

impl FeatureVector {
    /// Extract features from a diagnostic and source code
    pub fn extract(diagnostic: &Diagnostic, source: &str) -> Self {
        let mut features = Self::default();

        // Extract code prefix and numeric
        let code = &diagnostic.code;
        features.code_prefix = code.chars().take_while(|c| c.is_alphabetic()).collect();
        features.code_numeric = code
            .chars()
            .skip_while(|c| c.is_alphabetic())
            .collect::<String>()
            .parse()
            .unwrap_or(0);

        // Lexical features from message
        let message = &diagnostic.message;
        features.message_length = message.len();
        features.has_variable_reference = message.contains('$');
        features.has_path_reference = message.contains('/');
        features.has_command_reference = message.contains('`') || message.contains("$(");
        features.has_array_reference = message.contains('[') && message.contains(']');
        features.has_arithmetic = message.contains("$((") || message.contains("$[");
        features.has_subshell = message.contains("$(") || message.contains('`');
        features.has_pipe = message.contains('|');
        features.has_redirect = message.contains('>') || message.contains('<');
        features.has_glob = message.contains('*') || message.contains('?');
        features.has_quote_chars = message.contains('"') || message.contains('\'');
        features.word_count = message.split_whitespace().count();
        features.special_char_count = message
            .chars()
            .filter(|c| !c.is_alphanumeric() && !c.is_whitespace())
            .count();

        let alpha_count = message.chars().filter(|c| c.is_alphabetic()).count();
        let upper_count = message.chars().filter(|c| c.is_uppercase()).count();
        features.uppercase_ratio = if alpha_count > 0 {
            upper_count as f64 / alpha_count as f64
        } else {
            0.0
        };

        let total_chars = message.len();
        let digit_count = message.chars().filter(|c| c.is_numeric()).count();
        features.digit_ratio = if total_chars > 0 {
            digit_count as f64 / total_chars as f64
        } else {
            0.0
        };

        features.has_file_extension = message.contains(".sh") || message.contains(".bash");
        features.has_url = message.contains("http://") || message.contains("https://");
        features.has_env_var =
            message.contains("PATH") || message.contains("HOME") || message.contains("USER");

        // Structural features from span
        features.start_line = diagnostic.span.start_line;
        features.start_column = diagnostic.span.start_col;
        features.span_length = diagnostic
            .span
            .end_col
            .saturating_sub(diagnostic.span.start_col);
        features.is_multiline = diagnostic.span.end_line > diagnostic.span.start_line;

        // Extract features from source context
        if let Some(line) = source
            .lines()
            .nth(diagnostic.span.start_line.saturating_sub(1))
        {
            features.line_length = line.len();
            features.indentation_level = line.len() - line.trim_start().len();
            features.has_continuation = line.trim_end().ends_with('\\');
            features.statement_count_in_line = line.matches(';').count() + 1;
        }

        // Semantic classification based on code prefix
        match features.code_prefix.to_uppercase().as_str() {
            "SEC" => {
                features.has_side_effects = true;
                features.is_deterministic = false;
                features.operation_type = "security".to_string();
            }
            "DET" => {
                features.is_deterministic = false;
                features.random_operation =
                    code.contains("001") || message.to_lowercase().contains("random");
                features.date_time_operation =
                    code.contains("002") || message.to_lowercase().contains("date");
                features.process_operation =
                    code.contains("003") || message.to_lowercase().contains("pid");
                features.operation_type = "determinism".to_string();
            }
            "IDEM" => {
                features.is_idempotent = false;
                features.has_side_effects = true;
                features.operation_type = "idempotency".to_string();
            }
            "SC" => {
                features.operation_type = "shellcheck".to_string();
                // SC2086 is quoting
                if code.contains("2086") {
                    features.string_operation = true;
                }
            }
            _ => {
                features.operation_type = "unknown".to_string();
            }
        }

        features
    }

    /// Convert to f64 array for distance calculations
    pub fn to_vec(&self) -> Vec<f64> {
        vec![
            self.code_numeric as f64,
            self.message_length as f64,
            bool_f64(self.has_variable_reference),
            bool_f64(self.has_path_reference),
            bool_f64(self.has_command_reference),
            self.word_count as f64,
            self.special_char_count as f64,
            self.uppercase_ratio,
            self.digit_ratio,
            self.span_length as f64,
            self.start_line as f64,
            bool_f64(self.is_multiline),
            self.nesting_depth as f64,
            bool_f64(self.in_function),
            bool_f64(self.in_loop),
            bool_f64(self.in_conditional),
            self.line_length as f64,
            self.indentation_level as f64,
            bool_f64(self.has_side_effects),
            bool_f64(self.is_deterministic),
            bool_f64(self.is_idempotent),
            bool_f64(self.random_operation),
            bool_f64(self.date_time_operation),
            bool_f64(self.process_operation),
        ]
    }
}

/// Fix pattern with success tracking (ML-009)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixPattern {
    pub id: String,
    pub category: ShellErrorCategory,
    pub pattern_name: String,
    pub regex_match: String,
    pub replacement_template: String,
    pub success_rate: f64,
    pub total_applications: usize,
    pub accepted_count: usize,
    pub rejected_count: usize,
    pub confidence: f64,
    pub description: String,
}

impl FixPattern {
    /// Create a new pattern
    pub fn new(
        id: &str,
        category: ShellErrorCategory,
        name: &str,
        regex: &str,
        replacement: &str,
        description: &str,
    ) -> Self {
        Self {
            id: id.to_string(),
            category,
            pattern_name: name.to_string(),
            regex_match: regex.to_string(),
            replacement_template: replacement.to_string(),
            success_rate: 0.9, // Bootstrap confidence
            total_applications: 0,
            accepted_count: 0,
            rejected_count: 0,
            confidence: 0.9,
            description: description.to_string(),
        }
    }

    /// Record that this fix was accepted
    pub fn record_accepted(&mut self) {
        self.total_applications += 1;
        self.accepted_count += 1;
        self.update_confidence();
    }

    /// Record that this fix was rejected
    pub fn record_rejected(&mut self) {
        self.total_applications += 1;
        self.rejected_count += 1;
        self.update_confidence();
    }

    fn update_confidence(&mut self) {
        if self.total_applications > 0 {
            self.success_rate = self.accepted_count as f64 / self.total_applications as f64;
            // Bayesian update: confidence increases with more data
            let n = self.total_applications as f64;
            self.confidence = self.success_rate * (1.0 - 1.0 / (n + 10.0));
        }
    }
}

/// Bootstrap the pattern library with 15 initial patterns (ML-009)
pub fn bootstrap_patterns() -> Vec<FixPattern> {
    vec![
        // Quoting patterns (5)
        FixPattern::new(
            "PAT-001",
            ShellErrorCategory::MissingQuotes,
            "quote_variable",
            r"\$(\w+)",
            r#""$$1""#,
            "Add double quotes around variable expansion",
        ),
        FixPattern::new(
            "PAT-002",
            ShellErrorCategory::MissingQuotes,
            "quote_command_sub",
            r"\$\(([^)]+)\)",
            r#""$$($$1)""#,
            "Add double quotes around command substitution",
        ),
        FixPattern::new(
            "PAT-003",
            ShellErrorCategory::WordSplitting,
            "quote_array_element",
            r"\$\{(\w+)\[([^\]]+)\]\}",
            r#""$${$$1[$$2]}""#,
            "Add double quotes around array element access",
        ),
        FixPattern::new(
            "PAT-004",
            ShellErrorCategory::GlobbingRisk,
            "quote_glob",
            r"(\*|\?)",
            r#""$$1""#,
            "Quote glob characters to prevent expansion",
        ),
        FixPattern::new(
            "PAT-005",
            ShellErrorCategory::MissingQuotes,
            "quote_path",
            r"(\$\w+/[^\s]+)",
            r#""$$1""#,
            "Quote path with variable",
        ),
        // Determinism patterns (4)
        FixPattern::new(
            "PAT-006",
            ShellErrorCategory::NonDeterministicRandom,
            "seed_random",
            r"\$RANDOM",
            r"$${SEED:-42}",
            "Replace $RANDOM with seeded value",
        ),
        FixPattern::new(
            "PAT-007",
            ShellErrorCategory::TimestampUsage,
            "fixed_timestamp",
            r"\$\(date[^)]*\)",
            r"$${TIMESTAMP:-$(date +%Y%m%d)}",
            "Replace dynamic date with fixed timestamp parameter",
        ),
        FixPattern::new(
            "PAT-008",
            ShellErrorCategory::ProcessIdDependency,
            "fixed_pid",
            r"\$\$",
            r"$${PID:-$$$$}",
            "Replace $$ with configurable PID",
        ),
        FixPattern::new(
            "PAT-009",
            ShellErrorCategory::NonDeterministicRandom,
            "uuid_seed",
            r"uuidgen",
            r"$${UUID:-$(uuidgen)}",
            "Replace uuidgen with seeded UUID",
        ),
        // Idempotency patterns (3)
        FixPattern::new(
            "PAT-010",
            ShellErrorCategory::NonIdempotentOperation,
            "mkdir_p",
            r"mkdir\s+([^-])",
            r"mkdir -p $$1",
            "Add -p flag to mkdir for idempotency",
        ),
        FixPattern::new(
            "PAT-011",
            ShellErrorCategory::UnsafeOverwrite,
            "rm_f",
            r"rm\s+([^-])",
            r"rm -f $$1",
            "Add -f flag to rm for idempotency",
        ),
        FixPattern::new(
            "PAT-012",
            ShellErrorCategory::MissingGuard,
            "ln_sf",
            r"ln\s+-s\s+",
            r"ln -sf ",
            "Add -f flag to ln -s for idempotency",
        ),
        // Security patterns (3)
        FixPattern::new(
            "PAT-013",
            ShellErrorCategory::CommandInjection,
            "safe_eval",
            r"eval\s+(.+)",
            r"# SECURITY: eval removed - $$1",
            "Remove eval to prevent command injection",
        ),
        FixPattern::new(
            "PAT-014",
            ShellErrorCategory::PathTraversal,
            "safe_path",
            r"/tmp/",
            r"$${TMPDIR:-/tmp}/",
            "Use TMPDIR variable instead of hardcoded /tmp",
        ),
        FixPattern::new(
            "PAT-015",
            ShellErrorCategory::UnsafeExpansion,
            "safe_array_at",
            r"\$\*",
            r#""$$@""#,
            "Replace $* with quoted $@ for safe expansion",
        ),
    ]
}

/// k-NN classifier for error categorization (ML-008)
pub struct KnnClassifier {
    k: usize,
    training_data: Vec<(FeatureVector, ShellErrorCategory)>,
}

impl KnnClassifier {
    /// Create a new k-NN classifier
    pub fn new(k: usize) -> Self {
        Self {
            k,
            training_data: Vec::new(),
        }
    }

    /// Add training example
    pub fn add_example(&mut self, features: FeatureVector, category: ShellErrorCategory) {
        self.training_data.push((features, category));
    }

    /// Classify a diagnostic
    pub fn classify(&self, features: &FeatureVector) -> ClassificationResult {
        if self.training_data.is_empty() {
            // Fall back to rule-based classification
            return self.rule_based_classify(features);
        }

        let target = features.to_vec();
        let mut distances: Vec<(f64, ShellErrorCategory)> = self
            .training_data
            .iter()
            .map(|(f, cat)| (self.euclidean_distance(&target, &f.to_vec()), *cat))
            .collect();

        distances.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        // Count votes from k nearest neighbors
        let mut votes: HashMap<ShellErrorCategory, usize> = HashMap::new();
        for (_, cat) in distances.iter().take(self.k) {
            *votes.entry(*cat).or_default() += 1;
        }

        // Find majority
        let (category, vote_count) = votes
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .unwrap_or((ShellErrorCategory::Unknown, 0));

        let confidence = if self.k > 0 {
            vote_count as f64 / self.k as f64
        } else {
            0.0
        };

        ClassificationResult {
            category,
            confidence,
            method: "k-NN".to_string(),
        }
    }

    /// Rule-based fallback classification
    fn rule_based_classify(&self, features: &FeatureVector) -> ClassificationResult {
        let category = classify_by_prefix(features);

        ClassificationResult {
            category,
            confidence: 0.85, // Rule-based has fixed confidence
            method: "rule-based".to_string(),
        }
    }

    fn euclidean_distance(&self, a: &[f64], b: &[f64]) -> f64 {
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f64>()
            .sqrt()
    }
}

/// Classify an error category based on the code prefix
fn classify_by_prefix(features: &FeatureVector) -> ShellErrorCategory {
    match features.code_prefix.to_uppercase().as_str() {
        "SEC" => classify_sec(features),
        "DET" => classify_det(features),
        "IDEM" => classify_idem(features),
        "SC" => classify_sc(features),
        _ => ShellErrorCategory::Unknown,
    }
}

/// Classify security (SEC) errors by code number
fn classify_sec(features: &FeatureVector) -> ShellErrorCategory {
    if features.code_numeric == 1 || features.code_numeric == 2 {
        ShellErrorCategory::CommandInjection
    } else if features.code_numeric == 10 {
        ShellErrorCategory::PathTraversal
    } else {
        ShellErrorCategory::UnsafeExpansion
    }
}

/// Classify determinism (DET) errors by operation type
fn classify_det(features: &FeatureVector) -> ShellErrorCategory {
    if features.random_operation {
        ShellErrorCategory::NonDeterministicRandom
    } else if features.date_time_operation {
        ShellErrorCategory::TimestampUsage
    } else if features.process_operation {
        ShellErrorCategory::ProcessIdDependency
    } else {
        ShellErrorCategory::NonDeterministicRandom
    }
}

/// Classify idempotency (IDEM) errors by write status
fn classify_idem(features: &FeatureVector) -> ShellErrorCategory {
    if features.is_write_operation {
        ShellErrorCategory::UnsafeOverwrite
    } else {
        ShellErrorCategory::NonIdempotentOperation
    }
}

/// Classify shellcheck (SC) errors by code number and features
fn classify_sc(features: &FeatureVector) -> ShellErrorCategory {
    if features.code_numeric == 2086 {
        ShellErrorCategory::MissingQuotes
    } else if features.has_glob {
        ShellErrorCategory::GlobbingRisk
    } else {
        ShellErrorCategory::WordSplitting
    }
}

/// Classification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationResult {
    pub category: ShellErrorCategory,
    pub confidence: f64,
    pub method: String,
}

/// Drift detector for monitoring fix acceptance (ML-010)
pub struct DriftDetector {
    window_size: usize,
    acceptance_history: Vec<bool>,
    baseline_rate: f64,
    drift_threshold: f64,
}

impl DriftDetector {
    /// Create a new drift detector
    pub fn new(window_size: usize, baseline_rate: f64, drift_threshold: f64) -> Self {
        Self {
            window_size,
            acceptance_history: Vec::new(),
            baseline_rate,
            drift_threshold,
        }
    }

    /// Record a fix acceptance/rejection
    pub fn record(&mut self, accepted: bool) {
        self.acceptance_history.push(accepted);
        if self.acceptance_history.len() > self.window_size {
            self.acceptance_history.remove(0);
        }
    }

    /// Check if drift has been detected
    pub fn detect_drift(&self) -> DriftStatus {
        if self.acceptance_history.len() < self.window_size / 2 {
            return DriftStatus::InsufficientData;
        }

        let current_rate = self.current_acceptance_rate();
        let drift = (current_rate - self.baseline_rate).abs();

        if drift > self.drift_threshold {
            if current_rate < self.baseline_rate {
                DriftStatus::NegativeDrift {
                    baseline: self.baseline_rate,
                    current: current_rate,
                }
            } else {
                DriftStatus::PositiveDrift {
                    baseline: self.baseline_rate,
                    current: current_rate,
                }
            }
        } else {
            DriftStatus::Stable { rate: current_rate }
        }
    }

    /// Get current acceptance rate
    pub fn current_acceptance_rate(&self) -> f64 {
        if self.acceptance_history.is_empty() {
            return self.baseline_rate;
        }

        let accepted = self.acceptance_history.iter().filter(|&&x| x).count();
        accepted as f64 / self.acceptance_history.len() as f64
    }

    /// Update baseline rate
    pub fn update_baseline(&mut self, new_baseline: f64) {
        self.baseline_rate = new_baseline;
    }
}

/// Drift detection status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DriftStatus {
    InsufficientData,
    Stable { rate: f64 },
    PositiveDrift { baseline: f64, current: f64 },
    NegativeDrift { baseline: f64, current: f64 },
}

impl DriftStatus {
    /// Check if retraining is recommended
    pub fn needs_retrain(&self) -> bool {
        matches!(self, DriftStatus::NegativeDrift { .. })
    }
}

/// Oracle system combining all ML components
pub struct Oracle {
    classifier: KnnClassifier,
    patterns: Vec<FixPattern>,
    drift_detector: DriftDetector,
}

impl Default for Oracle {
    fn default() -> Self {
        Self::new()
    }
}

impl Oracle {
    /// Create a new Oracle with default settings
    pub fn new() -> Self {
        Self {
            classifier: KnnClassifier::new(5),
            patterns: bootstrap_patterns(),
            drift_detector: DriftDetector::new(100, 0.85, 0.15),
        }
    }

    /// Classify a diagnostic
    pub fn classify(&self, diagnostic: &Diagnostic, source: &str) -> ClassificationResult {
        let features = FeatureVector::extract(diagnostic, source);
        self.classifier.classify(&features)
    }

    /// Get suggested fix patterns for a category
    pub fn get_patterns(&self, category: ShellErrorCategory) -> Vec<&FixPattern> {
        self.patterns
            .iter()
            .filter(|p| p.category == category)
            .collect()
    }

    /// Get the best pattern for a category
    pub fn best_pattern(&self, category: ShellErrorCategory) -> Option<&FixPattern> {
        self.patterns
            .iter()
            .filter(|p| p.category == category)
            .max_by(|a, b| {
                a.confidence
                    .partial_cmp(&b.confidence)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    /// Record fix acceptance
    pub fn record_fix_result(&mut self, pattern_id: &str, accepted: bool) {
        if let Some(pattern) = self.patterns.iter_mut().find(|p| p.id == pattern_id) {
            if accepted {
                pattern.record_accepted();
            } else {
                pattern.record_rejected();
            }
        }
        self.drift_detector.record(accepted);
    }

    /// Check drift status
    pub fn drift_status(&self) -> DriftStatus {
        self.drift_detector.detect_drift()
    }

    /// Get all patterns
    pub fn all_patterns(&self) -> &[FixPattern] {
        &self.patterns
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::linter::Span;

    fn sample_diagnostic(code: &str, message: &str) -> Diagnostic {
        Diagnostic {
            code: code.to_string(),
            severity: Severity::Warning,
            message: message.to_string(),
            span: Span::new(10, 5, 10, 20),
            fix: None,
        }
    }

    #[test]
    fn test_ml_007_feature_extraction_basic() {
        let diag = sample_diagnostic("SC2086", "Double quote to prevent globbing");
        let source = "echo $var";

        let features = FeatureVector::extract(&diag, source);

        assert_eq!(features.code_prefix, "SC");
        assert_eq!(features.code_numeric, 2086);
        assert!(features.message_length > 0);
        assert_eq!(features.operation_type, "shellcheck");
    }

    #[test]
    fn test_ml_007_feature_extraction_determinism() {
        let diag = sample_diagnostic("DET001", "Non-deterministic use of $RANDOM");
        let source = "x=$RANDOM";

        let features = FeatureVector::extract(&diag, source);

        assert_eq!(features.code_prefix, "DET");
        assert_eq!(features.code_numeric, 1);
        assert!(!features.is_deterministic);
        assert!(features.random_operation);
    }

    #[test]
    fn test_ml_007_feature_extraction_security() {
        let diag = sample_diagnostic("SEC010", "Hardcoded path /tmp detected");
        let source = "cd /tmp";

        let features = FeatureVector::extract(&diag, source);

        assert_eq!(features.code_prefix, "SEC");
        assert!(features.has_path_reference);
        assert!(features.has_side_effects);
    }

    #[test]
    fn test_ml_008_knn_rule_based() {
        let classifier = KnnClassifier::new(5);

        // Without training data, falls back to rule-based
        let diag = sample_diagnostic("SC2086", "Quote this");
        let features = FeatureVector::extract(&diag, "echo $x");

        let result = classifier.classify(&features);

        assert_eq!(result.category, ShellErrorCategory::MissingQuotes);
        assert_eq!(result.method, "rule-based");
        assert!(result.confidence > 0.0);
    }

    #[test]
    fn test_ml_008_knn_with_training() {
        let mut classifier = KnnClassifier::new(3);

        // Add training examples
        for _ in 0..5 {
            let diag = sample_diagnostic("SC2086", "Quote variable");
            let features = FeatureVector::extract(&diag, "echo $x");
            classifier.add_example(features, ShellErrorCategory::MissingQuotes);
        }

        let diag = sample_diagnostic("SC2086", "Quote variable expansion");
        let features = FeatureVector::extract(&diag, "echo $var");
        let result = classifier.classify(&features);

        assert_eq!(result.category, ShellErrorCategory::MissingQuotes);
        assert_eq!(result.method, "k-NN");
    }

    #[test]
    fn test_ml_009_bootstrap_patterns() {
        let patterns = bootstrap_patterns();

        assert_eq!(patterns.len(), 15);

        // Check categories are distributed
        let quoting = patterns
            .iter()
            .filter(|p| p.category == ShellErrorCategory::MissingQuotes)
            .count();
        let determinism = patterns
            .iter()
            .filter(|p| p.category == ShellErrorCategory::NonDeterministicRandom)
            .count();

        assert!(quoting > 0);
        assert!(determinism > 0);
    }

    #[test]
    fn test_ml_009_pattern_tracking() {
        let mut pattern = FixPattern::new(
            "TEST-001",
            ShellErrorCategory::MissingQuotes,
            "test",
            r"\$x",
            "\"$x\"",
            "Test pattern",
        );

        assert_eq!(pattern.total_applications, 0);

        pattern.record_accepted();
        pattern.record_accepted();
        pattern.record_rejected();

        assert_eq!(pattern.total_applications, 3);
        assert_eq!(pattern.accepted_count, 2);
        assert_eq!(pattern.rejected_count, 1);
        assert!((pattern.success_rate - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_ml_010_drift_detection_stable() {
        let mut detector = DriftDetector::new(10, 0.8, 0.2);

        // Add mostly accepted (matching baseline)
        for _ in 0..8 {
            detector.record(true);
        }
        for _ in 0..2 {
            detector.record(false);
        }

        match detector.detect_drift() {
            DriftStatus::Stable { rate } => assert!((rate - 0.8).abs() < 0.1),
            _ => panic!("Expected stable status"),
        }
    }

    #[test]
    fn test_ml_010_drift_detection_negative() {
        let mut detector = DriftDetector::new(10, 0.9, 0.2);

        // Add mostly rejected (below baseline)
        for _ in 0..3 {
            detector.record(true);
        }
        for _ in 0..7 {
            detector.record(false);
        }

        let status = detector.detect_drift();
        assert!(status.needs_retrain());
    }

    #[test]
    fn test_ml_010_oracle_integration() {
        let oracle = Oracle::new();

        let diag = sample_diagnostic("SC2086", "Double quote to prevent globbing");
        let result = oracle.classify(&diag, "echo $x");

        assert_eq!(result.category, ShellErrorCategory::MissingQuotes);

        let patterns = oracle.get_patterns(ShellErrorCategory::MissingQuotes);
        assert!(!patterns.is_empty());
    }

    #[test]
    fn test_ml_007_feature_vector_to_vec() {
        let features = FeatureVector {
            code_numeric: 2086,
            message_length: 30,
            has_variable_reference: true,
            ..Default::default()
        };

        let vec = features.to_vec();
        assert!(!vec.is_empty());
        assert_eq!(vec[0], 2086.0);
        assert_eq!(vec[1], 30.0);
    }

    #[test]
    fn test_shell_error_category_names() {
        assert_eq!(
            ShellErrorCategory::CommandInjection.name(),
            "Command Injection"
        );
        assert_eq!(ShellErrorCategory::MissingQuotes.name(), "Missing Quotes");
        assert_eq!(
            ShellErrorCategory::NonDeterministicRandom.name(),
            "Non-Deterministic Random"
        );
    }

    #[test]
    fn test_shell_error_category_severity() {
        assert_eq!(
            ShellErrorCategory::CommandInjection.default_severity(),
            Severity::Error
        );
        assert_eq!(
            ShellErrorCategory::StyleViolation.default_severity(),
            Severity::Info
        );
        assert_eq!(
            ShellErrorCategory::MissingQuotes.default_severity(),
            Severity::Warning
        );
    }

    // ===== Additional tests for coverage =====

    #[test]
    fn test_shell_error_category_all_names() {
        assert_eq!(ShellErrorCategory::PathTraversal.name(), "Path Traversal");
        assert_eq!(
            ShellErrorCategory::UnsafeExpansion.name(),
            "Unsafe Expansion"
        );
        assert_eq!(ShellErrorCategory::TimestampUsage.name(), "Timestamp Usage");
        assert_eq!(
            ShellErrorCategory::ProcessIdDependency.name(),
            "Process ID Dependency"
        );
        assert_eq!(
            ShellErrorCategory::NonIdempotentOperation.name(),
            "Non-Idempotent Operation"
        );
        assert_eq!(ShellErrorCategory::MissingGuard.name(), "Missing Guard");
        assert_eq!(
            ShellErrorCategory::UnsafeOverwrite.name(),
            "Unsafe Overwrite"
        );
        assert_eq!(ShellErrorCategory::GlobbingRisk.name(), "Globbing Risk");
        assert_eq!(ShellErrorCategory::WordSplitting.name(), "Word Splitting");
        assert_eq!(ShellErrorCategory::SyntaxError.name(), "Syntax Error");
        assert_eq!(ShellErrorCategory::Unknown.name(), "Unknown");
    }

    #[test]
    fn test_shell_error_category_all_severities() {
        assert_eq!(
            ShellErrorCategory::PathTraversal.default_severity(),
            Severity::Error
        );
        assert_eq!(
            ShellErrorCategory::UnsafeExpansion.default_severity(),
            Severity::Warning
        );
        assert_eq!(
            ShellErrorCategory::TimestampUsage.default_severity(),
            Severity::Warning
        );
        assert_eq!(
            ShellErrorCategory::ProcessIdDependency.default_severity(),
            Severity::Warning
        );
        assert_eq!(
            ShellErrorCategory::NonIdempotentOperation.default_severity(),
            Severity::Warning
        );
        assert_eq!(
            ShellErrorCategory::MissingGuard.default_severity(),
            Severity::Warning
        );
        assert_eq!(
            ShellErrorCategory::UnsafeOverwrite.default_severity(),
            Severity::Error
        );
        assert_eq!(
            ShellErrorCategory::GlobbingRisk.default_severity(),
            Severity::Warning
        );
        assert_eq!(
            ShellErrorCategory::WordSplitting.default_severity(),
            Severity::Warning
        );
        assert_eq!(
            ShellErrorCategory::SyntaxError.default_severity(),
            Severity::Error
        );
        assert_eq!(
            ShellErrorCategory::Unknown.default_severity(),
            Severity::Info
        );
    }

    #[test]
    fn test_feature_extraction_idem_prefix() {
        let diag = sample_diagnostic("IDEM001", "Non-idempotent mkdir operation");
        let source = "mkdir /tmp/test";

        let features = FeatureVector::extract(&diag, source);

        assert_eq!(features.code_prefix, "IDEM");
        assert_eq!(features.code_numeric, 1);
        assert!(!features.is_idempotent);
        assert!(features.has_side_effects);
        assert_eq!(features.operation_type, "idempotency");
    }

    #[test]
    fn test_feature_extraction_unknown_prefix() {
        let diag = sample_diagnostic("XYZ123", "Some unknown rule");
        let source = "echo test";

        let features = FeatureVector::extract(&diag, source);

        assert_eq!(features.code_prefix, "XYZ");
        assert_eq!(features.code_numeric, 123);
        assert_eq!(features.operation_type, "unknown");
    }

    #[test]
    fn test_feature_extraction_det_date() {
        let diag = sample_diagnostic("DET002", "Non-deterministic use of date command");
        let source = "date=$(date +%Y%m%d)";

        let features = FeatureVector::extract(&diag, source);

        assert_eq!(features.code_prefix, "DET");
        assert!(!features.is_deterministic);
        assert!(features.date_time_operation);
    }

    #[test]
    fn test_feature_extraction_det_pid() {
        let diag = sample_diagnostic("DET003", "Non-deterministic use of pid $$");
        let source = "echo $$";

        let features = FeatureVector::extract(&diag, source);

        assert_eq!(features.code_prefix, "DET");
        assert!(features.process_operation);
    }

    #[test]
    fn test_feature_extraction_lexical_features() {
        let diag = sample_diagnostic(
            "SC2086",
            "Quote $var expansion in /path/* to prevent globbing",
        );
        let source = "echo $HOME/*.txt | cat > output.log";

        let features = FeatureVector::extract(&diag, source);

        assert!(features.has_variable_reference); // $var
        assert!(features.has_path_reference); // / in message
        assert!(features.has_glob); // * in message
        assert!(features.word_count > 0);
        assert!(features.special_char_count > 0);
    }

    #[test]
    fn test_feature_extraction_command_sub() {
        let diag = sample_diagnostic("SC2086", "Quote $(command) substitution");
        let source = "x=$(echo test)";

        let features = FeatureVector::extract(&diag, source);

        assert!(features.has_command_reference);
        assert!(features.has_subshell);
    }

    #[test]
    fn test_feature_extraction_backtick_command() {
        let diag = sample_diagnostic("SC2086", "Quote `command` backtick");
        let source = "x=`echo test`";

        let features = FeatureVector::extract(&diag, source);

        assert!(features.has_command_reference);
        assert!(features.has_subshell);
    }

    #[test]
    fn test_feature_extraction_array() {
        let diag = sample_diagnostic("SC2086", "Array element [0] needs quoting");
        let source = "echo ${arr[0]}";

        let features = FeatureVector::extract(&diag, source);

        assert!(features.has_array_reference);
    }

    #[test]
    fn test_feature_extraction_arithmetic() {
        let diag = sample_diagnostic("SC2086", "Arithmetic $((x+1)) expansion");
        let source = "x=$((a + b))";

        let features = FeatureVector::extract(&diag, source);

        assert!(features.has_arithmetic);
    }

    #[test]
    fn test_feature_extraction_pipe_redirect() {
        let diag = sample_diagnostic("SC2086", "Pipe | and redirect > operations");
        let source = "cat file | grep test > output";

        let features = FeatureVector::extract(&diag, source);

        assert!(features.has_pipe);
        assert!(features.has_redirect);
    }

    #[test]
    fn test_feature_extraction_quotes() {
        let diag = sample_diagnostic("SC2086", "Missing quotes around \"var\" and 'literal'");
        let source = "echo \"$var\"";

        let features = FeatureVector::extract(&diag, source);

        assert!(features.has_quote_chars);
    }

    #[test]
    fn test_feature_extraction_file_extension() {
        let diag = sample_diagnostic("SC2086", "Script file.sh needs quoting");
        let source = "bash script.sh";

        let features = FeatureVector::extract(&diag, source);

        assert!(features.has_file_extension);
    }

    #[test]
    fn test_feature_extraction_url() {
        let diag = sample_diagnostic("SC2086", "Download from https://example.com");
        let source = "curl https://example.com";

        let features = FeatureVector::extract(&diag, source);

        assert!(features.has_url);
    }

    #[test]
    fn test_feature_extraction_multiline() {
        let diag = Diagnostic {
            code: "SC2086".to_string(),
            severity: Severity::Warning,
            message: "Multiline span".to_string(),
            span: Span::new(1, 1, 5, 10),
            fix: None,
        };
        let source = "line1\nline2\nline3\nline4\nline5";

        let features = FeatureVector::extract(&diag, source);

        assert!(features.is_multiline);
    }

    #[test]
    fn test_feature_extraction_continuation() {
        let diag = Diagnostic {
            code: "SC2086".to_string(),
            severity: Severity::Warning,
            message: "Line continuation".to_string(),
            span: Span::new(1, 1, 1, 20),
            fix: None,
        };
        let source = "echo hello world \\";

        let features = FeatureVector::extract(&diag, source);

        assert!(features.has_continuation);
    }

    #[test]
    fn test_feature_extraction_uppercase_ratio() {
        let diag = sample_diagnostic("SC2086", "ALL CAPS MESSAGE");
        let source = "echo test";

        let features = FeatureVector::extract(&diag, source);

        // "ALL CAPS MESSAGE" has high uppercase ratio
        assert!(features.uppercase_ratio > 0.5);
    }

    #[test]
    fn test_feature_extraction_digit_ratio() {
        let diag = sample_diagnostic("SC2086", "Error 12345 on line 67890");
        let source = "echo test";

        let features = FeatureVector::extract(&diag, source);

        // Message has many digits
        assert!(features.digit_ratio > 0.0);
    }

    #[test]
    fn test_feature_extraction_empty_message() {
        let diag = sample_diagnostic("SC2086", "");
        let source = "echo test";

        let features = FeatureVector::extract(&diag, source);

        assert_eq!(features.message_length, 0);
        assert_eq!(features.uppercase_ratio, 0.0);
        assert_eq!(features.digit_ratio, 0.0);
    }

    #[test]
    fn test_feature_extraction_sc_string_operation() {
        let diag = sample_diagnostic("SC2086", "Double quote to prevent word splitting");
        let source = "echo $x";

        let features = FeatureVector::extract(&diag, source);

        assert!(features.string_operation);
    }

    #[test]
    fn test_knn_rule_based_sec_injection() {
        let classifier = KnnClassifier::new(5);

        let diag = sample_diagnostic("SEC001", "Command injection via eval");
        let features = FeatureVector::extract(&diag, "eval $cmd");
        let result = classifier.classify(&features);

        assert_eq!(result.category, ShellErrorCategory::CommandInjection);
    }

    #[test]
    fn test_knn_rule_based_sec_traversal() {
        let classifier = KnnClassifier::new(5);

        let diag = sample_diagnostic("SEC010", "Path traversal detected");
        let features = FeatureVector::extract(&diag, "cat ../../../etc/passwd");
        let result = classifier.classify(&features);

        assert_eq!(result.category, ShellErrorCategory::PathTraversal);
    }

    #[test]
    fn test_knn_rule_based_sec_other() {
        let classifier = KnnClassifier::new(5);

        let diag = sample_diagnostic("SEC099", "Other security issue");
        let features = FeatureVector::extract(&diag, "chmod 777 file");
        let result = classifier.classify(&features);

        assert_eq!(result.category, ShellErrorCategory::UnsafeExpansion);
    }

    #[test]
    fn test_knn_rule_based_det_default() {
        let classifier = KnnClassifier::new(5);

        // DET without specific flags falls back to random
        let diag = sample_diagnostic("DET099", "Unknown determinism issue");
        let mut features = FeatureVector::extract(&diag, "echo test");
        features.random_operation = false;
        features.date_time_operation = false;
        features.process_operation = false;

        let result = classifier.rule_based_classify(&features);

        assert_eq!(result.category, ShellErrorCategory::NonDeterministicRandom);
    }

    #[test]
    fn test_knn_rule_based_idem_write() {
        let classifier = KnnClassifier::new(5);

        let diag = sample_diagnostic("IDEM001", "Non-idempotent write");
        let mut features = FeatureVector::extract(&diag, "echo > file");
        features.is_write_operation = true;

        let result = classifier.rule_based_classify(&features);

        assert_eq!(result.category, ShellErrorCategory::UnsafeOverwrite);
    }

    #[test]
    fn test_knn_rule_based_idem_non_write() {
        let classifier = KnnClassifier::new(5);

        let diag = sample_diagnostic("IDEM002", "Non-idempotent operation");
        let mut features = FeatureVector::extract(&diag, "mkdir /tmp/test");
        features.is_write_operation = false;

        let result = classifier.rule_based_classify(&features);

        assert_eq!(result.category, ShellErrorCategory::NonIdempotentOperation);
    }

    #[test]
    fn test_knn_rule_based_sc_glob() {
        let classifier = KnnClassifier::new(5);

        let diag = sample_diagnostic("SC2035", "Glob * may expand incorrectly");
        let mut features = FeatureVector::extract(&diag, "ls *.txt");
        features.has_glob = true;

        let result = classifier.rule_based_classify(&features);

        assert_eq!(result.category, ShellErrorCategory::GlobbingRisk);
    }

    #[test]
    fn test_knn_rule_based_sc_word_split() {
        let classifier = KnnClassifier::new(5);

        let diag = sample_diagnostic("SC2087", "Word splitting issue");
        let mut features = FeatureVector::extract(&diag, "echo $x");
        features.has_glob = false;

        let result = classifier.rule_based_classify(&features);

        assert_eq!(result.category, ShellErrorCategory::WordSplitting);
    }

    #[test]
    fn test_knn_rule_based_unknown() {
        let classifier = KnnClassifier::new(5);

        let diag = sample_diagnostic("UNKNOWN123", "Unknown rule");
        let features = FeatureVector::extract(&diag, "echo test");

        let result = classifier.rule_based_classify(&features);

        assert_eq!(result.category, ShellErrorCategory::Unknown);
    }

    #[test]
    fn test_knn_euclidean_distance() {
        let classifier = KnnClassifier::new(5);

        let a = vec![0.0, 0.0, 0.0];
        let b = vec![3.0, 4.0, 0.0];

        let distance = classifier.euclidean_distance(&a, &b);
        assert!((distance - 5.0).abs() < 0.0001);
    }

    #[test]
    fn test_drift_detector_insufficient_data() {
        let detector = DriftDetector::new(10, 0.8, 0.2);

        // No data recorded yet
        match detector.detect_drift() {
            DriftStatus::InsufficientData => (),
            _ => panic!("Expected InsufficientData"),
        }
    }

    #[test]
    fn test_drift_detector_positive_drift() {
        let mut detector = DriftDetector::new(10, 0.5, 0.2);

        // All accepted (above baseline of 0.5)
        for _ in 0..10 {
            detector.record(true);
        }

        match detector.detect_drift() {
            DriftStatus::PositiveDrift { baseline, current } => {
                assert!((baseline - 0.5).abs() < 0.01);
                assert!((current - 1.0).abs() < 0.01);
            }
            other => panic!("Expected PositiveDrift, got {:?}", other),
        }
    }

    #[test]
    fn test_drift_detector_update_baseline() {
        let mut detector = DriftDetector::new(10, 0.8, 0.2);

        detector.update_baseline(0.9);
        assert!((detector.current_acceptance_rate() - 0.9).abs() < 0.01); // Returns baseline when empty
    }

    #[test]
    fn test_drift_detector_window_overflow() {
        let mut detector = DriftDetector::new(5, 0.8, 0.2);

        // Add more than window size
        for _ in 0..10 {
            detector.record(true);
        }

        // Should only keep last 5
        assert_eq!(detector.acceptance_history.len(), 5);
    }

    #[test]
    fn test_drift_status_needs_retrain() {
        assert!(!DriftStatus::InsufficientData.needs_retrain());
        assert!(!DriftStatus::Stable { rate: 0.8 }.needs_retrain());
        assert!(!DriftStatus::PositiveDrift {
            baseline: 0.8,
            current: 0.9
        }
        .needs_retrain());
        assert!(DriftStatus::NegativeDrift {
            baseline: 0.8,
            current: 0.5
        }
        .needs_retrain());
    }

    #[test]
    fn test_fix_pattern_new() {
        let pattern = FixPattern::new(
            "TEST-001",
            ShellErrorCategory::MissingQuotes,
            "test_pattern",
            r"\$x",
            "\"$x\"",
            "Test description",
        );

        assert_eq!(pattern.id, "TEST-001");
        assert_eq!(pattern.category, ShellErrorCategory::MissingQuotes);
        assert_eq!(pattern.pattern_name, "test_pattern");
        assert_eq!(pattern.regex_match, r"\$x");
        assert_eq!(pattern.replacement_template, "\"$x\"");
        assert_eq!(pattern.description, "Test description");
        assert_eq!(pattern.total_applications, 0);
        assert!((pattern.confidence - 0.9).abs() < 0.01);
    }

    #[test]
    fn test_fix_pattern_confidence_calculation() {
        let mut pattern = FixPattern::new(
            "TEST-002",
            ShellErrorCategory::MissingQuotes,
            "test",
            r"\$x",
            "\"$x\"",
            "Test",
        );

        // Record 10 applications: 9 accepted, 1 rejected
        for _ in 0..9 {
            pattern.record_accepted();
        }
        pattern.record_rejected();

        assert_eq!(pattern.total_applications, 10);
        assert_eq!(pattern.accepted_count, 9);
        assert_eq!(pattern.rejected_count, 1);
        assert!((pattern.success_rate - 0.9).abs() < 0.01);
        // Confidence should be less than success_rate due to Bayesian update
        assert!(pattern.confidence < pattern.success_rate);
        assert!(pattern.confidence > 0.0);
    }

    #[test]
    fn test_oracle_default() {
        let oracle = Oracle::default();

        assert!(!oracle.all_patterns().is_empty());
    }

    #[test]
    fn test_oracle_best_pattern() {
        let oracle = Oracle::new();

        let best = oracle.best_pattern(ShellErrorCategory::MissingQuotes);
        assert!(best.is_some());
        let pattern = best.unwrap();
        assert_eq!(pattern.category, ShellErrorCategory::MissingQuotes);
    }

    #[test]
    fn test_oracle_best_pattern_none() {
        let oracle = Oracle::new();

        // SyntaxError has no patterns in bootstrap
        let best = oracle.best_pattern(ShellErrorCategory::SyntaxError);
        assert!(best.is_none());
    }

    #[test]
    fn test_oracle_record_fix_result_accepted() {
        let mut oracle = Oracle::new();

        // Get initial state of a pattern
        let initial_accepted = oracle.all_patterns()[0].accepted_count;

        // Record accepted fix for first pattern
        let pattern_id = oracle.all_patterns()[0].id.clone();
        oracle.record_fix_result(&pattern_id, true);

        // Find pattern and verify it was updated
        let pattern = oracle
            .all_patterns()
            .iter()
            .find(|p| p.id == pattern_id)
            .unwrap();
        assert_eq!(pattern.accepted_count, initial_accepted + 1);
    }

    #[test]
    fn test_oracle_record_fix_result_rejected() {
        let mut oracle = Oracle::new();

        let initial_rejected = oracle.all_patterns()[0].rejected_count;
        let pattern_id = oracle.all_patterns()[0].id.clone();
        oracle.record_fix_result(&pattern_id, false);

        let pattern = oracle
            .all_patterns()
            .iter()
            .find(|p| p.id == pattern_id)
            .unwrap();
        assert_eq!(pattern.rejected_count, initial_rejected + 1);
    }

    #[test]
    fn test_oracle_record_fix_result_unknown_pattern() {
        let mut oracle = Oracle::new();

        // Recording for unknown pattern should not panic
        oracle.record_fix_result("NONEXISTENT-999", true);
    }

    #[test]
    fn test_oracle_drift_status() {
        let oracle = Oracle::new();

        // Should return InsufficientData initially
        match oracle.drift_status() {
            DriftStatus::InsufficientData => (),
            _ => panic!("Expected InsufficientData for new Oracle"),
        }
    }

    #[test]
    fn test_oracle_classify_det() {
        let oracle = Oracle::new();

        let diag = sample_diagnostic("DET001", "Non-deterministic $RANDOM usage");
        let result = oracle.classify(&diag, "x=$RANDOM");

        assert_eq!(result.category, ShellErrorCategory::NonDeterministicRandom);
    }

    #[test]
    fn test_oracle_classify_sec() {
        let oracle = Oracle::new();

        let diag = sample_diagnostic("SEC001", "Command injection risk");
        let result = oracle.classify(&diag, "eval $cmd");

        assert_eq!(result.category, ShellErrorCategory::CommandInjection);
    }

    #[test]
    fn test_oracle_get_patterns_multiple() {
        let oracle = Oracle::new();

        let patterns = oracle.get_patterns(ShellErrorCategory::MissingQuotes);

        // Should have multiple quote-related patterns
        assert!(patterns.len() >= 3);
    }

    #[test]
    fn test_oracle_get_patterns_empty() {
        let oracle = Oracle::new();

        let patterns = oracle.get_patterns(ShellErrorCategory::SyntaxError);

        assert!(patterns.is_empty());
    }

    #[test]
    fn test_classification_result_clone() {
        let result = ClassificationResult {
            category: ShellErrorCategory::MissingQuotes,
            confidence: 0.95,
            method: "k-NN".to_string(),
        };

        let cloned = result.clone();
        assert_eq!(cloned.category, ShellErrorCategory::MissingQuotes);
        assert!((cloned.confidence - 0.95).abs() < 0.001);
        assert_eq!(cloned.method, "k-NN");
    }

    #[test]
    fn test_fix_pattern_clone() {
        let pattern = FixPattern::new(
            "TEST-003",
            ShellErrorCategory::MissingQuotes,
            "test",
            r"\$x",
            "\"$x\"",
            "Test",
        );

        let cloned = pattern.clone();
        assert_eq!(cloned.id, pattern.id);
        assert_eq!(cloned.category, pattern.category);
    }

    #[test]
    fn test_feature_vector_default() {
        let features = FeatureVector::default();

        assert!(features.code_prefix.is_empty());
        assert_eq!(features.code_numeric, 0);
        assert_eq!(features.message_length, 0);
        assert!(!features.has_variable_reference);
    }

    #[test]
    fn test_feature_vector_clone() {
        let features = FeatureVector {
            code_prefix: "SC".to_string(),
            code_numeric: 2086,
            message_length: 100,
            ..Default::default()
        };

        let cloned = features.clone();
        assert_eq!(cloned.code_prefix, "SC");
        assert_eq!(cloned.code_numeric, 2086);
    }

    #[test]
    fn test_knn_k_zero() {
        let classifier = KnnClassifier::new(0);

        let diag = sample_diagnostic("SC2086", "Test");
        let features = FeatureVector::extract(&diag, "echo $x");
        let result = classifier.classify(&features);

        // With k=0 and no training data, should fall back to rule-based
        assert_eq!(result.method, "rule-based");
    }

    #[test]
    fn test_bootstrap_patterns_categories() {
        let patterns = bootstrap_patterns();

        // Check all expected categories are present
        let categories: std::collections::HashSet<_> =
            patterns.iter().map(|p| p.category).collect();

        assert!(categories.contains(&ShellErrorCategory::MissingQuotes));
        assert!(categories.contains(&ShellErrorCategory::NonDeterministicRandom));
        assert!(categories.contains(&ShellErrorCategory::NonIdempotentOperation));
        assert!(categories.contains(&ShellErrorCategory::CommandInjection));
        assert!(categories.contains(&ShellErrorCategory::WordSplitting));
        assert!(categories.contains(&ShellErrorCategory::GlobbingRisk));
        assert!(categories.contains(&ShellErrorCategory::TimestampUsage));
        assert!(categories.contains(&ShellErrorCategory::ProcessIdDependency));
        assert!(categories.contains(&ShellErrorCategory::UnsafeOverwrite));
        assert!(categories.contains(&ShellErrorCategory::MissingGuard));
        assert!(categories.contains(&ShellErrorCategory::PathTraversal));
        assert!(categories.contains(&ShellErrorCategory::UnsafeExpansion));
    }

    #[test]
    fn test_shell_error_category_hash() {
        use std::collections::HashMap;
        let mut map = HashMap::new();
        map.insert(ShellErrorCategory::CommandInjection, "injection");
        map.insert(ShellErrorCategory::MissingQuotes, "quotes");

        assert_eq!(
            map.get(&ShellErrorCategory::CommandInjection),
            Some(&"injection")
        );
        assert_eq!(map.get(&ShellErrorCategory::MissingQuotes), Some(&"quotes"));
    }

    #[test]
    fn test_shell_error_category_debug() {
        let category = ShellErrorCategory::CommandInjection;
        let debug_str = format!("{:?}", category);
        assert!(debug_str.contains("CommandInjection"));
    }

    #[test]
    fn test_feature_extraction_no_source_line() {
        let diag = Diagnostic {
            code: "SC2086".to_string(),
            severity: Severity::Warning,
            message: "Test".to_string(),
            span: Span::new(999, 1, 999, 10), // Line 999 doesn't exist
            fix: None,
        };
        let source = "echo test"; // Only one line

        let features = FeatureVector::extract(&diag, source);

        // Should handle missing line gracefully
        assert_eq!(features.line_length, 0);
        assert_eq!(features.indentation_level, 0);
    }

    #[test]
    fn test_feature_extraction_semicolon_statements() {
        let diag = Diagnostic {
            code: "SC2086".to_string(),
            severity: Severity::Warning,
            message: "Test".to_string(),
            span: Span::new(1, 1, 1, 50),
            fix: None,
        };
        let source = "echo a; echo b; echo c";

        let features = FeatureVector::extract(&diag, source);

        assert_eq!(features.statement_count_in_line, 3);
    }

    #[test]
    fn test_drift_status_debug() {
        let status = DriftStatus::Stable { rate: 0.85 };
        let debug_str = format!("{:?}", status);
        assert!(debug_str.contains("Stable"));
        assert!(debug_str.contains("0.85"));
    }

    #[test]
    fn test_knn_with_mixed_training() {
        let mut classifier = KnnClassifier::new(3);

        // Add training examples for different categories
        for _ in 0..3 {
            let diag = sample_diagnostic("SC2086", "Quote variable");
            let features = FeatureVector::extract(&diag, "echo $x");
            classifier.add_example(features, ShellErrorCategory::MissingQuotes);
        }

        for _ in 0..2 {
            let diag = sample_diagnostic("DET001", "Random usage");
            let features = FeatureVector::extract(&diag, "echo $RANDOM");
            classifier.add_example(features, ShellErrorCategory::NonDeterministicRandom);
        }

        // Test classification for a quote-related issue
        let diag = sample_diagnostic("SC2086", "Quote this variable");
        let features = FeatureVector::extract(&diag, "echo $y");
        let result = classifier.classify(&features);

        // Should classify as MissingQuotes since we have more training examples for it
        assert_eq!(result.method, "k-NN");
    }
}
