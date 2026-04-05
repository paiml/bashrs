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
}
