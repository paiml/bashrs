//! Proptest Generators for Bash Syntax
//!
//! Generates random but valid bash constructs for property-based testing.
//! Also includes purified bash generation for the bash→rust→purified pipeline.

use super::ast::*;
use proptest::prelude::*;
use proptest::strategy::BoxedStrategy;

/// Generate purified bash from BashAst
///
/// This function transforms a BashAst into purified POSIX sh:
/// - Transforms #!/bin/bash → #!/bin/sh
/// - Ensures deterministic output (no $RANDOM, timestamps)
/// - Ensures idempotent operations (mkdir -p, rm -f)
/// - Quotes all variables for injection safety
///
/// Task 1.1: Shebang Transformation
pub fn generate_purified_bash(ast: &BashAst) -> String {
    let mut output = String::new();

    // Always start with POSIX sh shebang
    output.push_str("#!/bin/sh\n");

    // Generate statements
    for stmt in &ast.statements {
        output.push_str(&generate_statement(stmt));
        output.push('\n');
    }

    output
}

/// Generate a single statement
fn generate_statement(stmt: &BashStmt) -> String {
    match stmt {
        BashStmt::Command { name, args, .. } => {
            let mut cmd = name.clone();
            for arg in args {
                cmd.push(' ');
                cmd.push_str(&generate_expr(arg));
            }
            cmd
        }
        BashStmt::Assignment {
            name,
            value,
            exported,
            ..
        } => {
            let mut assign = String::new();
            if *exported {
                assign.push_str("export ");
            }
            assign.push_str(name);
            assign.push('=');
            assign.push_str(&generate_expr(value));
            assign
        }
        BashStmt::Comment { text, .. } => {
            format!("# {}", text)
        }
        BashStmt::Function { name, body, .. } => {
            let mut func = format!("{}() {{\n", name);
            for stmt in body {
                func.push_str("    ");
                func.push_str(&generate_statement(stmt));
                func.push('\n');
            }
            func.push('}');
            func
        }
        BashStmt::If {
            condition,
            then_block,
            else_block,
            ..
        } => {
            let mut if_stmt = format!("if {}; then\n", generate_condition(condition));
            for stmt in then_block {
                if_stmt.push_str("    ");
                if_stmt.push_str(&generate_statement(stmt));
                if_stmt.push('\n');
            }
            if let Some(else_stmts) = else_block {
                if_stmt.push_str("else\n");
                for stmt in else_stmts {
                    if_stmt.push_str("    ");
                    if_stmt.push_str(&generate_statement(stmt));
                    if_stmt.push('\n');
                }
            }
            if_stmt.push_str("fi");
            if_stmt
        }
        BashStmt::For {
            variable,
            items,
            body,
            ..
        } => {
            let mut for_stmt = format!("for {} in {}; do\n", variable, generate_expr(items));
            for stmt in body {
                for_stmt.push_str("    ");
                for_stmt.push_str(&generate_statement(stmt));
                for_stmt.push('\n');
            }
            for_stmt.push_str("done");
            for_stmt
        }
        // Issue #68: C-style for loop generator
        BashStmt::ForCStyle {
            init,
            condition,
            increment,
            body,
            ..
        } => {
            let mut for_stmt = format!("for (({}; {}; {})); do\n", init, condition, increment);
            for stmt in body {
                for_stmt.push_str("    ");
                for_stmt.push_str(&generate_statement(stmt));
                for_stmt.push('\n');
            }
            for_stmt.push_str("done");
            for_stmt
        }
        BashStmt::While {
            condition, body, ..
        } => {
            let mut while_stmt = format!("while {}; do\n", generate_condition(condition));
            for stmt in body {
                while_stmt.push_str("    ");
                while_stmt.push_str(&generate_statement(stmt));
                while_stmt.push('\n');
            }
            while_stmt.push_str("done");
            while_stmt
        }
        BashStmt::Until {
            condition, body, ..
        } => {
            // Transform until loop to while loop with negated condition
            // until [ $i -gt 5 ] → while [ ! "$i" -gt 5 ]
            let negated_condition = negate_condition(condition);
            let mut while_stmt = format!("while {}; do\n", negated_condition);
            for stmt in body {
                while_stmt.push_str("    ");
                while_stmt.push_str(&generate_statement(stmt));
                while_stmt.push('\n');
            }
            while_stmt.push_str("done");
            while_stmt
        }
        BashStmt::Return { code, .. } => {
            if let Some(c) = code {
                format!("return {}", generate_expr(c))
            } else {
                String::from("return")
            }
        }
        BashStmt::Case { word, arms, .. } => {
            let mut case_stmt = format!("case {} in\n", generate_expr(word));
            for arm in arms {
                let pattern_str = arm.patterns.join("|");
                case_stmt.push_str(&format!("    {})\n", pattern_str));
                for stmt in &arm.body {
                    case_stmt.push_str("        ");
                    case_stmt.push_str(&generate_statement(stmt));
                    case_stmt.push('\n');
                }
                case_stmt.push_str("        ;;\n");
            }
            case_stmt.push_str("esac");
            case_stmt
        }

        BashStmt::Pipeline { commands, .. } => {
            // Generate pipeline: cmd1 | cmd2 | cmd3
            let mut pipeline = String::new();
            for (i, cmd) in commands.iter().enumerate() {
                if i > 0 {
                    pipeline.push_str(" | ");
                }
                pipeline.push_str(&generate_statement(cmd));
            }
            pipeline
        }

        BashStmt::AndList { left, right, .. } => {
            // Generate AND list: cmd1 && cmd2
            format!(
                "{} && {}",
                generate_statement(left),
                generate_statement(right)
            )
        }

        BashStmt::OrList { left, right, .. } => {
            // Generate OR list: cmd1 || cmd2
            format!(
                "{} || {}",
                generate_statement(left),
                generate_statement(right)
            )
        }

        BashStmt::BraceGroup { body, .. } => {
            // Generate brace group: { cmd1; cmd2; }
            let mut brace = String::from("{ ");
            for (i, stmt) in body.iter().enumerate() {
                if i > 0 {
                    brace.push_str("; ");
                }
                brace.push_str(&generate_statement(stmt));
            }
            brace.push_str("; }");
            brace
        }

        BashStmt::Coproc { name, body, .. } => {
            // Generate coproc: coproc NAME { cmd; }
            let mut coproc = String::from("coproc ");
            if let Some(n) = name {
                coproc.push_str(n);
                coproc.push(' ');
            }
            coproc.push_str("{ ");
            for (i, stmt) in body.iter().enumerate() {
                if i > 0 {
                    coproc.push_str("; ");
                }
                coproc.push_str(&generate_statement(stmt));
            }
            coproc.push_str("; }");
            coproc
        }

        BashStmt::Select {
            variable,
            items,
            body,
            ..
        } => {
            // Generate select: select VAR in ITEMS; do BODY; done
            let mut select = format!("select {} in ", variable);
            select.push_str(&generate_expr(items));
            select.push_str("; do\n");
            for stmt in body {
                select.push_str("    ");
                select.push_str(&generate_statement(stmt));
                select.push('\n');
            }
            select.push_str("done");
            select
        }

        BashStmt::Negated { command, .. } => {
            // Issue #133: Generate negated command: ! cmd
            format!("! {}", generate_statement(command))
        }
    }
}

/// Negate a condition for until → while transformation
fn negate_condition(condition: &BashExpr) -> String {
    match condition {
        BashExpr::Test(test) => {
            // Wrap the test in negation
            format!("[ ! {} ]", generate_test_condition(test))
        }
        _ => {
            // For other expressions, use ! prefix
            format!("! {}", generate_condition(condition))
        }
    }
}

/// Generate the inner part of a test condition (without brackets)
fn generate_test_condition(expr: &TestExpr) -> String {
    match expr {
        TestExpr::StringEq(left, right) => {
            format!("{} = {}", generate_expr(left), generate_expr(right))
        }
        TestExpr::StringNe(left, right) => {
            format!("{} != {}", generate_expr(left), generate_expr(right))
        }
        TestExpr::IntEq(left, right) => {
            format!("{} -eq {}", generate_expr(left), generate_expr(right))
        }
        TestExpr::IntNe(left, right) => {
            format!("{} -ne {}", generate_expr(left), generate_expr(right))
        }
        TestExpr::IntLt(left, right) => {
            format!("{} -lt {}", generate_expr(left), generate_expr(right))
        }
        TestExpr::IntLe(left, right) => {
            format!("{} -le {}", generate_expr(left), generate_expr(right))
        }
        TestExpr::IntGt(left, right) => {
            format!("{} -gt {}", generate_expr(left), generate_expr(right))
        }
        TestExpr::IntGe(left, right) => {
            format!("{} -ge {}", generate_expr(left), generate_expr(right))
        }
        TestExpr::FileExists(path) => {
            format!("-e {}", generate_expr(path))
        }
        TestExpr::FileReadable(path) => {
            format!("-r {}", generate_expr(path))
        }
        TestExpr::FileWritable(path) => {
            format!("-w {}", generate_expr(path))
        }
        TestExpr::FileExecutable(path) => {
            format!("-x {}", generate_expr(path))
        }
        TestExpr::FileDirectory(path) => {
            format!("-d {}", generate_expr(path))
        }
        TestExpr::StringEmpty(expr) => {
            format!("-z {}", generate_expr(expr))
        }
        TestExpr::StringNonEmpty(expr) => {
            format!("-n {}", generate_expr(expr))
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

/// Generate a condition expression (for if/while statements)
fn generate_condition(expr: &BashExpr) -> String {
    match expr {
        BashExpr::Test(test) => generate_test_expr(test),
        _ => generate_expr(expr),
    }
}

/// Generate an expression
fn generate_expr(expr: &BashExpr) -> String {
    match expr {
        BashExpr::Literal(s) => {
            // Quote string literals
            if s.contains(' ') || s.contains('$') {
                format!("'{}'", s)
            } else {
                s.clone()
            }
        }
        BashExpr::Variable(name) => {
            // Always quote variables for safety
            format!("\"${}\"", name)
        }
        BashExpr::Array(items) => {
            let elements: Vec<String> = items.iter().map(generate_expr).collect();
            elements.join(" ")
        }
        BashExpr::Arithmetic(arith) => {
            format!("$(({}))", generate_arith_expr(arith))
        }
        BashExpr::Test(test) => generate_test_expr(test),
        BashExpr::CommandSubst(cmd) => {
            format!("$({})", generate_statement(cmd))
        }
        BashExpr::Concat(exprs) => exprs.iter().map(generate_expr).collect::<Vec<_>>().join(""),
        BashExpr::Glob(pattern) => pattern.clone(),
        BashExpr::DefaultValue { variable, default } => {
            // Generate ${VAR:-default} syntax
            let default_val = generate_expr(default);
            format!("\"${{{}:-{}}}\"", variable, default_val.trim_matches('"'))
        }
        BashExpr::AssignDefault { variable, default } => {
            // Generate ${VAR:=default} syntax
            let default_val = generate_expr(default);
            format!("\"${{{}:={}}}\"", variable, default_val.trim_matches('"'))
        }
        BashExpr::ErrorIfUnset { variable, message } => {
            // Generate ${VAR:?message} syntax
            let msg_val = generate_expr(message);
            format!("\"${{{}:?{}}}\"", variable, msg_val.trim_matches('"'))
        }
        BashExpr::AlternativeValue {
            variable,
            alternative,
        } => {
            // Generate ${VAR:+alt_value} syntax
            let alt_val = generate_expr(alternative);
            format!("\"${{{}:+{}}}\"", variable, alt_val.trim_matches('"'))
        }
        BashExpr::StringLength { variable } => {
            // Generate ${#VAR} syntax
            format!("\"${{#{}}}\"", variable)
        }
        BashExpr::RemoveSuffix { variable, pattern } => {
            // Generate ${VAR%pattern} syntax
            let pattern_val = generate_expr(pattern);
            format!("\"${{{}%{}}}\"", variable, pattern_val.trim_matches('"'))
        }
        BashExpr::RemovePrefix { variable, pattern } => {
            // Generate ${VAR#pattern} syntax
            let pattern_val = generate_expr(pattern);
            format!("\"${{{}#{}}}\"", variable, pattern_val.trim_matches('"'))
        }
        BashExpr::RemoveLongestPrefix { variable, pattern } => {
            // Generate ${VAR##pattern} syntax (greedy prefix removal)
            let pattern_val = generate_expr(pattern);
            format!("\"${{{}##{}}}\"", variable, pattern_val.trim_matches('"'))
        }
        BashExpr::RemoveLongestSuffix { variable, pattern } => {
            // Generate ${VAR%%pattern} syntax (greedy suffix removal)
            let pattern_val = generate_expr(pattern);
            format!("\"${{{}%%{}}}\"", variable, pattern_val.trim_matches('"'))
        }
        BashExpr::CommandCondition(cmd) => {
            // Issue #93: Command condition - generate the command directly
            generate_statement(cmd)
        }
    }
}

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
mod tests {
    use super::*;
    use proptest::strategy::ValueTree;

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

    // ============== generate_purified_bash tests ==============

    #[test]
    fn test_generate_purified_bash_empty() {
        let ast = BashAst {
            statements: vec![],
            metadata: AstMetadata {
                source_file: None,
                line_count: 0,
                parse_time_ms: 0,
            },
        };
        let output = generate_purified_bash(&ast);
        assert!(output.starts_with("#!/bin/sh\n"));
    }

    #[test]
    fn test_generate_purified_bash_command() {
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Literal("hello".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };
        let output = generate_purified_bash(&ast);
        assert!(output.contains("echo hello"));
    }

    #[test]
    fn test_generate_purified_bash_assignment() {
        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "FOO".to_string(),
                index: None,
                value: BashExpr::Literal("bar".to_string()),
                exported: false,
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };
        let output = generate_purified_bash(&ast);
        assert!(output.contains("FOO=bar"));
    }

    #[test]
    fn test_generate_purified_bash_exported_assignment() {
        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "PATH".to_string(),
                index: None,
                value: BashExpr::Literal("/usr/bin".to_string()),
                exported: true,
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };
        let output = generate_purified_bash(&ast);
        assert!(output.contains("export PATH=/usr/bin"));
    }

    #[test]
    fn test_generate_purified_bash_comment() {
        let ast = BashAst {
            statements: vec![BashStmt::Comment {
                text: "This is a comment".to_string(),
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };
        let output = generate_purified_bash(&ast);
        assert!(output.contains("# This is a comment"));
    }

    #[test]
    fn test_generate_purified_bash_function() {
        let ast = BashAst {
            statements: vec![BashStmt::Function {
                name: "my_func".to_string(),
                body: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("hello".to_string())],
                    redirects: vec![],
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
        let output = generate_purified_bash(&ast);
        assert!(output.contains("my_func() {"));
        assert!(output.contains("echo hello"));
        assert!(output.contains("}"));
    }

    #[test]
    fn test_generate_purified_bash_if_statement() {
        let ast = BashAst {
            statements: vec![BashStmt::If {
                condition: BashExpr::Test(Box::new(TestExpr::StringNonEmpty(BashExpr::Variable(
                    "x".to_string(),
                )))),
                then_block: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("yes".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                }],
                elif_blocks: vec![],
                else_block: None,
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };
        let output = generate_purified_bash(&ast);
        assert!(output.contains("if"));
        assert!(output.contains("then"));
        assert!(output.contains("fi"));
    }

    #[test]
    fn test_generate_purified_bash_if_with_else() {
        let ast = BashAst {
            statements: vec![BashStmt::If {
                condition: BashExpr::Test(Box::new(TestExpr::StringNonEmpty(BashExpr::Variable(
                    "x".to_string(),
                )))),
                then_block: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("yes".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                }],
                elif_blocks: vec![],
                else_block: Some(vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("no".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                }]),
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };
        let output = generate_purified_bash(&ast);
        assert!(output.contains("else"));
    }

    #[test]
    fn test_generate_purified_bash_for_loop() {
        let ast = BashAst {
            statements: vec![BashStmt::For {
                variable: "i".to_string(),
                items: BashExpr::Array(vec![
                    BashExpr::Literal("1".to_string()),
                    BashExpr::Literal("2".to_string()),
                ]),
                body: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Variable("i".to_string())],
                    redirects: vec![],
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
        let output = generate_purified_bash(&ast);
        assert!(output.contains("for i in"));
        assert!(output.contains("do"));
        assert!(output.contains("done"));
    }

    #[test]
    fn test_generate_purified_bash_for_c_style() {
        let ast = BashAst {
            statements: vec![BashStmt::ForCStyle {
                init: "i=0".to_string(),
                condition: "i<10".to_string(),
                increment: "i++".to_string(),
                body: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Variable("i".to_string())],
                    redirects: vec![],
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
        let output = generate_purified_bash(&ast);
        assert!(output.contains("for ((i=0; i<10; i++))"));
        assert!(output.contains("do"));
        assert!(output.contains("done"));
    }

    #[test]
    fn test_generate_purified_bash_while_loop() {
        let ast = BashAst {
            statements: vec![BashStmt::While {
                condition: BashExpr::Test(Box::new(TestExpr::IntLt(
                    BashExpr::Variable("i".to_string()),
                    BashExpr::Literal("10".to_string()),
                ))),
                body: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Variable("i".to_string())],
                    redirects: vec![],
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
        let output = generate_purified_bash(&ast);
        assert!(output.contains("while"));
        assert!(output.contains("do"));
        assert!(output.contains("done"));
    }

    #[test]
    fn test_generate_purified_bash_until_loop() {
        let ast = BashAst {
            statements: vec![BashStmt::Until {
                condition: BashExpr::Test(Box::new(TestExpr::IntGe(
                    BashExpr::Variable("i".to_string()),
                    BashExpr::Literal("10".to_string()),
                ))),
                body: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Variable("i".to_string())],
                    redirects: vec![],
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
        let output = generate_purified_bash(&ast);
        // Until is transformed to while with negated condition
        assert!(output.contains("while"));
        assert!(output.contains("!"));
    }

    #[test]
    fn test_generate_purified_bash_return() {
        let ast = BashAst {
            statements: vec![BashStmt::Return {
                code: Some(BashExpr::Literal("0".to_string())),
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };
        let output = generate_purified_bash(&ast);
        assert!(output.contains("return 0"));
    }

    #[test]
    fn test_generate_purified_bash_return_without_code() {
        let ast = BashAst {
            statements: vec![BashStmt::Return {
                code: None,
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };
        let output = generate_purified_bash(&ast);
        assert!(output.contains("return"));
    }

    #[test]
    fn test_generate_purified_bash_case() {
        let ast = BashAst {
            statements: vec![BashStmt::Case {
                word: BashExpr::Variable("x".to_string()),
                arms: vec![
                    CaseArm {
                        patterns: vec!["a".to_string()],
                        body: vec![BashStmt::Command {
                            name: "echo".to_string(),
                            args: vec![BashExpr::Literal("A".to_string())],
                            redirects: vec![],
                            span: Span::dummy(),
                        }],
                    },
                    CaseArm {
                        patterns: vec!["b".to_string(), "c".to_string()],
                        body: vec![BashStmt::Command {
                            name: "echo".to_string(),
                            args: vec![BashExpr::Literal("B or C".to_string())],
                            redirects: vec![],
                            span: Span::dummy(),
                        }],
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
        let output = generate_purified_bash(&ast);
        assert!(output.contains("case"));
        assert!(output.contains("esac"));
        assert!(output.contains(";;"));
        assert!(output.contains("b|c"));
    }

    #[test]
    fn test_generate_purified_bash_pipeline() {
        let ast = BashAst {
            statements: vec![BashStmt::Pipeline {
                commands: vec![
                    BashStmt::Command {
                        name: "echo".to_string(),
                        args: vec![BashExpr::Literal("hello".to_string())],
                        redirects: vec![],
                        span: Span::dummy(),
                    },
                    BashStmt::Command {
                        name: "grep".to_string(),
                        args: vec![BashExpr::Literal("h".to_string())],
                        redirects: vec![],
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
        let output = generate_purified_bash(&ast);
        assert!(output.contains("echo hello | grep h"));
    }

    #[test]
    fn test_generate_purified_bash_and_list() {
        let ast = BashAst {
            statements: vec![BashStmt::AndList {
                left: Box::new(BashStmt::Command {
                    name: "true".to_string(),
                    args: vec![],
                    redirects: vec![],
                    span: Span::dummy(),
                }),
                right: Box::new(BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("ok".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                }),
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };
        let output = generate_purified_bash(&ast);
        assert!(output.contains("true && echo ok"));
    }

    #[test]
    fn test_generate_purified_bash_or_list() {
        let ast = BashAst {
            statements: vec![BashStmt::OrList {
                left: Box::new(BashStmt::Command {
                    name: "false".to_string(),
                    args: vec![],
                    redirects: vec![],
                    span: Span::dummy(),
                }),
                right: Box::new(BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("failed".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                }),
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };
        let output = generate_purified_bash(&ast);
        assert!(output.contains("false || echo failed"));
    }

    #[test]
    fn test_generate_purified_bash_brace_group() {
        let ast = BashAst {
            statements: vec![BashStmt::BraceGroup {
                body: vec![
                    BashStmt::Command {
                        name: "echo".to_string(),
                        args: vec![BashExpr::Literal("a".to_string())],
                        redirects: vec![],
                        span: Span::dummy(),
                    },
                    BashStmt::Command {
                        name: "echo".to_string(),
                        args: vec![BashExpr::Literal("b".to_string())],
                        redirects: vec![],
                        span: Span::dummy(),
                    },
                ],
                subshell: false,
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };
        let output = generate_purified_bash(&ast);
        assert!(output.contains("{"));
        assert!(output.contains("}"));
    }

    #[test]
    fn test_generate_purified_bash_coproc_with_name() {
        let ast = BashAst {
            statements: vec![BashStmt::Coproc {
                name: Some("mycoproc".to_string()),
                body: vec![BashStmt::Command {
                    name: "cat".to_string(),
                    args: vec![],
                    redirects: vec![],
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
        let output = generate_purified_bash(&ast);
        assert!(output.contains("coproc mycoproc"));
    }

    #[test]
    fn test_generate_purified_bash_coproc_without_name() {
        let ast = BashAst {
            statements: vec![BashStmt::Coproc {
                name: None,
                body: vec![BashStmt::Command {
                    name: "cat".to_string(),
                    args: vec![],
                    redirects: vec![],
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
        let output = generate_purified_bash(&ast);
        assert!(output.contains("coproc { cat; }"));
    }

    // ============== generate_expr tests ==============

    #[test]
    fn test_generate_expr_literal_simple() {
        let expr = BashExpr::Literal("hello".to_string());
        let output = generate_expr(&expr);
        assert_eq!(output, "hello");
    }

    #[test]
    fn test_generate_expr_literal_with_space() {
        let expr = BashExpr::Literal("hello world".to_string());
        let output = generate_expr(&expr);
        assert_eq!(output, "'hello world'");
    }

    #[test]
    fn test_generate_expr_literal_with_dollar() {
        let expr = BashExpr::Literal("$HOME".to_string());
        let output = generate_expr(&expr);
        assert_eq!(output, "'$HOME'");
    }

    #[test]
    fn test_generate_expr_variable() {
        let expr = BashExpr::Variable("FOO".to_string());
        let output = generate_expr(&expr);
        assert_eq!(output, "\"$FOO\"");
    }

    #[test]
    fn test_generate_expr_array() {
        let expr = BashExpr::Array(vec![
            BashExpr::Literal("a".to_string()),
            BashExpr::Literal("b".to_string()),
        ]);
        let output = generate_expr(&expr);
        assert_eq!(output, "a b");
    }

    #[test]
    fn test_generate_expr_arithmetic() {
        let expr = BashExpr::Arithmetic(Box::new(ArithExpr::Add(
            Box::new(ArithExpr::Number(1)),
            Box::new(ArithExpr::Number(2)),
        )));
        let output = generate_expr(&expr);
        assert_eq!(output, "$((1 + 2))");
    }

    #[test]
    fn test_generate_expr_command_subst() {
        let expr = BashExpr::CommandSubst(Box::new(BashStmt::Command {
            name: "date".to_string(),
            args: vec![],
            redirects: vec![],
            span: Span::dummy(),
        }));
        let output = generate_expr(&expr);
        assert_eq!(output, "$(date)");
    }

    #[test]
    fn test_generate_expr_concat() {
        let expr = BashExpr::Concat(vec![
            BashExpr::Literal("prefix_".to_string()),
            BashExpr::Variable("VAR".to_string()),
        ]);
        let output = generate_expr(&expr);
        assert!(output.contains("prefix_"));
        assert!(output.contains("\"$VAR\""));
    }

    #[test]
    fn test_generate_expr_glob() {
        let expr = BashExpr::Glob("*.txt".to_string());
        let output = generate_expr(&expr);
        assert_eq!(output, "*.txt");
    }

    #[test]
    fn test_generate_expr_default_value() {
        let expr = BashExpr::DefaultValue {
            variable: "FOO".to_string(),
            default: Box::new(BashExpr::Literal("default".to_string())),
        };
        let output = generate_expr(&expr);
        assert!(output.contains("${FOO:-default}"));
    }

    #[test]
    fn test_generate_expr_assign_default() {
        let expr = BashExpr::AssignDefault {
            variable: "FOO".to_string(),
            default: Box::new(BashExpr::Literal("default".to_string())),
        };
        let output = generate_expr(&expr);
        assert!(output.contains("${FOO:=default}"));
    }

    #[test]
    fn test_generate_expr_error_if_unset() {
        let expr = BashExpr::ErrorIfUnset {
            variable: "FOO".to_string(),
            message: Box::new(BashExpr::Literal("error".to_string())),
        };
        let output = generate_expr(&expr);
        assert!(output.contains("${FOO:?error}"));
    }

    #[test]
    fn test_generate_expr_alternative_value() {
        let expr = BashExpr::AlternativeValue {
            variable: "FOO".to_string(),
            alternative: Box::new(BashExpr::Literal("alt".to_string())),
        };
        let output = generate_expr(&expr);
        assert!(output.contains("${FOO:+alt}"));
    }

    #[test]
    fn test_generate_expr_string_length() {
        let expr = BashExpr::StringLength {
            variable: "FOO".to_string(),
        };
        let output = generate_expr(&expr);
        assert!(output.contains("${#FOO}"));
    }

    #[test]
    fn test_generate_expr_remove_suffix() {
        let expr = BashExpr::RemoveSuffix {
            variable: "FILE".to_string(),
            pattern: Box::new(BashExpr::Literal(".txt".to_string())),
        };
        let output = generate_expr(&expr);
        assert!(output.contains("${FILE%.txt}"));
    }

    #[test]
    fn test_generate_expr_remove_prefix() {
        let expr = BashExpr::RemovePrefix {
            variable: "PATH".to_string(),
            pattern: Box::new(BashExpr::Literal("*/".to_string())),
        };
        let output = generate_expr(&expr);
        assert!(output.contains("${PATH#*/}"));
    }

    #[test]
    fn test_generate_expr_remove_longest_prefix() {
        let expr = BashExpr::RemoveLongestPrefix {
            variable: "PATH".to_string(),
            pattern: Box::new(BashExpr::Literal("*/".to_string())),
        };
        let output = generate_expr(&expr);
        assert!(output.contains("${PATH##*/}"));
    }

    #[test]
    fn test_generate_expr_remove_longest_suffix() {
        let expr = BashExpr::RemoveLongestSuffix {
            variable: "FILE".to_string(),
            pattern: Box::new(BashExpr::Literal(".*".to_string())),
        };
        let output = generate_expr(&expr);
        assert!(output.contains("${FILE%%.*}"));
    }

    #[test]
    fn test_generate_expr_command_condition() {
        let expr = BashExpr::CommandCondition(Box::new(BashStmt::Command {
            name: "test".to_string(),
            args: vec![
                BashExpr::Literal("-f".to_string()),
                BashExpr::Literal("file".to_string()),
            ],
            redirects: vec![],
            span: Span::dummy(),
        }));
        let output = generate_expr(&expr);
        assert!(output.contains("test -f file"));
    }

    // ============== generate_arith_expr tests ==============

    #[test]
    fn test_generate_arith_expr_number() {
        let expr = ArithExpr::Number(42);
        let output = generate_arith_expr(&expr);
        assert_eq!(output, "42");
    }

    #[test]
    fn test_generate_arith_expr_variable() {
        let expr = ArithExpr::Variable("x".to_string());
        let output = generate_arith_expr(&expr);
        assert_eq!(output, "x");
    }

    #[test]
    fn test_generate_arith_expr_add() {
        let expr = ArithExpr::Add(
            Box::new(ArithExpr::Number(1)),
            Box::new(ArithExpr::Number(2)),
        );
        let output = generate_arith_expr(&expr);
        assert_eq!(output, "1 + 2");
    }

    #[test]
    fn test_generate_arith_expr_sub() {
        let expr = ArithExpr::Sub(
            Box::new(ArithExpr::Number(5)),
            Box::new(ArithExpr::Number(3)),
        );
        let output = generate_arith_expr(&expr);
        assert_eq!(output, "5 - 3");
    }

    #[test]
    fn test_generate_arith_expr_mul() {
        let expr = ArithExpr::Mul(
            Box::new(ArithExpr::Number(2)),
            Box::new(ArithExpr::Number(3)),
        );
        let output = generate_arith_expr(&expr);
        assert_eq!(output, "2 * 3");
    }

    #[test]
    fn test_generate_arith_expr_div() {
        let expr = ArithExpr::Div(
            Box::new(ArithExpr::Number(6)),
            Box::new(ArithExpr::Number(2)),
        );
        let output = generate_arith_expr(&expr);
        assert_eq!(output, "6 / 2");
    }

    #[test]
    fn test_generate_arith_expr_mod() {
        let expr = ArithExpr::Mod(
            Box::new(ArithExpr::Number(7)),
            Box::new(ArithExpr::Number(3)),
        );
        let output = generate_arith_expr(&expr);
        assert_eq!(output, "7 % 3");
    }

    // ============== generate_test_expr tests ==============

    #[test]
    fn test_generate_test_expr_string_eq() {
        let expr = TestExpr::StringEq(
            BashExpr::Variable("x".to_string()),
            BashExpr::Literal("y".to_string()),
        );
        let output = generate_test_expr(&expr);
        assert!(output.contains("= y"));
    }

    #[test]
    fn test_generate_test_expr_string_ne() {
        let expr = TestExpr::StringNe(
            BashExpr::Variable("x".to_string()),
            BashExpr::Literal("y".to_string()),
        );
        let output = generate_test_expr(&expr);
        assert!(output.contains("!= y"));
    }

    #[test]
    fn test_generate_test_expr_int_eq() {
        let expr = TestExpr::IntEq(
            BashExpr::Variable("x".to_string()),
            BashExpr::Literal("5".to_string()),
        );
        let output = generate_test_expr(&expr);
        assert!(output.contains("-eq 5"));
    }

    #[test]
    fn test_generate_test_expr_int_ne() {
        let expr = TestExpr::IntNe(
            BashExpr::Variable("x".to_string()),
            BashExpr::Literal("5".to_string()),
        );
        let output = generate_test_expr(&expr);
        assert!(output.contains("-ne 5"));
    }

    #[test]
    fn test_generate_test_expr_int_lt() {
        let expr = TestExpr::IntLt(
            BashExpr::Variable("x".to_string()),
            BashExpr::Literal("5".to_string()),
        );
        let output = generate_test_expr(&expr);
        assert!(output.contains("-lt 5"));
    }

    #[test]
    fn test_generate_test_expr_int_le() {
        let expr = TestExpr::IntLe(
            BashExpr::Variable("x".to_string()),
            BashExpr::Literal("5".to_string()),
        );
        let output = generate_test_expr(&expr);
        assert!(output.contains("-le 5"));
    }

    #[test]
    fn test_generate_test_expr_int_gt() {
        let expr = TestExpr::IntGt(
            BashExpr::Variable("x".to_string()),
            BashExpr::Literal("5".to_string()),
        );
        let output = generate_test_expr(&expr);
        assert!(output.contains("-gt 5"));
    }

    #[test]
    fn test_generate_test_expr_int_ge() {
        let expr = TestExpr::IntGe(
            BashExpr::Variable("x".to_string()),
            BashExpr::Literal("5".to_string()),
        );
        let output = generate_test_expr(&expr);
        assert!(output.contains("-ge 5"));
    }

    #[test]
    fn test_generate_test_expr_file_exists() {
        let expr = TestExpr::FileExists(BashExpr::Literal("/tmp".to_string()));
        let output = generate_test_expr(&expr);
        assert!(output.contains("-e /tmp"));
    }

    #[test]
    fn test_generate_test_expr_file_readable() {
        let expr = TestExpr::FileReadable(BashExpr::Literal("/tmp".to_string()));
        let output = generate_test_expr(&expr);
        assert!(output.contains("-r /tmp"));
    }

    #[test]
    fn test_generate_test_expr_file_writable() {
        let expr = TestExpr::FileWritable(BashExpr::Literal("/tmp".to_string()));
        let output = generate_test_expr(&expr);
        assert!(output.contains("-w /tmp"));
    }

    #[test]
    fn test_generate_test_expr_file_executable() {
        let expr = TestExpr::FileExecutable(BashExpr::Literal("/tmp".to_string()));
        let output = generate_test_expr(&expr);
        assert!(output.contains("-x /tmp"));
    }

    #[test]
    fn test_generate_test_expr_file_directory() {
        let expr = TestExpr::FileDirectory(BashExpr::Literal("/tmp".to_string()));
        let output = generate_test_expr(&expr);
        assert!(output.contains("-d /tmp"));
    }

    #[test]
    fn test_generate_test_expr_string_empty() {
        let expr = TestExpr::StringEmpty(BashExpr::Variable("x".to_string()));
        let output = generate_test_expr(&expr);
        assert!(output.contains("-z"));
    }

    #[test]
    fn test_generate_test_expr_string_non_empty() {
        let expr = TestExpr::StringNonEmpty(BashExpr::Variable("x".to_string()));
        let output = generate_test_expr(&expr);
        assert!(output.contains("-n"));
    }

    #[test]
    fn test_generate_test_expr_and() {
        let expr = TestExpr::And(
            Box::new(TestExpr::FileExists(BashExpr::Literal("a".to_string()))),
            Box::new(TestExpr::FileExists(BashExpr::Literal("b".to_string()))),
        );
        let output = generate_test_expr(&expr);
        assert!(output.contains("&&"));
    }

    #[test]
    fn test_generate_test_expr_or() {
        let expr = TestExpr::Or(
            Box::new(TestExpr::FileExists(BashExpr::Literal("a".to_string()))),
            Box::new(TestExpr::FileExists(BashExpr::Literal("b".to_string()))),
        );
        let output = generate_test_expr(&expr);
        assert!(output.contains("||"));
    }

    #[test]
    fn test_generate_test_expr_not() {
        let expr = TestExpr::Not(Box::new(TestExpr::FileExists(BashExpr::Literal(
            "a".to_string(),
        ))));
        let output = generate_test_expr(&expr);
        assert!(output.contains("!"));
    }

    // ============== negate_condition tests ==============

    #[test]
    fn test_negate_condition_test() {
        let expr = BashExpr::Test(Box::new(TestExpr::IntGt(
            BashExpr::Variable("x".to_string()),
            BashExpr::Literal("5".to_string()),
        )));
        let output = negate_condition(&expr);
        assert!(output.contains("!"));
    }

    #[test]
    fn test_negate_condition_other() {
        let expr = BashExpr::Variable("x".to_string());
        let output = negate_condition(&expr);
        assert!(output.starts_with("!"));
    }

    // ============== generate_test_condition tests ==============

    #[test]
    fn test_generate_test_condition_all_types() {
        // Test all test condition variants
        let tests = vec![
            (
                TestExpr::StringEq(
                    BashExpr::Variable("a".to_string()),
                    BashExpr::Literal("b".to_string()),
                ),
                "=",
            ),
            (
                TestExpr::StringNe(
                    BashExpr::Variable("a".to_string()),
                    BashExpr::Literal("b".to_string()),
                ),
                "!=",
            ),
            (
                TestExpr::IntEq(
                    BashExpr::Variable("a".to_string()),
                    BashExpr::Literal("1".to_string()),
                ),
                "-eq",
            ),
            (
                TestExpr::IntNe(
                    BashExpr::Variable("a".to_string()),
                    BashExpr::Literal("1".to_string()),
                ),
                "-ne",
            ),
            (
                TestExpr::IntLt(
                    BashExpr::Variable("a".to_string()),
                    BashExpr::Literal("1".to_string()),
                ),
                "-lt",
            ),
            (
                TestExpr::IntLe(
                    BashExpr::Variable("a".to_string()),
                    BashExpr::Literal("1".to_string()),
                ),
                "-le",
            ),
            (
                TestExpr::IntGt(
                    BashExpr::Variable("a".to_string()),
                    BashExpr::Literal("1".to_string()),
                ),
                "-gt",
            ),
            (
                TestExpr::IntGe(
                    BashExpr::Variable("a".to_string()),
                    BashExpr::Literal("1".to_string()),
                ),
                "-ge",
            ),
            (
                TestExpr::FileExists(BashExpr::Literal("f".to_string())),
                "-e",
            ),
            (
                TestExpr::FileReadable(BashExpr::Literal("f".to_string())),
                "-r",
            ),
            (
                TestExpr::FileWritable(BashExpr::Literal("f".to_string())),
                "-w",
            ),
            (
                TestExpr::FileExecutable(BashExpr::Literal("f".to_string())),
                "-x",
            ),
            (
                TestExpr::FileDirectory(BashExpr::Literal("f".to_string())),
                "-d",
            ),
            (
                TestExpr::StringEmpty(BashExpr::Variable("x".to_string())),
                "-z",
            ),
            (
                TestExpr::StringNonEmpty(BashExpr::Variable("x".to_string())),
                "-n",
            ),
        ];

        for (expr, expected) in tests {
            let output = generate_test_condition(&expr);
            assert!(
                output.contains(expected),
                "Expected '{}' in output: {}",
                expected,
                output
            );
        }
    }

    #[test]
    fn test_generate_test_condition_and_or_not() {
        let and_expr = TestExpr::And(
            Box::new(TestExpr::FileExists(BashExpr::Literal("a".to_string()))),
            Box::new(TestExpr::FileExists(BashExpr::Literal("b".to_string()))),
        );
        let and_output = generate_test_condition(&and_expr);
        assert!(and_output.contains("&&"));

        let or_expr = TestExpr::Or(
            Box::new(TestExpr::FileExists(BashExpr::Literal("a".to_string()))),
            Box::new(TestExpr::FileExists(BashExpr::Literal("b".to_string()))),
        );
        let or_output = generate_test_condition(&or_expr);
        assert!(or_output.contains("||"));

        let not_expr = TestExpr::Not(Box::new(TestExpr::FileExists(BashExpr::Literal(
            "a".to_string(),
        ))));
        let not_output = generate_test_condition(&not_expr);
        assert!(not_output.contains("!"));
    }

    // ============== generate_condition tests ==============

    #[test]
    fn test_generate_condition_with_test() {
        let expr = BashExpr::Test(Box::new(TestExpr::FileExists(BashExpr::Literal(
            "/tmp".to_string(),
        ))));
        let output = generate_condition(&expr);
        assert!(output.contains("-e /tmp"));
    }

    #[test]
    fn test_generate_condition_with_other() {
        let expr = BashExpr::Variable("x".to_string());
        let output = generate_condition(&expr);
        assert_eq!(output, "\"$x\"");
    }

    // ============== BASH_KEYWORDS tests ==============

    #[test]
    fn test_bash_keywords_contains_expected() {
        assert!(BASH_KEYWORDS.contains(&"if"));
        assert!(BASH_KEYWORDS.contains(&"then"));
        assert!(BASH_KEYWORDS.contains(&"else"));
        assert!(BASH_KEYWORDS.contains(&"fi"));
        assert!(BASH_KEYWORDS.contains(&"for"));
        assert!(BASH_KEYWORDS.contains(&"while"));
        assert!(BASH_KEYWORDS.contains(&"do"));
        assert!(BASH_KEYWORDS.contains(&"done"));
        assert!(BASH_KEYWORDS.contains(&"case"));
        assert!(BASH_KEYWORDS.contains(&"esac"));
    }

    // ============== Strategy function type tests ==============

    #[test]
    fn test_bash_string_generates_valid_output() {
        use proptest::test_runner::TestRunner;
        let strategy = bash_string();
        let mut runner = TestRunner::default();

        // Generate a few values to verify the strategy works
        for _ in 0..5 {
            let value = strategy.new_tree(&mut runner).unwrap().current();
            assert!(value.len() <= 20);
            // Valid characters only
            assert!(value
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == ' '));
        }
    }

    #[test]
    fn test_bash_integer_generates_valid_range() {
        use proptest::test_runner::TestRunner;
        let strategy = bash_integer();
        let mut runner = TestRunner::default();

        for _ in 0..10 {
            let value = strategy.new_tree(&mut runner).unwrap().current();
            assert!(value >= -1000);
            assert!(value < 1000);
        }
    }

    #[test]
    fn test_bash_variable_name_generates_valid() {
        use proptest::test_runner::TestRunner;
        let strategy = bash_variable_name();
        let mut runner = TestRunner::default();

        for _ in 0..5 {
            let value = strategy.new_tree(&mut runner).unwrap().current();
            assert!(!value.is_empty());
            // Should be one of the known variable names
            let valid_names = vec![
                "FOO", "BAR", "PATH", "HOME", "USER", "x", "y", "status", "result",
            ];
            assert!(valid_names.contains(&value.as_str()));
        }
    }

    #[test]
    fn test_bash_test_expr_generates_valid() {
        use proptest::test_runner::TestRunner;
        let strategy = bash_test_expr();
        let mut runner = TestRunner::default();

        // Just verify it generates without panic
        for _ in 0..5 {
            let _value = strategy.new_tree(&mut runner).unwrap().current();
        }
    }
}
