#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(unused_imports)]

use super::super::*;
use crate::make_parser::ast::{MakeCondition, Span, VarFlavor};

        #[test]
        fn prop_VAR_SUBST_001_multiple_substitutions_preserved(
            var1 in "[A-Z]{2,8}",
            var2 in "[A-Z]{2,8}",
            ref1 in "[A-Z]{2,8}",
            ref2 in "[A-Z]{2,8}"
        ) {
            // ARRANGE: Multiple variables with different substitutions
            let makefile = format!(
                "{} = $({}:.c=.o)\n{} = $({}:.a=.so)",
                var1, ref1, var2, ref2
            );

            // ACT: Parse makefile
            let result = parse_makefile(&makefile);

            // ASSERT: Both substitutions preserved
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            prop_assert_eq!(ast.items.len(), 2);

            // Check first substitution
            match &ast.items[0] {
                MakeItem::Variable { name, value, .. } => {
                    prop_assert_eq!(name, &var1);
                    let expected = format!("$({}:.c=.o)", ref1);
                    prop_assert_eq!(value, &expected);
                }
                other => return Err(TestCaseError::fail(format!("Expected Variable, got {:?}", other))),
            }

            // Check second substitution
            match &ast.items[1] {
                MakeItem::Variable { name, value, .. } => {
                    prop_assert_eq!(name, &var2);
                    let expected = format!("$({}:.a=.so)", ref2);
                    prop_assert_eq!(value, &expected);
                }
                other => return Err(TestCaseError::fail(format!("Expected Variable, got {:?}", other))),
            }
        }
    }
}
