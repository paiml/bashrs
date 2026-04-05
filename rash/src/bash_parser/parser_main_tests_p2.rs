#[cfg(test)]
mod tests {
    use super::parser_arith::ArithToken;
    use super::*;
    #[test]
    fn test_FORCSTYLE_COV_014_identifier_and_number() {
        // Tests both Token::Identifier and Token::Number paths
        let (init, cond, incr) =
            parse_for_c_style_parts("for ((count=0; count<5; count++)); do echo $count; done");
        assert!(init.contains("count"));
        assert!(cond.contains("count"));
        assert!(incr.contains("count"));
    }

    #[test]
    fn test_FORCSTYLE_COV_015_empty_body() {
        // For loop with colon (no-op) body
        let input = "for ((i=0; i<3; i++)); do :; done";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        assert!(matches!(&ast.statements[0], BashStmt::ForCStyle { .. }));
    }

    // ============================================================================
    // Coverage Tests - Case Statement
    // ============================================================================

    #[test]
    fn test_parse_case_basic() {
        let input = r#"
case $x in
    a) echo a;;
    b) echo b;;
    *) echo default;;
esac
"#;
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        match &ast.statements[0] {
            BashStmt::Case { arms, .. } => {
                assert_eq!(arms.len(), 3);
            }
            _ => panic!("Expected Case statement"),
        }
    }

    #[test]
    fn test_parse_case_multiple_patterns() {
        let input = r#"
case $x in
    a|b|c) echo abc;;
esac
"#;
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        match &ast.statements[0] {
            BashStmt::Case { arms, .. } => {
                assert_eq!(arms[0].patterns.len(), 3);
            }
            _ => panic!("Expected Case statement"),
        }
    }

    // ============================================================================
    // Coverage Tests - Function Syntax
    // ============================================================================

    #[test]
    fn test_parse_function_shorthand() {
        let input = "greet() { echo hello; }";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        match &ast.statements[0] {
            BashStmt::Function { name, .. } => {
                assert_eq!(name, "greet");
            }
            _ => panic!("Expected Function statement"),
        }
    }

    #[test]
    fn test_parse_function_keyword() {
        let input = "function hello { echo hi; }";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        assert!(matches!(&ast.statements[0], BashStmt::Function { .. }));
    }

    // ============================================================================
    // Coverage Tests - Return and Export
    // ============================================================================

    #[test]
    fn test_parse_return_with_code() {
        let input = "return 0";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        match &ast.statements[0] {
            BashStmt::Return { code, .. } => {
                assert!(code.is_some());
            }
            _ => panic!("Expected Return statement"),
        }
    }

    #[test]
    fn test_parse_return_without_code() {
        let input = "return";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        match &ast.statements[0] {
            BashStmt::Return { code, .. } => {
                assert!(code.is_none());
            }
            _ => panic!("Expected Return statement"),
        }
    }

    #[test]
    fn test_parse_export_assignment() {
        let input = "export FOO=bar";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        match &ast.statements[0] {
            BashStmt::Assignment { exported, name, .. } => {
                assert!(*exported);
                assert_eq!(name, "FOO");
            }
            _ => panic!("Expected exported Assignment"),
        }
    }

    #[test]
    fn test_parse_local_assignment() {
        let input = "local myvar=value";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        assert!(matches!(&ast.statements[0], BashStmt::Assignment { .. }));
    }

    // ============================================================================
    // Coverage Tests - Brace Groups
    // ============================================================================

    #[test]
    fn test_parse_brace_group() {
        let input = "{ echo a; echo b; }";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        assert!(matches!(&ast.statements[0], BashStmt::BraceGroup { .. }));
    }

    // ============================================================================
    // Coverage Tests - Redirects
    // ============================================================================

    #[test]
    fn test_parse_redirect_output() {
        let input = "echo hello > file.txt";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        match &ast.statements[0] {
            BashStmt::Command { redirects, .. } => {
                assert!(!redirects.is_empty());
            }
            _ => panic!("Expected Command with redirects"),
        }
    }

    #[test]
    fn test_parse_redirect_append() {
        let input = "echo hello >> file.txt";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        match &ast.statements[0] {
            BashStmt::Command { redirects, .. } => {
                assert!(matches!(&redirects[0], Redirect::Append { .. }));
            }
            _ => panic!("Expected Command with append redirect"),
        }
    }

    #[test]
    fn test_parse_redirect_input() {
        let input = "cat < input.txt";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        match &ast.statements[0] {
            BashStmt::Command { redirects, .. } => {
                assert!(matches!(&redirects[0], Redirect::Input { .. }));
            }
            _ => panic!("Expected Command with input redirect"),
        }
    }

    #[test]
    fn test_parse_redirect_stderr() {
        let input = "cmd 2> error.log";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        match &ast.statements[0] {
            BashStmt::Command { redirects, .. } => {
                assert!(matches!(&redirects[0], Redirect::Error { .. }));
            }
            _ => panic!("Expected Command with stderr redirect"),
        }
    }

    #[test]
    fn test_parse_redirect_combined() {
        let input = "cmd &> all.log";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        match &ast.statements[0] {
            BashStmt::Command { redirects, .. } => {
                assert!(!redirects.is_empty());
            }
            _ => panic!("Expected Command with combined redirect"),
        }
    }

    // ============================================================================
    // Coverage Tests - Pipelines and Lists
    // ============================================================================

    #[test]
    fn test_parse_pipeline() {
        let input = "ls | grep foo | sort";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        assert!(matches!(&ast.statements[0], BashStmt::Pipeline { .. }));
    }

    #[test]
    fn test_parse_and_list() {
        let input = "mkdir dir && cd dir";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        assert!(matches!(&ast.statements[0], BashStmt::AndList { .. }));
    }

    #[test]
    fn test_parse_or_list() {
        let input = "test -f file || echo missing";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        assert!(matches!(&ast.statements[0], BashStmt::OrList { .. }));
    }

    // ============================================================================
    // Coverage Tests - Test Conditions
    // ============================================================================

    #[test]
    fn test_parse_test_string_eq() {
        let input = r#"[ "$x" = "foo" ]"#;
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        assert!(!ast.statements.is_empty());
    }

    #[test]
    fn test_parse_test_string_ne() {
        let input = r#"[ "$x" != "bar" ]"#;
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        assert!(!ast.statements.is_empty());
    }

    #[test]
    fn test_parse_test_int_eq() {
        let input = "[ $x -eq 5 ]";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        assert!(!ast.statements.is_empty());
    }

    #[test]
    fn test_parse_test_int_ne() {
        let input = "[ $x -ne 0 ]";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        assert!(!ast.statements.is_empty());
    }

    #[test]
    fn test_parse_test_int_lt() {
        let input = "[ $x -lt 10 ]";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        assert!(!ast.statements.is_empty());
    }

    #[test]
    fn test_parse_test_int_le() {
        let input = "[ $x -le 100 ]";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        assert!(!ast.statements.is_empty());
    }

    #[test]
    fn test_parse_test_int_gt() {
        let input = "[ $x -gt 0 ]";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        assert!(!ast.statements.is_empty());
    }

    #[test]
    fn test_parse_test_int_ge() {
        let input = "[ $x -ge 1 ]";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        assert!(!ast.statements.is_empty());
    }

    #[test]
    fn test_parse_test_file_exists() {
        let input = "[ -e /tmp/file ]";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        assert!(!ast.statements.is_empty());
    }

    #[test]
    fn test_parse_test_file_readable() {
        let input = "[ -r /tmp/file ]";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        assert!(!ast.statements.is_empty());
    }

    #[test]
    fn test_parse_test_file_writable() {
        let input = "[ -w /tmp/file ]";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        assert!(!ast.statements.is_empty());
    }

    #[test]
    fn test_parse_test_file_executable() {
        let input = "[ -x /bin/sh ]";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        assert!(!ast.statements.is_empty());
    }

    #[test]
    fn test_parse_test_file_directory() {
        let input = "[ -d /tmp ]";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        assert!(!ast.statements.is_empty());
    }

    #[test]
    fn test_parse_test_string_empty() {
        let input = "[ -z \"$x\" ]";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        assert!(!ast.statements.is_empty());
    }

    #[test]
    fn test_parse_test_string_non_empty() {
        let input = "[ -n \"$x\" ]";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        assert!(!ast.statements.is_empty());
    }

    // ============================================================================
    // Coverage Tests - Extended Test [[ ]]
    // ============================================================================

    #[test]
    fn test_parse_extended_test() {
        let input = "[[ $x == pattern ]]";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        assert!(!ast.statements.is_empty());
    }

    // ============================================================================
    // Coverage Tests - Parameter Expansion
    // ============================================================================

    #[test]
    fn test_parse_default_value() {
        let input = "echo ${x:-default}";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        match &ast.statements[0] {
            BashStmt::Command { args, .. } => {
                assert!(matches!(&args[0], BashExpr::DefaultValue { .. }));
            }
            _ => panic!("Expected Command with DefaultValue"),
        }
    }

    #[test]
    fn test_parse_assign_default() {
        let input = "echo ${x:=default}";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        match &ast.statements[0] {
            BashStmt::Command { args, .. } => {
                assert!(matches!(&args[0], BashExpr::AssignDefault { .. }));
            }
            _ => panic!("Expected Command with AssignDefault"),
        }
    }

    #[test]
    fn test_parse_alternative_value() {
        let input = "echo ${x:+alternative}";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        match &ast.statements[0] {
            BashStmt::Command { args, .. } => {
                assert!(matches!(&args[0], BashExpr::AlternativeValue { .. }));
            }
            _ => panic!("Expected Command with AlternativeValue"),
        }
    }

    #[test]
    fn test_parse_error_if_unset() {
        let input = "echo ${x:?error message}";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        match &ast.statements[0] {
            BashStmt::Command { args, .. } => {
                assert!(matches!(&args[0], BashExpr::ErrorIfUnset { .. }));
            }
            _ => panic!("Expected Command with ErrorIfUnset"),
        }
    }

    #[test]
    fn test_parse_string_length() {
        let input = "echo ${#x}";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        match &ast.statements[0] {
            BashStmt::Command { args, .. } => {
                assert!(matches!(&args[0], BashExpr::StringLength { .. }));
            }
            _ => panic!("Expected Command with StringLength"),
        }
    }

    #[test]
    fn test_parse_remove_prefix() {
        let input = "echo ${x#pattern}";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        match &ast.statements[0] {
            BashStmt::Command { args, .. } => {
                assert!(matches!(&args[0], BashExpr::RemovePrefix { .. }));
            }
            _ => panic!("Expected Command with RemovePrefix"),
        }
    }
}
