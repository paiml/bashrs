// Boundary condition testing - comprehensive edge case coverage
// Following SQLite's exhaustive boundary testing methodology

use crate::models::{Config, Result};
use crate::transpile;

/// Comprehensive boundary condition test suite
pub struct BoundaryTester {
    config: Config,
    test_count: usize,
}

impl BoundaryTester {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            test_count: 0,
        }
    }

    /// Run all boundary condition tests
    pub fn run_all_boundary_tests(&mut self) -> Result<BoundaryTestResults> {
        let mut results = BoundaryTestResults::default();

        // Integer boundary tests
        results.merge(self.test_integer_boundaries()?);

        // String boundary tests
        results.merge(self.test_string_boundaries()?);

        // Memory boundary tests
        results.merge(self.test_memory_boundaries()?);

        // Syntax boundary tests
        results.merge(self.test_syntax_boundaries()?);

        // Unicode boundary tests
        results.merge(self.test_unicode_boundaries()?);

        // Nesting boundary tests
        results.merge(self.test_nesting_boundaries()?);

        Ok(results)
    }

    /// Test integer overflow and underflow conditions
    pub fn test_integer_boundaries(&mut self) -> Result<BoundaryTestResults> {
        let mut results = BoundaryTestResults::default();

        let test_cases = vec![
            // Basic boundaries
            ("0", true),
            ("1", true),
            ("-1", false), // Negative not supported in our subset
            // u32 boundaries
            ("4294967295", true),  // u32::MAX
            ("4294967296", false), // u32::MAX + 1
            // Edge cases around powers of 2
            ("255", true),   // 2^8 - 1
            ("256", true),   // 2^8
            ("65535", true), // 2^16 - 1
            ("65536", true), // 2^16
            // Common overflow points
            ("2147483647", true), // i32::MAX
            ("2147483648", true), // i32::MAX + 1
            // Leading zeros
            ("00042", true),
            ("000000000000042", true),
            // Hex literals (should fail in our subset)
            ("0x42", false),
            ("0xFF", false),
        ];

        for (input, should_succeed) in test_cases {
            let source = format!("fn main() {{ let x = {input}; }}");
            let result = self.test_transpile(&source);

            match (result.is_ok(), should_succeed) {
                (true, true) => results.passed += 1,
                (false, false) => results.passed += 1,
                _ => {
                    results.failed += 1;
                    results.failures.push(format!(
                        "Integer boundary test failed for: {} (expected: {})",
                        input,
                        if should_succeed { "success" } else { "failure" }
                    ));
                }
            }
            results.total += 1;
        }

        // Test arithmetic boundary conditions
        let arithmetic_cases = vec![
            "let x = 2147483647; let y = x + 1;", // Potential overflow
            "let x = 0; let y = x - 1;",          // Potential underflow
            "let x = 1000000; let y = x * x;",    // Large multiplication
        ];

        for case in arithmetic_cases {
            let source = format!("fn main() {{ {case} }}");
            let result = self.test_transpile(&source);

            // These may or may not succeed depending on implementation
            if result.is_ok() {
                results.passed += 1;
            } else {
                results.failed += 1;
                results
                    .failures
                    .push(format!("Arithmetic boundary test failed: {case}"));
            }
            results.total += 1;
        }

        Ok(results)
    }

    /// Test string boundary conditions
    pub fn test_string_boundaries(&mut self) -> Result<BoundaryTestResults> {
        let mut results = BoundaryTestResults::default();

        // Empty string
        results.merge_test(self.test_transpile(r#"fn main() { let x = ""; }"#), true);

        // Single character strings
        for ch in ['a', '0', ' ', '\t', '\n'] {
            let source = format!(r#"fn main() {{ let x = "{ch}"; }}"#);
            results.merge_test(self.test_transpile(&source), true);
        }

        // Special characters that need escaping
        let special_chars = vec![
            (r#"\""#, true), // Quote
            (r#"\\"#, true), // Backslash
            (r#"\n"#, true), // Newline
            (r#"\t"#, true), // Tab
            (r#"\r"#, true), // Carriage return
        ];

        for (escape_seq, should_succeed) in special_chars {
            let source = format!(r#"fn main() {{ let x = "{escape_seq}"; }}"#);
            results.merge_test(self.test_transpile(&source), should_succeed);
        }

        // Very long strings
        let sizes = vec![1, 10, 100, 1000, 10000, 100000];
        for size in sizes {
            let long_string = "x".repeat(size);
            let source = format!(r#"fn main() {{ let x = "{long_string}"; }}"#);
            results.merge_test(self.test_transpile(&source), true);
        }

        // String with all ASCII characters
        let mut all_ascii = String::new();
        for byte in 1..128u8 {
            // Skip null character
            if byte != b'"' && byte != b'\\' {
                // Skip characters that need escaping
                all_ascii.push(byte as char);
            }
        }
        let source = format!(r#"fn main() {{ let x = "{all_ascii}"; }}"#);
        results.merge_test(self.test_transpile(&source), true);

        Ok(results)
    }

    /// Test memory allocation boundaries
    pub fn test_memory_boundaries(&mut self) -> Result<BoundaryTestResults> {
        let mut results = BoundaryTestResults::default();

        // Test with increasing numbers of variables
        for count in [1, 10, 100, 1000] {
            let mut lets = String::new();
            for i in 0..count {
                lets.push_str(&format!("let var{i} = {i}; "));
            }
            let source = format!("fn main() {{ {lets} }}");
            results.merge_test(self.test_transpile(&source), true);
        }

        // Test with increasing function parameter counts
        for param_count in [0, 1, 5, 10, 20] {
            let mut params = Vec::new();
            for i in 0..param_count {
                params.push(format!("param{i}: u32"));
            }
            let source = format!("fn main({}) {{ let x = 42; }}", params.join(", "));
            results.merge_test(self.test_transpile(&source), param_count <= 10);
            // Reasonable limit
        }

        Ok(results)
    }

    /// Test syntax boundary conditions
    pub fn test_syntax_boundaries(&mut self) -> Result<BoundaryTestResults> {
        let mut results = BoundaryTestResults::default();

        // Valid minimal cases
        let minimal_cases = vec!["fn main(){}", "fn main(){let x=1;}", "fn main(){return;}"];

        for case in minimal_cases {
            results.merge_test(self.test_transpile(case), true);
        }

        // Whitespace boundaries
        let whitespace_cases = vec![
            "fn main() { let x = 42; }",           // Normal spacing
            "fn main(){let x=42;}",                // No spaces
            "fn  main ( )  {  let  x  =  42  ; }", // Extra spaces
            "fn\nmain()\n{\nlet\nx\n=\n42;\n}",    // Newlines
            "fn\tmain()\t{\tlet\tx\t=\t42;\t}",    // Tabs
        ];

        for case in whitespace_cases {
            results.merge_test(self.test_transpile(case), true);
        }

        // Identifier length boundaries
        let id_lengths = vec![1, 2, 10, 50, 100, 255];
        for len in id_lengths {
            let long_id = "a".repeat(len);
            let source = format!("fn main() {{ let {long_id} = 42; }}");
            results.merge_test(self.test_transpile(&source), len <= 255);
        }

        // Comment boundaries (should be ignored)
        let comment_cases = vec![
            "fn main() { /* comment */ let x = 1; }",
            "fn main() { // comment\nlet x = 1; }",
            "// comment\nfn main() { let x = 1; }",
        ];

        for case in comment_cases {
            results.merge_test(self.test_transpile(case), true);
        }

        Ok(results)
    }

    /// Test Unicode boundary conditions
    pub fn test_unicode_boundaries(&mut self) -> Result<BoundaryTestResults> {
        let mut results = BoundaryTestResults::default();

        // Basic Unicode characters in strings
        let unicode_cases = vec![
            ("α", true),       // Greek letter
            ("中", true),      // Chinese character
            ("🚀", true),      // Emoji
            ("𝔘𝔫𝔦𝔠𝔬𝔡𝔢", true), // Mathematical symbols
        ];

        for (unicode_char, should_succeed) in unicode_cases {
            let source = format!(r#"fn main() {{ let x = "{unicode_char}"; }}"#);
            results.merge_test(self.test_transpile(&source), should_succeed);
        }

        // Unicode in identifiers (typically not allowed)
        let unicode_id_cases = vec![("α", false), ("test_α", false), ("café", false)];

        for (unicode_id, should_succeed) in unicode_id_cases {
            let source = format!("fn main() {{ let {unicode_id} = 42; }}");
            results.merge_test(self.test_transpile(&source), should_succeed);
        }

        // Null bytes and control characters
        let control_chars = vec!['\0', '\x01', '\x7F', '\u{FEFF}']; // null, SOH, DEL, BOM
        for ch in control_chars {
            let source = format!("fn main() {{ let x = \"{ch}\"; }}");
            // These should generally be rejected or handled safely
            results.merge_test(self.test_transpile(&source), false);
        }

        Ok(results)
    }

    /// Test nesting depth boundaries
    pub fn test_nesting_boundaries(&mut self) -> Result<BoundaryTestResults> {
        let mut results = BoundaryTestResults::default();

        // Test nested if statements
        for depth in [1, 5, 10, 20, 50] {
            let mut source = "fn main() {".to_string();

            // Build nested if statements
            for i in 0..depth {
                source.push_str(&format!("if true {{ let x{i} = {i}; "));
            }

            // Close all if statements
            for _ in 0..depth {
                source.push_str("} ");
            }
            source.push('}');

            results.merge_test(self.test_transpile(&source), depth <= 20); // Reasonable nesting limit
        }

        // Test nested function calls
        for depth in [1, 5, 10, 15] {
            let mut call_chain = "func".to_string();
            for _ in 1..depth {
                call_chain = format!("func({call_chain})");
            }
            let source = format!("fn main() {{ let x = {call_chain}; }}");
            results.merge_test(self.test_transpile(&source), depth <= 10);
        }

        // Test nested expressions
        for depth in [1, 5, 10, 20] {
            let mut expr = "1".to_string();
            for i in 1..depth {
                expr = format!("({expr} + {i})");
            }
            let source = format!("fn main() {{ let x = {expr}; }}");
            results.merge_test(self.test_transpile(&source), depth <= 15);
        }

        Ok(results)
    }

    fn test_transpile(&mut self, input: &str) -> Result<String> {
        self.test_count += 1;
        transpile(input, self.config.clone())
    }
}

/// Results from boundary testing
#[derive(Debug, Default)]
pub struct BoundaryTestResults {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub failures: Vec<String>,
}

impl BoundaryTestResults {
    fn merge(&mut self, other: BoundaryTestResults) {
        self.total += other.total;
        self.passed += other.passed;
        self.failed += other.failed;
        self.failures.extend(other.failures);
    }

    fn merge_test(&mut self, result: Result<String>, expected_success: bool) {
        self.total += 1;
        match (result.is_ok(), expected_success) {
            (true, true) | (false, false) => self.passed += 1,
            _ => {
                self.failed += 1;
                self.failures.push("Test expectation mismatch".to_string());
            }
        }
    }

    pub fn success_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.passed as f64 / self.total as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integer_boundaries() {
        let mut tester = BoundaryTester::new(Config::default());
        let results = tester.test_integer_boundaries().unwrap();

        assert!(results.total > 0);
        assert!(
            results.success_rate() > 80.0,
            "Success rate too low: {:.1}%",
            results.success_rate()
        );

        if results.failed > 0 {
            println!("Boundary test failures: {:?}", results.failures);
        }
    }

    #[test]
    fn test_string_boundaries() {
        let mut tester = BoundaryTester::new(Config::default());
        let results = tester.test_string_boundaries().unwrap();

        assert!(results.total > 0);
        assert!(
            results.success_rate() > 80.0,
            "Success rate too low: {:.1}%",
            results.success_rate()
        );
    }

    #[test]
    fn test_all_boundaries() {
        let mut tester = BoundaryTester::new(Config::default());
        let results = tester.run_all_boundary_tests().unwrap();

        assert!(results.total > 50, "Not enough boundary tests executed");
        assert!(
            results.success_rate() > 75.0,
            "Overall boundary test success rate too low: {:.1}%",
            results.success_rate()
        );

        println!(
            "Boundary testing complete: {}/{} passed ({:.1}%)",
            results.passed,
            results.total,
            results.success_rate()
        );
    }
}
