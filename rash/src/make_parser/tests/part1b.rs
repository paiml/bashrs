#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]

use super::super::*;
use crate::make_parser::ast::{MakeCondition, Span, VarFlavor};

    #[test]
    fn test_SYNTAX_001_mut_comment_line_numbers_correct() {
        // ARRANGE: Makefile with comments on specific lines
        let makefile = "# Line 1 comment\n# Line 2 comment\ntarget:\n\trecipe";

        // ACT: Parse makefile
        let result = parse_makefile(makefile);

        // ASSERT: Successfully parsed (verifies no infinite loop)
        assert!(result.is_ok(), "Parser must not infinite loop on comments");

        let ast = result.unwrap();
        assert_eq!(ast.items.len(), 3, "Should parse 2 comments and 1 target");

        // ASSERT: Line numbers are correct (1-indexed)
        match &ast.items[0] {
            MakeItem::Comment { text, span } => {
                assert_eq!(text, "Line 1 comment");
                assert_eq!(span.line, 1, "First comment should be on line 1");
            }
            _ => panic!("Expected Comment item"),
        }

        match &ast.items[1] {
            MakeItem::Comment { text, span } => {
                assert_eq!(text, "Line 2 comment");
                assert_eq!(span.line, 2, "Second comment should be on line 2");
            }
            _ => panic!("Expected Comment item"),
        }

        match &ast.items[2] {
            MakeItem::Target { name, span, .. } => {
                assert_eq!(name, "target");
                assert_eq!(span.line, 3, "Target should be on line 3");
            }
            _ => panic!("Expected Target item"),
        }
    }

    /// Kill mutant: line 63 `i += 1` → `i -= 1` (would cause infinite loop or crash)
    ///
    /// This test ensures that the comment parsing loop advances forward correctly.
    #[test]
    fn test_SYNTAX_001_mut_comment_loop_advances_forward() {
        // ARRANGE: Multiple consecutive comments
        let makefile = "# Comment 1\n# Comment 2\n# Comment 3\n# Comment 4\n# Comment 5";

        // ACT: Parse makefile (must not infinite loop or go backwards)
        let result = parse_makefile(makefile);

        // ASSERT: Successfully parsed all comments
        assert!(result.is_ok(), "Parser must advance through all comments");

        let ast = result.unwrap();
        assert_eq!(ast.items.len(), 5, "Should parse all 5 comments");

        // Verify all are comments
        for (i, item) in ast.items.iter().enumerate() {
            match item {
                MakeItem::Comment { text, .. } => {
                    assert_eq!(text, &format!("Comment {}", i + 1));
                }
                _ => panic!("Expected Comment item at index {}", i),
            }
        }
    }

    /// Kill mutant: Ensure comment parsing doesn't affect other parsing logic
    ///
    /// This test verifies that enabling comment parsing doesn't break
    /// the parsing of variables, targets, or other constructs.
    #[test]
    fn test_SYNTAX_001_mut_comment_parsing_isolated() {
        // ARRANGE: Complex Makefile with comments interspersed
        let makefile = r#"
# This is a header comment
CC = gcc
# Compiler flags comment
CFLAGS = -Wall -O2

# Build target comment
build: main.c
	# Recipe comment (tab-indented, should be ignored)
	$(CC) $(CFLAGS) -o output main.c

# Clean target comment
clean:
	rm -f output
"#;

        // ACT: Parse makefile
        let result = parse_makefile(makefile);

        // ASSERT: Successfully parsed
        assert!(result.is_ok());

        let ast = result.unwrap();

        // Count each item type
        let mut comment_count = 0;
        let mut variable_count = 0;
        let mut target_count = 0;

        for item in &ast.items {
            match item {
                MakeItem::Comment { .. } => comment_count += 1,
                MakeItem::Variable { .. } => variable_count += 1,
                MakeItem::Target { .. } => target_count += 1,
                _ => {} // Ignore other types for this test
            }
        }

        // ASSERT: Correct counts
        assert_eq!(comment_count, 4, "Should parse 4 non-indented comments");
        assert_eq!(variable_count, 2, "Should parse 2 variables (CC, CFLAGS)");
        assert_eq!(target_count, 2, "Should parse 2 targets (build, clean)");
    }

    /// Kill mutant: Verify span tracking is accurate for comments
    ///
    /// This ensures that mutations to span calculation are caught.
    #[test]
    fn test_SYNTAX_001_mut_span_tracking_accurate() {
        // ARRANGE: Comments with varying lengths
        let makefile = "# Short\n# Medium length comment\n# Very long comment with many words here";

        // ACT: Parse makefile
        let result = parse_makefile(makefile);

        // ASSERT: Successfully parsed
        assert!(result.is_ok());

        let ast = result.unwrap();
        assert_eq!(ast.items.len(), 3);

        // ASSERT: Spans track lengths correctly
        match &ast.items[0] {
            MakeItem::Comment { span, .. } => {
                assert_eq!(span.line, 1);
                let length = span.end - span.start;
                assert_eq!(length, "# Short".len());
            }
            _ => panic!("Expected Comment"),
        }

        match &ast.items[1] {
            MakeItem::Comment { span, .. } => {
                assert_eq!(span.line, 2);
                let length = span.end - span.start;
                assert_eq!(length, "# Medium length comment".len());
            }
            _ => panic!("Expected Comment"),
        }

        match &ast.items[2] {
            MakeItem::Comment { span, .. } => {
                assert_eq!(span.line, 3);
                let length = span.end - span.start;
                assert_eq!(length, "# Very long comment with many words here".len());
            }
            _ => panic!("Expected Comment"),
        }
    }

    /// Kill mutant: Empty comments should not break parsing
    ///
    /// Tests edge case where comment has no text after #.
    #[test]
    fn test_SYNTAX_001_mut_empty_comment_edge_case() {
        // ARRANGE: Mix of empty and non-empty comments
        let makefile = "#\n# Has text\n#\n#   \ntarget:\n\trecipe";

        // ACT: Parse makefile
        let result = parse_makefile(makefile);

        // ASSERT: Successfully parsed
        assert!(result.is_ok());

        let ast = result.unwrap();

        // Count comments (should be 4)
        let comment_count = ast
            .items
            .iter()
            .filter(|item| matches!(item, MakeItem::Comment { .. }))
            .count();

        assert_eq!(
            comment_count, 4,
            "Should parse all 4 comments, even empty ones"
        );
    }
}

// ============================================================================
// RULE-SYNTAX-002: Multiple Prerequisites Tests
// Task: Verify parser correctly handles targets with multiple prerequisites
// ============================================================================

#[cfg(test)]
mod rule_syntax_002_tests {
    use crate::make_parser::{parse_makefile, MakeItem};

    // Unit Tests
    #[test]
    fn test_RULE_SYNTAX_002_basic_multiple_prerequisites() {
        let makefile = "all: build test deploy\n\techo done";
        let result = parse_makefile(makefile);
        assert!(
            result.is_ok(),
            "Parser should handle multiple prerequisites"
        );

        let ast = result.unwrap();
        assert_eq!(ast.items.len(), 1);

        match &ast.items[0] {
            MakeItem::Target {
                name,
                prerequisites,
                recipe,
                ..
            } => {
                assert_eq!(name, "all");
                assert_eq!(prerequisites.len(), 3, "Should have 3 prerequisites");
                assert_eq!(prerequisites[0], "build");
                assert_eq!(prerequisites[1], "test");
                assert_eq!(prerequisites[2], "deploy");
                assert_eq!(recipe.len(), 1);
            }
            other => panic!("Expected Target, got {:?}", other),
        }
    }

    #[test]
    fn test_RULE_SYNTAX_002_two_prerequisites() {
        let makefile = "link: main.o util.o\n\t$(CC) -o app $^";
        let result = parse_makefile(makefile);
        assert!(result.is_ok());

        let ast = result.unwrap();
        match &ast.items[0] {
            MakeItem::Target {
                name,
                prerequisites,
                ..
            } => {
                assert_eq!(name, "link");
                assert_eq!(prerequisites.len(), 2);
                assert_eq!(prerequisites[0], "main.o");
                assert_eq!(prerequisites[1], "util.o");
            }
            _ => panic!("Expected Target"),
        }
    }

    #[test]
    fn test_RULE_SYNTAX_002_many_prerequisites() {
        let makefile = "all: a b c d e f g h\n\techo all";
        let result = parse_makefile(makefile);
        assert!(result.is_ok());

        let ast = result.unwrap();
        match &ast.items[0] {
            MakeItem::Target { prerequisites, .. } => {
                assert_eq!(prerequisites.len(), 8, "Should handle many prerequisites");
                assert_eq!(prerequisites[0], "a");
                assert_eq!(prerequisites[7], "h");
            }
            _ => panic!("Expected Target"),
        }
    }

    #[test]
    fn test_RULE_SYNTAX_002_prerequisites_with_paths() {
        let makefile = "build: src/main.c include/util.h lib/helper.c\n\tgcc -o app";
        let result = parse_makefile(makefile);
        assert!(result.is_ok());

        let ast = result.unwrap();
        match &ast.items[0] {
            MakeItem::Target { prerequisites, .. } => {
                assert_eq!(prerequisites.len(), 3);
                assert_eq!(prerequisites[0], "src/main.c");
                assert_eq!(prerequisites[1], "include/util.h");
                assert_eq!(prerequisites[2], "lib/helper.c");
            }
            _ => panic!("Expected Target"),
        }
    }
}

#[cfg(test)]
mod rule_syntax_002_property_tests {
    use crate::make_parser::{parse_makefile, MakeItem};
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_RULE_SYNTAX_002_prop_multiple_prereqs_always_parse(
            prereqs in prop::collection::vec("[a-z]{1,10}", 2..10)
        ) {
            let prereq_str = prereqs.join(" ");
            let makefile = format!("target: {}\n\techo done", prereq_str);

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok(), "Multiple prerequisites should always parse");

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::Target { prerequisites, .. } => {
                    prop_assert_eq!(prerequisites.len(), prereqs.len());
                    for (i, prereq) in prereqs.iter().enumerate() {
                        prop_assert_eq!(&prerequisites[i], prereq);
                    }
                }
                _ => return Err(TestCaseError::fail("Expected Target")),
            }
        }

        #[test]
        fn test_RULE_SYNTAX_002_prop_prereqs_order_preserved(
            prereqs in prop::collection::vec("[a-z]{1,8}", 1..15)
        ) {
            let prereq_str = prereqs.join(" ");
            let makefile = format!("all: {}\n\techo all", prereq_str);

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::Target { prerequisites, .. } => {
                    // Order must be preserved
                    for (i, expected) in prereqs.iter().enumerate() {
                        prop_assert_eq!(&prerequisites[i], expected, "Order not preserved at index {}", i);
                    }
                }
                _ => return Err(TestCaseError::fail("Expected Target")),
            }
        }

        #[test]
        fn test_RULE_SYNTAX_002_prop_prereqs_with_dots(
            names in prop::collection::vec("[a-z]{1,5}", 2..8)
        ) {
            // Create prerequisites like file.o, main.c, util.h
            let prereqs: Vec<String> = names.iter()
                .enumerate()
                .map(|(i, name)| {
                    let ext = match i % 3 {
                        0 => ".o",
                        1 => ".c",
                        _ => ".h",
                    };
                    format!("{}{}", name, ext)
                })
                .collect();

            let prereq_str = prereqs.join(" ");
            let makefile = format!("build: {}\n\tgcc", prereq_str);

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::Target { prerequisites: parsed, .. } => {
                    prop_assert_eq!(parsed.len(), prereqs.len());
                    for (i, expected) in prereqs.iter().enumerate() {
                        prop_assert_eq!(&parsed[i], expected);
                    }
                }
                _ => return Err(TestCaseError::fail("Expected Target")),
            }
        }

        #[test]
        fn test_RULE_SYNTAX_002_prop_prereqs_whitespace_normalized(
            prereqs in prop::collection::vec("[a-z]{1,6}", 2..6),
            spaces in prop::collection::vec(1..5usize, 1..5)
        ) {
            // Join prerequisites with varying amounts of spaces
            let mut prereq_str = String::new();
            for (i, prereq) in prereqs.iter().enumerate() {
                if i > 0 {
                    let space_count = spaces.get(i - 1).unwrap_or(&1);
                    prereq_str.push_str(&" ".repeat(*space_count));
                }
                prereq_str.push_str(prereq);
            }

            let makefile = format!("target: {}\n\techo", prereq_str);
            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::Target { prerequisites: parsed, .. } => {
                    // Whitespace should be normalized - all prerequisites parsed correctly
                    prop_assert_eq!(parsed.len(), prereqs.len());
                    for (i, expected) in prereqs.iter().enumerate() {
                        prop_assert_eq!(&parsed[i], expected);
                    }
                }
                _ => return Err(TestCaseError::fail("Expected Target")),
            }
        }

        #[test]
        fn test_RULE_SYNTAX_002_prop_prereqs_with_slashes(
            dirs in prop::collection::vec("[a-z]{1,5}", 2..5),
            files in prop::collection::vec("[a-z]{1,6}", 2..5)
        ) {
            // Create prerequisites like src/main.c, lib/util.o
            let prereqs: Vec<String> = dirs.iter()
                .zip(files.iter())
                .map(|(dir, file)| format!("{}/{}.c", dir, file))
                .collect();

            let prereq_str = prereqs.join(" ");
            let makefile = format!("compile: {}\n\tgcc -o app", prereq_str);

            let result = parse_makefile(&makefile);
            prop_assert!(result.is_ok());

            let ast = result.unwrap();
            match &ast.items[0] {
                MakeItem::Target { prerequisites: parsed, .. } => {
                    prop_assert_eq!(parsed.len(), prereqs.len());
                    for (i, expected) in prereqs.iter().enumerate() {
                        prop_assert_eq!(&parsed[i], expected);
                    }
                }
                _ => return Err(TestCaseError::fail("Expected Target")),
            }
        }
    }
}

#[cfg(test)]
mod rule_syntax_002_mutation_killing_tests {
    use crate::make_parser::{parse_makefile, MakeItem};

    #[test]
    fn test_RULE_SYNTAX_002_mut_split_whitespace_correctness() {
        // Target: line 203-206 split_whitespace() and map() logic
        // Kill mutants that break whitespace splitting
        let makefile = "target:   build    test     deploy  \n\techo";
        let result = parse_makefile(makefile);
        assert!(result.is_ok(), "Must handle excessive whitespace");

        let ast = result.unwrap();
        match &ast.items[0] {
            MakeItem::Target { prerequisites, .. } => {
                assert_eq!(prerequisites.len(), 3, "Must split on any whitespace");
                assert_eq!(prerequisites[0], "build");
                assert_eq!(prerequisites[1], "test");
                assert_eq!(prerequisites[2], "deploy");
                // Ensure no empty strings
                for prereq in prerequisites {
                    assert!(!prereq.is_empty(), "No empty prerequisites allowed");
                }
            }
            _ => panic!("Expected Target"),
        }
    }

