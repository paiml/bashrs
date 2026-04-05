fn test_DOCKER_BUILD_015_let_binding_becomes_env() {
    let ast = make_simple_ast(vec![
        Stmt::Expr(Expr::FunctionCall {
            name: "from_image".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("python".to_string())),
                Expr::Literal(Literal::Str("3.12".to_string())),
            ],
        }),
        Stmt::Let {
            name: "app_port".to_string(),
            value: Expr::Literal(Literal::Str("8080".to_string())),
            declaration: true,
        },
    ]);

    let result = emit_dockerfile(&ast).unwrap();
    assert!(result.contains("ENV APP_PORT=8080"), "Let→ENV in: {result}");
}

#[test]
fn test_DOCKER_BUILD_016_no_from_image_error() {
    let ast = make_simple_ast(vec![Stmt::Expr(Expr::FunctionCall {
        name: "workdir".to_string(),
        args: vec![Expr::Literal(Literal::Str("/app".to_string()))],
    })]);

    let err = emit_dockerfile(&ast).unwrap_err();
    assert!(
        format!("{err}").contains("from_image"),
        "Error should mention from_image: {err}"
    );
}

#[test]
fn test_DOCKER_BUILD_017_from_image_single_arg() {
    // Single arg from_image("alpine") → FROM alpine:latest
    let ast = make_simple_ast(vec![Stmt::Expr(Expr::FunctionCall {
        name: "from_image".to_string(),
        args: vec![Expr::Literal(Literal::Str("alpine".to_string()))],
    })]);

    let result = emit_dockerfile(&ast).expect("single-arg from_image should succeed");
    assert!(
        result.contains("FROM alpine:latest"),
        "Expected FROM alpine:latest, got: {result}"
    );
}

#[test]
fn test_DOCKER_BUILD_018_from_image_as_two_args() {
    // Two-arg from_image_as("rust:1.75", "builder") → FROM rust:1.75 AS builder
    let ast = make_simple_ast(vec![Stmt::Expr(Expr::FunctionCall {
        name: "from_image_as".to_string(),
        args: vec![
            Expr::Literal(Literal::Str("rust:1.75".to_string())),
            Expr::Literal(Literal::Str("builder".to_string())),
        ],
    })]);

    let result = emit_dockerfile(&ast).expect("2-arg from_image_as should succeed");
    assert!(
        result.contains("FROM rust:1.75 AS builder"),
        "Expected FROM rust:1.75 AS builder, got: {result}"
    );
}

#[test]
fn test_DOCKER_BUILD_019_copy_too_few_args() {
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
            args: vec![Expr::Literal(Literal::Str(".".to_string()))],
        }),
    ]);

    let err = emit_dockerfile(&ast).unwrap_err();
    assert!(format!("{err}").contains("2 arguments"), "Error: {err}");
}

#[test]
fn test_DOCKER_BUILD_020_copy_from_too_few_args() {
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
                Expr::Literal(Literal::Str("/app".to_string())),
            ],
        }),
    ]);

    let err = emit_dockerfile(&ast).unwrap_err();
    assert!(format!("{err}").contains("3 arguments"), "Error: {err}");
}

#[test]
fn test_DOCKER_BUILD_021_env_too_few_args() {
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
fn test_DOCKER_BUILD_022_expose_invalid_type() {
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
            args: vec![Expr::Literal(Literal::Str("not-a-port".to_string()))],
        }),
    ]);

    let err = emit_dockerfile(&ast).unwrap_err();
    assert!(format!("{err}").contains("integer"), "Error: {err}");
}

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

include!("dockerfile_tests_tests_DOCKER_2.rs");
