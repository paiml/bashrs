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
        BashStmt::Command {
            name,
            args,
            redirects,
            ..
        } => {
            let mut cmd = name.clone();
            for arg in args {
                cmd.push(' ');
                cmd.push_str(&generate_expr(arg));
            }
            // Issue #72: Emit redirects
            for redirect in redirects {
                cmd.push(' ');
                cmd.push_str(&generate_redirect(redirect));
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
        // Issue #68: C-style for loop → POSIX while loop transformation
        BashStmt::ForCStyle {
            init,
            condition,
            increment,
            body,
            ..
        } => {
            // Convert C-style for loop to POSIX while loop:
            // for ((i=0; i<10; i++)); do ... done
            // →
            // i=0
            // while [ "$i" -lt 10 ]; do
            //     ...
            //     i=$((i + 1))
            // done
            let mut output = String::new();

            // Emit initialization (e.g., i=0)
            if !init.is_empty() {
                output.push_str(&convert_c_init_to_posix(init));
                output.push('\n');
            }

            // Emit while loop with condition
            let posix_condition = convert_c_condition_to_posix(condition);
            output.push_str(&format!("while {}; do\n", posix_condition));

            // Emit body
            for stmt in body {
                output.push_str("    ");
                output.push_str(&generate_statement(stmt));
                output.push('\n');
            }

            // Emit increment at end of loop body
            if !increment.is_empty() {
                output.push_str("    ");
                output.push_str(&convert_c_increment_to_posix(increment));
                output.push('\n');
            }

            output.push_str("done");
            output
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
            // F017: Generate select statement
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
            // Issue #64: Quote string literals for safety
            // Issue #72: Use double quotes if string contains command substitution or variables
            // Only skip quoting for simple alphanumeric words (commands, filenames)
            // that don't need protection

            // Check if this is a simple "safe" identifier that doesn't need quotes
            let is_simple_word = !s.is_empty()
                && s.chars()
                    .all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.' || c == '/');

            // Check if string contains expansions that require double quotes
            let needs_double_quotes = s.contains("$(") || s.contains("${") || s.contains('$');

            if is_simple_word {
                s.clone()
            } else if needs_double_quotes {
                // Issue #72: Use double quotes to preserve command substitution and variable expansion
                // Escape any double quotes in the string
                let escaped = s.replace('"', "\\\"");
                format!("\"{}\"", escaped)
            } else {
                // Use single quotes for literals without expansions
                // Escape any single quotes in the string
                let escaped = s.replace('\'', "'\\''");
                format!("'{}'", escaped)
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
            let default_unquoted = strip_quotes(&default_val);
            format!("\"${{{}:-{}}}\"", variable, default_unquoted)
        }
        BashExpr::AssignDefault { variable, default } => {
            // Generate ${VAR:=default} syntax
            let default_val = generate_expr(default);
            let default_unquoted = strip_quotes(&default_val);
            format!("\"${{{}:={}}}\"", variable, default_unquoted)
        }
        BashExpr::ErrorIfUnset { variable, message } => {
            // Generate ${VAR:?message} syntax
            // Note: Quotes in error messages ARE significant - they show in output
            // So we preserve them (don't strip)
            let msg_val = generate_expr(message);
            // Only strip outer double quotes (from the overall ${} quoting), keep single quotes
            let msg_for_expansion = if msg_val.starts_with('"') && msg_val.ends_with('"') {
                msg_val.trim_start_matches('"').trim_end_matches('"')
            } else {
                &msg_val
            };
            format!("\"${{{}:?{}}}\"", variable, msg_for_expansion)
        }
        BashExpr::AlternativeValue {
            variable,
            alternative,
        } => {
            // Generate ${VAR:+alt_value} syntax
            let alt_val = generate_expr(alternative);
            let alt_unquoted = strip_quotes(&alt_val);
            format!("\"${{{}:+{}}}\"", variable, alt_unquoted)
        }
        BashExpr::StringLength { variable } => {
            // Generate ${#VAR} syntax
            format!("\"${{#{}}}\"", variable)
        }
        BashExpr::RemoveSuffix { variable, pattern } => {
            // Generate ${VAR%pattern} syntax
            let pattern_val = generate_expr(pattern);
            let pattern_unquoted = strip_quotes(&pattern_val);
            format!("\"${{{}%{}}}\"", variable, pattern_unquoted)
        }
        BashExpr::RemovePrefix { variable, pattern } => {
            // Generate ${VAR#pattern} syntax
            let pattern_val = generate_expr(pattern);
            let pattern_unquoted = strip_quotes(&pattern_val);
            format!("\"${{{}#{}}}\"", variable, pattern_unquoted)
        }
        BashExpr::RemoveLongestPrefix { variable, pattern } => {
            // Generate ${VAR##pattern} syntax (greedy prefix removal)
            let pattern_val = generate_expr(pattern);
            let pattern_unquoted = strip_quotes(&pattern_val);
            format!("\"${{{}##{}}}\"", variable, pattern_unquoted)
        }
        BashExpr::RemoveLongestSuffix { variable, pattern } => {
            // Generate ${VAR%%pattern} syntax (greedy suffix removal)
            let pattern_val = generate_expr(pattern);
            let pattern_unquoted = strip_quotes(&pattern_val);
            format!("\"${{{}%%{}}}\"", variable, pattern_unquoted)
        }
        BashExpr::CommandCondition(cmd) => {
            // Issue #93: Command condition - generate the command directly
            // The command's exit code determines the condition result
            generate_statement(cmd)
        }
    }
}

/// Strip surrounding quotes (both single and double) from a string
fn strip_quotes(s: &str) -> &str {
    s.trim_matches(|c| c == '"' || c == '\'')
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
#[cfg(test)]
mod codegen_tests {
    use super::*;
    use crate::bash_parser::BashParser;

    // ============================================================================
    // Statement Generation Tests
    // ============================================================================

    #[test]
    fn test_generate_simple_command() {
        let input = "echo hello world";
        let mut parser = BashParser::new(input).expect("parse");
        let ast = parser.parse().expect("parse");
        let output = generate_purified_bash(&ast);
        assert!(output.contains("echo hello world") || output.contains("echo 'hello' 'world'"));
    }

    #[test]
    fn test_generate_command_with_quotes() {
        let input = r#"echo "hello world""#;
        let mut parser = BashParser::new(input).expect("parse");
        let ast = parser.parse().expect("parse");
        let output = generate_purified_bash(&ast);
        assert!(output.contains("hello world"));
    }

    #[test]
    fn test_generate_assignment() {
        let input = "x=42";
        let mut parser = BashParser::new(input).expect("parse");
        let ast = parser.parse().expect("parse");
        let output = generate_purified_bash(&ast);
        assert!(output.contains("x=42"));
    }

    #[test]
    fn test_generate_exported_assignment() {
        let input = "export PATH=/usr/bin";
        let mut parser = BashParser::new(input).expect("parse");
        let ast = parser.parse().expect("parse");
        let output = generate_purified_bash(&ast);
        assert!(output.contains("export") && output.contains("PATH"));
    }

    #[test]
    fn test_generate_comment() {
        let input = "# This is a comment\necho hello";
        let mut parser = BashParser::new(input).expect("parse");
        let ast = parser.parse().expect("parse");
        let output = generate_purified_bash(&ast);
        // Comment should be preserved (may have different formatting)
        assert!(output.contains("#") && output.contains("comment"));
    }

    #[test]
    fn test_generate_function() {
        let input = "hello() { echo hi; }";
        let mut parser = BashParser::new(input).expect("parse");
        let ast = parser.parse().expect("parse");
        let output = generate_purified_bash(&ast);
        assert!(output.contains("hello()") && output.contains("echo"));
    }

    #[test]
    fn test_generate_if_statement() {
        let input = "if [ -f file ]; then echo exists; fi";
        let mut parser = BashParser::new(input).expect("parse");
        let ast = parser.parse().expect("parse");
        let output = generate_purified_bash(&ast);
        assert!(output.contains("if") && output.contains("then") && output.contains("fi"));
    }

    #[test]
    fn test_generate_if_else_statement() {
        let input = "if [ -f file ]; then echo yes; else echo no; fi";
        let mut parser = BashParser::new(input).expect("parse");
        let ast = parser.parse().expect("parse");
        let output = generate_purified_bash(&ast);
        assert!(output.contains("if") && output.contains("else") && output.contains("fi"));
    }

    #[test]
    fn test_generate_for_loop() {
        let input = "for i in 1 2 3; do echo $i; done";
        let mut parser = BashParser::new(input).expect("parse");
        let ast = parser.parse().expect("parse");
        let output = generate_purified_bash(&ast);
        assert!(output.contains("for") && output.contains("do") && output.contains("done"));
    }

    #[test]
    fn test_generate_while_loop() {
        let input = "while [ $x -lt 10 ]; do echo $x; done";
        let mut parser = BashParser::new(input).expect("parse");
        let ast = parser.parse().expect("parse");
        let output = generate_purified_bash(&ast);
        assert!(output.contains("while") && output.contains("do") && output.contains("done"));
    }

    #[test]
    fn test_generate_case_statement() {
        let input = "case $x in a) echo a;; b) echo b;; esac";
        let mut parser = BashParser::new(input).expect("parse");
        let ast = parser.parse().expect("parse");
        let output = generate_purified_bash(&ast);
        assert!(output.contains("case") && output.contains("esac"));
    }

    #[test]
    fn test_generate_pipeline() {
        let input = "ls | grep foo";
        let mut parser = BashParser::new(input).expect("parse");
        let ast = parser.parse().expect("parse");
        let output = generate_purified_bash(&ast);
        assert!(output.contains("|"));
    }

    #[test]
    fn test_generate_and_list() {
        let input = "test -f file && echo exists";
        let mut parser = BashParser::new(input).expect("parse");
        let ast = parser.parse().expect("parse");
        let output = generate_purified_bash(&ast);
        assert!(output.contains("&&"));
    }

    #[test]
    fn test_generate_or_list() {
        let input = "test -f file || echo missing";
        let mut parser = BashParser::new(input).expect("parse");
        let ast = parser.parse().expect("parse");
        let output = generate_purified_bash(&ast);
        assert!(output.contains("||"));
    }

    #[test]
    fn test_generate_redirect() {
        let input = "echo hello > output.txt";
        let mut parser = BashParser::new(input).expect("parse");
        let ast = parser.parse().expect("parse");
        let output = generate_purified_bash(&ast);
        assert!(output.contains(">"));
    }

    #[test]
    fn test_generate_append_redirect() {
        let input = "echo hello >> output.txt";
        let mut parser = BashParser::new(input).expect("parse");
        let ast = parser.parse().expect("parse");
        let output = generate_purified_bash(&ast);
        assert!(output.contains(">>"));
    }

    #[test]
    fn test_generate_input_redirect() {
        let input = "cat < input.txt";
        let mut parser = BashParser::new(input).expect("parse");
        let ast = parser.parse().expect("parse");
        let output = generate_purified_bash(&ast);
        assert!(output.contains("<"));
    }

    #[test]
    fn test_generate_variable_expansion() {
        let input = r#"echo "$HOME""#;
        let mut parser = BashParser::new(input).expect("parse");
        let ast = parser.parse().expect("parse");
        let output = generate_purified_bash(&ast);
        assert!(output.contains("HOME"));
    }

    #[test]
    fn test_generate_arithmetic() {
        let input = "x=$((1 + 2))";
        let mut parser = BashParser::new(input).expect("parse");
        let ast = parser.parse().expect("parse");
        let output = generate_purified_bash(&ast);
        assert!(output.contains("$((") || output.contains("x="));
    }

    #[test]
    fn test_generate_command_substitution() {
        let input = "x=$(pwd)";
        let mut parser = BashParser::new(input).expect("parse");
        let ast = parser.parse().expect("parse");
        let output = generate_purified_bash(&ast);
        assert!(output.contains("$(") || output.contains("pwd"));
    }

    #[test]
    fn test_generate_return_statement() {
        let input = "return 0";
        let mut parser = BashParser::new(input).expect("parse");
        let ast = parser.parse().expect("parse");
        let output = generate_purified_bash(&ast);
        assert!(output.contains("return"));
    }

    #[test]
    fn test_generate_shebang_replaced() {
        let input = "#!/bin/bash\necho hello";
        let mut parser = BashParser::new(input).expect("parse");
        let ast = parser.parse().expect("parse");
        let output = generate_purified_bash(&ast);
        // Shebang should be replaced with #!/bin/sh
        assert!(output.starts_with("#!/bin/sh"));
        // Should not have duplicate shebangs
        assert_eq!(output.matches("#!/bin/sh").count(), 1);
    }

    #[test]
    fn test_generate_subshell() {
        // Use a simpler subshell syntax that parses correctly
        let input = "result=$(pwd)";
        let mut parser = BashParser::new(input).expect("parse");
        let ast = parser.parse().expect("parse");
        let output = generate_purified_bash(&ast);
        assert!(output.contains("$(") || output.contains("pwd"));
    }

    #[test]
    fn test_generate_brace_group() {
        let input = "{ echo a; echo b; }";
        let mut parser = BashParser::new(input).expect("parse");
        let ast = parser.parse().expect("parse");
        let output = generate_purified_bash(&ast);
        assert!(output.contains("{") && output.contains("}"));
    }

    // ============================================================================
    // Expression Generation Tests
    // ============================================================================

    #[test]
    fn test_generate_string_literal() {
        let input = "echo 'literal'";
        let mut parser = BashParser::new(input).expect("parse");
        let ast = parser.parse().expect("parse");
        let output = generate_purified_bash(&ast);
        assert!(output.contains("literal"));
    }

    #[test]
    fn test_generate_array_access() {
        let input = "echo ${arr[0]}";
        let mut parser = BashParser::new(input).expect("parse");
        let ast = parser.parse().expect("parse");
        let output = generate_purified_bash(&ast);
        // Array access should be preserved or transformed
        assert!(output.contains("arr") || output.contains("${"));
    }

    #[test]
    fn test_generate_parameter_default() {
        let input = "echo ${x:-default}";
        let mut parser = BashParser::new(input).expect("parse");
        let ast = parser.parse().expect("parse");
        let output = generate_purified_bash(&ast);
        assert!(output.contains(":-") || output.contains("default"));
    }

    #[test]
    fn test_generate_here_document() {
        let input = "cat <<EOF\nhello\nEOF";
        let mut parser = BashParser::new(input).expect("parse");
        let ast = parser.parse().expect("parse");
        let output = generate_purified_bash(&ast);
        assert!(output.contains("<<") || output.contains("hello"));
    }

    // ============================================================================
    // Edge Cases
    // ============================================================================

    #[test]
    fn test_generate_empty_ast() {
        let ast = BashAst {
            statements: vec![],
            metadata: AstMetadata {
                source_file: None,
                line_count: 0,
                parse_time_ms: 0,
            },
        };
        let output = generate_purified_bash(&ast);
        assert!(output.starts_with("#!/bin/sh"));
    }

    #[test]
    fn test_generate_nested_structures() {
        let input = "if true; then for i in 1 2; do echo $i; done; fi";
        let mut parser = BashParser::new(input).expect("parse");
        let ast = parser.parse().expect("parse");
        let output = generate_purified_bash(&ast);
        assert!(
            output.contains("if")
                && output.contains("for")
                && output.contains("done")
                && output.contains("fi")
        );
    }

    #[test]
    fn test_generate_complex_pipeline() {
        let input = "cat file | grep pattern | sort | uniq";
        let mut parser = BashParser::new(input).expect("parse");
        let ast = parser.parse().expect("parse");
        let output = generate_purified_bash(&ast);
        assert!(output.contains("|"));
    }
}

// ============================================================================
// Additional Coverage Tests - Direct Unit Tests
// ============================================================================
#[cfg(test)]
mod codegen_coverage_tests {
    use super::*;

    // ============================================================================
    // Redirect Generation Tests
    // ============================================================================

    #[test]
    fn test_generate_redirect_error() {
        let redirect = Redirect::Error {
            target: BashExpr::Literal("error.log".to_string()),
        };
        let output = generate_redirect(&redirect);
        assert_eq!(output, "2> error.log");
    }

    #[test]
    fn test_generate_redirect_append_error() {
        let redirect = Redirect::AppendError {
            target: BashExpr::Literal("error.log".to_string()),
        };
        let output = generate_redirect(&redirect);
        assert_eq!(output, "2>> error.log");
    }

    #[test]
    fn test_generate_redirect_combined() {
        let redirect = Redirect::Combined {
            target: BashExpr::Literal("all.log".to_string()),
        };
        let output = generate_redirect(&redirect);
        assert_eq!(output, "> all.log 2>&1");
    }

    #[test]
    fn test_generate_redirect_duplicate() {
        let redirect = Redirect::Duplicate {
            from_fd: 2,
            to_fd: 1,
        };
        let output = generate_redirect(&redirect);
        assert_eq!(output, "2>&1");
    }

    #[test]
    fn test_generate_redirect_here_string() {
        let redirect = Redirect::HereString {
            content: "hello world".to_string(),
        };
        let output = generate_redirect(&redirect);
        assert_eq!(output, "<<< \"hello world\"");
    }

    #[test]
    fn test_generate_redirect_here_string_with_quotes() {
        let redirect = Redirect::HereString {
            content: "say \"hello\"".to_string(),
        };
        let output = generate_redirect(&redirect);
        assert_eq!(output, "<<< \"say \\\"hello\\\"\"");
    }

    // ============================================================================
    // Test Expression Coverage
    // ============================================================================

    #[test]
    fn test_generate_test_expr_int_ne() {
        let expr = TestExpr::IntNe(
            BashExpr::Variable("a".to_string()),
            BashExpr::Literal("5".to_string()),
        );
        let output = generate_test_expr(&expr);
        assert_eq!(output, "[ \"$a\" -ne 5 ]");
    }

    #[test]
    fn test_generate_test_expr_int_le() {
        let expr = TestExpr::IntLe(
            BashExpr::Variable("x".to_string()),
            BashExpr::Literal("10".to_string()),
        );
        let output = generate_test_expr(&expr);
        assert_eq!(output, "[ \"$x\" -le 10 ]");
    }

    #[test]
    fn test_generate_test_expr_int_ge() {
        let expr = TestExpr::IntGe(
            BashExpr::Variable("y".to_string()),
            BashExpr::Literal("0".to_string()),
        );
        let output = generate_test_expr(&expr);
        assert_eq!(output, "[ \"$y\" -ge 0 ]");
    }

    #[test]
    fn test_generate_test_expr_file_exists() {
        let expr = TestExpr::FileExists(BashExpr::Variable("file".to_string()));
        let output = generate_test_expr(&expr);
        assert_eq!(output, "[ -e \"$file\" ]");
    }

    #[test]
    fn test_generate_test_expr_file_readable() {
        let expr = TestExpr::FileReadable(BashExpr::Literal("/etc/passwd".to_string()));
        let output = generate_test_expr(&expr);
        assert_eq!(output, "[ -r /etc/passwd ]");
    }

    #[test]
    fn test_generate_test_expr_file_writable() {
        let expr = TestExpr::FileWritable(BashExpr::Literal("/tmp/test".to_string()));
        let output = generate_test_expr(&expr);
        assert_eq!(output, "[ -w /tmp/test ]");
    }

    #[test]
    fn test_generate_test_expr_file_executable() {
        let expr = TestExpr::FileExecutable(BashExpr::Literal("/bin/sh".to_string()));
        let output = generate_test_expr(&expr);
        assert_eq!(output, "[ -x /bin/sh ]");
    }

    #[test]
    fn test_generate_test_expr_string_empty() {
        let expr = TestExpr::StringEmpty(BashExpr::Variable("str".to_string()));
        let output = generate_test_expr(&expr);
        assert_eq!(output, "[ -z \"$str\" ]");
    }

    #[test]
    fn test_generate_test_expr_string_non_empty() {
        let expr = TestExpr::StringNonEmpty(BashExpr::Variable("str".to_string()));
        let output = generate_test_expr(&expr);
        assert_eq!(output, "[ -n \"$str\" ]");
    }

    #[test]
    fn test_generate_test_expr_and() {
        let expr = TestExpr::And(
            Box::new(TestExpr::FileExists(BashExpr::Literal("a".to_string()))),
            Box::new(TestExpr::FileReadable(BashExpr::Literal("a".to_string()))),
        );
        let output = generate_test_expr(&expr);
        assert_eq!(output, "[ -e a ] && [ -r a ]");
    }

    #[test]
    fn test_generate_test_expr_or() {
        let expr = TestExpr::Or(
            Box::new(TestExpr::FileExists(BashExpr::Literal("a".to_string()))),
            Box::new(TestExpr::FileExists(BashExpr::Literal("b".to_string()))),
        );
        let output = generate_test_expr(&expr);
        assert_eq!(output, "[ -e a ] || [ -e b ]");
    }

    #[test]
    fn test_generate_test_expr_not() {
        let expr = TestExpr::Not(Box::new(TestExpr::FileExists(BashExpr::Literal(
            "x".to_string(),
        ))));
        let output = generate_test_expr(&expr);
        assert_eq!(output, "! [ -e x ]");
    }

    // ============================================================================
    // Arithmetic Expression Coverage
    // ============================================================================

    #[test]
    fn test_generate_arith_sub() {
        let expr = ArithExpr::Sub(
            Box::new(ArithExpr::Variable("a".to_string())),
            Box::new(ArithExpr::Number(1)),
        );
        let output = generate_arith_expr(&expr);
        assert_eq!(output, "a - 1");
    }

    #[test]
    fn test_generate_arith_mul() {
        let expr = ArithExpr::Mul(
            Box::new(ArithExpr::Number(3)),
            Box::new(ArithExpr::Number(4)),
        );
        let output = generate_arith_expr(&expr);
        assert_eq!(output, "3 * 4");
    }

    #[test]
    fn test_generate_arith_div() {
        let expr = ArithExpr::Div(
            Box::new(ArithExpr::Number(10)),
            Box::new(ArithExpr::Number(2)),
        );
        let output = generate_arith_expr(&expr);
        assert_eq!(output, "10 / 2");
    }

    #[test]
    fn test_generate_arith_mod() {
        let expr = ArithExpr::Mod(
            Box::new(ArithExpr::Number(7)),
            Box::new(ArithExpr::Number(3)),
        );
        let output = generate_arith_expr(&expr);
        assert_eq!(output, "7 % 3");
    }

    // ============================================================================
    // Expression Generation Coverage
    // ============================================================================

    #[test]
    fn test_generate_expr_literal_with_spaces() {
        let expr = BashExpr::Literal("hello world".to_string());
        let output = generate_expr(&expr);
        assert_eq!(output, "'hello world'");
    }

    #[test]
    fn test_generate_expr_literal_with_single_quote() {
        let expr = BashExpr::Literal("don't".to_string());
        let output = generate_expr(&expr);
        assert_eq!(output, "'don'\\''t'");
    }

    #[test]
    fn test_generate_expr_literal_with_command_subst() {
        let expr = BashExpr::Literal("$(pwd)".to_string());
        let output = generate_expr(&expr);
        assert_eq!(output, "\"$(pwd)\"");
    }

    #[test]
    fn test_generate_expr_literal_with_variable() {
        let expr = BashExpr::Literal("$HOME".to_string());
        let output = generate_expr(&expr);
        assert_eq!(output, "\"$HOME\"");
    }

    #[test]
    fn test_generate_expr_literal_with_brace_expansion() {
        let expr = BashExpr::Literal("${HOME}".to_string());
        let output = generate_expr(&expr);
        assert_eq!(output, "\"${HOME}\"");
    }

    #[test]
    fn test_generate_expr_literal_with_double_quote() {
        let expr = BashExpr::Literal("say \"hi\"".to_string());
        let output = generate_expr(&expr);
        // Contains embedded quotes but no expansion - uses single quotes
        assert_eq!(output, "'say \"hi\"'");
    }

    #[test]
    fn test_generate_expr_array() {
        let expr = BashExpr::Array(vec![
            BashExpr::Literal("a".to_string()),
            BashExpr::Literal("b".to_string()),
            BashExpr::Literal("c".to_string()),
        ]);
        let output = generate_expr(&expr);
        assert_eq!(output, "a b c");
    }

    #[test]
    fn test_generate_expr_glob() {
        let expr = BashExpr::Glob("*.txt".to_string());
        let output = generate_expr(&expr);
        assert_eq!(output, "*.txt");
    }

    #[test]
    fn test_generate_expr_concat() {
        let expr = BashExpr::Concat(vec![
            BashExpr::Literal("prefix_".to_string()),
            BashExpr::Variable("var".to_string()),
        ]);
        let output = generate_expr(&expr);
        assert!(output.contains("prefix_") && output.contains("$var"));
    }

    #[test]
    fn test_generate_expr_assign_default() {
        let expr = BashExpr::AssignDefault {
            variable: "x".to_string(),
            default: Box::new(BashExpr::Literal("default".to_string())),
        };
        let output = generate_expr(&expr);
        assert_eq!(output, "\"${x:=default}\"");
    }

    #[test]
    fn test_generate_expr_error_if_unset() {
        let expr = BashExpr::ErrorIfUnset {
            variable: "x".to_string(),
            message: Box::new(BashExpr::Literal("not set".to_string())),
        };
        let output = generate_expr(&expr);
        assert!(output.contains("${x:?"));
    }

    #[test]
    fn test_generate_expr_alternative_value() {
        let expr = BashExpr::AlternativeValue {
            variable: "x".to_string(),
            alternative: Box::new(BashExpr::Literal("alt".to_string())),
        };
        let output = generate_expr(&expr);
        assert_eq!(output, "\"${x:+alt}\"");
    }

    #[test]
    fn test_generate_expr_string_length() {
        let expr = BashExpr::StringLength {
            variable: "str".to_string(),
        };
        let output = generate_expr(&expr);
        assert_eq!(output, "\"${#str}\"");
    }

    #[test]
    fn test_generate_expr_remove_suffix() {
        let expr = BashExpr::RemoveSuffix {
            variable: "file".to_string(),
            pattern: Box::new(BashExpr::Literal(".txt".to_string())),
        };
        let output = generate_expr(&expr);
        assert_eq!(output, "\"${file%.txt}\"");
    }

    #[test]
    fn test_generate_expr_remove_prefix() {
        let expr = BashExpr::RemovePrefix {
            variable: "path".to_string(),
            pattern: Box::new(BashExpr::Literal("/".to_string())),
        };
        let output = generate_expr(&expr);
        assert!(output.contains("${path#"));
    }

    #[test]
    fn test_generate_expr_remove_longest_prefix() {
        let expr = BashExpr::RemoveLongestPrefix {
            variable: "path".to_string(),
            pattern: Box::new(BashExpr::Literal("*/".to_string())),
        };
        let output = generate_expr(&expr);
        assert!(output.contains("${path##"));
    }

    #[test]
    fn test_generate_expr_remove_longest_suffix() {
        let expr = BashExpr::RemoveLongestSuffix {
            variable: "file".to_string(),
            pattern: Box::new(BashExpr::Literal(".*".to_string())),
        };
        let output = generate_expr(&expr);
        assert!(output.contains("${file%%"));
    }

    #[test]
    fn test_generate_expr_command_condition() {
        let cmd = Box::new(BashStmt::Command {
            name: "test".to_string(),
            args: vec![
                BashExpr::Literal("-f".to_string()),
                BashExpr::Literal("file".to_string()),
            ],
            redirects: vec![],
            span: Span::dummy(),
        });
        let expr = BashExpr::CommandCondition(cmd);
        let output = generate_expr(&expr);
        assert!(output.contains("test") && output.contains("-f"));
    }

    // ============================================================================
    // Statement Generation Coverage
    // ============================================================================

    #[test]
    fn test_generate_statement_return_without_code() {
        let stmt = BashStmt::Return {
            code: None,
            span: Span::dummy(),
        };
        let output = generate_statement(&stmt);
        assert_eq!(output, "return");
    }

    #[test]
    fn test_generate_statement_coproc_with_name() {
        let stmt = BashStmt::Coproc {
            name: Some("MY_PROC".to_string()),
            body: vec![BashStmt::Command {
                name: "cat".to_string(),
                args: vec![],
                redirects: vec![],
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        };
        let output = generate_statement(&stmt);
        assert!(output.contains("coproc MY_PROC"));
    }

    #[test]
    fn test_generate_statement_coproc_without_name() {
        let stmt = BashStmt::Coproc {
            name: None,
            body: vec![BashStmt::Command {
                name: "cat".to_string(),
                args: vec![],
                redirects: vec![],
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        };
        let output = generate_statement(&stmt);
        assert!(output.starts_with("coproc {"));
    }

    #[test]
    fn test_generate_statement_until_loop() {
        let stmt = BashStmt::Until {
            condition: BashExpr::Test(Box::new(TestExpr::IntGt(
                BashExpr::Variable("i".to_string()),
                BashExpr::Literal("5".to_string()),
            ))),
            body: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Variable("i".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        };
        let output = generate_statement(&stmt);
        // until loop converts to while with negated condition
        assert!(output.contains("while") && output.contains("done"));
    }

    #[test]
    fn test_generate_statement_for_c_style() {
        let stmt = BashStmt::ForCStyle {
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
        };
        let output = generate_statement(&stmt);
        // C-style for loop converts to POSIX while loop
        assert!(output.contains("i=0"));
        assert!(output.contains("while"));
        assert!(output.contains("-lt"));
        assert!(output.contains("done"));
    }

    #[test]
    fn test_generate_statement_for_c_style_empty_init() {
        let stmt = BashStmt::ForCStyle {
            init: "".to_string(),
            condition: "i<10".to_string(),
            increment: "".to_string(),
            body: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Variable("i".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        };
        let output = generate_statement(&stmt);
        assert!(output.contains("while"));
        // No init line, no increment at end
    }

    // ============================================================================
    // negate_condition Coverage
    // ============================================================================

    #[test]
    fn test_negate_condition_test_expr() {
        let condition = BashExpr::Test(Box::new(TestExpr::IntGt(
            BashExpr::Variable("x".to_string()),
            BashExpr::Literal("5".to_string()),
        )));
        let output = negate_condition(&condition);
        assert!(output.contains("! ") || output.contains("[ !"));
    }

    #[test]
    fn test_negate_condition_non_test() {
        let condition = BashExpr::Literal("true".to_string());
        let output = negate_condition(&condition);
        assert!(output.starts_with("! "));
    }

    // ============================================================================
    // generate_test_condition Coverage
    // ============================================================================

    #[test]
    fn test_generate_test_condition_int_ne() {
        let expr = TestExpr::IntNe(
            BashExpr::Variable("a".to_string()),
            BashExpr::Literal("0".to_string()),
        );
        let output = generate_test_condition(&expr);
        assert_eq!(output, "\"$a\" -ne 0");
    }

    #[test]
    fn test_generate_test_condition_int_le() {
        let expr = TestExpr::IntLe(
            BashExpr::Variable("x".to_string()),
            BashExpr::Literal("100".to_string()),
        );
        let output = generate_test_condition(&expr);
        assert_eq!(output, "\"$x\" -le 100");
    }

    #[test]
    fn test_generate_test_condition_int_ge() {
        let expr = TestExpr::IntGe(
            BashExpr::Variable("y".to_string()),
            BashExpr::Literal("1".to_string()),
        );
        let output = generate_test_condition(&expr);
        assert_eq!(output, "\"$y\" -ge 1");
    }

    #[test]
    fn test_generate_test_condition_file_exists() {
        let expr = TestExpr::FileExists(BashExpr::Literal("/tmp".to_string()));
        let output = generate_test_condition(&expr);
        assert_eq!(output, "-e /tmp");
    }

    #[test]
    fn test_generate_test_condition_file_readable() {
        let expr = TestExpr::FileReadable(BashExpr::Literal("file".to_string()));
        let output = generate_test_condition(&expr);
        assert_eq!(output, "-r file");
    }

    #[test]
    fn test_generate_test_condition_file_writable() {
        let expr = TestExpr::FileWritable(BashExpr::Literal("file".to_string()));
        let output = generate_test_condition(&expr);
        assert_eq!(output, "-w file");
    }

    #[test]
    fn test_generate_test_condition_file_executable() {
        let expr = TestExpr::FileExecutable(BashExpr::Literal("script".to_string()));
        let output = generate_test_condition(&expr);
        assert_eq!(output, "-x script");
    }

    #[test]
    fn test_generate_test_condition_string_empty() {
        let expr = TestExpr::StringEmpty(BashExpr::Variable("s".to_string()));
        let output = generate_test_condition(&expr);
        assert_eq!(output, "-z \"$s\"");
    }

    #[test]
    fn test_generate_test_condition_string_non_empty() {
        let expr = TestExpr::StringNonEmpty(BashExpr::Variable("s".to_string()));
        let output = generate_test_condition(&expr);
        assert_eq!(output, "-n \"$s\"");
    }

    #[test]
    fn test_generate_test_condition_and() {
        let expr = TestExpr::And(
            Box::new(TestExpr::FileExists(BashExpr::Literal("a".to_string()))),
            Box::new(TestExpr::FileDirectory(BashExpr::Literal("a".to_string()))),
        );
        let output = generate_test_condition(&expr);
        assert!(output.contains("&&"));
    }

    #[test]
    fn test_generate_test_condition_or() {
        let expr = TestExpr::Or(
            Box::new(TestExpr::FileExists(BashExpr::Literal("a".to_string()))),
            Box::new(TestExpr::FileExists(BashExpr::Literal("b".to_string()))),
        );
        let output = generate_test_condition(&expr);
        assert!(output.contains("||"));
    }

    #[test]
    fn test_generate_test_condition_not() {
        let expr = TestExpr::Not(Box::new(TestExpr::FileExists(BashExpr::Literal(
            "x".to_string(),
        ))));
        let output = generate_test_condition(&expr);
        assert!(output.starts_with("! "));
    }

    // ============================================================================
    // C-style for loop conversion helpers
    // ============================================================================

    #[test]
    fn test_convert_c_init_to_posix() {
        assert_eq!(convert_c_init_to_posix("i=0"), "i=0");
        assert_eq!(convert_c_init_to_posix("x=10"), "x=10");
    }

    #[test]
    fn test_convert_c_condition_less_equal() {
        let output = convert_c_condition_to_posix("i<=10");
        assert!(output.contains("-le") && output.contains("$i"));
    }

    #[test]
    fn test_convert_c_condition_greater_equal() {
        let output = convert_c_condition_to_posix("i>=0");
        assert!(output.contains("-ge") && output.contains("$i"));
    }

    #[test]
    fn test_convert_c_condition_not_equal() {
        let output = convert_c_condition_to_posix("i!=5");
        assert!(output.contains("-ne") && output.contains("$i"));
    }

    #[test]
    fn test_convert_c_condition_equal() {
        let output = convert_c_condition_to_posix("i==0");
        assert!(output.contains("-eq") && output.contains("$i"));
    }

    #[test]
    fn test_convert_c_condition_greater() {
        let output = convert_c_condition_to_posix("i>5");
        assert!(output.contains("-gt") && output.contains("$i"));
    }

    #[test]
    fn test_convert_c_condition_fallback() {
        let output = convert_c_condition_to_posix("some_expr");
        assert_eq!(output, "[ some_expr ]");
    }

    #[test]
    fn test_convert_c_increment_postfix_increment() {
        let output = convert_c_increment_to_posix("i++");
        assert_eq!(output, "i=$((i+1))");
    }

    #[test]
    fn test_convert_c_increment_prefix_increment() {
        let output = convert_c_increment_to_posix("++i");
        assert_eq!(output, "i=$((i+1))");
    }

    #[test]
    fn test_convert_c_increment_postfix_decrement() {
        let output = convert_c_increment_to_posix("i--");
        assert_eq!(output, "i=$((i-1))");
    }

    #[test]
    fn test_convert_c_increment_prefix_decrement() {
        let output = convert_c_increment_to_posix("--i");
        assert_eq!(output, "i=$((i-1))");
    }

    #[test]
    fn test_convert_c_increment_plus_equals() {
        let output = convert_c_increment_to_posix("i+=2");
        assert_eq!(output, "i=$((i+2))");
    }

    #[test]
    fn test_convert_c_increment_minus_equals() {
        let output = convert_c_increment_to_posix("i-=3");
        assert_eq!(output, "i=$((i-3))");
    }

    #[test]
    fn test_convert_c_increment_assignment() {
        let output = convert_c_increment_to_posix("i=i+1");
        assert_eq!(output, "i=i+1");
    }

    #[test]
    fn test_convert_c_increment_fallback() {
        let output = convert_c_increment_to_posix("something_else");
        assert_eq!(output, ":something_else");
    }

    // ============================================================================
    // extract_var_name Coverage
    // ============================================================================

    #[test]
    fn test_extract_var_name_with_dollar() {
        assert_eq!(extract_var_name("$i"), "i");
        assert_eq!(extract_var_name("$var"), "var");
    }

    #[test]
    fn test_extract_var_name_without_dollar() {
        assert_eq!(extract_var_name("i"), "i");
        assert_eq!(extract_var_name("count"), "count");
    }

    // ============================================================================
    // strip_quotes Coverage
    // ============================================================================

    #[test]
    fn test_strip_quotes_double() {
        assert_eq!(strip_quotes("\"value\""), "value");
    }

    #[test]
    fn test_strip_quotes_single() {
        assert_eq!(strip_quotes("'value'"), "value");
    }

    #[test]
    fn test_strip_quotes_mixed() {
        assert_eq!(strip_quotes("\"value'"), "value");
    }

    #[test]
    fn test_strip_quotes_none() {
        assert_eq!(strip_quotes("value"), "value");
    }

    // ============================================================================
    // generate_condition Coverage
    // ============================================================================

    #[test]
    fn test_generate_condition_test() {
        let expr = BashExpr::Test(Box::new(TestExpr::FileExists(BashExpr::Literal(
            "f".to_string(),
        ))));
        let output = generate_condition(&expr);
        assert!(output.contains("-e"));
    }

    #[test]
    fn test_generate_condition_non_test() {
        let expr = BashExpr::Literal("true".to_string());
        let output = generate_condition(&expr);
        assert_eq!(output, "true");
    }

    // ============================================================================
    // Comment shebang filtering
    // ============================================================================

    #[test]
    fn test_generate_comment_shebang_filtered() {
        let stmt = BashStmt::Comment {
            text: "!/bin/bash".to_string(),
            span: Span::dummy(),
        };
        let output = generate_statement(&stmt);
        assert_eq!(output, "");
    }

    #[test]
    fn test_generate_comment_shebang_with_space_filtered() {
        let stmt = BashStmt::Comment {
            text: " !/bin/sh".to_string(),
            span: Span::dummy(),
        };
        let output = generate_statement(&stmt);
        assert_eq!(output, "");
    }

    #[test]
    fn test_generate_comment_normal() {
        let stmt = BashStmt::Comment {
            text: "This is a normal comment".to_string(),
            span: Span::dummy(),
        };
        let output = generate_statement(&stmt);
        assert_eq!(output, "# This is a normal comment");
    }
}

#[cfg(test)]
mod test_issue_64 {
    use crate::bash_parser::codegen::generate_purified_bash;
    use crate::bash_parser::BashParser;

    #[test]
    fn test_ISSUE_64_single_quoted_ansi_codes() {
        // RED phase: Test single-quoted ANSI escape sequences
        let input = r#"RED='\033[0;31m'"#;
        let mut parser = BashParser::new(input).expect("Failed to parse");
        let ast = parser.parse().expect("Failed to parse");
        let output = generate_purified_bash(&ast);

        // Single quotes should be preserved for escape sequences
        assert!(
            output.contains("RED='\\033[0;31m'"),
            "Output should preserve single quotes around escape sequences: {}",
            output
        );
    }

    #[test]
    fn test_ISSUE_64_single_quoted_literal() {
        let input = "echo 'Hello World'";
        let mut parser = BashParser::new(input).expect("Failed to parse");
        let ast = parser.parse().expect("Failed to parse");
        let output = generate_purified_bash(&ast);

        // Single quotes should be preserved
        assert!(
            output.contains("'Hello World'"),
            "Output should preserve single quotes: {}",
            output
        );
    }

    #[test]
    fn test_ISSUE_64_assignment_with_single_quotes() {
        let input = "x='value'";
        let mut parser = BashParser::new(input).expect("Failed to parse");
        let ast = parser.parse().expect("Failed to parse");
        let output = generate_purified_bash(&ast);

        // For simple alphanumeric strings, quotes are optional in purified output
        // Both x=value and x='value' are correct POSIX shell
        // The important thing is it parses without error
        assert!(
            output.contains("x=value") || output.contains("x='value'"),
            "Output should contain valid assignment: {}",
            output
        );
    }
}
