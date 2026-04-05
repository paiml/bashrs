fn generate_function_stmt(pad: &str, name: &str, body: &[BashStmt], indent: usize) -> String {
    let mut func = format!("{}{}() {{\n", pad, name);
    for stmt in body {
        func.push_str(&generate_stmt(stmt, indent + 1));
        func.push('\n');
    }
    func.push_str(pad);
    func.push('}');
    func
}

/// Generate a loop body with header and "done" terminator
fn generate_loop_body(header: &str, pad: &str, body: &[BashStmt], indent: usize) -> String {
    let mut s = format!("{}\n", header);
    for stmt in body {
        s.push_str(&generate_stmt(stmt, indent + 1));
        s.push('\n');
    }
    s.push_str(pad);
    s.push_str("done");
    s
}

/// Generate a pipeline
fn generate_pipeline(pad: &str, commands: &[BashStmt]) -> String {
    let mut pipeline = pad.to_string();
    for (i, cmd) in commands.iter().enumerate() {
        if i > 0 {
            pipeline.push_str(" | ");
        }
        pipeline.push_str(&generate_statement(cmd));
    }
    pipeline
}

/// Generate an if/elif/else statement
fn generate_if_stmt(
    pad: &str,
    condition: &BashExpr,
    then_block: &[BashStmt],
    elif_blocks: &[(BashExpr, Vec<BashStmt>)],
    else_block: &Option<Vec<BashStmt>>,
    indent: usize,
) -> String {
    let mut s = format!("{}if {}; then\n", pad, generate_condition(condition));
    for stmt in then_block {
        s.push_str(&generate_stmt(stmt, indent + 1));
        s.push('\n');
    }
    for (elif_cond, elif_body) in elif_blocks {
        s.push_str(&format!(
            "{}elif {}; then\n",
            pad,
            generate_condition(elif_cond)
        ));
        for stmt in elif_body {
            s.push_str(&generate_stmt(stmt, indent + 1));
            s.push('\n');
        }
    }
    if let Some(else_stmts) = else_block {
        s.push_str(&format!("{}else\n", pad));
        for stmt in else_stmts {
            s.push_str(&generate_stmt(stmt, indent + 1));
            s.push('\n');
        }
    }
    s.push_str(pad);
    s.push_str("fi");
    s
}

/// Generate a C-style for loop as POSIX while loop
fn generate_for_c_style(
    pad: &str,
    inner_pad: &str,
    init: &str,
    condition: &str,
    increment: &str,
    body: &[BashStmt],
    indent: usize,
) -> String {
    let mut s = String::new();
    if !init.is_empty() {
        s.push_str(pad);
        s.push_str(&convert_c_init_to_posix(init));
        s.push('\n');
    }
    let posix_condition = convert_c_condition_to_posix(condition);
    s.push_str(&format!("{}while {}; do\n", pad, posix_condition));
    for stmt in body {
        s.push_str(&generate_stmt(stmt, indent + 1));
        s.push('\n');
    }
    if !increment.is_empty() {
        s.push_str(inner_pad);
        s.push_str(&convert_c_increment_to_posix(increment));
        s.push('\n');
    }
    s.push_str(pad);
    s.push_str("done");
    s
}

/// Generate a case statement
fn generate_case_stmt(pad: &str, word: &BashExpr, arms: &[CaseArm], indent: usize) -> String {
    let arm_pad = "    ".repeat(indent + 1);
    let body_pad = "    ".repeat(indent + 2);
    let mut s = format!("{}case {} in\n", pad, generate_expr(word));
    for arm in arms {
        let pattern_str = arm.patterns.join("|");
        s.push_str(&format!("{}{})\n", arm_pad, pattern_str));
        for stmt in &arm.body {
            s.push_str(&generate_stmt(stmt, indent + 2));
            s.push('\n');
        }
        s.push_str(&format!("{};;\n", body_pad));
    }
    s.push_str(pad);
    s.push_str("esac");
    s
}

/// Generate a brace group or subshell
fn generate_brace_group(pad: &str, body: &[BashStmt], subshell: bool, indent: usize) -> String {
    if subshell {
        let mut s = format!("{}(\n", pad);
        for stmt in body {
            s.push_str(&generate_stmt(stmt, indent + 1));
            s.push('\n');
        }
        s.push_str(pad);
        s.push(')');
        s
    } else {
        let mut brace = format!("{}{{ ", pad);
        for (i, stmt) in body.iter().enumerate() {
            if i > 0 {
                brace.push_str("; ");
            }
            brace.push_str(&generate_statement(stmt));
        }
        brace.push_str("; }");
        brace
    }
}

/// Generate a coproc statement
fn generate_coproc(pad: &str, name: &Option<String>, body: &[BashStmt]) -> String {
    let mut coproc = format!("{}coproc ", pad);
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
        BashExpr::Literal(s) => generate_literal_expr(s),
        BashExpr::Variable(name) => format!("\"${}\"", name),
        BashExpr::Array(items) => items
            .iter()
            .map(generate_expr)
            .collect::<Vec<_>>()
            .join(" "),
        BashExpr::Arithmetic(arith) => format!("$(({}))", generate_arith_expr(arith)),
        BashExpr::Test(test) => generate_test_expr(test),
        BashExpr::CommandSubst(cmd) => format!("$({})", generate_statement(cmd)),
        BashExpr::Concat(exprs) => exprs.iter().map(generate_expr).collect::<String>(),
        BashExpr::Glob(pattern) => pattern.clone(),
        BashExpr::DefaultValue { variable, default } => {
            format_param_expansion(variable, ":-", default)
        }
        BashExpr::AssignDefault { variable, default } => {
            format_param_expansion(variable, ":=", default)
        }
        BashExpr::ErrorIfUnset { variable, message } => generate_error_if_unset(variable, message),
        BashExpr::AlternativeValue {
            variable,
            alternative,
        } => format_param_expansion(variable, ":+", alternative),
        BashExpr::StringLength { variable } => format!("\"${{#{}}}\"", variable),
        BashExpr::RemoveSuffix { variable, pattern } => {
            format_param_expansion(variable, "%", pattern)
        }
        BashExpr::RemovePrefix { variable, pattern } => {
            format_param_expansion(variable, "#", pattern)
        }
        BashExpr::RemoveLongestPrefix { variable, pattern } => {
            format_param_expansion(variable, "##", pattern)
        }
        BashExpr::RemoveLongestSuffix { variable, pattern } => {
            format_param_expansion(variable, "%%", pattern)
        }
        BashExpr::CommandCondition(cmd) => generate_statement(cmd),
    }
}

/// Generate a quoted literal expression with proper quoting strategy
fn generate_literal_expr(s: &str) -> String {
    let is_simple_word = !s.is_empty()
        && s.chars().all(|c| {
            c.is_alphanumeric() || c == '_' || c == '-' || c == '.' || c == '/' || c == '='
        });

    if is_simple_word && !is_shell_keyword(s) {
        return s.to_string();
    }
    if is_shell_keyword(s) {
        return format!("\"{}\"", s);
    }

    let needs_double_quotes = s.contains("$(") || s.contains("${") || s.contains('$');
    if needs_double_quotes {
        let escaped = s.replace('"', "\\\"");
        format!("\"{}\"", escaped)
    } else {
        let escaped = s.replace('\'', "'\\''");
        format!("'{}'", escaped)
    }
}

/// Format a parameter expansion like ${VAR:-default}, ${VAR%pattern}, etc.
fn format_param_expansion(variable: &str, operator: &str, operand: &BashExpr) -> String {
    let val = generate_expr(operand);
    let unquoted = strip_quotes(&val);
    format!("\"${{{}{}{}}}\"", variable, operator, unquoted)
}

/// Generate ${VAR:?message} with special quote handling
fn generate_error_if_unset(variable: &str, message: &BashExpr) -> String {
    let msg_val = generate_expr(message);
    let msg_for_expansion = if msg_val.starts_with('"') && msg_val.ends_with('"') {
        msg_val.trim_start_matches('"').trim_end_matches('"')
    } else {
        &msg_val
    };
    format!("\"${{{}:?{}}}\"", variable, msg_for_expansion)
}

/// Strip surrounding quotes (both single and double) from a string
fn strip_quotes(s: &str) -> &str {
    s.trim_matches(|c| c == '"' || c == '\'')
}

/// Check if a string is a POSIX/bash shell keyword that needs quoting in argument context.
/// These keywords can confuse POSIX sh parsers when unquoted (shellcheck SC1010).
fn is_shell_keyword(s: &str) -> bool {
    matches!(
        s,
        "if" | "then"
            | "elif"
            | "else"
            | "fi"
            | "for"
            | "while"
            | "until"
            | "do"
            | "done"
            | "case"
            | "esac"
            | "in"
            | "function"
            | "select"
            | "coproc"
    )
}

/// Convert `declare`/`typeset` to POSIX equivalents.
/// - `declare -i var=val` → `var=val` (integer attribute is a hint, not POSIX)
/// - `declare -r var=val` → `readonly var=val`
/// - `declare -x var=val` → `export var=val`
/// - `declare -a var` → comment (arrays are not POSIX)
/// - `declare -A var` → comment (assoc arrays are not POSIX)
/// - `declare var=val` → `var=val` (plain declare → plain assignment)
fn generate_declare_posix(args: &[BashExpr], redirects: &[Redirect]) -> String {
    let mut flags = Vec::new();
    let mut assignments = Vec::new();

    for arg in args {
        match arg {
            BashExpr::Literal(s) if s.starts_with('-') => {
                flags.push(s.as_str());
            }
            _ => {
                assignments.push(generate_expr(arg));
            }
        }
    }

    let has_readonly = flags.iter().any(|f| f.contains('r'));
    let has_export = flags.iter().any(|f| f.contains('x'));
    let has_array = flags.iter().any(|f| f.contains('a'));
    let has_assoc = flags.iter().any(|f| f.contains('A'));

    // Arrays and associative arrays have no POSIX equivalent
    if has_array || has_assoc {
        let flag_str = flags.join(" ");
        let assign_str = assignments.join(" ");
        if assignments.is_empty() || !assign_str.contains('=') {
            return format!("# declare {} {} (not POSIX)", flag_str, assign_str)
                .trim_end()
                .to_string();
        }
        // Array with assignment: declare -a arr=(items) — emit comment
        return format!("# declare {} {} (not POSIX)", flag_str, assign_str)
            .trim_end()
            .to_string();
    }

    let mut output = String::new();

    // Build the POSIX command prefix
    if has_readonly && has_export {
        output.push_str("export ");
        // Note: readonly + export in a single declare; emit export first, readonly after
        let assign_str = assignments.join(" ");
        output.push_str(&assign_str);
        // Append redirects
        for redirect in redirects {
            output.push(' ');
            output.push_str(&generate_redirect(redirect));
        }
        // Add a second line for readonly
        output.push('\n');
        output.push_str("readonly ");
        output.push_str(&assign_str);
    } else if has_readonly {
        output.push_str("readonly ");
        output.push_str(&assignments.join(" "));
    } else if has_export {
        output.push_str("export ");
        output.push_str(&assignments.join(" "));
    } else {
        // Plain declare or declare -i/-l/-u → just emit the assignment
        output.push_str(&assignments.join(" "));
    }

    // Append redirects
    for redirect in redirects {
        output.push(' ');
        output.push_str(&generate_redirect(redirect));
    }

    output
}

include!("codegen_generate_arith.rs");
