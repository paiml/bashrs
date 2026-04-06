use super::*;
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_FUNC_SUBST_001_basic_subst_always_preserved(
        var in "[A-Z]{1,8}",
        from in "\\.[a-z]{1,3}",
        to in "\\.[a-z]{1,3}",
        text in "[a-z]{1,10}"
    ) {
        // ARRANGE: Variable with $(subst from,to,text)
        let makefile = format!("{} = $(subst {},{},{})", var, from, to, text);

        // ACT: Parse makefile
        let result = parse_makefile(&makefile);

        // ASSERT: Successfully parsed
        prop_assert!(result.is_ok());

        let ast = result.unwrap();
        prop_assert_eq!(ast.items.len(), 1);

        // ASSERT: $(subst) function preserved
        match &ast.items[0] {
            MakeItem::Variable { name, value, .. } => {
                prop_assert_eq!(name, &var);
                let expected = format!("$(subst {},{},{})", from, to, text);
                prop_assert_eq!(value, &expected);
            }
            other => return Err(TestCaseError::fail(format!("Expected Variable, got {:?}", other))),
        }
    }

    #[test]
    fn prop_FUNC_SUBST_001_parsing_is_deterministic(
        var in "[A-Z]{1,8}",
        from in "\\.[a-z]{1,3}",
        to in "\\.[a-z]{1,3}",
        varref in "[A-Z]{1,8}"
    ) {
        // ARRANGE: Variable with $(subst from,to,$(VAR))
        let makefile = format!("{} = $(subst {},{},$({})) ", var, from, to, varref);

        // ACT: Parse twice
        let result1 = parse_makefile(&makefile);
        let result2 = parse_makefile(&makefile);

        // ASSERT: Same results
        prop_assert!(result1.is_ok());
        prop_assert!(result2.is_ok());

        let ast1 = result1.unwrap();
        let ast2 = result2.unwrap();

        // Same number of items
        prop_assert_eq!(ast1.items.len(), ast2.items.len());

        // Same variable value
        match (&ast1.items[0], &ast2.items[0]) {
            (MakeItem::Variable { value: v1, .. }, MakeItem::Variable { value: v2, .. }) => {
                prop_assert_eq!(v1, v2);
            }
            _ => return Err(TestCaseError::fail("Expected Variables")),
        }
    }

    #[test]
    fn prop_FUNC_SUBST_001_nested_functions_preserved(
        var in "[A-Z]{1,8}",
        from1 in "\\.[a-z]{1,2}",
        to1 in "\\.[a-z]{1,2}",
        from2 in "[a-z]{1,5}/",
        to2 in "[a-z]{1,5}/",
        varref in "[A-Z]{1,8}"
    ) {
        // ARRANGE: Nested $(subst)
        let makefile = format!(
            "{} = $(subst {},{},$(subst {},{},$({})))",
            var, from1, to1, from2, to2, varref
        );

        // ACT: Parse makefile
        let result = parse_makefile(&makefile);

        // ASSERT: Successfully parsed
        prop_assert!(result.is_ok());

        let ast = result.unwrap();
        prop_assert_eq!(ast.items.len(), 1);

        // ASSERT: Nested functions preserved
        match &ast.items[0] {
            MakeItem::Variable { value, .. } => {
                // Should contain both subst calls
                prop_assert!(value.contains("$(subst"));
                prop_assert!(value.contains(&from1));
                prop_assert!(value.contains(&to1));
                prop_assert!(value.contains(&from2));
                prop_assert!(value.contains(&to2));
            }
            other => return Err(TestCaseError::fail(format!("Expected Variable, got {:?}", other))),
        }
    }

    #[test]
    fn prop_FUNC_SUBST_001_multiple_functions_preserved(
        var1 in "[A-Z]{1,8}",
        var2 in "[A-Z]{1,8}",
        from1 in "\\.[a-z]{1,3}",
        to1 in "\\.[a-z]{1,3}",
        from2 in "\\.[a-z]{1,3}",
        to2 in "\\.[a-z]{1,3}",
        ref1 in "[A-Z]{1,8}",
        ref2 in "[A-Z]{1,8}"
    ) {
        prop_assume!(var1 != var2);

        // ARRANGE: Two variables with $(subst) functions
        let makefile = format!(
            "{} = $(subst {},{},$({})) \n{} = $(subst {},{},$({})) ",
            var1, from1, to1, ref1, var2, from2, to2, ref2
        );

        // ACT: Parse makefile
        let result = parse_makefile(&makefile);

        // ASSERT: Successfully parsed
        prop_assert!(result.is_ok());

        let ast = result.unwrap();
        prop_assert_eq!(ast.items.len(), 2);

        // ASSERT: Both functions preserved
        match &ast.items[0] {
            MakeItem::Variable { name, value, .. } => {
                prop_assert_eq!(name, &var1);
                prop_assert!(value.contains("$(subst"));
                prop_assert!(value.contains(&from1));
            }
            other => return Err(TestCaseError::fail(format!("Expected Variable, got {:?}", other))),
        }

        match &ast.items[1] {
            MakeItem::Variable { name, value, .. } => {
                prop_assert_eq!(name, &var2);
                prop_assert!(value.contains("$(subst"));
                prop_assert!(value.contains(&from2));
            }
            other => return Err(TestCaseError::fail(format!("Expected Variable, got {:?}", other))),
        }
    }

    #[test]
    fn prop_FUNC_SUBST_001_combined_with_wildcard(
        var in "[A-Z]{1,8}",
        from in "\\.[a-z]{1,3}",
        to in "\\.[a-z]{1,3}",
        pattern in "[a-z]{1,8}",
        ext in "[a-z]{1,3}"
    ) {
        // ARRANGE: $(subst) with $(wildcard)
        let makefile = format!(
            "{} = $(subst {},{},$(wildcard {}/*.{}))",
            var, from, to, pattern, ext
        );

        // ACT: Parse makefile
        let result = parse_makefile(&makefile);

        // ASSERT: Successfully parsed
        prop_assert!(result.is_ok());

        let ast = result.unwrap();
        prop_assert_eq!(ast.items.len(), 1);

        // ASSERT: Combined functions preserved
        match &ast.items[0] {
            MakeItem::Variable { value, .. } => {
                prop_assert!(value.contains("$(subst"));
                prop_assert!(value.contains("$(wildcard"));
            }
            other => return Err(TestCaseError::fail(format!("Expected Variable, got {:?}", other))),
        }
    }

    #[test]
    fn prop_FUNC_SUBST_001_no_spaces_in_function(
        var in "[A-Z]{1,8}",
        from in "[a-z]{1,5}",
        to in "[a-z]{1,5}",
        text in "[a-z]{1,10}"
    ) {
        // ARRANGE: $(subst) without spaces (single token)
        let makefile = format!("{} = $(subst {},{},{})", var, from, to, text);

        // ACT: Parse makefile
        let result = parse_makefile(&makefile);

        // ASSERT: Successfully parsed
        prop_assert!(result.is_ok());

        let ast = result.unwrap();
        prop_assert_eq!(ast.items.len(), 1);

        // ASSERT: Function preserved as one value
        match &ast.items[0] {
            MakeItem::Variable { value, .. } => {
                let expected = format!("$(subst {},{},{})", from, to, text);
                prop_assert_eq!(value, &expected);
            }
            other => return Err(TestCaseError::fail(format!("Expected Variable, got {:?}", other))),
        }
    }
}
