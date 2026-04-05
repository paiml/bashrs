#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]

use super::super::ast::Redirect;
use super::super::lexer::Lexer;
use super::super::parser::BashParser;
use super::super::semantic::SemanticAnalyzer;
use super::super::*;

#[test]
fn test_remove_suffix_expansion() {
    use crate::bash_parser::ast::*;

    // INPUT: Parameter expansion with suffix removal
    // file="test.txt"; echo "${file%.txt}"
    // Remove shortest matching suffix pattern from variable

    // Manually construct AST with remove suffix expansion
    let remove_suffix_expr = BashExpr::RemoveSuffix {
        variable: "file".to_string(),
        pattern: Box::new(BashExpr::Literal(".txt".to_string())),
    };

    let ast = BashAst {
        statements: vec![BashStmt::Command {
            name: "echo".to_string(),
            args: vec![remove_suffix_expr],
            redirects: vec![],
            span: Span::dummy(),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };

    // Generate purified bash
    let purified = generators::generate_purified_bash(&ast);

    // EXPECTED: Purified bash should preserve ${file%.txt} syntax
    // echo "${file%.txt}"

    // ASSERT: Should contain parameter expansion syntax with %
    assert!(
        purified.contains("$") && purified.contains("file") && purified.contains("%"),
        "Purified output should preserve ${{file%.txt}} syntax, got: {}",
        purified
    );

    // ASSERT: Should contain pattern
    assert!(
        purified.contains(".txt") || purified.contains("txt"),
        "Purified output should contain pattern, got: {}",
        purified
    );

    // ASSERT: Should contain the command
    assert!(
        purified.contains("echo"),
        "Purified output should contain echo command, got: {}",
        purified
    );

    // PROPERTY: Deterministic output
    let purified2 = generators::generate_purified_bash(&ast);
    assert_eq!(purified, purified2, "Purification must be deterministic");

    // TODO: Test Rust transpilation
    // Expected: let name = file.strip_suffix(".txt").unwrap_or(&file);
}

// BASH MANUAL VALIDATION - Task EXP-PARAM-007: Remove Prefix Expansion
// EXTREME TDD RED Phase - This test MUST fail first

#[test]
fn test_remove_prefix_expansion() {
    use crate::bash_parser::ast::*;

    // INPUT: Parameter expansion with prefix removal
    // path="/usr/local/bin"; echo "${path#/usr/}"
    // Remove shortest matching prefix pattern from variable

    // Manually construct AST with remove prefix expansion
    let remove_prefix_expr = BashExpr::RemovePrefix {
        variable: "path".to_string(),
        pattern: Box::new(BashExpr::Literal("/usr/".to_string())),
    };

    let ast = BashAst {
        statements: vec![BashStmt::Command {
            name: "echo".to_string(),
            args: vec![remove_prefix_expr],
            redirects: vec![],
            span: Span::dummy(),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };

    // Generate purified bash
    let purified = generators::generate_purified_bash(&ast);

    // EXPECTED: Purified bash should preserve ${path#/usr/} syntax
    // echo "${path#/usr/}"

    // ASSERT: Should contain parameter expansion syntax with #
    assert!(
        purified.contains("$") && purified.contains("path") && purified.contains("#"),
        "Purified output should preserve ${{path#/usr/}} syntax, got: {}",
        purified
    );

    // ASSERT: Should contain pattern
    assert!(
        purified.contains("/usr/") || purified.contains("usr"),
        "Purified output should contain pattern, got: {}",
        purified
    );

    // ASSERT: Should contain the command
    assert!(
        purified.contains("echo"),
        "Purified output should contain echo command, got: {}",
        purified
    );

    // PROPERTY: Deterministic output
    let purified2 = generators::generate_purified_bash(&ast);
    assert_eq!(purified, purified2, "Purification must be deterministic");

    // TODO: Test Rust transpilation
    // Expected: let name = path.strip_prefix("/usr/").unwrap_or(&path);
}

// BASH MANUAL VALIDATION - Task EXP-PARAM-008: Remove Longest Prefix Expansion
// EXTREME TDD RED Phase - This test MUST fail first

#[test]
fn test_remove_longest_prefix_expansion() {
    use crate::bash_parser::ast::*;

    // INPUT: Parameter expansion with longest prefix removal (greedy)
    // path="/usr/local/bin"; echo "${path##*/}"
    // Remove longest matching prefix pattern from variable
    // ${path##*/} removes everything up to the last / - gets just "bin"

    // Manually construct AST with remove longest prefix expansion
    let remove_longest_prefix_expr = BashExpr::RemoveLongestPrefix {
        variable: "path".to_string(),
        pattern: Box::new(BashExpr::Literal("*/".to_string())),
    };

    let ast = BashAst {
        statements: vec![BashStmt::Command {
            name: "echo".to_string(),
            args: vec![remove_longest_prefix_expr],
            redirects: vec![],
            span: Span::dummy(),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };

    // Generate purified bash
    let purified = generators::generate_purified_bash(&ast);

    // EXPECTED: Purified bash should preserve ${path##*/} syntax
    // echo "${path##*/}"

    // ASSERT: Should contain parameter expansion syntax with ##
    assert!(
        purified.contains("$") && purified.contains("path") && purified.contains("##"),
        "Purified output should preserve ${{path##*/}} syntax, got: {}",
        purified
    );

    // ASSERT: Should contain pattern
    assert!(
        purified.contains("*/") || purified.contains("*"),
        "Purified output should contain pattern, got: {}",
        purified
    );

    // ASSERT: Should contain the command
    assert!(
        purified.contains("echo"),
        "Purified output should contain echo command, got: {}",
        purified
    );

    // PROPERTY: Deterministic output
    let purified2 = generators::generate_purified_bash(&ast);
    assert_eq!(purified, purified2, "Purification must be deterministic");

    // TODO: Test Rust transpilation
    // Expected: let name = path.rsplit_once('/').map_or(&path, |(_, name)| name);
}

// BASH MANUAL VALIDATION - Task EXP-PARAM-009: Remove Longest Suffix Expansion
// EXTREME TDD RED Phase - This test MUST fail first

#[test]
fn test_remove_longest_suffix_expansion() {
    use crate::bash_parser::ast::*;

    // INPUT: Parameter expansion with longest suffix removal (greedy)
    // file="archive.tar.gz"; echo "${file%%.*}"
    // Remove longest matching suffix pattern from variable
    // ${file%%.*} removes everything from the first . - gets just "archive"

    // Manually construct AST with remove longest suffix expansion
    let remove_longest_suffix_expr = BashExpr::RemoveLongestSuffix {
        variable: "file".to_string(),
        pattern: Box::new(BashExpr::Literal(".*".to_string())),
    };

    let ast = BashAst {
        statements: vec![BashStmt::Command {
            name: "echo".to_string(),
            args: vec![remove_longest_suffix_expr],
            redirects: vec![],
            span: Span::dummy(),
        }],
        metadata: AstMetadata {
            source_file: None,
            line_count: 1,
            parse_time_ms: 0,
        },
    };

    // Generate purified bash
    let purified = generators::generate_purified_bash(&ast);

    // EXPECTED: Purified bash should preserve ${file%%.*} syntax
    // echo "${file%%.*}"

    // ASSERT: Should contain parameter expansion syntax with %%
    assert!(
        purified.contains("$") && purified.contains("file") && purified.contains("%%"),
        "Purified output should preserve ${{file%%.*}} syntax, got: {}",
        purified
    );

    // ASSERT: Should contain pattern
    assert!(
        purified.contains(".*") || purified.contains("*"),
        "Purified output should contain pattern, got: {}",
        purified
    );

    // ASSERT: Should contain the command
    assert!(
        purified.contains("echo"),
        "Purified output should contain echo command, got: {}",
        purified
    );

    // PROPERTY: Deterministic output
    let purified2 = generators::generate_purified_bash(&ast);
    assert_eq!(purified, purified2, "Purification must be deterministic");

    // TODO: Test Rust transpilation
    // Expected: let name = file.split_once('.').map_or(&file, |(name, _)| name);
}

// PROPERTY TESTING: Until Loop Transformation
// Verify until→while transformation properties hold across all valid inputs

#[cfg(test)]
mod property_tests {
    use super::*;
    use crate::bash_parser::ast::*;
    use proptest::prelude::*;

    // Property: All Until loops must be transformed to While loops
    // This verifies the core transformation rule
    proptest! {
        #[test]
        fn prop_until_always_becomes_while(
            var_name in "[a-z][a-z0-9]{0,5}",
            threshold in 1i64..100i64
        ) {
            // Create an until loop: until [ $var -gt threshold ]; do ...; done
            let ast = BashAst {
                statements: vec![BashStmt::Until {
                    condition: BashExpr::Test(Box::new(TestExpr::IntGt(
                        BashExpr::Variable(var_name.clone()),
                        BashExpr::Literal(threshold.to_string()),
                    ))),
                    body: vec![BashStmt::Command {
                        name: "echo".to_string(),
                        args: vec![BashExpr::Variable(var_name)],
                        redirects: vec![],
                        span: Span::dummy(),
                    }],
                    span: Span::dummy(),
                }],
                metadata: AstMetadata {
                    source_file: None,
                    line_count: 1,
                    parse_time_ms: 0,
                },
            };

            let purified = generators::generate_purified_bash(&ast);

            // PROPERTY: Must contain "while"
            prop_assert!(
                purified.contains("while"),
                "Until loop must be transformed to while, got: {}",
                purified
            );

            // PROPERTY: Must NOT contain "until"
            prop_assert!(
                !purified.contains("until"),
                "Purified output must not contain 'until', got: {}",
                purified
            );

            // PROPERTY: Must contain negation "!"
            prop_assert!(
                purified.contains("!"),
                "Until condition must be negated in while loop, got: {}",
                purified
            );
        }
    }

    // Property: Until transformation must be deterministic
    // Same input must always produce same output
    proptest! {
        #[test]
        fn prop_until_transformation_is_deterministic(
            var_name in "[a-z][a-z0-9]{0,5}",
            threshold in 1i64..100i64
        ) {
            let ast = BashAst {
                statements: vec![BashStmt::Until {
                    condition: BashExpr::Test(Box::new(TestExpr::IntLt(
                        BashExpr::Variable(var_name.clone()),
                        BashExpr::Literal(threshold.to_string()),
                    ))),
                    body: vec![BashStmt::Assignment {
                        name: var_name.clone(),
            index: None,
                        value: BashExpr::Arithmetic(Box::new(ArithExpr::Add(
                            Box::new(ArithExpr::Variable(var_name)),
                            Box::new(ArithExpr::Number(1)),
                        ))),
                        exported: false,
                        span: Span::dummy(),
                    }],
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
                "Until transformation must be deterministic"
            );
        }
    }

    // Property: Until loops with different test expressions all transform correctly
    proptest! {
        #[test]
        fn prop_until_handles_all_test_types(
            var_name in "[a-z][a-z0-9]{0,5}",
            threshold in 1i64..10i64
        ) {
            // Test with different comparison operators
            for test_expr in [
                TestExpr::IntEq(
                    BashExpr::Variable(var_name.clone()),
                    BashExpr::Literal(threshold.to_string())
                ),
                TestExpr::IntNe(
                    BashExpr::Variable(var_name.clone()),
                    BashExpr::Literal(threshold.to_string())
                ),
                TestExpr::IntLt(
                    BashExpr::Variable(var_name.clone()),
                    BashExpr::Literal(threshold.to_string())
                ),
                TestExpr::IntGt(
                    BashExpr::Variable(var_name.clone()),
                    BashExpr::Literal(threshold.to_string())
                ),
            ] {
                let ast = BashAst {
                    statements: vec![BashStmt::Until {
                        condition: BashExpr::Test(Box::new(test_expr)),
                        body: vec![BashStmt::Comment {
                            text: "loop body".to_string(),
                            span: Span::dummy(),
                        }],
                        span: Span::dummy(),
                    }],
                    metadata: AstMetadata {
                        source_file: None,
                        line_count: 1,
                        parse_time_ms: 0,
                    },
                };

                let purified = generators::generate_purified_bash(&ast);

                // PROPERTY: All test types must be transformed
                prop_assert!(
                    purified.contains("while") && !purified.contains("until"),
                    "All until test types must transform to while, got: {}",
                    purified
                );
            }
        }
    }

    // Property: Default value expansion preserves variable name
    proptest! {
        #[test]
        fn prop_default_value_preserves_variable_name(
            var_name in "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
            default_val in "[a-z]{1,10}"
        ) {
            let ast = BashAst {
                statements: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::DefaultValue {
                        variable: var_name.clone(),
                        default: Box::new(BashExpr::Literal(default_val.clone())),
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

            // PROPERTY: Must contain the default value
            prop_assert!(
                purified.contains(&default_val),
                "Purified output must contain default value '{}', got: {}",
                default_val,
                purified
            );

            // PROPERTY: Must contain :- operator
            prop_assert!(
                purified.contains(":-"),
                "Purified output must contain :- operator, got: {}",
                purified
            );
        }
    }

    // Property: Default value expansion is deterministic
    proptest! {
}
}
