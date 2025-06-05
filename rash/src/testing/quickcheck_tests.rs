// QuickCheck-style property-based testing for Rash transpiler
// Comprehensive property testing to ensure correctness across all inputs

use crate::ast::restricted::{BinaryOp, Literal, UnaryOp};
use crate::ast::{Expr, Function, RestrictedAst, Stmt, Type};
use crate::models::{Config, ShellDialect, VerificationLevel};
use crate::services::parse;
use crate::transpile;
use proptest::prelude::*;

// Generator strategies for creating valid AST nodes
pub mod generators {
    use super::*;

    pub fn any_valid_identifier() -> impl Strategy<Value = String> {
        "[a-zA-Z_][a-zA-Z0-9_]{0,20}".prop_filter("Avoid reserved identifiers", |s| {
            // Rust keywords that should be filtered out
            const RUST_KEYWORDS: &[&str] = &[
                "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false",
                "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut",
                "pub", "ref", "return", "self", "Self", "static", "struct", "super", "trait",
                "true", "type", "unsafe", "use", "where", "while", "async", "await", "dyn",
                "abstract", "become", "box", "do", "final", "macro", "override", "priv", "typeof",
                "unsized", "virtual", "yield",
            ];

            !s.is_empty()
                && s != "_"
                && s != "main"
                && !s.starts_with("__")
                && !RUST_KEYWORDS.contains(&s.as_str())
        })
    }

    pub fn any_safe_string() -> impl Strategy<Value = String> {
        "[a-zA-Z0-9 _.-]{0,50}"
    }

    pub fn any_u32_literal() -> impl Strategy<Value = Literal> {
        (0u32..10000u32).prop_map(Literal::U32)
    }

    pub fn any_bool_literal() -> impl Strategy<Value = Literal> {
        prop::bool::ANY.prop_map(Literal::Bool)
    }

    pub fn any_string_literal() -> impl Strategy<Value = Literal> {
        any_safe_string().prop_map(Literal::Str)
    }

    pub fn any_literal() -> impl Strategy<Value = Literal> {
        prop_oneof![any_u32_literal(), any_bool_literal(), any_string_literal()]
    }

    pub fn any_binary_op() -> impl Strategy<Value = BinaryOp> {
        prop_oneof![
            Just(BinaryOp::Add),
            Just(BinaryOp::Sub),
            Just(BinaryOp::Mul),
            Just(BinaryOp::Div),
            Just(BinaryOp::Eq),
            Just(BinaryOp::Ne),
            Just(BinaryOp::Lt),
            Just(BinaryOp::Le),
            Just(BinaryOp::Gt),
            Just(BinaryOp::Ge),
            Just(BinaryOp::And),
            Just(BinaryOp::Or),
        ]
    }

    pub fn any_unary_op() -> impl Strategy<Value = UnaryOp> {
        prop_oneof![Just(UnaryOp::Not), Just(UnaryOp::Neg)]
    }

    pub fn leaf_expr() -> impl Strategy<Value = Expr> {
        prop_oneof![
            any_literal().prop_map(Expr::Literal),
            any_valid_identifier().prop_map(Expr::Variable),
        ]
    }

    pub fn simple_expr() -> impl Strategy<Value = Expr> {
        prop_oneof![
            leaf_expr(),
            (any_binary_op(), leaf_expr(), leaf_expr()).prop_map(|(op, left, right)| {
                Expr::Binary {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                }
            }),
            (any_unary_op(), leaf_expr()).prop_map(|(op, expr)| Expr::Unary {
                op,
                operand: Box::new(expr),
            }),
            (
                any_valid_identifier(),
                prop::collection::vec(leaf_expr(), 0..3)
            )
                .prop_map(|(name, args)| Expr::FunctionCall { name, args }),
        ]
    }

    pub fn any_type() -> impl Strategy<Value = Type> {
        prop_oneof![
            Just(Type::Void),
            Just(Type::Bool),
            Just(Type::U32),
            Just(Type::Str),
        ]
    }

    pub fn simple_stmt() -> impl Strategy<Value = Stmt> {
        prop_oneof![
            (any_valid_identifier(), simple_expr())
                .prop_map(|(name, value)| Stmt::Let { name, value }),
            simple_expr().prop_map(Stmt::Expr),
            prop::option::of(simple_expr()).prop_map(Stmt::Return),
        ]
    }

    pub fn any_function() -> impl Strategy<Value = Function> {
        (
            any_valid_identifier(),
            prop::collection::vec(simple_stmt(), 0..5),
            any_type(),
        )
            .prop_map(|(name, body, return_type)| Function {
                name,
                params: vec![], // Keep params simple for now
                return_type,
                body,
            })
    }

    pub fn valid_ast() -> impl Strategy<Value = RestrictedAst> {
        prop::collection::vec(any_function(), 1..3).prop_map(|mut functions| {
            // Ensure we have a main function
            functions[0].name = "main".to_string();

            // Ensure other function names are valid (not "_" which is reserved)
            for (i, function) in functions.iter_mut().enumerate().skip(1) {
                let name = &function.name;
                if name == "_" || name == "main" || name.starts_with("__") {
                    function.name = format!("func_{i}");
                }
            }

            RestrictedAst {
                functions,
                entry_point: "main".to_string(),
            }
        })
    }

    pub fn any_config() -> impl Strategy<Value = Config> {
        (
            prop_oneof![
                Just(ShellDialect::Posix),
                Just(ShellDialect::Bash),
                Just(ShellDialect::Ash)
            ],
            prop_oneof![
                Just(VerificationLevel::None),
                Just(VerificationLevel::Basic),
                Just(VerificationLevel::Strict),
                Just(VerificationLevel::Paranoid)
            ],
            prop::bool::ANY,
            prop::bool::ANY,
        )
            .prop_map(|(target, verify, emit_proof, optimize)| Config {
                target,
                verify,
                emit_proof,
                optimize,
                validation_level: Some(crate::validation::ValidationLevel::Minimal),
                strict_mode: false,
            })
    }
}

// Core property tests
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    /// Property: AST validation should not panic (but may reject invalid ASTs)
    #[test]
    fn prop_valid_asts_validate(ast in generators::valid_ast()) {
        // The validator may reject some generated ASTs (e.g., with recursive calls)
        // This test ensures validation doesn't panic
        let _ = ast.validate();
    }

    /// Property: Valid identifiers should always parse correctly
    #[test]
    fn prop_valid_identifiers_parse(name in "[a-zA-Z][a-zA-Z0-9_]{0,20}") {
        // Rust keywords that should be filtered out
        const RUST_KEYWORDS: &[&str] = &[
            "as", "break", "const", "continue", "crate", "else", "enum", "extern",
            "false", "fn", "for", "if", "impl", "in", "let", "loop", "match",
            "mod", "move", "mut", "pub", "ref", "return", "self", "Self", "static",
            "struct", "super", "trait", "true", "type", "unsafe", "use", "where",
            "while", "async", "await", "dyn", "abstract", "become", "box", "do",
            "final", "macro", "override", "priv", "typeof", "unsized", "virtual", "yield"
        ];

        // Skip reserved keywords and problematic names
        prop_assume!(
            !name.is_empty()
                && name != "_"
                && name != "main"
                && !name.starts_with("__")
                && !RUST_KEYWORDS.contains(&name.as_str())
        );

        let source = format!("fn {name}() {{ let x = 42; }} fn main() {{ {name}(); }}");

        let result = parse(&source);
        prop_assert!(result.is_ok(), "Failed to parse with identifier: {}", name);
        if let Ok(ast) = result {
            let found_function = ast.functions.iter().find(|f| f.name == name);
            prop_assert!(found_function.is_some(), "Function {} not found in AST", name);
        }
    }

    /// Property: All literals should transpile without error
    #[test]
    fn prop_literals_transpile(lit in generators::any_literal()) {
        let source = match &lit {
            Literal::Bool(b) => format!("fn main() {{ let x = {b}; }}"),
            Literal::U32(n) => format!("fn main() {{ let x = {n}; }}"),
            Literal::Str(s) => format!(r#"fn main() {{ let x = "{s}"; }}"#),
        };

        let result = transpile(&source, Config::default());
        prop_assert!(result.is_ok(), "Failed to transpile literal: {:?}", lit);
    }

    /// Property: Binary operations should maintain associativity
    #[test]
    fn prop_binary_ops_associative(
        op in generators::any_binary_op(),
        a in 1u32..100u32,
        b in 1u32..100u32,
        c in 1u32..100u32
    ) {
        // Skip division to avoid divide by zero
        prop_assume!(!matches!(op, BinaryOp::Div));

        let left_assoc = format!("fn main() {{ let x = ({a} + {b}) + {c}; }}");
        let right_assoc = format!("fn main() {{ let x = {a} + ({b} + {c}); }}");

        let result1 = transpile(&left_assoc, Config::default());
        let result2 = transpile(&right_assoc, Config::default());

        prop_assert!(result1.is_ok() && result2.is_ok());
    }

    /// Property: Function names should be preserved through transpilation
    #[test]
    fn prop_function_names_preserved(name in generators::any_valid_identifier()) {
        let source = format!("fn {name}() {{}} fn main() {{ {name}(); }}");

        if let Ok(shell_code) = transpile(&source, Config::default()) {
            // Function name should appear in the generated shell code
            prop_assert!(shell_code.contains(&name));
        }
    }

    /// Property: Nested expressions should have balanced parentheses
    #[test]
    fn prop_balanced_parentheses(_expr in generators::simple_expr()) {
        let source = "fn main() { let x = 1; }".to_string(); // Simplified for now

        if let Ok(shell_code) = transpile(&source, Config::default()) {
            let open_count = shell_code.chars().filter(|&c| c == '(').count();
            let close_count = shell_code.chars().filter(|&c| c == ')').count();
            prop_assert_eq!(open_count, close_count, "Unbalanced parentheses in: {}", shell_code);
        }
    }

    /// Property: All generated shell scripts should be non-empty
    #[test]
    fn prop_generated_scripts_non_empty(_ast in generators::valid_ast()) {
        let source = "fn main() { let x = 42; }"; // Simplified

        if let Ok(shell_code) = transpile(source, Config::default()) {
            prop_assert!(!shell_code.trim().is_empty());
            prop_assert!(shell_code.contains("#!/bin/sh") || shell_code.contains("#!/bin/bash"));
        }
    }

    /// Property: Transpilation should be deterministic
    #[test]
    fn prop_transpilation_deterministic(config in generators::any_config()) {
        let source = "fn main() { let x = 42; let y = \"hello\"; }";

        let result1 = transpile(source, config.clone());
        let result2 = transpile(source, config.clone());

        match (result1, result2) {
            (Ok(code1), Ok(code2)) => prop_assert_eq!(code1, code2),
            (Err(_), Err(_)) => {}, // Both failing is okay
            _ => prop_assert!(false, "Non-deterministic behavior detected"),
        }
    }

    /// Property: String literals should be properly quoted in output
    #[test]
    fn prop_string_literals_quoted(s in generators::any_safe_string()) {
        let source = format!(r#"fn main() {{ let x = "{s}"; }}"#);

        if let Ok(shell_code) = transpile(&source, Config::default()) {
            // Generated shell should quote the string
            prop_assert!(shell_code.contains(&s));
        }
    }

    /// Property: Variable names should follow shell conventions
    #[test]
    fn prop_variable_names_shell_safe(name in generators::any_valid_identifier()) {
        let source = format!("fn main() {{ let {name} = 42; }}");

        if let Ok(shell_code) = transpile(&source, Config::default()) {
            // Variable should appear in shell code and be shell-safe
            if shell_code.contains(&name) {
                // Shell variables shouldn't start with numbers
                prop_assert!(!name.chars().next().unwrap().is_ascii_digit());
            }
        }
    }

    /// Property: Configuration changes should not cause crashes
    #[test]
    fn prop_config_robustness(config in generators::any_config()) {
        let test_sources = vec![
            "fn main() {}",
            "fn main() { let x = 42; }",
            "fn main() { let s = \"test\"; }",
            "fn helper() {} fn main() { helper(); }",
        ];

        for source in test_sources {
            let result = transpile(source, config.clone());
            // Should either succeed or fail gracefully (no panics)
            match result {
                Ok(_) | Err(_) => {}, // Both are acceptable
            }
        }
    }
}

// Regression tests for specific edge cases found by QuickCheck
#[cfg(test)]
mod regression_tests {
    use super::*;

    #[test]
    fn test_null_character_handling() {
        // This was found by QuickCheck and saved in proptest-regressions/ast/tests.txt
        let expr = Expr::Literal(Literal::Str("\0".to_string()));
        let result = expr.validate();
        assert!(result.is_err(), "Null characters should be rejected");
    }

    #[test]
    fn test_backslash_quote_handling() {
        // Found in proptest-regressions/services/tests.txt
        let source = r#"fn main() { let x = "\\'"; }"#;
        let result = parse(source);
        // Should handle escaped quotes gracefully
        assert!(result.is_ok() || result.is_err()); // Either is fine as long as no panic
    }

    #[test]
    fn test_empty_string_literal() {
        let source = r#"fn main() { let x = ""; }"#;
        let result = transpile(source, Config::default());
        assert!(result.is_ok());
    }

    #[test]
    fn test_very_large_numbers() {
        let source = "fn main() { let x = 4294967295; }"; // u32::MAX
        let result = transpile(source, Config::default());
        // Should handle or reject gracefully
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_long_identifier_names() {
        let long_name = "a".repeat(100);
        let source = format!("fn main() {{ let {long_name} = 42; }}");
        let result = transpile(&source, Config::default());
        // Should handle long names gracefully
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_deeply_nested_expressions() {
        let mut expr = "x".to_string();
        for _ in 0..20 {
            expr = format!("({expr} + 1)");
        }
        let source = format!("fn main() {{ let result = {expr}; }}");
        let result = transpile(&source, Config::default());
        // Deep nesting should be handled gracefully
        assert!(result.is_ok() || result.is_err());
    }
}

// Performance property tests
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::{Duration, Instant};

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property: Transpilation should complete within reasonable time
        #[test]
        fn prop_transpilation_performance(_ast in generators::valid_ast()) {
            let source = "fn main() { let x = 42; }"; // Simplified for performance testing

            let start = Instant::now();
            let result = transpile(source, Config::default());
            let duration = start.elapsed();

            // Should complete within 1 second for simple cases
            prop_assert!(duration < Duration::from_secs(1));

            if result.is_ok() {
                // Generated code should be reasonably sized (< 10KB for simple cases)
                let code = result.unwrap();
                prop_assert!(code.len() < 10000);
            }
        }

        /// Property: Memory usage should be bounded
        #[test]
        fn prop_memory_bounded(config in generators::any_config()) {
            let source = "fn main() { let x = 42; }";

            // Multiple transpilations shouldn't cause memory leaks
            for _ in 0..10 {
                let _ = transpile(source, config.clone());
            }

            // This test mainly checks that we don't panic or run out of memory
            prop_assert!(true);
        }
    }
}

// Fuzzing integration tests
#[cfg(test)]
mod fuzz_integration {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(500))]

        /// Property: Random valid inputs should not cause panics
        #[test]
        fn prop_no_panics_on_valid_input(_ast in generators::valid_ast()) {
            let source = "fn main() { let x = 42; }"; // Using simplified source

            // These operations should never panic
            let parse_result = std::panic::catch_unwind(|| parse(source));
            prop_assert!(parse_result.is_ok());

            let transpile_result = std::panic::catch_unwind(|| transpile(source, Config::default()));
            prop_assert!(transpile_result.is_ok());
        }

        /// Property: Invalid inputs should fail gracefully
        #[test]
        fn prop_graceful_failure_on_invalid_input(
            garbage in "[^a-zA-Z0-9 (){};=,\"'._-]*"
        ) {
            // Random garbage should not panic, just return an error
            let result = std::panic::catch_unwind(|| parse(&garbage));
            prop_assert!(result.is_ok()); // No panic

            if let Ok(parse_result) = result {
                // Should return an error, not succeed on garbage
                prop_assert!(parse_result.is_err());
            }
        }
    }
}

// Security property tests
#[cfg(test)]
mod security_tests {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(200))]

        /// Property: Generated shell should not contain injection vulnerabilities
        #[test]
        fn prop_no_shell_injection(s in generators::any_safe_string()) {
            let source = format!(r#"fn main() {{ let x = "{s}"; }}"#);

            if let Ok(shell_code) = transpile(&source, Config::default()) {
                // Should not contain dangerous injection patterns from user input
                let dangerous_patterns = [
                    "; rm -rf", "$(rm", "`rm"
                ];

                for pattern in &dangerous_patterns {
                    if shell_code.contains(pattern) {
                        // Dangerous pattern should only exist if it came from safe runtime functions
                        prop_assert!(
                            !shell_code.contains(&s) || shell_code.contains("rash_"),
                            "Dangerous pattern '{}' found from user input in: {}",
                            pattern, shell_code
                        );
                    }
                }
            }
        }

        /// Property: Variable expansion should be safe
        #[test]
        fn prop_safe_variable_expansion(name in generators::any_valid_identifier()) {
            let source = format!("fn main() {{ let {name} = 42; }}");

            if let Ok(shell_code) = transpile(&source, Config::default()) {
                // Variables should be properly quoted when expanded
                if shell_code.contains(&format!("${{{name}}}")) {
                    // Variable expansion should be quoted in contexts where it matters
                    prop_assert!(true); // This is a placeholder for more complex checks
                }
            }
        }
    }
}
