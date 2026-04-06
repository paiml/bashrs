#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::bash_parser::parser_arith::ArithToken;
    #[test]
    fn test_LOCAL_FLAG_002_local_dash_r() {
        let input = "local -r FOO=\"bar\"";
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser.parse().expect("should parse local -r");
        assert!(!ast.statements.is_empty());
    }

    #[test]
    fn test_VARCMD_001_variable_as_command() {
        let input = r#"$CMD foo bar"#;
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser.parse().expect("should parse $VAR as command");
        match &ast.statements[0] {
            BashStmt::Command { name, args, .. } => {
                assert_eq!(name, "$CMD");
                assert_eq!(args.len(), 2);
            }
            other => panic!("Expected Command, got {other:?}"),
        }
    }

    #[test]
    fn test_VARCMD_002_variable_command_in_function() {
        let input = r#"deploy() {
    $KUBECTL scale deployment/foo --replicas=3
}"#;
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser
            .parse()
            .expect("should parse $VAR command in function");
        match &ast.statements[0] {
            BashStmt::Function { body, .. } => match &body[0] {
                BashStmt::Command { name, .. } => {
                    assert_eq!(name, "$KUBECTL");
                }
                other => panic!("Expected Command in function body, got {other:?}"),
            },
            other => panic!("Expected Function, got {other:?}"),
        }
    }

    #[test]
    fn test_ENVPREFIX_001_ifs_read_while_condition() {
        // IFS= read -r line is a common pattern: env prefix before command in while condition
        let input = "while IFS= read -r line; do\n    echo \"$line\"\ndone";
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser
            .parse()
            .expect("should parse IFS= read in while condition");
        match &ast.statements[0] {
            BashStmt::While {
                condition, body, ..
            } => {
                // Condition should be a CommandCondition with "IFS= read" as name
                match condition {
                    BashExpr::CommandCondition(stmt) => match stmt.as_ref() {
                        BashStmt::Command { name, args, .. } => {
                            assert_eq!(name, "IFS= read");
                            assert!(args
                                .iter()
                                .any(|a| matches!(a, BashExpr::Literal(s) if s == "-r")));
                        }
                        other => panic!("Expected Command in condition, got {other:?}"),
                    },
                    other => panic!("Expected CommandCondition, got {other:?}"),
                }
                assert!(!body.is_empty());
            }
            other => panic!("Expected While, got {other:?}"),
        }
    }

    #[test]
    fn test_ENVPREFIX_002_lc_all_sort_condition() {
        // LC_ALL=C sort is another common env prefix pattern
        let input = "while LC_ALL=C read -r line; do\n    echo \"$line\"\ndone";
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser.parse().expect("should parse LC_ALL=C read in while");
        match &ast.statements[0] {
            BashStmt::While { condition, .. } => match condition {
                BashExpr::CommandCondition(stmt) => match stmt.as_ref() {
                    BashStmt::Command { name, .. } => {
                        assert!(name.starts_with("LC_ALL=C"));
                    }
                    other => panic!("Expected Command, got {other:?}"),
                },
                other => panic!("Expected CommandCondition, got {other:?}"),
            },
            other => panic!("Expected While, got {other:?}"),
        }
    }

    #[test]
    fn test_ENVPREFIX_003_while_with_process_substitution() {
        // `done < <(cmd)` — process substitution redirect on while loop
        let input = "while IFS= read -r line; do\n    echo \"$line\"\ndone < <(echo test)";
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser
            .parse()
            .expect("should parse while with process substitution redirect");
        assert!(matches!(&ast.statements[0], BashStmt::While { .. }));
    }

    #[test]
    fn test_ENVPREFIX_004_multiple_functions_with_ifs_read() {
        // Regression: multiple functions + IFS= read crashed parser
        let input = r#"func_a() {
    if [ $? -eq 0 ]; then
        echo ok
    else
        echo fail
    fi
}

func_b() {
    while IFS= read -r db; do
        echo "$db"
    done
}"#;
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser
            .parse()
            .expect("should parse multiple functions with IFS= read");
        assert_eq!(ast.statements.len(), 2);
        assert!(matches!(&ast.statements[0], BashStmt::Function { name, .. } if name == "func_a"));
        assert!(matches!(&ast.statements[1], BashStmt::Function { name, .. } if name == "func_b"));
    }

    #[test]
    fn test_HEREDOC_001_heredoc_in_for_loop_body() {
        // BUG: heredoc inside for loop caused "expected 'done', found Eof"
        // because read_heredoc consumed the trailing newline, preventing the
        // parser from seeing the statement boundary before `done`
        let input = "for i in 1 2 3; do\n    cat <<EOF\nhello\nEOF\ndone";
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser.parse().expect("should parse heredoc in for loop");
        assert_eq!(ast.statements.len(), 1);
        assert!(matches!(&ast.statements[0], BashStmt::For { body, .. } if body.len() == 1));
    }

    #[test]
    fn test_HEREDOC_002_heredoc_in_while_loop_body() {
        let input = "while true; do\n    cat <<EOF\ndata\nEOF\ndone";
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser.parse().expect("should parse heredoc in while loop");
        assert_eq!(ast.statements.len(), 1);
        assert!(matches!(&ast.statements[0], BashStmt::While { .. }));
    }

    #[test]
    fn test_HEREDOC_003_heredoc_in_for_loop_in_function() {
        let input = "send_alert() {\n    for email in a b c; do\n        mail -s ALERT \"$email\" <<EOF\nalert content\nEOF\n    done\n}";
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser
            .parse()
            .expect("should parse heredoc in for loop in function");
        assert_eq!(ast.statements.len(), 1);
        assert!(matches!(&ast.statements[0], BashStmt::Function { .. }));
    }

    #[test]
    fn test_HEREDOC_004_heredoc_followed_by_more_statements() {
        // Verify heredoc doesn't eat following statements
        let input = "cat <<EOF\nhello\nEOF\necho after";
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser
            .parse()
            .expect("should parse heredoc + following statement");
        assert_eq!(ast.statements.len(), 2);
    }

    #[test]
    fn test_ASSIGN_COND_001_if_assignment_as_condition() {
        // `if pid=$(check_pid); then` — assignment exit status is the condition
        let input = "if pid=$(check_pid); then\n    echo \"$pid\"\nfi";
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser
            .parse()
            .expect("should parse assignment-as-condition");
        assert_eq!(ast.statements.len(), 1);
        assert!(matches!(&ast.statements[0], BashStmt::If { .. }));
    }

    #[test]
    fn test_ASSIGN_COND_002_negated_assignment_condition() {
        // `if ! pid=$(check_pid); then` — negated assignment condition
        let input = "if ! pid=$(check_pid); then\n    echo fail\nfi";
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser
            .parse()
            .expect("should parse negated assignment condition");
        assert_eq!(ast.statements.len(), 1);
    }

    #[test]
    fn test_CASE_PATTERN_001_dotted_patterns() {
        // Case patterns with dots: server.host), db.*)
        let input = r#"case "$key" in
    server.host) HOST="$value" ;;
    db.*) echo "db" ;;
    *) echo "other" ;;
esac"#;
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser.parse().expect("should parse dotted case patterns");
        assert!(matches!(&ast.statements[0], BashStmt::Case { arms, .. } if arms.len() == 3));
    }

    #[test]
    fn test_CASE_PATTERN_002_bracket_char_class() {
        // Case patterns with bracket char class: 5[0-9][0-9])
        let input = r#"case "$status" in
    200|201) echo ok ;;
    5[0-9][0-9]) echo error ;;
    *) echo unknown ;;
esac"#;
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser
            .parse()
            .expect("should parse bracket char class in case pattern");
        assert!(matches!(&ast.statements[0], BashStmt::Case { arms, .. } if arms.len() == 3));
    }

    #[test]
    fn test_CASE_PATTERN_003_pipe_alternatives() {
        // Case patterns with pipe alternatives including empty string
        let input = "case \"$key\" in\n    \\#*|\"\") continue ;;\n    *) echo other ;;\nesac";
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser
            .parse()
            .expect("should parse pipe alternative patterns");
        assert!(matches!(&ast.statements[0], BashStmt::Case { arms, .. } if arms.len() == 2));
    }

    #[test]
    fn test_GLOB_BRACKET_001_for_loop_glob() {
        // Glob bracket pattern in for loop items: [0-9]*.sql
        let input = "for f in /tmp/[0-9]*.sql; do\n    echo \"$f\"\ndone";
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser
            .parse()
            .expect("should parse glob bracket in for items");
        assert!(matches!(&ast.statements[0], BashStmt::For { .. }));
    }

    #[test]
    fn test_PIPE_COMPOUND_001_pipe_into_while() {
        // Pipeline with while loop on the right side: cmd | while read line; do ...; done
        let input = "find /etc -name \"*.conf\" | while read -r conf; do\n    echo \"$conf\"\ndone";
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser.parse().expect("should parse pipe into while");
        assert_eq!(ast.statements.len(), 1);
        assert!(
            matches!(&ast.statements[0], BashStmt::Pipeline { commands, .. } if commands.len() == 2)
        );
    }

    #[test]
    fn test_PIPE_COMPOUND_002_pipe_into_if() {
        // Pipeline with if on the right side: cmd | if ...; then ...; fi
        let input = "cat file | if read line; then echo \"$line\"; fi";
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser.parse().expect("should parse pipe into if");
        assert!(matches!(&ast.statements[0], BashStmt::Pipeline { .. }));
    }

    #[test]
    fn test_PIPE_COMPOUND_003_pipe_into_brace_group() {
        // Pipeline with brace group: cmd | { cmd1; cmd2; }
        let input = "echo hello | { read line; echo \"$line\"; }";
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser.parse().expect("should parse pipe into brace group");
        assert!(matches!(&ast.statements[0], BashStmt::Pipeline { .. }));
    }

    #[test]
    fn test_COND_REDIRECT_001_if_test_with_stderr_redirect() {
        // if [ condition ] 2>/dev/null; then
        let input = "if [ \"$x\" -ge 10 ] 2>/dev/null; then\n    echo yes\nfi";
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser
            .parse()
            .expect("should parse test with stderr redirect");
        assert!(matches!(&ast.statements[0], BashStmt::If { .. }));
    }

    #[test]
    fn test_COND_REDIRECT_002_while_test_with_redirect() {
        // while [ condition ] 2>/dev/null; do
        let input = "while [ -f /tmp/lock ] 2>/dev/null; do\n    sleep 1\ndone";
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser
            .parse()
            .expect("should parse while test with redirect");
        assert!(matches!(&ast.statements[0], BashStmt::While { .. }));
    }

    #[test]
    fn test_COMPOUND_REDIRECT_001_brace_group_with_redirects() {
        // { cmd; } > out 2> err
        let input = "{\n    echo stdout\n    echo stderr >&2\n} > /tmp/out.log 2> /tmp/err.log";
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser
            .parse()
            .expect("should parse brace group with redirects");
        assert!(matches!(&ast.statements[0], BashStmt::BraceGroup { .. }));
    }

    #[test]
    fn test_COMPOUND_REDIRECT_002_subshell_with_redirects() {
        // ( cmd ) > out
        let input = "(\n    echo hello\n) > /tmp/out.log";
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser
            .parse()
            .expect("should parse subshell with redirects");
        assert!(matches!(
            &ast.statements[0],
            BashStmt::BraceGroup { subshell: true, .. }
        ));
    }

    #[test]
    fn test_BACKGROUND_001_subshell_with_ampersand() {
        // ( cmd ) & — background subshell
        let input = "for i in 1 2 3; do\n    (\n        echo \"$i\"\n    ) &\ndone";
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser
            .parse()
            .expect("should parse background subshell in loop");
        assert!(matches!(&ast.statements[0], BashStmt::For { .. }));
    }

    #[test]
    fn test_BACKGROUND_002_command_with_ampersand() {
        // cmd & — background command
        let input = "sleep 10 &\necho running";
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser.parse().expect("should parse background command");
        assert_eq!(ast.statements.len(), 2);
    }

    #[test]
    fn test_ARITH_BASE_001_hex_base_notation() {
        // $((16#FF)) — hex base notation
        let input = "hex_val=$((16#FF))";
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser.parse().expect("should parse hex base notation");
        assert!(matches!(&ast.statements[0], BashStmt::Assignment { .. }));
    }

    #[test]
    fn test_ARITH_BASE_002_octal_base_notation() {
        // $((8#77)) — octal base notation
        let input = "oct_val=$((8#77))";
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser.parse().expect("should parse octal base notation");
        assert!(matches!(&ast.statements[0], BashStmt::Assignment { .. }));
    }

    // --- Subshell as if-condition tests ---

    #[test]
    fn test_SUBSHELL_COND_001_simple_subshell_condition() {
        let input = "if ( true ); then\n    echo ok\nfi";
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser.parse();
        assert!(
            ast.is_ok(),
            "Subshell as if-condition should parse: {:?}",
            ast.err()
        );
    }

    #[test]
    fn test_SUBSHELL_COND_002_subshell_with_semicolons() {
        let input = "if ( set -o noclobber; echo hi ); then\n    echo ok\nfi";
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser.parse();
        assert!(
            ast.is_ok(),
            "Subshell with ; in if-condition should parse: {:?}",
            ast.err()
        );
    }

    #[test]
    fn test_SUBSHELL_COND_003_subshell_with_redirect() {
        let input = "if ( echo test ) 2>/dev/null; then\n    echo ok\nfi";
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser.parse();
        assert!(
            ast.is_ok(),
            "Subshell condition with redirect should parse: {:?}",
            ast.err()
        );
    }

    // --- (( expr )) && / || tests ---

    #[test]
    fn test_ARITH_CMD_001_standalone_arith_and() {
        let input = "(( x > 10 )) && echo big";
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser.parse();
        assert!(ast.is_ok(), "(( )) && cmd should parse: {:?}", ast.err());
    }

    #[test]
    fn test_ARITH_CMD_002_standalone_arith_or() {
        let input = "(( y < 5 )) || echo default";
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser.parse();
        assert!(ast.is_ok(), "(( )) || cmd should parse: {:?}", ast.err());
    }

    // --- =~ regex match tests ---

    #[test]
    fn test_REGEX_MATCH_001_simple_regex() {
        let input = "if [[ \"hello\" =~ ^hel ]]; then\n    echo match\nfi";
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser.parse();
        assert!(ast.is_ok(), "=~ regex should parse: {:?}", ast.err());
    }

    #[test]
    fn test_REGEX_MATCH_002_complex_regex() {
        let input = "if [[ \"$v\" =~ ^[0-9]+$ ]]; then\n    echo num\nfi";
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser.parse();
        assert!(
            ast.is_ok(),
            "Complex =~ regex should parse: {:?}",
            ast.err()
        );
    }

    // --- POSIX char class in case tests ---

    #[test]
    fn test_POSIX_CLASS_001_space_class_in_case() {
        let input = "case \"$ch\" in\n    [[:space:]])\n        echo ws\n        ;;\nesac";
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser.parse();
        assert!(
            ast.is_ok(),
            "[[:space:]] in case should parse: {:?}",
            ast.err()
        );
    }

    #[test]
    fn test_POSIX_CLASS_002_alpha_class_in_case() {
        let input = "case \"$ch\" in\n    [[:alpha:]])\n        echo letter\n        ;;\nesac";
        let mut parser = BashParser::new(input).expect("parser");
        let ast = parser.parse();
        assert!(
            ast.is_ok(),
            "[[:alpha:]] in case should parse: {:?}",
            ast.err()
        );
    }

    // --- Extended glob in paths tests ---
}
