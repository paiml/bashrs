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
