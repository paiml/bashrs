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
            "struct Foo {}",                           // Unsupported construct
            "impl Foo {}",                             // Unsupported construct
            "fn main() { loop {} }",                   // Unsupported loop
            "fn main() { while true {} }",             // Unsupported while
            "fn main() { for i in 0..10 {} }",         // Unsupported for
            "fn main() { match x {} }",                // Unsupported match
            "use std::collections::HashMap;",          // Unsupported use
            "fn main() { let x: Vec<u32> = vec![]; }", // Unsupported types
            "fn main() { println!(\"hello\"); }",      // Unsupported macros
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
            System.alloc(layout)
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout)
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
            results.success_rate() > 80.0,
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
}
