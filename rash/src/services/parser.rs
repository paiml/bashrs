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
                // Check if this is the main function marked with #[bashrs::main]
                let is_main = item_fn.attrs.iter().any(|attr| {
                    // Check if the attribute path matches "bashrs::main" or legacy "rash::main"
                    let path = attr.path();
                    path.segments.len() == 2
                        && (path.segments[0].ident == "bashrs" || path.segments[0].ident == "rash")
                        && path.segments[1].ident == "main"
                }) || item_fn.sig.ident == "main";

                let function = convert_function(item_fn)?;

                if is_main {
                    if entry_point.is_some() {
                        return Err(Error::Validation(
                            "Multiple #[bashrs::main] functions found".to_string(),
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
        .ok_or_else(|| Error::Validation("No #[bashrs::main] function found".to_string()))?;

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
        SynStmt::Local(local) => convert_let_stmt(local),
        SynStmt::Expr(expr, _) => convert_expr_stmt(expr),
        _ => Err(Error::Validation("Unsupported statement type".to_string())),
    }
}

fn convert_let_stmt(local: &syn::Local) -> Result<Stmt> {
    let Pat::Ident(pat_ident) = &local.pat else {
        return Err(Error::Validation(
            "Complex patterns not supported".to_string(),
        ));
    };

    let name = pat_ident.ident.to_string();

    let Some(init) = &local.init else {
        return Err(Error::Validation(
            "Let bindings must have initializers".to_string(),
        ));
    };

    let value = convert_expr(&init.expr)?;
    Ok(Stmt::Let { name, value })
}

fn convert_expr_stmt(expr: &SynExpr) -> Result<Stmt> {
    if let SynExpr::If(expr_if) = expr {
        convert_if_stmt(expr_if)
    } else {
        Ok(Stmt::Expr(convert_expr(expr)?))
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

fn convert_else_block(else_branch: &Option<(syn::token::Else, Box<SynExpr>)>) -> Result<Option<Vec<Stmt>>> {
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

fn convert_nested_else(else_branch: &Option<(syn::token::Else, Box<SynExpr>)>) -> Result<Option<Vec<Stmt>>> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_stmt_simple_let_binding() {
        let source = r#"
            fn main() {
                let x = 42;
            }
        "#;
        let ast = parse(source).unwrap();
        assert_eq!(ast.functions.len(), 1);
        assert_eq!(ast.functions[0].body.len(), 1);

        match &ast.functions[0].body[0] {
            Stmt::Let { name, value } => {
                assert_eq!(name, "x");
                assert!(matches!(value, Expr::Literal(Literal::U32(42))));
            }
            _ => panic!("Expected Let statement"),
        }
    }

    #[test]
    fn test_convert_stmt_string_let_binding() {
        let source = r#"
            fn main() {
                let greeting = "Hello, world!";
            }
        "#;
        let ast = parse(source).unwrap();

        match &ast.functions[0].body[0] {
            Stmt::Let { name, value } => {
                assert_eq!(name, "greeting");
                assert!(matches!(value, Expr::Literal(Literal::Str(_))));
            }
            _ => panic!("Expected Let statement"),
        }
    }

    #[test]
    fn test_convert_stmt_let_without_init() {
        let source = r#"
            fn main() {
                let x;
            }
        "#;
        let result = parse(source);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must have initializers"));
    }

    #[test]
    fn test_convert_stmt_simple_if() {
        let source = r#"
            fn main() {
                if true {
                    let x = 1;
                }
            }
        "#;
        let ast = parse(source).unwrap();

        match &ast.functions[0].body[0] {
            Stmt::If { condition, then_block, else_block } => {
                assert!(matches!(condition, Expr::Literal(Literal::Bool(true))));
                assert_eq!(then_block.len(), 1);
                assert!(else_block.is_none());
            }
            _ => panic!("Expected If statement"),
        }
    }

    #[test]
    fn test_convert_stmt_if_else() {
        let source = r#"
            fn main() {
                if true {
                    let x = 1;
                } else {
                    let y = 2;
                }
            }
        "#;
        let ast = parse(source).unwrap();

        match &ast.functions[0].body[0] {
            Stmt::If { condition, then_block, else_block } => {
                assert!(matches!(condition, Expr::Literal(Literal::Bool(true))));
                assert_eq!(then_block.len(), 1);
                assert!(else_block.is_some());
                assert_eq!(else_block.as_ref().unwrap().len(), 1);
            }
            _ => panic!("Expected If statement"),
        }
    }

    #[test]
    fn test_convert_stmt_else_if_chain_two_levels() {
        let source = r#"
            fn main() {
                if true {
                    let x = 1;
                } else if false {
                    let y = 2;
                }
            }
        "#;
        let ast = parse(source).unwrap();

        match &ast.functions[0].body[0] {
            Stmt::If { condition, then_block, else_block } => {
                assert!(matches!(condition, Expr::Literal(Literal::Bool(true))));
                assert_eq!(then_block.len(), 1);

                // Verify else block contains nested if
                assert!(else_block.is_some());
                let else_stmts = else_block.as_ref().unwrap();
                assert_eq!(else_stmts.len(), 1);

                match &else_stmts[0] {
                    Stmt::If { condition: nested_cond, then_block: nested_then, else_block: nested_else } => {
                        assert!(matches!(nested_cond, Expr::Literal(Literal::Bool(false))));
                        assert_eq!(nested_then.len(), 1);
                        assert!(nested_else.is_none());
                    }
                    _ => panic!("Expected nested If statement in else block"),
                }
            }
            _ => panic!("Expected If statement"),
        }
    }

    #[test]
    fn test_convert_stmt_else_if_chain_three_levels() {
        let source = r#"
            fn main() {
                if true {
                    let x = 1;
                } else if false {
                    let y = 2;
                } else if true {
                    let z = 3;
                }
            }
        "#;
        let ast = parse(source).unwrap();

        // Verify first level if
        match &ast.functions[0].body[0] {
            Stmt::If { else_block, .. } => {
                assert!(else_block.is_some());
                let first_else = else_block.as_ref().unwrap();
                assert_eq!(first_else.len(), 1);

                // Verify second level else-if
                match &first_else[0] {
                    Stmt::If { else_block: second_else_block, .. } => {
                        assert!(second_else_block.is_some());
                        let second_else = second_else_block.as_ref().unwrap();
                        assert_eq!(second_else.len(), 1);

                        // Verify third level else-if
                        match &second_else[0] {
                            Stmt::If { condition, else_block: third_else, .. } => {
                                assert!(matches!(condition, Expr::Literal(Literal::Bool(true))));
                                assert!(third_else.is_none());
                            }
                            _ => panic!("Expected third-level If statement"),
                        }
                    }
                    _ => panic!("Expected second-level If statement"),
                }
            }
            _ => panic!("Expected If statement"),
        }
    }

    #[test]
    fn test_convert_stmt_else_if_with_final_else() {
        let source = r#"
            fn main() {
                if true {
                    let x = 1;
                } else if false {
                    let y = 2;
                } else {
                    let z = 3;
                }
            }
        "#;
        let ast = parse(source).unwrap();

        match &ast.functions[0].body[0] {
            Stmt::If { else_block, .. } => {
                let first_else = else_block.as_ref().unwrap();

                // Second level should be else-if with an else block
                match &first_else[0] {
                    Stmt::If { else_block: second_else_block, .. } => {
                        assert!(second_else_block.is_some());
                        let final_else = second_else_block.as_ref().unwrap();
                        assert_eq!(final_else.len(), 1);

                        // Final else should contain a Let statement, not another If
                        assert!(matches!(final_else[0], Stmt::Let { .. }));
                    }
                    _ => panic!("Expected second-level If statement"),
                }
            }
            _ => panic!("Expected If statement"),
        }
    }

    #[test]
    fn test_convert_stmt_expr_call() {
        let source = r#"
            fn main() {
                echo("test");
            }
        "#;
        let ast = parse(source).unwrap();

        match &ast.functions[0].body[0] {
            Stmt::Expr(expr) => {
                assert!(matches!(expr, Expr::FunctionCall { .. }));
            }
            _ => panic!("Expected Expr statement"),
        }
    }

    #[test]
    fn test_convert_stmt_unsupported_type() {
        let source = r#"
            fn main() {
                loop { }
            }
        "#;
        let result = parse(source);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        // Loop statements are unsupported and produce validation error
        assert!(err_msg.contains("Unsupported") || err_msg.contains("loop"));
    }
}
