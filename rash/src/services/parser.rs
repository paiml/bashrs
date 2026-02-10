use crate::ast::restricted::{
    BinaryOp, Expr, Function, Literal, MatchArm, Parameter, Pattern, RestrictedAst, Stmt, Type,
    UnaryOp,
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
                "u16" => Ok(Type::U16),
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
                // Single-letter or unknown types: treat as Str (shell is untyped)
                // This handles generic type params like T, U, V, etc.
                _ => Ok(Type::Str),
            }
        }
        SynType::Reference(type_ref) => {
            // Handle &str, &[T] and other reference types
            match &*type_ref.elem {
                SynType::Path(path) => {
                    let path_str = path
                        .path
                        .segments
                        .iter()
                        .map(|seg| seg.ident.to_string())
                        .collect::<Vec<_>>()
                        .join("::");

                    match path_str.as_str() {
                        "str" => Ok(Type::Str),
                        // All other reference types (&i32, &mut T, etc.) → Str (shell is untyped)
                        _ => Ok(Type::Str),
                    }
                }
                SynType::Slice(_) => {
                    // &[T] slice references are treated as Str for shell compatibility
                    // The actual array content is handled at the expression level
                    Ok(Type::Str)
                }
                // All other reference types: &[i32; 5], &dyn Trait, etc. → Str
                _ => Ok(Type::Str),
            }
        }
        SynType::Tuple(_) => {
            // Tuple types like (i32, i32) → map to Str (shell is untyped)
            Ok(Type::Str)
        }
        SynType::Array(_) => {
            // Fixed-size array types like [i32; 5] → Str (shell is untyped)
            Ok(Type::Str)
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
    Ok(Stmt::Let { name, value })
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
    Ok(Stmt::Let { name, value })
}

/// Extract array name and combined index suffix for nested index targets like arr[i][j].
fn extract_nested_index_target(expr_index: &syn::ExprIndex) -> Result<(String, String)> {
    let index_suffix = extract_index_suffix(&expr_index.index)?;
    match &*expr_index.expr {
        SynExpr::Path(path) => {
            let name = path
                .path
                .segments
                .iter()
                .map(|seg| seg.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");
            Ok((name, index_suffix))
        }
        SynExpr::Index(inner_index) => {
            // Nested: arr[i][j] → (arr, i_j)
            let (name, inner_suffix) = extract_nested_index_target(inner_index)?;
            Ok((name, format!("{}_{}", inner_suffix, index_suffix)))
        }
        _ => Err(Error::Validation(
            "Complex array index target not supported".to_string(),
        )),
    }
}

/// Extract a naming suffix from an array index expression.
/// Handles literal integers, variables, and simple binary expressions.
fn extract_index_suffix(expr: &SynExpr) -> Result<String> {
    match expr {
        SynExpr::Lit(lit) => match &lit.lit {
            syn::Lit::Int(lit_int) => Ok(lit_int.base10_digits().to_string()),
            _ => Err(Error::Validation(
                "Array index must be integer or variable".to_string(),
            )),
        },
        SynExpr::Path(path) => Ok(path
            .path
            .segments
            .iter()
            .map(|seg| seg.ident.to_string())
            .collect::<Vec<_>>()
            .join("_")),
        SynExpr::Binary(bin) => {
            let left = extract_index_suffix(&bin.left)?;
            let right = extract_index_suffix(&bin.right)?;
            Ok(format!("{}_{}", left, right))
        }
        SynExpr::Paren(paren) => extract_index_suffix(&paren.expr),
        SynExpr::Index(idx) => {
            // Nested array index: arr[other[i]] → "other_i"
            let obj = extract_index_suffix(&idx.expr)?;
            let inner = extract_index_suffix(&idx.index)?;
            Ok(format!("{}_{}", obj, inner))
        }
        SynExpr::MethodCall(mc) => {
            // method calls like arr.len() → "arr_len"
            let recv = extract_index_suffix(&mc.receiver)?;
            Ok(format!("{}_{}", recv, mc.method))
        }
        SynExpr::Unary(unary) => {
            // Handle -i, !x etc
            let inner = extract_index_suffix(&unary.expr)?;
            Ok(inner)
        }
        SynExpr::Call(call) => {
            // Function call as index: arr[hash(val)] → "hash_val"
            let func_name = if let SynExpr::Path(path) = &*call.func {
                path.path
                    .segments
                    .iter()
                    .map(|seg| seg.ident.to_string())
                    .collect::<Vec<_>>()
                    .join("_")
            } else {
                "call".to_string()
            };
            let args: Vec<String> = call
                .args
                .iter()
                .filter_map(|arg| extract_index_suffix(arg).ok())
                .collect();
            if args.is_empty() {
                Ok(func_name)
            } else {
                Ok(format!("{}_{}", func_name, args.join("_")))
            }
        }
        _ => Err(Error::Validation(
            "Unsupported array index expression".to_string(),
        )),
    }
}

/// Check if a BinOp is a compound assignment operator (+=, -=, *=, /=, %=)
fn is_compound_assign(op: &BinOp) -> bool {
    matches!(
        op,
        BinOp::AddAssign(_)
            | BinOp::SubAssign(_)
            | BinOp::MulAssign(_)
            | BinOp::DivAssign(_)
            | BinOp::RemAssign(_)
            | BinOp::BitAndAssign(_)
            | BinOp::BitOrAssign(_)
            | BinOp::BitXorAssign(_)
            | BinOp::ShlAssign(_)
            | BinOp::ShrAssign(_)
    )
}

/// Desugar compound assignment: x += expr -> x = x + expr
fn compound_assign_to_binary_op(op: &BinOp) -> Result<BinaryOp> {
    match op {
        BinOp::AddAssign(_) => Ok(BinaryOp::Add),
        BinOp::SubAssign(_) => Ok(BinaryOp::Sub),
        BinOp::MulAssign(_) => Ok(BinaryOp::Mul),
        BinOp::DivAssign(_) => Ok(BinaryOp::Div),
        BinOp::RemAssign(_) => Ok(BinaryOp::Rem),
        BinOp::BitAndAssign(_) => Ok(BinaryOp::BitAnd),
        BinOp::BitOrAssign(_) => Ok(BinaryOp::BitOr),
        BinOp::BitXorAssign(_) => Ok(BinaryOp::BitXor),
        BinOp::ShlAssign(_) => Ok(BinaryOp::Shl),
        BinOp::ShrAssign(_) => Ok(BinaryOp::Shr),
        _ => Err(Error::Validation(
            "Unsupported compound assignment operator".to_string(),
        )),
    }
}

fn convert_compound_assign_stmt(expr_binary: &syn::ExprBinary) -> Result<Stmt> {
    let name = match &*expr_binary.left {
        SynExpr::Path(path) => path
            .path
            .segments
            .iter()
            .map(|seg| seg.ident.to_string())
            .collect::<Vec<_>>()
            .join("::"),
        SynExpr::Index(expr_index) => {
            let (array_name, index_suffix) = extract_nested_index_target(expr_index)?;
            format!("{}_{}", array_name, index_suffix)
        }
        SynExpr::Field(expr_field) => {
            // self.value += expr → value += expr (strip receiver, use field name)
            match &expr_field.member {
                syn::Member::Named(ident) => ident.to_string(),
                syn::Member::Unnamed(idx) => format!("field_{}", idx.index),
            }
        }
        SynExpr::Unary(expr_unary) if matches!(expr_unary.op, UnOp::Deref(_)) => {
            // *val += expr → val += expr (shell has no pointers)
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
    let op = compound_assign_to_binary_op(&expr_binary.op)?;
    let right = convert_expr(&expr_binary.right)?;
    let left = Expr::Variable(name.clone());
    let value = Expr::Binary {
        op,
        left: Box::new(left),
        right: Box::new(right),
    };
    Ok(Stmt::Let { name, value })
}

/// Split macro arguments on commas, respecting nested `()`, `[]`, `{}`, and string literals.
fn split_macro_args(token_str: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut depth_paren = 0i32;
    let mut depth_bracket = 0i32;
    let mut depth_brace = 0i32;
    let mut in_string = false;
    let mut escape_count = 0u32;

    for ch in token_str.chars() {
        if in_string {
            current.push(ch);
            if ch == '\\' {
                escape_count += 1;
            } else {
                if ch == '"' && escape_count % 2 == 0 {
                    in_string = false;
                }
                escape_count = 0;
            }
            continue;
        }

        match ch {
            '"' => {
                in_string = true;
                current.push(ch);
            }
            '(' => {
                depth_paren += 1;
                current.push(ch);
            }
            ')' => {
                depth_paren -= 1;
                current.push(ch);
            }
            '[' => {
                depth_bracket += 1;
                current.push(ch);
            }
            ']' => {
                depth_bracket -= 1;
                current.push(ch);
            }
            '{' => {
                depth_brace += 1;
                current.push(ch);
            }
            '}' => {
                depth_brace -= 1;
                current.push(ch);
            }
            ',' if depth_paren == 0 && depth_bracket == 0 && depth_brace == 0 => {
                parts.push(current.trim().to_string());
                current.clear();
            }
            _ => {
                current.push(ch);
            }
        }
    }

    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() {
        parts.push(trimmed);
    }

    parts
}

/// Parse a format string like `"hello {} world {}"` and return (literal_segments, placeholder_count).
/// literal_segments alternate between text and `{}` positions.
fn parse_format_string(fmt: &str) -> Vec<FormatSegment> {
    let mut segments = Vec::new();
    let mut current = String::new();
    let mut chars = fmt.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '{' {
            if chars.peek() == Some(&'{') {
                // Escaped brace {{
                chars.next();
                current.push('{');
            } else if chars.peek() == Some(&'}') {
                // Format placeholder {}
                chars.next();
                if !current.is_empty() {
                    segments.push(FormatSegment::Literal(current.clone()));
                    current.clear();
                }
                segments.push(FormatSegment::Placeholder);
            } else {
                // Could be {:fmt} - consume until closing }
                chars.next(); // skip what's after {
                while let Some(&c) = chars.peek() {
                    if c == '}' {
                        chars.next();
                        break;
                    }
                    chars.next();
                }
                if !current.is_empty() {
                    segments.push(FormatSegment::Literal(current.clone()));
                    current.clear();
                }
                segments.push(FormatSegment::Placeholder);
            }
        } else if ch == '}' {
            if chars.peek() == Some(&'}') {
                // Escaped brace }}
                chars.next();
                current.push('}');
            } else {
                current.push(ch);
            }
        } else {
            current.push(ch);
        }
    }

    if !current.is_empty() {
        segments.push(FormatSegment::Literal(current));
    }

    segments
}

#[derive(Debug)]
enum FormatSegment {
    Literal(String),
    Placeholder,
}

/// Build a Concat expression from format segments and argument expressions.
fn build_format_concat(segments: &[FormatSegment], args: &[Expr]) -> Expr {
    let mut parts = Vec::new();
    let mut arg_idx = 0;

    for segment in segments {
        match segment {
            FormatSegment::Literal(s) => {
                parts.push(Expr::Literal(Literal::Str(s.clone())));
            }
            FormatSegment::Placeholder => {
                if arg_idx < args.len() {
                    parts.push(args[arg_idx].clone());
                    arg_idx += 1;
                }
            }
        }
    }

    // If we have exactly one part, return it directly
    if parts.len() == 1 {
        return parts.into_iter().next().expect("verified len == 1");
    }

    // Return as a FunctionCall to rash_concat which will be handled in IR
    // Actually, we use Concat pattern: wrap in a multi-part expression
    // The simplest approach: build a chain that the IR can handle
    // We'll represent this as a FunctionCall to a special internal function
    // that the IR layer converts to ShellValue::Concat
    Expr::FunctionCall {
        name: "__format_concat".to_string(),
        args: parts,
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

    if macro_name == "println" || macro_name == "eprintln" || macro_name == "print" {
        let tokens = macro_stmt.mac.tokens.clone();
        let token_str = tokens.to_string();

        // Split on top-level commas
        let parts = split_macro_args(&token_str);

        let arg = if parts.len() <= 1 {
            // Single argument: try to parse directly
            let parsed: syn::Expr = syn::parse2(tokens)
                .map_err(|_| Error::Validation(format!("Invalid {macro_name}! arguments")))?;
            convert_expr(&parsed)?
        } else {
            // Multiple arguments: format string + args
            // First part is the format string
            let fmt_str = &parts[0];
            // Parse the format string literal
            let fmt_value = if let Ok(parsed) = syn::parse_str::<syn::Expr>(fmt_str) {
                if let SynExpr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(lit_str),
                    ..
                }) = &parsed
                {
                    lit_str.value()
                } else {
                    // Not a string literal, try as expression
                    return Ok(Stmt::Expr(Expr::FunctionCall {
                        name: if macro_name == "eprintln" {
                            "rash_eprintln"
                        } else if macro_name == "print" {
                            "rash_print"
                        } else {
                            "rash_println"
                        }
                        .to_string(),
                        args: vec![convert_expr(&parsed)?],
                    }));
                }
            } else {
                return Err(Error::Validation(format!(
                    "Invalid {macro_name}! format string"
                )));
            };

            // Parse remaining arguments as expressions
            let mut expr_args = Vec::new();
            for part in &parts[1..] {
                let parsed = syn::parse_str::<syn::Expr>(part).map_err(|_| {
                    Error::Validation(format!("Invalid {macro_name}! argument: {part}"))
                })?;
                expr_args.push(convert_expr(&parsed)?);
            }

            // Parse format string into segments
            let segments = parse_format_string(&fmt_value);
            build_format_concat(&segments, &expr_args)
        };

        let func_name = if macro_name == "eprintln" {
            "rash_eprintln"
        } else if macro_name == "print" {
            "rash_print"
        } else {
            "rash_println"
        };
        Ok(Stmt::Expr(Expr::FunctionCall {
            name: func_name.to_string(),
            args: vec![arg],
        }))
    } else {
        Err(Error::Validation(format!(
            "Unsupported macro: {macro_name}!"
        )))
    }
}

/// Convert an if-expression used in expression position (e.g., `let x = if c { a } else { b }`).
/// For simple single-expression branches: __if_expr(cond, then_val, else_val).
/// For multi-statement branches: Expr::Block([Stmt::If{...}]) to preserve all statements.
fn convert_if_expr(expr_if: &syn::ExprIf) -> Result<Expr> {
    // Check if either branch has multiple statements — if so, use Stmt::If to preserve them
    let then_multi = expr_if.then_branch.stmts.len() > 1;
    let else_multi = if let Some((_, else_expr)) = &expr_if.else_branch {
        match &**else_expr {
            SynExpr::Block(block) => block.block.stmts.len() > 1,
            _ => false,
        }
    } else {
        false
    };

    if then_multi || else_multi {
        // Multi-statement branch: produce Expr::Block([Stmt::If{...}])
        // This preserves all let bindings in each branch
        let if_stmt = convert_if_stmt(expr_if)?;
        return Ok(Expr::Block(vec![if_stmt]));
    }

    let condition = convert_expr(&expr_if.cond)?;

    // Extract the last expression from the then branch
    let then_value = extract_branch_value(&expr_if.then_branch)?;

    // Extract the else branch value
    let else_value = if let Some((_, else_expr)) = &expr_if.else_branch {
        match &**else_expr {
            SynExpr::Block(block) => extract_branch_value(&block.block)?,
            SynExpr::If(nested_if) => convert_if_expr(nested_if)?,
            other => convert_expr(other)?,
        }
    } else {
        // No else branch - use empty string as default
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
        SynExpr::Repeat(expr_repeat) => {
            // [value; count] → Array of count copies of value
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
        SynExpr::Block(expr_block) => {
            // Block expressions: { stmts; expr }
            let mut stmts = Vec::new();
            for stmt in &expr_block.block.stmts {
                stmts.push(convert_stmt(stmt)?);
            }
            Ok(Expr::Block(stmts))
        }
        SynExpr::Cast(expr_cast) => {
            // `expr as Type` → just convert the inner expression (shell is untyped)
            convert_expr(&expr_cast.expr)
        }
        SynExpr::Match(expr_match) => {
            // Match in expression position → convert to Block containing match statement
            let match_stmt = convert_match_stmt(expr_match)?;
            Ok(Expr::Block(vec![match_stmt]))
        }
        SynExpr::Tuple(expr_tuple) => {
            // (a, b, c) → treat as array [a, b, c] for shell
            let mut elements = Vec::new();
            for elem in &expr_tuple.elems {
                elements.push(convert_expr(elem)?);
            }
            Ok(Expr::Array(elements))
        }
        SynExpr::Closure(expr_closure) => {
            // |args| body → convert body as a simple expression
            // Shell doesn't have closures, so convert to the body expression directly
            convert_expr(&expr_closure.body)
        }
        SynExpr::Let(expr_let) => {
            // `if let PAT = expr` → desugar to comparison
            // `if let 0 = x` → x == 0
            // `if let Some(v) = x` → x != "" (non-empty check)
            let rhs = convert_expr(&expr_let.expr)?;
            match &*expr_let.pat {
                Pat::Lit(lit_pat) => {
                    // `if let 0 = x` → x == 0
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
                _ => {
                    // Default: just use the expression value (truthy check)
                    Ok(rhs)
                }
            }
        }
        SynExpr::Struct(expr_struct) => {
            // Struct literal: Point { x: 3, y: 4 } → array of field values
            let mut values = Vec::new();
            for field in &expr_struct.fields {
                values.push(convert_expr(&field.expr)?);
            }
            Ok(Expr::Array(values))
        }
        SynExpr::Field(expr_field) => {
            // Field access: p.x → array index (struct fields mapped to array positions)
            let object = convert_expr(&expr_field.base)?;
            // Field name → index based on position (simplified)
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

fn convert_macro_expr(expr_macro: &syn::ExprMacro) -> Result<Expr> {
    let macro_name = expr_macro
        .mac
        .path
        .segments
        .last()
        .ok_or_else(|| Error::Validation("Empty macro path".to_string()))?
        .ident
        .to_string();

    if macro_name == "format" {
        // format!("{}", x) -> treat as string interpolation
        let tokens = expr_macro.mac.tokens.clone();
        let token_str = tokens.to_string();

        // Split on top-level commas
        let parts = split_macro_args(&token_str);

        if parts.len() <= 1 {
            // Single argument: format!("hello") or format!("{}", x) where x is the only token
            if let Ok(parsed) = syn::parse_str::<syn::Expr>(&token_str) {
                return convert_expr(&parsed);
            }
            return Err(Error::Validation(
                "Could not parse format! arguments".to_string(),
            ));
        }

        // Multiple arguments: format!("{} {}", a, b)
        let fmt_str = &parts[0];
        let fmt_value = if let Ok(parsed) = syn::parse_str::<syn::Expr>(fmt_str) {
            if let SynExpr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(lit_str),
                ..
            }) = &parsed
            {
                lit_str.value()
            } else {
                // Not a string literal - just use first arg
                return convert_expr(&parsed);
            }
        } else {
            return Err(Error::Validation(
                "Could not parse format! format string".to_string(),
            ));
        };

        // Parse remaining arguments
        let mut expr_args = Vec::new();
        for part in &parts[1..] {
            let parsed = syn::parse_str::<syn::Expr>(part).map_err(|_| {
                Error::Validation(format!("Could not parse format! argument: {part}"))
            })?;
            expr_args.push(convert_expr(&parsed)?);
        }

        let segments = parse_format_string(&fmt_value);
        return Ok(build_format_concat(&segments, &expr_args));
    } else if macro_name == "vec" {
        // vec![...] -> array
        let tokens = expr_macro.mac.tokens.clone();
        let token_str = tokens.to_string();
        // Try to parse as array elements
        let array_str = format!("[{token_str}]");
        if let Ok(parsed) = syn::parse_str::<syn::ExprArray>(&array_str) {
            return convert_array_expr(&parsed);
        }
        Err(Error::Validation(
            "Could not parse vec! arguments".to_string(),
        ))
    } else if macro_name == "println" || macro_name == "eprintln" || macro_name == "print" {
        // Handle println!/eprintln!/print! in expression position (e.g., match arms)
        let tokens = expr_macro.mac.tokens.clone();
        let token_str = tokens.to_string();
        let parts = split_macro_args(&token_str);

        let arg = if parts.len() <= 1 {
            let parsed: syn::Expr = syn::parse2(tokens)
                .map_err(|_| Error::Validation(format!("Invalid {macro_name}! arguments")))?;
            convert_expr(&parsed)?
        } else {
            let fmt_str = &parts[0];
            let fmt_value = if let Ok(parsed) = syn::parse_str::<syn::Expr>(fmt_str) {
                if let SynExpr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(lit_str),
                    ..
                }) = &parsed
                {
                    lit_str.value()
                } else {
                    let func_name = if macro_name == "eprintln" {
                        "rash_eprintln"
                    } else if macro_name == "print" {
                        "rash_print"
                    } else {
                        "rash_println"
                    };
                    return Ok(Expr::FunctionCall {
                        name: func_name.to_string(),
                        args: vec![convert_expr(&parsed)?],
                    });
                }
            } else {
                return Err(Error::Validation(format!(
                    "Invalid {macro_name}! format string"
                )));
            };

            let mut expr_args = Vec::new();
            for part in &parts[1..] {
                let parsed = syn::parse_str::<syn::Expr>(part).map_err(|_| {
                    Error::Validation(format!("Invalid {macro_name}! argument: {part}"))
                })?;
                expr_args.push(convert_expr(&parsed)?);
            }

            let segments = parse_format_string(&fmt_value);
            build_format_concat(&segments, &expr_args)
        };

        let func_name = if macro_name == "eprintln" {
            "rash_eprintln"
        } else if macro_name == "print" {
            "rash_print"
        } else {
            "rash_println"
        };
        Ok(Expr::FunctionCall {
            name: func_name.to_string(),
            args: vec![arg],
        })
    } else {
        Err(Error::Validation(format!(
            "Unsupported macro expression: {macro_name}!"
        )))
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
    // Dereference (*expr) → just convert inner expression (shell has no pointers)
    if let UnOp::Deref(_) = &expr_unary.op {
        return convert_expr(&expr_unary.expr);
    }

    // Special case: negative integer literals (-1, -42, etc.)
    // Simplify UnaryOp(Neg, Literal(n)) to Literal(-n)
    if let UnOp::Neg(_) = &expr_unary.op {
        if let SynExpr::Lit(lit_expr) = &*expr_unary.expr {
            if let Lit::Int(lit_int) = &lit_expr.lit {
                // Special case: i32::MIN (-2147483648)
                // Can't parse 2147483648 as i32 since i32::MAX = 2147483647
                let lit_str = lit_int.to_string();
                if lit_str == "2147483648" {
                    return Ok(Expr::Literal(Literal::I32(i32::MIN)));
                }

                // Parse as i32 for other negative numbers
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

fn convert_pattern(pat: &Pat) -> Result<Pattern> {
    match pat {
        Pat::Lit(lit_pat) => {
            // Convert literal patterns like 1, 2, "hello"
            let literal = convert_literal(&lit_pat.lit)?;
            Ok(Pattern::Literal(literal))
        }
        Pat::Ident(ident_pat) => {
            // Check if this is a wildcard (_)
            let name = ident_pat.ident.to_string();
            if name == "_" {
                Ok(Pattern::Wildcard)
            } else {
                Ok(Pattern::Variable(name))
            }
        }
        Pat::Wild(_) => Ok(Pattern::Wildcard),
        Pat::TupleStruct(tuple_struct) => {
            // Handle Some(x), Ok(x), Err(x), Color::Red, etc.
            // Extract the constructor name and treat as a variable binding
            let path_str = tuple_struct
                .path
                .segments
                .iter()
                .map(|seg| seg.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");
            // For Option/Result patterns, bind the inner variable and use wildcard pattern
            // The scrutinee value flows through in shell
            match path_str.as_str() {
                "Some" | "Ok" => {
                    // Extract inner binding name if present
                    if let Some(first) = tuple_struct.elems.first() {
                        if let Pat::Ident(ident) = first {
                            return Ok(Pattern::Variable(ident.ident.to_string()));
                        }
                    }
                    Ok(Pattern::Wildcard)
                }
                "Err" => {
                    if let Some(first) = tuple_struct.elems.first() {
                        if let Pat::Ident(ident) = first {
                            return Ok(Pattern::Variable(ident.ident.to_string()));
                        }
                    }
                    Ok(Pattern::Wildcard)
                }
                "None" => Ok(Pattern::Literal(Literal::Str(String::new()))),
                _ => {
                    // Enum variant: Color::Red → treat as string literal
                    Ok(Pattern::Literal(Literal::Str(path_str)))
                }
            }
        }
        Pat::Path(pat_path) => {
            // Handle bare enum variants like None, Color::Red (no parens)
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
        Pat::Range(range_pat) => {
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
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must have initializers"));
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
            Stmt::If {
                condition,
                then_block,
                else_block,
            } => {
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
            Stmt::If {
                condition,
                then_block,
                else_block,
            } => {
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
            Stmt::If {
                condition,
                then_block,
                else_block,
            } => {
                assert!(matches!(condition, Expr::Literal(Literal::Bool(true))));
                assert_eq!(then_block.len(), 1);

                // Verify else block contains nested if
                assert!(else_block.is_some());
                let else_stmts = else_block.as_ref().unwrap();
                assert_eq!(else_stmts.len(), 1);

                match &else_stmts[0] {
                    Stmt::If {
                        condition: nested_cond,
                        then_block: nested_then,
                        else_block: nested_else,
                    } => {
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
                    Stmt::If {
                        else_block: second_else_block,
                        ..
                    } => {
                        assert!(second_else_block.is_some());
                        let second_else = second_else_block.as_ref().unwrap();
                        assert_eq!(second_else.len(), 1);

                        // Verify third level else-if
                        match &second_else[0] {
                            Stmt::If {
                                condition,
                                else_block: third_else,
                                ..
                            } => {
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
                    Stmt::If {
                        else_block: second_else_block,
                        ..
                    } => {
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
    fn test_convert_stmt_loop_supported() {
        // Loop is now supported: `loop { }` converts to `while true { }`
        let source = r#"
            fn main() {
                loop { }
            }
        "#;
        let result = parse(source);
        assert!(
            result.is_ok(),
            "loop {{}} should be supported (converts to while true): {:?}",
            result.err()
        );
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
            Stmt::Let { value, .. } => match value {
                Expr::Variable(name) => assert_eq!(name, "x"),
                _ => panic!("Expected Variable expression"),
            },
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
            Stmt::Expr(expr) => match expr {
                Expr::FunctionCall { name, args } => {
                    assert_eq!(name, "echo");
                    assert_eq!(args.len(), 1);
                }
                _ => panic!("Expected FunctionCall expression"),
            },
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
            Stmt::Let { value, .. } => match value {
                Expr::Binary { left, right, .. } => {
                    assert!(matches!(**left, Expr::Literal(Literal::U32(1))));
                    assert!(matches!(**right, Expr::Binary { .. }));
                }
                _ => panic!("Expected Binary expression"),
            },
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
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No #[bashrs::main] function found"));
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
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Multiple #[bashrs::main] functions found"));
    }

    #[test]
    fn test_parse_non_function_item_skipped() {
        // Non-function items (const, struct, enum, etc.) are now gracefully skipped
        let source = r#"
            const X: u32 = 42;

            fn main() {
                let x = 1;
            }
        "#;
        let result = parse(source);
        assert!(
            result.is_ok(),
            "Non-function items should be gracefully skipped: {:?}",
            result.err()
        );
        let ast = result.expect("parse should succeed");
        assert_eq!(ast.functions.len(), 1);
        assert_eq!(ast.entry_point, "main");
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
