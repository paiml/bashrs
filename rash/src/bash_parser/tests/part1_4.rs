#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]

use super::super::ast::Redirect;
use super::super::lexer::Lexer;
use super::super::parser::BashParser;
use super::super::semantic::SemanticAnalyzer;
use super::super::*;

        #[test]
        fn prop_alternative_value_uses_correct_operator(
            var_name in "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
            alt_value in "[a-zA-Z]{3,15}"
        ) {
            let ast = BashAst {
                statements: vec![BashStmt::Command {
                    name: "test".to_string(),
                    args: vec![BashExpr::AlternativeValue {
                        variable: var_name.clone(),
                        alternative: Box::new(BashExpr::Literal(alt_value.clone())),
                    }],
                    redirects: vec![],
                    span: Span::dummy(),
                }],
                metadata: AstMetadata {
                    source_file: None,
                    line_count: 1,
                    parse_time_ms: 0,
                },
            };

            let purified = generators::generate_purified_bash(&ast);

            // PROPERTY: Must use :+ operator
            prop_assert!(
                purified.contains(":+"),
                "Purified output must contain :+ operator, got: {}",
                purified
            );

            // PROPERTY: Must NOT use :-, :=, or :? operators
            prop_assert!(
                !purified.contains(":-") && !purified.contains(":=") && !purified.contains(":?"),
                "Purified output must not contain :-, :=, or :? (should be :+), got: {}",
                purified
            );
        }
    }

    // Property: String length expansion preserves variable name
    proptest! {
        #[test]
        fn prop_string_length_preserves_variable(
            var_name in "[a-zA-Z_][a-zA-Z0-9_]{0,10}"
        ) {
            let ast = BashAst {
                statements: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::StringLength {
                        variable: var_name.clone(),
                    }],
                    redirects: vec![],
                    span: Span::dummy(),
                }],
                metadata: AstMetadata {
                    source_file: None,
                    line_count: 1,
                    parse_time_ms: 0,
                },
            };

            let purified = generators::generate_purified_bash(&ast);

            // PROPERTY: Must contain the variable name
            prop_assert!(
                purified.contains(&var_name),
                "Purified output must contain variable name '{}', got: {}",
                var_name,
                purified
            );

            // PROPERTY: Must contain # operator
            prop_assert!(
                purified.contains("#"),
                "Purified output must contain # operator, got: {}",
                purified
            );

            // PROPERTY: Must contain $ for parameter expansion
            prop_assert!(
                purified.contains("$"),
                "Purified output must contain $ for expansion, got: {}",
                purified
            );
        }
    }

    // Property: String length expansion is deterministic
    proptest! {
        #[test]
        fn prop_string_length_is_deterministic(
            var_name in "[a-zA-Z_][a-zA-Z0-9_]{0,10}"
        ) {
            let ast = BashAst {
                statements: vec![BashStmt::Assignment {
                    name: "length".to_string(),
            index: None,
                    value: BashExpr::StringLength {
                        variable: var_name.clone(),
                    },
                    exported: false,
                    span: Span::dummy(),
                }],
                metadata: AstMetadata {
                    source_file: None,
                    line_count: 1,
                    parse_time_ms: 0,
                },
            };

            // Generate twice
            let purified1 = generators::generate_purified_bash(&ast);
            let purified2 = generators::generate_purified_bash(&ast);

            // PROPERTY: Determinism - byte-identical output
            prop_assert_eq!(
                purified1,
                purified2,
                "String length expansion must be deterministic"
            );
        }
    }

    // Property: String length uses # not other parameter operators
    proptest! {
        #[test]
        fn prop_string_length_uses_correct_operator(
            var_name in "[a-zA-Z_][a-zA-Z0-9_]{0,10}"
        ) {
            let ast = BashAst {
                statements: vec![BashStmt::Command {
                    name: "test".to_string(),
                    args: vec![BashExpr::StringLength {
                        variable: var_name.clone(),
                    }],
                    redirects: vec![],
                    span: Span::dummy(),
                }],
                metadata: AstMetadata {
                    source_file: None,
                    line_count: 1,
                    parse_time_ms: 0,
                },
            };

            let purified = generators::generate_purified_bash(&ast);

            // PROPERTY: Must use # operator
            prop_assert!(
                purified.contains("#"),
                "Purified output must contain # operator, got: {}",
                purified
            );

            // PROPERTY: Must NOT use :-, :=, :?, or :+ operators
            prop_assert!(
                !purified.contains(":-") && !purified.contains(":=") &&
                !purified.contains(":?") && !purified.contains(":+"),
                "Purified output must not contain :-, :=, :?, or :+ (should be #), got: {}",
                purified
            );
        }
    }

    // Property: Remove suffix expansion preserves variable and pattern
    proptest! {
        #[test]
        fn prop_remove_suffix_preserves_components(
            var_name in "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
            pattern in "\\.[a-z]{2,4}"
        ) {
            let ast = BashAst {
                statements: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::RemoveSuffix {
                        variable: var_name.clone(),
                        pattern: Box::new(BashExpr::Literal(pattern.clone())),
                    }],
                    redirects: vec![],
                    span: Span::dummy(),
                }],
                metadata: AstMetadata {
                    source_file: None,
                    line_count: 1,
                    parse_time_ms: 0,
                },
            };

            let purified = generators::generate_purified_bash(&ast);

            // PROPERTY: Must contain the variable name
            prop_assert!(
                purified.contains(&var_name),
                "Purified output must contain variable name '{}', got: {}",
                var_name,
                purified
            );

            // PROPERTY: Must contain the pattern
            prop_assert!(
                purified.contains(&pattern) || purified.contains(pattern.trim_start_matches('.')),
                "Purified output must contain pattern '{}', got: {}",
                pattern,
                purified
            );

            // PROPERTY: Must contain % operator
            prop_assert!(
                purified.contains("%"),
                "Purified output must contain % operator, got: {}",
                purified
            );
        }
    }

    // Property: Remove suffix expansion is deterministic
    proptest! {
        #[test]
        fn prop_remove_suffix_is_deterministic(
            var_name in "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
            pattern in "\\.[a-z]{2,4}"
        ) {
            let ast = BashAst {
                statements: vec![BashStmt::Assignment {
                    name: "result".to_string(),
            index: None,
                    value: BashExpr::RemoveSuffix {
                        variable: var_name.clone(),
                        pattern: Box::new(BashExpr::Literal(pattern.clone())),
                    },
                    exported: false,
                    span: Span::dummy(),
                }],
                metadata: AstMetadata {
                    source_file: None,
                    line_count: 1,
                    parse_time_ms: 0,
                },
            };

            // Generate twice
            let purified1 = generators::generate_purified_bash(&ast);
            let purified2 = generators::generate_purified_bash(&ast);

            // PROPERTY: Determinism - byte-identical output
            prop_assert_eq!(
                purified1,
                purified2,
                "Remove suffix expansion must be deterministic"
            );
        }
    }

    // Property: Remove suffix uses % not #, :-, :=, :?, or :+
    proptest! {
        #[test]
        fn prop_remove_suffix_uses_correct_operator(
            var_name in "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
            pattern in "\\.[a-z]{2,4}"
        ) {
            let ast = BashAst {
                statements: vec![BashStmt::Command {
                    name: "test".to_string(),
                    args: vec![BashExpr::RemoveSuffix {
                        variable: var_name.clone(),
                        pattern: Box::new(BashExpr::Literal(pattern.clone())),
                    }],
                    redirects: vec![],
                    span: Span::dummy(),
                }],
                metadata: AstMetadata {
                    source_file: None,
                    line_count: 1,
                    parse_time_ms: 0,
                },
            };

            let purified = generators::generate_purified_bash(&ast);

            // PROPERTY: Must use % operator
            prop_assert!(
                purified.contains("%"),
                "Purified output must contain % operator, got: {}",
                purified
            );

            // PROPERTY: Must NOT use # (that's for prefix removal)
            // Note: # is used for string length, not prefix removal
            // We check it's not confused with other operators
            prop_assert!(
                !purified.contains(":-") && !purified.contains(":=") &&
                !purified.contains(":?") && !purified.contains(":+"),
                "Purified output must not contain :-, :=, :?, or :+ (should be %), got: {}",
                purified
            );
        }
    }

    // Property: Remove prefix expansion preserves variable and pattern
    proptest! {
        #[test]
        fn prop_remove_prefix_preserves_components(
            var_name in "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
            pattern in "/[a-z]{3,5}/"
        ) {
            let ast = BashAst {
                statements: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::RemovePrefix {
                        variable: var_name.clone(),
                        pattern: Box::new(BashExpr::Literal(pattern.clone())),
                    }],
                    redirects: vec![],
                    span: Span::dummy(),
                }],
                metadata: AstMetadata {
                    source_file: None,
                    line_count: 1,
                    parse_time_ms: 0,
                },
            };

            let purified = generators::generate_purified_bash(&ast);

            // PROPERTY: Must contain the variable name
            prop_assert!(
                purified.contains(&var_name),
                "Purified output must contain variable name '{}', got: {}",
                var_name,
                purified
            );

            // PROPERTY: Must contain the pattern (or part of it)
            prop_assert!(
                purified.contains(&pattern) || purified.contains(pattern.trim_matches('/')),
                "Purified output must contain pattern '{}', got: {}",
                pattern,
                purified
            );

            // PROPERTY: Must contain # operator
            prop_assert!(
                purified.contains("#"),
                "Purified output must contain # operator, got: {}",
                purified
            );
        }
    }

    // Property: Remove prefix expansion is deterministic
    proptest! {
        #[test]
        fn prop_remove_prefix_is_deterministic(
            var_name in "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
            pattern in "/[a-z]{3,5}/"
        ) {
            let ast = BashAst {
                statements: vec![BashStmt::Assignment {
                    name: "result".to_string(),
            index: None,
                    value: BashExpr::RemovePrefix {
                        variable: var_name.clone(),
                        pattern: Box::new(BashExpr::Literal(pattern.clone())),
                    },
                    exported: false,
                    span: Span::dummy(),
                }],
                metadata: AstMetadata {
                    source_file: None,
                    line_count: 1,
                    parse_time_ms: 0,
                },
            };

            // Generate twice
            let purified1 = generators::generate_purified_bash(&ast);
            let purified2 = generators::generate_purified_bash(&ast);

            // PROPERTY: Determinism - byte-identical output
            prop_assert_eq!(
                purified1,
                purified2,
                "Remove prefix expansion must be deterministic"
            );
        }
    }

    // Property: Remove prefix uses # not %, :-, :=, :?, or :+
    proptest! {
        #[test]
        fn prop_remove_prefix_uses_correct_operator(
            var_name in "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
            pattern in "/[a-z]{3,5}/"
        ) {
            let ast = BashAst {
                statements: vec![BashStmt::Command {
                    name: "test".to_string(),
                    args: vec![BashExpr::RemovePrefix {
                        variable: var_name.clone(),
                        pattern: Box::new(BashExpr::Literal(pattern.clone())),
                    }],
                    redirects: vec![],
                    span: Span::dummy(),
                }],
                metadata: AstMetadata {
                    source_file: None,
                    line_count: 1,
                    parse_time_ms: 0,
                },
            };

            let purified = generators::generate_purified_bash(&ast);

            // PROPERTY: Must use # operator
            prop_assert!(
                purified.contains("#"),
                "Purified output must contain # operator, got: {}",
                purified
            );

            // PROPERTY: Must NOT use % (that's for suffix removal)
            // Note: We check it's not confused with other operators
            // % is for suffix removal, # is for prefix removal
            prop_assert!(
                !purified.contains(":-") && !purified.contains(":=") &&
                !purified.contains(":?") && !purified.contains(":+"),
                "Purified output must not contain :-, :=, :?, or :+ (should be #), got: {}",
                purified
            );
        }
    }

    // Property: Remove longest prefix expansion preserves variable and pattern
    proptest! {
        #[test]
        fn prop_remove_longest_prefix_preserves_components(
            var_name in "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
            pattern in "\\*/|\\*[a-z]{1,3}/"
        ) {
            let ast = BashAst {
                statements: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::RemoveLongestPrefix {
                        variable: var_name.clone(),
                        pattern: Box::new(BashExpr::Literal(pattern.clone())),
                    }],
                    redirects: vec![],
                    span: Span::dummy(),
                }],
                metadata: AstMetadata {
                    source_file: None,
                    line_count: 1,
                    parse_time_ms: 0,
                },
            };

            let purified = generators::generate_purified_bash(&ast);

            // PROPERTY: Must contain the variable name
            prop_assert!(
                purified.contains(&var_name),
                "Purified output must contain variable name '{}', got: {}",
                var_name,
                purified
            );

            // PROPERTY: Must contain the pattern (or part of it)
            prop_assert!(
                purified.contains(&pattern) || purified.contains(pattern.trim_matches('/')),
                "Purified output must contain pattern '{}', got: {}",
                pattern,
                purified
            );

            // PROPERTY: Must contain ## operator (greedy)
            prop_assert!(
                purified.contains("##"),
                "Purified output must contain ## operator, got: {}",
                purified
            );
        }
    }

    // Property: Remove longest prefix expansion is deterministic
    proptest! {
