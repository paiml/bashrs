//! Tests extracted from services/parser.rs for file health compliance.
#![allow(clippy::unwrap_used)]

use crate::services::parser::*;

use super::*;

#[test]
fn test_parse_non_function_item_skipped() {
    // Non-function items (const, struct, enum, etc.) are now gracefully skipped
    let source = r#"
        const X: u32 = 42;

        fn main() {
            let x = 1;
        }
    "#;
    let result = parse(source);
    assert!(
        result.is_ok(),
        "Non-function items should be gracefully skipped: {:?}",
        result.err()
    );
    let ast = result.expect("parse should succeed");
    assert_eq!(ast.functions.len(), 1);
    assert_eq!(ast.entry_point, "main");
}

#[test]
fn test_parse_legacy_rash_main_attribute() {
    let source = r#"
        #[rash::main]
        fn entry() {
            let x = 1;
        }
    "#;
    let ast = parse(source).unwrap();
    assert_eq!(ast.entry_point, "entry");
}
