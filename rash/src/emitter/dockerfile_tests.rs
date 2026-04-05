#[allow(clippy::unwrap_used)]
use super::*;
use crate::ast::restricted::{Function, Type};

fn make_simple_ast(stmts: Vec<Stmt>) -> RestrictedAst {
    RestrictedAst {
        functions: vec![Function {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::Void,
            body: stmts,
        }],
        entry_point: "main".to_string(),
    }
}

#[test]
fn test_DOCKER_BUILD_001_basic_generation() {
    let ast = make_simple_ast(vec![
        Stmt::Expr(Expr::FunctionCall {
            name: "from_image".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("rust".to_string())),
                Expr::Literal(Literal::Str("1.75-alpine".to_string())),
            ],
        }),
        Stmt::Expr(Expr::FunctionCall {
            name: "workdir".to_string(),
            args: vec![Expr::Literal(Literal::Str("/app".to_string()))],
        }),
        Stmt::Expr(Expr::FunctionCall {
            name: "copy".to_string(),
            args: vec![
                Expr::Literal(Literal::Str(".".to_string())),
                Expr::Literal(Literal::Str(".".to_string())),
            ],
        }),
        Stmt::Expr(Expr::FunctionCall {
            name: "user".to_string(),
            args: vec![Expr::Literal(Literal::Str("65534".to_string()))],
        }),
    ]);

    let result = emit_dockerfile(&ast).unwrap();
    assert!(
        result.contains("FROM rust:1.75-alpine"),
        "Expected FROM in: {}",
        result
    );
    assert!(result.contains("WORKDIR /app"));
    assert!(result.contains("COPY . ."));
    assert!(result.contains("USER 65534"));
}

#[test]
fn test_DOCKER_BUILD_002_multi_stage() {
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
            args: vec![Expr::Literal(Literal::Str("/app".to_string()))],
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
                Expr::Literal(Literal::Str("/app/bin".to_string())),
                Expr::Literal(Literal::Str("/usr/local/bin/".to_string())),
            ],
        }),
    ]);

    let result = emit_dockerfile(&ast).unwrap();
    assert!(result.contains("FROM rust:1.75-alpine AS builder"));
    assert!(result.contains("FROM alpine:3.18"));
    assert!(result.contains("COPY --from=builder /app/bin /usr/local/bin/"));
}

#[test]
fn test_DOCKER_BUILD_003_run_chaining() {
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
                Expr::Literal(Literal::Str("apt-get update".to_string())),
                Expr::Literal(Literal::Str("apt-get install -y curl".to_string())),
                Expr::Literal(Literal::Str("rm -rf /var/lib/apt/lists/*".to_string())),
            ])],
        }),
    ]);

    let result = emit_dockerfile(&ast).unwrap();
    assert!(result.contains("RUN apt-get update && \\\n"));
}

#[test]
fn test_DOCKER_BUILD_004_entrypoint_exec_form() {
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
            args: vec![Expr::Array(vec![Expr::Literal(Literal::Str(
                "/app".to_string(),
            ))])],
        }),
    ]);

    let result = emit_dockerfile(&ast).unwrap();
    assert!(result.contains("ENTRYPOINT [\"/app\"]"));
}

#[test]
fn test_DOCKER_BUILD_005_user_directive() {
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
            args: vec![Expr::Literal(Literal::Str("65534".to_string()))],
        }),
    ]);

    let result = emit_dockerfile(&ast).unwrap();
    assert!(
        result.contains("USER 65534"),
        "USER directive must be present for DOCKER003 compliance"
    );
}

#[test]
fn test_DOCKER_BUILD_006_no_latest_tag() {
    let ast = make_simple_ast(vec![Stmt::Expr(Expr::FunctionCall {
        name: "from_image".to_string(),
        args: vec![
            Expr::Literal(Literal::Str("rust".to_string())),
            Expr::Literal(Literal::Str("1.75-alpine".to_string())),
        ],
    })]);

    let result = emit_dockerfile(&ast).unwrap();
    assert!(
        !result.contains(":latest"),
        "Generated Dockerfile should use pinned versions (DOCKER002 compliance)"
    );
    assert!(result.contains("FROM rust:1.75-alpine"));
}

#[test]
fn test_DOCKER_BUILD_007_env_directive() {
    let ast = make_simple_ast(vec![
        Stmt::Expr(Expr::FunctionCall {
            name: "from_image".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("node".to_string())),
                Expr::Literal(Literal::Str("20-alpine".to_string())),
            ],
        }),
        Stmt::Expr(Expr::FunctionCall {
            name: "env".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("NODE_ENV".to_string())),
                Expr::Literal(Literal::Str("production".to_string())),
            ],
        }),
    ]);

    let result = emit_dockerfile(&ast).unwrap();
    assert!(
        result.contains("ENV NODE_ENV=production"),
        "ENV directive in: {result}"
    );
}

#[test]
fn test_DOCKER_BUILD_008_expose_u16() {
    let ast = make_simple_ast(vec![
        Stmt::Expr(Expr::FunctionCall {
            name: "from_image".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("nginx".to_string())),
                Expr::Literal(Literal::Str("alpine".to_string())),
            ],
        }),
        Stmt::Expr(Expr::FunctionCall {
            name: "expose".to_string(),
            args: vec![Expr::Literal(Literal::U16(8080))],
        }),
    ]);

    let result = emit_dockerfile(&ast).unwrap();
    assert!(result.contains("EXPOSE 8080"), "EXPOSE in: {result}");
}

#[test]
fn test_DOCKER_BUILD_009_expose_u32() {
    let ast = make_simple_ast(vec![
        Stmt::Expr(Expr::FunctionCall {
            name: "from_image".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("nginx".to_string())),
                Expr::Literal(Literal::Str("alpine".to_string())),
            ],
        }),
        Stmt::Expr(Expr::FunctionCall {
            name: "expose".to_string(),
            args: vec![Expr::Literal(Literal::U32(3000))],
        }),
    ]);

    let result = emit_dockerfile(&ast).unwrap();
    assert!(result.contains("EXPOSE 3000"), "EXPOSE U32 in: {result}");
}

#[test]
fn test_DOCKER_BUILD_010_expose_i32() {
    let ast = make_simple_ast(vec![
        Stmt::Expr(Expr::FunctionCall {
            name: "from_image".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("nginx".to_string())),
                Expr::Literal(Literal::Str("alpine".to_string())),
            ],
        }),
        Stmt::Expr(Expr::FunctionCall {
            name: "expose".to_string(),
            args: vec![Expr::Literal(Literal::I32(443))],
        }),
    ]);

    let result = emit_dockerfile(&ast).unwrap();
    assert!(result.contains("EXPOSE 443"), "EXPOSE I32 in: {result}");
}

#[test]
fn test_DOCKER_BUILD_011_cmd_directive() {
    let ast = make_simple_ast(vec![
        Stmt::Expr(Expr::FunctionCall {
            name: "from_image".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("node".to_string())),
                Expr::Literal(Literal::Str("20".to_string())),
            ],
        }),
        Stmt::Expr(Expr::FunctionCall {
            name: "cmd".to_string(),
            args: vec![Expr::Array(vec![
                Expr::Literal(Literal::Str("node".to_string())),
                Expr::Literal(Literal::Str("server.js".to_string())),
            ])],
        }),
    ]);

    let result = emit_dockerfile(&ast).unwrap();
    assert!(result.contains("CMD"), "CMD in: {result}");
}

#[test]
fn test_DOCKER_BUILD_012_label_directive() {
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

    let result = emit_dockerfile(&ast).unwrap();
    assert!(result.contains("LABEL"), "LABEL in: {result}");
}

#[test]
fn test_DOCKER_BUILD_013_healthcheck() {
    let ast = make_simple_ast(vec![
        Stmt::Expr(Expr::FunctionCall {
            name: "from_image".to_string(),
            args: vec![
                Expr::Literal(Literal::Str("nginx".to_string())),
                Expr::Literal(Literal::Str("alpine".to_string())),
            ],
        }),
        Stmt::Expr(Expr::FunctionCall {
            name: "healthcheck".to_string(),
            args: vec![Expr::Literal(Literal::Str(
                "curl -f http://localhost/".to_string(),
            ))],
        }),
    ]);

    let result = emit_dockerfile(&ast).unwrap();
    assert!(result.contains("HEALTHCHECK"), "HEALTHCHECK in: {result}");
}

#[test]
fn test_DOCKER_BUILD_014_comment() {
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

    let result = emit_dockerfile(&ast).unwrap();
    assert!(
        result.contains("Install dependencies"),
        "Comment in: {result}"
    );
}

#[test]

include!("dockerfile_tests_tests_DOCKER_3.rs");
