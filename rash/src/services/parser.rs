use crate::ast::restricted::{
    BinaryOp, Expr, Function, Literal, MatchArm, Parameter, Pattern, RestrictedAst, Stmt, Type,
    UnaryOp,
};
use crate::models::{Error, Result};
use syn::{
    parse::Parser, BinOp, Block, Expr as SynExpr, File, FnArg, Item, ItemFn, Lit, Pat, ReturnType,
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
    let item_fn = match item {
        Item::Fn(f) => f,
        // Gracefully skip non-function items (struct, enum, use, const, static, type, trait)
        // Shell is untyped — these Rust constructs have no shell equivalent
        Item::Struct(_)
        | Item::Enum(_)
        | Item::Use(_)
        | Item::Const(_)
        | Item::Static(_)
        | Item::Type(_)
        | Item::Trait(_) => return Ok(()),
        Item::Impl(item_impl) => {
            // Extract methods from impl blocks as regular functions
            for impl_item in item_impl.items {
                if let syn::ImplItem::Fn(method) = impl_item {
                    let item_fn = ItemFn {
                        attrs: method.attrs,
                        vis: method.vis,
                        sig: method.sig,
                        block: Box::new(method.block),
                    };
                    let function = convert_function(item_fn)?;
                    functions.push(function);
                }
            }
            return Ok(());
        }
        _ => {
            return Err(Error::Validation(
                "Only functions are allowed in Rash code".to_string(),
            ));
        }
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
        && path
            .segments
            .get(0)
            .is_some_and(|seg| seg.ident == "bashrs" || seg.ident == "rash")
        && path.segments.get(1).is_some_and(|seg| seg.ident == "main")
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
                // Skip self parameters (shell functions don't have receivers)
                continue;
            }
        }
    }

    // Convert return type
    let return_type = match &item_fn.sig.output {
        ReturnType::Default => Type::Void, // Default to void/unit type
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
        SynType::Path(type_path) => convert_path_type(type_path),
        SynType::Reference(type_ref) => convert_reference_type(type_ref),
        SynType::Tuple(_) => Ok(Type::Str),
        SynType::Array(_) => Ok(Type::Str),
        _ => Err(Error::Validation("Complex types not supported".to_string())),
    }
}

/// Convert a path type (e.g. bool, String, Result<T, E>) to our Type enum
fn convert_path_type(type_path: &syn::TypePath) -> Result<Type> {
    let path_str = type_path
        .path
        .segments
        .iter()
        .map(|seg| seg.ident.to_string())
        .collect::<Vec<_>>()
        .join("::");

    match path_str.as_str() {
        "bool" => Ok(Type::Bool),
        "u16" => Ok(Type::U16),
        "u32" | "i32" => Ok(Type::U32),
        "str" | "String" => Ok(Type::Str),
        path if path.starts_with("Result") => Ok(Type::Result {
            ok_type: Box::new(Type::Str),
            err_type: Box::new(Type::Str),
        }),
        path if path.starts_with("Option") => Ok(Type::Option {
            inner_type: Box::new(Type::Str),
        }),
        _ => Ok(Type::Str),
    }
}

/// Convert a reference type (e.g. &str, &[T]) to our Type enum
fn convert_reference_type(type_ref: &syn::TypeReference) -> Result<Type> {
    match &*type_ref.elem {
        SynType::Path(_) | SynType::Slice(_) => Ok(Type::Str),
        _ => Ok(Type::Str),
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
    // Handle tuple destructuring: let (a, b) = expr → Block containing multiple lets
    if let Pat::Tuple(pat_tuple) = &local.pat {
        let Some(init) = &local.init else {
            return Err(Error::Validation(
                "Let bindings must have initializers".to_string(),
            ));
        };
        let value = convert_expr(&init.expr)?;
        // Generate: { let __tuple = expr; let a = __tuple[0]; let b = __tuple[1]; }
        let tmp_name = "__tuple_tmp".to_string();
        let mut stmts = vec![Stmt::Let {
            name: tmp_name.clone(),
            value,
            declaration: true,
        }];
        for (i, elem) in pat_tuple.elems.iter().enumerate() {
            if let Pat::Ident(ident) = elem {
                let elem_name = ident.ident.to_string();
                stmts.push(Stmt::Let {
                    name: elem_name,
                    value: Expr::Index {
                        object: Box::new(Expr::Variable(tmp_name.clone())),
                        index: Box::new(Expr::Literal(Literal::I32(i as i32))),
                    },
                    declaration: true,
                });
            }
        }
        return Ok(Stmt::Expr(Expr::Block(stmts)));
    }

    // Extract identifier from pattern (handle both Pat::Ident and Pat::Type)
    // This allows: `let x = 5` and `let x: i32 = 5`
    let pat_ident = match &local.pat {
        Pat::Ident(ident) => ident,
        Pat::Type(pat_type) => {
            // For type-annotated patterns like `let args: Vec<String> = ...`
            match &*pat_type.pat {
                Pat::Ident(ident) => ident,
                _ => {
                    return Err(Error::Validation(
                        "Complex patterns not supported in type annotations".to_string(),
                    ));
                }
            }
        }
        _ => {
            return Err(Error::Validation(
                "Complex patterns not supported".to_string(),
            ));
        }
    };

    let name = pat_ident.ident.to_string();

    let Some(init) = &local.init else {
        return Err(Error::Validation(
            "Let bindings must have initializers".to_string(),
        ));
    };

    let value = convert_expr(&init.expr)?;
    Ok(Stmt::Let {
        name,
        value,
        declaration: true,
    })
}

fn convert_expr_stmt(expr: &SynExpr) -> Result<Stmt> {
    match expr {
        SynExpr::If(expr_if) => convert_if_stmt(expr_if),
        SynExpr::ForLoop(for_loop) => convert_for_loop(for_loop),
        SynExpr::While(expr_while) => convert_while_loop(expr_while),
        SynExpr::Loop(expr_loop) => {
            // loop { body } → while true { body }
            let body = convert_block(&expr_loop.body)?;
            Ok(Stmt::While {
                condition: Expr::Literal(Literal::Bool(true)),
                body,
                max_iterations: Some(10000),
            })
        }
        SynExpr::Match(expr_match) => convert_match_stmt(expr_match),
        SynExpr::Break(_) => Ok(Stmt::Break),
        SynExpr::Continue(_) => Ok(Stmt::Continue),
        SynExpr::Return(ret_expr) => {
            if let Some(inner) = &ret_expr.expr {
                Ok(Stmt::Return(Some(convert_expr(inner)?)))
            } else {
                Ok(Stmt::Return(None))
            }
        }
        SynExpr::Assign(expr_assign) => convert_assign_stmt(expr_assign),
        // Compound assignment: x += expr, x -= expr, x *= expr, etc.
        SynExpr::Binary(expr_binary) if is_compound_assign(&expr_binary.op) => {
            convert_compound_assign_stmt(expr_binary)
        }
        _ => Ok(Stmt::Expr(convert_expr(expr)?)),
    }
}

fn convert_assign_stmt(expr_assign: &syn::ExprAssign) -> Result<Stmt> {
    // x = expr -> Stmt::Let { name: "x", value: expr }
    // arr[i] = expr -> Stmt::Let { name: "arr_i", value: expr } (flat array convention)
    // In shell, reassignment is the same syntax as initial assignment
    let name = match &*expr_assign.left {
        SynExpr::Path(path) => path
            .path
            .segments
            .iter()
            .map(|seg| seg.ident.to_string())
            .collect::<Vec<_>>()
            .join("::"),
        SynExpr::Index(expr_index) => {
            // Handle both arr[i] and arr[i][j] (nested indices)
            let (array_name, index_suffix) = extract_nested_index_target(expr_index)?;
            format!("{}_{}", array_name, index_suffix)
        }
        SynExpr::Field(expr_field) => {
            // self.value = expr → value = expr (strip receiver, use field name)
            match &expr_field.member {
                syn::Member::Named(ident) => ident.to_string(),
                syn::Member::Unnamed(idx) => format!("field_{}", idx.index),
            }
        }
        SynExpr::Unary(expr_unary) if matches!(expr_unary.op, UnOp::Deref(_)) => {
            // *a = expr → a = expr (shell has no pointers, dereference is identity)
            match &*expr_unary.expr {
                SynExpr::Path(path) => path
                    .path
                    .segments
                    .iter()
                    .map(|seg| seg.ident.to_string())
                    .collect::<Vec<_>>()
                    .join("::"),
                _ => {
                    return Err(Error::Validation(
                        "Complex assignment targets not supported".to_string(),
                    ))
                }
            }
        }
        _ => {
            return Err(Error::Validation(
                "Complex assignment targets not supported".to_string(),
            ))
        }
    };
    let value = convert_expr(&expr_assign.right)?;
    Ok(Stmt::Let {
        name,
        value,
        declaration: false,
    })
}

include!("parser_incl2.rs");
