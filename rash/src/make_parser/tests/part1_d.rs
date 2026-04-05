#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]

use super::super::*;
use crate::make_parser::ast::{MakeCondition, Span, VarFlavor};

/// RED PHASE: Test for RULE-SYNTAX-001 - Basic rule syntax
///
/// This test validates the fundamental building block of Makefiles:
/// a target with prerequisites and a recipe.
///
/// Input Makefile:
/// ```makefile
/// target: prerequisites
///     recipe
/// ```
///
/// Expected AST:
/// - One MakeItem::Target
/// - name: "target"
/// - prerequisites: ["prerequisites"]
/// - recipe: ["recipe"]
/// - phony: false (will be detected/added in purification)
        #[test]
        fn test_SYNTAX_001_prop_empty_comments_valid(
            whitespace in "[ \t]{0,10}"
        ) {
            // ARRANGE: Comment with only hash and optional whitespace
            let makefile = format!("#{}", whitespace);

            // ACT: Parse makefile
            let result = parse_makefile(&makefile);

            // ASSERT: Successfully parsed
            prop_assert!(result.is_ok());
            let ast = result.unwrap();
            prop_assert_eq!(ast.items.len(), 1);

            // ASSERT: Comment text is empty (trimmed)
            match &ast.items[0] {
                MakeItem::Comment { text, .. } => {
                    prop_assert_eq!(text, "");
                }
                _ => return Err(TestCaseError::fail("Expected Comment item")),
            }
        }
    }
}

/// MUTATION TESTING PHASE: Mutation-killing tests for SYNTAX-001
///
/// These tests target specific mutants identified during mutation testing.
#[cfg(test)]
mod syntax_001_mutation_killing_tests {
    use super::*;

    /// Kill mutant: line 60 `i + 1` → `i * 1` (would produce wrong line numbers)
    /// Kill mutant: line 63 `i += 1` → `i *= 1` (would cause infinite loop)
    ///
    /// This test verifies that comment line numbers are calculated correctly
    /// and that the parser advances past comment lines without infinite looping.
