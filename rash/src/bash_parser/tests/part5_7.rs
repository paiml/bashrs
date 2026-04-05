#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(unused_imports)]

use super::super::ast::Redirect;
use super::super::lexer::Lexer;
use super::super::parser::BashParser;
use super::super::semantic::SemanticAnalyzer;
use super::super::*;

#[test]
fn test_ISSUE_060_002_standalone_brace_group() {
    let script = r#"{ echo "hello"; echo "world"; }"#;

    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let result = parser.parse();

    // ASSERT: Parser must accept standalone brace groups
    assert!(
        result.is_ok(),
        "Parser MUST accept standalone brace group: {:?}",
        result.err()
    );

    let ast = result.expect("Should parse");
    assert!(
        !ast.statements.is_empty(),
        "Should have at least one statement"
    );

    // Should be a BraceGroup
    match &ast.statements[0] {
        BashStmt::BraceGroup { body, .. } => {
            assert!(
                body.len() >= 2,
                "Brace group should have at least 2 statements, got: {}",
                body.len()
            );
        }
        other => panic!("Expected BraceGroup statement, got: {:?}", other),
    }
}

/// Issue #60: Test parsing brace group after && operator
/// INPUT: test -f file && { echo "exists"; cat file; }
#[test]
fn test_ISSUE_060_003_brace_group_after_and() {
    let script = r#"test -f file && { echo "exists"; cat file; }"#;

    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let result = parser.parse();

    // ASSERT: Parser must accept brace groups after &&
    assert!(
        result.is_ok(),
        "Parser MUST accept brace group after &&: {:?}",
        result.err()
    );
}

// ============================================================================
// Issue #62: Extended test [[ ]] conditionals
// ============================================================================
// Bug: Parser fails on bash [[ ]] extended test syntax
// Root cause: Parser only handles POSIX [ ] tests, not bash [[ ]] tests

/// Issue #62: Test basic [[ ]] conditional in if statement
/// INPUT: if [[ -f file ]]; then echo exists; fi
/// EXPECTED: Parse successfully with ExtendedTest expression
#[test]
fn test_ISSUE_062_001_extended_test_file_exists() {
    let script = r#"if [[ -f /tmp/test.txt ]]; then echo exists; fi"#;

    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let result = parser.parse();

    // ASSERT: Parser must accept [[ ]] extended test syntax
    assert!(
        result.is_ok(),
        "Parser MUST accept [[ ]] extended test: {:?}",
        result.err()
    );
}

/// Issue #62: Test [[ ]] with negation
/// INPUT: if [[ ! -s file ]]; then echo empty; fi
/// EXPECTED: Parse successfully with negated test
#[test]
fn test_ISSUE_062_002_extended_test_negation() {
    let script = r#"if [[ ! -s /tmp/file.txt ]]; then echo "File is empty"; exit 1; fi"#;

    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "Parser MUST accept [[ ! ... ]] negated test: {:?}",
        result.err()
    );
}

/// Issue #62: Test [[ ]] with string comparison
/// INPUT: if [[ "$var" == "value" ]]; then ...; fi
/// EXPECTED: Parse successfully
#[test]
fn test_ISSUE_062_003_extended_test_string_comparison() {
    let script = r#"if [[ "$total" -eq 0 ]]; then echo "No data"; exit 1; fi"#;

    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "Parser MUST accept [[ ]] string comparison: {:?}",
        result.err()
    );
}

/// Issue #62: Test standalone [[ ]] as condition
/// INPUT: [[ -d /tmp ]] && echo "exists"
/// EXPECTED: Parse successfully
#[test]
fn test_ISSUE_062_004_extended_test_standalone() {
    let script = r#"[[ -d /tmp ]] && echo "directory exists""#;

    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "Parser MUST accept standalone [[ ]] test: {:?}",
        result.err()
    );
}

// ============================================================================
// Issue #61: Parser error with here-strings (<<<)
// ============================================================================
// Here-strings are a bash feature that provide a string to a command's stdin.
// Syntax: cmd <<< "string"
// This is NOT a heredoc (<<), it's a simpler single-line input mechanism.
//
// Master Ticket: #63 (Bash Syntax Coverage Gaps)
// ============================================================================

/// Test: Issue #61 - Basic here-string with variable
/// Input: `read line <<< "$data"`
/// Expected: Parser accepts here-string redirection
#[test]
fn test_ISSUE_061_001_herestring_basic() {
    let script = r#"data="hello world"
read line <<< "$data"
echo "$line""#;

    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "Parser MUST accept here-string <<<: {:?}",
        result.err()
    );
}

/// Test: Issue #61 - Here-string with literal string
/// Input: `cat <<< "hello world"`
/// Expected: Parser accepts here-string with literal
#[test]
fn test_ISSUE_061_002_herestring_literal() {
    let script = r#"cat <<< "hello world""#;

    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "Parser MUST accept here-string with literal: {:?}",
        result.err()
    );
}

/// Test: Issue #61 - Here-string with unquoted word
/// Input: `read word <<< hello`
/// Expected: Parser accepts here-string with unquoted word
#[test]
fn test_ISSUE_061_003_herestring_unquoted() {
    let script = r#"read word <<< hello"#;

    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "Parser MUST accept here-string with unquoted word: {:?}",
        result.err()
    );
}

/// Test: Issue #61 - Here-string in pipeline
/// Input: `cat <<< "test" | grep t`
/// Expected: Parser accepts here-string in pipeline
#[test]
fn test_ISSUE_061_004_herestring_pipeline() {
    let script = r#"cat <<< "test" | grep t"#;

    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "Parser MUST accept here-string in pipeline: {:?}",
        result.err()
    );
}

// =============================================================================
// F001-F020: Parser Falsification Tests (Issue #93, #103)
// Specification: docs/specifications/unix-runtime-improvements-docker-mac-bash-zsh-daemons.md
// =============================================================================

/// F001: Parser handles inline if/then/else/fi
/// Issue #93: Parser fails on valid inline if/then/else/fi syntax
/// Falsification: If this test fails, the hypothesis "parser handles inline if" is falsified
#[test]
fn test_F001_inline_if_then_else_fi() {
    let script = r#"if grep -q "pattern" "$FILE"; then echo "found"; else echo "not found"; fi"#;

    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "F001 FALSIFIED: Parser MUST handle inline if/then/else/fi. Error: {:?}",
        result.err()
    );

    let ast = result.unwrap();
    assert_eq!(
        ast.statements.len(),
        1,
        "F001 FALSIFIED: Should produce exactly one If statement"
    );

    match &ast.statements[0] {
        BashStmt::If {
            then_block,
            else_block,
            ..
        } => {
            assert!(
                !then_block.is_empty(),
                "F001 FALSIFIED: then_block should not be empty"
            );
            assert!(
                else_block.is_some(),
                "F001 FALSIFIED: else_block should be present"
            );
        }
        other => panic!("F001 FALSIFIED: Expected If statement, got {:?}", other),
    }
}

/// F001 variant: Inline if with command condition (Issue #93 exact reproduction)
#[test]
fn test_F001_issue93_exact_reproduction() {
    // Exact test case from Issue #93
    let script =
        r#"if grep -q "MAX_QUEUE_DEPTH.*=.*3" "$BRIDGE"; then pass "1"; else fail "2"; fi"#;

    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "F001 FALSIFIED: Issue #93 exact case must parse. Error: {:?}",
        result.err()
    );
}

/// F002: Parser handles empty array initialization
/// Issue #103: Parser fails on common bash array syntax
#[test]
fn test_F002_empty_array_initialization() {
    let script = r#"local arr=()"#;

    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "F002 FALSIFIED: Parser MUST handle empty array initialization. Error: {:?}",
        result.err()
    );
}

/// F003: Parser handles array append operator
/// Issue #103: Parser fails on arr+=("item") syntax
#[test]
fn test_F003_array_append_operator() {
    let script = r#"arr+=("item")"#;

    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "F003 FALSIFIED: Parser MUST handle array append operator. Error: {:?}",
        result.err()
    );
}

/// F004: Parser handles stderr redirect shorthand
/// Issue #103: Parser fails on >&2 syntax
#[test]
fn test_F004_stderr_redirect_shorthand() {
    let script = r#"echo "error" >&2"#;

    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "F004 FALSIFIED: Parser MUST handle stderr redirect shorthand >&2. Error: {:?}",
        result.err()
    );
}

/// F005: Parser handles combined redirect &>/dev/null
/// Issue #103: Parser fails on &>/dev/null syntax
#[test]
fn test_F005_combined_redirect() {
    let script = r#"command &>/dev/null"#;

    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "F005 FALSIFIED: Parser MUST handle combined redirect &>. Error: {:?}",
        result.err()
    );
}

/// F006: Parser handles heredoc with quoted delimiter (content not shell-parsed)
/// Issue #120: SC2247 triggers on Python in heredoc
#[test]
fn test_F006_heredoc_quoted_delimiter() {
    let script = r#"cat << 'EOF'
target_bytes = $gb * 1024
chunks = []
EOF"#;

    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "F006 FALSIFIED: Parser MUST handle heredoc with quoted delimiter. Error: {:?}",
        result.err()
    );
}

/// F007: Parser handles line continuation in shell
#[test]
fn test_F007_line_continuation() {
    let script = "echo \"line1 \\\nline2\"";

    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "F007 FALSIFIED: Parser MUST handle line continuation. Error: {:?}",
        result.err()
    );
}

/// F008: Parser handles case statement with all branches assigning variable
/// Issue #99: SC2154 false positive for case variables
#[test]
fn test_F008_case_all_branches_assign() {
    let script = r#"
case "$SHELL" in
    */zsh)  shell_rc="$HOME/.zshrc" ;;
    */bash) shell_rc="$HOME/.bashrc" ;;
    *)      shell_rc="$HOME/.profile" ;;
esac
echo "$shell_rc"
"#;

    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "F008 FALSIFIED: Parser MUST handle case with all branches. Error: {:?}",
        result.err()
    );

    let ast = result.unwrap();
    // Should have case statement and echo
    assert!(
        ast.statements.len() >= 2,
        "F008 FALSIFIED: Should have case and echo statements"
    );
}

/// F009: Parser handles nested command substitution
#[test]
fn test_F009_nested_command_substitution() {
    let script = r#"echo "$(dirname "$(pwd)")""#;

    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "F009 FALSIFIED: Parser MUST handle nested command substitution. Error: {:?}",
        result.err()
    );
}

/// F010: Parser handles process substitution
#[test]
fn test_F010_process_substitution() {
    let script = r#"diff <(ls dir1) <(ls dir2)"#;

    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "F010 FALSIFIED: Parser MUST handle process substitution. Error: {:?}",
        result.err()
    );
}

/// F011: Parser distinguishes brace expansion from parameter expansion
/// Issue #93: SC2125 false positive
#[test]
fn test_F011_brace_vs_parameter_expansion() {
    let script = r#"VAR=${VAR:-default}"#;

    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "F011 FALSIFIED: Parser MUST handle parameter expansion with default. Error: {:?}",
        result.err()
    );
}

/// F012: Parser handles arithmetic expansion
#[test]
fn test_F012_arithmetic_expansion() {
    let script = r#"result=$((x + y * 2))"#;

    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "F012 FALSIFIED: Parser MUST handle arithmetic expansion. Error: {:?}",
        result.err()
    );
}

/// F013: Parser handles parameter expansion modifiers
#[test]
fn test_F013_parameter_expansion_modifiers() {
    let script = r#"
echo "${var:+set}"
echo "${var:?error message}"
echo "${var:-default}"
echo "${var:=assign}"
"#;

    let mut parser = BashParser::new(script).expect("Lexer should succeed");
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "F013 FALSIFIED: Parser MUST handle parameter expansion modifiers. Error: {:?}",
        result.err()
    );
}
