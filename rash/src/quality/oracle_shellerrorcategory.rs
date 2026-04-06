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

}

    include!("oracle_part2_incl2.rs");
