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
