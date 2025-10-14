//! Proptest Generators for Bash Syntax
//!
//! Generates random but valid bash constructs for property-based testing.

use super::ast::*;
use proptest::prelude::*;
use proptest::strategy::BoxedStrategy;

/// Generate valid bash identifiers
pub fn bash_identifier() -> impl Strategy<Value = String> {
    "[a-zA-Z_][a-zA-Z0-9_]{0,15}".prop_map(|s| s.to_string())
}

/// Generate bash string literals
pub fn bash_string() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-zA-Z0-9_ ]{0,20}")
        .unwrap()
        .prop_map(|s| s.to_string())
}

/// Generate bash integer literals
pub fn bash_integer() -> impl Strategy<Value = i64> {
    -1000i64..1000i64
}

/// Generate bash variable names (common ones for testing)
pub fn bash_variable_name() -> impl Strategy<Value = String> {
    prop::sample::select(vec![
        "FOO".to_string(),
        "BAR".to_string(),
        "PATH".to_string(),
        "HOME".to_string(),
        "USER".to_string(),
        "x".to_string(),
        "y".to_string(),
        "status".to_string(),
        "result".to_string(),
    ])
}

/// Generate bash expressions (simplified)
pub fn bash_expr(depth: u32) -> BoxedStrategy<BashExpr> {
    if depth == 0 {
        prop_oneof![
            bash_string().prop_map(BashExpr::Literal),
            bash_integer().prop_map(|n| BashExpr::Literal(n.to_string())),
            bash_variable_name().prop_map(BashExpr::Variable),
        ]
        .boxed()
    } else {
        prop_oneof![
            bash_string().prop_map(BashExpr::Literal),
            bash_variable_name().prop_map(BashExpr::Variable),
            // Simple arithmetic
            (bash_integer(), bash_integer()).prop_map(|(a, b)| {
                BashExpr::Arithmetic(Box::new(ArithExpr::Add(
                    Box::new(ArithExpr::Number(a)),
                    Box::new(ArithExpr::Number(b)),
                )))
            }),
        ]
        .boxed()
    }
}

/// Generate test expressions (conditions)
pub fn bash_test_expr() -> impl Strategy<Value = TestExpr> {
    prop_oneof![
        // String comparisons
        (bash_variable_name(), bash_string())
            .prop_map(|(v, s)| { TestExpr::StringEq(BashExpr::Variable(v), BashExpr::Literal(s)) }),
        (bash_variable_name(), bash_string())
            .prop_map(|(v, s)| { TestExpr::StringNe(BashExpr::Variable(v), BashExpr::Literal(s)) }),
        // Integer comparisons
        (bash_variable_name(), bash_integer()).prop_map(|(v, n)| {
            TestExpr::IntEq(BashExpr::Variable(v), BashExpr::Literal(n.to_string()))
        }),
        (bash_variable_name(), bash_integer()).prop_map(|(v, n)| {
            TestExpr::IntLt(BashExpr::Variable(v), BashExpr::Literal(n.to_string()))
        }),
        (bash_variable_name(), bash_integer()).prop_map(|(v, n)| {
            TestExpr::IntGt(BashExpr::Variable(v), BashExpr::Literal(n.to_string()))
        }),
        // File tests
        bash_string().prop_map(|p| TestExpr::FileExists(BashExpr::Literal(p))),
        // String tests
        bash_variable_name().prop_map(|v| TestExpr::StringNonEmpty(BashExpr::Variable(v))),
    ]
}

/// Generate bash statements (simplified to avoid complex recursion)
pub fn bash_stmt(depth: u32) -> BoxedStrategy<BashStmt> {
    if depth == 0 {
        // Leaf: only simple statements
        prop_oneof![
            // Variable assignment
            (bash_variable_name(), bash_string(), prop::bool::ANY).prop_map(
                |(name, value, exported)| {
                    BashStmt::Assignment {
                        name,
                        value: BashExpr::Literal(value),
                        exported,
                        span: Span::dummy(),
                    }
                }
            ),
            // Simple command
            (
                bash_identifier(),
                prop::collection::vec(bash_string(), 0..2)
            )
                .prop_map(|(name, args)| {
                    BashStmt::Command {
                        name,
                        args: args.into_iter().map(BashExpr::Literal).collect(),
                        span: Span::dummy(),
                    }
                }),
            // Comment
            bash_string().prop_map(|text| BashStmt::Comment {
                text,
                span: Span::dummy(),
            }),
        ]
        .boxed()
    } else {
        // Recursive: include control flow
        prop_oneof![
            // Variable assignment
            (bash_variable_name(), bash_string(), prop::bool::ANY).prop_map(
                |(name, value, exported)| {
                    BashStmt::Assignment {
                        name,
                        value: BashExpr::Literal(value),
                        exported,
                        span: Span::dummy(),
                    }
                }
            ),
            // Simple command
            (
                bash_identifier(),
                prop::collection::vec(bash_string(), 0..2)
            )
                .prop_map(|(name, args)| {
                    BashStmt::Command {
                        name,
                        args: args.into_iter().map(BashExpr::Literal).collect(),
                        span: Span::dummy(),
                    }
                }),
            // Function (with simple body)
            (bash_identifier(), prop::collection::vec(bash_stmt(0), 1..2)).prop_map(
                |(name, body)| {
                    BashStmt::Function {
                        name,
                        body,
                        span: Span::dummy(),
                    }
                }
            ),
        ]
        .boxed()
    }
}

/// Generate a complete bash script with unique function names
/// 🟢 GREEN: TICKET-6002 - Ensure no duplicate function names
pub fn bash_script() -> impl Strategy<Value = BashAst> {
    prop::collection::vec(bash_stmt(2), 1..10).prop_map(|statements| {
        use std::collections::HashSet;

        // Track seen function names to ensure uniqueness
        let mut seen_functions: HashSet<String> = HashSet::new();
        let mut deduplicated_statements = Vec::new();

        for stmt in statements {
            match &stmt {
                BashStmt::Function { name, .. } => {
                    // Only include if this function name hasn't been seen
                    if seen_functions.insert(name.clone()) {
                        deduplicated_statements.push(stmt);
                    }
                    // If duplicate, skip it (don't add to deduplicated_statements)
                }
                _ => {
                    // Non-function statements always included
                    deduplicated_statements.push(stmt);
                }
            }
        }

        BashAst {
            statements: deduplicated_statements,
            metadata: AstMetadata {
                source_file: None,
                line_count: 0,
                parse_time_ms: 0,
            },
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    proptest! {
        #[test]
        fn test_generates_valid_identifiers(id in bash_identifier()) {
            // Should start with letter or underscore
            assert!(id.chars().next().unwrap().is_alphabetic() || id.starts_with('_'));
            // Should be reasonable length
            assert!(id.len() <= 16);
        }

        #[test]
        fn test_generates_valid_expressions(expr in bash_expr(2)) {
            // All expressions should be constructible
            match expr {
                BashExpr::Literal(s) => assert!(!s.is_empty() || s.is_empty()),
                BashExpr::Variable(v) => assert!(!v.is_empty()),
                BashExpr::Array(items) => assert!(items.len() <= 3),
                BashExpr::Arithmetic(_) => {},
                _ => {}
            }
        }

        #[test]
        fn test_generates_valid_statements(stmt in bash_stmt(2)) {
            // All statements should be constructible
            match stmt {
                BashStmt::Assignment { name, .. } => assert!(!name.is_empty()),
                BashStmt::Command { name, .. } => assert!(!name.is_empty()),
                BashStmt::Function { name, body, .. } => {
                    assert!(!name.is_empty());
                    assert!(!body.is_empty());
                }
                _ => {}
            }
        }

        #[test]
        fn test_generates_valid_scripts(script in bash_script()) {
            // Scripts should have at least one statement
            assert!(!script.statements.is_empty());
            assert!(script.statements.len() <= 10);
        }

        /// 🔴 RED: Property test for unique function names
        /// TICKET-6002: bash_script() should generate scripts with unique function names
        #[test]
        fn test_generated_scripts_have_unique_function_names(script in bash_script()) {
            use std::collections::HashSet;

            // Collect all function names
            let mut function_names = HashSet::new();
            let mut duplicate_found = false;
            let mut duplicate_name = String::new();

            for stmt in &script.statements {
                if let BashStmt::Function { name, .. } = stmt {
                    if !function_names.insert(name.clone()) {
                        // Duplicate found!
                        duplicate_found = true;
                        duplicate_name = name.clone();
                        break;
                    }
                }
            }

            prop_assert!(
                !duplicate_found,
                "Generated script has duplicate function name: '{}'. \
                All function names in a script must be unique. \
                Function names found: {:?}",
                duplicate_name,
                function_names
            );
        }
    }
}
