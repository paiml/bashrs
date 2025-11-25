//! Property-Based Tests for Bash Parser and Transpiler
//!
//! These tests use proptest to verify key properties with generated inputs.

use super::generators::*;
use super::SemanticAnalyzer;
use crate::bash_transpiler::codegen::{BashToRashTranspiler, TranspileOptions};
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 100, // Start with 100 for faster test runs
        max_shrink_iters: 1000,
        .. ProptestConfig::default()
    })]

    /// Property: Valid bash scripts can be analyzed
    /// FIXED: TICKET-6002 - bash_script() now generates unique function names
    #[test]
    fn prop_valid_scripts_analyze_successfully(script in bash_script()) {
        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&script);
        prop_assert!(result.is_ok());
    }

    /// Property: Transpilation is deterministic
    #[test]
    fn prop_transpilation_is_deterministic(script in bash_script()) {
        let mut transpiler1 = BashToRashTranspiler::new(TranspileOptions::default());
        let mut transpiler2 = BashToRashTranspiler::new(TranspileOptions::default());

        let result1 = transpiler1.transpile(&script)?;
        let result2 = transpiler2.transpile(&script)?;

        prop_assert_eq!(result1, result2);
    }

    /// Property: Variable names are preserved
    #[test]
    fn prop_variable_names_preserved(name in bash_variable_name()) {
        use crate::bash_parser::ast::*;

        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: name.clone(),
                value: BashExpr::Literal("test".to_string()),
                exported: false,
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let mut transpiler = BashToRashTranspiler::new(TranspileOptions::default());
        let rash_code = transpiler.transpile(&ast)?;

        let expected = format!("let {}", name);
        prop_assert!(rash_code.contains(&expected));
    }

    /// Property: Exported variables tracked
    #[test]
    fn prop_exported_vars_tracked(name in bash_variable_name()) {
        use crate::bash_parser::ast::*;

        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: name.clone(),
                value: BashExpr::Literal("value".to_string()),
                exported: true,
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let mut analyzer = SemanticAnalyzer::new();
        let report = analyzer.analyze(&ast)?;

        prop_assert!(report.effects.env_modifications.contains(&name));
    }

    /// Property: Purified bash always uses POSIX sh shebang
    /// Task 1.1: Shebang transformation property test
    #[test]
    fn prop_purified_bash_uses_posix_shebang(script in bash_script()) {
        let purified = generate_purified_bash(&script);

        // Property 1: Must start with #!/bin/sh
        prop_assert!(
            purified.starts_with("#!/bin/sh"),
            "Purified bash must use POSIX sh shebang, got: {}",
            purified.lines().next().unwrap_or("")
        );

        // Property 3: Must not contain bash-specific shebangs
        prop_assert!(
            !purified.contains("#!/bin/bash") && !purified.contains("#!/usr/bin/bash"),
            "Purified bash must not contain bash-specific shebangs"
        );

        // Property 2: Must be deterministic (compare after other checks)
        let purified2 = generate_purified_bash(&script);
        prop_assert_eq!(purified, purified2, "Purification must be deterministic");
    }

    /// Property: Purified bash preserves command structure
    #[test]
    fn prop_purified_bash_preserves_commands(name in bash_identifier(), arg in bash_string()) {
        use crate::bash_parser::ast::*;

        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: name.clone(),
                args: vec![BashExpr::Literal(arg.clone())],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let purified = generate_purified_bash(&ast);

        // Command name should be preserved
        prop_assert!(purified.contains(&name), "Command name '{}' not preserved in: {}", name, purified);
    }

    // ===== Issue #59 Property Tests =====
    // Tests for nested quotes in command substitution and || && operators

    /// Property: Strings with command substitutions parse without panic
    /// Issue #59: Nested quotes inside command substitutions must not crash parser
    #[test]
    fn prop_ISSUE_059_001_command_subst_strings_parse_safely(
        cmd in "[a-z]{1,10}",
        arg in "[a-zA-Z0-9_]{1,10}"
    ) {
        use crate::bash_parser::BashParser;

        // Build a string with command substitution containing nested quotes
        let script = format!(r#"OUTPUT="$({} "{}")" "#, cmd, arg);

        // Must not panic - parsing should succeed or fail gracefully
        let result = BashParser::new(&script);
        if let Ok(mut parser) = result {
            // Parse should complete without panic
            let _ = parser.parse();
        }
    }

    /// Property: AndList AST nodes round-trip through codegen
    /// Issue #59: && operator must be preserved in code generation
    #[test]
    fn prop_ISSUE_059_002_andlist_roundtrip(
        left_cmd in "[a-z]{1,10}",
        right_cmd in "[a-z]{1,10}"
    ) {
        use crate::bash_parser::ast::*;

        let ast = BashAst {
            statements: vec![BashStmt::AndList {
                left: Box::new(BashStmt::Command {
                    name: left_cmd.clone(),
                    args: vec![],
                    redirects: vec![],
                    span: Span::dummy(),
                }),
                right: Box::new(BashStmt::Command {
                    name: right_cmd.clone(),
                    args: vec![],
                    redirects: vec![],
                    span: Span::dummy(),
                }),
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let purified = generate_purified_bash(&ast);

        // Must contain && operator
        prop_assert!(
            purified.contains("&&"),
            "AndList must generate && in output: {}",
            purified
        );

        // Must contain both command names
        prop_assert!(
            purified.contains(&left_cmd) && purified.contains(&right_cmd),
            "Both commands must be preserved: {}",
            purified
        );
    }

    /// Property: OrList AST nodes round-trip through codegen
    /// Issue #59: || operator must be preserved in code generation
    #[test]
    fn prop_ISSUE_059_003_orlist_roundtrip(
        left_cmd in "[a-z]{1,10}",
        right_cmd in "[a-z]{1,10}"
    ) {
        use crate::bash_parser::ast::*;

        let ast = BashAst {
            statements: vec![BashStmt::OrList {
                left: Box::new(BashStmt::Command {
                    name: left_cmd.clone(),
                    args: vec![],
                    redirects: vec![],
                    span: Span::dummy(),
                }),
                right: Box::new(BashStmt::Command {
                    name: right_cmd.clone(),
                    args: vec![],
                    redirects: vec![],
                    span: Span::dummy(),
                }),
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };

        let purified = generate_purified_bash(&ast);

        // Must contain || operator
        prop_assert!(
            purified.contains("||"),
            "OrList must generate || in output: {}",
            purified
        );

        // Must contain both command names
        prop_assert!(
            purified.contains(&left_cmd) && purified.contains(&right_cmd),
            "Both commands must be preserved: {}",
            purified
        );
    }

    /// Property: Parsing logical operators never panics
    /// Issue #59: || and && after commands must parse safely
    #[test]
    fn prop_ISSUE_059_004_logical_operators_parse_safely(
        // Use bash_identifier() which excludes keywords like fi, if, do, done, etc.
        cmd in bash_identifier(),
        op in prop::sample::select(vec!["&&", "||"])
    ) {
        use crate::bash_parser::BashParser;

        let script = format!("{} {} true", cmd, op);

        // Must not panic
        let result = BashParser::new(&script);
        prop_assert!(result.is_ok(), "Lexer should succeed");

        let mut parser = result.unwrap();
        let parse_result = parser.parse();

        // Must parse successfully
        prop_assert!(
            parse_result.is_ok(),
            "Parser should accept '{}': {:?}",
            script,
            parse_result.err()
        );
    }
}
