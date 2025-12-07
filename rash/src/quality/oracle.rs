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
            if self.has_variable_reference {
                1.0
            } else {
                0.0
            },
            if self.has_path_reference { 1.0 } else { 0.0 },
            if self.has_command_reference { 1.0 } else { 0.0 },
            self.word_count as f64,
            self.special_char_count as f64,
            self.uppercase_ratio,
            self.digit_ratio,
            self.span_length as f64,
            self.start_line as f64,
            if self.is_multiline { 1.0 } else { 0.0 },
            self.nesting_depth as f64,
            if self.in_function { 1.0 } else { 0.0 },
            if self.in_loop { 1.0 } else { 0.0 },
            if self.in_conditional { 1.0 } else { 0.0 },
            self.line_length as f64,
            self.indentation_level as f64,
            if self.has_side_effects { 1.0 } else { 0.0 },
            if self.is_deterministic { 1.0 } else { 0.0 },
            if self.is_idempotent { 1.0 } else { 0.0 },
            if self.random_operation { 1.0 } else { 0.0 },
            if self.date_time_operation { 1.0 } else { 0.0 },
            if self.process_operation { 1.0 } else { 0.0 },
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
            r#"\$(\w+)"#,
            r#""$$1""#,
            "Add double quotes around variable expansion",
        ),
        FixPattern::new(
            "PAT-002",
            ShellErrorCategory::MissingQuotes,
            "quote_command_sub",
            r#"\$\(([^)]+)\)"#,
            r#""$$($$1)""#,
            "Add double quotes around command substitution",
        ),
        FixPattern::new(
            "PAT-003",
            ShellErrorCategory::WordSplitting,
            "quote_array_element",
            r#"\$\{(\w+)\[([^\]]+)\]\}"#,
            r#""$${$$1[$$2]}""#,
            "Add double quotes around array element access",
        ),
        FixPattern::new(
            "PAT-004",
            ShellErrorCategory::GlobbingRisk,
            "quote_glob",
            r#"(\*|\?)"#,
            r#""$$1""#,
            "Quote glob characters to prevent expansion",
        ),
        FixPattern::new(
            "PAT-005",
            ShellErrorCategory::MissingQuotes,
            "quote_path",
            r#"(\$\w+/[^\s]+)"#,
            r#""$$1""#,
            "Quote path with variable",
        ),
        // Determinism patterns (4)
        FixPattern::new(
            "PAT-006",
            ShellErrorCategory::NonDeterministicRandom,
            "seed_random",
            r#"\$RANDOM"#,
            r#"$${SEED:-42}"#,
            "Replace $RANDOM with seeded value",
        ),
        FixPattern::new(
            "PAT-007",
            ShellErrorCategory::TimestampUsage,
            "fixed_timestamp",
            r#"\$\(date[^)]*\)"#,
            r#"$${TIMESTAMP:-$(date +%Y%m%d)}"#,
            "Replace dynamic date with fixed timestamp parameter",
        ),
        FixPattern::new(
            "PAT-008",
            ShellErrorCategory::ProcessIdDependency,
            "fixed_pid",
            r#"\$\$"#,
            r#"$${PID:-$$$$}"#,
            "Replace $$ with configurable PID",
        ),
        FixPattern::new(
            "PAT-009",
            ShellErrorCategory::NonDeterministicRandom,
            "uuid_seed",
            r#"uuidgen"#,
            r#"$${UUID:-$(uuidgen)}"#,
            "Replace uuidgen with seeded UUID",
        ),
        // Idempotency patterns (3)
        FixPattern::new(
            "PAT-010",
            ShellErrorCategory::NonIdempotentOperation,
            "mkdir_p",
            r#"mkdir\s+([^-])"#,
            r#"mkdir -p $$1"#,
            "Add -p flag to mkdir for idempotency",
        ),
        FixPattern::new(
            "PAT-011",
            ShellErrorCategory::UnsafeOverwrite,
            "rm_f",
            r#"rm\s+([^-])"#,
            r#"rm -f $$1"#,
            "Add -f flag to rm for idempotency",
        ),
        FixPattern::new(
            "PAT-012",
            ShellErrorCategory::MissingGuard,
            "ln_sf",
            r#"ln\s+-s\s+"#,
            r#"ln -sf "#,
            "Add -f flag to ln -s for idempotency",
        ),
        // Security patterns (3)
        FixPattern::new(
            "PAT-013",
            ShellErrorCategory::CommandInjection,
            "safe_eval",
            r#"eval\s+(.+)"#,
            r#"# SECURITY: eval removed - $$1"#,
            "Remove eval to prevent command injection",
        ),
        FixPattern::new(
            "PAT-014",
            ShellErrorCategory::PathTraversal,
            "safe_path",
            r#"/tmp/"#,
            r#"$${TMPDIR:-/tmp}/"#,
            "Use TMPDIR variable instead of hardcoded /tmp",
        ),
        FixPattern::new(
            "PAT-015",
            ShellErrorCategory::UnsafeExpansion,
            "safe_array_at",
            r#"\$\*"#,
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
        let category = match features.code_prefix.to_uppercase().as_str() {
            "SEC" => {
                if features.code_numeric == 1 || features.code_numeric == 2 {
                    ShellErrorCategory::CommandInjection
                } else if features.code_numeric == 10 {
                    ShellErrorCategory::PathTraversal
                } else {
                    ShellErrorCategory::UnsafeExpansion
                }
            }
            "DET" => {
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
            "IDEM" => {
                if features.is_write_operation {
                    ShellErrorCategory::UnsafeOverwrite
                } else {
                    ShellErrorCategory::NonIdempotentOperation
                }
            }
            "SC" => {
                if features.code_numeric == 2086 {
                    ShellErrorCategory::MissingQuotes
                } else if features.has_glob {
                    ShellErrorCategory::GlobbingRisk
                } else {
                    ShellErrorCategory::WordSplitting
                }
            }
            _ => ShellErrorCategory::Unknown,
        };

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
}
