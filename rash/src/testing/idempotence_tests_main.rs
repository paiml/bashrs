fn test_complex_boolean_conditions_idempotent() {
    let source = r#"
        fn main() {
            let a = true;
            let b = false;
            let c = true;

            if a && b {
                write_file("result.txt", "a and b");
            } else if a || c {
                write_file("result.txt", "a or c");
            } else {
                write_file("result.txt", "neither");
            }
        }

        fn write_file(path: &str, content: &str) {
            let noop = true;
        }
    "#;

    let config = Config::default();
    let shell = transpile(source, &config).unwrap();

    let temp_dir1 = TempDir::new().unwrap();
    let state1 = execute_and_capture_state(&shell, &temp_dir1);

    let temp_dir2 = TempDir::new().unwrap();
    let state2 = execute_and_capture_state(&shell, &temp_dir2);

    assert_eq!(state1, state2, "Complex boolean conditions not idempotent");
}

// ============================================================================
// PROPERTY 8: Deterministic script execution (smoke test)
// ============================================================================

#[test]
fn test_simple_script_deterministic() {
    let source = r#"
        fn main() {
            write_file("test.txt", "hello world");
        }

        fn write_file(path: &str, content: &str) {
            let noop = true;
        }
    "#;

    let config = Config::default();
    let shell = transpile(source, &config).unwrap();

    // Run 10 times
    let states: Vec<ScriptState> = (0..10)
        .map(|_| {
            let temp = TempDir::new().unwrap();
            execute_and_capture_state(&shell, &temp)
        })
        .collect();

    // All must be identical
    for (i, state) in states.iter().enumerate().skip(1) {
        assert_eq!(&states[0], state, "Non-deterministic on run {}", i);
    }
}

// ============================================================================
// PROPERTY 9: No unintended side effects in conditions
// ============================================================================

#[test]
fn test_condition_evaluation_no_side_effects() {
    let source = r#"
        fn main() {
            let counter = 0;

            // Condition should not modify state
            if counter == 0 {
                write_file("zero.txt", "counter is zero");
            }

            // Counter should still be 0
            if counter == 0 {
                write_file("still_zero.txt", "counter still zero");
            }
        }

        fn write_file(path: &str, content: &str) {
            let noop = true;
        }
    "#;

    let config = Config::default();
    let shell = transpile(source, &config).unwrap();

    let temp_dir1 = TempDir::new().unwrap();
    let state1 = execute_and_capture_state(&shell, &temp_dir1);

    let temp_dir2 = TempDir::new().unwrap();
    let state2 = execute_and_capture_state(&shell, &temp_dir2);

    // Both files should be created both times
    assert_eq!(state1.files.len(), state2.files.len());
    assert_eq!(state1, state2, "Condition evaluation has side effects");
}

// ============================================================================
// PROPERTY 10: Empty branches don't cause issues
// ============================================================================

#[test]
fn test_empty_branches_idempotent() {
    let source = r#"
        fn main() {
            let condition = true;

            if condition {
                // Empty true branch
            } else {
                write_file("else.txt", "else branch");
            }

            write_file("after.txt", "after if");
        }

        fn write_file(path: &str, content: &str) {
            let noop = true;
        }
    "#;

    let config = Config::default();
    let shell = transpile(source, &config).unwrap();

    let temp_dir1 = TempDir::new().unwrap();
    let state1 = execute_and_capture_state(&shell, &temp_dir1);

    let temp_dir2 = TempDir::new().unwrap();
    let state2 = execute_and_capture_state(&shell, &temp_dir2);

    assert_eq!(state1, state2, "Empty branches not idempotent");
}

// ============================================================================
// Future: Property-based tests with QuickCheck/Proptest
// ============================================================================
//
// These would test arbitrary combinations of:
// - Conditions (bool expressions)
// - Branch depths (nested if/else)
// - Statement types in branches
// - Variable scoping
//
// Example (to be implemented):
//
// #[proptest]
// fn prop_generated_if_else_always_idempotent(
//     #[strategy(bool_expr_strategy())] condition: BoolExpr,
//     #[strategy(statement_list_strategy())] then_stmts: Vec<Statement>,
//     #[strategy(statement_list_strategy())] else_stmts: Vec<Statement>,
// ) {
//     // Generate source from AST
//     // Transpile
//     // Run twice
//     // Assert states identical
// }
