//! Issue #521: the "Unsupported expression type" error must name the
//! expression kind and where it occurs.
use super::parse;

fn err_of(src: &str) -> String {
    parse(src).unwrap_err().to_string()
}

#[test]
fn test_unsupported_try_expr_names_kind_and_line() {
    let msg = err_of("fn main() {\n    let x = foo()?;\n}");
    assert!(msg.contains("Unsupported expression type"), "{msg}");
    assert!(msg.contains("`?`"), "kind not named: {msg}");
    assert!(msg.contains("line 2"), "line not named: {msg}");
}

#[test]
fn test_unsupported_unsafe_expr_names_kind() {
    let msg = err_of("fn main() { let x = unsafe { 1 }; }");
    assert!(msg.contains("unsafe"), "kind not named: {msg}");
}

#[test]
fn test_unsupported_async_expr_names_kind() {
    let msg = err_of("fn main() { let f = async { 1 }; }");
    assert!(msg.contains("async"), "kind not named: {msg}");
}

#[test]
fn test_unsupported_expr_diagnostic_stays_unsupported_category() {
    use crate::models::diagnostic::{Diagnostic, ErrorCategory};
    let err = parse("fn main() { let x = foo()?; }").unwrap_err();
    let diag = Diagnostic::from_error(&err, None);
    assert_eq!(diag.category, ErrorCategory::UnsupportedFeature);
}
