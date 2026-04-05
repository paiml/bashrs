    #[test]
    fn test_DOCKER_CONV_012_workdir_basic() {
        let ast = make_simple_ast(vec![
            Stmt::Expr(Expr::FunctionCall {
                name: "from_image".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("node".to_string())),
                    Expr::Literal(Literal::Str("20".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "workdir".to_string(),
                args: vec![Expr::Literal(Literal::Str("/app".to_string()))],
            }),
        ]);
        let result = emit_dockerfile(&ast).expect("should emit dockerfile");
        assert!(
            result.contains("WORKDIR /app"),
            "Expected WORKDIR /app, got: {result}"
        );
    }

    // ─── 13. env basic ──────────────────────────────────────────────
    #[test]
    fn test_DOCKER_CONV_013_env_basic() {
        let ast = make_simple_ast(vec![
            Stmt::Expr(Expr::FunctionCall {
                name: "from_image".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("python".to_string())),
                    Expr::Literal(Literal::Str("3.12".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "env".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("PYTHONUNBUFFERED".to_string())),
                    Expr::Literal(Literal::Str("1".to_string())),
                ],
            }),
        ]);
        let result = emit_dockerfile(&ast).expect("should emit dockerfile");
        assert!(
            result.contains("ENV PYTHONUNBUFFERED=1"),
            "Expected ENV KEY=value, got: {result}"
        );
    }

    // ─── 14. env too few args ────────────────────────────────────────
    #[test]
    fn test_DOCKER_CONV_014_env_insufficient_args() {
        let ast = make_simple_ast(vec![
            Stmt::Expr(Expr::FunctionCall {
                name: "from_image".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("python".to_string())),
                    Expr::Literal(Literal::Str("3.12".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "env".to_string(),
                args: vec![Expr::Literal(Literal::Str("KEY".to_string()))],
            }),
        ]);
        let err = emit_dockerfile(&ast).unwrap_err();
        let msg = format!("{err}");
        assert!(
            msg.contains("requires 2 arguments"),
            "Expected validation error, got: {msg}"
        );
    }

    // ─── 15. expose U16 ─────────────────────────────────────────────
    #[test]
    fn test_DOCKER_CONV_015_expose_u16() {
        let ast = make_simple_ast(vec![
            Stmt::Expr(Expr::FunctionCall {
                name: "from_image".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("nginx".to_string())),
                    Expr::Literal(Literal::Str("latest".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "expose".to_string(),
                args: vec![Expr::Literal(Literal::U16(8080))],
            }),
        ]);
        let result = emit_dockerfile(&ast).expect("should emit dockerfile");
        assert!(
            result.contains("EXPOSE 8080"),
            "Expected EXPOSE 8080, got: {result}"
        );
    }

    // ─── 16. expose U32 ─────────────────────────────────────────────
    #[test]
    fn test_DOCKER_CONV_016_expose_u32() {
        let ast = make_simple_ast(vec![
            Stmt::Expr(Expr::FunctionCall {
                name: "from_image".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("nginx".to_string())),
                    Expr::Literal(Literal::Str("latest".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "expose".to_string(),
                args: vec![Expr::Literal(Literal::U32(3000))],
            }),
        ]);
        let result = emit_dockerfile(&ast).expect("should emit dockerfile");
        assert!(
            result.contains("EXPOSE 3000"),
            "Expected EXPOSE 3000, got: {result}"
        );
    }

    // ─── 17. expose I32 ─────────────────────────────────────────────
    #[test]
    fn test_DOCKER_CONV_017_expose_i32() {
        let ast = make_simple_ast(vec![
            Stmt::Expr(Expr::FunctionCall {
                name: "from_image".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("nginx".to_string())),
                    Expr::Literal(Literal::Str("latest".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "expose".to_string(),
                args: vec![Expr::Literal(Literal::I32(443))],
            }),
        ]);
        let result = emit_dockerfile(&ast).expect("should emit dockerfile");
        assert!(
            result.contains("EXPOSE 443"),
            "Expected EXPOSE 443, got: {result}"
        );
    }

    // ─── 18. expose with string → error ─────────────────────────────
    #[test]
    fn test_DOCKER_CONV_018_expose_string_error() {
        let ast = make_simple_ast(vec![
            Stmt::Expr(Expr::FunctionCall {
                name: "from_image".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("nginx".to_string())),
                    Expr::Literal(Literal::Str("latest".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "expose".to_string(),
                args: vec![Expr::Literal(Literal::Str("not-a-port".to_string()))],
            }),
        ]);
        let err = emit_dockerfile(&ast).unwrap_err();
        let msg = format!("{err}");
        assert!(
            msg.contains("expose() requires an integer port number"),
            "Expected expose validation error, got: {msg}"
        );
    }

    // ─── 19. user basic ─────────────────────────────────────────────
    #[test]
    fn test_DOCKER_CONV_019_user_basic() {
        let ast = make_simple_ast(vec![
            Stmt::Expr(Expr::FunctionCall {
                name: "from_image".to_string(),
                args: vec![
                    Expr::Literal(Literal::Str("alpine".to_string())),
                    Expr::Literal(Literal::Str("3.18".to_string())),
                ],
            }),
            Stmt::Expr(Expr::FunctionCall {
                name: "user".to_string(),
                args: vec![Expr::Literal(Literal::Str("nobody".to_string()))],
            }),
        ]);
        let result = emit_dockerfile(&ast).expect("should emit dockerfile");
        assert!(
            result.contains("USER nobody"),
            "Expected USER nobody, got: {result}"
        );
    }

    // ─── 20. entrypoint basic ────────────────────────────────────────
    #[test]
    fn test_DOCKER_CONV_020_entrypoint_basic() {
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
                args: vec![Expr::Array(vec![
                    Expr::Literal(Literal::Str("/usr/bin/app".to_string())),
                    Expr::Literal(Literal::Str("--config".to_string())),
                ])],
            }),
        ]);
        let result = emit_dockerfile(&ast).expect("should emit dockerfile");
        assert!(
            result.contains("ENTRYPOINT"),
            "Expected ENTRYPOINT, got: {result}"
        );
        assert!(
            result.contains("/usr/bin/app"),
            "Expected app path in entrypoint, got: {result}"
        );
    }

    // ─── 21. cmd basic ──────────────────────────────────────────────

include!("dockerfile_tests_tests_DOCKER_DOCKER_DOCKER_DOCKER.rs");
