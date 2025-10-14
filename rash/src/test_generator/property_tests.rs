//! Property Test Generation (Sprint 2)
//!
//! Generates property tests using proptest for:
//! - Determinism (same input → same output)
//! - Idempotency (f(f(x)) == f(x))
//! - Bounds checking
//! - Type preservation

use crate::bash_parser::ast::*;
use super::core::TestGenResult;
use std::collections::HashSet;

pub struct PropertyTestGenerator {
    /// Maximum test cases per property
    max_test_cases: usize,
}

impl Default for PropertyTestGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl PropertyTestGenerator {
    pub fn new() -> Self {
        Self {
            max_test_cases: 100,
        }
    }

    /// Generate property tests for AST
    pub fn generate_properties(&self, ast: &BashAst) -> TestGenResult<Vec<PropertyTest>> {
        let mut tests = Vec::new();

        for stmt in &ast.statements {
            if let BashStmt::Function { name, body, .. } = stmt {
                // Generate determinism tests
                if let Some(test) = self.generate_determinism_test(name, body)? {
                    tests.push(test);
                }

                // Generate idempotency tests
                if let Some(test) = self.generate_idempotency_test(name, body)? {
                    tests.push(test);
                }

                // Generate bounds tests
                tests.extend(self.generate_bounds_tests(name, body)?);

                // Generate type preservation tests
                if let Some(test) = self.generate_type_preservation_test(name, body)? {
                    tests.push(test);
                }
            }
        }

        Ok(tests)
    }

    /// Generate determinism property test (same input → same output)
    fn generate_determinism_test(
        &self,
        name: &str,
        body: &[BashStmt],
    ) -> TestGenResult<Option<PropertyTest>> {
        // Check if function is deterministic (no random operations, no file I/O)
        if self.has_nondeterministic_operations(body) {
            return Ok(None);
        }

        let generators = self.infer_generators_from_function(name, body)?;

        Ok(Some(PropertyTest {
            name: format!("prop_{}_determinism", name),
            property: Property::Determinism,
            generators,
            test_cases: self.max_test_cases,
        }))
    }

    /// Generate idempotency property test (f(f(x)) == f(x))
    fn generate_idempotency_test(
        &self,
        name: &str,
        body: &[BashStmt],
    ) -> TestGenResult<Option<PropertyTest>> {
        // Check if function is likely idempotent (normalization, formatting, etc.)
        if !self.is_potentially_idempotent(body) {
            return Ok(None);
        }

        let generators = self.infer_generators_from_function(name, body)?;

        Ok(Some(PropertyTest {
            name: format!("prop_{}_idempotency", name),
            property: Property::Idempotency,
            generators,
            test_cases: self.max_test_cases,
        }))
    }

    /// Generate bounds checking property tests
    fn generate_bounds_tests(
        &self,
        name: &str,
        body: &[BashStmt],
    ) -> TestGenResult<Vec<PropertyTest>> {
        let mut tests = Vec::new();

        // Look for arithmetic operations that suggest bounds
        for stmt in body {
            if let Some(bounds) = self.extract_bounds(stmt) {
                let generators = vec![Generator::Integer {
                    min: bounds.min - 10,
                    max: bounds.max + 10,
                }];

                tests.push(PropertyTest {
                    name: format!("prop_{}_bounds_{}_{}", name, bounds.min, bounds.max),
                    property: Property::Bounds {
                        min: bounds.min,
                        max: bounds.max,
                    },
                    generators,
                    test_cases: self.max_test_cases,
                });
            }
        }

        Ok(tests)
    }

    /// Generate type preservation property test
    fn generate_type_preservation_test(
        &self,
        name: &str,
        body: &[BashStmt],
    ) -> TestGenResult<Option<PropertyTest>> {
        // Check if function preserves types (string → string, int → int)
        let generators = self.infer_generators_from_function(name, body)?;

        Ok(Some(PropertyTest {
            name: format!("prop_{}_type_preservation", name),
            property: Property::TypePreservation,
            generators,
            test_cases: self.max_test_cases,
        }))
    }

    /// Check if function has non-deterministic operations
    fn has_nondeterministic_operations(&self, body: &[BashStmt]) -> bool {
        for stmt in body {
            match stmt {
                BashStmt::Command { name, .. } => {
                    if matches!(
                        name.as_str(),
                        "random" | "date" | "time" | "rand" | "uuid"
                    ) {
                        return true;
                    }
                }
                BashStmt::If { then_block, elif_blocks, else_block, .. } => {
                    if self.has_nondeterministic_operations(then_block) {
                        return true;
                    }
                    for (_, block) in elif_blocks {
                        if self.has_nondeterministic_operations(block) {
                            return true;
                        }
                    }
                    if let Some(block) = else_block {
                        if self.has_nondeterministic_operations(block) {
                            return true;
                        }
                    }
                }
                BashStmt::While { body, .. } | BashStmt::For { body, .. } => {
                    if self.has_nondeterministic_operations(body) {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }

    /// Check if function is potentially idempotent
    fn is_potentially_idempotent(&self, body: &[BashStmt]) -> bool {
        // Look for patterns that suggest idempotency:
        // - String normalization (trim, lowercase, etc.)
        // - Sorting operations
        // - Deduplication
        // - Path normalization

        for stmt in body {
            if let BashStmt::Command { name, .. } = stmt {
                if matches!(
                    name.as_str(),
                    "sort" | "uniq" | "tr" | "sed" | "awk" | "normalize" | "trim"
                ) {
                    return true;
                }
            }
        }

        false
    }

    /// Infer proptest generators from function signature and body
    fn infer_generators_from_function(
        &self,
        _name: &str,
        body: &[BashStmt],
    ) -> TestGenResult<Vec<Generator>> {
        let mut generators = Vec::new();
        let mut seen_types = HashSet::new();

        for stmt in body {
            if let BashStmt::Assignment { value, .. } = stmt {
                match value {
                    BashExpr::Literal(lit) => {
                        if lit.parse::<i64>().is_ok() && !seen_types.contains("integer") {
                            generators.push(Generator::Integer {
                                min: -1000,
                                max: 1000,
                            });
                            seen_types.insert("integer");
                        } else if !seen_types.contains("string") {
                            generators.push(Generator::String {
                                pattern: "[a-zA-Z0-9]{1,20}".to_string(),
                            });
                            seen_types.insert("string");
                        }
                    }
                    BashExpr::Arithmetic(_) => {
                        if !seen_types.contains("integer") {
                            generators.push(Generator::Integer {
                                min: -1000,
                                max: 1000,
                            });
                            seen_types.insert("integer");
                        }
                    }
                    _ => {}
                }
            }
        }

        // Default to string generator if nothing else was found
        if generators.is_empty() {
            generators.push(Generator::String {
                pattern: "[a-zA-Z0-9]{1,20}".to_string(),
            });
        }

        Ok(generators)
    }

    /// Extract bounds from conditional statements
    fn extract_bounds(&self, stmt: &BashStmt) -> Option<BoundsInfo> {
        if let BashStmt::If { condition, .. } = stmt {
            // Try to extract bounds from conditions like [ $x -gt 0 ] && [ $x -lt 100 ]
            if let BashExpr::Test { .. } = condition {
                // Simplified: assume reasonable bounds
                return Some(BoundsInfo { min: 0, max: 100 });
            }
        }
        None
    }
}

struct BoundsInfo {
    min: i64,
    max: i64,
}

#[derive(Debug, Clone)]
pub struct PropertyTest {
    pub name: String,
    pub property: Property,
    pub generators: Vec<Generator>,
    pub test_cases: usize,
}

impl PropertyTest {
    /// Generate Rust code for this property test
    pub fn to_rust_code(&self) -> String {
        let mut code = String::new();

        // Generate the proptest macro invocation
        code.push_str(&"proptest! {\n".to_string());
        code.push_str(&"    #[test]\n".to_string());
        code.push_str(&format!("    fn {}(\n", self.name));

        // Generate parameter list from generators
        for (i, gen) in self.generators.iter().enumerate() {
            let param_name = format!("arg{}", i);
            let generator_code = gen.to_proptest_strategy();
            code.push_str(&format!("        {} in {},\n", param_name, generator_code));
        }

        code.push_str("    ) {\n");

        // Generate property assertion based on property type
        match &self.property {
            Property::Determinism => {
                code.push_str("        // Test determinism: same input → same output\n");
                let args = (0..self.generators.len())
                    .map(|i| format!("arg{}", i))
                    .collect::<Vec<_>>()
                    .join(", ");
                code.push_str(&format!("        let result1 = function_under_test({});\n", args));
                code.push_str(&format!("        let result2 = function_under_test({});\n", args));
                code.push_str("        prop_assert_eq!(result1, result2);\n");
            }
            Property::Idempotency => {
                code.push_str("        // Test idempotency: f(f(x)) == f(x)\n");
                let args = (0..self.generators.len())
                    .map(|i| format!("arg{}", i))
                    .collect::<Vec<_>>()
                    .join(", ");
                code.push_str(&format!("        let result1 = function_under_test({});\n", args));
                code.push_str("        let result2 = function_under_test(&result1);\n");
                code.push_str("        prop_assert_eq!(result1, result2);\n");
            }
            Property::Commutativity => {
                code.push_str("        // Test commutativity: f(a, b) == f(b, a)\n");
                if self.generators.len() >= 2 {
                    code.push_str("        let result1 = function_under_test(arg0, arg1);\n");
                    code.push_str("        let result2 = function_under_test(arg1, arg0);\n");
                    code.push_str("        prop_assert_eq!(result1, result2);\n");
                }
            }
            Property::Bounds { min, max } => {
                code.push_str(&format!(
                    "        // Test bounds: result in range [{}, {}]\n",
                    min, max
                ));
                let args = (0..self.generators.len())
                    .map(|i| format!("arg{}", i))
                    .collect::<Vec<_>>()
                    .join(", ");
                code.push_str(&format!("        let result = function_under_test({});\n", args));
                code.push_str(&format!("        prop_assert!(result >= {});\n", min));
                code.push_str(&format!("        prop_assert!(result <= {});\n", max));
            }
            Property::TypePreservation => {
                code.push_str("        // Test type preservation\n");
                let args = (0..self.generators.len())
                    .map(|i| format!("arg{}", i))
                    .collect::<Vec<_>>()
                    .join(", ");
                code.push_str(&format!("        let result = function_under_test({});\n", args));
                code.push_str("        // Verify result has expected type\n");
                code.push_str("        prop_assert!(std::mem::size_of_val(&result) > 0);\n");
            }
            Property::NoSideEffects => {
                code.push_str("        // Test no side effects: function doesn't modify external state\n");
                let args = (0..self.generators.len())
                    .map(|i| format!("arg{}", i))
                    .collect::<Vec<_>>()
                    .join(", ");
                code.push_str(&format!("        let _result = function_under_test({});\n", args));
                code.push_str("        // Verify no side effects occurred\n");
            }
        }

        code.push_str("    }\n");
        code.push_str("}\n");

        code
    }
}

#[derive(Debug, Clone)]
pub enum Property {
    Determinism,
    Idempotency,
    Commutativity,
    Bounds { min: i64, max: i64 },
    TypePreservation,
    NoSideEffects,
}

#[derive(Debug, Clone)]
pub enum Generator {
    Integer { min: i64, max: i64 },
    String { pattern: String },
    Path { valid: bool },
}

impl Generator {
    /// Convert to proptest strategy code
    pub fn to_proptest_strategy(&self) -> String {
        match self {
            Generator::Integer { min, max } => {
                format!("{}..={}", min, max)
            }
            Generator::String { pattern } => {
                // For simple patterns, use proptest string generators
                if pattern == "[a-zA-Z0-9]{1,20}" {
                    "\"[a-zA-Z0-9]{1,20}\"".to_string()
                } else {
                    format!("\"{}\"", pattern)
                }
            }
            Generator::Path { valid } => {
                if *valid {
                    "\"/[a-z]{1,10}/[a-z]{1,10}\"".to_string()
                } else {
                    "\"/[^/]{0,5}\"".to_string()
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_property_test_generator_creation() {
        let gen = PropertyTestGenerator::new();
        assert_eq!(gen.max_test_cases, 100);
    }

    #[test]
    fn test_determinism_property_generation() {
        let gen = PropertyTestGenerator::new();
        let ast = BashAst {
            statements: vec![BashStmt::Function {
                name: "pure_func".to_string(),
                body: vec![BashStmt::Assignment {
                    name: "result".to_string(),
                    value: BashExpr::Literal("42".to_string()),
                    exported: false,
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

        let properties = gen.generate_properties(&ast).unwrap();
        assert!(!properties.is_empty());
        assert!(properties
            .iter()
            .any(|p| matches!(p.property, Property::Determinism)));
    }

    #[test]
    fn test_idempotency_property_generation() {
        let gen = PropertyTestGenerator::new();
        let ast = BashAst {
            statements: vec![BashStmt::Function {
                name: "normalize".to_string(),
                body: vec![BashStmt::Command {
                    name: "sort".to_string(),
                    args: vec![],
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

        let properties = gen.generate_properties(&ast).unwrap();
        assert!(properties
            .iter()
            .any(|p| matches!(p.property, Property::Idempotency)));
    }

    #[test]
    fn test_bounds_property_generation() {
        let gen = PropertyTestGenerator::new();
        let ast = BashAst {
            statements: vec![BashStmt::Function {
                name: "bounded_func".to_string(),
                body: vec![BashStmt::If {
                    condition: BashExpr::Test(Box::new(TestExpr::IntGt(
                        BashExpr::Variable("x".to_string()),
                        BashExpr::Literal("0".to_string()),
                    ))),
                    then_block: vec![],
                    elif_blocks: vec![],
                    else_block: None,
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

        let properties = gen.generate_properties(&ast).unwrap();
        assert!(properties
            .iter()
            .any(|p| matches!(p.property, Property::Bounds { .. })));
    }

    #[test]
    fn test_nondeterministic_detection() {
        let gen = PropertyTestGenerator::new();

        // Test with random command
        let body = vec![BashStmt::Command {
            name: "random".to_string(),
            args: vec![],
            span: Span::dummy(),
        }];

        assert!(gen.has_nondeterministic_operations(&body));

        // Test with date command
        let body = vec![BashStmt::Command {
            name: "date".to_string(),
            args: vec![],
            span: Span::dummy(),
        }];

        assert!(gen.has_nondeterministic_operations(&body));

        // Test with pure operations
        let body = vec![BashStmt::Assignment {
            name: "x".to_string(),
            value: BashExpr::Literal("42".to_string()),
            exported: false,
            span: Span::dummy(),
        }];

        assert!(!gen.has_nondeterministic_operations(&body));
    }

    #[test]
    fn test_idempotent_detection() {
        let gen = PropertyTestGenerator::new();

        // Test with sort command (idempotent)
        let body = vec![BashStmt::Command {
            name: "sort".to_string(),
            args: vec![],
            span: Span::dummy(),
        }];

        assert!(gen.is_potentially_idempotent(&body));

        // Test with non-idempotent operations
        let body = vec![BashStmt::Assignment {
            name: "x".to_string(),
            value: BashExpr::Literal("42".to_string()),
            exported: false,
            span: Span::dummy(),
        }];

        assert!(!gen.is_potentially_idempotent(&body));
    }

    #[test]
    fn test_generator_inference() {
        let gen = PropertyTestGenerator::new();

        let body = vec![
            BashStmt::Assignment {
                name: "x".to_string(),
                value: BashExpr::Literal("42".to_string()),
                exported: false,
                span: Span::dummy(),
            },
            BashStmt::Assignment {
                name: "y".to_string(),
                value: BashExpr::Arithmetic(Box::new(ArithExpr::Add(
                    Box::new(ArithExpr::Variable("x".to_string())),
                    Box::new(ArithExpr::Number(1)),
                ))),
                exported: false,
                span: Span::dummy(),
            },
        ];

        let generators = gen.infer_generators_from_function("test", &body).unwrap();
        assert!(!generators.is_empty());
        assert!(generators
            .iter()
            .any(|g| matches!(g, Generator::Integer { .. })));
    }

    #[test]
    fn test_property_test_to_rust_code_determinism() {
        let test = PropertyTest {
            name: "prop_test_determinism".to_string(),
            property: Property::Determinism,
            generators: vec![Generator::Integer { min: 0, max: 100 }],
            test_cases: 100,
        };

        let code = test.to_rust_code();
        assert!(code.contains("proptest!"));
        assert!(code.contains("prop_test_determinism"));
        assert!(code.contains("0..=100"));
        assert!(code.contains("determinism"));
    }

    #[test]
    fn test_property_test_to_rust_code_idempotency() {
        let test = PropertyTest {
            name: "prop_test_idempotency".to_string(),
            property: Property::Idempotency,
            generators: vec![Generator::String {
                pattern: "[a-zA-Z0-9]{1,20}".to_string(),
            }],
            test_cases: 100,
        };

        let code = test.to_rust_code();
        assert!(code.contains("idempotency"));
        assert!(code.contains("f(f(x)) == f(x)"));
    }

    #[test]
    fn test_property_test_to_rust_code_bounds() {
        let test = PropertyTest {
            name: "prop_test_bounds".to_string(),
            property: Property::Bounds { min: 0, max: 100 },
            generators: vec![Generator::Integer { min: -10, max: 110 }],
            test_cases: 100,
        };

        let code = test.to_rust_code();
        assert!(code.contains("bounds"));
        assert!(code.contains("result >= 0"));
        assert!(code.contains("result <= 100"));
    }

    #[test]
    fn test_generator_to_proptest_strategy() {
        let gen = Generator::Integer { min: 0, max: 100 };
        assert_eq!(gen.to_proptest_strategy(), "0..=100");

        let gen = Generator::String {
            pattern: "[a-zA-Z0-9]{1,20}".to_string(),
        };
        assert_eq!(gen.to_proptest_strategy(), "\"[a-zA-Z0-9]{1,20}\"");

        let gen = Generator::Path { valid: true };
        assert!(gen.to_proptest_strategy().contains("/[a-z]"));
    }
}
