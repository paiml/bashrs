#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::bash_parser::parser_arith::ArithToken;
    #[test]
    fn test_EXT_GLOB_PATH_001_at_glob_in_for() {
        let input = "for f in /tmp/@(a|b|c).sh; do\n    echo \"$f\"\ndone";
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser.parse();
        assert!(ast.is_ok(), "@() in path should parse: {:?}", ast.err());
    }

    #[test]
    fn test_EXT_GLOB_PATH_002_plus_glob_in_path() {
        let input = "ls /tmp/file+(a|b).txt";
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser.parse();
        assert!(ast.is_ok(), "+() in path should parse: {:?}", ast.err());
    }

    #[test]
    fn test_EXT_GLOB_PATH_003_question_glob_in_path() {
        let input = "ls /tmp/?(opt).txt";
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser.parse();
        assert!(ast.is_ok(), "?() in path should parse: {:?}", ast.err());
    }

    #[test]
    fn test_coverage_case_statement() {
        let input = r#"case $var in
    a) echo "a";;
    b) echo "b";;
    *) echo "other";;
esac"#;
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        assert!(ast
            .statements
            .iter()
            .any(|s| matches!(s, BashStmt::Case { .. })));
    }

    #[test]
    fn test_coverage_select_statement() {
        let input = r#"select opt in "opt1" "opt2" "opt3"; do
    echo "Selected: $opt"
    break
done"#;
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        assert!(ast
            .statements
            .iter()
            .any(|s| matches!(s, BashStmt::Select { .. })));
    }

    #[test]
    fn test_coverage_until_loop() {
        let input = r#"until [ $count -ge 5 ]; do
    echo $count
    count=$((count + 1))
done"#;
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        assert!(ast
            .statements
            .iter()
            .any(|s| matches!(s, BashStmt::Until { .. })));
    }

    #[test]
    fn test_coverage_function_posix() {
        let input = r#"greet() {
    echo "Hello $1"
}"#;
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        assert!(ast
            .statements
            .iter()
            .any(|s| matches!(s, BashStmt::Function { .. })));
    }

    #[test]
    fn test_coverage_trap_command() {
        let input = "trap 'cleanup' EXIT";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        assert!(!ast.statements.is_empty());
    }

    #[test]
    fn test_coverage_return_statement() {
        let input = "return 0";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        assert!(ast
            .statements
            .iter()
            .any(|s| matches!(s, BashStmt::Return { .. })));
    }

    #[test]
    fn test_coverage_break_statement() {
        let input = "break";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        assert!(!ast.statements.is_empty());
    }

    #[test]
    fn test_coverage_continue_statement() {
        let input = "continue";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        assert!(!ast.statements.is_empty());
    }

    #[test]
    fn test_coverage_export_statement() {
        let input = "export VAR=value";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        assert!(!ast.statements.is_empty());
    }

    #[test]
    fn test_coverage_local_statement() {
        let input = "local var=value";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        assert!(!ast.statements.is_empty());
    }

    #[test]
    fn test_coverage_readonly_statement() {
        // readonly with name=value should parse as a command with literal arg
        let input = "readonly VAR=value";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        assert!(!ast.statements.is_empty());
    }

    #[test]
    fn test_KEYWORD_001_echo_done_parses() {
        let input = "echo done";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        assert_eq!(ast.statements.len(), 1);
        match &ast.statements[0] {
            BashStmt::Command { name, args, .. } => {
                assert_eq!(name, "echo");
                assert_eq!(args.len(), 1);
                assert!(matches!(&args[0], BashExpr::Literal(s) if s == "done"));
            }
            other => panic!("Expected Command, got {other:?}"),
        }
    }

    #[test]
    fn test_KEYWORD_002_echo_fi_then_else() {
        let input = "echo fi then else";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        assert_eq!(ast.statements.len(), 1);
        match &ast.statements[0] {
            BashStmt::Command { name, args, .. } => {
                assert_eq!(name, "echo");
                assert_eq!(args.len(), 3);
                assert!(matches!(&args[0], BashExpr::Literal(s) if s == "fi"));
                assert!(matches!(&args[1], BashExpr::Literal(s) if s == "then"));
                assert!(matches!(&args[2], BashExpr::Literal(s) if s == "else"));
            }
            other => panic!("Expected Command, got {other:?}"),
        }
    }

    #[test]
    fn test_KEYWORD_003_echo_done_in_for_loop() {
        // echo done inside a for loop — done as arg, then done terminates loop
        let input = "for i in 1 2; do\necho done\ndone";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        assert_eq!(ast.statements.len(), 1);
        match &ast.statements[0] {
            BashStmt::For { body, .. } => {
                assert_eq!(body.len(), 1);
                match &body[0] {
                    BashStmt::Command { name, args, .. } => {
                        assert_eq!(name, "echo");
                        assert_eq!(args.len(), 1);
                        assert!(matches!(&args[0], BashExpr::Literal(s) if s == "done"));
                    }
                    other => panic!("Expected Command in body, got {other:?}"),
                }
            }
            other => panic!("Expected For, got {other:?}"),
        }
    }

    #[test]
    fn test_KEYWORD_004_echo_all_keywords() {
        // All keyword tokens should be parseable as echo arguments
        let input = "echo if then elif else fi for while until do done case esac in";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        assert_eq!(ast.statements.len(), 1);
        match &ast.statements[0] {
            BashStmt::Command { name, args, .. } => {
                assert_eq!(name, "echo");
                let kws: Vec<&str> = args
                    .iter()
                    .map(|a| match a {
                        BashExpr::Literal(s) => s.as_str(),
                        _ => panic!("Expected Literal"),
                    })
                    .collect();
                assert_eq!(
                    kws,
                    vec![
                        "if", "then", "elif", "else", "fi", "for", "while", "until", "do", "done",
                        "case", "esac", "in"
                    ]
                );
            }
            other => panic!("Expected Command, got {other:?}"),
        }
    }

    #[test]
    fn test_KEYWORD_005_for_in_done_item() {
        // `done` as a for-in item
        let input = "for word in hello done world; do echo $word; done";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        assert_eq!(ast.statements.len(), 1);
        assert!(matches!(&ast.statements[0], BashStmt::For { .. }));
    }

    #[test]
    fn test_GLOB_001_unquoted_star_is_glob() {
        let input = "ls *.sh";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        match &ast.statements[0] {
            BashStmt::Command { args, .. } => {
                assert!(matches!(&args[0], BashExpr::Glob(p) if p == "*.sh"));
            }
            other => panic!("Expected Command, got {other:?}"),
        }
    }

    #[test]
    fn test_GLOB_002_path_glob_preserved() {
        let input = "cp dist/* /tmp/";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        match &ast.statements[0] {
            BashStmt::Command { args, .. } => {
                assert!(matches!(&args[0], BashExpr::Glob(p) if p == "dist/*"));
            }
            other => panic!("Expected Command, got {other:?}"),
        }
    }

    #[test]
    fn test_GLOB_003_absolute_path_glob() {
        let input = "rm -f /tmp/*.log";
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        match &ast.statements[0] {
            BashStmt::Command { args, .. } => {
                assert!(matches!(&args[1], BashExpr::Glob(p) if p == "/tmp/*.log"));
            }
            other => panic!("Expected Command, got {other:?}"),
        }
    }

    #[test]
    fn test_GLOB_004_quoted_star_not_glob() {
        // Quoted * should remain a Literal, not a Glob
        let input = r#"find . -name "*.txt""#;
        let mut parser = BashParser::new(input).unwrap();
        let ast = parser.parse().unwrap();
        match &ast.statements[0] {
            BashStmt::Command { args, .. } => {
                // The "*.txt" comes from Token::String, so it's a Literal
                assert!(matches!(&args[2], BashExpr::Literal(s) if s == "*.txt"));
            }
            other => panic!("Expected Command, got {other:?}"),
        }
    }

    #[test]
    fn test_NAMEVALUE_001_echo_name_equals_value() {
        let input = "echo name=myapp";
        let mut parser = BashParser::new(input).expect("parser should init");
        let ast = parser.parse().expect("should parse name=value in argument");
        match &ast.statements[0] {
            BashStmt::Command { name, args, .. } => {
                assert_eq!(name, "echo");
                assert_eq!(args.len(), 1);
                assert!(matches!(&args[0], BashExpr::Literal(s) if s == "name=myapp"));
            }
            other => panic!("Expected Command, got {other:?}"),
        }
    }

    #[test]
    fn test_NAMEVALUE_002_docker_filter() {
        let input = "docker ps --filter name=myapp";
        let mut parser = BashParser::new(input).expect("parser should init");
        let ast = parser
            .parse()
            .expect("should parse docker --filter name=value");
        match &ast.statements[0] {
            BashStmt::Command { name, args, .. } => {
                assert_eq!(name, "docker");
                assert!(args.len() >= 3); // ps, --filter, name=myapp
                                          // Find the name=myapp argument
                let has_namevalue = args
                    .iter()
                    .any(|a| matches!(a, BashExpr::Literal(s) if s == "name=myapp"));
                assert!(has_namevalue, "args should contain name=myapp: {args:?}");
            }
            other => panic!("Expected Command, got {other:?}"),
        }
    }

    #[test]
    fn test_NAMEVALUE_003_env_var_equals_val() {
        let input = "env LANG=C sort file.txt";
        let mut parser = BashParser::new(input).expect("parser should init");
        let ast = parser.parse().expect("should parse env VAR=value");
        match &ast.statements[0] {
            BashStmt::Command { name, args, .. } => {
                assert_eq!(name, "env");
                assert!(matches!(&args[0], BashExpr::Literal(s) if s == "LANG=C"));
            }
            other => panic!("Expected Command, got {other:?}"),
        }
    }

    #[test]
    fn test_NAMEVALUE_004_multiple_equals() {
        let input = "docker run -e DB_HOST=localhost -e DB_PORT=5432 myimage";
        let mut parser = BashParser::new(input).expect("parser should init");
        let ast = parser
            .parse()
            .expect("should parse multiple name=value args");
        match &ast.statements[0] {
            BashStmt::Command { name, args, .. } => {
                assert_eq!(name, "docker");
                let has_host = args
                    .iter()
                    .any(|a| matches!(a, BashExpr::Literal(s) if s == "DB_HOST=localhost"));
                let has_port = args
                    .iter()
                    .any(|a| matches!(a, BashExpr::Literal(s) if s == "DB_PORT=5432"));
                assert!(has_host, "should have DB_HOST=localhost: {args:?}");
                assert!(has_port, "should have DB_PORT=5432: {args:?}");
            }
            other => panic!("Expected Command, got {other:?}"),
        }
    }

    #[test]
    fn test_URL_001_http_url_single_token() {
        let input = "curl http://localhost:8080/health";
        let mut parser = BashParser::new(input).expect("parser should init");
        let ast = parser.parse().expect("should parse URL as single token");
        match &ast.statements[0] {
            BashStmt::Command { name, args, .. } => {
                assert_eq!(name, "curl");
                assert_eq!(args.len(), 1);
                assert!(
                    matches!(&args[0], BashExpr::Literal(s) if s == "http://localhost:8080/health"),
                    "URL should be single token: {args:?}"
                );
            }
            other => panic!("Expected Command, got {other:?}"),
        }
    }

    #[test]
    fn test_URL_002_port_mapping_single_token() {
        let input = "docker run -p 8080:8080 myimage";
        let mut parser = BashParser::new(input).expect("parser should init");
        let ast = parser
            .parse()
            .expect("should parse port mapping as single token");
        match &ast.statements[0] {
            BashStmt::Command { name, args, .. } => {
                assert_eq!(name, "docker");
                let has_port = args
                    .iter()
                    .any(|a| matches!(a, BashExpr::Literal(s) if s == "8080:8080"));
                assert!(has_port, "should have 8080:8080 as single token: {args:?}");
            }
            other => panic!("Expected Command, got {other:?}"),
        }
    }

    #[test]
    fn test_URL_003_https_url() {
        let input = "wget https://example.com/file.tar.gz";
        let mut parser = BashParser::new(input).expect("parser should init");
        let ast = parser.parse().expect("should parse HTTPS URL");
        match &ast.statements[0] {
            BashStmt::Command { name, args, .. } => {
                assert_eq!(name, "wget");
                assert_eq!(args.len(), 1);
                assert!(
                    matches!(&args[0], BashExpr::Literal(s) if s == "https://example.com/file.tar.gz"),
                    "HTTPS URL should be single token: {args:?}"
                );
            }
            other => panic!("Expected Command, got {other:?}"),
        }
    }

    #[test]
    fn test_COMPOUND_001_if_and_condition() {
        let input = r#"if [ "$X" = "a" ] && [ "$Y" -gt 0 ]; then
    echo yes
fi"#;
        let mut parser = BashParser::new(input).expect("parser should init");
        let ast = parser.parse().expect("should parse && in if condition");
        assert_eq!(ast.statements.len(), 1);
        match &ast.statements[0] {
            BashStmt::If {
                condition,
                then_block,
                ..
            } => {
                // Condition should be a compound test with And
                let cond_str = format!("{condition:?}");
                assert!(
                    cond_str.contains("And"),
                    "condition should contain And: {cond_str}"
                );
                assert!(!then_block.is_empty());
            }
            other => panic!("Expected If, got {other:?}"),
        }
    }

    #[test]
    fn test_COMPOUND_002_if_or_condition() {
        let input = r#"if [ -f /tmp/a ] || [ -f /tmp/b ]; then
    echo found
fi"#;
        let mut parser = BashParser::new(input).expect("parser should init");
        let ast = parser.parse().expect("should parse || in if condition");
        match &ast.statements[0] {
            BashStmt::If { condition, .. } => {
                let cond_str = format!("{condition:?}");
                assert!(
                    cond_str.contains("Or"),
                    "condition should contain Or: {cond_str}"
                );
            }
            other => panic!("Expected If, got {other:?}"),
        }
    }
}
