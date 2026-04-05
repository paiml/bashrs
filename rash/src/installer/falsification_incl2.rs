impl FalsificationGenerator {
    /// Create new generator with default config
    pub fn new() -> Self {
        Self {
            config: FalsificationConfig::all(),
        }
    }

    /// Create generator with custom config
    pub fn with_config(config: FalsificationConfig) -> Self {
        Self { config }
    }

    /// Generate hypotheses for an installer specification
    pub fn generate_hypotheses(&self, spec: &InstallerInfo) -> Vec<FalsificationHypothesis> {
        let mut hypotheses = Vec::new();

        for step in &spec.steps {
            // Idempotency hypothesis
            if self.config.test_idempotency {
                hypotheses.push(FalsificationHypothesis {
                    id: format!("IDEM-{}", step.id),
                    claim: format!("Step '{}' is idempotent", step.name),
                    category: HypothesisCategory::Idempotency,
                    falsification_method: "Execute step twice, compare final states".to_string(),
                    step_ids: vec![step.id.clone()],
                    expected_evidence: "State after first run equals state after second run"
                        .to_string(),
                    falsifying_evidence: "States differ after repeated execution".to_string(),
                    priority: 9,
                });
            }

            // Determinism hypothesis
            if self.config.test_determinism {
                hypotheses.push(FalsificationHypothesis {
                    id: format!("DET-{}", step.id),
                    claim: format!("Step '{}' is deterministic", step.name),
                    category: HypothesisCategory::Determinism,
                    falsification_method: "Execute step with same inputs twice, compare outputs"
                        .to_string(),
                    step_ids: vec![step.id.clone()],
                    expected_evidence: "Outputs are byte-identical across runs".to_string(),
                    falsifying_evidence: "Outputs differ between runs with same inputs".to_string(),
                    priority: 9,
                });
            }

            // Rollback hypothesis
            if self.config.test_rollback && step.has_rollback {
                hypotheses.push(FalsificationHypothesis {
                    id: format!("ROLL-{}", step.id),
                    claim: format!("Rollback for '{}' is complete", step.name),
                    category: HypothesisCategory::RollbackCompleteness,
                    falsification_method:
                        "Capture state, execute step, rollback, compare to original state"
                            .to_string(),
                    step_ids: vec![step.id.clone()],
                    expected_evidence: "State after rollback equals state before execution"
                        .to_string(),
                    falsifying_evidence: "State differs after rollback".to_string(),
                    priority: 8,
                });
            }

            // Dry-run accuracy hypothesis
            if self.config.test_dry_run {
                hypotheses.push(FalsificationHypothesis {
                    id: format!("DRY-{}", step.id),
                    claim: format!("Dry-run for '{}' accurately predicts changes", step.name),
                    category: HypothesisCategory::DryRunAccuracy,
                    falsification_method:
                        "Run dry-run, capture prediction, execute, compare to actual".to_string(),
                    step_ids: vec![step.id.clone()],
                    expected_evidence: "Dry-run prediction matches actual execution".to_string(),
                    falsifying_evidence: "Prediction differs from actual changes".to_string(),
                    priority: 7,
                });
            }

            // Postcondition validity
            if self.config.test_postconditions && !step.postconditions.is_empty() {
                for (i, pc) in step.postconditions.iter().enumerate() {
                    hypotheses.push(FalsificationHypothesis {
                        id: format!("POST-{}-{}", step.id, i),
                        claim: format!("Postcondition '{}' holds after '{}'", pc, step.name),
                        category: HypothesisCategory::PostconditionValidity,
                        falsification_method: "Execute step, verify postcondition".to_string(),
                        step_ids: vec![step.id.clone()],
                        expected_evidence: format!("Postcondition '{}' is true", pc),
                        falsifying_evidence: format!("Postcondition '{}' is false", pc),
                        priority: 8,
                    });
                }
            }

            // Precondition guard
            if self.config.test_preconditions && !step.preconditions.is_empty() {
                for (i, pre) in step.preconditions.iter().enumerate() {
                    hypotheses.push(FalsificationHypothesis {
                        id: format!("PRE-{}-{}", step.id, i),
                        claim: format!(
                            "Precondition '{}' prevents invalid execution of '{}'",
                            pre, step.name
                        ),
                        category: HypothesisCategory::PreconditionGuard,
                        falsification_method:
                            "Violate precondition, attempt execution, verify failure".to_string(),
                        step_ids: vec![step.id.clone()],
                        expected_evidence: format!(
                            "Step fails when precondition '{}' is not met",
                            pre
                        ),
                        falsifying_evidence: format!(
                            "Step succeeds despite precondition '{}' being false",
                            pre
                        ),
                        priority: 7,
                    });
                }
            }

            // Performance bounds
            if self.config.test_performance {
                if let Some(max_duration) = step.max_duration_ms {
                    hypotheses.push(FalsificationHypothesis {
                        id: format!("PERF-{}", step.id),
                        claim: format!("Step '{}' completes within {}ms", step.name, max_duration),
                        category: HypothesisCategory::PerformanceBound,
                        falsification_method: "Execute step, measure duration".to_string(),
                        step_ids: vec![step.id.clone()],
                        expected_evidence: format!(
                            "Execution completes in under {}ms",
                            max_duration
                        ),
                        falsifying_evidence: format!("Execution exceeds {}ms", max_duration),
                        priority: 5,
                    });
                }
            }
        }

        // Sort by priority (highest first)
        hypotheses.sort_by(|a, b| b.priority.cmp(&a.priority));
        hypotheses
    }

    /// Generate test cases for hypotheses
    pub fn generate_tests(&self, hypotheses: &[FalsificationHypothesis]) -> Vec<FalsificationTest> {
        hypotheses
            .iter()
            .map(|h| self.generate_test_for_hypothesis(h))
            .collect()
    }

    /// Generate a test for a specific hypothesis
    fn generate_test_for_hypothesis(
        &self,
        hypothesis: &FalsificationHypothesis,
    ) -> FalsificationTest {
        let step_id = hypothesis
            .step_ids
            .first()
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());

        match hypothesis.category {
            HypothesisCategory::Idempotency => FalsificationTest {
                name: format!("test_falsify_idempotency_{}", step_id),
                hypothesis_id: hypothesis.id.clone(),
                setup: vec![TestAction::CaptureState {
                    label: "initial".to_string(),
                }],
                action: TestAction::ExecuteStep {
                    step_id: step_id.clone(),
                },
                verification: vec![Verification::StatesEqual {
                    state_a: "after_first".to_string(),
                    state_b: "after_second".to_string(),
                }],
                cleanup: vec![],
            },
            HypothesisCategory::Determinism => FalsificationTest {
                name: format!("test_falsify_determinism_{}", step_id),
                hypothesis_id: hypothesis.id.clone(),
                setup: vec![TestAction::CaptureState {
                    label: "initial".to_string(),
                }],
                action: TestAction::ExecuteStep {
                    step_id: step_id.clone(),
                },
                verification: vec![Verification::StatesEqual {
                    state_a: "output_1".to_string(),
                    state_b: "output_2".to_string(),
                }],
                cleanup: vec![],
            },
            HypothesisCategory::RollbackCompleteness => FalsificationTest {
                name: format!("test_falsify_rollback_{}", step_id),
                hypothesis_id: hypothesis.id.clone(),
                setup: vec![TestAction::CaptureState {
                    label: "before".to_string(),
                }],
                action: TestAction::ExecuteStep {
                    step_id: step_id.clone(),
                },
                verification: vec![Verification::StatesEqual {
                    state_a: "before".to_string(),
                    state_b: "after_rollback".to_string(),
                }],
                cleanup: vec![TestAction::Rollback {
                    step_id: step_id.clone(),
                }],
            },
            HypothesisCategory::DryRunAccuracy => FalsificationTest {
                name: format!("test_falsify_dry_run_{}", step_id),
                hypothesis_id: hypothesis.id.clone(),
                setup: vec![TestAction::DryRun {
                    step_id: step_id.clone(),
                }],
                action: TestAction::ExecuteStep {
                    step_id: step_id.clone(),
                },
                verification: vec![Verification::DryRunMatchesExecution {
                    step_id: step_id.clone(),
                }],
                cleanup: vec![],
            },
            HypothesisCategory::PostconditionValidity => FalsificationTest {
                name: format!("test_falsify_postcondition_{}", hypothesis.id),
                hypothesis_id: hypothesis.id.clone(),
                setup: vec![],
                action: TestAction::ExecuteStep {
                    step_id: step_id.clone(),
                },
                verification: vec![Verification::CommandSucceeds {
                    command: format!("verify_postcondition_{}", hypothesis.id),
                }],
                cleanup: vec![],
            },
            HypothesisCategory::PreconditionGuard => FalsificationTest {
                name: format!("test_falsify_precondition_{}", hypothesis.id),
                hypothesis_id: hypothesis.id.clone(),
                setup: vec![],
                action: TestAction::ExecuteStep {
                    step_id: step_id.clone(),
                },
                verification: vec![Verification::CommandFails {
                    command: format!("execute_step_{}", step_id),
                }],
                cleanup: vec![],
            },
            HypothesisCategory::PerformanceBound => FalsificationTest {
                name: format!("test_falsify_performance_{}", step_id),
                hypothesis_id: hypothesis.id.clone(),
                setup: vec![],
                action: TestAction::ExecuteStep {
                    step_id: step_id.clone(),
                },
                verification: vec![Verification::DurationBelow {
                    max_ms: 60000, // Default 1 minute
                }],
                cleanup: vec![],
            },
            HypothesisCategory::ResourceLimit => FalsificationTest {
                name: format!("test_falsify_resources_{}", step_id),
                hypothesis_id: hypothesis.id.clone(),
                setup: vec![],
                action: TestAction::ExecuteStep {
                    step_id: step_id.clone(),
                },
                verification: vec![],
                cleanup: vec![],
            },
        }
    }

    /// Generate Rust test code for hypotheses
    pub fn generate_rust_tests(&self, hypotheses: &[FalsificationHypothesis]) -> String {
        let mut code = String::new();

        code.push_str("//! Auto-generated falsification tests\n");
        code.push_str("//! Generated by bashrs falsification test generator\n\n");
        code.push_str("#[cfg(test)]\n");
        code.push_str("mod falsification_tests {\n");
        code.push_str("    use super::*;\n\n");

        for h in hypotheses {
            code.push_str(&format!("    /// FALSIFIABLE: \"{}\"\n", h.claim));
            code.push_str(&format!("    /// DISPROOF: {}\n", h.falsifying_evidence));
            code.push_str("    #[test]\n");
            code.push_str(&format!(
                "    fn test_falsify_{}() {{\n",
                h.id.to_lowercase().replace('-', "_")
            ));
            code.push_str("        // Placeholder: implement with step execution\n");
            code.push_str(&format!("        // Method: {}\n", h.falsification_method));
            code.push_str(&format!("        // Expected: {}\n", h.expected_evidence));
            code.push_str("        assert!(true, \"Implement falsification test\");\n");
            code.push_str("    }\n\n");
        }

        code.push_str("}\n");
        code
    }
}

/// Minimal installer info for test generation
#[derive(Debug, Clone, Default)]
pub struct InstallerInfo {
    /// Installer name
    pub name: String,
    /// Installer version
    pub version: String,
    /// Steps in the installer
    pub steps: Vec<StepInfo>,
}

/// Minimal step info for test generation
#[derive(Debug, Clone, Default)]
pub struct StepInfo {
    /// Step ID
    pub id: String,
    /// Step name
    pub name: String,
    /// Whether step has rollback
    pub has_rollback: bool,
    /// Preconditions
    pub preconditions: Vec<String>,
    /// Postconditions
    pub postconditions: Vec<String>,
    /// Max duration in ms
    pub max_duration_ms: Option<u64>,
}

/// Summary report of falsification testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FalsificationReport {
    /// Installer tested
    pub installer_name: String,
    /// Total hypotheses tested
    pub total_hypotheses: usize,
    /// Hypotheses that were falsified (bugs found)
    pub falsified_count: usize,
    /// Hypotheses that held (no bugs found)
    pub validated_count: usize,
    /// Tests that failed to run
    pub error_count: usize,
    /// Results by category
    pub by_category: HashMap<String, CategorySummary>,
    /// All results
    pub results: Vec<FalsificationResult>,
}

/// Summary for a category
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CategorySummary {
    /// Total tests in category
    pub total: usize,
    /// Tests that falsified hypothesis
    pub falsified: usize,
    /// Tests that validated hypothesis
    pub validated: usize,
}

impl FalsificationReport {
    /// Create report from results
    pub fn from_results(
        installer_name: &str,
        results: Vec<FalsificationResult>,
        hypotheses: &[FalsificationHypothesis],
    ) -> Self {
        let mut by_category: HashMap<String, CategorySummary> = HashMap::new();

        let mut falsified_count = 0;
        let mut validated_count = 0;
        let mut error_count = 0;

        for result in &results {
            if result.error.is_some() {
                error_count += 1;
                continue;
            }

            if result.falsified {
                falsified_count += 1;
            } else {
                validated_count += 1;
            }

            // Find hypothesis category
            if let Some(h) = hypotheses.iter().find(|h| h.id == result.hypothesis_id) {
                let cat = format!("{:?}", h.category);
                let entry = by_category.entry(cat).or_default();
                entry.total += 1;
                if result.falsified {
                    entry.falsified += 1;
                } else {
                    entry.validated += 1;
                }
            }
        }

        Self {
            installer_name: installer_name.to_string(),
            total_hypotheses: results.len(),
            falsified_count,
            validated_count,
            error_count,
            by_category,
            results,
        }
    }

    /// Format as human-readable report
    pub fn format(&self) -> String {
        let mut report = String::new();

        report.push_str(&format!("Falsification Report: {}\n", self.installer_name));
        report.push_str(&"=".repeat(50));
        report.push('\n');

        report.push_str(&format!(
            "Total hypotheses tested: {}\n",
            self.total_hypotheses
        ));
        report.push_str(&format!(
            "  ✓ Validated: {} (no bugs found)\n",
            self.validated_count
        ));
        report.push_str(&format!(
            "  ✗ Falsified: {} (bugs found!)\n",
            self.falsified_count
        ));
        if self.error_count > 0 {
            report.push_str(&format!("  ⚠ Errors: {}\n", self.error_count));
        }

        report.push_str("\nBy Category:\n");
        for (cat, summary) in &self.by_category {
            report.push_str(&format!(
                "  {}: {}/{} validated\n",
                cat, summary.validated, summary.total
            ));
        }

        if self.falsified_count > 0 {
            report.push_str("\nFalsified Hypotheses (Bugs Found):\n");
            for result in &self.results {
                if result.falsified {
                    report.push_str(&format!("  - {}\n", result.hypothesis_id));
                    for evidence in &result.evidence {
                        if !evidence.supports_hypothesis {
                            report.push_str(&format!("    → {}\n", evidence.observation));
                        }
                    }
                }
            }
        }

        report
    }
}

#[cfg(test)]
#[path = "falsification_tests_extracted.rs"]
mod tests_extracted;
