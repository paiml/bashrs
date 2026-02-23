//! Core Test Generator Infrastructure

use super::coverage::CoverageTracker;
use super::doctests::DoctestGenerator;
use super::mutation_config::MutationConfigGenerator;
use super::property_tests::PropertyTestGenerator;
use super::unit_tests::UnitTestGenerator;
use crate::bash_parser::ast::BashAst;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TestGenError {
    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Transpile error: {0}")]
    TranspileError(String),

    #[error("Coverage insufficient: {actual:.1}% (target: {target:.1}%)")]
    CoverageInsufficient { actual: f64, target: f64 },

    #[error("Mutation score too low: {actual:.1}% (target: {target:.1}%)")]
    MutationScoreLow { actual: f64, target: f64 },

    #[error("Test generation failed: {0}")]
    GenerationFailed(String),
}

pub type TestGenResult<T> = Result<T, TestGenError>;

/// Configuration options for test generation
#[derive(Debug, Clone)]
pub struct TestGenOptions {
    /// Generate unit tests for each function
    pub generate_unit_tests: bool,

    /// Generate property tests using proptest
    pub generate_property_tests: bool,

    /// Generate mutation test configuration
    pub generate_mutation_config: bool,

    /// Generate doctests from comments
    pub generate_doctests: bool,

    /// Target coverage percentage (0-100)
    pub target_coverage: f64,

    /// Target mutation score (0-100)
    pub target_mutation_score: f64,

    /// Number of property test cases
    pub property_test_cases: usize,

    /// Use existing bashrs_runtime
    pub use_runtime: bool,
}

impl Default for TestGenOptions {
    fn default() -> Self {
        Self {
            generate_unit_tests: true,
            generate_property_tests: true,
            generate_mutation_config: true,
            generate_doctests: true,
            target_coverage: 80.0,
            target_mutation_score: 85.0,
            property_test_cases: 1000,
            use_runtime: true,
        }
    }
}

/// Main test generator orchestrator
pub struct TestGenerator {
    options: TestGenOptions,
    coverage_tracker: CoverageTracker,
    unit_gen: UnitTestGenerator,
    property_gen: PropertyTestGenerator,
    doctest_gen: DoctestGenerator,
    mutation_gen: MutationConfigGenerator,
}

impl TestGenerator {
    pub fn new(options: TestGenOptions) -> Self {
        Self {
            options: options.clone(),
            coverage_tracker: CoverageTracker::new(),
            unit_gen: UnitTestGenerator::new(),
            property_gen: PropertyTestGenerator::new(),
            doctest_gen: DoctestGenerator::new(),
            mutation_gen: MutationConfigGenerator::new(),
        }
    }

    /// Generate complete test suite from bash AST
    pub fn generate(&mut self, ast: &BashAst) -> TestGenResult<GeneratedTestSuite> {
        let mut suite = GeneratedTestSuite::new();

        // 1. Generate unit tests
        if self.options.generate_unit_tests {
            suite.unit_tests = self.unit_gen.generate_tests(ast)?;
        }

        // 2. Generate property tests
        if self.options.generate_property_tests {
            suite.property_tests = self.property_gen.generate_properties(ast)?;
        }

        // 3. Generate doctests
        if self.options.generate_doctests {
            let mut doctests = self.doctest_gen.generate_doctests(ast)?;
            doctests.extend(self.doctest_gen.extract_inline_examples(ast)?);
            suite.doctests = doctests;
        }

        // 4. Generate mutation config
        if self.options.generate_mutation_config {
            suite.mutation_config = self.mutation_gen.generate_config(ast)?;
        }

        // 5. Verify coverage
        self.coverage_tracker.analyze(&suite);
        if !self
            .coverage_tracker
            .is_sufficient(self.options.target_coverage)
        {
            return Err(TestGenError::CoverageInsufficient {
                actual: self.coverage_tracker.coverage_percentage(),
                target: self.options.target_coverage,
            });
        }

        Ok(suite)
    }

    /// Generate tests until coverage target is met
    pub fn generate_until_coverage_met(
        &mut self,
        ast: &BashAst,
    ) -> TestGenResult<GeneratedTestSuite> {
        let mut suite = self.generate(ast)?;

        // Iteratively add tests until coverage is sufficient
        let mut iterations = 0;
        const MAX_ITERATIONS: usize = 10;

        while !self
            .coverage_tracker
            .is_sufficient(self.options.target_coverage)
            && iterations < MAX_ITERATIONS
        {
            // Identify uncovered paths
            let uncovered = self.coverage_tracker.uncovered_paths();

            // Generate targeted tests for uncovered paths
            let additional = self.unit_gen.generate_targeted_tests(&uncovered)?;
            suite.unit_tests.extend(additional);

            // Re-analyze coverage
            self.coverage_tracker.analyze(&suite);
            iterations += 1;
        }

        if !self
            .coverage_tracker
            .is_sufficient(self.options.target_coverage)
        {
            return Err(TestGenError::CoverageInsufficient {
                actual: self.coverage_tracker.coverage_percentage(),
                target: self.options.target_coverage,
            });
        }

        Ok(suite)
    }

    /// Get coverage report
    pub fn coverage_report(&self) -> &CoverageTracker {
        &self.coverage_tracker
    }
}

/// Complete generated test suite
#[derive(Debug, Default)]
pub struct GeneratedTestSuite {
    pub unit_tests: Vec<super::unit_tests::UnitTest>,
    pub property_tests: Vec<super::property_tests::PropertyTest>,
    pub doctests: Vec<super::doctests::Doctest>,
    pub mutation_config: String,
}

impl GeneratedTestSuite {
    pub fn new() -> Self {
        Self::default()
    }

    /// Format as Rust test module
    pub fn to_rust_code(&self) -> String {
        let mut code = String::new();

        // Unit tests
        if !self.unit_tests.is_empty() {
            code.push_str("#[cfg(test)]\n");
            code.push_str("mod tests {\n");
            code.push_str("    use super::*;\n\n");

            for test in &self.unit_tests {
                code.push_str(&format!("    {}\n", test.to_rust_code()));
            }

            code.push_str("}\n\n");
        }

        // Property tests
        if !self.property_tests.is_empty() {
            code.push_str("#[cfg(test)]\n");
            code.push_str("mod property_tests {\n");
            code.push_str("    use super::*;\n");
            code.push_str("    use proptest::prelude::*;\n\n");

            code.push_str("    proptest! {\n");
            for test in &self.property_tests {
                code.push_str(&format!("        {}\n", test.to_rust_code()));
            }
            code.push_str("    }\n");
            code.push_str("}\n");
        }

        code
    }

    /// Write mutation config to file
    pub fn mutation_config_content(&self) -> &str {
        &self.mutation_config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bash_parser::ast::{AstMetadata, BashExpr, BashStmt, Span};

    fn make_ast(statements: Vec<BashStmt>) -> BashAst {
        BashAst {
            statements,
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        }
    }

    #[test]
    fn test_generator_creation() {
        let options = TestGenOptions::default();
        let gen = TestGenerator::new(options);

        assert_eq!(gen.options.target_coverage, 80.0);
        assert_eq!(gen.options.target_mutation_score, 85.0);
    }

    #[test]
    fn test_options_default() {
        let options = TestGenOptions::default();
        assert!(options.generate_unit_tests);
        assert!(options.generate_property_tests);
        assert!(options.generate_mutation_config);
        assert!(options.generate_doctests);
        assert_eq!(options.target_coverage, 80.0);
        assert_eq!(options.target_mutation_score, 85.0);
        assert_eq!(options.property_test_cases, 1000);
        assert!(options.use_runtime);
    }

    #[test]
    fn test_options_custom() {
        let options = TestGenOptions {
            generate_unit_tests: false,
            generate_property_tests: false,
            generate_mutation_config: false,
            generate_doctests: false,
            target_coverage: 95.0,
            target_mutation_score: 90.0,
            property_test_cases: 500,
            use_runtime: false,
        };
        assert!(!options.generate_unit_tests);
        assert_eq!(options.target_coverage, 95.0);
        assert_eq!(options.property_test_cases, 500);
    }

    #[test]
    fn test_empty_suite() {
        let suite = GeneratedTestSuite::new();
        assert!(suite.unit_tests.is_empty());
        assert!(suite.property_tests.is_empty());
        assert!(suite.doctests.is_empty());
        assert!(suite.mutation_config.is_empty());
    }

    #[test]
    fn test_suite_default() {
        let suite = GeneratedTestSuite::default();
        assert!(suite.unit_tests.is_empty());
        assert!(suite.property_tests.is_empty());
    }

    #[test]
    fn test_suite_to_rust_code() {
        let suite = GeneratedTestSuite::new();
        let code = suite.to_rust_code();

        // Empty suite should produce no test code
        assert_eq!(code, "");
    }

    #[test]
    fn test_suite_to_rust_code_with_unit_tests() {
        let mut suite = GeneratedTestSuite::new();
        suite.unit_tests.push(super::super::unit_tests::UnitTest {
            name: "test_example".to_string(),
            test_fn: "example()".to_string(),
            assertions: vec![super::super::unit_tests::Assertion::Comment(
                "Test comment".to_string(),
            )],
        });

        let code = suite.to_rust_code();
        assert!(code.contains("#[cfg(test)]"));
        assert!(code.contains("mod tests"));
        assert!(code.contains("use super::*;"));
    }

    #[test]
    fn test_suite_to_rust_code_with_property_tests() {
        use super::super::property_tests::{Generator, Property, PropertyTest};

        let mut suite = GeneratedTestSuite::new();
        suite.property_tests.push(PropertyTest {
            name: "prop_test".to_string(),
            property: Property::Determinism,
            generators: vec![Generator::String {
                pattern: "[a-zA-Z0-9]{1,20}".to_string(),
            }],
            test_cases: 100,
        });

        let code = suite.to_rust_code();
        assert!(code.contains("#[cfg(test)]"));
        assert!(code.contains("mod property_tests"));
        assert!(code.contains("proptest!"));
    }

    #[test]
    fn test_suite_mutation_config_content() {
        let mut suite = GeneratedTestSuite::new();
        suite.mutation_config = "# Mutation config".to_string();
        assert_eq!(suite.mutation_config_content(), "# Mutation config");
    }

    #[test]
    fn test_coverage_report() {
        let options = TestGenOptions::default();
        let gen = TestGenerator::new(options);
        let _report = gen.coverage_report();
        // Just verify it doesn't panic
    }

    #[test]
    fn test_test_gen_error_display() {
        let err = TestGenError::ParseError("test error".to_string());
        assert_eq!(format!("{}", err), "Parse error: test error");

        let err = TestGenError::TranspileError("transpile issue".to_string());
        assert_eq!(format!("{}", err), "Transpile error: transpile issue");

        let err = TestGenError::CoverageInsufficient {
            actual: 70.0,
            target: 80.0,
        };
        assert!(format!("{}", err).contains("70.0%"));
        assert!(format!("{}", err).contains("80.0%"));

        let err = TestGenError::MutationScoreLow {
            actual: 60.0,
            target: 85.0,
        };
        assert!(format!("{}", err).contains("60.0%"));

        let err = TestGenError::GenerationFailed("failed".to_string());
        assert_eq!(format!("{}", err), "Test generation failed: failed");
    }

    #[test]
    fn test_generator_with_function_ast() {
        // Disable checks that require actual coverage tracking
        let options = TestGenOptions {
            target_coverage: 0.0,
            ..Default::default()
        };

        let mut gen = TestGenerator::new(options);
        let ast = make_ast(vec![BashStmt::Function {
            name: "my_func".to_string(),
            body: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Literal("hello".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        }]);

        let result = gen.generate(&ast);
        assert!(result.is_ok());
        let suite = result.unwrap();
        // Should have generated some unit tests for the function
        assert!(!suite.unit_tests.is_empty());
    }

    #[test]
    fn test_generator_disabled_options() {
        let options = TestGenOptions {
            generate_unit_tests: false,
            generate_property_tests: false,
            generate_mutation_config: false,
            generate_doctests: false,
            target_coverage: 0.0,
            target_mutation_score: 0.0,
            property_test_cases: 0,
            use_runtime: false,
        };

        let mut gen = TestGenerator::new(options);
        let ast = make_ast(vec![]);

        let result = gen.generate(&ast);
        assert!(result.is_ok());
        let suite = result.unwrap();
        assert!(suite.unit_tests.is_empty());
        assert!(suite.property_tests.is_empty());
        assert!(suite.doctests.is_empty());
    }
}
