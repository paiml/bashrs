#[test]
fn test_DOCKER_BUILD_023_label_too_few_args() {
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
            args: vec![Expr::Literal(Literal::Str("key".to_string()))],
        }),
    ]);

    let err = emit_dockerfile(&ast).unwrap_err();
    assert!(format!("{err}").contains("2 arguments"), "Error: {err}");
}

#[test]
fn test_DOCKER_BUILD_024_variable_in_expr() {
    let ast = make_simple_ast(vec![
        Stmt::Expr(Expr::FunctionCall {
            name: "from_image".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("alpine".to_string())),
                Expr::Literal(Literal::Str("3.18".to_string())),
            ],
        }),
        Stmt::Expr(Expr::FunctionCall {
            name: "env".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("PATH".to_string())),
                Expr::Variable("app_dir".to_string()),
            ],
        }),
    ]);

    let result = emit_dockerfile(&ast).unwrap();
    assert!(result.contains("${APP_DIR}"), "Variable ref in: {result}");
}

// --- Coverage tests for uncovered lines ---

#[test]
fn test_DOCKER_COV_001_convert_stmt_return_catchall() {
    // Line 86: _ => Ok(()) in convert_stmt — Stmt::Return is not Expr or Let
    let ast = make_simple_ast(vec![
        Stmt::Expr(Expr::FunctionCall {
            name: "from_image".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("alpine".to_string())),
                Expr::Literal(Literal::Str("3.18".to_string())),
            ],
        }),
        Stmt::Return(None),
    ]);

    let result = emit_dockerfile(&ast).unwrap();
    assert!(result.contains("FROM alpine:3.18"));
}

#[test]
fn test_DOCKER_COV_002_convert_expr_non_functioncall() {
    // Line 100: _ => Ok(()) in convert_expr — Expr::Variable is not FunctionCall
    let ast = make_simple_ast(vec![
        Stmt::Expr(Expr::FunctionCall {
            name: "from_image".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("alpine".to_string())),
                Expr::Literal(Literal::Str("3.18".to_string())),
            ],
        }),
        Stmt::Expr(Expr::Variable("ignored".to_string())),
    ]);

    let result = emit_dockerfile(&ast).unwrap();
    assert!(result.contains("FROM alpine:3.18"));
}

#[test]
fn test_DOCKER_COV_003_multi_stage_from_image_as_twice() {
    // Line 142: ir.add_stage(stage) in from_image_as when existing stage
    let ast = make_simple_ast(vec![
        Stmt::Expr(Expr::FunctionCall {
            name: "from_image_as".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("rust".to_string())),
                Expr::Literal(Literal::Str("1.75".to_string())),
                Expr::Literal(Literal::Str("builder".to_string())),
            ],
        }),
        Stmt::Expr(Expr::FunctionCall {
            name: "run".to_string(),
            args: vec![Expr::Literal(Literal::Str("cargo build".to_string()))],
        }),
        Stmt::Expr(Expr::FunctionCall {
            name: "from_image_as".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("alpine".to_string())),
                Expr::Literal(Literal::Str("3.18".to_string())),
                Expr::Literal(Literal::Str("runtime".to_string())),
            ],
        }),
    ]);

    let result = emit_dockerfile(&ast).unwrap();
    assert!(result.contains("FROM rust:1.75 AS builder"));
    assert!(result.contains("FROM alpine:3.18 AS runtime"));
}

#[test]
fn test_DOCKER_COV_004_unknown_function_call() {
    // Line 288: _ => Ok(()) in convert_function_call — unknown function name
    let ast = make_simple_ast(vec![
        Stmt::Expr(Expr::FunctionCall {
            name: "from_image".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("alpine".to_string())),
                Expr::Literal(Literal::Str("3.18".to_string())),
            ],
        }),
        Stmt::Expr(Expr::FunctionCall {
            name: "unknown_instruction".to_string(),
            args: vec![Expr::Literal(Literal::Str("arg".to_string()))],
        }),
    ]);

    let result = emit_dockerfile(&ast).unwrap();
    assert!(result.contains("FROM alpine:3.18"));
}

#[test]
fn test_DOCKER_COV_005_extract_string_args_non_array() {
    // Line 302: _ => in extract_string_args — non-Array arg
    let ast = make_simple_ast(vec![
        Stmt::Expr(Expr::FunctionCall {
            name: "from_image".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("alpine".to_string())),
                Expr::Literal(Literal::Str("3.18".to_string())),
            ],
        }),
        Stmt::Expr(Expr::FunctionCall {
            name: "run".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("apt-get update".to_string())),
                Expr::Literal(Literal::Str("apt-get install -y curl".to_string())),
            ],
        }),
    ]);

    let result = emit_dockerfile(&ast).unwrap();
    assert!(result.contains("RUN"), "RUN in: {result}");
}

#[test]
fn test_DOCKER_COV_006_expr_to_string_u16() {
    // Line 312: Literal::U16 in expr_to_string
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
                Expr::Literal(Literal::Str("port".to_string())),
                Expr::Literal(Literal::U16(8080)),
            ],
        }),
    ]);

    let result = emit_dockerfile(&ast).unwrap();
    assert!(result.contains("8080"), "U16 label in: {result}");
}

#[test]
fn test_DOCKER_COV_007_expr_to_string_u32() {
    // Line 313: Literal::U32 in expr_to_string
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
                Expr::Literal(Literal::Str("size".to_string())),
                Expr::Literal(Literal::U32(65536)),
            ],
        }),
    ]);

    let result = emit_dockerfile(&ast).unwrap();
    assert!(result.contains("65536"), "U32 label in: {result}");
}

#[test]
fn test_DOCKER_COV_008_expr_to_string_i32() {
    // Line 314: Literal::I32 in expr_to_string
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
                Expr::Literal(Literal::Str("count".to_string())),
                Expr::Literal(Literal::I32(-1)),
            ],
        }),
    ]);

    let result = emit_dockerfile(&ast).unwrap();
    assert!(result.contains("-1"), "I32 label in: {result}");
}

#[test]
fn test_DOCKER_COV_009_expr_to_string_bool() {
    // Line 315: Literal::Bool in expr_to_string
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
                Expr::Literal(Literal::Str("debug".to_string())),
                Expr::Literal(Literal::Bool(true)),
            ],
        }),
    ]);

    let result = emit_dockerfile(&ast).unwrap();
    assert!(result.contains("true"), "Bool label in: {result}");
}

#[test]
fn test_DOCKER_COV_010_expr_to_string_error() {
    // Lines 317-319: catch-all _ => Err(...) in expr_to_string
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
                Expr::Literal(Literal::Str("key".to_string())),
                Expr::Array(vec![Expr::Literal(Literal::Str("a".to_string()))]),
            ],
        }),
    ]);

    let err = emit_dockerfile(&ast).unwrap_err();
    assert!(
        format!("{err}").contains("Cannot convert expression"),
        "Error: {err}"
    );
}

#[test]
fn test_DOCKER_COV_011_instructions_before_from_image() {
    // Lines 196, 227, 236, 276, 285: instructions when current_stage is None
    // workdir, expose, user, healthcheck, comment all silently skip
    let ast = make_simple_ast(vec![
        Stmt::Expr(Expr::FunctionCall {
            name: "workdir".to_string(),
            args: vec![Expr::Literal(Literal::Str("/app".to_string()))],
        }),
        Stmt::Expr(Expr::FunctionCall {
            name: "expose".to_string(),
            args: vec![Expr::Literal(Literal::U16(8080))],
        }),
        Stmt::Expr(Expr::FunctionCall {
            name: "user".to_string(),
            args: vec![Expr::Literal(Literal::Str("root".to_string()))],
        }),
        Stmt::Expr(Expr::FunctionCall {
            name: "healthcheck".to_string(),
            args: vec![Expr::Literal(Literal::Str("curl localhost".to_string()))],
        }),
        Stmt::Expr(Expr::FunctionCall {
            name: "comment".to_string(),
            args: vec![Expr::Literal(Literal::Str("test".to_string()))],
        }),
        // Now add from_image so we don't get "no stages" error
        Stmt::Expr(Expr::FunctionCall {
            name: "from_image".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("alpine".to_string())),
                Expr::Literal(Literal::Str("3.18".to_string())),
            ],
        }),
    ]);

    let result = emit_dockerfile(&ast).unwrap();
    assert!(result.contains("FROM alpine:3.18"));
    // Instructions before from_image had no stage → silently skipped
    assert!(!result.contains("WORKDIR"), "No WORKDIR before FROM");
}

#[test]
fn test_DOCKER_COV_012_instructions_with_empty_args() {
    // Lines 196, 227, 236, 276, 285: implicit else when args is empty
    let ast = make_simple_ast(vec![
        Stmt::Expr(Expr::FunctionCall {
            name: "from_image".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("alpine".to_string())),
                Expr::Literal(Literal::Str("3.18".to_string())),
            ],
        }),
        Stmt::Expr(Expr::FunctionCall {
            name: "workdir".to_string(),
            args: vec![],
        }),
        Stmt::Expr(Expr::FunctionCall {
            name: "expose".to_string(),
            args: vec![],
        }),
        Stmt::Expr(Expr::FunctionCall {
            name: "user".to_string(),
            args: vec![],
        }),
        Stmt::Expr(Expr::FunctionCall {
            name: "healthcheck".to_string(),
            args: vec![],
        }),
        Stmt::Expr(Expr::FunctionCall {
            name: "comment".to_string(),
            args: vec![],
        }),
    ]);

    let result = emit_dockerfile(&ast).unwrap();
    assert!(result.contains("FROM alpine:3.18"));
    // Empty args means no instructions generated
    assert!(!result.contains("WORKDIR"), "No WORKDIR with empty args");
    assert!(!result.contains("EXPOSE"), "No EXPOSE with empty args");
    assert!(!result.contains("USER"), "No USER with empty args");
    assert!(
        !result.contains("HEALTHCHECK"),
        "No HEALTHCHECK with empty args"
    );
}

// ============================================================================
// Coverage Tests - convert_function_call branches (DOCKER_COV_013-025)
// ============================================================================

#[test]
fn test_DOCKER_COV_013_from_image_single_arg_with_tag() {
    // Single arg from_image("alpine:3.18") → FROM alpine:3.18
    let ast = make_simple_ast(vec![Stmt::Expr(Expr::FunctionCall {
        name: "from_image".to_string(),
        args: vec![Expr::Literal(Literal::Str("alpine:3.18".to_string()))],
    })]);
    let result = emit_dockerfile(&ast).expect("single-arg with tag should succeed");
    assert!(
        result.contains("FROM alpine:3.18"),
        "Expected FROM alpine:3.18, got: {result}"
    );
}

#[test]
fn test_DOCKER_COV_014_from_image_as_two_args_no_tag() {
    // Two-arg from_image_as("rust", "builder") → FROM rust:latest AS builder
    let ast = make_simple_ast(vec![Stmt::Expr(Expr::FunctionCall {
        name: "from_image_as".to_string(),
        args: vec![
            Expr::Literal(Literal::Str("rust".to_string())),
            Expr::Literal(Literal::Str("builder".to_string())),
        ],
    })]);
    let result = emit_dockerfile(&ast).expect("2-arg from_image_as should succeed");
    assert!(
        result.contains("FROM rust:latest AS builder"),
        "Expected FROM rust:latest AS builder, got: {result}"
    );
}

#[test]
fn test_DOCKER_COV_015_copy_too_few_args() {
    let ast = make_simple_ast(vec![
        Stmt::Expr(Expr::FunctionCall {
            name: "from_image".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("alpine".to_string())),
                Expr::Literal(Literal::Str("3.18".to_string())),
            ],
        }),
        Stmt::Expr(Expr::FunctionCall {
            name: "copy".to_string(),
            args: vec![Expr::Literal(Literal::Str("src".to_string()))],
        }),
    ]);
    let err = emit_dockerfile(&ast).unwrap_err();
    assert!(format!("{err}").contains("2 arguments"), "Error: {err}");
}

#[test]
fn test_DOCKER_COV_016_copy_from_too_few_args() {
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
                Expr::Literal(Literal::Str("/src".to_string())),
            ],
        }),
    ]);
    let err = emit_dockerfile(&ast).unwrap_err();
    assert!(format!("{err}").contains("3 arguments"), "Error: {err}");
}

#[test]
fn test_DOCKER_COV_017_env_too_few_args() {
    let ast = make_simple_ast(vec![
        Stmt::Expr(Expr::FunctionCall {
            name: "from_image".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("alpine".to_string())),
                Expr::Literal(Literal::Str("3.18".to_string())),
            ],
        }),
        Stmt::Expr(Expr::FunctionCall {
            name: "env".to_string(),
            args: vec![Expr::Literal(Literal::Str("KEY".to_string()))],
        }),
    ]);
    let err = emit_dockerfile(&ast).unwrap_err();
    assert!(format!("{err}").contains("2 arguments"), "Error: {err}");
}

#[test]
fn test_DOCKER_COV_018_expose_u32() {
    let ast = make_simple_ast(vec![
        Stmt::Expr(Expr::FunctionCall {
            name: "from_image".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("alpine".to_string())),
                Expr::Literal(Literal::Str("3.18".to_string())),
            ],
        }),
        Stmt::Expr(Expr::FunctionCall {
            name: "expose".to_string(),
            args: vec![Expr::Literal(Literal::U32(3000))],
        }),
    ]);
    let result = emit_dockerfile(&ast).unwrap();
    assert!(result.contains("EXPOSE 3000"), "U32 expose in: {result}");
}

#[test]
fn test_DOCKER_COV_019_expose_i32() {
    let ast = make_simple_ast(vec![
        Stmt::Expr(Expr::FunctionCall {
            name: "from_image".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("alpine".to_string())),
                Expr::Literal(Literal::Str("3.18".to_string())),
            ],
        }),
        Stmt::Expr(Expr::FunctionCall {
            name: "expose".to_string(),
            args: vec![Expr::Literal(Literal::I32(9090))],
        }),
    ]);
    let result = emit_dockerfile(&ast).unwrap();
    assert!(result.contains("EXPOSE 9090"), "I32 expose in: {result}");
}
