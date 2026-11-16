//! Issue #521: the "Unsupported expression type" error must name the
//! expression kind and where it occurs.
//!
//! Each row of `CASES` is a source snippet that really reaches the
//! unsupported-expression arm of `convert_expr` (the test asserts the full
//! message, so a row that is handled earlier fails instead of passing
//! silently). `expected_line` is the line where the offending expression
//! STARTS, which differs from its end line for the multi-line rows.
use super::parse;

struct Case {
    src: &'static str,
    kind: &'static str,
    expected_line: usize,
}

const CASES: &[Case] = &[
    Case { src: "fn main() {\n    let x = foo()?;\n}", kind: "`?` (try operator)", expected_line: 2 },
    Case { src: "fn main() { let x = unsafe { 1 }; }", kind: "`unsafe` block", expected_line: 1 },
    Case { src: "fn main() { let f = async { 1 }; }", kind: "`async` block", expected_line: 1 },
    Case { src: "fn main() { let x = f().await; }", kind: "`.await`", expected_line: 1 },
    Case { src: "fn main() { let x = for i in 0..3 {}; }", kind: "`for` loop in expression position", expected_line: 1 },
    Case { src: "fn main() { let x = while c {}; }", kind: "`while` loop in expression position", expected_line: 1 },
    Case { src: "fn main() { let x = loop { break 1; }; }", kind: "`loop` in expression position", expected_line: 1 },
    Case { src: "fn main() { let x = break; }", kind: "`break`", expected_line: 1 },
    Case { src: "fn main() { let x = continue; }", kind: "`continue`", expected_line: 1 },
    Case { src: "fn main() { let x = (a = 1); }", kind: "assignment in expression position", expected_line: 1 },
    Case { src: "fn main() { let x = const { 1 }; }", kind: "`const` block", expected_line: 1 },
    Case { src: "fn main() { let x = _; }", kind: "`_` placeholder", expected_line: 1 },
    Case { src: "fn main() { let p = &raw const x; }", kind: "raw address-of (`&raw`)", expected_line: 1 },
    Case { src: "fn main() { let x = try { 1 }; }", kind: "`try` block", expected_line: 1 },
    Case { src: "fn main() { let x = yield 1; }", kind: "`yield`", expected_line: 1 },
    Case { src: "fn main() { let x = become f(); }", kind: "unparsed (verbatim) expression", expected_line: 1 },
    // Expression starts on line 4 and ends on line 6: a hardcoded line or the
    // span end (6) both fail.
    Case {
        src: "fn main() {\n    let a = 1;\n    let b = 2;\n    let x = loop {\n        break a;\n    };\n}",
        kind: "`loop` in expression position",
        expected_line: 4,
    },
];

fn err_of(src: &str) -> String {
    parse(src).unwrap_err().to_string()
}

#[test]
fn test_unsupported_expr_names_kind_and_line_table() {
    for case in CASES {
        let msg = err_of(case.src);
        assert!(
            msg.ends_with(&format!(
                "Unsupported expression type: {} at line {}",
                case.kind, case.expected_line
            )),
            "src {:?}: got {msg}",
            case.src
        );
    }
}

#[test]
fn test_unsupported_expr_diagnostic_stays_unsupported_category() {
    use crate::models::diagnostic::{Diagnostic, ErrorCategory};
    let err = parse("fn main() { let x = foo()?; }").unwrap_err();
    let diag = Diagnostic::from_error(&err, None);
    assert_eq!(diag.category, ErrorCategory::UnsupportedFeature);
}
