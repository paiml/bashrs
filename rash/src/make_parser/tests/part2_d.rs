#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(unused_imports)]

use super::super::*;
use crate::make_parser::ast::{MakeCondition, Span, VarFlavor};

// Phase 5 - MUTATION TESTING: Mutation-killing tests for = recursive assignment

// Target: parser.rs:116 - is_variable_assignment() contains('=') check
        #[test]
        fn prop_INCLUDE_001_includes_always_parse(
            filename in "[a-zA-Z0-9_.-]{1,30}\\.mk"
        ) {
            let makefile = format!("include {}", filename);
            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok(), "Include should always parse valid filenames");

            let ast = result.unwrap();
            prop_assert_eq!(ast.items.len(), 1);

            match &ast.items[0] {
                MakeItem::Include { path, optional, .. } => {
                    prop_assert_eq!(path, &filename);
                    prop_assert!(!optional);
                }
                _ => prop_assert!(false, "Expected MakeItem::Include"),
            }
        }

        #[test]
        fn prop_INCLUDE_001_parsing_is_deterministic(
            filename in "[a-zA-Z0-9/_.-]{1,50}\\.mk"
        ) {
            let makefile = format!("include {}", filename);
            let ast1 = parse_makefile(&makefile);
            let ast2 = parse_makefile(&makefile);

            match (ast1, ast2) {
                (Ok(a1), Ok(a2)) => {
                    prop_assert_eq!(a1.items.len(), a2.items.len());
                    match (&a1.items[0], &a2.items[0]) {
                        (MakeItem::Include { path: p1, .. }, MakeItem::Include { path: p2, .. }) => {
                            prop_assert_eq!(p1, p2);
                        }
                        _ => prop_assert!(false, "Expected matching Include items"),
                    }
                }
                _ => prop_assert!(false, "Parsing should be deterministic"),
            }
        }

