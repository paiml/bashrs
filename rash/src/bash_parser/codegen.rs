//! Bash Code Generation
//!
//! Generates purified bash scripts from BashAst.
//! Used by the `bashrs purify` command to emit safe, deterministic bash.

use super::ast::*;

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
            // Skip shebang comments to maintain idempotency
            // Shebangs look like "!/bin/bash" or "!/bin/sh" when parsed as comments
            if text.starts_with("!/bin/") || text.starts_with(" !/bin/") {
                return String::new();
            }
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
