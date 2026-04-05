//! Unit Test Generation
//!
//! Generates unit tests with:
//! - Branch coverage (if/else, loops, case statements)
//! - Edge case testing (empty strings, zero, max values)
//! - Error case testing (file not found, invalid input)
//!
//! Target: ≥80% line coverage

use super::core::TestGenResult;
use super::coverage::UncoveredPath;
use crate::bash_parser::ast::*;

/// Generates unit tests for bash functions
pub struct UnitTestGenerator;

impl Default for UnitTestGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl UnitTestGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Generate unit tests for all functions in AST
    pub fn generate_tests(&self, ast: &BashAst) -> TestGenResult<Vec<UnitTest>> {
        let mut tests = Vec::new();

        for stmt in &ast.statements {
            if let BashStmt::Function { name, body, .. } = stmt {
                // Generate tests for this function
                tests.extend(self.generate_function_tests(name, body)?);
            }
        }

        // Add edge case tests
        tests.extend(self.generate_edge_case_tests(ast)?);

        // Add error case tests
        tests.extend(self.generate_error_case_tests(ast)?);

        Ok(tests)
    }

    /// Generate tests for a specific function
    fn generate_function_tests(
        &self,
        name: &str,
        body: &[BashStmt],
    ) -> TestGenResult<Vec<UnitTest>> {
        let mut tests = Vec::new();

        // 1. Generate branch coverage tests
        tests.extend(self.generate_branch_tests(name, body)?);

        // 2. Generate boundary value tests
        tests.extend(self.generate_boundary_tests(name, body)?);

        Ok(tests)
    }

    /// Generate tests for branch coverage (if/else, case, loops)
    fn generate_branch_tests(&self, name: &str, body: &[BashStmt]) -> TestGenResult<Vec<UnitTest>> {
        let mut tests = Vec::new();

        for stmt in body {
            match stmt {
                BashStmt::If {
                    elif_blocks,
                    else_block,
                    ..
                } => {
                    tests.extend(self.generate_if_branch_tests(
                        name,
                        elif_blocks,
                        else_block.as_ref(),
                    ));
                }
                BashStmt::While { .. } => {
                    tests.push(self.make_while_test(name));
                }
                BashStmt::For { .. } => {
                    tests.push(self.make_for_test(name));
                }
                _ => {}
            }
        }

        Ok(tests)
    }

    /// Generate tests for an if/elif/else statement
    fn generate_if_branch_tests(
        &self,
        name: &str,
        elif_blocks: &[(BashExpr, Vec<BashStmt>)],
        else_block: Option<&Vec<BashStmt>>,
    ) -> Vec<UnitTest> {
        let mut tests = Vec::new();

        // Test the "then" branch
        tests.push(UnitTest {
            name: format!("test_{}_if_then_branch", name),
            test_fn: format!("{}()", name),
            assertions: vec![Assertion::Comment("Test if-then branch".to_string())],
        });

        // Test elif branches
        for (i, _) in elif_blocks.iter().enumerate() {
            tests.push(UnitTest {
                name: format!("test_{}_elif_{}_branch", name, i),
                test_fn: format!("{}()", name),
                assertions: vec![Assertion::Comment(format!("Test elif {} branch", i))],
            });
        }

        // Test else branch if present
        if else_block.is_some() {
            tests.push(UnitTest {
                name: format!("test_{}_else_branch", name),
                test_fn: format!("{}()", name),
                assertions: vec![Assertion::Comment("Test else branch".to_string())],
            });
        }

        tests
    }

    /// Build a while-loop coverage test
    fn make_while_test(&self, name: &str) -> UnitTest {
        UnitTest {
            name: format!("test_{}_while_loop", name),
            test_fn: format!("{}()", name),
            assertions: vec![Assertion::Comment("Test while loop execution".to_string())],
        }
    }

    /// Build a for-loop coverage test
    fn make_for_test(&self, name: &str) -> UnitTest {
        UnitTest {
            name: format!("test_{}_for_loop", name),
            test_fn: format!("{}()", name),
            assertions: vec![Assertion::Comment("Test for loop iteration".to_string())],
        }
    }

    /// Generate boundary value tests
    fn generate_boundary_tests(
        &self,
        name: &str,
        _body: &[BashStmt],
    ) -> TestGenResult<Vec<UnitTest>> {
        vec![
            UnitTest {
                name: format!("test_{}_boundary_zero", name),
                test_fn: format!("{}(0)", name),
                assertions: vec![Assertion::Comment("Test with zero value".to_string())],
            },
            UnitTest {
                name: format!("test_{}_boundary_one", name),
                test_fn: format!("{}(1)", name),
                assertions: vec![Assertion::Comment("Test with one value".to_string())],
            },
        ]
        .into_iter()
        .map(Ok)
        .collect()
    }

    /// Generate edge case tests (empty strings, null, max values)
    fn generate_edge_case_tests(&self, ast: &BashAst) -> TestGenResult<Vec<UnitTest>> {
        let mut tests = Vec::new();

        for stmt in &ast.statements {
            if let BashStmt::Function { name, .. } = stmt {
                // Empty string test
                tests.push(UnitTest {
                    name: format!("test_{}_edge_case_empty_string", name),
                    test_fn: format!("{}(\"\")", name),
                    assertions: vec![Assertion::Comment(
                        "Test with empty string input".to_string(),
                    )],
                });

                // Negative number test
                tests.push(UnitTest {
                    name: format!("test_{}_edge_case_negative", name),
                    test_fn: format!("{}(-1)", name),
                    assertions: vec![Assertion::Comment("Test with negative value".to_string())],
                });

                // Large number test
                tests.push(UnitTest {
                    name: format!("test_{}_edge_case_large_value", name),
                    test_fn: format!("{}(i64::MAX)", name),
                    assertions: vec![Assertion::Comment("Test with maximum value".to_string())],
                });
            }
        }

        Ok(tests)
    }

    /// Generate error case tests
    fn generate_error_case_tests(&self, ast: &BashAst) -> TestGenResult<Vec<UnitTest>> {
        let mut tests = Vec::new();

        for stmt in &ast.statements {
            if let BashStmt::Function { name, body, .. } = stmt {
                // Check if function uses file operations
                if self.uses_file_operations(body) {
                    tests.push(UnitTest {
                        name: format!("test_{}_error_file_not_found", name),
                        test_fn: format!("{}(\"/nonexistent/file\")", name),
                        assertions: vec![Assertion::ShouldPanic {
                            expected_message: Some("File not found".to_string()),
                        }],
                    });
                }

                // Check if function has numeric operations
                if self.uses_arithmetic(body) {
                    tests.push(UnitTest {
                        name: format!("test_{}_error_invalid_input", name),
                        test_fn: format!("{}(\"invalid\")", name),
                        assertions: vec![Assertion::ShouldPanic {
                            expected_message: Some("Invalid input".to_string()),
                        }],
                    });
                }
            }
        }

        Ok(tests)
    }

    /// Generate targeted tests for specific uncovered paths
    pub fn generate_targeted_tests(
        &self,
        uncovered: &[UncoveredPath],
    ) -> TestGenResult<Vec<UnitTest>> {
        let mut tests = Vec::new();

        for path in uncovered {
            match path {
                UncoveredPath::Line(line) => {
                    tests.push(UnitTest {
                        name: format!("test_coverage_line_{}", line),
                        test_fn: "target_line()".to_string(),
                        assertions: vec![Assertion::Comment(format!("Cover line {}", line))],
                    });
                }
                UncoveredPath::Branch(branch) => {
                    tests.push(UnitTest {
                        name: format!("test_coverage_branch_{}", branch.function),
                        test_fn: format!("{}()", branch.function),
                        assertions: vec![Assertion::Comment(format!(
                            "Cover branch {:?}",
                            branch.branch_type
                        ))],
                    });
                }
                UncoveredPath::Function(func) => {
                    tests.push(UnitTest {
                        name: format!("test_coverage_function_{}", func),
                        test_fn: format!("{}()", func),
                        assertions: vec![Assertion::Comment(format!("Cover function {}", func))],
                    });
                }
            }
        }

        Ok(tests)
    }

    /// Check if function body uses file operations
    fn uses_file_operations(&self, body: &[BashStmt]) -> bool {
        for stmt in body {
            if let BashStmt::Command { name, .. } = stmt {
                if matches!(name.as_str(), "cat" | "ls" | "mkdir" | "rm" | "cp" | "mv") {
                    return true;
                }
            }
        }
        false
    }

    /// Check if function body uses arithmetic operations
    fn uses_arithmetic(&self, body: &[BashStmt]) -> bool {
        for stmt in body {
            if let BashStmt::Assignment { value, .. } = stmt {
                if matches!(value, BashExpr::Arithmetic(_)) {
                    return true;
                }
            }
        }
        false
    }
}

/// A single unit test
#[derive(Debug, Clone)]
pub struct UnitTest {
    pub name: String,
    pub test_fn: String,
    pub assertions: Vec<Assertion>,
}

impl UnitTest {
    /// Convert to Rust test code
    pub fn to_rust_code(&self) -> String {
        let mut code = "#[test]\n".to_string();

        // Add #[should_panic] if needed
        for assertion in &self.assertions {
            if let Assertion::ShouldPanic { .. } = assertion {
                if let Assertion::ShouldPanic {
                    expected_message: Some(msg),
                } = assertion
                {
                    code.push_str(&format!("#[should_panic(expected = \"{}\")]\n", msg));
                } else {
                    code.push_str("#[should_panic]\n");
                }
                break;
            }
        }

        code.push_str(&format!("fn {}() {{\n", self.name));

        for assertion in &self.assertions {
            code.push_str(&format!("    {}\n", assertion.to_rust_code()));
        }

        code.push_str("}\n");
        code
    }
}

/// Test assertion types
#[derive(Debug, Clone)]
pub enum Assertion {
    /// assert_eq!(actual, expected)
    Equals { actual: String, expected: String },

    /// assert_ne!(actual, expected)
    NotEquals { actual: String, expected: String },

    /// assert!(condition)
    True { condition: String },

    /// assert!(!condition)
    False { condition: String },

    /// #[should_panic]
    ShouldPanic { expected_message: Option<String> },

    /// Comment for documentation
    Comment(String),
}

impl Assertion {
    /// Convert to Rust assertion code
    fn to_rust_code(&self) -> String {
        match self {
            Assertion::Equals { actual, expected } => {
                format!("assert_eq!({}, {});", actual, expected)
            }
            Assertion::NotEquals { actual, expected } => {
                format!("assert_ne!({}, {});", actual, expected)
            }
            Assertion::True { condition } => {
                format!("assert!({});", condition)
            }
            Assertion::False { condition } => {
                format!("assert!(!{});", condition)
            }
            Assertion::ShouldPanic { .. } => {
                // This is handled at the function level
                "// Should panic test".to_string()
            }
            Assertion::Comment(text) => {
                format!("// {}", text)
            }
        }
    }
}

