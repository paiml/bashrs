    #[test]
    fn test_DOCKER_CONV_021_cmd_basic() {
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
                    Expr::Literal(Literal::Str("--port".to_string())),
                    Expr::Literal(Literal::Str("8080".to_string())),
                ])],
            }),
        ]);
        let result = emit_dockerfile(&ast).expect("should emit dockerfile");
        assert!(
            result.contains("CMD"),
            "Expected CMD instruction, got: {result}"
        );
        assert!(
            result.contains("--port"),
            "Expected --port in CMD, got: {result}"
        );
    }

    // ─── 22. label basic ─────────────────────────────────────────────
    #[test]
    fn test_DOCKER_CONV_022_label_basic() {
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
                    Expr::Literal(Literal::Str("team@example.com".to_string())),
                ],
            }),
        ]);
        let result = emit_dockerfile(&ast).expect("should emit dockerfile");
        assert!(
            result.contains("LABEL maintainer="),
            "Expected LABEL key=value, got: {result}"
        );
        assert!(
            result.contains("team@example.com"),
            "Expected value in LABEL, got: {result}"
        );
    }

    // ─── 23. label too few args ──────────────────────────────────────
    #[test]
    fn test_DOCKER_CONV_023_label_insufficient_args() {
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
                args: vec![Expr::Literal(Literal::Str("version".to_string()))],
            }),
        ]);
        let err = emit_dockerfile(&ast).unwrap_err();
        let msg = format!("{err}");
        assert!(
            msg.contains("label() requires 2 arguments"),
            "Expected validation error, got: {msg}"
        );
    }

    // ─── 24. healthcheck basic ───────────────────────────────────────
    #[test]
    fn test_DOCKER_CONV_024_healthcheck_basic() {
        let ast = make_simple_ast(vec![
            Stmt::Expr(Expr::FunctionCall {
                name: "from_image".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("nginx".to_string())),
                    Expr::Literal(Literal::Str("latest".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "healthcheck".to_string(),
                args: vec![Expr::Literal(Literal::Str(
                    "curl -f http://localhost/".to_string(),
                ))],
            }),
        ]);
        let result = emit_dockerfile(&ast).expect("should emit dockerfile");
        assert!(
            result.contains("HEALTHCHECK CMD curl -f http://localhost/"),
            "Expected HEALTHCHECK CMD, got: {result}"
        );
    }

    // ─── 25. comment basic ───────────────────────────────────────────
    #[test]
    fn test_DOCKER_CONV_025_comment_basic() {
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
                args: vec![Expr::Literal(Literal::Str(
                    "Install dependencies".to_string(),
                ))],
            }),
        ]);
        let result = emit_dockerfile(&ast).expect("should emit dockerfile");
        assert!(
            result.contains("# Install dependencies"),
            "Expected # comment, got: {result}"
        );
    }

    // ─── 26. unknown function → ignored ──────────────────────────────
    #[test]
    fn test_DOCKER_CONV_026_unknown_function_ignored() {
        let ast = make_simple_ast(vec![
            Stmt::Expr(Expr::FunctionCall {
                name: "from_image".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("alpine".to_string())),
                    Expr::Literal(Literal::Str("3.18".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "totally_unknown_func".to_string(),
                args: vec![Expr::Literal(Literal::Str("ignored".to_string()))],
            }),
        ]);
        let result = emit_dockerfile(&ast).expect("should succeed (unknown is ignored)");
        assert!(
            result.contains("FROM alpine:3.18"),
            "Expected FROM, got: {result}"
        );
        assert!(
            !result.contains("totally_unknown_func"),
            "Unknown function should not appear in output"
        );
        assert!(
            !result.contains("ignored"),
            "Args to unknown function should not appear in output"
        );
    }

    // ─── 27. multiple instructions in one stage ──────────────────────
    #[test]
    fn test_DOCKER_CONV_027_multiple_instructions_single_stage() {
        let ast = make_simple_ast(vec![
            Stmt::Expr(Expr::FunctionCall {
                name: "from_image".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("python".to_string())),
                    Expr::Literal(Literal::Str("3.12-slim".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "workdir".to_string(),
                args: vec![Expr::Literal(Literal::Str("/app".to_string()))],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "copy".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("requirements.txt".to_string())),
                    Expr::Literal(Literal::Str("/app/requirements.txt".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "run".to_string(),
                args: vec![Expr::Array(vec![Expr::Literal(Literal::Str(
                    "pip install -r requirements.txt".to_string(),
                ))])],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "copy".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str(".".to_string())),
                    Expr::Literal(Literal::Str("/app".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "env".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("PYTHONUNBUFFERED".to_string())),
                    Expr::Literal(Literal::Str("1".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "expose".to_string(),
                args: vec![Expr::Literal(Literal::U16(8000))],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "cmd".to_string(),
                args: vec![Expr::Array(vec![
                    Expr::Literal(Literal::Str("python".to_string())),
                    Expr::Literal(Literal::Str("app.py".to_string())),
                ])],
            }),
        ]);
        let result = emit_dockerfile(&ast).expect("should emit dockerfile");
        assert!(result.contains("FROM python:3.12-slim"));
        assert!(result.contains("WORKDIR /app"));
        assert!(result.contains("COPY requirements.txt /app/requirements.txt"));
        assert!(result.contains("RUN"));
        assert!(result.contains("COPY . /app"));
        assert!(result.contains("ENV PYTHONUNBUFFERED=1"));
        assert!(result.contains("EXPOSE 8000"));
        assert!(result.contains("CMD"));
    }

    // ─── 28. multi-stage with from_image_as + copy_from ──────────────
    #[test]
    fn test_DOCKER_CONV_028_multi_stage_build_with_copy_from() {
        let ast = make_simple_ast(vec![
            // Builder stage
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
                args: vec![Expr::Literal(Literal::Str("/build".to_string()))],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "copy".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str(".".to_string())),
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
                    Expr::Literal(Literal::Str("/build/target/release/app".to_string())),
                    Expr::Literal(Literal::Str("/usr/local/bin/app".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "user".to_string(),
                args: vec![Expr::Literal(Literal::Str("65534".to_string()))],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "entrypoint".to_string(),
                args: vec![Expr::Array(vec![Expr::Literal(Literal::Str(
                    "/usr/local/bin/app".to_string(),
                ))])],
            }),
        ]);
        let result = emit_dockerfile(&ast).expect("should emit dockerfile");
        assert!(
            result.contains("FROM rust:1.75-alpine AS builder"),
            "Expected builder stage, got: {result}"
        );
        assert!(
            result.contains("FROM alpine:3.18"),
            "Expected runtime stage, got: {result}"
        );
        assert!(
            result.contains("COPY --from=builder /build/target/release/app /usr/local/bin/app"),
            "Expected COPY --from=builder, got: {result}"
        );
        assert!(result.contains("USER 65534"));
        assert!(result.contains("ENTRYPOINT"));
    }
}
