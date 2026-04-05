/// Convert a multi-arg println!/eprintln!/print! into a format_concat expression.
/// `parsed_args` must have len > 1.
fn convert_print_format_args(
    parsed_args: &syn::punctuated::Punctuated<syn::Expr, syn::Token![,]>,
    macro_name: &str,
) -> Result<Expr> {
    let first_arg = &parsed_args[0];

    let fmt_value = if let SynExpr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(lit_str),
        ..
    }) = first_arg
    {
        lit_str.value()
    } else {
        return Ok(Expr::FunctionCall {
            name: print_macro_func_name(macro_name).to_string(),
            args: vec![convert_expr(first_arg)?],
        });
    };

    let mut expr_args = Vec::new();
    for arg_expr in parsed_args.iter().skip(1) {
        expr_args.push(convert_expr(arg_expr)?);
    }

    let segments = parse_format_string(&fmt_value);
    Ok(build_format_concat(&segments, &expr_args))
}

fn convert_macro_stmt(macro_stmt: &syn::StmtMacro) -> Result<Stmt> {
    let macro_path = &macro_stmt.mac.path;
    let macro_name = macro_path
        .segments
        .last()
        .ok_or_else(|| Error::Validation("Empty macro path".to_string()))?
        .ident
        .to_string();

    if macro_name != "println" && macro_name != "eprintln" && macro_name != "print" {
        return Err(Error::Validation(format!(
            "Unsupported macro: {macro_name}!"
        )));
    }

    let tokens = macro_stmt.mac.tokens.clone();
    let parsed_args: syn::punctuated::Punctuated<syn::Expr, syn::Token![,]> =
        syn::punctuated::Punctuated::parse_terminated
            .parse2(tokens.clone())
            .map_err(|_| Error::Validation(format!("Invalid {macro_name}! arguments")))?;

    let arg = if parsed_args.len() <= 1 {
        let parsed: syn::Expr = syn::parse2(tokens)
            .map_err(|_| Error::Validation(format!("Invalid {macro_name}! arguments")))?;
        convert_expr(&parsed)?
    } else {
        convert_print_format_args(&parsed_args, &macro_name)?
    };

    let func_name = print_macro_func_name(&macro_name);
    Ok(Stmt::Expr(Expr::FunctionCall {
        name: func_name.to_string(),
        args: vec![arg],
    }))
}

/// Convert an if-expression used in expression position (e.g., `let x = if c { a } else { b }`).
/// For simple single-expression branches: __if_expr(cond, then_val, else_val).
/// For multi-statement branches: Expr::Block([Stmt::If{...}]) to preserve all statements.
/// Check if an if-expression has any multi-statement branches.
fn has_multi_stmt_branch(expr_if: &syn::ExprIf) -> bool {
    if expr_if.then_branch.stmts.len() > 1 {
        return true;
    }
    if let Some((_, else_expr)) = &expr_if.else_branch {
        if let SynExpr::Block(block) = &**else_expr {
            return block.block.stmts.len() > 1;
        }
    }
    false
}

/// Extract the value from an else branch expression.
fn extract_else_value(else_expr: &SynExpr) -> Result<Expr> {
    match else_expr {
        SynExpr::Block(block) => extract_branch_value(&block.block),
        SynExpr::If(nested_if) => convert_if_expr(nested_if),
        other => convert_expr(other),
    }
}

fn convert_if_expr(expr_if: &syn::ExprIf) -> Result<Expr> {
    if has_multi_stmt_branch(expr_if) {
        let if_stmt = convert_if_stmt(expr_if)?;
        return Ok(Expr::Block(vec![if_stmt]));
    }

    let condition = convert_expr(&expr_if.cond)?;
    let then_value = extract_branch_value(&expr_if.then_branch)?;

    let else_value = if let Some((_, else_expr)) = &expr_if.else_branch {
        extract_else_value(else_expr)?
    } else {
        Expr::Literal(Literal::Str(String::new()))
    };

    Ok(Expr::FunctionCall {
        name: "__if_expr".to_string(),
        args: vec![condition, then_value, else_value],
    })
}

/// Extract the last expression from a block as the block's value.
fn extract_branch_value(block: &syn::Block) -> Result<Expr> {
    if let Some(last_stmt) = block.stmts.last() {
        match last_stmt {
            SynStmt::Expr(expr, None) => convert_expr(expr),
            SynStmt::Expr(expr, Some(_)) => convert_expr(expr),
            _ => Ok(Expr::Literal(Literal::Str(String::new()))),
        }
    } else {
        Ok(Expr::Literal(Literal::Str(String::new())))
    }
}

fn convert_if_stmt(expr_if: &syn::ExprIf) -> Result<Stmt> {
    let condition = convert_expr(&expr_if.cond)?;
    let then_block = convert_block(&expr_if.then_branch)?;
    let else_block = convert_else_block(&expr_if.else_branch)?;

    Ok(Stmt::If {
        condition,
        then_block,
        else_block,
    })
}

fn convert_else_block(
    else_branch: &Option<(syn::token::Else, Box<SynExpr>)>,
) -> Result<Option<Vec<Stmt>>> {
    let Some((_, else_expr)) = else_branch else {
        return Ok(None);
    };

    match &**else_expr {
        SynExpr::Block(block) => Ok(Some(convert_block(&block.block)?)),
        SynExpr::If(nested_if) => convert_else_if(nested_if),
        _ => Ok(None),
    }
}

fn convert_else_if(nested_if: &syn::ExprIf) -> Result<Option<Vec<Stmt>>> {
    let nested_condition = convert_expr(&nested_if.cond)?;
    let nested_then = convert_block(&nested_if.then_branch)?;
    let nested_else = convert_nested_else(&nested_if.else_branch)?;

    Ok(Some(vec![Stmt::If {
        condition: nested_condition,
        then_block: nested_then,
        else_block: nested_else,
    }]))
}

fn convert_nested_else(
    else_branch: &Option<(syn::token::Else, Box<SynExpr>)>,
) -> Result<Option<Vec<Stmt>>> {
    let Some((_, nested_else_expr)) = else_branch else {
        return Ok(None);
    };

    match &**nested_else_expr {
        SynExpr::Block(block) => Ok(Some(convert_block(&block.block)?)),
        SynExpr::If(_) => {
            // Recursively handle else-if-else-if chains
            let stmt = SynStmt::Expr((**nested_else_expr).clone(), None);
            Ok(Some(vec![convert_stmt(&stmt)?]))
        }
        _ => Ok(None),
    }
}

fn convert_repeat_expr(expr_repeat: &syn::ExprRepeat) -> Result<Expr> {
    let value = convert_expr(&expr_repeat.expr)?;
    let count = if let SynExpr::Lit(lit) = &*expr_repeat.len {
        if let syn::Lit::Int(lit_int) = &lit.lit {
            lit_int.base10_digits().parse::<usize>().unwrap_or(1)
        } else {
            1
        }
    } else {
        1
    };
    Ok(Expr::Array(vec![value; count]))
}

fn convert_block_expr(expr_block: &syn::ExprBlock) -> Result<Expr> {
    let mut stmts = Vec::new();
    for stmt in &expr_block.block.stmts {
        stmts.push(convert_stmt(stmt)?);
    }
    Ok(Expr::Block(stmts))
}

fn convert_let_expr(expr_let: &syn::ExprLet) -> Result<Expr> {
    let rhs = convert_expr(&expr_let.expr)?;
    match &*expr_let.pat {
        Pat::Lit(lit_pat) => {
            let lhs = convert_literal_expr(&syn::ExprLit {
                attrs: vec![],
                lit: lit_pat.lit.clone(),
            })?;
            Ok(Expr::Binary {
                op: BinaryOp::Eq,
                left: Box::new(rhs),
                right: Box::new(lhs),
            })
        }
        _ => Ok(rhs),
    }
}

fn convert_struct_expr(expr_struct: &syn::ExprStruct) -> Result<Expr> {
    let mut values = Vec::new();
    for field in &expr_struct.fields {
        values.push(convert_expr(&field.expr)?);
    }
    Ok(Expr::Array(values))
}

fn convert_tuple_expr(expr_tuple: &syn::ExprTuple) -> Result<Expr> {
    let mut elements = Vec::new();
    for elem in &expr_tuple.elems {
        elements.push(convert_expr(elem)?);
    }
    Ok(Expr::Array(elements))
}

fn convert_expr(expr: &SynExpr) -> Result<Expr> {
    match expr {
        SynExpr::Lit(expr_lit) => convert_literal_expr(expr_lit),
        SynExpr::Path(expr_path) => convert_path_expr(expr_path),
        SynExpr::Call(expr_call) => convert_call_expr(expr_call),
        SynExpr::Binary(expr_binary) => convert_binary_expr(expr_binary),
        SynExpr::Unary(expr_unary) => convert_unary_expr(expr_unary),
        SynExpr::MethodCall(method_call) => convert_method_call_expr(method_call),
        SynExpr::Return(ret_expr) => convert_return_expr(ret_expr),
        SynExpr::Paren(expr_paren) => convert_expr(&expr_paren.expr),
        SynExpr::Array(expr_array) => convert_array_expr(expr_array),
        SynExpr::Reference(expr_ref) => convert_reference_expr(expr_ref),
        SynExpr::If(expr_if) => convert_if_expr(expr_if),
        SynExpr::Range(range_expr) => convert_range_expr(range_expr),
        SynExpr::Macro(expr_macro) => convert_macro_expr(expr_macro),
        SynExpr::Index(expr_index) => {
            let object = convert_expr(&expr_index.expr)?;
            let index = convert_expr(&expr_index.index)?;
            Ok(Expr::Index {
                object: Box::new(object),
                index: Box::new(index),
            })
        }
        SynExpr::Repeat(expr_repeat) => convert_repeat_expr(expr_repeat),
        SynExpr::Block(expr_block) => convert_block_expr(expr_block),
        SynExpr::Cast(expr_cast) => convert_expr(&expr_cast.expr),
        SynExpr::Match(expr_match) => {
            let match_stmt = convert_match_stmt(expr_match)?;
            Ok(Expr::Block(vec![match_stmt]))
        }
        SynExpr::Tuple(expr_tuple) => convert_tuple_expr(expr_tuple),
        SynExpr::Closure(expr_closure) => convert_expr(&expr_closure.body),
        SynExpr::Let(expr_let) => convert_let_expr(expr_let),
        SynExpr::Struct(expr_struct) => convert_struct_expr(expr_struct),
        SynExpr::Field(expr_field) => {
            let object = convert_expr(&expr_field.base)?;
            Ok(Expr::Index {
                object: Box::new(object),
                index: Box::new(Expr::Literal(Literal::I32(0))),
            })
        }
        _ => Err(Error::Validation("Unsupported expression type".to_string())),
    }
}

fn convert_array_expr(expr_array: &syn::ExprArray) -> Result<Expr> {
    let mut elements = Vec::new();
    for elem in &expr_array.elems {
        elements.push(convert_expr(elem)?);
    }
    Ok(Expr::Array(elements))
}

fn convert_reference_expr(expr_ref: &syn::ExprReference) -> Result<Expr> {
    // &[...] -> unwrap reference, convert inner array expression
    // &expr -> unwrap reference, convert inner expression
    convert_expr(&expr_ref.expr)
}

/// Map a print-family macro name to its rash function name.
fn print_macro_func_name(macro_name: &str) -> &'static str {
    match macro_name {
        "eprintln" => "rash_eprintln",
        "print" => "rash_print",
        _ => "rash_println",
    }
}

/// Parse format-string macro args (format!, println!, etc.) into a format_concat expression.
/// `parts` must have len > 1 (format string + at least one arg).
fn parse_format_macro_args(parts: &[String], macro_name: &str) -> Result<Expr> {
    let fmt_str = &parts[0];
    let parsed = syn::parse_str::<syn::Expr>(fmt_str)
        .map_err(|_| Error::Validation(format!("Could not parse {macro_name}! format string")))?;

    let fmt_value = if let SynExpr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(lit_str),
        ..
    }) = &parsed
    {
        lit_str.value()
    } else {
        return convert_expr(&parsed);
    };

    let mut expr_args = Vec::new();
    for part in &parts[1..] {
        let arg = syn::parse_str::<syn::Expr>(part)
            .map_err(|_| Error::Validation(format!("Invalid {macro_name}! argument: {part}")))?;
        expr_args.push(convert_expr(&arg)?);
    }

    let segments = parse_format_string(&fmt_value);
    Ok(build_format_concat(&segments, &expr_args))
}

fn convert_macro_expr_format(tokens: proc_macro2::TokenStream) -> Result<Expr> {
    let token_str = tokens.to_string();
    let parts = split_macro_args(&token_str);

    if parts.len() <= 1 {
        if let Ok(parsed) = syn::parse_str::<syn::Expr>(&token_str) {
            return convert_expr(&parsed);
        }
        return Err(Error::Validation(
            "Could not parse format! arguments".to_string(),
        ));
    }

    parse_format_macro_args(&parts, "format")
}

fn convert_macro_expr_vec(tokens: proc_macro2::TokenStream) -> Result<Expr> {
    let token_str = tokens.to_string();
    let array_str = format!("[{token_str}]");
    if let Ok(parsed) = syn::parse_str::<syn::ExprArray>(&array_str) {
        return convert_array_expr(&parsed);
    }
    Err(Error::Validation(
        "Could not parse vec! arguments".to_string(),
    ))
}


include!("parser_convert.rs");
