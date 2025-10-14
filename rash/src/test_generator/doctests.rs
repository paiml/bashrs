//! Doctest Generation (Sprint 3)
//!
//! Extracts doctests from bash comments and usage examples

use crate::bash_parser::ast::*;
use super::core::TestGenResult;

pub struct DoctestGenerator;

impl Default for DoctestGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl DoctestGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Generate doctests from bash comments
    pub fn generate_doctests(&self, ast: &BashAst) -> TestGenResult<Vec<Doctest>> {
        let mut doctests = Vec::new();

        for stmt in &ast.statements {
            if let BashStmt::Function { name, body, .. } = stmt {
                // Extract doctests from function comments
                doctests.extend(self.extract_from_function(name, body)?);
            }
        }

        Ok(doctests)
    }

    /// Extract doctests from a function's comments
    fn extract_from_function(
        &self,
        function_name: &str,
        body: &[BashStmt],
    ) -> TestGenResult<Vec<Doctest>> {
        let mut doctests = Vec::new();
        let mut current_example: Option<String> = None;
        let mut current_output: Option<String> = None;

        for stmt in body {
            if let BashStmt::Comment { text, .. } = stmt {
                let text_lower = text.to_lowercase();

                // Check for "Example: expr => output" pattern
                if text_lower.contains("example:") {
                    if let Some(after_example) = text.split_once("example:").or_else(|| text.split_once("Example:")) {
                        let content = after_example.1.trim();

                        // Check if it has " => " separator
                        if let Some((example, output)) = content.split_once("=>") {
                            doctests.push(Doctest {
                                function_name: function_name.to_string(),
                                example: example.trim().to_string(),
                                expected_output: output.trim().to_string(),
                                description: None,
                            });
                        } else {
                            current_example = Some(content.to_string());
                        }
                    }
                }

                // Check for "Usage: ..." pattern (not else if - can have both)
                if text_lower.contains("usage:") {
                    if let Some(after_usage) = text.split_once("usage:").or_else(|| text.split_once("Usage:")) {
                        current_example = Some(after_usage.1.trim().to_string());
                    }
                }

                // Check for "Output: ..." pattern (not else if - can have both)
                if text_lower.contains("output:") {
                    if let Some(after_output) = text.split_once("output:").or_else(|| text.split_once("Output:")) {
                        current_output = Some(after_output.1.trim().to_string());
                    }
                }
            }

            // Check after each statement if we have both example and output
            if let (Some(ex), Some(out)) = (&current_example, &current_output) {
                doctests.push(Doctest {
                    function_name: function_name.to_string(),
                    example: ex.clone(),
                    expected_output: out.clone(),
                    description: None,
                });
                current_example = None;
                current_output = None;
            }
        }

        // Generate default examples if no examples found
        if doctests.is_empty() {
            doctests.extend(self.generate_default_examples(function_name, body)?);
        }

        Ok(doctests)
    }

    /// Generate default examples based on function structure
    fn generate_default_examples(
        &self,
        function_name: &str,
        body: &[BashStmt],
    ) -> TestGenResult<Vec<Doctest>> {
        let mut examples = Vec::new();

        // Analyze function to create meaningful examples
        let has_return = body.iter().any(|stmt| matches!(stmt, BashStmt::Return { .. }));

        // Create a basic example
        examples.push(Doctest {
            function_name: function_name.to_string(),
            example: format!("{}()", function_name),
            expected_output: if has_return {
                "// Returns result".to_string()
            } else {
                "// Executes successfully".to_string()
            },
            description: Some(format!("Basic usage of {}", function_name)),
        });

        Ok(examples)
    }

    /// Extract examples from inline comments
    pub fn extract_inline_examples(&self, ast: &BashAst) -> TestGenResult<Vec<Doctest>> {
        let mut doctests = Vec::new();

        // Look for standalone comment blocks before functions
        let mut pending_examples: Vec<(String, String)> = Vec::new();

        for stmt in ast.statements.iter() {
            match stmt {
                BashStmt::Comment { text, .. } => {
                    let text_lower = text.to_lowercase();

                    // Check if this comment has an example with => separator
                    if text_lower.contains("example:") {
                        if let Some(after_example) = text.split_once("example:").or_else(|| text.split_once("Example:")) {
                            let content = after_example.1.trim();

                            if let Some((example, output)) = content.split_once("=>") {
                                pending_examples.push((
                                    example.trim().to_string(),
                                    output.trim().to_string(),
                                ));
                            }
                        }
                    }
                }
                BashStmt::Function { name, .. } => {
                    // Associate pending examples with this function
                    for (example, output) in pending_examples.drain(..) {
                        doctests.push(Doctest {
                            function_name: name.clone(),
                            example,
                            expected_output: output,
                            description: None,
                        });
                    }
                }
                _ => {}
            }
        }

        Ok(doctests)
    }
}

#[derive(Debug, Clone)]
pub struct Doctest {
    pub function_name: String,
    pub example: String,
    pub expected_output: String,
    pub description: Option<String>,
}

impl Doctest {
    /// Convert to Rust doctest code
    pub fn to_rust_code(&self) -> String {
        let mut code = String::new();

        // Add description if present
        if let Some(desc) = &self.description {
            code.push_str(&format!("/// {}\n///\n", desc));
        }

        code.push_str("/// # Examples\n");
        code.push_str("///\n");
        code.push_str("/// ```\n");

        // Add the example
        code.push_str(&format!("/// use crate::{};\n", self.function_name));
        code.push_str(&format!("/// {}\n", self.example));

        // Add assertion for expected output if meaningful
        if !self.expected_output.starts_with("//") {
            code.push_str(&format!("/// assert_eq!(result, {});\n", self.expected_output));
        } else {
            code.push_str(&format!("/// {}\n", self.expected_output));
        }

        code.push_str("/// ```\n");

        code
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doctest_generator_creation() {
        let gen = DoctestGenerator::new();
        let empty_ast = BashAst {
            statements: vec![],
            metadata: AstMetadata {
                source_file: None,
                line_count: 0,
                parse_time_ms: 0,
            },
        };

        let doctests = gen.generate_doctests(&empty_ast).unwrap();
        assert!(doctests.is_empty());
    }

    #[test]
    fn test_extract_example_with_arrow() {
        let gen = DoctestGenerator::new();
        let ast = BashAst {
            statements: vec![BashStmt::Function {
                name: "factorial".to_string(),
                body: vec![BashStmt::Comment {
                    text: " Example: factorial(5) => 120".to_string(),
                    span: Span::dummy(),
                }],
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let doctests = gen.generate_doctests(&ast).unwrap();
        assert_eq!(doctests.len(), 1);
        assert_eq!(doctests[0].function_name, "factorial");
        assert_eq!(doctests[0].example, "factorial(5)");
        assert_eq!(doctests[0].expected_output, "120");
    }

    #[test]
    fn test_extract_usage_and_output() {
        let gen = DoctestGenerator::new();
        let ast = BashAst {
            statements: vec![BashStmt::Function {
                name: "greet".to_string(),
                body: vec![
                    BashStmt::Comment {
                        text: " Usage: greet(\"Alice\")".to_string(),
                        span: Span::dummy(),
                    },
                    BashStmt::Comment {
                        text: " Output: Hello, Alice!".to_string(),
                        span: Span::dummy(),
                    },
                ],
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let doctests = gen.generate_doctests(&ast).unwrap();
        assert_eq!(doctests.len(), 1);
        assert_eq!(doctests[0].function_name, "greet");
        assert_eq!(doctests[0].example, "greet(\"Alice\")");
        assert_eq!(doctests[0].expected_output, "Hello, Alice!");
    }

    #[test]
    fn test_generate_default_example() {
        let gen = DoctestGenerator::new();
        let ast = BashAst {
            statements: vec![BashStmt::Function {
                name: "test_func".to_string(),
                body: vec![BashStmt::Return {
                    code: Some(BashExpr::Literal("0".to_string())),
                    span: Span::dummy(),
                }],
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let doctests = gen.generate_doctests(&ast).unwrap();
        assert_eq!(doctests.len(), 1);
        assert_eq!(doctests[0].function_name, "test_func");
        assert!(doctests[0].example.contains("test_func"));
        assert!(doctests[0].expected_output.contains("Returns result"));
    }

    #[test]
    fn test_doctest_to_rust_code() {
        let doctest = Doctest {
            function_name: "factorial".to_string(),
            example: "let result = factorial(5);".to_string(),
            expected_output: "120".to_string(),
            description: Some("Calculate factorial".to_string()),
        };

        let code = doctest.to_rust_code();
        assert!(code.contains("/// # Examples"));
        assert!(code.contains("/// ```"));
        assert!(code.contains("use crate::factorial"));
        assert!(code.contains("let result = factorial(5);"));
        assert!(code.contains("assert_eq!(result, 120);"));
        assert!(code.contains("Calculate factorial"));
    }

    #[test]
    fn test_doctest_to_rust_code_comment_output() {
        let doctest = Doctest {
            function_name: "test_func".to_string(),
            example: "test_func()".to_string(),
            expected_output: "// Executes successfully".to_string(),
            description: None,
        };

        let code = doctest.to_rust_code();
        assert!(code.contains("/// # Examples"));
        assert!(code.contains("test_func()"));
        assert!(code.contains("// Executes successfully"));
        assert!(!code.contains("assert_eq!"));
    }

    #[test]
    fn test_extract_multiple_examples() {
        let gen = DoctestGenerator::new();
        let ast = BashAst {
            statements: vec![BashStmt::Function {
                name: "math".to_string(),
                body: vec![
                    BashStmt::Comment {
                        text: " Example: math(1, 2) => 3".to_string(),
                        span: Span::dummy(),
                    },
                    BashStmt::Comment {
                        text: " Example: math(10, 5) => 15".to_string(),
                        span: Span::dummy(),
                    },
                ],
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let doctests = gen.generate_doctests(&ast).unwrap();
        assert_eq!(doctests.len(), 2);
        assert_eq!(doctests[0].example, "math(1, 2)");
        assert_eq!(doctests[0].expected_output, "3");
        assert_eq!(doctests[1].example, "math(10, 5)");
        assert_eq!(doctests[1].expected_output, "15");
    }

    #[test]
    fn test_extract_inline_examples() {
        let gen = DoctestGenerator::new();
        let ast = BashAst {
            statements: vec![
                BashStmt::Comment {
                    text: " Example: process(data) => result".to_string(),
                    span: Span::dummy(),
                },
                BashStmt::Function {
                    name: "process".to_string(),
                    body: vec![],
                    span: Span::dummy(),
                },
            ],
            metadata: AstMetadata {
                source_file: None,
                line_count: 2,
                parse_time_ms: 0,
            },
        };

        let doctests = gen.extract_inline_examples(&ast).unwrap();
        assert_eq!(doctests.len(), 1);
        assert_eq!(doctests[0].function_name, "process");
        assert_eq!(doctests[0].example, "process(data)");
        assert_eq!(doctests[0].expected_output, "result");
    }
}
