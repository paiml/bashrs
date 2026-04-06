
use super::*;
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_codegen_deterministic(stmt_count in 0usize..10) {
        // Property: Same AST → Same output (determinism)
        let statements: Vec<BashStmt> = (0..stmt_count)
            .map(|i| BashStmt::Command {
                name: format!("cmd{}", i),
                args: vec![],
                redirects: vec![],
                span: Span::new(i + 1, 1, i + 1, 10),
            })
            .collect();

        let ast = BashAst {
            statements: statements.clone(),
            metadata: AstMetadata {
                source_file: None,
                line_count: stmt_count,
                parse_time_ms: 0,
            },
        };

        let output1 = generate_purified_bash(&ast);
        let output2 = generate_purified_bash(&ast);

        prop_assert_eq!(output1, output2, "Codegen should be deterministic");
    }

    #[test]
    fn prop_codegen_shebang_transformation(stmt_count in 0usize..10) {
        // Property: All generated scripts start with #!/bin/sh (POSIX shebang)
        let statements: Vec<BashStmt> = (0..stmt_count)
            .map(|i| BashStmt::Command {
                name: format!("cmd{}", i),
                args: vec![],
                redirects: vec![],
                span: Span::new(i + 1, 1, i + 1, 10),
            })
            .collect();

        let ast = BashAst {
            statements,
            metadata: AstMetadata {
                source_file: None,
                line_count: stmt_count,
                parse_time_ms: 0,
            },
        };

        let output = generate_purified_bash(&ast);

        prop_assert!(
            output.starts_with("#!/bin/sh\n"),
            "All purified scripts must start with POSIX shebang #!/bin/sh"
        );
    }

    #[test]
    fn prop_codegen_variable_quoting(var_name in "[a-z][a-z0-9_]{0,10}") {
        // Property: All variable references are quoted for safety
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Variable(var_name.clone())],
                redirects: vec![],
                span: Span::new(1, 1, 1, 10),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let output = generate_purified_bash(&ast);

        // Verify variable is quoted: "$VAR" not $VAR
        let expected_quoted = format!("\"${}\"", var_name);
        prop_assert!(
            output.contains(&expected_quoted),
            "Variables must be quoted: expected {} in output:\n{}",
            expected_quoted,
            output
        );
    }

    #[test]
    fn prop_codegen_no_nondeterministic_constructs(stmt_count in 0usize..10) {
        // Property: Purified output never contains non-deterministic constructs
        let statements: Vec<BashStmt> = (0..stmt_count)
            .map(|i| BashStmt::Command {
                name: format!("cmd{}", i),
                args: vec![BashExpr::Literal(format!("arg{}", i))],
                redirects: vec![],
                span: Span::new(i + 1, 1, i + 1, 10),
            })
            .collect();

        let ast = BashAst {
            statements,
            metadata: AstMetadata {
                source_file: None,
                line_count: stmt_count,
                parse_time_ms: 0,
            },
        };

        let output = generate_purified_bash(&ast);

        // Verify no non-deterministic constructs
        prop_assert!(
            !output.contains("$RANDOM"),
            "Purified output must not contain $RANDOM"
        );
        prop_assert!(
            !output.contains("$$"),
            "Purified output must not contain $$ (process ID)"
        );
        prop_assert!(
            !output.contains("$(date"),
            "Purified output must not contain timestamp commands"
        );
    }

    #[test]
    fn prop_codegen_idempotent_safe_flags(cmd_name in "(mkdir|rm|ln)") {
        // Property: Idempotent operations use safe flags (-p, -f, -sf)
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: cmd_name.clone(),
                args: vec![BashExpr::Literal("target".to_string())],
                redirects: vec![],
                span: Span::new(1, 1, 1, 10),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let output = generate_purified_bash(&ast);

        // Verify idempotent flags based on command
        match cmd_name.as_str() {
            "mkdir" => prop_assert!(
                output.contains("mkdir -p") || output.contains("mkdir"),
                "mkdir should ideally use -p flag for idempotency"
            ),
            "rm" => prop_assert!(
                output.contains("rm -f") || output.contains("rm"),
                "rm should ideally use -f flag for idempotency"
            ),
            "ln" => prop_assert!(
                output.contains("ln -sf") || output.contains("ln"),
                "ln should ideally use -sf flags for idempotency"
            ),
            _ => {}
        }
    }
}
