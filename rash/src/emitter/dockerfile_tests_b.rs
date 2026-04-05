#[test]
fn test_DOCKER_COV_020_cmd_instruction() {
    let ast = make_simple_ast(vec![
        Stmt::Expr(Expr::FunctionCall {
            name: "from_image".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("alpine".to_string())),
                Expr::Literal(Literal::Str("3.18".to_string())),
            ],
        }),
        Stmt::Expr(Expr::FunctionCall {
            name: "cmd".to_string(),
            args: vec![Expr::Array(vec![
                Expr::Literal(Literal::Str("/bin/sh".to_string())),
            ])],
        }),
    ]);
    let result = emit_dockerfile(&ast).unwrap();
    assert!(result.contains("CMD"), "CMD instruction in: {result}");
}
