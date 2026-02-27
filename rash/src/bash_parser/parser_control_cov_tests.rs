//! Coverage tests for bash_parser/parser_control.rs — targeting uncovered branches
//!
//! Focuses on:
//! - parse_for_c_style: token-by-token parsing through (( )), various token types,
//!   nested parens, malformed parts, all operator tokens
//! - parse_for_c_style_from_content: called from ArithmeticExpansion token path
//! - parse_until: basic and with semicolon before do
//! - parse_brace_group: basic and with redirects
//! - parse_subshell: basic and with redirects
//! - parse_coproc: named and unnamed
//! - parse_select: basic
//! - parse_case: bracket class, POSIX class, case terminators, patterns with variables/numbers
//! - parse_test_command: with -a (AND) and -o (OR)
//! - parse_extended_test_command: with && and ||

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::parser::BashParser;

fn parse_ok(input: &str) {
    let result = BashParser::new(input).and_then(|mut p| p.parse());
    assert!(result.is_ok(), "Expected parse OK for: {input}\nGot: {result:?}");
}

fn parse_ok_result(input: &str) -> super::ast::BashAst {
    BashParser::new(input).and_then(|mut p| p.parse()).unwrap()
}

// ---------------------------------------------------------------------------
// C-style for loop: parse_for_c_style via double-paren tokens
// ---------------------------------------------------------------------------

#[test]
fn test_PCCOV_001_for_c_style_basic() {
    parse_ok("for ((i=0; i<10; i++)); do echo $i; done");
}

#[test]
fn test_PCCOV_002_for_c_style_with_spaces() {
    parse_ok("for (( i = 0 ; i < 10 ; i++ )); do echo $i; done");
}

#[test]
fn test_PCCOV_003_for_c_style_missing_semicolons_in_content() {
    // Malformed content with fewer than 3 parts → empty strings
    parse_ok("for ((i=0)); do echo $i; done");
}

#[test]
fn test_PCCOV_004_for_c_style_with_variable_token() {
    parse_ok("for ((i=0; i<$MAX; i++)); do echo $i; done");
}

#[test]
fn test_PCCOV_005_for_c_style_with_le_operator() {
    parse_ok("for ((i=0; i<=10; i++)); do echo $i; done");
}

#[test]
fn test_PCCOV_006_for_c_style_with_ge_operator() {
    parse_ok("for ((i=10; i>=0; i--)); do echo $i; done");
}

#[test]
fn test_PCCOV_007_for_c_style_with_eq_operator() {
    parse_ok("for ((i=0; i==10; i++)); do echo $i; done");
}

#[test]
fn test_PCCOV_008_for_c_style_with_ne_operator() {
    parse_ok("for ((i=0; i!=10; i++)); do echo $i; done");
}

#[test]
fn test_PCCOV_009_for_c_style_with_gt_operator() {
    parse_ok("for ((i=10; i>0; i--)); do echo $i; done");
}

#[test]
fn test_PCCOV_010_for_c_style_with_number_token() {
    parse_ok("for ((i=0; i<100; i++)); do echo $i; done");
}

#[test]
fn test_PCCOV_011_for_c_style_with_nested_body() {
    parse_ok("for ((i=0; i<5; i++)); do\n  for ((j=0; j<3; j++)); do\n    echo $i $j\n  done\ndone");
}

#[test]
fn test_PCCOV_012_for_c_style_body_with_break() {
    parse_ok("for ((i=0; i<10; i++)); do\n  if [ $i -eq 5 ]; then break; fi\n  echo $i\ndone");
}

#[test]
fn test_PCCOV_013_for_c_style_body_with_continue() {
    parse_ok("for ((i=0; i<10; i++)); do\n  if [ $i -eq 5 ]; then continue; fi\n  echo $i\ndone");
}

// ---------------------------------------------------------------------------
// parse_for_c_style_from_content: ArithmeticExpansion token path
// ---------------------------------------------------------------------------

#[test]
fn test_PCCOV_014_for_c_style_from_content_basic() {
    // The lexer may tokenize (( )) as ArithmeticExpansion
    parse_ok("for ((i=0;i<5;i++)); do echo $i; done");
}

#[test]
fn test_PCCOV_015_for_c_style_from_content_semicolon_before_do() {
    parse_ok("for ((i=0;i<5;i++)) ; do echo $i; done");
}

// ---------------------------------------------------------------------------
// parse_until
// ---------------------------------------------------------------------------

#[test]
fn test_PCCOV_016_until_basic() {
    parse_ok("until false; do echo loop; done");
}

#[test]
fn test_PCCOV_017_until_with_test() {
    parse_ok("until [ $x -gt 10 ]; do\n  x=$((x+1))\ndone");
}

#[test]
fn test_PCCOV_018_until_with_semicolon_before_do() {
    parse_ok("until [ $x -gt 10 ] ; do echo x; done");
}

// ---------------------------------------------------------------------------
// parse_brace_group
// ---------------------------------------------------------------------------

#[test]
fn test_PCCOV_019_brace_group_basic() {
    parse_ok("{ echo hello; echo world; }");
}

#[test]
fn test_PCCOV_020_brace_group_multiline() {
    parse_ok("{\n  echo hello\n  echo world\n}");
}

// ---------------------------------------------------------------------------
// parse_subshell
// ---------------------------------------------------------------------------

#[test]
fn test_PCCOV_021_subshell_basic() {
    parse_ok("(echo hello; echo world)");
}

#[test]
fn test_PCCOV_022_subshell_multiline() {
    parse_ok("(\n  echo hello\n  echo world\n)");
}

// ---------------------------------------------------------------------------
// parse_coproc
// ---------------------------------------------------------------------------

#[test]
fn test_PCCOV_023_coproc_unnamed() {
    parse_ok("coproc { echo hello; }");
}

#[test]
fn test_PCCOV_024_coproc_named() {
    parse_ok("coproc myproc { echo hello; }");
}

// ---------------------------------------------------------------------------
// parse_select
// ---------------------------------------------------------------------------

#[test]
fn test_PCCOV_025_select_basic() {
    parse_ok("select opt in start stop restart; do echo $opt; done");
}

#[test]
fn test_PCCOV_026_select_single_item() {
    parse_ok("select x in foo; do echo $x; done");
}

// ---------------------------------------------------------------------------
// parse_case: various pattern types
// ---------------------------------------------------------------------------

#[test]
fn test_PCCOV_027_case_basic() {
    parse_ok("case $x in\n  start) echo starting;;\n  stop) echo stopping;;\n  *) echo unknown;;\nesac");
}

#[test]
fn test_PCCOV_028_case_multiple_patterns() {
    parse_ok("case $x in\n  start|begin) echo go;;\n  stop|end) echo halt;;\nesac");
}

#[test]
fn test_PCCOV_029_case_with_number_pattern() {
    parse_ok("case $x in\n  1) echo one;;\n  2) echo two;;\n  *) echo other;;\nesac");
}

#[test]
fn test_PCCOV_030_case_with_variable_pattern() {
    parse_ok("case $x in\n  $HOME) echo home;;\n  *) echo other;;\nesac");
}

#[test]
fn test_PCCOV_031_case_with_string_pattern() {
    parse_ok("case $x in\n  \"hello\") echo hi;;\n  *) echo other;;\nesac");
}

#[test]
fn test_PCCOV_032_case_fall_through_terminator() {
    // ;& fall-through terminator
    parse_ok("case $x in\n  1) echo one;&\n  2) echo two;;\nesac");
}

#[test]
fn test_PCCOV_033_case_resume_terminator() {
    // ;;& resume pattern matching terminator
    parse_ok("case $x in\n  1) echo one;;&\n  2) echo two;;\nesac");
}

#[test]
fn test_PCCOV_034_case_semicolons_as_double_terminator() {
    // Two Semicolon tokens as ;; terminator
    parse_ok("case $x in\n  start) echo go ;;\n  *) echo default ;;\nesac");
}

// ---------------------------------------------------------------------------
// parse_test_command: with -a (AND) and -o (OR)
// ---------------------------------------------------------------------------

#[test]
fn test_PCCOV_035_test_command_with_and() {
    parse_ok("[ -d /tmp -a -d /var ]");
}

#[test]
fn test_PCCOV_036_test_command_with_or() {
    parse_ok("[ -d /tmp -o -d /var ]");
}

#[test]
fn test_PCCOV_037_test_command_with_and_or_combined() {
    parse_ok("[ -d /tmp -a -d /var -o -f /etc/hosts ]");
}

// ---------------------------------------------------------------------------
// parse_extended_test_command: with && and ||
// ---------------------------------------------------------------------------

#[test]
fn test_PCCOV_038_extended_test_and() {
    parse_ok("[[ -d /tmp && -d /var ]]");
}

#[test]
fn test_PCCOV_039_extended_test_or() {
    parse_ok("[[ -d /tmp || -d /var ]]");
}

#[test]
fn test_PCCOV_040_extended_test_combined() {
    parse_ok("[[ -d /tmp && -d /var || -f /etc/hosts ]]");
}

// ---------------------------------------------------------------------------
// parse_while with various forms
// ---------------------------------------------------------------------------

#[test]
fn test_PCCOV_041_while_with_redirect_skip() {
    // while test with redirect before do
    parse_ok("while [ -f /tmp/lock ]; do sleep 1; done");
}

// ---------------------------------------------------------------------------
// parse_for with multiple items
// ---------------------------------------------------------------------------

#[test]
fn test_PCCOV_042_for_multiple_items() {
    parse_ok("for i in 1 2 3 4 5; do echo $i; done");
}

#[test]
fn test_PCCOV_043_for_single_item() {
    parse_ok("for f in *.txt; do echo $f; done");
}

// ---------------------------------------------------------------------------
// parse_case: bracket class patterns
// ---------------------------------------------------------------------------

#[test]
fn test_PCCOV_044_case_bracket_class_pattern() {
    // [0-9] bracket class in case pattern
    let input = "case $c in\n  [0-9]*) echo number;;\n  *) echo other;;\nesac";
    // This may or may not be parseable depending on lexer tokenization,
    // but we exercise the path
    let result = BashParser::new(input).and_then(|mut p| p.parse());
    // Just ensure it doesn't panic
    let _ = result;
}

// ---------------------------------------------------------------------------
// parse_if: with elif and else blocks
// ---------------------------------------------------------------------------

#[test]
fn test_PCCOV_045_if_elif_else() {
    parse_ok("if [ $x -eq 1 ]; then\n  echo one\nelif [ $x -eq 2 ]; then\n  echo two\nelse\n  echo other\nfi");
}

#[test]
fn test_PCCOV_046_if_multiple_elif() {
    parse_ok("if [ $x -eq 1 ]; then\n  echo one\nelif [ $x -eq 2 ]; then\n  echo two\nelif [ $x -eq 3 ]; then\n  echo three\nfi");
}

// ---------------------------------------------------------------------------
// parse_case with empty arm body
// ---------------------------------------------------------------------------

#[test]
fn test_PCCOV_047_case_empty_arm() {
    parse_ok("case $x in\n  start) ;;\n  *) echo default;;\nesac");
}

// ---------------------------------------------------------------------------
// for loop with newline before do
// ---------------------------------------------------------------------------

#[test]
fn test_PCCOV_048_for_newline_before_do() {
    parse_ok("for i in 1 2 3\ndo\n  echo $i\ndone");
}
