fn convert_macro_expr_print(tokens: proc_macro2::TokenStream, macro_name: &str) -> Result<Expr> {
    let token_str = tokens.to_string();
    let parts = split_macro_args(&token_str);
    let func_name = print_macro_func_name(macro_name);

    let arg = if parts.len() <= 1 {
        let parsed: syn::Expr = syn::parse2(tokens)
            .map_err(|_| Error::Validation(format!("Invalid {macro_name}! arguments")))?;
        convert_expr(&parsed)?
    } else {
        parse_format_macro_args(&parts, macro_name)?
    };

    Ok(Expr::FunctionCall {
        name: func_name.to_string(),
        args: vec![arg],
    })
}

fn convert_macro_expr(expr_macro: &syn::ExprMacro) -> Result<Expr> {
    let macro_name = expr_macro
        .mac
        .path
        .segments
        .last()
        .ok_or_else(|| Error::Validation("Empty macro path".to_string()))?
        .ident
        .to_string();

    let tokens = expr_macro.mac.tokens.clone();

    match macro_name.as_str() {
        "format" => convert_macro_expr_format(tokens),
        "vec" => convert_macro_expr_vec(tokens),
        "println" | "eprintln" | "print" => convert_macro_expr_print(tokens, &macro_name),
        _ => Err(Error::Validation(format!(
            "Unsupported macro expression: {macro_name}!"
        ))),
    }
}

fn convert_literal_expr(expr_lit: &syn::ExprLit) -> Result<Expr> {
    let literal = convert_literal(&expr_lit.lit)?;
    Ok(Expr::Literal(literal))
}

fn convert_path_expr(expr_path: &syn::ExprPath) -> Result<Expr> {
    let name = expr_path
        .path
        .segments
        .iter()
        .map(|seg| seg.ident.to_string())
        .collect::<Vec<_>>()
        .join("::");
    Ok(Expr::Variable(name))
}

fn convert_call_expr(expr_call: &syn::ExprCall) -> Result<Expr> {
    let SynExpr::Path(path) = &*expr_call.func else {
        return Err(Error::Validation(
            "Complex function calls not supported".to_string(),
        ));
    };

    let name = path
        .path
        .segments
        .iter()
        .map(|seg| seg.ident.to_string())
        .collect::<Vec<_>>()
        .join("::");

    let mut args = Vec::new();
    for arg in &expr_call.args {
        args.push(convert_expr(arg)?);
    }

    Ok(Expr::FunctionCall { name, args })
}

fn convert_binary_expr(expr_binary: &syn::ExprBinary) -> Result<Expr> {
    let left = Box::new(convert_expr(&expr_binary.left)?);
    let right = Box::new(convert_expr(&expr_binary.right)?);
    let op = convert_binary_op(&expr_binary.op)?;
    Ok(Expr::Binary { op, left, right })
}

/// Try to simplify `-<int_literal>` to a negative literal directly.
fn try_negate_int_literal(expr: &SynExpr) -> Option<Result<Expr>> {
    let SynExpr::Lit(lit_expr) = expr else {
        return None;
    };
    let Lit::Int(lit_int) = &lit_expr.lit else {
        return None;
    };
    let lit_str = lit_int.to_string();
    if lit_str == "2147483648" {
        return Some(Ok(Expr::Literal(Literal::I32(i32::MIN))));
    }
    let value: i32 = match lit_int.base10_parse() {
        Ok(v) => v,
        Err(_) => {
            return Some(Err(Error::Validation(
                "Invalid integer literal".to_string(),
            )))
        }
    };
    Some(Ok(Expr::Literal(Literal::I32(-value))))
}

fn convert_unary_expr(expr_unary: &syn::ExprUnary) -> Result<Expr> {
    if let UnOp::Deref(_) = &expr_unary.op {
        return convert_expr(&expr_unary.expr);
    }

    if let UnOp::Neg(_) = &expr_unary.op {
        if let Some(result) = try_negate_int_literal(&expr_unary.expr) {
            return result;
        }
    }

    let operand = Box::new(convert_expr(&expr_unary.expr)?);
    let op = convert_unary_op(&expr_unary.op)?;
    Ok(Expr::Unary { op, operand })
}

fn convert_method_call_expr(method_call: &syn::ExprMethodCall) -> Result<Expr> {
    let receiver = Box::new(convert_expr(&method_call.receiver)?);
    let method = method_call.method.to_string();

    // Special case: std::env::args().collect() → PositionalArgs
    if method == "collect" && method_call.args.is_empty() {
        if let Expr::FunctionCall { name, args } = &*receiver {
            if name == "std::env::args" && args.is_empty() {
                return Ok(Expr::PositionalArgs);
            }
        }
    }

    // General case: regular method call
    let mut args = Vec::new();
    for arg in &method_call.args {
        args.push(convert_expr(arg)?);
    }

    Ok(Expr::MethodCall {
        receiver,
        method,
        args,
    })
}

fn convert_return_expr(ret_expr: &syn::ExprReturn) -> Result<Expr> {
    if let Some(expr) = &ret_expr.expr {
        convert_expr(expr)
    } else {
        Ok(Expr::Literal(crate::ast::restricted::Literal::Str(
            "()".to_string(),
        )))
    }
}

fn convert_literal(lit: &Lit) -> Result<Literal> {
    match lit {
        Lit::Bool(lit_bool) => Ok(Literal::Bool(lit_bool.value)),
        Lit::Int(lit_int) => {
            let suffix = lit_int.suffix();
            if suffix == "u16" {
                let value: u16 = lit_int
                    .base10_parse()
                    .map_err(|_| Error::Validation("Invalid u16 literal".to_string()))?;
                Ok(Literal::U16(value))
            } else if let Ok(value) = lit_int.base10_parse::<u32>() {
                Ok(Literal::U32(value))
            } else {
                // Fallback to i32 for negative literals (e.g., -1 in match patterns)
                let value: i32 = lit_int
                    .base10_parse()
                    .map_err(|_| Error::Validation("Invalid integer literal".to_string()))?;
                Ok(Literal::I32(value))
            }
        }
        Lit::Str(lit_str) => Ok(Literal::Str(lit_str.value())),
        _ => Err(Error::Validation("Unsupported literal type".to_string())),
    }
}

fn convert_binary_op(op: &BinOp) -> Result<BinaryOp> {
    match op {
        BinOp::Add(_) => Ok(BinaryOp::Add),
        BinOp::Sub(_) => Ok(BinaryOp::Sub),
        BinOp::Mul(_) => Ok(BinaryOp::Mul),
        BinOp::Div(_) => Ok(BinaryOp::Div),
        BinOp::Rem(_) => Ok(BinaryOp::Rem),
        BinOp::Eq(_) => Ok(BinaryOp::Eq),
        BinOp::Ne(_) => Ok(BinaryOp::Ne),
        BinOp::Lt(_) => Ok(BinaryOp::Lt),
        BinOp::Le(_) => Ok(BinaryOp::Le),
        BinOp::Gt(_) => Ok(BinaryOp::Gt),
        BinOp::Ge(_) => Ok(BinaryOp::Ge),
        BinOp::And(_) => Ok(BinaryOp::And),
        BinOp::Or(_) => Ok(BinaryOp::Or),
        BinOp::BitAnd(_) => Ok(BinaryOp::BitAnd),
        BinOp::BitOr(_) => Ok(BinaryOp::BitOr),
        BinOp::BitXor(_) => Ok(BinaryOp::BitXor),
        BinOp::Shl(_) => Ok(BinaryOp::Shl),
        BinOp::Shr(_) => Ok(BinaryOp::Shr),
        _ => Err(Error::Validation("Unsupported binary operator".to_string())),
    }
}

fn convert_unary_op(op: &UnOp) -> Result<UnaryOp> {
    match op {
        UnOp::Not(_) => Ok(UnaryOp::Not),
        UnOp::Neg(_) => Ok(UnaryOp::Neg),
        _ => Err(Error::Validation("Unsupported unary operator".to_string())),
    }
}

fn convert_range_expr(range_expr: &syn::ExprRange) -> Result<Expr> {
    let start = range_expr
        .start
        .as_ref()
        .ok_or_else(|| Error::Validation("Range must have start".to_string()))?;
    let end = range_expr
        .end
        .as_ref()
        .ok_or_else(|| Error::Validation("Range must have end".to_string()))?;

    let inclusive = matches!(range_expr.limits, syn::RangeLimits::Closed(_));

    Ok(Expr::Range {
        start: Box::new(convert_expr(start)?),
        end: Box::new(convert_expr(end)?),
        inclusive,
    })
}

fn convert_for_loop(for_loop: &syn::ExprForLoop) -> Result<Stmt> {
    // Extract pattern (e.g., "i" in "for i in..." or "_" in "for _ in...")
    let pattern = match &*for_loop.pat {
        Pat::Ident(pat_ident) => Pattern::Variable(pat_ident.ident.to_string()),
        Pat::Wild(_) => Pattern::Variable("_unused_".to_string()),
        _ => {
            return Err(Error::Validation(
                "Complex patterns in for loops not supported".to_string(),
            ));
        }
    };

    // Convert iterator expression (e.g., 0..3)
    let iter = convert_expr(&for_loop.expr)?;

    // Convert body
    let body = convert_block(&for_loop.body)?;

    Ok(Stmt::For {
        pattern,
        iter,
        body,
        max_iterations: Some(1000), // Default safety limit
    })
}

fn convert_while_loop(expr_while: &syn::ExprWhile) -> Result<Stmt> {
    // Convert condition expression
    let condition = convert_expr(&expr_while.cond)?;

    // Convert body
    let body = convert_block(&expr_while.body)?;

    Ok(Stmt::While {
        condition,
        body,
        max_iterations: Some(10000), // Safety limit for infinite loops
    })
}

fn convert_match_stmt(expr_match: &syn::ExprMatch) -> Result<Stmt> {
    // Convert the scrutinee (the expression being matched)
    let scrutinee = convert_expr(&expr_match.expr)?;

    // Convert each match arm
    let mut arms = Vec::new();
    for arm in &expr_match.arms {
        let pattern = convert_pattern(&arm.pat)?;
        let guard = if let Some((_, guard_expr)) = &arm.guard {
            Some(convert_expr(guard_expr)?)
        } else {
            None
        };

        // Convert the arm body (use convert_expr_stmt for statement-like expressions
        // like nested match, if, while, loop, and macros like println!)
        let body = match &*arm.body {
            SynExpr::Block(block) => convert_block(&block.block)?,
            other => vec![convert_expr_stmt(other)?],
        };

        arms.push(MatchArm {
            pattern,
            guard,
            body,
        });
    }

    Ok(Stmt::Match { scrutinee, arms })
}

/// Extract inner binding from a TupleStruct pattern's first element, or Wildcard.
fn extract_inner_binding(tuple_struct: &syn::PatTupleStruct) -> Pattern {
    if let Some(Pat::Ident(ident)) = tuple_struct.elems.first() {
        Pattern::Variable(ident.ident.to_string())
    } else {
        Pattern::Wildcard
    }
}

fn convert_tuple_struct_pattern(tuple_struct: &syn::PatTupleStruct) -> Result<Pattern> {
    let path_str = tuple_struct
        .path
        .segments
        .iter()
        .map(|seg| seg.ident.to_string())
        .collect::<Vec<_>>()
        .join("::");

    match path_str.as_str() {
        "Some" | "Ok" | "Err" => Ok(extract_inner_binding(tuple_struct)),
        "None" => Ok(Pattern::Literal(Literal::Str(String::new()))),
        _ => Ok(Pattern::Literal(Literal::Str(path_str))),
    }
}

fn convert_range_pattern(range_pat: &syn::PatRange) -> Result<Pattern> {
    let start = range_pat
        .start
        .as_ref()
        .ok_or_else(|| Error::Validation("Range pattern must have start".to_string()))?;
    let end = range_pat
        .end
        .as_ref()
        .ok_or_else(|| Error::Validation("Range pattern must have end".to_string()))?;
    let start_lit = extract_pattern_literal(start)?;
    let end_lit = extract_pattern_literal(end)?;
    let inclusive = matches!(range_pat.limits, syn::RangeLimits::Closed(_));
    Ok(Pattern::Range {
        start: start_lit,
        end: end_lit,
        inclusive,
    })
}

fn convert_pattern(pat: &Pat) -> Result<Pattern> {
    match pat {
        Pat::Lit(lit_pat) => {
            let literal = convert_literal(&lit_pat.lit)?;
            Ok(Pattern::Literal(literal))
        }
        Pat::Ident(ident_pat) => {
            let name = ident_pat.ident.to_string();
            if name == "_" {
                Ok(Pattern::Wildcard)
            } else {
                Ok(Pattern::Variable(name))
            }
        }
        Pat::Wild(_) => Ok(Pattern::Wildcard),
        Pat::TupleStruct(tuple_struct) => convert_tuple_struct_pattern(tuple_struct),
        Pat::Path(pat_path) => {
            let path_str = pat_path
                .path
                .segments
                .iter()
                .map(|seg| seg.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");
            match path_str.as_str() {
                "None" => Ok(Pattern::Literal(Literal::Str(String::new()))),
                _ => Ok(Pattern::Literal(Literal::Str(path_str))),
            }
        }
        Pat::Range(range_pat) => convert_range_pattern(range_pat),
        _ => Err(Error::Validation(format!(
            "Unsupported pattern type: {:?}",
            pat
        ))),
    }
}

/// Extract a Literal from an expression used as a range pattern bound.
fn extract_pattern_literal(expr: &SynExpr) -> Result<Literal> {
    match expr {
        SynExpr::Lit(expr_lit) => convert_literal(&expr_lit.lit),
        SynExpr::Unary(expr_unary) if matches!(expr_unary.op, UnOp::Neg(_)) => {
            if let SynExpr::Lit(lit_expr) = &*expr_unary.expr {
                if let Lit::Int(lit_int) = &lit_expr.lit {
                    let value: i32 = lit_int.base10_parse().map_err(|_| {
                        Error::Validation("Invalid integer in range pattern".to_string())
                    })?;
                    Ok(Literal::I32(-value))
                } else {
                    Err(Error::Validation(
                        "Range pattern bounds must be literals".to_string(),
                    ))
                }
            } else {
                Err(Error::Validation(
                    "Range pattern bounds must be literals".to_string(),
                ))
            }
        }
        _ => Err(Error::Validation(
            "Range pattern bounds must be literals".to_string(),
        )),
    }
}

#[cfg(test)]
#[path = "parser_tests_extracted.rs"]
mod tests_extracted;
