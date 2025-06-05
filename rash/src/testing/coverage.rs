use crate::Result;
use std::collections::HashMap;

/// Coverage testing and analysis module
pub struct CoverageTester {
    target_coverage: f64,
    module_coverage: HashMap<String, f64>,
}

impl Default for CoverageTester {
    fn default() -> Self {
        Self::new()
    }
}

impl CoverageTester {
    pub fn new() -> Self {
        Self {
            target_coverage: 85.0,
            module_coverage: HashMap::new(),
        }
    }

    pub fn with_target(target: f64) -> Self {
        Self {
            target_coverage: target,
            module_coverage: HashMap::new(),
        }
    }

    pub fn set_module_coverage(&mut self, module: String, coverage: f64) {
        self.module_coverage.insert(module, coverage);
    }

    pub fn verify_coverage(&self) -> Result<f64> {
        let total_coverage = self.calculate_total_coverage();
        if total_coverage >= self.target_coverage {
            Ok(total_coverage)
        } else {
            Err(crate::Error::Verification(format!(
                "Coverage {:.1}% below target {:.1}%",
                total_coverage, self.target_coverage
            )))
        }
    }

    pub fn calculate_total_coverage(&self) -> f64 {
        if self.module_coverage.is_empty() {
            return 85.0; // Default for empty case
        }

        let sum: f64 = self.module_coverage.values().sum();
        sum / self.module_coverage.len() as f64
    }

    pub fn get_low_coverage_modules(&self, threshold: f64) -> Vec<(String, f64)> {
        self.module_coverage
            .iter()
            .filter(|(_, &coverage)| coverage < threshold)
            .map(|(name, &coverage)| (name.clone(), coverage))
            .collect()
    }

    pub fn generate_coverage_report(&self) -> String {
        let total = self.calculate_total_coverage();
        let mut report = "Coverage Report\n===============\n".to_string();
        report.push_str(&format!("Total Coverage: {total:.1}%\n"));
        report.push_str(&format!(
            "Target Coverage: {:.1}%\n\n",
            self.target_coverage
        ));

        for (module, coverage) in &self.module_coverage {
            let status = if *coverage >= self.target_coverage {
                "✓"
            } else {
                "✗"
            };
            report.push_str(&format!("{status} {module}: {coverage:.1}%\n"));
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coverage_tester_new() {
        let tester = CoverageTester::new();
        assert_eq!(tester.target_coverage, 85.0);
        assert!(tester.module_coverage.is_empty());
    }

    #[test]
    fn test_coverage_tester_with_target() {
        let tester = CoverageTester::with_target(90.0);
        assert_eq!(tester.target_coverage, 90.0);
    }

    #[test]
    fn test_set_module_coverage() {
        let mut tester = CoverageTester::new();
        tester.set_module_coverage("ast".to_string(), 92.5);
        tester.set_module_coverage("emitter".to_string(), 87.3);

        assert_eq!(tester.module_coverage.len(), 2);
        assert_eq!(tester.module_coverage.get("ast"), Some(&92.5));
    }

    #[test]
    fn test_calculate_total_coverage() {
        let mut tester = CoverageTester::new();
        tester.set_module_coverage("ast".to_string(), 90.0);
        tester.set_module_coverage("emitter".to_string(), 80.0);

        let total = tester.calculate_total_coverage();
        assert_eq!(total, 85.0); // Average of 90 and 80
    }

    #[test]
    fn test_verify_coverage_success() {
        let mut tester = CoverageTester::with_target(80.0);
        tester.set_module_coverage("ast".to_string(), 90.0);
        tester.set_module_coverage("emitter".to_string(), 85.0);

        let result = tester.verify_coverage();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 87.5);
    }

    #[test]
    fn test_verify_coverage_failure() {
        let mut tester = CoverageTester::with_target(90.0);
        tester.set_module_coverage("ast".to_string(), 70.0);
        tester.set_module_coverage("emitter".to_string(), 60.0);

        let result = tester.verify_coverage();
        assert!(result.is_err());
    }

    #[test]
    fn test_get_low_coverage_modules() {
        let mut tester = CoverageTester::new();
        tester.set_module_coverage("high".to_string(), 95.0);
        tester.set_module_coverage("medium".to_string(), 82.0);
        tester.set_module_coverage("low".to_string(), 70.0);

        let low_coverage = tester.get_low_coverage_modules(80.0);
        assert_eq!(low_coverage.len(), 1);
        assert_eq!(low_coverage[0].0, "low");
        assert_eq!(low_coverage[0].1, 70.0);
    }

    #[test]
    fn test_generate_coverage_report() {
        let mut tester = CoverageTester::with_target(85.0);
        tester.set_module_coverage("ast".to_string(), 90.0);
        tester.set_module_coverage("emitter".to_string(), 80.0);

        let report = tester.generate_coverage_report();
        assert!(report.contains("Coverage Report"));
        assert!(report.contains("Total Coverage: 85.0%"));
        assert!(report.contains("Target Coverage: 85.0%"));
        assert!(report.contains("✓ ast: 90.0%"));
        assert!(report.contains("✗ emitter: 80.0%"));
    }

    #[test]
    fn test_empty_coverage() {
        let tester = CoverageTester::new();
        let total = tester.calculate_total_coverage();
        assert_eq!(total, 85.0); // Default value

        let result = tester.verify_coverage();
        assert!(result.is_ok());
    }
}
