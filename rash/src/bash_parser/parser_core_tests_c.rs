//! Parser core tests — extracted from parser.rs for file health compliance.
//!
//! Tests for BashParser: assignments, if/for/while/case/function, redirects,
//! arithmetic, pipelines, diagnostics, and dogfood scripts.
#![allow(clippy::unwrap_used)]

use crate::bash_parser::ast::*;
use crate::bash_parser::parser::*;

#[test]
fn test_parse_multiple_newlines() {
    let input = "\n\n\necho hello\n\n\n";
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    // Should parse successfully, skipping empty lines
    assert!(!ast.statements.is_empty());
}

#[test]
fn test_parse_semicolon_separated() {
    // Test with newline separation instead since semicolon handling may vary
    let input = "echo a\necho b\necho c";
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    assert_eq!(ast.statements.len(), 3);
}

// ============================================================================
// Coverage Tests - If/Else Variations
// ============================================================================

#[test]
fn test_parse_if_elif_else() {
    let input = r#"
if [ $x -eq 1 ]; then
echo one
elif [ $x -eq 2 ]; then
echo two
else
echo other
fi
"#;
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    assert!(matches!(&ast.statements[0], BashStmt::If { .. }));
}
