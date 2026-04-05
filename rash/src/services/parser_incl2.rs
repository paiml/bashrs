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
/// Extract a suffix string from a function call index expression: arr[hash(val)] → "hash_val"
fn extract_call_index_suffix(call: &syn::ExprCall) -> Result<String> {
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

/// Convert a syn path to an underscore-joined string (e.g., `std::io` → `std_io`).
fn path_to_suffix(path: &syn::ExprPath) -> String {
    path.path
        .segments
        .iter()
        .map(|seg| seg.ident.to_string())
        .collect::<Vec<_>>()
        .join("_")
}

/// Extract an integer literal as a suffix string, or error if not an integer.
fn lit_to_index_suffix(lit: &syn::ExprLit) -> Result<String> {
    if let syn::Lit::Int(lit_int) = &lit.lit {
        Ok(lit_int.base10_digits().to_string())
    } else {
        Err(Error::Validation(
            "Array index must be integer or variable".to_string(),
        ))
    }
}

fn extract_index_suffix(expr: &SynExpr) -> Result<String> {
    match expr {
        SynExpr::Lit(lit) => lit_to_index_suffix(lit),
        SynExpr::Path(path) => Ok(path_to_suffix(path)),
        SynExpr::Binary(bin) => {
            let left = extract_index_suffix(&bin.left)?;
            let right = extract_index_suffix(&bin.right)?;
            Ok(format!("{}_{}", left, right))
        }
        SynExpr::Paren(paren) => extract_index_suffix(&paren.expr),
        SynExpr::Index(idx) => {
            let obj = extract_index_suffix(&idx.expr)?;
            let inner = extract_index_suffix(&idx.index)?;
            Ok(format!("{}_{}", obj, inner))
        }
        SynExpr::MethodCall(mc) => {
            let recv = extract_index_suffix(&mc.receiver)?;
            Ok(format!("{}_{}", recv, mc.method))
        }
        SynExpr::Unary(unary) => extract_index_suffix(&unary.expr),
        SynExpr::Call(call) => extract_call_index_suffix(call),
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
    Ok(Stmt::Let {
        name,
        value,
        declaration: false,
    })
}

/// Split macro arguments on commas, respecting nested `()`, `[]`, `{}`, and string literals.
/// State tracker for splitting macro args on top-level commas.
struct MacroArgSplitter {
    depth_paren: i32,
    depth_bracket: i32,
    depth_brace: i32,
    in_string: bool,
    escape_count: u32,
}

impl MacroArgSplitter {
    fn new() -> Self {
        Self {
            depth_paren: 0,
            depth_bracket: 0,
            depth_brace: 0,
            in_string: false,
            escape_count: 0,
        }
    }

    fn at_top_level(&self) -> bool {
        self.depth_paren == 0 && self.depth_bracket == 0 && self.depth_brace == 0
    }

    /// Process a character inside a string literal. Returns true if still in string.
    fn process_string_char(&mut self, ch: char) {
        if ch == '\\' {
            self.escape_count += 1;
        } else {
            if ch == '"' && self.escape_count.is_multiple_of(2) {
                self.in_string = false;
            }
            self.escape_count = 0;
        }
    }

    fn process_char(&mut self, ch: char) {
        match ch {
            '"' => self.in_string = true,
            '(' => self.depth_paren += 1,
            ')' => self.depth_paren -= 1,
            '[' => self.depth_bracket += 1,
            ']' => self.depth_bracket -= 1,
            '{' => self.depth_brace += 1,
            '}' => self.depth_brace -= 1,
            _ => {}
        }
    }
}

fn split_macro_args(token_str: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut state = MacroArgSplitter::new();

    for ch in token_str.chars() {
        if state.in_string {
            current.push(ch);
            state.process_string_char(ch);
            continue;
        }

        if ch == ',' && state.at_top_level() {
            parts.push(current.trim().to_string());
            current.clear();
        } else {
            state.process_char(ch);
            current.push(ch);
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
/// Flush accumulated literal text as a segment if non-empty.
fn flush_literal(segments: &mut Vec<FormatSegment>, current: &mut String) {
    if !current.is_empty() {
        segments.push(FormatSegment::Literal(current.clone()));
        current.clear();
    }
}

/// Handle a `{` character in a format string: escaped `{{`, placeholder `{}`, or `{:fmt}`.
fn handle_open_brace(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    segments: &mut Vec<FormatSegment>,
    current: &mut String,
) {
    if chars.peek() == Some(&'{') {
        chars.next();
        current.push('{');
    } else if chars.peek() == Some(&'}') {
        chars.next();
        flush_literal(segments, current);
        segments.push(FormatSegment::Placeholder);
    } else {
        // {:fmt} — consume until closing }
        chars.next();
        while let Some(&c) = chars.peek() {
            if c == '}' {
                chars.next();
                break;
            }
            chars.next();
        }
        flush_literal(segments, current);
        segments.push(FormatSegment::Placeholder);
    }
}

fn parse_format_string(fmt: &str) -> Vec<FormatSegment> {
    let mut segments = Vec::new();
    let mut current = String::new();
    let mut chars = fmt.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '{' => handle_open_brace(&mut chars, &mut segments, &mut current),
            '}' if chars.peek() == Some(&'}') => {
                chars.next();
                current.push('}');
            }
            _ => current.push(ch),
        }
    }

    flush_literal(&mut segments, &mut current);
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
        // SAFETY: len() == 1 checked above, so next() always returns Some
        #[allow(clippy::expect_used)]
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


include!("parser_incl2_incl2.rs");
