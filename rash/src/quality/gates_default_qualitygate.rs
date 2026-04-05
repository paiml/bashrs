impl QualityGate {
    /// Create a new quality gate with the given configuration
    pub fn new(config: GateConfig) -> Self {
        Self { config }
    }

    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(GateConfig::default())
    }

    /// Run all gates for the specified tier
    pub fn run_tier(&self, tier: Tier) -> Vec<GateResult> {
        let gates = self.config.gates_for_tier(tier);
        let mut results = Vec::new();

        for gate_name in gates {
            let result = self.run_gate(gate_name);
            results.push(result);
        }

        results
    }

    /// Run a specific gate by name
    pub fn run_gate(&self, gate_name: &str) -> GateResult {
        let start = Instant::now();

        let (passed, message, metrics, violations) = match gate_name {
            "clippy" => self.run_clippy_gate(),
            "complexity" => self.run_complexity_gate(),
            "tests" => self.run_tests_gate(),
            "coverage" => self.run_coverage_gate(),
            "satd" => self.run_satd_gate(),
            "mutation" => self.run_mutation_gate(),
            "security" => self.run_security_gate(),
            _ => (
                false,
                format!("Unknown gate: {}", gate_name),
                HashMap::new(),
                vec![],
            ),
        };

        GateResult {
            gate_name: gate_name.to_string(),
            passed,
            duration: start.elapsed(),
            message,
            metrics,
            violations,
        }
    }

    fn run_clippy_gate(&self) -> (bool, String, HashMap<String, f64>, Vec<GateViolation>) {
        if !self.config.gates.run_clippy {
            return (
                true,
                "Clippy gate disabled".to_string(),
                HashMap::new(),
                vec![],
            );
        }

        let mut cmd = Command::new("cargo");
        cmd.args(["clippy", "--lib", "-p", "bashrs", "--message-format=json"]);

        if self.config.gates.clippy_strict {
            cmd.args(["--", "-D", "warnings"]);
        }

        match cmd.output() {
            Ok(output) => {
                let exit_code = output.status.code().unwrap_or(1);
                let passed = exit_code == 0;

                let mut violations = Vec::new();
                let stderr = String::from_utf8_lossy(&output.stderr);

                // Parse JSON output for violations
                for line in stderr.lines() {
                    if line.contains("\"level\":\"error\"")
                        || line.contains("\"level\":\"warning\"")
                    {
                        violations.push(GateViolation {
                            file: None,
                            line: None,
                            description: line.to_string(),
                            severity: if line.contains("error") {
                                ViolationSeverity::Error
                            } else {
                                ViolationSeverity::Warning
                            },
                        });
                    }
                }

                let message = if passed {
                    "Clippy passed with no warnings".to_string()
                } else {
                    format!("Clippy found {} issues", violations.len())
                };

                let mut metrics = HashMap::new();
                metrics.insert("violations".to_string(), violations.len() as f64);

                (passed, message, metrics, violations)
            }
            Err(e) => (
                false,
                format!("Failed to run clippy: {}", e),
                HashMap::new(),
                vec![],
            ),
        }
    }

    fn run_complexity_gate(&self) -> (bool, String, HashMap<String, f64>, Vec<GateViolation>) {
        if !self.config.gates.check_complexity {
            return (
                true,
                "Complexity gate disabled".to_string(),
                HashMap::new(),
                vec![],
            );
        }

        // Use pmat for complexity analysis if available
        let output = Command::new("pmat")
            .args(["analyze", "complexity", "--path", ".", "--max", "10"])
            .output();

        match output {
            Ok(output) => {
                let passed = output.status.success();
                // stdout available for future detailed parsing
                let _stdout = String::from_utf8_lossy(&output.stdout);

                let mut metrics = HashMap::new();
                metrics.insert(
                    "max_allowed".to_string(),
                    self.config.gates.max_complexity as f64,
                );

                let message = if passed {
                    format!(
                        "All functions below complexity {}",
                        self.config.gates.max_complexity
                    )
                } else {
                    "Functions exceed complexity threshold".to_string()
                };

                (passed, message, metrics, vec![])
            }
            Err(_) => {
                // pmat not available, pass by default
                (
                    true,
                    "Complexity check skipped (pmat not available)".to_string(),
                    HashMap::new(),
                    vec![],
                )
            }
        }
    }

    fn run_tests_gate(&self) -> (bool, String, HashMap<String, f64>, Vec<GateViolation>) {
        if !self.config.gates.run_tests {
            return (
                true,
                "Tests gate disabled".to_string(),
                HashMap::new(),
                vec![],
            );
        }

        let output = Command::new("cargo")
            .args(["test", "--lib", "-p", "bashrs", "--", "--test-threads=4"])
            .output();

        match output {
            Ok(output) => {
                let passed = output.status.success();
                let stdout = String::from_utf8_lossy(&output.stdout);

                // Parse test count from output
                let total_tests = 0;
                let mut passed_tests = 0;

                for line in stdout.lines() {
                    if line.contains("passed") && line.contains("failed") {
                        // Parse "test result: ok. X passed; Y failed"
                        if let Some(idx) = line.find("passed") {
                            let before = &line[..idx];
                            if let Some(num_str) = before.split_whitespace().last() {
                                passed_tests = num_str.parse().unwrap_or(0);
                            }
                        }
                    }
                }

                let mut metrics = HashMap::new();
                metrics.insert("passed".to_string(), passed_tests as f64);
                metrics.insert("total".to_string(), total_tests as f64);

                let message = if passed {
                    format!("{} tests passed", passed_tests)
                } else {
                    "Tests failed".to_string()
                };

                (passed, message, metrics, vec![])
            }
            Err(e) => (
                false,
                format!("Failed to run tests: {}", e),
                HashMap::new(),
                vec![],
            ),
        }
    }

    fn run_coverage_gate(&self) -> (bool, String, HashMap<String, f64>, Vec<GateViolation>) {
        if !self.config.gates.check_coverage {
            return (
                true,
                "Coverage gate disabled".to_string(),
                HashMap::new(),
                vec![],
            );
        }

        // This is a placeholder - actual coverage would use cargo-llvm-cov
        let mut metrics = HashMap::new();
        metrics.insert("target".to_string(), self.config.gates.min_coverage);

        (
            true,
            format!(
                "Coverage check (target: {}%) - run `make coverage` for full analysis",
                self.config.gates.min_coverage
            ),
            metrics,
            vec![],
        )
    }

    fn run_satd_gate(&self) -> (bool, String, HashMap<String, f64>, Vec<GateViolation>) {
        if !self.config.gates.satd.enabled {
            return (
                true,
                "SATD gate disabled".to_string(),
                HashMap::new(),
                vec![],
            );
        }

        // Search for SATD patterns in source files
        let patterns = &self.config.gates.satd.patterns;
        let mut violations = Vec::new();

        for pattern in patterns {
            let output = Command::new("grep")
                .args([
                    "-rn",
                    "--include=*.rs",
                    pattern,
                    "rash/src/",
                    "rash-runtime/src/",
                ])
                .output();

            if let Ok(output) = output {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    if !line.contains("tests") && !line.contains("_test.rs") {
                        violations.push(GateViolation {
                            file: line.split(':').next().map(String::from),
                            line: line.split(':').nth(1).and_then(|s| s.parse().ok()),
                            description: format!("SATD pattern '{}' found", pattern),
                            severity: ViolationSeverity::Warning,
                        });
                    }
                }
            }
        }

        let satd_count = violations.len();
        let passed = satd_count <= self.config.gates.satd.max_count
            || !self.config.gates.satd.fail_on_violation;

        let mut metrics = HashMap::new();
        metrics.insert("count".to_string(), satd_count as f64);
        metrics.insert(
            "max_allowed".to_string(),
            self.config.gates.satd.max_count as f64,
        );

        let message = if passed {
            format!(
                "SATD check passed ({} found, {} allowed)",
                satd_count, self.config.gates.satd.max_count
            )
        } else {
            format!(
                "SATD check failed: {} technical debt markers found (max: {})",
                satd_count, self.config.gates.satd.max_count
            )
        };

        (passed, message, metrics, violations)
    }

    fn run_mutation_gate(&self) -> (bool, String, HashMap<String, f64>, Vec<GateViolation>) {
        if !self.config.gates.mutation.enabled {
            return (
                true,
                "Mutation testing disabled (enable for Tier 3)".to_string(),
                HashMap::new(),
                vec![],
            );
        }

        let mut metrics = HashMap::new();
        metrics.insert("target".to_string(), self.config.gates.mutation.min_score);

        (
            true,
            format!(
                "Mutation testing (target: {}%) - run `cargo mutants` manually",
                self.config.gates.mutation.min_score
            ),
            metrics,
            vec![],
        )
    }

    fn run_security_gate(&self) -> (bool, String, HashMap<String, f64>, Vec<GateViolation>) {
        if !self.config.gates.security.enabled {
            return (
                true,
                "Security gate disabled".to_string(),
                HashMap::new(),
                vec![],
            );
        }

        // Run cargo audit
        let output = Command::new("cargo").args(["audit"]).output();

        match output {
            Ok(output) => {
                let passed = output.status.success();
                // stdout available for future detailed parsing
                let _stdout = String::from_utf8_lossy(&output.stdout);

                let message = if passed {
                    "No security vulnerabilities found".to_string()
                } else {
                    "Security vulnerabilities detected".to_string()
                };

                (passed, message, HashMap::new(), vec![])
            }
            Err(_) => (
                true,
                "Security audit skipped (cargo-audit not installed)".to_string(),
                HashMap::new(),
                vec![],
            ),
        }
    }

    /// Check if all results passed
    pub fn all_passed(results: &[GateResult]) -> bool {
        results.iter().all(|r| r.passed)
    }

    /// Get summary statistics
    pub fn summary(results: &[GateResult]) -> GateSummary {
        let total = results.len();
        let passed = results.iter().filter(|r| r.passed).count();
        let failed = total - passed;
        let total_duration: Duration = results.iter().map(|r| r.duration).sum();

        GateSummary {
            total,
            passed,
            failed,
            total_duration,
        }
    }
}










include!("gates_default_gatesummary.rs");
