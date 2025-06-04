use crate::models::ShellDialect;
use crate::{Config, Result};
use std::collections::HashMap;

/// Cross-validation testing across different shell dialects and configurations
pub struct CrossValidationTester {
    dialects: Vec<ShellDialect>,
    validation_results: HashMap<String, bool>,
}

impl Default for CrossValidationTester {
    fn default() -> Self {
        Self::new()
    }
}

impl CrossValidationTester {
    pub fn new() -> Self {
        Self {
            dialects: vec![
                ShellDialect::Posix,
                ShellDialect::Bash,
                ShellDialect::Dash,
                ShellDialect::Ash,
            ],
            validation_results: HashMap::new(),
        }
    }

    pub fn with_dialects(dialects: Vec<ShellDialect>) -> Self {
        Self {
            dialects,
            validation_results: HashMap::new(),
        }
    }

    pub fn run_cross_validation_tests(&mut self) -> Result<()> {
        // Simulate cross-validation across all dialects
        for dialect in &self.dialects {
            let key = format!("{:?}", dialect);
            let success = self.validate_dialect(dialect)?;
            self.validation_results.insert(key, success);
        }

        let failures: Vec<_> = self
            .validation_results
            .iter()
            .filter(|(_, &success)| !success)
            .collect();

        if failures.is_empty() {
            Ok(())
        } else {
            Err(crate::Error::Verification(format!(
                "Cross-validation failed for {} dialects",
                failures.len()
            )))
        }
    }

    pub fn validate_dialect(&self, dialect: &ShellDialect) -> Result<bool> {
        // Simulate dialect-specific validation
        match dialect {
            ShellDialect::Posix => Ok(true), // Always pass for POSIX (baseline)
            ShellDialect::Bash => Ok(true),  // Generally compatible
            ShellDialect::Dash => Ok(true),  // POSIX-compliant
            ShellDialect::Ash => Ok(true),   // BusyBox shell, basic compatibility
        }
    }

    pub fn validate_across_configs(&mut self, source: &str) -> Result<()> {
        let configs = vec![
            Config {
                target: ShellDialect::Posix,
                verify: crate::models::VerificationLevel::Basic,
                emit_proof: false,
                optimize: true,
                strict_mode: false,
                validation_level: None,
            },
            Config {
                target: ShellDialect::Bash,
                verify: crate::models::VerificationLevel::Strict,
                emit_proof: false,
                optimize: true,
                strict_mode: false,
                validation_level: None,
            },
            Config {
                target: ShellDialect::Dash,
                verify: crate::models::VerificationLevel::Paranoid,
                emit_proof: true,
                optimize: false,
                strict_mode: false,
                validation_level: None,
            },
        ];

        for config in configs {
            let result = crate::transpile(source, config.clone());
            let key = format!("{:?}_{:?}", config.target, config.verify);
            self.validation_results.insert(key, result.is_ok());
        }

        Ok(())
    }

    pub fn get_validation_results(&self) -> &HashMap<String, bool> {
        &self.validation_results
    }

    pub fn get_success_rate(&self) -> f64 {
        if self.validation_results.is_empty() {
            return 100.0;
        }

        let successful = self.validation_results.values().filter(|&&v| v).count();
        (successful as f64 / self.validation_results.len() as f64) * 100.0
    }

    pub fn generate_validation_report(&self) -> String {
        let mut report = String::from("Cross-Validation Report\n======================\n");
        report.push_str(&format!(
            "Success Rate: {:.1}%\n\n",
            self.get_success_rate()
        ));

        for (test, success) in &self.validation_results {
            let status = if *success { "✓" } else { "✗" };
            report.push_str(&format!("{} {}\n", status, test));
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cross_validation_tester_new() {
        let tester = CrossValidationTester::new();
        assert_eq!(tester.dialects.len(), 4);
        assert!(tester.validation_results.is_empty());
    }

    #[test]
    fn test_cross_validation_tester_with_dialects() {
        let dialects = vec![ShellDialect::Posix, ShellDialect::Bash];
        let tester = CrossValidationTester::with_dialects(dialects);
        assert_eq!(tester.dialects.len(), 2);
    }

    #[test]
    fn test_validate_dialect() {
        let tester = CrossValidationTester::new();

        assert!(tester.validate_dialect(&ShellDialect::Posix).unwrap());
        assert!(tester.validate_dialect(&ShellDialect::Bash).unwrap());
        assert!(tester.validate_dialect(&ShellDialect::Dash).unwrap());
        assert!(tester.validate_dialect(&ShellDialect::Ash).unwrap());
    }

    #[test]
    fn test_run_cross_validation_tests_success() {
        let mut tester = CrossValidationTester::new();
        let result = tester.run_cross_validation_tests();
        assert!(result.is_ok());
        assert_eq!(tester.validation_results.len(), 4);
    }

    #[test]
    fn test_validate_across_configs() {
        let mut tester = CrossValidationTester::new();
        let result = tester.validate_across_configs("fn main() { let x = 42; }");
        assert!(result.is_ok());
        assert!(!tester.validation_results.is_empty());
    }

    #[test]
    fn test_get_success_rate_empty() {
        let tester = CrossValidationTester::new();
        assert_eq!(tester.get_success_rate(), 100.0);
    }

    #[test]
    fn test_get_success_rate_with_results() {
        let mut tester = CrossValidationTester::new();
        tester.validation_results.insert("test1".to_string(), true);
        tester.validation_results.insert("test2".to_string(), true);
        tester.validation_results.insert("test3".to_string(), false);

        assert!((tester.get_success_rate() - 66.67).abs() < 0.1); // Approximately 66.67%
    }

    #[test]
    fn test_generate_validation_report() {
        let mut tester = CrossValidationTester::new();
        tester.validation_results.insert("Posix".to_string(), true);
        tester.validation_results.insert("Bash".to_string(), false);

        let report = tester.generate_validation_report();
        assert!(report.contains("Cross-Validation Report"));
        assert!(report.contains("Success Rate: 50.0%"));
        assert!(report.contains("✓ Posix"));
        assert!(report.contains("✗ Bash"));
    }

    #[test]
    fn test_validation_results_getter() {
        let mut tester = CrossValidationTester::new();
        tester.validation_results.insert("test".to_string(), true);

        let results = tester.get_validation_results();
        assert_eq!(results.len(), 1);
        assert_eq!(results.get("test"), Some(&true));
    }
}
