fn test_format_parse_diagnostic_unexpected_token() {
    use super::parser::format_parse_diagnostic;
    let source = "if true; echo hi; fi";
    let err = ParseError::UnexpectedToken {
        expected: "'then' keyword".to_string(),
        found: "'echo'".to_string(),
        line: 1,
    };
    let diag = format_parse_diagnostic(&err, source, Some("test.sh"));
    assert!(diag.error.contains("expected") || diag.error.contains("then"));
}

/// format_parse_diagnostic with UnexpectedEof
#[test]
fn test_format_parse_diagnostic_unexpected_eof() {
    use super::parser::format_parse_diagnostic;
    let source = "if true; then\n  echo hi\n";
    let err = ParseError::UnexpectedEof;
    let diag = format_parse_diagnostic(&err, source, None);
    assert!(diag.error.contains("end of file") || diag.error.contains("unexpected"));
}

/// format_parse_diagnostic with InvalidSyntax
#[test]
fn test_format_parse_diagnostic_invalid_syntax() {
    use super::parser::format_parse_diagnostic;
    let source = "echo test";
    let err = ParseError::InvalidSyntax("something went wrong".to_string());
    let diag = format_parse_diagnostic(&err, source, None);
    assert!(diag.error.contains("something went wrong"));
}

/// build_snippet utility — exercises multi-line snippet rendering
#[test]
fn test_build_snippet_line_context() {
    use super::parser::build_snippet;
    let source = "line1\nline2\nline3\nline4";
    let snippet = build_snippet(source, 2, Some(3), 4);
    assert!(snippet.contains("line2"));
}

#[test]
fn test_build_snippet_first_line() {
    use super::parser::build_snippet;
    let source = "only_one_line";
    let snippet = build_snippet(source, 1, None, 1);
    assert!(snippet.contains("only_one_line"));
}

/// `skip_newlines` coverage — exercised inside parse loops, but test directly through
/// a multi-newline script to ensure the branch is hit
#[test]
fn test_skip_newlines_between_statements() {
    let input = "\n\n\necho hello\n\n\necho world\n\n";
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    assert_eq!(ast.statements.len(), 2);
}

/// `syntax_error` path — InvalidSyntax produced internally
#[test]
fn test_syntax_error_in_pipeline_without_rhs() {
    // Pipe at end of input with nothing on RHS
    let input = "echo hello |";
    let result = BashParser::new(input).and_then(|mut p| p.parse());
    // May error or succeed depending on parser; just ensure no panic
    let _ = result;
}

/// `check` / `is_at_end` exercised via empty input
#[test]
fn test_parse_empty_input() {
    let input = "";
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    assert!(ast.statements.is_empty());
}

/// `source()` accessor on BashParser
#[test]
fn test_parser_source_accessor() {
    let input = "echo hello";
    let parser = BashParser::new(input).unwrap();
    assert_eq!(parser.source(), input);
}

/// Until loop — exercises parse_until path (distinct from while)
#[test]
fn test_parse_until_loop() {
    let input = "until false; do\n  echo waiting\ndone";
    let result = BashParser::new(input).and_then(|mut p| p.parse());
    // until is supported
    let _ = result;
}

/// Brace group — exercises parse_brace_group
#[test]
fn test_parse_brace_group() {
    let input = "{ echo a; echo b; }";
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    assert!(!ast.statements.is_empty());
}

/// Subshell — exercises parse_subshell
#[test]
fn test_parse_subshell() {
    let input = "(echo hello; echo world)";
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    assert!(!ast.statements.is_empty());
}

/// Nested if-elif-else — deeper branch coverage
#[test]
fn test_parse_if_elif_else() {
    let input = r#"if [ $x = 1 ]; then
  echo one
elif [ $x = 2 ]; then
  echo two
else
  echo other
fi"#;
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    assert!(ast
        .statements
        .iter()
        .any(|s| matches!(s, BashStmt::If { .. })));
}

/// Arithmetic expansion in assignment
#[test]
fn test_parse_arithmetic_expansion_assignment() {
    let input = "RESULT=$((1 + 2))";
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    assert!(!ast.statements.is_empty());
}

/// Command substitution in assignment
#[test]
fn test_parse_command_substitution_assignment() {
    let input = "PWD=$(pwd)";
    let mut parser = BashParser::new(input).unwrap();
    let ast = parser.parse().unwrap();
    assert!(!ast.statements.is_empty());
}

/// Multiple FD redirects on the same command
#[test]
fn test_parse_multiple_redirects_on_command() {
    let input = "cmd 2>/dev/null 1>/tmp/out";
    let result = BashParser::new(input).and_then(|mut p| p.parse());
    let _ = result;
}

/// Negated pipeline (! pipeline)
#[test]
fn test_parse_negated_pipeline() {
    let input = "! grep foo bar.txt";
    let result = BashParser::new(input).and_then(|mut p| p.parse());
    let _ = result;
}
