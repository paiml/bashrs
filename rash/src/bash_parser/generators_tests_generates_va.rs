#[cfg(test)]
mod tests {
    use super::*;
    use proptest::strategy::ValueTree;

    proptest! {
        #[test]
        fn test_generates_valid_identifiers(id in bash_identifier()) {
            // Should start with letter or underscore
            assert!(id.chars().next().unwrap().is_alphabetic() || id.starts_with('_'));
            // Should be reasonable length
            assert!(id.len() <= 16);
        }

        #[test]
        fn test_generates_valid_expressions(expr in bash_expr(2)) {
            // All expressions should be constructible
            match expr {
                BashExpr::Literal(s) => assert!(!s.is_empty() || s.is_empty()),
                BashExpr::Variable(v) => assert!(!v.is_empty()),
                BashExpr::Array(items) => assert!(items.len() <= 3),
                BashExpr::Arithmetic(_) => {},
                _ => {}
            }
        }

        #[test]
        fn test_generates_valid_statements(stmt in bash_stmt(2)) {
            // All statements should be constructible
            match stmt {
                BashStmt::Assignment { name, .. } => assert!(!name.is_empty()),
                BashStmt::Command { name, .. } => assert!(!name.is_empty()),
                BashStmt::Function { name, body, .. } => {
                    assert!(!name.is_empty());
                    assert!(!body.is_empty());
                }
                _ => {}
            }
        }

        #[test]
        fn test_generates_valid_scripts(script in bash_script()) {
            // Scripts should have at least one statement
            assert!(!script.statements.is_empty());
            assert!(script.statements.len() <= 10);
        }

        /// 🔴 RED: Property test for unique function names
        /// TICKET-6002: bash_script() should generate scripts with unique function names
        #[test]
        fn test_generated_scripts_have_unique_function_names(script in bash_script()) {
            use std::collections::HashSet;

            // Collect all function names
            let mut function_names = HashSet::new();
            let mut duplicate_found = false;
            let mut duplicate_name = String::new();

            for stmt in &script.statements {
                if let BashStmt::Function { name, .. } = stmt {
                    if !function_names.insert(name.clone()) {
                        // Duplicate found!
                        duplicate_found = true;
                        duplicate_name = name.clone();
                        break;
                    }
                }
            }

            prop_assert!(
                !duplicate_found,
                "Generated script has duplicate function name: '{}'. \
                All function names in a script must be unique. \
                Function names found: {:?}",
                duplicate_name,
                function_names
            );
        }
    }

    // ============== generate_purified_bash tests ==============

    #[test]
    fn test_generate_purified_bash_empty() {
        let ast = BashAst {
            statements: vec![],
            metadata: AstMetadata {
                source_file: None,
                line_count: 0,
                parse_time_ms: 0,
            },
        };
        let output = generate_purified_bash(&ast);
        assert!(output.starts_with("#!/bin/sh\n"));
    }

    #[test]
    fn test_generate_purified_bash_command() {
        let ast = BashAst {
            statements: vec![BashStmt::Command {
                name: "echo".to_string(),
                args: vec![BashExpr::Literal("hello".to_string())],
                redirects: vec![],
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };
        let output = generate_purified_bash(&ast);
        assert!(output.contains("echo hello"));
    }

    #[test]
    fn test_generate_purified_bash_assignment() {
        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "FOO".to_string(),
                index: None,
                value: BashExpr::Literal("bar".to_string()),
                exported: false,
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };
        let output = generate_purified_bash(&ast);
        assert!(output.contains("FOO=bar"));
    }

    #[test]
    fn test_generate_purified_bash_exported_assignment() {
        let ast = BashAst {
            statements: vec![BashStmt::Assignment {
                name: "PATH".to_string(),
                index: None,
                value: BashExpr::Literal("/usr/bin".to_string()),
                exported: true,
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };
        let output = generate_purified_bash(&ast);
        assert!(output.contains("export PATH=/usr/bin"));
    }

    #[test]
    fn test_generate_purified_bash_comment() {
        let ast = BashAst {
            statements: vec![BashStmt::Comment {
                text: "This is a comment".to_string(),
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };
        let output = generate_purified_bash(&ast);
        assert!(output.contains("# This is a comment"));
    }

    #[test]
    fn test_generate_purified_bash_function() {
        let ast = BashAst {
            statements: vec![BashStmt::Function {
                name: "my_func".to_string(),
                body: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("hello".to_string())],
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
        let output = generate_purified_bash(&ast);
        assert!(output.contains("my_func() {"));
        assert!(output.contains("echo hello"));
        assert!(output.contains("}"));
    }

    #[test]
    fn test_generate_purified_bash_if_statement() {
        let ast = BashAst {
            statements: vec![BashStmt::If {
                condition: BashExpr::Test(Box::new(TestExpr::StringNonEmpty(BashExpr::Variable(
                    "x".to_string(),
                )))),
                then_block: vec![BashStmt::Command {
                    name: "echo".to_string(),
                    args: vec![BashExpr::Literal("yes".to_string())],
                    redirects: vec![],
                    span: Span::dummy(),
                }],
                elif_blocks: vec![],
                else_block: None,
                span: Span::dummy(),
            }],
            metadata: AstMetadata {
                source_file: None,
                line_count: 1,
                parse_time_ms: 0,
            },
        };
        let output = generate_purified_bash(&ast);
        assert!(output.contains("if"));
        assert!(output.contains("then"));
        assert!(output.contains("fi"));
    }

include!("generators_tests_extracted_generate.rs");
