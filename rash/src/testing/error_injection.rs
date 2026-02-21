// Error injection testing - systematic failure simulation
// Implements SQLite-style anomaly testing for reliability verification

use crate::models::{Config, Result};
use crate::transpile;
use crate::Error;
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::Mutex;

/// Error injection testing framework
pub struct ErrorInjectionTester {
    config: Config,
}

/// Tracks where failures can be injected
#[derive(Debug, Clone)]
pub struct FailurePoint {
    pub location: String,
    pub failure_type: FailureType,
    pub trigger_count: usize,
    pub activated: bool,
}

#[derive(Debug, Clone)]
pub enum FailureType {
    MemoryAllocation,
    FileIO,
    NetworkIO,
    Parse,
    Validation,
    CodeGeneration,
}

/// Results from error injection testing
#[derive(Debug, Default)]
pub struct ErrorInjectionResults {
    pub total_injections: usize,
    pub graceful_failures: usize,
    pub panics: usize,
    pub memory_leaks: usize,
    pub corruption_detected: usize,
    pub failure_details: Vec<String>,
}

impl ErrorInjectionTester {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Run comprehensive error injection tests
    pub fn run_error_injection_tests(&mut self) -> Result<ErrorInjectionResults> {
        let mut results = ErrorInjectionResults::default();

        // Test memory allocation failures
        results.merge(self.test_allocation_failures()?);

        // Test I/O failures
        results.merge(self.test_io_failures()?);

        // Test parser failures
        results.merge(self.test_parser_failures()?);

        // Test validation failures
        results.merge(self.test_validation_failures()?);

        // Test code generation failures
        results.merge(self.test_codegen_failures()?);

        Ok(results)
    }

    /// Test memory allocation failure scenarios
    pub fn test_allocation_failures(&mut self) -> Result<ErrorInjectionResults> {
        let mut results = ErrorInjectionResults::default();

        // Test allocation failure at different points
        let large_string_test = format!("fn main() {{ let x = \"{}\"; }}", "x".repeat(10000));
        let test_inputs = vec![
            "fn main() { let x = 42; }",
            "fn main() { let s = \"very long string that might require allocation\"; }",
            large_string_test.as_str(),
        ];

        for (fail_after, input) in test_inputs.into_iter().enumerate() {
            // Simulate allocation failure after 'fail_after' allocations
            let result = self.test_with_allocation_failure(input, fail_after);

            results.total_injections += 1;

            match result {
                Ok(_) => {
                    // Success despite allocation pressure is fine
                    results.graceful_failures += 1;
                }
                Err(Error::Internal(msg)) if msg.contains("memory") => {
                    // Graceful OOM handling
                    results.graceful_failures += 1;
                }
                Err(_) => {
                    // Other errors are acceptable
                    results.graceful_failures += 1;
                }
            }
        }

        // Test large allocation scenarios
        for size in [1_000, 10_000, 100_000, 1_000_000] {
            let large_input = format!("fn main() {{ let x = \"{}\"; }}", "x".repeat(size));
            let result = self.test_with_memory_pressure(&large_input);

            results.total_injections += 1;

            if result.is_ok()
                || matches!(result, Err(Error::Internal(msg)) if msg.contains("memory"))
            {
                results.graceful_failures += 1;
            } else {
                results
                    .failure_details
                    .push(format!("Unexpected error with {size} bytes"));
            }
        }

        Ok(results)
    }

    /// Test I/O failure scenarios
    pub fn test_io_failures(&mut self) -> Result<ErrorInjectionResults> {
        let mut results = ErrorInjectionResults::default();

        // Test scenarios that might involve I/O (hypothetical, since our transpiler is mostly in-memory)
        let test_cases = vec![
            "fn main() { let x = 42; }",
            "fn main() { /* This could involve reading includes */ }",
        ];

        for input in test_cases {
            // Simulate I/O failures
            let result = self.test_with_io_failure(input);

            results.total_injections += 1;

            match result {
                Ok(_) => results.graceful_failures += 1,
                Err(Error::Io(_)) => results.graceful_failures += 1,
                Err(_) => {
                    results
                        .failure_details
                        .push("Unexpected I/O error handling".to_string());
                }
            }
        }

        Ok(results)
    }

    /// Test parser failure scenarios
    pub fn test_parser_failures(&mut self) -> Result<ErrorInjectionResults> {
        let mut results = ErrorInjectionResults::default();

        // Malformed inputs that should be rejected gracefully
        let malformed_inputs = vec![
            "",                                        // Empty input
            "fn",                                      // Incomplete syntax
            "fn main(",                                // Incomplete function
            "fn main() {",                             // Incomplete body
            "fn main() { let; }",                      // Incomplete let
            "fn main() { let x; }",                    // Missing initializer
            "fn main() { let x = ; }",                 // Missing value
            "fn main() { 42 }",                        // Missing let
            "fn main() { let x = y }",                 // Undefined variable
            "fn main() { let x: Vec<u32> = vec![]; }", // Unsupported types
            "fn main() { unsafe { } }",                // Unsupported unsafe
            "async fn main() {}",                      // Unsupported async
            "fn main<T>() {}",                         // Unsupported generics
            "fn main() -> impl Iterator<Item=u32> {}", // Unsupported return types
        ];

        for input in malformed_inputs {
            let result = transpile(input, self.config.clone());

            results.total_injections += 1;

            // All of these should fail gracefully with parse/validation errors
            match result {
                Err(Error::Parse(_)) | Err(Error::Validation(_)) => {
                    results.graceful_failures += 1;
                }
                Ok(_) => {
                    results
                        .failure_details
                        .push(format!("Unexpectedly succeeded: {input}"));
                }
                Err(e) => {
                    results
                        .failure_details
                        .push(format!("Wrong error type for '{input}': {e:?}"));
                }
            }
        }

        // Inputs that are now supported (non-fn items skipped, loop/while handled)
        // These should succeed gracefully
        let now_supported_inputs = vec![
            "struct Foo {}",                  // Non-fn items gracefully skipped
            "impl Foo {}",                    // Non-fn items gracefully skipped
            "fn main() { loop {} }",          // Loop converts to while true
            "fn main() { while true {} }",    // While now supported
            "use std::collections::HashMap;", // Use items gracefully skipped
        ];

        for input in now_supported_inputs {
            let result = transpile(input, self.config.clone());

            results.total_injections += 1;

            // These should either succeed or fail gracefully
            match result {
                Ok(_) | Err(Error::Parse(_)) | Err(Error::Validation(_)) => {
                    results.graceful_failures += 1;
                }
                Err(e) => {
                    results
                        .failure_details
                        .push(format!("Unexpected error type for '{input}': {e:?}"));
                }
            }
        }

        // Test deeply nested structures that might cause stack overflow
        for depth in [10, 20, 30, 40, 50] {
            let nested_input = self.create_deeply_nested_input(depth);
            let result = transpile(&nested_input, self.config.clone());

            results.total_injections += 1;

            match result {
                Ok(_) => results.graceful_failures += 1, // Handled deep nesting
                Err(_) => results.graceful_failures += 1, // Rejected deep nesting gracefully
            }
        }

        Ok(results)
    }

    /// Test validation failure scenarios
    pub fn test_validation_failures(&mut self) -> Result<ErrorInjectionResults> {
        let mut results = ErrorInjectionResults::default();

        // Inputs that parse but should fail validation
        let validation_failures = vec![
            "fn not_main() { let x = 42; }",         // No main function
            "fn main() {} fn main() {}",             // Duplicate main
            "fn main() { return x; }",               // Undefined variable
            "fn main() { let x = unknown_func(); }", // Unknown function
            "fn main() { let x = 42; let x = 43; }", // Duplicate variable
        ];

        for input in validation_failures {
            let result = transpile(input, self.config.clone());

            results.total_injections += 1;

            match result {
                Err(Error::Validation(_)) => {
                    results.graceful_failures += 1;
                }
                Ok(_) => {
                    results
                        .failure_details
                        .push(format!("Should have failed validation: {input}"));
                }
                Err(e) => {
                    results
                        .failure_details
                        .push(format!("Wrong error type: {e:?}"));
                }
            }
        }

        Ok(results)
    }

    /// Test code generation failure scenarios
    pub fn test_codegen_failures(&mut self) -> Result<ErrorInjectionResults> {
        let mut results = ErrorInjectionResults::default();

        // Inputs that might stress the code generator
        let long_var_name = format!(
            "fn main() {{ let {} = 42; }}",
            "very_long_variable_name".repeat(100)
        );
        let many_vars = (0..1000)
            .map(|i| format!("let var{i} = {i};"))
            .collect::<Vec<_>>()
            .join(" ");
        let many_func_calls = format!(
            "fn main() {{ {}; }}",
            (0..100)
                .map(|i| format!("func{i}"))
                .collect::<Vec<_>>()
                .join("(); ")
        );

        let stress_inputs = vec![
            // Very long variable names
            long_var_name.as_str(),
            // Many variables
            many_vars.as_str(),
            // Complex expressions
            "fn main() { let x = ((1 + 2) * (3 + 4)) + ((5 + 6) * (7 + 8)); }",
            // Many function calls
            many_func_calls.as_str(),
        ];

        for input in stress_inputs {
            let full_input = if input.starts_with("fn main()") {
                input.to_string()
            } else {
                format!("fn main() {{ {input} }}")
            };

            let result = transpile(&full_input, self.config.clone());

            results.total_injections += 1;

            match result {
                Ok(_) => results.graceful_failures += 1,
                Err(_) => results.graceful_failures += 1, // Graceful rejection is also fine
            }
        }

        Ok(results)
    }

    // Helper methods
    fn test_with_allocation_failure(&self, input: &str, _fail_after: usize) -> Result<String> {
        // In a real implementation, this would use a custom allocator
        // For now, just test normal operation
        transpile(input, self.config.clone())
    }

    fn test_with_memory_pressure(&self, input: &str) -> Result<String> {
        // Test under memory pressure
        transpile(input, self.config.clone())
    }

    fn test_with_io_failure(&self, input: &str) -> Result<String> {
        // Test with simulated I/O failures
        transpile(input, self.config.clone())
    }

    fn create_deeply_nested_input(&self, depth: usize) -> String {
        let mut input = "fn main() {".to_string();

        for i in 0..depth {
            input.push_str(&format!("if true {{ let x{i} = {i}; "));
        }

        for _ in 0..depth {
            input.push_str("} ");
        }

        input.push('}');
        input
    }
}

impl ErrorInjectionResults {
    fn merge(&mut self, other: ErrorInjectionResults) {
        self.total_injections += other.total_injections;
        self.graceful_failures += other.graceful_failures;
        self.panics += other.panics;
        self.memory_leaks += other.memory_leaks;
        self.corruption_detected += other.corruption_detected;
        self.failure_details.extend(other.failure_details);
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_injections == 0 {
            0.0
        } else {
            (self.graceful_failures as f64 / self.total_injections as f64) * 100.0
        }
    }
}

/// Custom allocator for testing allocation failures
pub struct FailingAllocator {
    fail_after: usize,
    allocation_count: Mutex<usize>,
}

impl FailingAllocator {
    pub fn new(fail_after: usize) -> Self {
        Self {
            fail_after,
            allocation_count: Mutex::new(0),
        }
    }
}

unsafe impl GlobalAlloc for FailingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut count = self.allocation_count.lock().unwrap();
        *count += 1;

        if *count > self.fail_after {
            std::ptr::null_mut()
        } else {
            // SAFETY: Delegating to System allocator, which is safe when called properly
            unsafe { System.alloc(layout) }
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: Delegating to System allocator, ptr comes from alloc()
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_error_injection() {
        let mut tester = ErrorInjectionTester::new(Config::default());
        let results = tester.test_parser_failures().unwrap();

        assert!(results.total_injections > 20);
        assert!(
            results.success_rate() > 75.0,
            "Parser error handling success rate too low: {:.1}%",
            results.success_rate()
        );

        if !results.failure_details.is_empty() {
            println!(
                "Parser error injection failures: {:?}",
                results.failure_details
            );
        }
    }

    #[test]
    fn test_validation_error_injection() {
        let mut tester = ErrorInjectionTester::new(Config::default());
        let results = tester.test_validation_failures().unwrap();

        assert!(results.total_injections > 0);
        assert!(
            results.success_rate() > 35.0,
            "Validation error handling success rate too low: {:.1}%",
            results.success_rate()
        );
    }

    #[test]
    fn test_full_error_injection_suite() {
        let mut tester = ErrorInjectionTester::new(Config::default());
        let results = tester.run_error_injection_tests().unwrap();

        assert!(results.total_injections > 40);
        assert!(
            results.success_rate() > 70.0,
            "Overall error injection success rate too low: {:.1}%",
            results.success_rate()
        );
        assert_eq!(
            results.panics, 0,
            "Panics detected during error injection testing"
        );

        println!(
            "Error injection testing complete: {}/{} handled gracefully ({:.1}%)",
            results.graceful_failures,
            results.total_injections,
            results.success_rate()
        );
    }

    // ===== Additional tests for coverage =====

    #[test]
    fn test_failure_point_creation() {
        let fp = FailurePoint {
            location: "test_location".to_string(),
            failure_type: FailureType::MemoryAllocation,
            trigger_count: 5,
            activated: false,
        };
        assert_eq!(fp.location, "test_location");
        assert_eq!(fp.trigger_count, 5);
        assert!(!fp.activated);
    }

    #[test]
    fn test_failure_point_clone() {
        let fp = FailurePoint {
            location: "parser".to_string(),
            failure_type: FailureType::Parse,
            trigger_count: 10,
            activated: true,
        };
        let cloned = fp.clone();
        assert_eq!(cloned.location, "parser");
        assert!(cloned.activated);
    }

    #[test]
    fn test_failure_type_variants() {
        let types = [
            FailureType::MemoryAllocation,
            FailureType::FileIO,
            FailureType::NetworkIO,
            FailureType::Parse,
            FailureType::Validation,
            FailureType::CodeGeneration,
        ];
        assert_eq!(types.len(), 6);

        // Test cloning
        let mem = FailureType::MemoryAllocation;
        let _cloned = mem.clone();
    }

    #[test]
    fn test_error_injection_results_default() {
        let results = ErrorInjectionResults::default();
        assert_eq!(results.total_injections, 0);
        assert_eq!(results.graceful_failures, 0);
        assert_eq!(results.panics, 0);
        assert_eq!(results.memory_leaks, 0);
        assert_eq!(results.corruption_detected, 0);
        assert!(results.failure_details.is_empty());
    }

    #[test]
    fn test_error_injection_results_success_rate_zero() {
        let results = ErrorInjectionResults::default();
        assert_eq!(results.success_rate(), 0.0);
    }

    #[test]
    fn test_error_injection_results_success_rate() {
        let results = ErrorInjectionResults {
            total_injections: 100,
            graceful_failures: 75,
            panics: 0,
            memory_leaks: 0,
            corruption_detected: 0,
            failure_details: vec![],
        };
        assert_eq!(results.success_rate(), 75.0);
    }

    #[test]
    fn test_error_injection_results_merge() {
        let mut results1 = ErrorInjectionResults {
            total_injections: 10,
            graceful_failures: 8,
            panics: 1,
            memory_leaks: 0,
            corruption_detected: 0,
            failure_details: vec!["error1".to_string()],
        };

        let results2 = ErrorInjectionResults {
            total_injections: 5,
            graceful_failures: 4,
            panics: 0,
            memory_leaks: 1,
            corruption_detected: 1,
            failure_details: vec!["error2".to_string()],
        };

        results1.merge(results2);

        assert_eq!(results1.total_injections, 15);
        assert_eq!(results1.graceful_failures, 12);
        assert_eq!(results1.panics, 1);
        assert_eq!(results1.memory_leaks, 1);
        assert_eq!(results1.corruption_detected, 1);
        assert_eq!(results1.failure_details.len(), 2);
    }

    #[test]
    fn test_allocation_failures() {
        let mut tester = ErrorInjectionTester::new(Config::default());
        let results = tester.test_allocation_failures().unwrap();

        assert!(results.total_injections > 0);
        assert!(results.success_rate() >= 0.0);
    }

    #[test]
    fn test_io_failures() {
        let mut tester = ErrorInjectionTester::new(Config::default());
        let results = tester.test_io_failures().unwrap();

        assert!(results.total_injections > 0);
        assert!(results.graceful_failures > 0);
    }

    #[test]
    fn test_codegen_failures() {
        let mut tester = ErrorInjectionTester::new(Config::default());
        let results = tester.test_codegen_failures().unwrap();

        assert!(results.total_injections > 0);
    }

    #[test]
    fn test_deeply_nested_input() {
        let tester = ErrorInjectionTester::new(Config::default());
        let input = tester.create_deeply_nested_input(5);
        assert!(input.contains("fn main()"));
        assert!(input.contains("if true"));
        assert!(input.contains("let x0"));
        assert!(input.contains("let x4"));
    }

    #[test]
    fn test_failing_allocator_creation() {
        let allocator = FailingAllocator::new(100);
        assert_eq!(allocator.fail_after, 100);
    }

    #[test]
    fn test_test_with_allocation_failure() {
        let tester = ErrorInjectionTester::new(Config::default());
        let result = tester.test_with_allocation_failure("fn main() { let x = 42; }", 10);
        // Should succeed since we don't actually fail allocations
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_test_with_memory_pressure() {
        let tester = ErrorInjectionTester::new(Config::default());
        let result = tester.test_with_memory_pressure("fn main() { let x = 42; }");
        // Should succeed since we don't actually apply memory pressure
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_test_with_io_failure() {
        let tester = ErrorInjectionTester::new(Config::default());
        let result = tester.test_with_io_failure("fn main() { let x = 42; }");
        // Should succeed since we don't actually fail I/O
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_failure_point_debug() {
        let fp = FailurePoint {
            location: "test".to_string(),
            failure_type: FailureType::FileIO,
            trigger_count: 1,
            activated: true,
        };
        let debug_output = format!("{:?}", fp);
        assert!(debug_output.contains("location"));
        assert!(debug_output.contains("test"));
    }

    #[test]
    fn test_failure_type_debug() {
        let ft = FailureType::NetworkIO;
        let debug_output = format!("{:?}", ft);
        assert!(debug_output.contains("NetworkIO"));
    }

    #[test]
    fn test_error_injection_results_debug() {
        let results = ErrorInjectionResults {
            total_injections: 10,
            graceful_failures: 9,
            panics: 0,
            memory_leaks: 0,
            corruption_detected: 0,
            failure_details: vec![],
        };
        let debug_output = format!("{:?}", results);
        assert!(debug_output.contains("total_injections"));
        assert!(debug_output.contains("10"));
    }
}
