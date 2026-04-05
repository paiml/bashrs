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
#[path = "oracle_tests_extracted.rs"]
mod tests_extracted;
