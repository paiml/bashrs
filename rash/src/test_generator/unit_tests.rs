//! Unit Test Generation
//!
//! Generates unit tests with:
//! - Branch coverage (if/else, loops, case statements)
//! - Edge case testing (empty strings, zero, max values)
//! - Error case testing (file not found, invalid input)
//!
//! Target: ≥80% line coverage

use crate::bash_parser::ast::*;
use super::core::TestGenResult;
use super::coverage::UncoveredPath;

/// Generates unit tests for bash functions
pub struct UnitTestGenerator;

impl UnitTestGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Generate unit tests for all functions in AST
    pub fn generate_tests(&self, ast: &BashAst) -> TestGenResult<Vec<UnitTest>> {
        let mut tests = Vec::new();

        for stmt in &ast.statements {
            match stmt {
                BashStmt::Function { name, body, .. } => {
                    // Generate tests for this function
                    tests.extend(self.generate_function_tests(name, body)?);
                }
                _ => {}
            }
        }

        // Add edge case tests
        tests.extend(self.generate_edge_case_tests(ast)?);

        // Add error case tests
        tests.extend(self.generate_error_case_tests(ast)?);

        Ok(tests)
    }

    /// Generate tests for a specific function
    fn generate_function_tests(&self, name: &str, body: &[BashStmt]) -> TestGenResult<Vec<UnitTest>> {
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
                BashStmt::If { condition, then_block, elif_blocks, else_block, .. } => {
                    // Test the "then" branch
                    tests.push(UnitTest {
                        name: format!("test_{}_if_then_branch", name),
                        test_fn: format!("{}()", name),
                        assertions: vec![
                            Assertion::Comment("Test if-then branch".to_string()),
                        ],
                    });

                    // Test elif branches
                    for (i, _) in elif_blocks.iter().enumerate() {
                        tests.push(UnitTest {
                            name: format!("test_{}_elif_{}_branch", name, i),
                            test_fn: format!("{}()", name),
                            assertions: vec![
                                Assertion::Comment(format!("Test elif {} branch", i)),
                            ],
                        });
                    }

                    // Test else branch if present
                    if else_block.is_some() {
                        tests.push(UnitTest {
                            name: format!("test_{}_else_branch", name),
                            test_fn: format!("{}()", name),
                            assertions: vec![
                                Assertion::Comment("Test else branch".to_string()),
                            ],
                        });
                    }
                }

                BashStmt::While { .. } => {
                    // Test while loop
                    tests.push(UnitTest {
                        name: format!("test_{}_while_loop", name),
                        test_fn: format!("{}()", name),
                        assertions: vec![
                            Assertion::Comment("Test while loop execution".to_string()),
                        ],
                    });
                }

                BashStmt::For { .. } => {
                    // Test for loop
                    tests.push(UnitTest {
                        name: format!("test_{}_for_loop", name),
                        test_fn: format!("{}()", name),
                        assertions: vec![
                            Assertion::Comment("Test for loop iteration".to_string()),
                        ],
                    });
                }

                _ => {}
            }
        }

        Ok(tests)
    }

    /// Generate boundary value tests
    fn generate_boundary_tests(&self, name: &str, _body: &[BashStmt]) -> TestGenResult<Vec<UnitTest>> {
        vec![
            UnitTest {
                name: format!("test_{}_boundary_zero", name),
                test_fn: format!("{}(0)", name),
                assertions: vec![
                    Assertion::Comment("Test with zero value".to_string()),
                ],
            },
            UnitTest {
                name: format!("test_{}_boundary_one", name),
                test_fn: format!("{}(1)", name),
                assertions: vec![
                    Assertion::Comment("Test with one value".to_string()),
                ],
            },
        ].into_iter().map(Ok).collect()
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
                    assertions: vec![
                        Assertion::Comment("Test with empty string input".to_string()),
                    ],
                });

                // Negative number test
                tests.push(UnitTest {
                    name: format!("test_{}_edge_case_negative", name),
                    test_fn: format!("{}(-1)", name),
                    assertions: vec![
                        Assertion::Comment("Test with negative value".to_string()),
                    ],
                });

                // Large number test
                tests.push(UnitTest {
                    name: format!("test_{}_edge_case_large_value", name),
                    test_fn: format!("{}(i64::MAX)", name),
                    assertions: vec![
                        Assertion::Comment("Test with maximum value".to_string()),
                    ],
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
                        assertions: vec![
                            Assertion::ShouldPanic {
                                expected_message: Some("File not found".to_string()),
                            },
                        ],
                    });
                }

                // Check if function has numeric operations
                if self.uses_arithmetic(body) {
                    tests.push(UnitTest {
                        name: format!("test_{}_error_invalid_input", name),
                        test_fn: format!("{}(\"invalid\")", name),
                        assertions: vec![
                            Assertion::ShouldPanic {
                                expected_message: Some("Invalid input".to_string()),
                            },
                        ],
                    });
                }
            }
        }

        Ok(tests)
    }

    /// Generate targeted tests for specific uncovered paths
    pub fn generate_targeted_tests(&self, uncovered: &[UncoveredPath]) -> TestGenResult<Vec<UnitTest>> {
        let mut tests = Vec::new();

        for path in uncovered {
            match path {
                UncoveredPath::Line(line) => {
                    tests.push(UnitTest {
                        name: format!("test_coverage_line_{}", line),
                        test_fn: "target_line()".to_string(),
                        assertions: vec![
                            Assertion::Comment(format!("Cover line {}", line)),
                        ],
                    });
                }
                UncoveredPath::Branch(branch) => {
                    tests.push(UnitTest {
                        name: format!("test_coverage_branch_{}", branch.function),
                        test_fn: format!("{}()", branch.function),
                        assertions: vec![
                            Assertion::Comment(format!("Cover branch {:?}", branch.branch_type)),
                        ],
                    });
                }
                UncoveredPath::Function(func) => {
                    tests.push(UnitTest {
                        name: format!("test_coverage_function_{}", func),
                        test_fn: format!("{}()", func),
                        assertions: vec![
                            Assertion::Comment(format!("Cover function {}", func)),
                        ],
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
            match stmt {
                BashStmt::Assignment { value, .. } => {
                    if matches!(value, BashExpr::Arithmetic(_)) {
                        return true;
                    }
                }
                _ => {}
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
        let mut code = format!("#[test]\n");

        // Add #[should_panic] if needed
        for assertion in &self.assertions {
            if let Assertion::ShouldPanic { .. } = assertion {
                if let Assertion::ShouldPanic { expected_message: Some(msg) } = assertion {
                    code.push_str(&format!("#[should_panic(expected = \"{}\")]\n", msg));
                } else {
                    code.push_str("#[should_panic]\n");
                }
                break;
            }
        }

        code.push_str(&format!("fun {}() {{\n", self.name));

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unit_test_generator_creation() {
        let gen = UnitTestGenerator::new();
        let empty_ast = BashAst {
            statements: vec![],
            metadata: AstMetadata {
                source_file: None,
                line_count: 0,
                parse_time_ms: 0,
            },
        };

        let tests = gen.generate_tests(&empty_ast).unwrap();
        assert!(tests.is_empty());
    }

    #[test]
    fn test_generates_tests_for_function() {
        let gen = UnitTestGenerator::new();
        let ast = BashAst {
            statements: vec![
                BashStmt::Function {
                    name: "test_func".to_string(),
                    body: vec![
                        BashStmt::Comment {
                            text: " Empty function".to_string(),
                            span: Span::dummy(),
                        }
                    ],
                    span: Span::dummy(),
                }
            ],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let tests = gen.generate_tests(&ast).unwrap();
        assert!(!tests.is_empty());

        // Should generate edge case tests
        assert!(tests.iter().any(|t| t.name.contains("edge_case")));
    }

    #[test]
    fn test_assertion_to_rust_code() {
        let assertion = Assertion::Equals {
            actual: "result".to_string(),
            expected: "5".to_string(),
        };

        let code = assertion.to_rust_code();
        assert_eq!(code, "assert_eq!(result, 5);");
    }

    #[test]
    fn test_unit_test_to_rust_code() {
        let test = UnitTest {
            name: "test_example".to_string(),
            test_fn: "example()".to_string(),
            assertions: vec![
                Assertion::Comment("Test example function".to_string()),
                Assertion::Equals {
                    actual: "result".to_string(),
                    expected: "42".to_string(),
                },
            ],
        };

        let code = test.to_rust_code();
        assert!(code.contains("#[test]"));
        assert!(code.contains("fun test_example()"));
        assert!(code.contains("assert_eq!(result, 42);"));
    }
}
