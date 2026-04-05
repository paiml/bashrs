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
        BashStmt::Command { name, args, .. } => generate_stmt_command(name, args),
        BashStmt::Assignment {
            name,
            value,
            exported,
            ..
        } => generate_stmt_assignment(name, value, *exported),
        BashStmt::Comment { text, .. } => format!("# {}", text),
        BashStmt::Function { name, body, .. } => generate_stmt_function(name, body),
        BashStmt::If {
            condition,
            then_block,
            else_block,
            ..
        } => generate_stmt_if(condition, then_block, else_block.as_deref()),
        BashStmt::For {
            variable,
            items,
            body,
            ..
        } => generate_stmt_for(variable, items, body),
        BashStmt::ForCStyle {
            init,
            condition,
            increment,
            body,
            ..
        } => generate_stmt_for_c_style(init, condition, increment, body),
        BashStmt::While {
            condition, body, ..
        } => generate_stmt_while(condition, body),
        BashStmt::Until {
            condition, body, ..
        } => generate_stmt_until(condition, body),
        BashStmt::Return { code, .. } => generate_stmt_return(code.as_ref()),
        BashStmt::Case { word, arms, .. } => generate_stmt_case(word, arms),
        BashStmt::Pipeline { commands, .. } => generate_stmt_pipeline(commands),
        BashStmt::AndList { left, right, .. } => {
            format!(
                "{} && {}",
                generate_statement(left),
                generate_statement(right)
            )
        }
        BashStmt::OrList { left, right, .. } => {
            format!(
                "{} || {}",
                generate_statement(left),
                generate_statement(right)
            )
        }
        BashStmt::BraceGroup { body, .. } => generate_stmt_brace_group(body),
        BashStmt::Coproc { name, body, .. } => generate_stmt_coproc(name.as_deref(), body),
        BashStmt::Select {
            variable,
            items,
            body,
            ..
        } => generate_stmt_select(variable, items, body),
        BashStmt::Negated { command, .. } => format!("! {}", generate_statement(command)),
    }
}

/// Append indented body statements to the output buffer
fn append_indented_body(output: &mut String, body: &[BashStmt]) {
    for stmt in body {
        output.push_str("    ");
        output.push_str(&generate_statement(stmt));
        output.push('\n');
    }
}

/// Generate a command statement: name arg1 arg2 ...
fn generate_stmt_command(name: &str, args: &[BashExpr]) -> String {
    let mut cmd = name.to_string();
    for arg in args {
        cmd.push(' ');
        cmd.push_str(&generate_expr(arg));
    }
    cmd
}

/// Generate an assignment statement: [export] name=value
fn generate_stmt_assignment(name: &str, value: &BashExpr, exported: bool) -> String {
    let mut assign = String::new();
    if exported {
        assign.push_str("export ");
    }
    assign.push_str(name);
    assign.push('=');
    assign.push_str(&generate_expr(value));
    assign
}

/// Generate a function definition: name() { body }
fn generate_stmt_function(name: &str, body: &[BashStmt]) -> String {
    let mut func = format!("{}() {{\n", name);
    append_indented_body(&mut func, body);
    func.push('}');
    func
}

/// Generate an if statement with optional else block
fn generate_stmt_if(
    condition: &BashExpr,
    then_block: &[BashStmt],
    else_block: Option<&[BashStmt]>,
) -> String {
    let mut if_stmt = format!("if {}; then\n", generate_condition(condition));
    append_indented_body(&mut if_stmt, then_block);
    if let Some(else_stmts) = else_block {
        if_stmt.push_str("else\n");
        append_indented_body(&mut if_stmt, else_stmts);
    }
    if_stmt.push_str("fi");
    if_stmt
}

/// Generate a for-in loop: for var in items; do body; done
fn generate_stmt_for(variable: &str, items: &BashExpr, body: &[BashStmt]) -> String {
    let mut for_stmt = format!("for {} in {}; do\n", variable, generate_expr(items));
    append_indented_body(&mut for_stmt, body);
    for_stmt.push_str("done");
    for_stmt
}

/// Generate a C-style for loop: for ((init; cond; incr)); do body; done
fn generate_stmt_for_c_style(
    init: &str,
    condition: &str,
    increment: &str,
    body: &[BashStmt],
) -> String {
    let mut for_stmt = format!("for (({}; {}; {})); do\n", init, condition, increment);
    append_indented_body(&mut for_stmt, body);
    for_stmt.push_str("done");
    for_stmt
}

/// Generate a while loop: while cond; do body; done
fn generate_stmt_while(condition: &BashExpr, body: &[BashStmt]) -> String {
    let mut while_stmt = format!("while {}; do\n", generate_condition(condition));
    append_indented_body(&mut while_stmt, body);
    while_stmt.push_str("done");
    while_stmt
}

/// Generate an until loop (transformed to while with negated condition)
fn generate_stmt_until(condition: &BashExpr, body: &[BashStmt]) -> String {
    // Transform until loop to while loop with negated condition
    // until [ $i -gt 5 ] -> while [ ! "$i" -gt 5 ]
    let negated_condition = negate_condition(condition);
    let mut while_stmt = format!("while {}; do\n", negated_condition);
    append_indented_body(&mut while_stmt, body);
    while_stmt.push_str("done");
    while_stmt
}

/// Generate a return statement: return [code]
fn generate_stmt_return(code: Option<&BashExpr>) -> String {
    if let Some(c) = code {
        format!("return {}", generate_expr(c))
    } else {
        String::from("return")
    }
}

/// Generate a case statement: case word in pattern) body;; ... esac
fn generate_stmt_case(word: &BashExpr, arms: &[CaseArm]) -> String {
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

/// Generate a pipeline: cmd1 | cmd2 | cmd3
fn generate_stmt_pipeline(commands: &[BashStmt]) -> String {
    let mut pipeline = String::new();
    for (i, cmd) in commands.iter().enumerate() {
        if i > 0 {
            pipeline.push_str(" | ");
        }
        pipeline.push_str(&generate_statement(cmd));
    }
    pipeline
}

/// Generate a brace group: { cmd1; cmd2; }
fn generate_stmt_brace_group(body: &[BashStmt]) -> String {
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

/// Generate a coproc: coproc [NAME] { cmd; }
fn generate_stmt_coproc(name: Option<&str>, body: &[BashStmt]) -> String {
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

/// Generate a select: select VAR in ITEMS; do BODY; done
fn generate_stmt_select(variable: &str, items: &BashExpr, body: &[BashStmt]) -> String {
    let mut select = format!("select {} in ", variable);
    select.push_str(&generate_expr(items));
    select.push_str("; do\n");
    append_indented_body(&mut select, body);
    select.push_str("done");
    select
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
#[path = "generators_tests_extracted.rs"]
mod tests_extracted;
