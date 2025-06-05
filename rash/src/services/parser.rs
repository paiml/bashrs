use crate::ast::restricted::{
    BinaryOp, Expr, Function, Literal, Parameter, RestrictedAst, Stmt, Type, UnaryOp,
};
use crate::models::{Error, Result};
use syn::{
    BinOp, Block, Expr as SynExpr, File, FnArg, Item, ItemFn, Lit, Pat, ReturnType,
    Stmt as SynStmt, Type as SynType, UnOp,
};

/// Parse Rust source code into a RestrictedAst
pub fn parse(input: &str) -> Result<RestrictedAst> {
    let file: File = syn::parse_str(input)?;

    let mut functions = Vec::new();
    let mut entry_point = None;

    for item in file.items {
        match item {
            Item::Fn(item_fn) => {
                // Check if this is the main function marked with #[rash::main]
                let is_main = item_fn.attrs.iter().any(|_attr| {
                    // For now, accept any attribute as #[rash::main]
                    // TODO: Proper attribute parsing
                    true
                }) || item_fn.sig.ident == "main";

                let function = convert_function(item_fn)?;

                if is_main {
                    if entry_point.is_some() {
                        return Err(Error::Validation(
                            "Multiple #[rash::main] functions found".to_string(),
                        ));
                    }
                    entry_point = Some(function.name.clone());
                }

                functions.push(function);
            }
            _ => {
                return Err(Error::Validation(
                    "Only functions are allowed in Rash code".to_string(),
                ));
            }
        }
    }

    let entry_point = entry_point
        .ok_or_else(|| Error::Validation("No #[rash::main] function found".to_string()))?;

    Ok(RestrictedAst {
        functions,
        entry_point,
    })
}

fn convert_function(item_fn: ItemFn) -> Result<Function> {
    let name = item_fn.sig.ident.to_string();

    // Convert parameters
    let mut params = Vec::new();
    for input in item_fn.sig.inputs {
        match input {
            FnArg::Typed(pat_type) => {
                if let Pat::Ident(pat_ident) = &*pat_type.pat {
                    let param_name = pat_ident.ident.to_string();
                    let param_type = convert_type(&pat_type.ty)?;
                    params.push(Parameter {
                        name: param_name,
                        param_type,
                    });
                } else {
                    return Err(Error::Validation(
                        "Complex parameter patterns not supported".to_string(),
                    ));
                }
            }
            FnArg::Receiver(_) => {
                return Err(Error::Validation("Self parameters not allowed".to_string()));
            }
        }
    }

    // Convert return type
    let return_type = match &item_fn.sig.output {
        ReturnType::Default => Type::Str, // Default to unit type represented as string
        ReturnType::Type(_, ty) => convert_type(ty)?,
    };

    // Convert function body
    let body = convert_block(&item_fn.block)?;

    Ok(Function {
        name,
        params,
        return_type,
        body,
    })
}

fn convert_type(ty: &SynType) -> Result<Type> {
    match ty {
        SynType::Path(type_path) => {
            let path_str = type_path
                .path
                .segments
                .iter()
                .map(|seg| seg.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");

            match path_str.as_str() {
                "bool" => Ok(Type::Bool),
                "u32" | "i32" => Ok(Type::U32), // Treat i32 as u32 for now
                "str" | "String" => Ok(Type::Str),
                path if path.starts_with("Result") => {
                    // Simple Result type parsing - assumes Result<T, E>
                    Ok(Type::Result {
                        ok_type: Box::new(Type::Str), // Simplified
                        err_type: Box::new(Type::Str),
                    })
                }
                path if path.starts_with("Option") => {
                    // Simple Option type parsing - assumes Option<T>
                    Ok(Type::Option {
                        inner_type: Box::new(Type::Str), // Simplified
                    })
                }
                _ => Err(Error::Validation(format!("Unsupported type: {path_str}"))),
            }
        }
        SynType::Reference(type_ref) => {
            // Handle &str and other reference types
            if let SynType::Path(path) = &*type_ref.elem {
                let path_str = path
                    .path
                    .segments
                    .iter()
                    .map(|seg| seg.ident.to_string())
                    .collect::<Vec<_>>()
                    .join("::");

                match path_str.as_str() {
                    "str" => Ok(Type::Str),
                    _ => Err(Error::Validation(format!(
                        "Unsupported reference type: &{path_str}"
                    ))),
                }
            } else {
                Err(Error::Validation(
                    "Complex reference types not supported".to_string(),
                ))
            }
        }
        _ => Err(Error::Validation("Complex types not supported".to_string())),
    }
}

fn convert_block(block: &Block) -> Result<Vec<Stmt>> {
    let mut statements = Vec::new();

    for stmt in &block.stmts {
        statements.push(convert_stmt(stmt)?);
    }

    Ok(statements)
}

fn convert_stmt(stmt: &SynStmt) -> Result<Stmt> {
    match stmt {
        SynStmt::Local(local) => {
            if let Pat::Ident(pat_ident) = &local.pat {
                let name = pat_ident.ident.to_string();
                if let Some(init) = &local.init {
                    let value = convert_expr(&init.expr)?;
                    Ok(Stmt::Let { name, value })
                } else {
                    Err(Error::Validation(
                        "Let bindings must have initializers".to_string(),
                    ))
                }
            } else {
                Err(Error::Validation(
                    "Complex patterns not supported".to_string(),
                ))
            }
        }
        SynStmt::Expr(expr, _) => {
            // Check if this is an if expression used as a statement
            if let SynExpr::If(expr_if) = expr {
                let condition = convert_expr(&expr_if.cond)?;
                let then_block = convert_block(&expr_if.then_branch)?;
                let else_block = if let Some((_, else_expr)) = &expr_if.else_branch {
                    match &**else_expr {
                        SynExpr::Block(block) => Some(convert_block(&block.block)?),
                        SynExpr::If(_) => {
                            // Handle else-if by converting to nested if statement
                            Some(vec![Stmt::Expr(convert_expr(else_expr)?)])
                        }
                        _ => None,
                    }
                } else {
                    None
                };
                Ok(Stmt::If {
                    condition,
                    then_block,
                    else_block,
                })
            } else {
                Ok(Stmt::Expr(convert_expr(expr)?))
            }
        }
        _ => Err(Error::Validation("Unsupported statement type".to_string())),
    }
}

fn convert_expr(expr: &SynExpr) -> Result<Expr> {
    match expr {
        SynExpr::Lit(expr_lit) => {
            let literal = convert_literal(&expr_lit.lit)?;
            Ok(Expr::Literal(literal))
        }
        SynExpr::Path(expr_path) => {
            let name = expr_path
                .path
                .segments
                .iter()
                .map(|seg| seg.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");
            Ok(Expr::Variable(name))
        }
        SynExpr::Call(expr_call) => {
            if let SynExpr::Path(path) = &*expr_call.func {
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
            } else {
                Err(Error::Validation(
                    "Complex function calls not supported".to_string(),
                ))
            }
        }
        SynExpr::Binary(expr_binary) => {
            let left = Box::new(convert_expr(&expr_binary.left)?);
            let right = Box::new(convert_expr(&expr_binary.right)?);
            let op = convert_binary_op(&expr_binary.op)?;
            Ok(Expr::Binary { op, left, right })
        }
        SynExpr::Unary(expr_unary) => {
            let operand = Box::new(convert_expr(&expr_unary.expr)?);
            let op = convert_unary_op(&expr_unary.op)?;
            Ok(Expr::Unary { op, operand })
        }
        SynExpr::MethodCall(method_call) => {
            let receiver = Box::new(convert_expr(&method_call.receiver)?);
            let method = method_call.method.to_string();
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
        SynExpr::Return(ret_expr) => {
            if let Some(expr) = &ret_expr.expr {
                convert_expr(expr)
            } else {
                Ok(Expr::Literal(crate::ast::restricted::Literal::Str(
                    "()".to_string(),
                )))
            }
        }
        SynExpr::Paren(expr_paren) => {
            // Handle parenthesized expressions by unwrapping them
            convert_expr(&expr_paren.expr)
        }
        SynExpr::If(_) => {
            // For now, reject if expressions in expression position
            // They should be used as statements instead
            Err(Error::Validation(
                "If expressions not supported in expression position".to_string(),
            ))
        }
        _ => Err(Error::Validation("Unsupported expression type".to_string())),
    }
}

fn convert_literal(lit: &Lit) -> Result<Literal> {
    match lit {
        Lit::Bool(lit_bool) => Ok(Literal::Bool(lit_bool.value)),
        Lit::Int(lit_int) => {
            let value: u32 = lit_int
                .base10_parse()
                .map_err(|_| Error::Validation("Invalid integer literal".to_string()))?;
            Ok(Literal::U32(value))
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
        BinOp::Eq(_) => Ok(BinaryOp::Eq),
        BinOp::Ne(_) => Ok(BinaryOp::Ne),
        BinOp::Lt(_) => Ok(BinaryOp::Lt),
        BinOp::Le(_) => Ok(BinaryOp::Le),
        BinOp::Gt(_) => Ok(BinaryOp::Gt),
        BinOp::Ge(_) => Ok(BinaryOp::Ge),
        BinOp::And(_) => Ok(BinaryOp::And),
        BinOp::Or(_) => Ok(BinaryOp::Or),
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
