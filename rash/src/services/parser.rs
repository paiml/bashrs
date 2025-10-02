use crate::ast::restricted::{
    BinaryOp, Expr, Function, Literal, Parameter, Pattern, RestrictedAst, Stmt, Type, UnaryOp,
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
        process_item(item, &mut functions, &mut entry_point)?;
    }

    let entry_point = entry_point
        .ok_or_else(|| Error::Validation("No #[bashrs::main] function found".to_string()))?;

    Ok(RestrictedAst {
        functions,
        entry_point,
    })
}

fn process_item(
    item: Item,
    functions: &mut Vec<Function>,
    entry_point: &mut Option<String>,
) -> Result<()> {
    let Item::Fn(item_fn) = item else {
        return Err(Error::Validation(
            "Only functions are allowed in Rash code".to_string(),
        ));
    };

    let is_main = has_main_attribute(&item_fn) || item_fn.sig.ident == "main";
    let function = convert_function(item_fn)?;

    if is_main {
        check_single_entry_point(entry_point, &function.name)?;
        *entry_point = Some(function.name.clone());
    }

    functions.push(function);
    Ok(())
}

fn has_main_attribute(item_fn: &ItemFn) -> bool {
    item_fn.attrs.iter().any(is_main_attribute)
}

fn is_main_attribute(attr: &syn::Attribute) -> bool {
    let path = attr.path();
    path.segments.len() == 2
        && (path.segments[0].ident == "bashrs" || path.segments[0].ident == "rash")
        && path.segments[1].ident == "main"
}

fn check_single_entry_point(current: &Option<String>, _new_name: &str) -> Result<()> {
    if current.is_some() {
        return Err(Error::Validation(
            "Multiple #[bashrs::main] functions found".to_string(),
        ));
    }
    Ok(())
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
        SynStmt::Macro(macro_stmt) => convert_macro_stmt(macro_stmt),
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
    match expr {
        SynExpr::If(expr_if) => convert_if_stmt(expr_if),
        SynExpr::ForLoop(for_loop) => convert_for_loop(for_loop),
        _ => Ok(Stmt::Expr(convert_expr(expr)?)),
    }
}

fn convert_macro_stmt(macro_stmt: &syn::StmtMacro) -> Result<Stmt> {
    let macro_path = &macro_stmt.mac.path;
    let macro_name = macro_path
        .segments
        .last()
        .ok_or_else(|| Error::Validation("Empty macro path".to_string()))?
        .ident
        .to_string();

    if macro_name == "println" {
        // Parse macro tokens to extract the format string
        let tokens = macro_stmt.mac.tokens.clone();
        let parsed: syn::Expr = syn::parse2(tokens)
            .map_err(|_| Error::Validation("Invalid println! arguments".to_string()))?;

        // Convert the first argument as the format string
        let arg = convert_expr(&parsed)?;

        // Generate a function call to rash_println
        Ok(Stmt::Expr(Expr::FunctionCall {
            name: "rash_println".to_string(),
            args: vec![arg],
        }))
    } else {
        Err(Error::Validation(format!(
            "Unsupported macro: {macro_name}!"
        )))
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
        SynExpr::Lit(expr_lit) => convert_literal_expr(expr_lit),
        SynExpr::Path(expr_path) => convert_path_expr(expr_path),
        SynExpr::Call(expr_call) => convert_call_expr(expr_call),
        SynExpr::Binary(expr_binary) => convert_binary_expr(expr_binary),
        SynExpr::Unary(expr_unary) => convert_unary_expr(expr_unary),
        SynExpr::MethodCall(method_call) => convert_method_call_expr(method_call),
        SynExpr::Return(ret_expr) => convert_return_expr(ret_expr),
        SynExpr::Paren(expr_paren) => convert_expr(&expr_paren.expr),
        SynExpr::If(_) => Err(Error::Validation(
            "If expressions not supported in expression position".to_string(),
        )),
        SynExpr::Range(range_expr) => convert_range_expr(range_expr),
        _ => Err(Error::Validation("Unsupported expression type".to_string())),
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

fn convert_unary_expr(expr_unary: &syn::ExprUnary) -> Result<Expr> {
    // Special case: negative integer literals (-1, -42, etc.)
    // Simplify UnaryOp(Neg, Literal(n)) to Literal(-n)
    if let UnOp::Neg(_) = &expr_unary.op {
        if let SynExpr::Lit(lit_expr) = &*expr_unary.expr {
            if let Lit::Int(lit_int) = &lit_expr.lit {
                // Parse as i32 instead of u32 for negative numbers
                let value: i32 = lit_int
                    .base10_parse()
                    .map_err(|_| Error::Validation("Invalid integer literal".to_string()))?;
                // Return as negative literal
                return Ok(Expr::Literal(Literal::I32(-value)));
            }
        }
    }

    // General case: convert normally
    let operand = Box::new(convert_expr(&expr_unary.expr)?);
    let op = convert_unary_op(&expr_unary.op)?;
    Ok(Expr::Unary { op, operand })
}

fn convert_method_call_expr(method_call: &syn::ExprMethodCall) -> Result<Expr> {
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
    // Extract pattern (e.g., "i" in "for i in...")
    let Pat::Ident(pat_ident) = &*for_loop.pat else {
        return Err(Error::Validation(
            "Complex patterns in for loops not supported".to_string(),
        ));
    };
    let pattern = Pattern::Variable(pat_ident.ident.to_string());

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

    // Tests for convert_expr function
    #[test]
    fn test_convert_expr_literal_bool() {
        let source = r#"
            fn main() {
                let x = true;
            }
        "#;
        let ast = parse(source).unwrap();
        match &ast.functions[0].body[0] {
            Stmt::Let { value, .. } => {
                assert!(matches!(value, Expr::Literal(Literal::Bool(true))));
            }
            _ => panic!("Expected Let statement"),
        }
    }

    #[test]
    fn test_convert_expr_literal_int() {
        let source = r#"
            fn main() {
                let x = 42;
            }
        "#;
        let ast = parse(source).unwrap();
        match &ast.functions[0].body[0] {
            Stmt::Let { value, .. } => {
                assert!(matches!(value, Expr::Literal(Literal::U32(42))));
            }
            _ => panic!("Expected Let statement"),
        }
    }

    #[test]
    fn test_convert_expr_variable() {
        let source = r#"
            fn main() {
                let x = 42;
                let y = x;
            }
        "#;
        let ast = parse(source).unwrap();
        match &ast.functions[0].body[1] {
            Stmt::Let { value, .. } => {
                match value {
                    Expr::Variable(name) => assert_eq!(name, "x"),
                    _ => panic!("Expected Variable expression"),
                }
            }
            _ => panic!("Expected Let statement"),
        }
    }

    #[test]
    fn test_convert_expr_function_call() {
        let source = r#"
            fn main() {
                echo("test");
            }
        "#;
        let ast = parse(source).unwrap();
        match &ast.functions[0].body[0] {
            Stmt::Expr(expr) => {
                match expr {
                    Expr::FunctionCall { name, args } => {
                        assert_eq!(name, "echo");
                        assert_eq!(args.len(), 1);
                    }
                    _ => panic!("Expected FunctionCall expression"),
                }
            }
            _ => panic!("Expected Expr statement"),
        }
    }

    #[test]
    fn test_convert_expr_binary_op() {
        let source = r#"
            fn main() {
                let x = 1 + 2;
            }
        "#;
        let ast = parse(source).unwrap();
        match &ast.functions[0].body[0] {
            Stmt::Let { value, .. } => {
                assert!(matches!(value, Expr::Binary { .. }));
            }
            _ => panic!("Expected Let statement"),
        }
    }

    #[test]
    fn test_convert_expr_unary_op() {
        let source = r#"
            fn main() {
                let x = !true;
            }
        "#;
        let ast = parse(source).unwrap();
        match &ast.functions[0].body[0] {
            Stmt::Let { value, .. } => {
                assert!(matches!(value, Expr::Unary { .. }));
            }
            _ => panic!("Expected Let statement"),
        }
    }

    #[test]
    fn test_convert_expr_parenthesized() {
        let source = r#"
            fn main() {
                let x = (42);
            }
        "#;
        let ast = parse(source).unwrap();
        match &ast.functions[0].body[0] {
            Stmt::Let { value, .. } => {
                // Parentheses should be unwrapped
                assert!(matches!(value, Expr::Literal(Literal::U32(42))));
            }
            _ => panic!("Expected Let statement"),
        }
    }

    #[test]
    fn test_convert_expr_nested_binary() {
        let source = r#"
            fn main() {
                let x = 1 + 2 * 3;
            }
        "#;
        let ast = parse(source).unwrap();
        match &ast.functions[0].body[0] {
            Stmt::Let { value, .. } => {
                match value {
                    Expr::Binary { left, right, .. } => {
                        assert!(matches!(**left, Expr::Literal(Literal::U32(1))));
                        assert!(matches!(**right, Expr::Binary { .. }));
                    }
                    _ => panic!("Expected Binary expression"),
                }
            }
            _ => panic!("Expected Let statement"),
        }
    }

    // Tests for parse function entry point
    #[test]
    fn test_parse_simple_main() {
        let source = r#"
            fn main() {
                let x = 42;
            }
        "#;
        let ast = parse(source).unwrap();
        assert_eq!(ast.entry_point, "main");
        assert_eq!(ast.functions.len(), 1);
        assert_eq!(ast.functions[0].name, "main");
    }

    #[test]
    fn test_parse_with_bashrs_main_attribute() {
        let source = r#"
            #[bashrs::main]
            fn custom_entry() {
                let x = 1;
            }
        "#;
        let ast = parse(source).unwrap();
        assert_eq!(ast.entry_point, "custom_entry");
        assert_eq!(ast.functions[0].name, "custom_entry");
    }

    #[test]
    fn test_parse_multiple_functions() {
        let source = r#"
            fn main() {
                helper();
            }

            fn helper() {
                let x = 1;
            }
        "#;
        let ast = parse(source).unwrap();
        assert_eq!(ast.entry_point, "main");
        assert_eq!(ast.functions.len(), 2);
        assert_eq!(ast.functions[0].name, "main");
        assert_eq!(ast.functions[1].name, "helper");
    }

    #[test]
    fn test_parse_no_main_function_error() {
        let source = r#"
            fn helper() {
                let x = 1;
            }
        "#;
        let result = parse(source);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No #[bashrs::main] function found"));
    }

    #[test]
    fn test_parse_multiple_main_functions_error() {
        let source = r#"
            fn main() {
                let x = 1;
            }

            #[bashrs::main]
            fn another_main() {
                let y = 2;
            }
        "#;
        let result = parse(source);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Multiple #[bashrs::main] functions found"));
    }

    #[test]
    fn test_parse_non_function_item_error() {
        let source = r#"
            const X: u32 = 42;

            fn main() {
                let x = 1;
            }
        "#;
        let result = parse(source);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Only functions are allowed"));
    }

    #[test]
    fn test_parse_legacy_rash_main_attribute() {
        let source = r#"
            #[rash::main]
            fn entry() {
                let x = 1;
            }
        "#;
        let ast = parse(source).unwrap();
        assert_eq!(ast.entry_point, "entry");
    }
}
