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
include!("generators_generate_generate_arith_.rs");
