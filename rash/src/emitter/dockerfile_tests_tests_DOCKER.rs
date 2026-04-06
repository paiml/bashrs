fn test_DOCKER_COV_022_from_image_pushes_existing_stage() {
    // Two from_image calls — second one pushes the first stage
    let ast = make_simple_ast(vec![
        Stmt::Expr(Expr::FunctionCall {
            name: "from_image".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("rust".to_string())),
                Expr::Literal(Literal::Str("1.75".to_string())),
            ],
        }),
        Stmt::Expr(Expr::FunctionCall {
            name: "run".to_string(),
            args: vec![Expr::Literal(Literal::Str("cargo build".to_string()))],
        }),
        Stmt::Expr(Expr::FunctionCall {
            name: "from_image".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("alpine".to_string())),
                Expr::Literal(Literal::Str("3.18".to_string())),
            ],
        }),
    ]);
    let result = emit_dockerfile(&ast).unwrap();
    assert!(
        result.contains("FROM rust:1.75"),
        "First stage in: {result}"
    );
    assert!(
        result.contains("FROM alpine:3.18"),
        "Second stage in: {result}"
    );
}

#[test]
fn test_DOCKER_COV_023_label_success() {
    let ast = make_simple_ast(vec![
        Stmt::Expr(Expr::FunctionCall {
            name: "from_image".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("alpine".to_string())),
                Expr::Literal(Literal::Str("3.18".to_string())),
            ],
        }),
        Stmt::Expr(Expr::FunctionCall {
            name: "label".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("maintainer".to_string())),
                Expr::Literal(Literal::Str("test@example.com".to_string())),
            ],
        }),
    ]);
    let result = emit_dockerfile(&ast).unwrap();
    assert!(result.contains("LABEL maintainer"), "LABEL in: {result}");
}

#[test]
fn test_DOCKER_COV_024_comment_success() {
    let ast = make_simple_ast(vec![
        Stmt::Expr(Expr::FunctionCall {
            name: "from_image".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("alpine".to_string())),
                Expr::Literal(Literal::Str("3.18".to_string())),
            ],
        }),
        Stmt::Expr(Expr::FunctionCall {
            name: "comment".to_string(),
            args: vec![Expr::Literal(Literal::Str("This is a comment".to_string()))],
        }),
    ]);
    let result = emit_dockerfile(&ast).unwrap();
    assert!(
        result.contains("# This is a comment"),
        "Comment in: {result}"
    );
}

#[test]
fn test_DOCKER_COV_025_entrypoint_single_string() {
    let ast = make_simple_ast(vec![
        Stmt::Expr(Expr::FunctionCall {
            name: "from_image".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("alpine".to_string())),
                Expr::Literal(Literal::Str("3.18".to_string())),
            ],
        }),
        Stmt::Expr(Expr::FunctionCall {
            name: "entrypoint".to_string(),
            args: vec![Expr::Literal(Literal::Str("/app/start".to_string()))],
        }),
    ]);
    let result = emit_dockerfile(&ast).unwrap();
    assert!(result.contains("ENTRYPOINT"), "ENTRYPOINT in: {result}");
}

#[test]
fn test_DOCKER_BUILD_025_comprehensive_dockerfile() {
    let ast = make_simple_ast(vec![
        Stmt::Expr(Expr::FunctionCall {
            name: "from_image_as".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("rust".to_string())),
                Expr::Literal(Literal::Str("1.75-alpine".to_string())),
                Expr::Literal(Literal::Str("builder".to_string())),
            ],
        }),
        Stmt::Expr(Expr::FunctionCall {
            name: "workdir".to_string(),
            args: vec![Expr::Literal(Literal::Str("/app".to_string()))],
        }),
        Stmt::Expr(Expr::FunctionCall {
            name: "copy".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("Cargo.toml".to_string())),
                Expr::Literal(Literal::Str(".".to_string())),
            ],
        }),
        Stmt::Expr(Expr::FunctionCall {
            name: "run".to_string(),
            args: vec![Expr::Array(vec![Expr::Literal(Literal::Str(
                "cargo build --release".to_string(),
            ))])],
        }),
        // Runtime stage
        Stmt::Expr(Expr::FunctionCall {
            name: "from_image".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("alpine".to_string())),
                Expr::Literal(Literal::Str("3.18".to_string())),
            ],
        }),
        Stmt::Expr(Expr::FunctionCall {
            name: "copy_from".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("builder".to_string())),
                Expr::Literal(Literal::Str("/app/target/release/app".to_string())),
                Expr::Literal(Literal::Str("/usr/local/bin/".to_string())),
            ],
        }),
        Stmt::Expr(Expr::FunctionCall {
            name: "expose".to_string(),
            args: vec![Expr::Literal(Literal::U16(8080))],
        }),
        Stmt::Expr(Expr::FunctionCall {
            name: "user".to_string(),
            args: vec![Expr::Literal(Literal::Str("65534".to_string()))],
        }),
        Stmt::Expr(Expr::FunctionCall {
            name: "healthcheck".to_string(),
            args: vec![Expr::Literal(Literal::Str(
                "curl -f http://localhost:8080/health".to_string(),
            ))],
        }),
        Stmt::Expr(Expr::FunctionCall {
            name: "entrypoint".to_string(),
            args: vec![Expr::Array(vec![Expr::Literal(Literal::Str(
                "/usr/local/bin/app".to_string(),
            ))])],
        }),
    ]);

    let result = emit_dockerfile(&ast).unwrap();
    assert!(result.contains("FROM rust:1.75-alpine AS builder"));
    assert!(result.contains("FROM alpine:3.18"));
    assert!(result.contains("COPY --from=builder"));
    assert!(result.contains("EXPOSE 8080"));
    assert!(result.contains("USER 65534"));
    assert!(result.contains("HEALTHCHECK"));
    assert!(result.contains("ENTRYPOINT"));
}

mod convert_function_call_tests {
    use super::*;

    // ─── 1. from_image basic ─────────────────────────────────────────
    #[test]
    fn test_DOCKER_CONV_001_from_image_basic() {
        let ast = make_simple_ast(vec![Stmt::Expr(Expr::FunctionCall {
            name: "from_image".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("ubuntu".to_string())),
                Expr::Literal(Literal::Str("22.04".to_string())),
            ],
        })]);
        let result = emit_dockerfile(&ast).expect("should emit dockerfile");
        assert!(
            result.contains("FROM ubuntu:22.04"),
            "Expected FROM ubuntu:22.04, got: {result}"
        );
    }

    // ─── 2. from_image single arg (image:tag or image → image:latest) ─
    #[test]
    fn test_DOCKER_CONV_002_from_image_single_arg() {
        let ast = make_simple_ast(vec![Stmt::Expr(Expr::FunctionCall {
            name: "from_image".to_string(),
            args: vec![Expr::Literal(Literal::Str("ubuntu".to_string()))],
        })]);
        let result = emit_dockerfile(&ast).expect("single-arg from_image should succeed");
        assert!(
            result.contains("FROM ubuntu:latest"),
            "Expected FROM ubuntu:latest, got: {result}"
        );
    }

    // ─── 3. from_image_as basic ──────────────────────────────────────

    include!("dockerfile_tests_tests_DOCKER_DOCKER.rs");
}
