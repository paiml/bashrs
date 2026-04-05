
    /// Property: Purification preserves number of statements (approximately)
    /// Note: Some transformations may add/remove statements
    #[test]
    fn prop_statement_count_reasonable(bash_code in "#!/bin/bash\n([a-z]=[0-9]\n){1,5}") {
        let mut parser = BashParser::new(&bash_code).unwrap();
        let ast = parser.parse().unwrap();
        let original_count = ast.statements.len();

        let mut purifier = Purifier::new(PurificationOptions::default());
        let purified_ast = purifier.purify(&ast).unwrap();
        let purified_count = purified_ast.statements.len();

        // INVARIANT: Statement count should be similar (within 50% margin)
        // This allows for reasonable transformations
        let min_expected = original_count / 2;
        let max_expected = original_count * 2;

        prop_assert!(
            purified_count >= min_expected && purified_count <= max_expected,
            "Purified statement count {} should be within reasonable range of original {}",
            purified_count, original_count
        );
    }

    /// Property: No bashisms in purified output (basic check)
    #[test]
    fn prop_no_bashisms_in_output(bash_code in "#!/bin/bash\n[a-z_][a-z0-9_]{0,10}=[0-9]{1,3}") {
        let mut parser = BashParser::new(&bash_code).unwrap();
        let ast = parser.parse().unwrap();
        let mut purifier = Purifier::new(PurificationOptions::default());
        let purified_ast = purifier.purify(&ast).unwrap();
        let output = generate_purified_bash(&purified_ast);

        // INVARIANT: Should not contain bash-specific patterns
        // (This is a basic check - could be expanded)
        prop_assert!(
            !output.contains("#!/bin/bash"),
            "Purified output should not use bash shebang"
        );

        // Common bashisms to check
        let bashisms = ["[[", "((", "${!", "=~"];
        for bashism in &bashisms {
            prop_assert!(
                !output.contains(bashism),
                "Purified output should not contain bashism: {}",
                bashism
            );
        }
    }

    /// Property: If statements with test conditions work correctly
    /// MUTATION TARGET: Tests generate_condition and generate_test_condition functions
    #[test]
    fn prop_if_with_test_conditions(
        var1 in "[a-z]",
        var2 in "[a-z]"
    ) {
        // Create bash with if/test (multi-line format)
        let bash_code = format!("#!/bin/bash\nif [ \"${}\" = \"${}\" ]\nthen\necho equal\nfi", var1, var2);

        if let Ok(mut parser) = BashParser::new(&bash_code) {
            if let Ok(ast) = parser.parse() {
                let mut purifier = Purifier::new(PurificationOptions::default());
                if let Ok(purified_ast) = purifier.purify(&ast) {
                    let output = generate_purified_bash(&purified_ast);

                    // INVARIANT: Should contain if statement
                    prop_assert!(
                        output.contains("if"),
                        "Should contain if statement, got: {}",
                        output
                    );

                    // INVARIANT: Should contain test brackets
                    prop_assert!(
                        output.contains("[") && output.contains("]"),
                        "Should contain test brackets, got: {}",
                        output
                    );

                    // INVARIANT: Should contain string equality operator
                    prop_assert!(
                        output.contains(" = "),
                        "Should contain = operator, got: {}",
                        output
                    );

                    // INVARIANT: Variables should be quoted (either single or double quotes)
                    let has_var1 = output.contains(&format!("\"${}\"", var1)) || output.contains(&format!("'${}'", var1));
                    let has_var2 = output.contains(&format!("\"${}\"", var2)) || output.contains(&format!("'${}'", var2));
                    prop_assert!(
                        has_var1 && has_var2,
                        "Variables should be quoted, got: {}",
                        output
                    );

                    // INVARIANT: Should contain then/fi
                    prop_assert!(
                        output.contains("then") && output.contains("fi"),
                        "Should contain then and fi, got: {}",
                        output
                    );
                }
            }
        }
    }

    /// Property: While loops with conditions work correctly
    /// MUTATION TARGET: Tests generate_condition function
    #[test]
    fn prop_while_with_conditions(
        var in "[a-z]",
        limit in 1..10i64
    ) {
        // Create bash with while loop (multi-line format)
        let bash_code = format!("#!/bin/bash\nwhile [ \"${}\" -lt {} ]\ndo\necho ${}\ndone", var, limit, var);

        if let Ok(mut parser) = BashParser::new(&bash_code) {
            if let Ok(ast) = parser.parse() {
                let mut purifier = Purifier::new(PurificationOptions::default());
                if let Ok(purified_ast) = purifier.purify(&ast) {
                    let output = generate_purified_bash(&purified_ast);

                    // INVARIANT: Should contain while loop
                    prop_assert!(
                        output.contains("while"),
                        "Should contain while loop, got: {}",
                        output
                    );

                    // INVARIANT: Should contain test condition
                    prop_assert!(
                        output.contains("[") && output.contains("]"),
                        "Should contain test brackets, got: {}",
                        output
                    );

                    // INVARIANT: Should contain -lt operator
                    prop_assert!(
                        output.contains(" -lt "),
                        "Should contain -lt operator, got: {}",
                        output
                    );

                    // INVARIANT: Should contain do/done
                    prop_assert!(
                        output.contains("do") && output.contains("done"),
                        "Should contain do and done, got: {}",
                        output
                    );
                }
            }
        }
    }

    /// Property: mkdir commands always get -p flag for idempotency (Phase 2)
    /// EXTREME TDD: Permission-aware purification (Toyota Way review §6.2)
    #[test]
    fn prop_mkdir_always_has_p_flag(
        dir_name in "/[a-z]{1,10}(/[a-z]{1,10}){0,2}"
    ) {
        let bash_code = format!("#!/bin/bash\nmkdir {}", dir_name);

        if let Ok(mut parser) = BashParser::new(&bash_code) {
            if let Ok(ast) = parser.parse() {
                let mut purifier = Purifier::new(PurificationOptions::default());
                if let Ok(purified_ast) = purifier.purify(&ast) {
                    let output = generate_purified_bash(&purified_ast);

                    // INVARIANT: mkdir must have -p flag
                    prop_assert!(
                        output.contains("mkdir -p") || output.contains("mkdir") && output.contains("-p"),
                        "mkdir command must have -p flag for idempotency, got: {}",
                        output
                    );
                }
            }
        }
    }

    /// Property: mkdir commands always get -p flag for idempotency
    #[test]
    fn prop_mkdir_has_p_flag(
        dir_name in "/[a-z]{1,10}(/[a-z]{1,10}){0,2}"
    ) {
        let bash_code = format!("#!/bin/bash\nmkdir {}", dir_name);

        if let Ok(mut parser) = BashParser::new(&bash_code) {
            if let Ok(ast) = parser.parse() {
                let mut purifier = Purifier::new(PurificationOptions::default());
                if let Ok(purified_ast) = purifier.purify(&ast) {
                    let output = generate_purified_bash(&purified_ast);

                    // INVARIANT: Must have mkdir -p
                    prop_assert!(
                        output.contains("mkdir -p") || (output.contains("mkdir") && output.contains("-p")),
                        "mkdir must have -p flag for idempotency, got: {}",
                        output
                    );
                }
            }
        }
    }

include!("purification_property_tests_cont_tests_prop_mkdir.rs");
