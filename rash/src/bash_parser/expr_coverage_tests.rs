//! Coverage tests for bash_parser/parser_expr.rs uncovered branches.
//!
//! Targets: variable expansion edge cases, parse_expression branches,
//! array literals, sparse arrays, glob bracket patterns, test expressions,
//! condition command redirect parsing, and keyword_as_str branches.
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::ast::{BashExpr, BashStmt};
use super::parser::BashParser;

/// Helper: parse input and return the AST, panicking on failure.
fn parse_ok(input: &str) -> super::ast::BashAst {
    let mut p = BashParser::new(input).unwrap();
    p.parse().unwrap()
}

/// Helper: parse input, accepting either Ok or Err (no panic).
fn parse_no_panic(input: &str) {
    let _ = BashParser::new(input).and_then(|mut p| p.parse());
}

// ---------------------------------------------------------------------------
// parse_variable_expansion — all parameter expansion operators
// ---------------------------------------------------------------------------

#[test]
fn test_var_expansion_all_operators() {
    // Each exercises a distinct branch in parse_variable_expansion
    let cases = [
        "echo ${#PATH}",          // StringLength
        "echo ${HOME:-/tmp}",     // DefaultValue :-
        "echo ${TMPDIR:=/tmp}",   // AssignDefault :=
        "echo ${DEBUG:+enabled}", // AlternativeValue :+
        "echo ${CFG:?required}",  // ErrorIfUnset :?
        "echo ${PATH##*/}",       // RemoveLongestPrefix ##
        "echo ${FILE#*/}",        // RemovePrefix #
        "echo ${FILE%%.*}",       // RemoveLongestSuffix %%
        "echo ${FILE%.*}",        // RemoveSuffix %
        "echo ${HOME}",           // Simple variable (no operator)
    ];
    for input in cases {
        let ast = parse_ok(input);
        assert!(!ast.statements.is_empty(), "failed for: {input}");
    }
}

// ---------------------------------------------------------------------------
// parse_expression — branch coverage for each token type
// ---------------------------------------------------------------------------

#[test]
fn test_expr_number_token() {
    let ast = parse_ok("echo 42");
    if let BashStmt::Command { args, .. } = &ast.statements[0] {
        assert!(matches!(args[0], BashExpr::Literal(ref s) if s == "42"));
    }
}

#[test]
fn test_expr_arithmetic_expansion() {
    let ast = parse_ok("X=$((2 + 3))");
    if let BashStmt::Assignment { value, .. } = &ast.statements[0] {
        assert!(matches!(value, BashExpr::Arithmetic(_)));
    }
}

#[test]
fn test_expr_command_substitution() {
    let ast = parse_ok("DIR=$(pwd)");
    if let BashStmt::Assignment { value, .. } = &ast.statements[0] {
        assert!(matches!(value, BashExpr::CommandSubst(_)));
    }
}

#[test]
fn test_expr_heredoc_token() {
    parse_no_panic("cat <<EOF\nhello world\nEOF");
}

#[test]
fn test_expr_glob_identifier() {
    let ast = parse_ok("ls *.txt");
    if let BashStmt::Command { args, .. } = &ast.statements[0] {
        assert!(matches!(&args[0], BashExpr::Glob(ref s) if s.contains('*')));
    }
}

#[test]
fn test_expr_brace_literal() {
    // {} literal in argument context (find -exec cmd {} \;)
    parse_no_panic("find . -name test -exec rm {} ;");
}

#[test]
fn test_expr_keywords_as_arguments() {
    // Each keyword token used as a literal argument exercises keyword_as_str
    let keywords = [
        "done", "fi", "then", "while", "for", "case", "in", "if", "elif", "else", "until", "do",
        "esac", "function", "return", "export", "local", "coproc", "select",
    ];
    for kw in keywords {
        let input = format!("echo {kw}");
        let result = BashParser::new(&input).and_then(|mut p| p.parse());
        // Some keywords may start control flow; we just verify no panic
        let _ = result;
    }
}

// ---------------------------------------------------------------------------
// Array literal and sparse array parsing
// ---------------------------------------------------------------------------

#[test]
fn test_array_literal_basic() {
    let ast = parse_ok("ARR=(one two three)");
    if let BashStmt::Assignment { value, .. } = &ast.statements[0] {
        assert!(matches!(value, BashExpr::Array(_)));
    }
}

#[test]
fn test_array_literal_sparse() {
    let ast = parse_ok("ARR=([0]=first [3]=fourth)");
    if let BashStmt::Assignment { value, .. } = &ast.statements[0] {
        assert!(matches!(value, BashExpr::Array(_)));
    }
}

#[test]
fn test_array_with_newlines() {
    let ast = parse_ok("ARR=(\nalpha\nbeta\ngamma\n)");
    assert!(!ast.statements.is_empty());
}

#[test]
fn test_sparse_array_edge_cases() {
    // Variable value and empty value in sparse array
    parse_no_panic("ARR=([0]=$HOME)");
    parse_no_panic("ARR=([0]=)");
}

// ---------------------------------------------------------------------------
// Glob bracket pattern parsing
// ---------------------------------------------------------------------------

#[test]
fn test_glob_bracket_patterns() {
    parse_no_panic("ls [0-9]"); // digits
    parse_no_panic("ls [a-z]"); // alpha
    parse_no_panic("ls [!abc]"); // negation (Not token)
    parse_no_panic("ls [0-9]*.sql"); // trailing glob absorption
}

// ---------------------------------------------------------------------------
// Test expressions — single bracket operators
// ---------------------------------------------------------------------------

#[test]
fn test_single_bracket_string_operators() {
    assert!(BashParser::new("if [ \"$x\" = \"y\" ]; then echo m; fi")
        .and_then(|mut p| p.parse())
        .is_ok());
    assert!(BashParser::new("if [ \"$x\" != \"y\" ]; then echo m; fi")
        .and_then(|mut p| p.parse())
        .is_ok());
}

#[test]
fn test_single_bracket_int_operators() {
    let ops = ["-eq", "-ne", "-lt", "-le", "-gt", "-ge"];
    for op in ops {
        let input = format!("if [ $x {op} 5 ]; then echo ok; fi");
        let result = BashParser::new(&input).and_then(|mut p| p.parse());
        assert!(result.is_ok(), "failed for operator: {op}");
    }
}

#[test]
fn test_single_bracket_and_or() {
    assert!(BashParser::new("if [ -f /a -a -f /b ]; then echo x; fi")
        .and_then(|mut p| p.parse())
        .is_ok());
    assert!(BashParser::new("if [ -f /a -o -f /b ]; then echo x; fi")
        .and_then(|mut p| p.parse())
        .is_ok());
}

#[test]
fn test_single_bracket_lt_gt_operators() {
    parse_no_panic("if [ \"$a\" < \"$b\" ]; then echo lt; fi");
    parse_no_panic("if [ \"$a\" > \"$b\" ]; then echo gt; fi");
}

// ---------------------------------------------------------------------------
// Test expressions — double bracket
// ---------------------------------------------------------------------------

#[test]
fn test_double_bracket_combinators() {
    assert!(BashParser::new("if [[ -f /a && -d /b ]]; then echo x; fi")
        .and_then(|mut p| p.parse())
        .is_ok());
    assert!(BashParser::new("if [[ -f /a || -d /b ]]; then echo x; fi")
        .and_then(|mut p| p.parse())
        .is_ok());
    assert!(BashParser::new("if [[ ! -f /tmp/no ]]; then echo x; fi")
        .and_then(|mut p| p.parse())
        .is_ok());
    assert!(BashParser::new("if [[ $x == yes ]]; then echo x; fi")
        .and_then(|mut p| p.parse())
        .is_ok());
}

// ---------------------------------------------------------------------------
// Test expression — unary file/string test operators
// ---------------------------------------------------------------------------

#[test]
fn test_unary_test_operators() {
    let ops = ["-f", "-e", "-s", "-d", "-r", "-w", "-x", "-L", "-n", "-z"];
    for op in ops {
        let input = format!("if [ {op} /tmp/test ]; then echo ok; fi");
        let result = BashParser::new(&input).and_then(|mut p| p.parse());
        assert!(result.is_ok(), "failed for unary op: {op}");
    }
}

// ---------------------------------------------------------------------------
// Negated test expression and compound tests
// ---------------------------------------------------------------------------

#[test]
fn test_negated_conditions() {
    parse_no_panic("if ! grep -q pattern file; then echo no; fi");
    parse_no_panic("if ! [ -f /tmp/x ]; then echo no; fi");
}

#[test]
fn test_compound_test_and_or() {
    assert!(
        BashParser::new("if [ -f /a ] && [ -f /b ]; then echo x; fi")
            .and_then(|mut p| p.parse())
            .is_ok()
    );
    assert!(
        BashParser::new("if [ -f /a ] || [ -f /b ]; then echo x; fi")
            .and_then(|mut p| p.parse())
            .is_ok()
    );
}

// ---------------------------------------------------------------------------
// Condition command parsing (bare command, pipeline, assignment, subshell)
// ---------------------------------------------------------------------------

#[test]
fn test_condition_command_variants() {
    assert!(BashParser::new("if grep -q pat f; then echo y; fi")
        .and_then(|mut p| p.parse())
        .is_ok());
    parse_no_panic("if echo t | grep -q t; then echo p; fi"); // pipeline
    parse_no_panic("if pid=$(pgrep sshd); then echo r; fi"); // assignment
    parse_no_panic("if ( cd /tmp && ls ); then echo ok; fi"); // subshell
    parse_no_panic("if $CMD; then echo ran; fi"); // variable
}

// ---------------------------------------------------------------------------
// Condition command with env prefixes and redirects
// ---------------------------------------------------------------------------

#[test]
fn test_condition_env_prefixes() {
    parse_no_panic("while IFS= read -r line; do echo $line; done");
    parse_no_panic("if LC_ALL=C sort --check f; then echo ok; fi");
}

#[test]
fn test_condition_redirects() {
    let redirects = [
        "if cmd > /dev/null; then echo ok; fi", // Output
        "if cmd >> /tmp/log; then echo ok; fi", // Append
        "if cmd < /tmp/in; then echo ok; fi",   // Input
        "if cmd 2>/dev/null; then echo ok; fi", // fd>file
        "if cmd 2>&1; then echo ok; fi",        // fd>&fd
        "if cmd &>/dev/null; then echo ok; fi", // Combined
        "if cmd >&2; then echo ok; fi",         // >&fd shorthand
    ];
    for input in redirects {
        parse_no_panic(input);
    }
}

// ---------------------------------------------------------------------------
// StringNonEmpty fallback (no binary operator after left operand)
// ---------------------------------------------------------------------------

#[test]
fn test_test_condition_bare_values() {
    assert!(BashParser::new("if [ hello ]; then echo x; fi")
        .and_then(|mut p| p.parse())
        .is_ok());
    assert!(BashParser::new("if [ $VAR ]; then echo x; fi")
        .and_then(|mut p| p.parse())
        .is_ok());
}

// ---------------------------------------------------------------------------
// at_condition_arg_boundary — edge cases
// ---------------------------------------------------------------------------

#[test]
fn test_condition_boundary_tokens() {
    parse_no_panic("if cmd arg1 & then echo ok; fi"); // ampersand bg
    parse_no_panic("if cmd arg1 # comment\nthen echo ok; fi"); // comment
    parse_no_panic("if (cmd arg); then echo ok; fi"); // right paren
}
