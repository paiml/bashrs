//! Property-Based Tests for Bash Parser and Transpiler
//!
//! These tests use proptest to verify key properties with generated inputs.

use super::generators::*;
use super::{BashParser, SemanticAnalyzer};
use crate::bash_transpiler::codegen::{BashToRashTranspiler, TranspileOptions};
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 100, // Start with 100 for faster test runs
        max_shrink_iters: 1000,
        .. ProptestConfig::default()
    })]

    /// Property: Valid bash scripts can be analyzed
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
}
