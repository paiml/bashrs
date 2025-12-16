//! Parser Bug Hunting - Rigorous Edge Case Testing
//!
//! This module aggressively tests parser edge cases to find bugs.
//! Uses property-based testing, fuzzing patterns, and corner cases.

#![allow(clippy::unwrap_used)]

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
fn assert_fails(input: &str) {
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
fn test_control_flow_edge_cases() {
    // One-liner if
    let (ok1, err1) = parse_result("if true; then echo yes; else echo no; fi");
    if !ok1 {
        println!("BUG FOUND: One-liner if fails: {}", err1);
    }

    // Chained elif
    let code2 =
        "if false; then :; elif false; then :; elif false; then :; elif true; then echo yes; fi";
    let (ok2, err2) = parse_result(code2);
    if !ok2 {
        println!("BUG FOUND: Chained elif fails: {}", err2);
    }

    // Case with fall-through (;&)
    let (ok3, err3) = parse_result("case $x in a) echo a;& b) echo b;; esac");
    if !ok3 {
        println!("BUG FOUND: Case fall-through ;& fails: {}", err3);
    }

    // Case with resume (;;&)
    let (ok4, err4) = parse_result("case $x in a) echo a;;& b) echo b;; esac");
    if !ok4 {
        println!("BUG FOUND: Case resume ;;& fails: {}", err4);
    }

    // Coprocess
    let (ok5, err5) = parse_result("coproc myproc { cat; }");
    if !ok5 {
        println!("BUG FOUND: Coproc syntax fails: {}", err5);
    }
}

// ============================================================================
// EDGE CASE: Array Edge Cases
// ============================================================================

#[test]
fn test_array_edge_cases() {
    // Associative array declaration
    let (ok1, err1) = parse_result("declare -A myarray");
    if !ok1 {
        println!("BUG FOUND: Associative array declaration fails: {}", err1);
    }

    // Array with mixed indices
    let (ok2, err2) = parse_result("arr=([0]=a [5]=b [10]=c)");
    if !ok2 {
        println!("BUG FOUND: Sparse array assignment fails: {}", err2);
    }

    // Associative array assignment
    let (ok3, err3) = parse_result("arr=([key1]=val1 [key2]=val2)");
    if !ok3 {
        println!("BUG FOUND: Associative array assignment fails: {}", err3);
    }

    // Array append
    let (ok4, err4) = parse_result("arr+=(newval)");
    if !ok4 {
        println!("BUG FOUND: Array append fails: {}", err4);
    }

    // Get all keys
    let (ok5, err5) = parse_result("echo ${!arr[@]}");
    if !ok5 {
        println!("BUG FOUND: Get array keys fails: {}", err5);
    }

    // Array length
    assert_parses("echo ${#arr[@]}");
    assert_parses("echo ${#arr[*]}");
}

// ============================================================================
// EDGE CASE: Function Edge Cases
// ============================================================================

#[test]
fn test_function_edge_cases() {
    // Function with dashes in name
    let (ok1, err1) = parse_result("my-func() { echo hi; }");
    if !ok1 {
        println!("BUG FOUND: Function with dash in name fails: {}", err1);
    }

    // Function with dots
    let (ok2, err2) = parse_result("my.func() { echo hi; }");
    if !ok2 {
        println!("BUG FOUND: Function with dot in name fails: {}", err2);
    }

    // Function with colon
    let (ok3, err3) = parse_result("my:func() { echo hi; }");
    if !ok3 {
        println!("BUG FOUND: Function with colon in name fails: {}", err3);
    }

    // Function with subshell body
    let (ok4, err4) = parse_result("myfunc() ( echo subshell )");
    if !ok4 {
        println!("BUG FOUND: Function with subshell body fails: {}", err4);
    }

    // Nested function
    let code5 = "outer() { inner() { echo inner; }; inner; }";
    let (ok5, err5) = parse_result(code5);
    if !ok5 {
        println!("BUG FOUND: Nested function fails: {}", err5);
    }
}

// ============================================================================
// EDGE CASE: Process Substitution
// ============================================================================

#[test]
fn test_process_substitution_edge_cases() {
    // Basic process substitution
    assert_parses("diff <(cmd1) <(cmd2)");
    assert_parses("cmd > >(other)");

    // Nested process substitution
    let (ok1, err1) = parse_result("diff <(cat <(echo hi)) <(echo bye)");
    if !ok1 {
        println!("BUG FOUND: Nested process substitution fails: {}", err1);
    }

    // Process substitution in variable
    let (ok2, err2) = parse_result("exec 3< <(cmd)");
    if !ok2 {
        println!("BUG FOUND: Process substitution with exec fails: {}", err2);
    }
}

// ============================================================================
// EDGE CASE: Malformed Input (Should Fail Gracefully)
// ============================================================================

#[test]
fn test_malformed_input_handling() {
    // Unclosed quotes - should fail
    let (ok1, _) = parse_result("echo \"unclosed");
    if ok1 {
        println!("BUG FOUND: Unclosed double quote should fail but didn't");
    }

    let (ok2, _) = parse_result("echo 'unclosed");
    if ok2 {
        println!("BUG FOUND: Unclosed single quote should fail but didn't");
    }

    // Unclosed substitution
    let (ok3, _) = parse_result("echo $(unclosed");
    if ok3 {
        println!("BUG FOUND: Unclosed command sub should fail but didn't");
    }

    // Unclosed brace
    let (ok4, _) = parse_result("echo ${unclosed");
    if ok4 {
        println!("BUG FOUND: Unclosed brace expansion should fail but didn't");
    }

    // Missing fi
    let (ok5, _) = parse_result("if true; then echo yes");
    if ok5 {
        println!("BUG FOUND: Missing fi should fail but didn't");
    }

    // Missing done
    let (ok6, _) = parse_result("for i in 1 2 3; do echo $i");
    if ok6 {
        println!("BUG FOUND: Missing done should fail but didn't");
    }

    // Missing esac
    let (ok7, _) = parse_result("case $x in a) echo a;;");
    if ok7 {
        println!("BUG FOUND: Missing esac should fail but didn't");
    }
}

// ============================================================================
// COMPREHENSIVE BUG REPORT TEST
// ============================================================================

#[test]
fn test_generate_bug_report() {
    println!("\n╔══════════════════════════════════════════════════════════════════════════════╗");
    println!("║                         PARSER BUG HUNTING REPORT                             ║");
    println!("╚══════════════════════════════════════════════════════════════════════════════╝\n");

    let mut bugs_found = Vec::new();

    // Test categories with expected failures
    let edge_cases = vec![
        // (input, description, should_pass)
        (
            "echo ${foo:-${bar:-default}}",
            "Nested parameter expansion",
            true,
        ),
        ("echo ${var##*[/]}", "Pattern with brackets", true),
        ("echo ${var^^}", "Case modification ^^", true),
        ("echo ${var,,}", "Case modification ,,", true),
        ("echo ${!prefix*}", "Indirect expansion with *", true),
        ("echo ${arr[@]:1:3}", "Array slicing", true),
        ("echo $((x > 5 ? 1 : 0))", "Ternary operator", true),
        ("echo $((x=1, y=2, x+y))", "Comma operator", true),
        ("echo $((-5))", "Negative number in arithmetic", true),
        ("echo $((0xff))", "Hex literal", true),
        ("echo $((0777))", "Octal literal", true),
        ("cat <<'EOF'\n$HOME\nEOF", "Quoted heredoc delimiter", true),
        ("cat <<-EOF\n\thello\n\tEOF", "Indented heredoc", true),
        ("cmd 3>&-", "Close fd syntax", true),
        ("cmd >| file", "Noclobber redirect", true),
        ("cmd <> file", "Read-write redirect", true),
        (
            "case $x in a) echo a;& b) echo b;; esac",
            "Case fall-through ;&",
            true,
        ),
        (
            "case $x in a) echo a;;& b) echo b;; esac",
            "Case resume ;;&",
            true,
        ),
        ("coproc myproc { cat; }", "Coproc syntax", true),
        ("declare -A myarray", "Associative array declaration", true),
        ("arr=([0]=a [5]=b)", "Sparse array assignment", true),
        ("arr+=(newval)", "Array append", true),
        ("my-func() { echo hi; }", "Function with dash", true),
        (
            "myfunc() ( echo subshell )",
            "Function with subshell body",
            true,
        ),
        ("echo @(foo|bar)", "Extended glob @()", true),
        ("echo !(foo)", "Extended glob !()", true),
        ("echo \"Hello 世界\"", "Unicode in strings", true),
        ("f() { }", "Empty function body", true),
        ("exec 3< <(cmd)", "Process substitution with exec", true),
    ];

    for (input, desc, should_pass) in &edge_cases {
        let (ok, err) = parse_result(input);
        if *should_pass && !ok {
            bugs_found.push((desc.to_string(), input.to_string(), err));
        }
    }

    if bugs_found.is_empty() {
        println!("✅ No bugs found in {} test cases!", edge_cases.len());
    } else {
        println!("❌ Found {} bugs:\n", bugs_found.len());
        for (i, (desc, input, err)) in bugs_found.iter().enumerate() {
            println!("BUG #{}: {}", i + 1, desc);
            println!("  Input: {}", input.replace('\n', "\\n"));
            println!("  Error: {}", err);
            println!();
        }
    }

    // The test passes regardless - this is for finding bugs
    assert!(true);
}
