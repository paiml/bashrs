fn assert_first_arm_is_wildcard(source: &str) {
    let ast = parse(source).unwrap();
    let main_func = &ast.functions[0];
    match &main_func.body[0] {
        crate::ast::Stmt::Match { arms, .. } => {
            assert!(!arms.is_empty(), "Should have at least one match arm");
            match &arms[0].pattern {
                Pattern::Wildcard => {}
                _ => panic!("Expected Wildcard pattern for _, got {:?}", arms[0].pattern),
            }
        }
        _ => panic!("Expected match statement"),
    }
}

fn assert_first_arm_is_variable(source: &str, expected_name: &str) {
    let ast = parse(source).unwrap();
    let main_func = &ast.functions[0];
    match &main_func.body[0] {
        crate::ast::Stmt::Match { arms, .. } => {
            assert!(!arms.is_empty(), "Should have at least one match arm");
            match &arms[0].pattern {
                Pattern::Variable(name) => {
                    assert_eq!(
                        name, expected_name,
                        "Expected variable pattern '{expected_name}'"
                    );
                }
                Pattern::Wildcard => {
                    panic!("Named identifier '{expected_name}' should NOT be treated as Wildcard");
                }
                _ => panic!(
                    "Expected Variable pattern for {expected_name}, got {:?}",
                    arms[0].pattern
                ),
            }
        }
        _ => panic!("Expected match statement"),
    }
}

#[test]
fn test_pattern_wildcard_vs_identifier() {
    // RED: Targets mutation at line 567: replace == with !=
    // Tests the condition: if name == "_" for wildcard detection
    // This must distinguish between "_" (wildcard) and named identifiers

    // Test wildcard pattern (_)
    let source_wildcard = r#"
        fn main() {
            match value {
                _ => {
                    let default = true;
                }
            }
        }
    "#;
    assert_first_arm_is_wildcard(source_wildcard);

    // Test named identifier pattern (not wildcard)
    let source_ident = r#"
        fn main() {
            match value {
                x => {
                    let named = x;
                }
            }
        }
    "#;
    assert_first_arm_is_variable(source_ident, "x");
}

#[test]
fn test_pattern_ident_arm_execution() {
    // RED: Targets mutation at line 564: delete match arm Pat::Ident(ident_pat)
    // This ensures the Pat::Ident branch in convert_pattern is exercised

    let source = r#"
        fn main() {
            match status {
                0 => {
                    let success = true;
                }
                code => {
                    let error_code = code;
                }
                _ => {
                    let unknown = true;
                }
            }
        }
    "#;

    let ast = parse(source).unwrap();
    let main_func = &ast.functions[0];

    match &main_func.body[0] {
        crate::ast::Stmt::Match { arms, .. } => {
            assert_eq!(arms.len(), 3, "Should have 3 match arms");

            // First arm: literal pattern (0)
            match &arms[0].pattern {
                Pattern::Literal(Literal::U32(0)) => {
                    // Correct
                }
                _ => panic!("Expected literal pattern 0"),
            }

            // Second arm: identifier pattern (code) - this tests Pat::Ident!
            match &arms[1].pattern {
                Pattern::Variable(name) => {
                    assert_eq!(name, "code", "Expected variable pattern 'code'");
                }
                _ => panic!(
                    "Expected identifier pattern 'code', got {:?}",
                    arms[1].pattern
                ),
            }

            // Third arm: wildcard pattern (_)
            match &arms[2].pattern {
                Pattern::Wildcard => {
                    // Correct
                }
                _ => panic!("Expected wildcard pattern"),
            }
        }
        _ => panic!("Expected match statement"),
    }
}

#[test]
fn test_comprehensive_pattern_matching() {
    // RED: Comprehensive test covering all pattern types
    // Ensures complete coverage of convert_pattern function

    let source = r#"
        fn main() {
            match value {
                42 => { let num = true; }
                "test" => { let str = true; }
                true => { let bool = true; }
                x => { let var = x; }
                _ => { let wild = true; }
            }
        }
    "#;

    let ast = parse(source).unwrap();
    let main_func = &ast.functions[0];

    match &main_func.body[0] {
        crate::ast::Stmt::Match { arms, .. } => {
            assert_eq!(arms.len(), 5, "Should have 5 match arms");

            // Numeric literal
            assert!(
                matches!(&arms[0].pattern, Pattern::Literal(Literal::U32(42))),
                "Expected numeric literal 42"
            );

            // String literal
            assert!(
                matches!(&arms[1].pattern, Pattern::Literal(Literal::Str(_))),
                "Expected string literal"
            );

            // Boolean literal
            assert!(
                matches!(&arms[2].pattern, Pattern::Literal(Literal::Bool(true))),
                "Expected boolean literal true"
            );

            // Variable pattern (identifier)
            match &arms[3].pattern {
                Pattern::Variable(name) => {
                    assert_eq!(name, "x", "Expected variable 'x'");
                }
                _ => panic!("Expected variable pattern"),
            }

            // Wildcard pattern
            assert!(
                matches!(&arms[4].pattern, Pattern::Wildcard),
                "Expected wildcard pattern"
            );
        }
        _ => panic!("Expected match statement"),
    }
}
