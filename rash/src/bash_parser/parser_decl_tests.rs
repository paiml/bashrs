//! Tests for `parse_assignment` in `parser_decl.rs`.
//!
//! Covers normal assignments, keyword-as-variable names, array element
//! assignments, the append (`+=`) operator, empty assignments, exported
//! assignments, and error cases.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::ast::{BashExpr, BashStmt};
use super::parser::BashParser;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Parse a single bash statement from `input` and return it.
fn parse_first(input: &str) -> BashStmt {
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    assert!(
        !ast.statements.is_empty(),
        "expected at least one statement from: {input}"
    );
    ast.statements.into_iter().next().unwrap()
}

/// Parse `input` and expect a parse/lex error.
fn parse_err(input: &str) {
    let result = BashParser::new(input).and_then(|mut p| p.parse());
    assert!(result.is_err(), "expected error from: {input}");
}

// ===========================================================================
// Normal assignments
// ===========================================================================

#[test]
fn test_parse_assignment_integer() {
    let stmt = parse_first("x=5");
    match stmt {
        BashStmt::Assignment {
            name,
            index,
            value,
            exported,
            ..
        } => {
            assert_eq!(name, "x");
            assert!(index.is_none());
            assert_eq!(value, BashExpr::Literal("5".to_string()));
            assert!(!exported);
        }
        other => panic!("expected Assignment, got {other:?}"),
    }
}

#[test]
fn test_parse_assignment_string_double_quoted() {
    let stmt = parse_first(r#"name="hello""#);
    match stmt {
        BashStmt::Assignment {
            name,
            index,
            value,
            exported,
            ..
        } => {
            assert_eq!(name, "name");
            assert!(index.is_none());
            assert_eq!(value, BashExpr::Literal("hello".to_string()));
            assert!(!exported);
        }
        other => panic!("expected Assignment, got {other:?}"),
    }
}

#[test]
fn test_parse_assignment_string_single_quoted() {
    let stmt = parse_first("greeting='world'");
    match stmt {
        BashStmt::Assignment {
            name,
            index,
            value,
            exported,
            ..
        } => {
            assert_eq!(name, "greeting");
            assert!(index.is_none());
            assert_eq!(value, BashExpr::Literal("world".to_string()));
            assert!(!exported);
        }
        other => panic!("expected Assignment, got {other:?}"),
    }
}

#[test]
fn test_parse_assignment_identifier_value() {
    let stmt = parse_first("x=abc");
    match stmt {
        BashStmt::Assignment { name, value, .. } => {
            assert_eq!(name, "x");
            // The value may be parsed as Literal or Identifier-based Literal
            match &value {
                BashExpr::Literal(s) => assert_eq!(s, "abc"),
                other => panic!("expected Literal value, got {other:?}"),
            }
        }
        other => panic!("expected Assignment, got {other:?}"),
    }
}

// ===========================================================================
// Keyword-as-variable-name assignments
// ===========================================================================

#[test]
fn test_parse_assignment_keyword_if() {
    let stmt = parse_first("if=1");
    match stmt {
        BashStmt::Assignment { name, value, .. } => {
            assert_eq!(name, "if");
            assert_eq!(value, BashExpr::Literal("1".to_string()));
        }
        other => panic!("expected Assignment for 'if=1', got {other:?}"),
    }
}

#[test]
fn test_parse_assignment_keyword_done() {
    let stmt = parse_first("done=2");
    match stmt {
        BashStmt::Assignment { name, value, .. } => {
            assert_eq!(name, "done");
            assert_eq!(value, BashExpr::Literal("2".to_string()));
        }
        other => panic!("expected Assignment for 'done=2', got {other:?}"),
    }
}

#[test]
fn test_parse_assignment_keyword_case() {
    let stmt = parse_first("case=3");
    match stmt {
        BashStmt::Assignment { name, value, .. } => {
            assert_eq!(name, "case");
            assert_eq!(value, BashExpr::Literal("3".to_string()));
        }
        other => panic!("expected Assignment for 'case=3', got {other:?}"),
    }
}

#[test]
fn test_parse_assignment_keyword_fi() {
    let stmt = parse_first("fi=100");
    match stmt {
        BashStmt::Assignment { name, value, .. } => {
            assert_eq!(name, "fi");
            assert_eq!(value, BashExpr::Literal("100".to_string()));
        }
        other => panic!("expected Assignment for 'fi=100', got {other:?}"),
    }
}

#[test]
fn test_parse_assignment_keyword_for() {
    let stmt = parse_first("for=hello");
    match stmt {
        BashStmt::Assignment { name, value, .. } => {
            assert_eq!(name, "for");
            assert_eq!(value, BashExpr::Literal("hello".to_string()));
        }
        other => panic!("expected Assignment for 'for=hello', got {other:?}"),
    }
}

#[test]
fn test_parse_assignment_keyword_while() {
    let stmt = parse_first("while=yes");
    match stmt {
        BashStmt::Assignment { name, value, .. } => {
            assert_eq!(name, "while");
            assert_eq!(value, BashExpr::Literal("yes".to_string()));
        }
        other => panic!("expected Assignment for 'while=yes', got {other:?}"),
    }
}

#[test]
fn test_parse_assignment_keyword_then() {
    let stmt = parse_first("then=42");
    match stmt {
        BashStmt::Assignment { name, value, .. } => {
            assert_eq!(name, "then");
            assert_eq!(value, BashExpr::Literal("42".to_string()));
        }
        other => panic!("expected Assignment for 'then=42', got {other:?}"),
    }
}

#[test]
fn test_parse_assignment_keyword_elif() {
    let stmt = parse_first("elif=99");
    match stmt {
        BashStmt::Assignment { name, value, .. } => {
            assert_eq!(name, "elif");
            assert_eq!(value, BashExpr::Literal("99".to_string()));
        }
        other => panic!("expected Assignment for 'elif=99', got {other:?}"),
    }
}

#[test]
fn test_parse_assignment_keyword_else() {
    let stmt = parse_first("else=0");
    match stmt {
        BashStmt::Assignment { name, value, .. } => {
            assert_eq!(name, "else");
            assert_eq!(value, BashExpr::Literal("0".to_string()));
        }
        other => panic!("expected Assignment for 'else=0', got {other:?}"),
    }
}

#[test]
fn test_parse_assignment_keyword_do() {
    let stmt = parse_first("do=loop");
    match stmt {
        BashStmt::Assignment { name, value, .. } => {
            assert_eq!(name, "do");
            assert_eq!(value, BashExpr::Literal("loop".to_string()));
        }
        other => panic!("expected Assignment for 'do=loop', got {other:?}"),
    }
}

#[test]
fn test_parse_assignment_keyword_esac() {
    let stmt = parse_first("esac=end");
    match stmt {
        BashStmt::Assignment { name, value, .. } => {
            assert_eq!(name, "esac");
            assert_eq!(value, BashExpr::Literal("end".to_string()));
        }
        other => panic!("expected Assignment for 'esac=end', got {other:?}"),
    }
}

#[test]
fn test_parse_assignment_keyword_in() {
    let stmt = parse_first("in=list");
    match stmt {
        BashStmt::Assignment { name, value, .. } => {
            assert_eq!(name, "in");
            assert_eq!(value, BashExpr::Literal("list".to_string()));
        }
        other => panic!("expected Assignment for 'in=list', got {other:?}"),
    }
}

#[test]
fn test_parse_assignment_keyword_function() {
    let stmt = parse_first("function=fn");
    match stmt {
        BashStmt::Assignment { name, value, .. } => {
            assert_eq!(name, "function");
            assert_eq!(value, BashExpr::Literal("fn".to_string()));
        }
        other => panic!("expected Assignment for 'function=fn', got {other:?}"),
    }
}

#[test]
fn test_parse_assignment_keyword_return() {
    let stmt = parse_first("return=back");
    match stmt {
        BashStmt::Assignment { name, value, .. } => {
            assert_eq!(name, "return");
            assert_eq!(value, BashExpr::Literal("back".to_string()));
        }
        other => panic!("expected Assignment for 'return=back', got {other:?}"),
    }
}

// ===========================================================================
// Array element assignment: name[index]=value
// ===========================================================================

#[test]
fn test_parse_assignment_array_numeric_index() {
    let stmt = parse_first("arr[0]=val");
    match stmt {
        BashStmt::Assignment {
            name, index, value, ..
        } => {
            assert_eq!(name, "arr");
            assert_eq!(index, Some("0".to_string()));
            assert_eq!(value, BashExpr::Literal("val".to_string()));
        }
        other => panic!("expected Assignment with array index, got {other:?}"),
    }
}

#[test]
fn test_parse_assignment_array_identifier_index() {
    let stmt = parse_first("arr[idx]=val");
    match stmt {
        BashStmt::Assignment {
            name, index, value, ..
        } => {
            assert_eq!(name, "arr");
            assert_eq!(index, Some("idx".to_string()));
            assert_eq!(value, BashExpr::Literal("val".to_string()));
        }
        other => panic!("expected Assignment with identifier index, got {other:?}"),
    }
}

#[test]
fn test_parse_assignment_array_variable_index() {
    let stmt = parse_first("arr[$var]=val");
    match stmt {
        BashStmt::Assignment {
            name, index, value, ..
        } => {
            assert_eq!(name, "arr");
            assert_eq!(index, Some("$var".to_string()));
            assert_eq!(value, BashExpr::Literal("val".to_string()));
        }
        other => panic!("expected Assignment with $var index, got {other:?}"),
    }
}

#[test]
fn test_parse_assignment_array_string_index() {
    let stmt = parse_first(r#"hash["key"]=val"#);
    match stmt {
        BashStmt::Assignment {
            name, index, value, ..
        } => {
            assert_eq!(name, "hash");
            assert_eq!(index, Some("key".to_string()));
            assert_eq!(value, BashExpr::Literal("val".to_string()));
        }
        other => panic!("expected Assignment with string index, got {other:?}"),
    }
}

#[test]
fn test_parse_assignment_array_large_numeric_index() {
    let stmt = parse_first("data[42]=answer");
    match stmt {
        BashStmt::Assignment {
            name, index, value, ..
        } => {
            assert_eq!(name, "data");
            assert_eq!(index, Some("42".to_string()));
            assert_eq!(value, BashExpr::Literal("answer".to_string()));
        }
        other => panic!("expected Assignment with numeric index 42, got {other:?}"),
    }
}

// ===========================================================================
// Append operator: +=
// ===========================================================================

#[test]
fn test_parse_assignment_append() {
    // The lexer tokenizes `x+=more` as Identifier("x"), Identifier("+="), Identifier("more")
    // but parse_statement dispatches to parse_identifier_statement which handles +=
    let stmt = parse_first("x+=more");
    match stmt {
        BashStmt::Assignment { name, value, .. } => {
            assert_eq!(name, "x");
            // The value should be "more"
            match &value {
                BashExpr::Literal(s) => assert_eq!(s, "more"),
                other => panic!("expected Literal value for append, got {other:?}"),
            }
        }
        other => panic!("expected Assignment for 'x+=more', got {other:?}"),
    }
}

#[test]
fn test_parse_assignment_append_string() {
    let stmt = parse_first(r#"path+="/bin""#);
    match stmt {
        BashStmt::Assignment { name, value, .. } => {
            assert_eq!(name, "path");
            assert_eq!(value, BashExpr::Literal("/bin".to_string()));
        }
        other => panic!("expected Assignment for append with string, got {other:?}"),
    }
}

// ===========================================================================
// Empty assignment: x=
// ===========================================================================

#[test]
fn test_parse_assignment_empty() {
    let stmt = parse_first("x=");
    match stmt {
        BashStmt::Assignment {
            name, value, index, ..
        } => {
            assert_eq!(name, "x");
            assert!(index.is_none());
            assert_eq!(value, BashExpr::Literal(String::new()));
        }
        other => panic!("expected Assignment with empty value, got {other:?}"),
    }
}

#[test]
fn test_parse_assignment_empty_followed_by_newline() {
    // `x=` followed by another statement on the next line
    let mut parser = BashParser::new("x=\necho hello").unwrap();
    let ast = parser.parse().unwrap();
    assert!(ast.statements.len() >= 2, "expected at least 2 statements");

    match &ast.statements[0] {
        BashStmt::Assignment { name, value, .. } => {
            assert_eq!(name, "x");
            assert_eq!(*value, BashExpr::Literal(String::new()));
        }
        other => panic!("expected empty Assignment, got {other:?}"),
    }
}

// ===========================================================================
// Exported assignment: export x=5
// ===========================================================================

#[test]
fn test_parse_assignment_exported() {
    let stmt = parse_first("export x=5");
    match stmt {
        BashStmt::Assignment {
            name,
            value,
            exported,
            ..
        } => {
            assert_eq!(name, "x");
            assert_eq!(value, BashExpr::Literal("5".to_string()));
            assert!(exported, "expected exported=true");
        }
        other => panic!("expected exported Assignment, got {other:?}"),
    }
}

#[test]
fn test_parse_assignment_exported_string() {
    let stmt = parse_first(r#"export PATH="/usr/bin""#);
    match stmt {
        BashStmt::Assignment {
            name,
            value,
            exported,
            ..
        } => {
            assert_eq!(name, "PATH");
            assert_eq!(value, BashExpr::Literal("/usr/bin".to_string()));
            assert!(exported);
        }
        other => panic!("expected exported Assignment, got {other:?}"),
    }
}

#[test]
fn test_parse_assignment_exported_empty() {
    let stmt = parse_first("export VAR=");
    match stmt {
        BashStmt::Assignment {
            name,
            value,
            exported,
            ..
        } => {
            assert_eq!(name, "VAR");
            assert_eq!(value, BashExpr::Literal(String::new()));
            assert!(exported);
        }
        other => panic!("expected exported empty Assignment, got {other:?}"),
    }
}

// ===========================================================================
// Multiple assignments in sequence
// ===========================================================================

#[test]
fn test_parse_multiple_assignments() {
    let mut parser = BashParser::new("a=1\nb=2\nc=3").unwrap();
    let ast = parser.parse().unwrap();
    assert_eq!(ast.statements.len(), 3);

    for (i, expected_name) in ["a", "b", "c"].iter().enumerate() {
        match &ast.statements[i] {
            BashStmt::Assignment { name, .. } => {
                assert_eq!(name, *expected_name);
            }
            other => panic!("expected Assignment at index {i}, got {other:?}"),
        }
    }
}

// ===========================================================================
// Variable reference as value
// ===========================================================================

#[test]
fn test_parse_assignment_variable_value() {
    let stmt = parse_first("x=$HOME");
    match stmt {
        BashStmt::Assignment { name, value, .. } => {
            assert_eq!(name, "x");
            match &value {
                BashExpr::Variable(v) => assert_eq!(v, "HOME"),
                BashExpr::Concat(parts) => {
                    // Some parsers wrap in Concat; accept either
                    assert!(!parts.is_empty());
                }
                other => panic!("expected Variable or Concat value, got {other:?}"),
            }
        }
        other => panic!("expected Assignment, got {other:?}"),
    }
}

// ===========================================================================
// Not-exported flag is false for regular assignments
// ===========================================================================

#[test]
fn test_parse_assignment_not_exported() {
    let stmt = parse_first("y=42");
    match stmt {
        BashStmt::Assignment { exported, .. } => {
            assert!(!exported, "regular assignment should have exported=false");
        }
        other => panic!("expected Assignment, got {other:?}"),
    }
}

// ===========================================================================
// Error cases
// ===========================================================================

#[test]
fn test_parse_assignment_unterminated_string() {
    // Unterminated string should be a lexer error
    parse_err(r#"x="unclosed"#);
}

#[test]
fn test_parse_assignment_unterminated_string_single_quote() {
    // Unterminated single-quoted string should be a lexer error
    parse_err("x='unclosed");
}

#[test]
fn test_parse_assignment_missing_bracket_not_array() {
    // Without closing bracket, the parser does NOT treat this as an array
    // assignment -- `arr` becomes a command and `[0=val` becomes arguments.
    // Verify it does not produce an Assignment with an index.
    let stmt = parse_first("arr[0=val");
    match &stmt {
        BashStmt::Assignment { index, .. } => {
            // If it somehow parses as assignment, index should be None
            // (the `[` would not have been consumed as array syntax)
            assert!(index.is_none(), "should not have an array index");
        }
        // More likely parsed as a Command
        BashStmt::Command { .. } => {
            // This is acceptable -- the parser saw no `=` after identifier
        }
        _ => {
            // Any other parse is fine as long as it's not an array assignment
        }
    }
}
