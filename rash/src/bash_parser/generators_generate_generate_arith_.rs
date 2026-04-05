
/// Generate arithmetic expression
fn generate_arith_expr(expr: &ArithExpr) -> String {
    match expr {
        ArithExpr::Number(n) => n.to_string(),
        ArithExpr::Variable(v) => v.clone(),
        ArithExpr::Add(left, right) => {
            format!(
                "{} + {}",
                generate_arith_expr(left),
                generate_arith_expr(right)
            )
        }
        ArithExpr::Sub(left, right) => {
            format!(
                "{} - {}",
                generate_arith_expr(left),
                generate_arith_expr(right)
            )
        }
        ArithExpr::Mul(left, right) => {
            format!(
                "{} * {}",
                generate_arith_expr(left),
                generate_arith_expr(right)
            )
        }
        ArithExpr::Div(left, right) => {
            format!(
                "{} / {}",
                generate_arith_expr(left),
                generate_arith_expr(right)
            )
        }
        ArithExpr::Mod(left, right) => {
            format!(
                "{} % {}",
                generate_arith_expr(left),
                generate_arith_expr(right)
            )
        }
    }
}

/// Generate test expression
fn generate_test_expr(expr: &TestExpr) -> String {
    match expr {
        TestExpr::StringEq(left, right) => {
            format!("[ {} = {} ]", generate_expr(left), generate_expr(right))
        }
        TestExpr::StringNe(left, right) => {
            format!("[ {} != {} ]", generate_expr(left), generate_expr(right))
        }
        TestExpr::IntEq(left, right) => {
            format!("[ {} -eq {} ]", generate_expr(left), generate_expr(right))
        }
        TestExpr::IntNe(left, right) => {
            format!("[ {} -ne {} ]", generate_expr(left), generate_expr(right))
        }
        TestExpr::IntLt(left, right) => {
            format!("[ {} -lt {} ]", generate_expr(left), generate_expr(right))
        }
        TestExpr::IntLe(left, right) => {
            format!("[ {} -le {} ]", generate_expr(left), generate_expr(right))
        }
        TestExpr::IntGt(left, right) => {
            format!("[ {} -gt {} ]", generate_expr(left), generate_expr(right))
        }
        TestExpr::IntGe(left, right) => {
            format!("[ {} -ge {} ]", generate_expr(left), generate_expr(right))
        }
        TestExpr::FileExists(path) => {
            format!("[ -e {} ]", generate_expr(path))
        }
        TestExpr::FileReadable(path) => {
            format!("[ -r {} ]", generate_expr(path))
        }
        TestExpr::FileWritable(path) => {
            format!("[ -w {} ]", generate_expr(path))
        }
        TestExpr::FileExecutable(path) => {
            format!("[ -x {} ]", generate_expr(path))
        }
        TestExpr::FileDirectory(path) => {
            format!("[ -d {} ]", generate_expr(path))
        }
        TestExpr::StringEmpty(expr) => {
            format!("[ -z {} ]", generate_expr(expr))
        }
        TestExpr::StringNonEmpty(expr) => {
            format!("[ -n {} ]", generate_expr(expr))
        }
        TestExpr::And(left, right) => {
            format!(
                "{} && {}",
                generate_test_expr(left),
                generate_test_expr(right)
            )
        }
        TestExpr::Or(left, right) => {
            format!(
                "{} || {}",
                generate_test_expr(left),
                generate_test_expr(right)
            )
        }
        TestExpr::Not(expr) => {
            format!("! {}", generate_test_expr(expr))
        }
    }
}

/// Bash reserved keywords that cannot be used as standalone command names
const BASH_KEYWORDS: &[&str] = &[
    "if", "then", "elif", "else", "fi", "case", "esac", "for", "while", "until", "do", "done",
    "in", "function", "select", "time", "coproc",
];

/// Generate valid bash identifiers (excluding reserved keywords)
pub fn bash_identifier() -> impl Strategy<Value = String> {
    "[a-zA-Z_][a-zA-Z0-9_]{0,15}".prop_filter_map("filter out keywords", |s| {
        let lower = s.to_lowercase();
        if BASH_KEYWORDS.contains(&lower.as_str()) {
            None // Filter out keywords
        } else {
            Some(s)
        }
    })
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
                        index: None,
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
                        redirects: vec![],
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
                        index: None,
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
                        redirects: vec![],
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
#[path = "generators_tests_generates_va.rs"]
mod tests_extracted;
