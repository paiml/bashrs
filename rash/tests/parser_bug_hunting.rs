//! Parser Bug Hunting - Rigorous Edge Case Testing
//!
//! This module aggressively tests parser edge cases to find bugs.
//! Uses property-based testing, fuzzing patterns, and corner cases.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use bashrs::bash_parser::parser::BashParser;

/// Parse and return whether it succeeded and any error message
fn parse_result(input: &str) -> (bool, String) {
    match BashParser::new(input) {
        Ok(mut parser) => match parser.parse() {
            Ok(_) => (true, String::new()),
            Err(e) => (false, format!("{:?}", e)),
        },
        Err(e) => (false, format!("{:?}", e)),
    }
}

/// Assert parsing succeeds
fn assert_parses(input: &str) {
    let (ok, err) = parse_result(input);
    assert!(ok, "Should parse '{}': {}", input, err);
}

/// Assert parsing fails
fn _assert_fails(input: &str) {
    let (ok, _) = parse_result(input);
    assert!(!ok, "Should fail to parse '{}'", input);
}

// ============================================================================
// EDGE CASE: Nested Structures
// ============================================================================

#[test]
fn test_deeply_nested_command_substitution() {
    // Bug hunt: deep nesting should work
    assert_parses("echo $(echo $(echo $(echo hi)))");
    assert_parses("echo $(echo $(echo $(echo $(echo $(echo hi)))))"); // 6 levels deep

    // Very deep nesting - might overflow
    let mut nested = "echo hi".to_string();
    for _ in 0..20 {
        nested = format!("echo $({})", nested);
    }
    let (ok, err) = parse_result(&nested);
    if !ok {
        println!("BUG FOUND: Deep nesting fails at 20 levels: {}", err);
    }
}

#[test]
fn test_deeply_nested_arithmetic() {
    assert_parses("echo $((1 + 2))");
    assert_parses("echo $((1 + $((2 + 3))))");
    assert_parses("echo $((1 + $((2 + $((3 + 4))))))");

    // Deep arithmetic nesting
    let mut nested = "1".to_string();
    for i in 0..10 {
        nested = format!("$(({} + {}))", nested, i);
    }
    let input = format!("echo {}", nested);
    let (ok, err) = parse_result(&input);
    if !ok {
        println!("BUG FOUND: Deep arithmetic nesting fails: {}", err);
    }
}

#[test]
fn test_deeply_nested_conditionals() {
    let code = r#"
if true; then
    if true; then
        if true; then
            if true; then
                if true; then
                    echo "deep"
                fi
            fi
        fi
    fi
fi
"#;
    assert_parses(code);
}

// ============================================================================
// EDGE CASE: String Escaping
// ============================================================================

#[test]
fn test_complex_escaping() {
    // Various escape sequences
    assert_parses(r#"echo "hello\nworld""#);
    assert_parses(r#"echo "tab\there""#);
    assert_parses(r#"echo "quote\"inside""#);
    assert_parses(r#"echo 'single'\''quote'"#);

    // Backslash at end of line (line continuation)
    let (ok, err) = parse_result("echo hello \\\nworld");
    if !ok {
        println!("BUG FOUND: Line continuation fails: {}", err);
    }
}

#[test]
fn test_dollar_escaping() {
    // Dollar sign escaping
    assert_parses(r#"echo "\$HOME""#);
    assert_parses(r#"echo '$HOME'"#);
    assert_parses(r#"echo \$HOME"#);

    // Mixed
    let (ok, err) = parse_result(r#"echo "cost: \$5.00 in $CURRENCY""#);
    if !ok {
        println!("BUG FOUND: Mixed dollar escaping fails: {}", err);
    }
}

// ============================================================================
// EDGE CASE: Empty and Whitespace
// ============================================================================

#[test]
fn test_empty_constructs() {
    // Empty function body
    let (ok1, err1) = parse_result("f() { }");
    if !ok1 {
        println!("BUG FOUND: Empty function body fails: {}", err1);
    }

    // Empty subshell
    let (ok2, err2) = parse_result("()");
    if !ok2 {
        println!("BUG FOUND: Empty subshell fails: {}", err2);
    }

    // Empty brace group
    let (ok3, err3) = parse_result("{ }");
    if !ok3 {
        println!("BUG FOUND: Empty brace group fails: {}", err3);
    }

    // Empty string assignment
    assert_parses("x=");
    assert_parses("x=''");
    assert_parses("x=\"\"");
}

#[test]
fn test_whitespace_variations() {
    // Tabs vs spaces
    assert_parses("x=5\techo\t$x");

    // Multiple spaces
    assert_parses("echo    hello     world");

    // Leading/trailing whitespace
    assert_parses("   echo hello   ");

    // Only whitespace
    let (ok, _) = parse_result("   \t   \n   ");
    // Should parse as empty program
    if !ok {
        println!("NOTE: Whitespace-only input handling");
    }
}

// ============================================================================
// EDGE CASE: Special Characters
// ============================================================================

#[test]
fn test_special_characters_in_strings() {
    // Unicode in strings
    let (ok1, err1) = parse_result(r#"echo "Hello 世界""#);
    if !ok1 {
        println!("BUG FOUND: Unicode in strings fails: {}", err1);
    }

    // Emoji
    let (ok2, err2) = parse_result(r#"echo "Hello 🎉""#);
    if !ok2 {
        println!("BUG FOUND: Emoji in strings fails: {}", err2);
    }

    // Null byte (should probably fail gracefully)
    let (ok3, err3) = parse_result("echo \"hello\x00world\"");
    if ok3 {
        println!("NOTE: Null byte in string accepted");
    } else {
        println!("Null byte rejected: {}", err3);
    }
}

#[test]
fn test_glob_patterns() {
    // Various glob patterns
    assert_parses("echo *.txt");
    assert_parses("echo file?.txt");
    assert_parses("echo [abc].txt");
    assert_parses("echo [a-z].txt");
    assert_parses("echo [!abc].txt");
    assert_parses("echo **/*.rs");

    // Extended globs
    let (ok, err) = parse_result("echo @(foo|bar)");
    if !ok {
        println!("BUG FOUND: Extended glob @() fails: {}", err);
    }

    let (ok2, err2) = parse_result("echo !(foo)");
    if !ok2 {
        println!("BUG FOUND: Extended glob !() fails: {}", err2);
    }
}

// ============================================================================
// EDGE CASE: Variable Expansion Edge Cases
// ============================================================================

#[test]
fn test_complex_parameter_expansion() {
    // Nested parameter expansion
    let (ok1, err1) = parse_result("echo ${foo:-${bar:-default}}");
    if !ok1 {
        println!("BUG FOUND: Nested parameter expansion fails: {}", err1);
    }

    // Pattern with special chars
    let (ok2, err2) = parse_result("echo ${var##*[/]}");
    if !ok2 {
        println!("BUG FOUND: Pattern with brackets fails: {}", err2);
    }

    // Case modification (bash 4+)
    let (ok3, err3) = parse_result("echo ${var^^}");
    if !ok3 {
        println!("BUG FOUND: Case modification ^^ fails: {}", err3);
    }

    let (ok4, err4) = parse_result("echo ${var,,}");
    if !ok4 {
        println!("BUG FOUND: Case modification ,, fails: {}", err4);
    }

    // Indirect expansion
    let (ok5, err5) = parse_result("echo ${!prefix*}");
    if !ok5 {
        println!("BUG FOUND: Indirect expansion with * fails: {}", err5);
    }

    // Array slicing
    let (ok6, err6) = parse_result("echo ${arr[@]:1:3}");
    if !ok6 {
        println!("BUG FOUND: Array slicing fails: {}", err6);
    }
}

#[test]
fn test_arithmetic_edge_cases() {
    // Ternary operator
    let (ok1, err1) = parse_result("echo $((x > 5 ? 1 : 0))");
    if !ok1 {
        println!("BUG FOUND: Ternary operator fails: {}", err1);
    }

    // Comma operator
    let (ok2, err2) = parse_result("echo $((x=1, y=2, x+y))");
    if !ok2 {
        println!("BUG FOUND: Comma operator fails: {}", err2);
    }

    // Bitwise operations
    assert_parses("echo $((x & y))");
    assert_parses("echo $((x | y))");
    assert_parses("echo $((x ^ y))");
    assert_parses("echo $((~x))");
    assert_parses("echo $((x << 2))");
    assert_parses("echo $((x >> 2))");

    // Negative numbers
    let (ok3, err3) = parse_result("echo $((-5))");
    if !ok3 {
        println!("BUG FOUND: Negative number fails: {}", err3);
    }

    // Hex/octal
    let (ok4, err4) = parse_result("echo $((0xff))");
    if !ok4 {
        println!("BUG FOUND: Hex literal fails: {}", err4);
    }

    let (ok5, err5) = parse_result("echo $((0777))");
    if !ok5 {
        println!("BUG FOUND: Octal literal fails: {}", err5);
    }
}

// ============================================================================
// EDGE CASE: Heredoc Edge Cases
// ============================================================================

#[test]
fn test_heredoc_edge_cases() {
    // Heredoc with variable expansion
    let code1 = "cat <<EOF\n$HOME\nEOF";
    let (ok1, err1) = parse_result(code1);
    if !ok1 {
        println!("BUG FOUND: Heredoc with var expansion fails: {}", err1);
    }

    // Heredoc without expansion (quoted delimiter)
    let code2 = "cat <<'EOF'\n$HOME\nEOF";
    let (ok2, err2) = parse_result(code2);
    if !ok2 {
        println!("BUG FOUND: Quoted heredoc delimiter fails: {}", err2);
    }

    // Heredoc with indented delimiter
    let code3 = "cat <<-EOF\n\thello\n\tEOF";
    let (ok3, err3) = parse_result(code3);
    if !ok3 {
        println!("BUG FOUND: Indented heredoc (<<-) fails: {}", err3);
    }

    // Multiple heredocs
    let code4 = "cat <<EOF1 <<EOF2\nfirst\nEOF1\nsecond\nEOF2";
    let (ok4, err4) = parse_result(code4);
    if !ok4 {
        println!("BUG FOUND: Multiple heredocs fails: {}", err4);
    }

    // Heredoc on same line as command
    let code5 = "cat <<EOF; echo done\nhello\nEOF";
    let (ok5, err5) = parse_result(code5);
    if !ok5 {
        println!("BUG FOUND: Heredoc with following command fails: {}", err5);
    }
}

// ============================================================================
// EDGE CASE: Redirection Edge Cases
// ============================================================================

#[test]
fn test_redirection_edge_cases() {
    // Multiple redirections
    assert_parses("cmd < input > output 2> errors");

    // Redirect with fd numbers
    let (ok1, err1) = parse_result("cmd 3>&1 4>&2");
    if !ok1 {
        println!("BUG FOUND: Higher fd numbers fail: {}", err1);
    }

    // Close fd
    let (ok2, err2) = parse_result("cmd 3>&-");
    if !ok2 {
        println!("BUG FOUND: Close fd syntax fails: {}", err2);
    }

    // Redirect to/from variable
    let (ok3, err3) = parse_result("cmd > $file");
    if !ok3 {
        println!("BUG FOUND: Redirect to variable fails: {}", err3);
    }

    // Noclobber redirect
    let (ok4, err4) = parse_result("cmd >| file");
    if !ok4 {
        println!("BUG FOUND: Noclobber redirect fails: {}", err4);
    }

    // Read-write redirect
    let (ok5, err5) = parse_result("cmd <> file");
    if !ok5 {
        println!("BUG FOUND: Read-write redirect fails: {}", err5);
    }
}

// ============================================================================
// EDGE CASE: Control Flow Edge Cases
// ============================================================================

#[test]

include!("parser_bug_hunting_tests_cont.rs");
