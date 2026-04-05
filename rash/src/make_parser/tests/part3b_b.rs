#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(unused_imports)]

use super::super::*;
use crate::make_parser::ast::{MakeCondition, Span, VarFlavor};

        #[test]
        fn prop_GENERATE_008_roundtrip_targets(
            target_name in "[a-z][a-z0-9_-]{0,15}",
            prereq in "[a-z][a-z0-9_.]{0,15}",
        ) {
            // ARRANGE: Create target AST
            let ast = MakeAst {
                items: vec![MakeItem::Target {
                    name: target_name.clone(),
                    prerequisites: vec![prereq.clone()],
                    recipe: vec!["echo test".to_string()],
                    phony: false,
            recipe_metadata: None,
                    span: Span::dummy(),
                }],
                metadata: MakeMetadata::new(),
            };

            // ACT: Generate and re-parse
            let generated = generate_purified_makefile(&ast);
            let reparsed = parse_makefile(&generated);

            // ASSERT: Should parse successfully
            prop_assert!(reparsed.is_ok(), "Failed to parse generated Makefile: {}", generated);

            let reparsed_ast = reparsed.unwrap();

            // ASSERT: Should have same number of items
            prop_assert_eq!(reparsed_ast.items.len(), 1);

            // ASSERT: Should preserve target structure
            if let MakeItem::Target { name, prerequisites, recipe, .. } = &reparsed_ast.items[0] {
                prop_assert_eq!(name, &target_name);
                prop_assert_eq!(prerequisites.len(), 1);
                prop_assert_eq!(&prerequisites[0], &prereq);
                prop_assert_eq!(recipe.len(), 1);
                prop_assert_eq!(&recipe[0], "echo test");
            } else {
                prop_assert!(false, "Expected Target item, got {:?}", reparsed_ast.items[0]);
            }
        }

        /// PROPERTY TEST: Generation is deterministic
        ///
        /// Property: generate(ast) always produces same output for same input
