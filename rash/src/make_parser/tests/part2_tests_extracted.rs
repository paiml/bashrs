#[cfg(test)]
mod property_tests_syntax_002 {
    use super::*;
    use proptest::prelude::*;

    // Property 1: Line continuation always produces valid parse result
    proptest! {
        #[test]
        fn test_SYNTAX_002_prop_continuation_always_parses(
            var_name in "[A-Z][A-Z0-9_]{0,10}",
            value1 in "[a-z0-9_]{1,10}",
            value2 in "[a-z0-9_]{1,10}"
        ) {
            // ARRANGE: Variable with line continuation
            let makefile = format!("{} = {} \\\n    {}", var_name, value1, value2);

            // ACT: Parse
            let result = parse_makefile(&makefile);

            // ASSERT: Must parse successfully
            prop_assert!(result.is_ok(), "Line continuation must always parse: {:?}", result);
        }
    }

    // Property 2: Continuation is equivalent to same-line definition
    proptest! {
        #[test]
        fn test_SYNTAX_002_prop_continuation_equivalent_to_sameline(
            var_name in "[A-Z][A-Z0-9_]{0,10}",
            value1 in "[a-z0-9_]{1,10}",
            value2 in "[a-z0-9_]{1,10}"
        ) {
            // ARRANGE: Two versions - with and without continuation
            let with_continuation = format!("{} = {} \\\n    {}", var_name, value1, value2);
            let without_continuation = format!("{} = {} {}", var_name, value1, value2);

            // ACT: Parse both
            let result1 = parse_makefile(&with_continuation);
            let result2 = parse_makefile(&without_continuation);

            // ASSERT: Both must parse successfully
            prop_assert!(result1.is_ok());
            prop_assert!(result2.is_ok());

            // ASSERT: Must produce same value
            let ast1 = result1.unwrap();
            let ast2 = result2.unwrap();

            match (&ast1.items[0], &ast2.items[0]) {
                (
                    MakeItem::Variable { value: v1, .. },
                    MakeItem::Variable { value: v2, .. }
                ) => {
                    prop_assert_eq!(v1, v2, "Continuation must be equivalent to same-line");
                }
                _ => return Err(TestCaseError::fail("Expected Variables")),
            }
        }
    }

    // Property 3: Multiple continuations work correctly
    proptest! {
        #[test]
        fn test_SYNTAX_002_prop_multiple_continuations(
            var_name in "[A-Z][A-Z0-9_]{0,10}",
            values in prop::collection::vec("[a-z0-9_]{1,10}", 2..5)
        ) {
            // ARRANGE: Variable with multiple continuations
            let mut makefile = format!("{} = {}", var_name, values[0]);
            for value in values.iter().skip(1) {
                makefile.push_str(" \\\n    ");
                makefile.push_str(value);
            }

            // ACT: Parse
            let result = parse_makefile(&makefile);

            // ASSERT: Must parse successfully
            prop_assert!(result.is_ok(), "Multiple continuations must parse: {:?}", result);

            let ast = result.unwrap();
            prop_assert_eq!(ast.items.len(), 1);

            match &ast.items[0] {
                MakeItem::Variable { value, .. } => {
                    // All values should be present in order
                    for v in &values {
                        prop_assert!(value.contains(v), "Value {:?} should contain {:?}", value, v);
                    }
                }
                _ => return Err(TestCaseError::fail("Expected Variable")),
            }
        }
    }

    // Property 4: Continuation preserves value order
    proptest! {
        #[test]
        fn test_SYNTAX_002_prop_preserves_order(
            var_name in "[A-Z][A-Z0-9_]{0,10}",
            value1 in "[a-z]{3,5}",
            value2 in "[a-z]{3,5}",
            value3 in "[a-z]{3,5}"
        ) {
            // Skip if any values are duplicates or substrings of each other
            // (can't reliably test order with overlapping strings)
            if value1 == value2 || value2 == value3 || value1 == value3 {
                return Ok(());
            }
            if value1.contains(&value2) || value2.contains(&value1) ||
               value2.contains(&value3) || value3.contains(&value2) ||
               value1.contains(&value3) || value3.contains(&value1) {
                return Ok(());
            }

            // ARRANGE: Variable with 3 values on continued lines
            let makefile = format!("{} = {} \\\n    {} \\\n    {}", var_name, value1, value2, value3);

            // ACT: Parse
            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::Variable { value, .. } => {
                    // Find positions of values in result
                    let pos1 = value.find(&value1);
                    let pos2 = value.find(&value2);
                    let pos3 = value.find(&value3);

                    prop_assert!(pos1.is_some());
                    prop_assert!(pos2.is_some());
                    prop_assert!(pos3.is_some());

                    // Order must be preserved: value1 < value2 < value3
                    prop_assert!(pos1.unwrap() < pos2.unwrap(), "Order: {} < {}", value1, value2);
                    prop_assert!(pos2.unwrap() < pos3.unwrap(), "Order: {} < {}", value2, value3);
                }
                _ => return Err(TestCaseError::fail("Expected Variable")),
            }
        }
    }

    // Property 5: Line continuation works with all variable flavors
    proptest! {
        #[test]
        fn test_SYNTAX_002_prop_works_with_all_flavors(
            var_name in "[A-Z][A-Z0-9_]{0,10}",
            value1 in "[a-z0-9_]{1,10}",
            value2 in "[a-z0-9_]{1,10}",
            flavor in prop::sample::select(vec!["=", ":=", "?=", "+=", "!="])
        ) {
            // ARRANGE: Variable with continuation using specific flavor
            let makefile = format!("{} {} {} \\\n    {}", var_name, flavor, value1, value2);

            // ACT: Parse
            let result = parse_makefile(&makefile);

            // ASSERT: Must parse successfully regardless of flavor
            prop_assert!(result.is_ok(), "Continuation with flavor {} must parse", flavor);

            let ast = result.unwrap();
            prop_assert_eq!(ast.items.len(), 1);

            match &ast.items[0] {
                MakeItem::Variable { .. } => {
                    // Successfully parsed as variable
                }
                _ => return Err(TestCaseError::fail("Expected Variable")),
            }
        }
    }
}
