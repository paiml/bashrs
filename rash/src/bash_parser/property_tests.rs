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
}
