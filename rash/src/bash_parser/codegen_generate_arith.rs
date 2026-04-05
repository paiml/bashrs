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

/// Generate redirect
/// Issue #72: Properly emit redirects in purified output
fn generate_redirect(redirect: &Redirect) -> String {
    match redirect {
        Redirect::Output { target } => {
            format!("> {}", generate_expr(target))
        }
        Redirect::Append { target } => {
            format!(">> {}", generate_expr(target))
        }
        Redirect::Input { target } => {
            format!("< {}", generate_expr(target))
        }
        Redirect::Error { target } => {
            format!("2> {}", generate_expr(target))
        }
        Redirect::AppendError { target } => {
            format!("2>> {}", generate_expr(target))
        }
        Redirect::Combined { target } => {
            // Bash &> → POSIX: >file 2>&1
            format!("> {} 2>&1", generate_expr(target))
        }
        Redirect::Duplicate { from_fd, to_fd } => {
            format!("{from_fd}>&{to_fd}")
        }
        Redirect::HereString { content } => {
            // Here-string: <<< "string"
            // Note: Not POSIX, but preserve for bash compatibility
            format!("<<< \"{}\"", content.replace('"', "\\\""))
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

/// Issue #68: Convert C-style for loop initialization to POSIX
/// Example: "i=0" → "i=0"
fn convert_c_init_to_posix(init: &str) -> String {
    // Most initializations are already valid POSIX (e.g., i=0)
    init.to_string()
}

/// Issue #68: Convert C-style for loop condition to POSIX test
/// Example: "i<10" → "[ \"$i\" -lt 10 ]"
fn convert_c_condition_to_posix(condition: &str) -> String {
    let condition = condition.trim();

    // Handle common comparison operators
    if let Some(pos) = condition.find("<=") {
        let (left, right) = condition.split_at(pos);
        let right = &right[2..]; // skip "<="
        let var = extract_var_name(left.trim());
        return format!("[ \"${}\" -le {} ]", var, right.trim());
    }
    if let Some(pos) = condition.find(">=") {
        let (left, right) = condition.split_at(pos);
        let right = &right[2..]; // skip ">="
        let var = extract_var_name(left.trim());
        return format!("[ \"${}\" -ge {} ]", var, right.trim());
    }
    if let Some(pos) = condition.find("!=") {
        let (left, right) = condition.split_at(pos);
        let right = &right[2..]; // skip "!="
        let var = extract_var_name(left.trim());
        return format!("[ \"${}\" -ne {} ]", var, right.trim());
    }
    if let Some(pos) = condition.find("==") {
        let (left, right) = condition.split_at(pos);
        let right = &right[2..]; // skip "=="
        let var = extract_var_name(left.trim());
        return format!("[ \"${}\" -eq {} ]", var, right.trim());
    }
    if let Some(pos) = condition.find('<') {
        let (left, right) = condition.split_at(pos);
        let right = &right[1..]; // skip "<"
        let var = extract_var_name(left.trim());
        return format!("[ \"${}\" -lt {} ]", var, right.trim());
    }
    if let Some(pos) = condition.find('>') {
        let (left, right) = condition.split_at(pos);
        let right = &right[1..]; // skip ">"
        let var = extract_var_name(left.trim());
        return format!("[ \"${}\" -gt {} ]", var, right.trim());
    }

    // Fallback: wrap as-is in test brackets
    format!("[ {} ]", condition)
}

/// Issue #68: Convert C-style increment/decrement to POSIX arithmetic
/// Example: "i++" → "i=$((i + 1))"
fn convert_c_increment_to_posix(increment: &str) -> String {
    let increment = increment.trim();

    // Handle i++
    if let Some(var) = increment.strip_suffix("++") {
        return format!("{}=$(({}+1))", var, var);
    }

    // Handle ++i
    if let Some(var) = increment.strip_prefix("++") {
        return format!("{}=$(({}+1))", var, var);
    }

    // Handle i--
    if let Some(var) = increment.strip_suffix("--") {
        return format!("{}=$(({}-1))", var, var);
    }

    // Handle --i
    if let Some(var) = increment.strip_prefix("--") {
        return format!("{}=$(({}-1))", var, var);
    }

    // Handle i+=n or i-=n
    if let Some(pos) = increment.find("+=") {
        let var = increment[..pos].trim();
        let val = increment[pos + 2..].trim();
        return format!("{}=$(({}+{}))", var, var, val);
    }
    if let Some(pos) = increment.find("-=") {
        let var = increment[..pos].trim();
        let val = increment[pos + 2..].trim();
        return format!("{}=$(({}-{}))", var, var, val);
    }

    // Handle i=i+1 style
    if increment.contains('=') {
        return increment.to_string();
    }

    // Fallback: wrap in arithmetic expansion
    format!(":{}", increment) // No-op fallback
}

/// Extract variable name from an expression (strip $ prefix if present)
fn extract_var_name(s: &str) -> String {
    if let Some(stripped) = s.strip_prefix('$') {
        stripped.to_string()
    } else {
        s.to_string()
    }
}

/// Generate purified bash with runtime type guards inserted after annotated assignments.
///
/// This function takes a purified AST and a TypeChecker (which has already been run
/// via `check_ast`), and emits guards for variables that have type annotations.
pub fn generate_purified_bash_with_guards(
    ast: &BashAst,
    checker: &crate::bash_transpiler::type_check::TypeChecker,
) -> String {
    let mut output = String::new();
    output.push_str("#!/bin/sh\n");

    for stmt in &ast.statements {
        let stmt_str = generate_statement(stmt);
        output.push_str(&stmt_str);
        output.push('\n');

        // After assignments, emit guard only for explicitly annotated variables
        if let BashStmt::Assignment { name, .. } = stmt {
            if let Some(hint) = checker.annotation_hint(name) {
                if let Some(ty) = checker.context().lookup(name) {
                    if let Some(guard) = crate::bash_transpiler::type_check::generate_guard_for_type(
                        name,
                        ty,
                        Some(hint),
                    ) {
                        output.push_str(&guard);
                        output.push('\n');
                    }
                }
            }
        }
    }

    output
}

#[cfg(test)]
#[path = "codegen_tests_generate_sim.rs"]
mod tests_ext;
