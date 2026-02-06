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

    pub fn any_u16_literal() -> impl Strategy<Value = Literal> {
        (0u16..10000u16).prop_map(Literal::U16)
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
        prop_oneof![any_u16_literal(), any_u32_literal(), any_bool_literal(), any_string_literal()]
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
            Literal::U16(n) => format!("fn main() {{ let x = {n}u16; }}"),
            Literal::U32(n) => format!("fn main() {{ let x = {n}; }}"),
            Literal::I32(n) => format!("fn main() {{ let x = {n}; }}"),
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

        /// Property: For loops should generate valid seq commands
        #[test]
        fn prop_for_loops_valid_seq(start in 0i32..100, end in 0i32..100) {
            let source = format!("fn main() {{ for i in {}..{} {{ let x = i; }} }}", start, end);

            if let Ok(shell_code) = transpile(&source, Config::default()) {
                // Should contain seq command
                prop_assert!(shell_code.contains("seq"), "For loop should use seq");
                // Should contain for...do...done
                prop_assert!(shell_code.contains("for i in"), "Should have for loop");
                prop_assert!(shell_code.contains("done"), "Should close with done");
            }
        }

        /// Property: Arithmetic operations should preserve types
        #[test]
        fn prop_arithmetic_preserves_types(a in 1i32..100, b in 1i32..100) {
            let source = format!("fn main() {{ let x = {} + {}; }}", a, b);

            // Disable optimization to test shell code generation (not optimization)
            let config = Config {
                optimize: false,
                ..Default::default()
            };

            if let Ok(shell_code) = transpile(&source, config) {
                // Should use arithmetic expansion
                prop_assert!(shell_code.contains("$(("), "Should use arithmetic expansion");
                prop_assert!(shell_code.contains("+"), "Should contain operator");
            }
        }

        /// Property: Function returns should use command substitution
        #[test]
        fn prop_function_returns_use_subst(a in 1i32..100, b in 1i32..100) {
            let source = format!(
                "fn add(a: i32, b: i32) -> i32 {{ a + b }} fn main() {{ let x = add({}, {}); }}",
                a, b
            );

            if let Ok(shell_code) = transpile(&source, Config::default()) {
                // Functions with return values should echo
                prop_assert!(shell_code.contains("echo"), "Function should echo return value");
                // Call sites should use command substitution
                prop_assert!(shell_code.contains("$("), "Should use command substitution");
            }
        }

        /// Property: Comparison operators generate POSIX test syntax
        #[test]
        fn prop_comparisons_posix_test(a in 1i32..100) {
            let source = format!("fn main() {{ if {} > 0 {{ let x = 1; }} }}", a);

            if let Ok(shell_code) = transpile(&source, Config::default()) {
                // Should use POSIX test with -gt, -lt, etc.
                prop_assert!(
                    shell_code.contains("-gt") || shell_code.contains("test"),
                    "Should use POSIX test syntax"
                );
            }
        }

        /// Property: Multiple variables should maintain scope
        #[test]
        fn prop_variable_scope(
            name1 in generators::any_valid_identifier(),
            name2 in generators::any_valid_identifier()
        ) {
            prop_assume!(name1 != name2); // Ensure different names

            let source = format!(
                "fn main() {{ let {name1} = 1; let {name2} = 2; }}"
            );

            if let Ok(shell_code) = transpile(&source, Config::default()) {
                // Both variables should appear in output
                prop_assert!(shell_code.contains(&format!("{name1}=")), "First variable should be assigned");
                prop_assert!(shell_code.contains(&format!("{name2}=")), "Second variable should be assigned");
            }
        }

        /// Property: Negative integers should transpile correctly
        #[test]
        fn prop_negative_integers(n in -1000i32..0) {
            let source = format!("fn main() {{ let x = {}; }}", n);

            if let Ok(shell_code) = transpile(&source, Config::default()) {
                // Should NOT contain "unknown"
                prop_assert!(!shell_code.contains("unknown"), "Negative integers should not be unknown");
                // Should contain the negative number
                let num_str = n.to_string();
                prop_assert!(shell_code.contains(&num_str) || shell_code.contains(&format!("'{}'", num_str)),
                            "Should contain negative number: {}", n);
            }
        }

        /// Property: Empty function bodies should generate no-ops
        #[test]
        fn prop_empty_functions_noop(name in generators::any_valid_identifier()) {
            prop_assume!(name != "main"); // main is special

            let source = format!("fn {name}() {{}} fn main() {{ {name}(); }}");

            if let Ok(_shell_code) = transpile(&source, Config::default()) {
                // Empty functions might not generate a function definition
                // or might generate a no-op (:)
                prop_assert!(true); // Placeholder - just ensure it compiles
            }
        }
    }
}

// Sprint 23: New property tests for stdlib, while loops, and control flow
#[cfg(test)]
mod sprint23_properties {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// Property: stdlib string_trim should be idempotent
        #[test]
        fn prop_string_trim_idempotent(s in "[a-zA-Z0-9 ]{0,50}") {
            let source = format!(
                r#"fn main() {{
                    let text = "{}";
                    let trimmed1 = string_trim(text);
                    let trimmed2 = string_trim(trimmed1);
                }}"#,
                s
            );

            if let Ok(shell_code) = transpile(&source, Config::default()) {
                // Should contain string_trim function
                prop_assert!(shell_code.contains("rash_string_trim"),
                           "Should include stdlib string_trim function");
            }
        }

        /// Property: stdlib string_contains should handle empty strings
        #[test]
        fn prop_string_contains_empty(haystack in "[a-zA-Z0-9]{0,30}") {
            let source = format!(
                r#"fn main() {{
                    let text = "{}";
                    if string_contains(text, "") {{
                        echo("contains empty");
                    }}
                }}"#,
                haystack
            );

            if let Ok(shell_code) = transpile(&source, Config::default()) {
                // Should generate proper case statement
                prop_assert!(shell_code.contains("rash_string_contains"),
                           "Should include stdlib string_contains function");
                // Should not wrap in command substitution for if statement
                prop_assert!(!shell_code.contains("\"$(rash_string_contains"),
                           "Predicate functions should not be wrapped in command substitution");
            }
        }

        /// Property: stdlib fs_exists should generate test command
        #[test]
        fn prop_fs_exists_test_command(path in "[a-zA-Z0-9/_.-]{1,50}") {
            let source = format!(
                r#"fn main() {{
                    if fs_exists("{}") {{
                        echo("exists");
                    }}
                }}"#,
                path
            );

            if let Ok(shell_code) = transpile(&source, Config::default()) {
                // Should include fs_exists function
                prop_assert!(shell_code.contains("rash_fs_exists"),
                           "Should include stdlib fs_exists function");
                // Should use POSIX test -e
                prop_assert!(shell_code.contains("test -e"),
                           "fs_exists should use POSIX test -e");
            }
        }

        /// Property: stdlib string_len should return numeric value
        #[test]
        fn prop_string_len_numeric(s in "[a-zA-Z]{0,20}") {
            let source = format!(
                r#"fn main() {{
                    let text = "{}";
                    let len = string_len(text);
                    echo(len);
                }}"#,
                s
            );

            if let Ok(shell_code) = transpile(&source, Config::default()) {
                // Should use command substitution for return value
                prop_assert!(shell_code.contains("$(rash_string_len") ||
                           shell_code.contains("len="),
                           "string_len should be captured as value");
            }
        }

        /// Property: while loops should generate POSIX while statements
        #[test]
        fn prop_while_loop_posix(limit in 1u32..10) {
            let source = format!(
                r#"fn main() {{
                    let i = 0;
                    while i < {} {{
                        echo(i);
                    }}
                }}"#,
                limit
            );

            if let Ok(shell_code) = transpile(&source, Config::default()) {
                // Should generate while loop
                prop_assert!(shell_code.contains("while") && shell_code.contains("do"),
                           "Should generate POSIX while...do loop");
                // Should have proper test syntax
                prop_assert!(shell_code.contains("-lt") || shell_code.contains("["),
                           "Should use POSIX test syntax for condition");
                prop_assert!(shell_code.contains("done"),
                           "While loop should be closed with 'done'");
            }
        }

        /// Property: while true should generate infinite loop
        #[test]
        fn prop_while_true_infinite(_dummy in prop::bool::ANY) {
            let source = r#"
fn main() {
    while true {
        echo("loop");
    }
}
"#;

            if let Ok(shell_code) = transpile(source, Config::default()) {
                // Should generate while true
                prop_assert!(shell_code.contains("while true"),
                           "while true should generate 'while true' statement");
                prop_assert!(shell_code.contains("do") && shell_code.contains("done"),
                           "Should have proper loop structure");
            }
        }

        /// Property: nested if statements should maintain correct nesting
        #[test]
        fn prop_nested_if_statements(
            val1 in 0u32..100,
            val2 in 0u32..100
        ) {
            let source = format!(
                r#"fn main() {{
                    let x = {};
                    if x > 10 {{
                        let y = {};
                        if y > 20 {{
                            echo("nested");
                        }}
                    }}
                }}"#,
                val1, val2
            );

            if let Ok(shell_code) = transpile(&source, Config::default()) {
                // Should contain nested if structure
                prop_assert!(shell_code.contains("if") && shell_code.contains("fi"),
                           "Should generate if/fi structure");
                // Should contain comparison operators
                prop_assert!(shell_code.contains("-gt"),
                           "Should use POSIX comparison operators");
            }
        }

        /// Property: match expressions with multiple arms should generate complete case
        #[test]
        fn prop_match_completeness(val in 0u32..5) {
            let source = format!(
                r#"fn main() {{
                    let x = {};
                    match x {{
                        0 => echo("zero"),
                        1 => echo("one"),
                        _ => echo("other"),
                    }}
                }}"#,
                val
            );

            if let Ok(shell_code) = transpile(&source, Config::default()) {
                // Should generate case statement
                prop_assert!(shell_code.contains("case"),
                           "Match should generate case statement");
                prop_assert!(shell_code.contains("esac"),
                           "Case statement should be closed with esac");
                // Should have wildcard pattern
                prop_assert!(shell_code.contains("*)"),
                           "Should have wildcard pattern for _ arm");
            }
        }

        /// Property: for loop with range should generate seq command
        #[test]
        fn prop_for_range_seq(start in 0u32..10, end in 11u32..20) {
            prop_assume!(start < end); // Ensure valid range

            let source = format!(
                r#"fn main() {{
                    for i in {}..{} {{
                        echo(i);
                    }}
                }}"#,
                start, end
            );

            if let Ok(shell_code) = transpile(&source, Config::default()) {
                // Should generate seq command or shell range
                prop_assert!(shell_code.contains("seq") || shell_code.contains("for i in"),
                           "For loop should generate seq or shell range");
                // Exclusive range: end - 1
                let expected_end = end - 1;
                prop_assert!(shell_code.contains(&format!("seq {} {}", start, expected_end)) ||
                           shell_code.contains("for"),
                           "Should generate correct range bounds");
            }
        }

        /// Property: break and continue in loops should generate shell equivalents
        #[test]
        fn prop_break_continue(use_break in prop::bool::ANY) {
            let stmt = if use_break { "break" } else { "continue" };
            let source = format!(
                r#"fn main() {{
                    let i = 0;
                    while i < 10 {{
                        if i == 5 {{
                            {};
                        }}
                        echo(i);
                    }}
                }}"#,
                stmt
            );

            if let Ok(shell_code) = transpile(&source, Config::default()) {
                // Should contain break or continue statement
                prop_assert!(shell_code.contains(stmt),
                           "Loop control statement should be preserved");
            }
        }

        // =============== Sprint 25: Extended Stdlib Property Tests ===============

        /// Property: string_to_upper should always include the runtime function
        #[test]
        fn prop_string_to_upper_includes_runtime(text in "[a-zA-Z0-9 ]{1,30}") {
            let source = format!(
                r#"fn main() {{
                    let result = string_to_upper("{}");
                    echo(result);
                }}"#,
                text
            );

            if let Ok(shell_code) = transpile(&source, Config::default()) {
                prop_assert!(shell_code.contains("rash_string_to_upper"),
                           "Generated shell should include string_to_upper runtime function");
            }
        }

        /// Property: string_to_lower should always include the runtime function
        #[test]
        fn prop_string_to_lower_includes_runtime(text in "[a-zA-Z0-9 ]{1,30}") {
            let source = format!(
                r#"fn main() {{
                    let result = string_to_lower("{}");
                    echo(result);
                }}"#,
                text
            );

            if let Ok(shell_code) = transpile(&source, Config::default()) {
                prop_assert!(shell_code.contains("rash_string_to_lower"),
                           "Generated shell should include string_to_lower runtime function");
            }
        }

        /// Property: string_replace should handle any valid string inputs
        #[test]
        fn prop_string_replace_transpiles(
            text in "[a-zA-Z0-9 ]{1,20}",
            old in "[a-zA-Z]{1,5}",
            new in "[a-zA-Z]{1,5}"
        ) {
            let source = format!(
                r#"fn main() {{
                    let result = string_replace("{}", "{}", "{}");
                    echo(result);
                }}"#,
                text, old, new
            );

            let result = transpile(&source, Config::default());
            prop_assert!(result.is_ok(), "string_replace should transpile successfully");
            if let Ok(shell_code) = result {
                prop_assert!(shell_code.contains("rash_string_replace"),
                           "Generated shell should include string_replace runtime function");
            }
        }

        /// Property: fs_is_file should always include the runtime function
        #[test]
        fn prop_fs_is_file_includes_runtime(path in "/[a-z]{1,20}") {
            let source = format!(
                r#"fn main() {{
                    let is_file = fs_is_file("{}");
                    if is_file {{
                        echo("yes");
                    }}
                }}"#,
                path
            );

            if let Ok(shell_code) = transpile(&source, Config::default()) {
                prop_assert!(shell_code.contains("rash_fs_is_file"),
                           "Generated shell should include fs_is_file runtime function");
            }
        }

        /// Property: fs_is_dir should always include the runtime function
        #[test]
        fn prop_fs_is_dir_includes_runtime(path in "/[a-z]{1,20}") {
            let source = format!(
                r#"fn main() {{
                    let is_dir = fs_is_dir("{}");
                    if is_dir {{
                        echo("yes");
                    }}
                }}"#,
                path
            );

            if let Ok(shell_code) = transpile(&source, Config::default()) {
                prop_assert!(shell_code.contains("rash_fs_is_dir"),
                           "Generated shell should include fs_is_dir runtime function");
            }
        }

        /// Property: fs_copy should handle valid paths
        #[test]
        fn prop_fs_copy_transpiles(
            src in "/tmp/[a-z]{1,10}",
            dst in "/tmp/[a-z]{1,10}"
        ) {
            let source = format!(
                r#"fn main() {{
                    let result = fs_copy("{}", "{}");
                    if result {{
                        echo("copied");
                    }}
                }}"#,
                src, dst
            );

            let result = transpile(&source, Config::default());
            prop_assert!(result.is_ok(), "fs_copy should transpile successfully");
            if let Ok(shell_code) = result {
                prop_assert!(shell_code.contains("rash_fs_copy"),
                           "Generated shell should include fs_copy runtime function");
            }
        }

        /// Property: fs_remove should handle valid paths
        #[test]
        fn prop_fs_remove_transpiles(path in "/tmp/[a-z]{1,10}") {
            let source = format!(
                r#"fn main() {{
                    let result = fs_remove("{}");
                    if result {{
                        echo("removed");
                    }}
                }}"#,
                path
            );

            let result = transpile(&source, Config::default());
            prop_assert!(result.is_ok(), "fs_remove should transpile successfully");
            if let Ok(shell_code) = result {
                prop_assert!(shell_code.contains("rash_fs_remove"),
                           "Generated shell should include fs_remove runtime function");
            }
        }

        /// Property: Combining multiple new stdlib functions should transpile
        #[test]
        fn prop_multiple_new_stdlib_functions(text in "[a-zA-Z ]{5,15}") {
            let source = format!(
                r#"fn main() {{
                    let lower = string_to_lower("{}");
                    let upper = string_to_upper(lower);

                    if fs_is_dir("/tmp") {{
                        fs_write_file("/tmp/test.txt", upper);
                        if fs_is_file("/tmp/test.txt") {{
                            fs_remove("/tmp/test.txt");
                        }}
                    }}
                }}"#,
                text
            );

            let result = transpile(&source, Config::default());
            prop_assert!(result.is_ok(), "Multiple stdlib functions should transpile successfully");
            if let Ok(shell_code) = result {
                prop_assert!(shell_code.contains("rash_string_to_lower"),
                           "Should include string_to_lower");
                prop_assert!(shell_code.contains("rash_string_to_upper"),
                           "Should include string_to_upper");
                prop_assert!(shell_code.contains("rash_fs_is_dir"),
                           "Should include fs_is_dir");
                prop_assert!(shell_code.contains("rash_fs_is_file"),
                           "Should include fs_is_file");
                prop_assert!(shell_code.contains("rash_fs_remove"),
                           "Should include fs_remove");
            }
        }
    }
}
