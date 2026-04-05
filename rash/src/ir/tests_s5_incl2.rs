fn test_exclusive_for_range_with_variable_end_subtracts_one() {
    use crate::ast::restricted::Pattern;

    // Test: for i in 0..n should produce seq 0 $((n-1)), not seq 0 $n
    let ast = RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: vec![Stmt::For {
                pattern: Pattern::Variable("i".to_string()),
                iter: Expr::Range {
                    start: Box::new(Expr::Literal(Literal::U32(0))),
                    end: Box::new(Expr::Variable("n".to_string())),
                    inclusive: false,
                },
                body: vec![Stmt::Expr(Expr::Variable("i".to_string()))],
                max_iterations: Some(1000),
            }],
        }],
        entry_point: "main".to_string(),
    };

    let ir = from_ast(&ast).expect("Should convert successfully");

    // The For node's end value should be Arithmetic { Sub, Variable("n"), String("1") }
    fn has_adjusted_end(ir: &ShellIR) -> bool {
        match ir {
            ShellIR::For { end, .. } => matches!(
                end,
                ShellValue::Arithmetic {
                    op: crate::ir::shell_ir::ArithmeticOp::Sub,
                    ..
                }
            ),
            ShellIR::Sequence(items) => items.iter().any(has_adjusted_end),
            _ => false,
        }
    }

    assert!(
        has_adjusted_end(&ir),
        "for i in 0..n should produce end=$((n-1)), not end=$n"
    );
}

#[test]
fn test_nested_match_in_match_arm_produces_nested_case() {
    // Regression: `let next = match state { 0 => match bit { ... }, ... }`
    // should produce nested Case statements, not flat assignments to '0'.
    let source = r#"
fn dispatch(state: u32, bit: u32) -> u32 {
    let next = match state {
        0 => match bit { 0 => 10, _ => 20, },
        _ => match bit { 0 => 30, _ => 40, },
    };
    return next;
}
fn main() { println!("{}", dispatch(0, 1)); }
"#;

    let ast = crate::services::parser::parse(source).expect("should parse");
    let ir = super::from_ast(&ast).expect("should lower");

    assert!(
        ir_has_nested_case(&ir),
        "nested match-in-match-arm should produce nested Case IR"
    );
}

#[test]
fn test_if_else_expression_in_match_block_arm_produces_if_assignment() {
    let source = r#"
fn categorize(x: u32) -> u32 {
    let r = match x % 3 {
        0 => {
            let half = x / 2;
            if half > 5 { half * 10 } else { half }
        },
        _ => x,
    };
    return r;
}
fn main() { println!("{}", categorize(12)); }
"#;

    let ast = crate::services::parser::parse(source).expect("should parse");
    let ir = super::from_ast(&ast).expect("should lower");

    assert!(
        ir_has_if_in_case(&ir),
        "if-else expression in match block arm should produce If IR inside Case arm"
    );
}

/// Walk the IR tree to find a nested Case inside a Case arm
fn ir_has_nested_case(ir: &super::ShellIR) -> bool {
    match ir {
        super::ShellIR::Case { arms, .. } => arms.iter().any(|arm| {
            matches!(&*arm.body, super::ShellIR::Case { .. }) || ir_has_nested_case(&arm.body)
        }),
        super::ShellIR::Sequence(stmts) => stmts.iter().any(ir_has_nested_case),
        super::ShellIR::Function { body, .. } => ir_has_nested_case(body),
        _ => false,
    }
}

/// Walk IR tree to find If inside a Case arm
fn ir_has_if_in_case(ir: &super::ShellIR) -> bool {
    match ir {
        super::ShellIR::Case { arms, .. } => arms.iter().any(|arm| ir_has_if_inside(&arm.body)),
        super::ShellIR::Sequence(stmts) => stmts.iter().any(ir_has_if_in_case),
        super::ShellIR::Function { body, .. } => ir_has_if_in_case(body),
        _ => false,
    }
}

/// Check if IR node contains an If statement
fn ir_has_if_inside(ir: &super::ShellIR) -> bool {
    match ir {
        super::ShellIR::If { .. } => true,
        super::ShellIR::Sequence(stmts) => stmts.iter().any(ir_has_if_inside),
        _ => false,
    }
}
