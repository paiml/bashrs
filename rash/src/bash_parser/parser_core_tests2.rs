//! Parser core tests part 2 — redirects, heredocs, dogfood scripts, edge cases.
#![allow(clippy::unwrap_used)]

use crate::bash_parser::ast::*;
use crate::bash_parser::parser::*;

#[test]
fn test_parse_if_no_else() {
    let input = "if [ $x -eq 1 ]; then echo one; fi";
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    match &ast.statements[0] {
        BashStmt::If { else_block, .. } => {
            assert!(else_block.is_none());
        }
        _ => panic!("Expected If statement"),
    }
}

// ============================================================================
// Coverage Tests - Complex Expressions
// ============================================================================

#[test]
fn test_parse_variable_in_double_quotes() {
    let input = r#"echo "Hello $name""#;
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    assert!(matches!(&ast.statements[0], BashStmt::Command { .. }));
}

#[test]
fn test_parse_command_with_args() {
    // Simple command with multiple arguments (no flags with dashes)
    let input = "echo hello world";
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    match &ast.statements[0] {
        BashStmt::Command { name, args, .. } => {
            assert_eq!(name, "echo");
            assert_eq!(args.len(), 2);
        }
        _ => panic!("Expected Command"),
    }
}

#[test]
fn test_parse_command_with_path() {
    let input = "ls /tmp";
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    match &ast.statements[0] {
        BashStmt::Command { name, args, .. } => {
            assert_eq!(name, "ls");
            assert_eq!(args.len(), 1);
        }
        _ => panic!("Expected Command"),
    }
}

// ============================================================================
// Additional Coverage Tests - Unique Edge Cases
// ============================================================================

#[test]
fn test_coverage_empty_input() {
    let mut parser = BashParser::new("").unwrap();
    let ast = parser.parse().unwrap();
    assert!(ast.statements.is_empty());
}

#[test]
fn test_coverage_whitespace_only() {
    let mut parser = BashParser::new("   \n\t  \n").unwrap();
    let ast = parser.parse().unwrap();
    assert!(ast.statements.is_empty());
}

#[test]
fn test_coverage_comments_only() {
    let mut parser = BashParser::new("# comment\n# another").unwrap();
    let ast = parser.parse().unwrap();
    assert!(ast
        .statements
        .iter()
        .all(|s| matches!(s, BashStmt::Comment { .. })));
}

#[test]
fn test_coverage_multiline_string() {
    let input = r#"echo "line1
line2
line3""#;
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    assert!(!ast.statements.is_empty());
}

#[test]
fn test_coverage_escaped_quotes() {
    let input = r#"echo "hello \"world\"""#;
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    assert!(!ast.statements.is_empty());
}

#[test]
fn test_coverage_single_quoted_string() {
    let input = "echo 'hello $world'";
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    assert!(!ast.statements.is_empty());
}

#[test]
fn test_coverage_heredoc_simple() {
    let input = r#"cat <<EOF
hello world
EOF"#;
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    assert!(!ast.statements.is_empty());
}

#[test]
fn test_coverage_heredoc_quoted_delimiter() {
    let input = r#"cat <<'EOF'
hello $world
EOF"#;
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    assert!(!ast.statements.is_empty());
}

#[test]
fn test_coverage_herestring() {
    let input = r#"cat <<< "hello world""#;
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    assert!(!ast.statements.is_empty());
}

#[test]
fn test_coverage_array_declaration() {
    let input = "arr=(one two three)";
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    assert!(!ast.statements.is_empty());
}

#[test]
fn test_coverage_array_access() {
    let input = "echo ${arr[0]}";
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    assert!(!ast.statements.is_empty());
}

#[test]
fn test_coverage_array_all_elements() {
    let input = "echo ${arr[@]}";
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    assert!(!ast.statements.is_empty());
}

#[test]
fn test_coverage_arithmetic_expansion() {
    let input = "echo $((1 + 2 * 3))";
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    assert!(!ast.statements.is_empty());
}

#[test]
fn test_coverage_complex_arithmetic() {
    let input = "result=$((a + b * c / d % e))";
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    assert!(!ast.statements.is_empty());
}

#[test]
fn test_coverage_parameter_default_value() {
    let input = "echo ${var:-default}";
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    assert!(!ast.statements.is_empty());
}

#[test]
fn test_coverage_parameter_assign_default() {
    let input = "echo ${var:=default}";
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    assert!(!ast.statements.is_empty());
}

#[test]
fn test_coverage_parameter_error_if_unset() {
    let input = "echo ${var:?error message}";
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    assert!(!ast.statements.is_empty());
}

#[test]
fn test_coverage_parameter_alternative_value() {
    let input = "echo ${var:+alternative}";
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    assert!(!ast.statements.is_empty());
}

#[test]
fn test_coverage_substring_extraction() {
    let input = "echo ${var:0:5}";
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    assert!(!ast.statements.is_empty());
}

#[test]
fn test_coverage_pattern_removal_prefix() {
    let input = "echo ${var#pattern}";
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    assert!(!ast.statements.is_empty());
}

#[test]
fn test_coverage_pattern_removal_suffix() {
    let input = "echo ${var%pattern}";
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    assert!(!ast.statements.is_empty());
}

#[test]
fn test_coverage_command_substitution_backticks_unsupported() {
    // Backticks are not supported by this parser - verify error handling
    let input = "echo `date`";
    let parser_result = BashParser::new(input);
    // Should fail at lexer stage with UnexpectedChar for backtick
    assert!(parser_result.is_err() || parser_result.unwrap().parse().is_err());
}

#[test]
fn test_coverage_command_substitution_dollar() {
    let input = "echo $(date)";
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    assert!(!ast.statements.is_empty());
}

#[test]
fn test_coverage_process_substitution_input() {
    let input = "diff <(sort file1) <(sort file2)";
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    assert!(!ast.statements.is_empty());
}

#[test]
fn test_coverage_pipeline_simple() {
    let input = "cat file | grep pattern";
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    assert!(ast
        .statements
        .iter()
        .any(|s| matches!(s, BashStmt::Pipeline { .. })));
}

#[test]
fn test_coverage_pipeline_long() {
    let input = "cat file | grep pattern | sort | uniq";
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    match &ast.statements[0] {
        BashStmt::Pipeline { commands, .. } => {
            assert_eq!(commands.len(), 4);
        }
        _ => panic!("Expected Pipeline"),
    }
}

#[test]
fn test_coverage_redirect_fd_duplicate() {
    let input = "cmd 2>&1";
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    assert!(!ast.statements.is_empty());
}

#[test]
fn test_coverage_background_job_supported() {
    // Background jobs with & are now supported as a statement terminator
    let input = "sleep 10 &";
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().expect("should parse background command");
    assert_eq!(ast.statements.len(), 1);
    assert!(matches!(&ast.statements[0], BashStmt::Command { name, .. } if name == "sleep"));
}

#[test]
fn test_coverage_mixed_and_or() {
    let input = "cmd1 && cmd2 || cmd3";
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    assert!(!ast.statements.is_empty());
}

#[test]
fn test_SUBSHELL_001_basic() {
    let input = "(cd /tmp && ls)";
    let mut parser = BashParser::new(input).expect("parser");
    let ast = parser.parse().expect("should parse subshell");
    match &ast.statements[0] {
        BashStmt::BraceGroup { subshell, body, .. } => {
            assert!(subshell, "should be marked as subshell");
            assert!(!body.is_empty(), "subshell should have body");
        }
        other => panic!("Expected BraceGroup(subshell), got {other:?}"),
    }
}

#[test]
fn test_SUBSHELL_002_simple_echo() {
    let input = "(echo hello)";
    let mut parser = BashParser::new(input).expect("parser");
    let ast = parser.parse().expect("should parse subshell");
    match &ast.statements[0] {
        BashStmt::BraceGroup { subshell, .. } => {
            assert!(subshell, "should be marked as subshell");
        }
        other => panic!("Expected BraceGroup(subshell), got {other:?}"),
    }
}

#[test]
fn test_LOCAL_FLAG_001_local_dash_i() {
    let input = r#"foo() {
local -i num=5
echo $num
}"#;
    let mut parser = BashParser::new(input).expect("parser");
    let ast = parser.parse().expect("should parse local -i");
    match &ast.statements[0] {
        BashStmt::Function { body, .. } => {
            // local -i num=5 should produce an assignment (flag skipped)
            assert!(
                body.len() >= 2,
                "function should have at least 2 statements: {:?}",
                body
            );
        }
        other => panic!("Expected Function, got {other:?}"),
    }
}

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


    include!("parser_core_tests2_incl2.rs");
