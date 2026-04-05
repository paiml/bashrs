    #[test]
    fn test_DOCKER_CONV_003_from_image_as_basic() {
        let ast = make_simple_ast(vec![Stmt::Expr(Expr::FunctionCall {
            name: "from_image_as".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("node".to_string())),
                Expr::Literal(Literal::Str("20-slim".to_string())),
                Expr::Literal(Literal::Str("builder".to_string())),
            ],
        })]);
        let result = emit_dockerfile(&ast).expect("should emit dockerfile");
        assert!(
            result.contains("FROM node:20-slim AS builder"),
            "Expected FROM with AS alias, got: {result}"
        );
    }

    // ─── 4. from_image_as 2-arg (image:tag, alias) ────────────────────
    #[test]
    fn test_DOCKER_CONV_004_from_image_as_two_args() {
        let ast = make_simple_ast(vec![Stmt::Expr(Expr::FunctionCall {
            name: "from_image_as".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("node:20".to_string())),
                Expr::Literal(Literal::Str("builder".to_string())),
            ],
        })]);
        let result = emit_dockerfile(&ast).expect("2-arg from_image_as should succeed");
        assert!(
            result.contains("FROM node:20 AS builder"),
            "Expected FROM node:20 AS builder, got: {result}"
        );
    }

    // ─── 5. two from_image calls push stages ────────────────────────
    #[test]
    fn test_DOCKER_CONV_005_from_image_pushes_previous_stage() {
        let ast = make_simple_ast(vec![
            Stmt::Expr(Expr::FunctionCall {
                name: "from_image".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("rust".to_string())),
                    Expr::Literal(Literal::Str("1.75".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "from_image".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("alpine".to_string())),
                    Expr::Literal(Literal::Str("3.18".to_string())),
                ],
            }),
        ]);
        let result = emit_dockerfile(&ast).expect("should emit dockerfile");
        assert!(
            result.contains("FROM rust:1.75"),
            "Expected first stage, got: {result}"
        );
        assert!(
            result.contains("FROM alpine:3.18"),
            "Expected second stage, got: {result}"
        );
    }

    // ─── 6. run with array ───────────────────────────────────────────
    #[test]
    fn test_DOCKER_CONV_006_run_with_array() {
        let ast = make_simple_ast(vec![
            Stmt::Expr(Expr::FunctionCall {
                name: "from_image".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("ubuntu".to_string())),
                    Expr::Literal(Literal::Str("22.04".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "run".to_string(),
                args: vec![Expr::Array(vec![
                    Expr::Literal(Literal::Str("apt-get".to_string())),
                    Expr::Literal(Literal::Str("update".to_string())),
                ])],
            }),
        ]);
        let result = emit_dockerfile(&ast).expect("should emit dockerfile");
        assert!(
            result.contains("RUN"),
            "Expected RUN instruction, got: {result}"
        );
        assert!(
            result.contains("apt-get"),
            "Expected apt-get in RUN, got: {result}"
        );
    }

    // ─── 7. run with single string arg ───────────────────────────────
    #[test]
    fn test_DOCKER_CONV_007_run_with_single_string() {
        let ast = make_simple_ast(vec![
            Stmt::Expr(Expr::FunctionCall {
                name: "from_image".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("ubuntu".to_string())),
                    Expr::Literal(Literal::Str("22.04".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "run".to_string(),
                args: vec![Expr::Literal(Literal::Str("echo hello".to_string()))],
            }),
        ]);
        let result = emit_dockerfile(&ast).expect("should emit dockerfile");
        assert!(
            result.contains("RUN"),
            "Expected RUN instruction, got: {result}"
        );
    }

    // ─── 8. copy basic ──────────────────────────────────────────────
    #[test]
    fn test_DOCKER_CONV_008_copy_basic() {
        let ast = make_simple_ast(vec![
            Stmt::Expr(Expr::FunctionCall {
                name: "from_image".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("rust".to_string())),
                    Expr::Literal(Literal::Str("latest".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "copy".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("Cargo.toml".to_string())),
                    Expr::Literal(Literal::Str("/app/Cargo.toml".to_string())),
                ],
            }),
        ]);
        let result = emit_dockerfile(&ast).expect("should emit dockerfile");
        assert!(
            result.contains("COPY Cargo.toml /app/Cargo.toml"),
            "Expected COPY src dst, got: {result}"
        );
    }

    // ─── 9. copy too few args ────────────────────────────────────────
    #[test]
    fn test_DOCKER_CONV_009_copy_insufficient_args() {
        let ast = make_simple_ast(vec![
            Stmt::Expr(Expr::FunctionCall {
                name: "from_image".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("rust".to_string())),
                    Expr::Literal(Literal::Str("latest".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "copy".to_string(),
                args: vec![Expr::Literal(Literal::Str("src".to_string()))],
            }),
        ]);
        let err = emit_dockerfile(&ast).unwrap_err();
        let msg = format!("{err}");
        assert!(
            msg.contains("2 arguments"),
            "Expected validation error, got: {msg}"
        );
    }

    // ─── 10. copy_from basic ─────────────────────────────────────────
    #[test]
    fn test_DOCKER_CONV_010_copy_from_basic() {
        let ast = make_simple_ast(vec![
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
                    Expr::Literal(Literal::Str("/app/target/release/myapp".to_string())),
                    Expr::Literal(Literal::Str("/usr/local/bin/myapp".to_string())),
                ],
            }),
        ]);
        let result = emit_dockerfile(&ast).expect("should emit dockerfile");
        assert!(
            result.contains("COPY --from=builder /app/target/release/myapp /usr/local/bin/myapp"),
            "Expected COPY --from=builder, got: {result}"
        );
    }

    // ─── 11. copy_from too few args ──────────────────────────────────
    #[test]
    fn test_DOCKER_CONV_011_copy_from_insufficient_args() {
        let ast = make_simple_ast(vec![
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
                    Expr::Literal(Literal::Str("/app/bin".to_string())),
                ],
            }),
        ]);
        let err = emit_dockerfile(&ast).unwrap_err();
        let msg = format!("{err}");
        assert!(
            msg.contains("copy_from() requires 3 arguments"),
            "Expected validation error, got: {msg}"
        );
    }

    // ─── 12. workdir basic ───────────────────────────────────────────

include!("dockerfile_tests_tests_DOCKER_DOCKER_DOCKER.rs");
