#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(unused_imports)]

use super::super::*;
use crate::make_parser::ast::{MakeCondition, Span, VarFlavor};

// Phase 5 - MUTATION TESTING: Mutation-killing tests for = recursive assignment

// Target: parser.rs:116 - is_variable_assignment() contains('=') check
#[test]
fn test_RECIPE_001_mut_non_tab_line_must_break_loop() {
    // MUTATION TARGET: line 283 in parser.rs (break on non-tab line)
    // Mutation: Replace break with continue or remove it
    // This test kills mutations that fail to stop at non-tab lines

    // ARRANGE: Recipe followed by variable assignment
    let makefile = "build:\n\tcargo build\nCC = gcc\n\tthis should not be recipe";

    // ACT: Parse
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();

    // ASSERT: Recipe should stop at CC = gcc
    assert_eq!(ast.items.len(), 2, "Should have target and variable");

    match &ast.items[0] {
        MakeItem::Target { recipe, .. } => {
            assert_eq!(recipe.len(), 1, "Recipe should stop at non-tab line");
            assert_eq!(recipe[0], "cargo build");
        }
        _ => panic!("Expected Target"),
    }

    match &ast.items[1] {
        MakeItem::Variable { name, .. } => {
            assert_eq!(name, "CC", "Variable should be parsed after recipe ends");
        }
        _ => panic!("Expected Variable"),
    }

    // CRITICAL: If break is removed, parsing would be incorrect
}

#[test]
fn test_RECIPE_001_mut_index_increment_must_happen() {
    // MUTATION TARGET: line 271 in parser.rs (*index += 1 in recipe loop)
    // Mutation: Remove or change index increment
    // This test kills mutations that break loop progression

    // ARRANGE: Target with multiple recipes followed by another target
    let makefile = "first:\n\tcommand1\n\tcommand2\nsecond:\n\tcommand3";

    // ACT: Parse
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();

    // ASSERT: Both targets should be parsed correctly
    assert_eq!(ast.items.len(), 2, "Should parse both targets");

    match &ast.items[0] {
        MakeItem::Target { name, recipe, .. } => {
            assert_eq!(name, "first");
            assert_eq!(recipe.len(), 2, "First target should have 2 recipes");
        }
        _ => panic!("Expected Target"),
    }

    match &ast.items[1] {
        MakeItem::Target { name, recipe, .. } => {
            assert_eq!(name, "second");
            assert_eq!(recipe.len(), 1, "Second target should have 1 recipe");
        }
        _ => panic!("Expected Target"),
    }

    // CRITICAL: If index isn't incremented, parser would loop infinitely or parse incorrectly
}

// ============================================================================
// Sprint 39: RECIPE-002 - Multi-line recipes
// ============================================================================
// Implements: Multiple recipe lines for a single target
// Verifies: Parser correctly collects all tab-indented recipe lines

// Phase 1 - RED: Unit tests for multi-line recipes

#[test]
fn test_RECIPE_002_basic_three_line_recipe() {
    // ARRANGE: Target with 3 distinct recipe lines
    let makefile = "deploy:\n\tcargo build --release\n\tcargo test\n\tcp target/release/app /opt/";

    // ACT: Parse the makefile
    let result = parse_makefile(makefile);

    // ASSERT: All 3 recipe lines should be parsed
    assert!(result.is_ok(), "Parser should handle 3-line recipe");

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 1, "Should parse one target");

    match &ast.items[0] {
        MakeItem::Target { name, recipe, .. } => {
            assert_eq!(name, "deploy", "Target name should be 'deploy'");
            assert_eq!(recipe.len(), 3, "Should have three recipe lines");
            assert_eq!(recipe[0], "cargo build --release", "First recipe line");
            assert_eq!(recipe[1], "cargo test", "Second recipe line");
            assert_eq!(
                recipe[2], "cp target/release/app /opt/",
                "Third recipe line"
            );
        }
        other => panic!("Expected Target, got {:?}", other),
    }
}

#[test]
fn test_RECIPE_002_many_recipe_lines() {
    // ARRANGE: Target with 5 recipe lines (typical CI/CD deploy)
    let makefile = "ci:\n\techo 'Starting CI'\n\tcargo fmt --check\n\tcargo clippy\n\tcargo test\n\techo 'CI passed'";

    // ACT: Parse
    let result = parse_makefile(makefile);
    assert!(result.is_ok(), "Many recipe lines should parse");

    let ast = result.unwrap();
    match &ast.items[0] {
        MakeItem::Target { name, recipe, .. } => {
            assert_eq!(name, "ci");
            assert_eq!(recipe.len(), 5, "Should have five recipe lines");
            assert_eq!(recipe[0], "echo 'Starting CI'");
            assert_eq!(recipe[4], "echo 'CI passed'");
        }
        _ => panic!("Expected Target"),
    }
}

#[test]
fn test_RECIPE_002_recipe_order_preserved() {
    // ARRANGE: Recipe with specific ordering (important for build steps)
    let makefile = "build:\n\tmkdir -p dist\n\tcargo build --release\n\tcp target/release/app dist/\n\tstrip dist/app";

    // ACT: Parse
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();
    match &ast.items[0] {
        MakeItem::Target { recipe, .. } => {
            // Order matters: mkdir before cp, build before cp, strip last
            assert_eq!(recipe.len(), 4);
            assert_eq!(recipe[0], "mkdir -p dist", "Step 1: create directory");
            assert_eq!(recipe[1], "cargo build --release", "Step 2: build");
            assert_eq!(recipe[2], "cp target/release/app dist/", "Step 3: copy");
            assert_eq!(recipe[3], "strip dist/app", "Step 4: strip");
        }
        _ => panic!("Expected Target"),
    }
}

#[test]
fn test_RECIPE_002_different_targets_different_recipes() {
    // ARRANGE: Multiple targets each with multiple recipe lines
    let makefile = "build:\n\tcargo build\n\tcargo test\n\nclean:\n\trm -rf target\n\tfind . -name '*.o' -delete";

    // ACT: Parse
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();
    assert_eq!(ast.items.len(), 2, "Should have two targets");

    match &ast.items[0] {
        MakeItem::Target { name, recipe, .. } => {
            assert_eq!(name, "build");
            assert_eq!(recipe.len(), 2, "build has 2 recipe lines");
        }
        _ => panic!("Expected first Target"),
    }

    match &ast.items[1] {
        MakeItem::Target { name, recipe, .. } => {
            assert_eq!(name, "clean");
            assert_eq!(recipe.len(), 2, "clean has 2 recipe lines");
        }
        _ => panic!("Expected second Target"),
    }
}

// Phase 4 - PROPERTY TESTING: Property tests for multi-line recipes

#[cfg(test)]
mod recipe_002_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_RECIPE_002_varying_recipe_line_count_always_parses(
            num_lines in 2usize..10,
            target_name in "[a-z][a-z0-9_]*"
        ) {
            // ARRANGE: Generate target with varying number of recipe lines (2-9)
            let mut makefile = format!("{}:\n", target_name);
            for i in 0..num_lines {
                makefile.push_str(&format!("\techo 'step {}'\n", i));
            }

            // ACT: Parse
            let result = parse_makefile(&makefile);

            // ASSERT: Should always parse successfully
            prop_assert!(result.is_ok(), "Should parse {} recipe lines", num_lines);

            let ast = result.unwrap();
            prop_assert_eq!(ast.items.len(), 1, "Should have one target");

            match &ast.items[0] {
                MakeItem::Target { name, recipe, .. } => {
                    prop_assert_eq!(name, &target_name, "Target name should match");
                    prop_assert_eq!(recipe.len(), num_lines, "Should have {} recipe lines", num_lines);
                }
                _ => return Err(TestCaseError::fail("Expected Target")),
            }
        }

        #[test]
        fn prop_RECIPE_002_multi_line_parsing_is_deterministic(
            target in "[a-z][a-z0-9_]*",
            cmd1 in "[a-z][a-z0-9 ]*",
            cmd2 in "[a-z][a-z0-9 ]*"
        ) {
            // ARRANGE: Multi-line recipe
            let makefile = format!("{}:\n\t{}\n\t{}", target, cmd1, cmd2);

            // ACT: Parse twice
            let result1 = parse_makefile(&makefile);
            let result2 = parse_makefile(&makefile);

            // ASSERT: Should produce identical results
            prop_assert!(result1.is_ok() && result2.is_ok());

            let ast1 = result1.unwrap();
            let ast2 = result2.unwrap();

            match (&ast1.items[0], &ast2.items[0]) {
                (MakeItem::Target { recipe: r1, .. }, MakeItem::Target { recipe: r2, .. }) => {
                    prop_assert_eq!(r1.len(), r2.len(), "Recipe lengths should match");
                    prop_assert_eq!(r1, r2, "Recipes should be identical");
                }
                _ => return Err(TestCaseError::fail("Expected Target")),
            }
        }

        #[test]
        fn prop_RECIPE_002_all_recipe_lines_collected(
            line_count in 2usize..8
        ) {
            // ARRANGE: Generate recipe with specific line count
            let mut makefile = "target:\n".to_string();
            let expected_lines: Vec<String> = (0..line_count)
                .map(|i| format!("command_{}", i))
                .collect();

            for cmd in &expected_lines {
                makefile.push_str(&format!("\t{}\n", cmd));
            }

            // ACT: Parse
            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();

            // ASSERT: All recipe lines should be present
            match &ast.items[0] {
                MakeItem::Target { recipe, .. } => {
                    prop_assert_eq!(recipe.len(), line_count, "Should collect all lines");

                    for (i, expected) in expected_lines.iter().enumerate() {
                        prop_assert_eq!(
                            &recipe[i],
                            expected,
                            "Recipe line {} should match",
                            i
                        );
                    }
                }
                _ => return Err(TestCaseError::fail("Expected Target")),
            }
        }

        #[test]
        fn prop_RECIPE_002_recipe_order_always_preserved(
            num_lines in 2usize..6
        ) {
            // ARRANGE: Recipe with numbered commands
            let mut makefile = "build:\n".to_string();
            for i in 0..num_lines {
                makefile.push_str(&format!("\tstep_{}\n", i));
            }

            // ACT: Parse
            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();

            // ASSERT: Order must be preserved
            match &ast.items[0] {
                MakeItem::Target { recipe, .. } => {
                    for (i, line) in recipe.iter().enumerate().take(num_lines) {
                        let expected = format!("step_{}", i);
                        prop_assert_eq!(
                            line,
                            &expected,
                            "Line {} should be in order",
                            i
                        );
                    }
                }
                _ => return Err(TestCaseError::fail("Expected Target")),
            }
        }

        #[test]
        fn prop_RECIPE_002_complex_commands_in_multiline_recipe(
            target in "[a-z][a-z0-9_]*"
        ) {
            // ARRANGE: Multi-line recipe with complex realistic commands
            let makefile = format!(
                "{}:\n\tmkdir -p target/release\n\tcargo build --release --features prod\n\tstrip target/release/{}\n\tcp target/release/{} /opt/bin/",
                target, target, target
            );

            // ACT: Parse
            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok(), "Complex multi-line recipes should parse");

            let ast = result.unwrap();

            // ASSERT: All complex commands should be parsed correctly
            match &ast.items[0] {
                MakeItem::Target { recipe, .. } => {
                    prop_assert_eq!(recipe.len(), 4, "Should have 4 recipe lines");
                    prop_assert!(recipe[0].starts_with("mkdir -p"), "First command");
                    prop_assert!(recipe[1].starts_with("cargo build"), "Second command");
                    prop_assert!(recipe[2].starts_with("strip"), "Third command");
                    prop_assert!(recipe[3].starts_with("cp"), "Fourth command");
                }
                _ => return Err(TestCaseError::fail("Expected Target")),
            }
        }
    }
}

// Phase 5 - MUTATION TESTING: Mutation-killing tests for multi-line recipes

#[test]
fn test_RECIPE_002_mut_all_recipe_lines_must_be_collected() {
    // MUTATION TARGET: line 270 in parser.rs
    // Mutation: Skip push or only push first/last line
    // This test kills mutations that fail to collect all recipe lines

    // ARRANGE: Target with 3 distinct recipe lines
    let makefile = "deploy:\n\tline1\n\tline2\n\tline3";

    // ACT: Parse
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();

    // ASSERT: MUST have ALL 3 lines
    match &ast.items[0] {
        MakeItem::Target { recipe, .. } => {
            assert_eq!(recipe.len(), 3, "MUST collect all 3 lines");
            assert_eq!(recipe[0], "line1", "First line must be present");
            assert_eq!(recipe[1], "line2", "Middle line must be present");
            assert_eq!(recipe[2], "line3", "Last line must be present");
        }
        _ => panic!("Expected Target"),
    }

    // CRITICAL: If recipe.push() is mutated or conditional, this test fails
}

#[test]
fn test_RECIPE_002_mut_recipe_count_must_be_exact() {
    // MUTATION TARGET: line 270 in parser.rs
    // Mutation: Push multiple times or skip lines
    // This test kills mutations that change the count of recipe lines

    // ARRANGE: Makefile with exactly 4 recipe lines
    let makefile = "build:\n\tcmd1\n\tcmd2\n\tcmd3\n\tcmd4";

    // ACT: Parse
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();

    // ASSERT: MUST have EXACTLY 4 lines (not 3, not 5)
    match &ast.items[0] {
        MakeItem::Target { recipe, .. } => {
            assert_eq!(
                recipe.len(),
                4,
                "MUST have exactly 4 recipe lines, no more, no less"
            );
        }
        _ => panic!("Expected Target"),
    }

    // CRITICAL: If loop mutates to skip lines or duplicate, count will be wrong
}

#[test]
fn test_RECIPE_002_mut_loop_bounds_must_be_correct() {
    // MUTATION TARGET: line 265 in parser.rs
    // Mutation: Replace < with <= or !=
    // This test kills mutations that break loop bounds

    // ARRANGE: Recipe at end of file (boundary condition)
    let makefile = "final:\n\techo last\n\techo done";

    // ACT: Parse
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();

    // ASSERT: Should handle EOF correctly
    match &ast.items[0] {
        MakeItem::Target { recipe, .. } => {
            assert_eq!(recipe.len(), 2, "Should parse both lines at EOF");
            assert_eq!(recipe[0], "echo last");
            assert_eq!(recipe[1], "echo done");
        }
        _ => panic!("Expected Target"),
    }

    // CRITICAL: If loop bounds are wrong (*index <= lines.len()), it would panic
}

#[test]
fn test_RECIPE_002_mut_recipe_vec_must_accumulate() {
    // MUTATION TARGET: line 263 in parser.rs
    // Mutation: Don't initialize Vec or clear it each iteration
    // This test kills mutations that break Vec accumulation

    // ARRANGE: Target with multiple distinct lines
    let makefile = "accumulate:\n\tfirst\n\tsecond\n\tthird\n\tfourth\n\tfifth";

    // ACT: Parse
    let result = parse_makefile(makefile);
    assert!(result.is_ok());

    let ast = result.unwrap();

    // ASSERT: Vec must accumulate all 5 lines
    match &ast.items[0] {
        MakeItem::Target { recipe, .. } => {
            assert_eq!(recipe.len(), 5, "Vec must accumulate 5 lines");

            // Verify all unique lines are present (not duplicates or missing)
            assert_eq!(recipe[0], "first");
            assert_eq!(recipe[1], "second");
            assert_eq!(recipe[2], "third");
            assert_eq!(recipe[3], "fourth");
            assert_eq!(recipe[4], "fifth");
        }
        _ => panic!("Expected Target"),
    }

    // CRITICAL: If Vec is cleared or not accumulated properly, test fails
}

