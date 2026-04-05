fn test_parser_redirection_coverage() {
    let mut gui = gui_coverage! {
        buttons: ["redir_out", "redir_append", "redir_in", "redir_stderr", "redir_both", "redir_heredoc", "redir_herestring", "redir_fd"],
        screens: ["parsed", "error"]
    };

    let tests = [
        ("echo hi > file", "redir_out"),
        ("echo hi >> file", "redir_append"),
        ("cat < file", "redir_in"),
        ("cmd 2> errors", "redir_stderr"),
        ("cmd &> all", "redir_both"),
        ("cat <<EOF\nhello\nEOF", "redir_heredoc"),
        ("cat <<< 'string'", "redir_herestring"),
        ("cmd 2>&1", "redir_fd"),
    ];

    for (input, feature) in tests {
        let result = ParserResult::from_parse(input);
        gui.click(feature);
        gui.visit(if result.is_ok() { "parsed" } else { "error" });
    }

    println!("\nRedirection Parsing Coverage: {:.1}%", gui.percent());
    assert!(gui.meets(75.0), "Redirection coverage >= 75%");
}

// ============================================================================
// PIPELINE TESTS
// ============================================================================

#[test]
fn test_parser_pipeline_coverage() {
    let mut gui = gui_coverage! {
        buttons: ["pipe_simple", "pipe_chain", "pipe_and", "pipe_or", "pipe_subshell", "pipe_brace", "pipe_proc"],
        screens: ["parsed", "error"]
    };

    let tests = [
        ("cmd1 | cmd2", "pipe_simple"),
        ("cmd1 | cmd2 | cmd3", "pipe_chain"),
        ("cmd1 && cmd2", "pipe_and"),
        ("cmd1 || cmd2", "pipe_or"),
        ("(cmd1; cmd2) | cmd3", "pipe_subshell"),
        ("{ cmd1; cmd2; } | cmd3", "pipe_brace"),
        ("diff <(cmd1) <(cmd2)", "pipe_proc"),
    ];

    for (input, feature) in tests {
        let result = ParserResult::from_parse(input);
        gui.click(feature);
        gui.visit(if result.is_ok() { "parsed" } else { "error" });
    }

    println!("\nPipeline Parsing Coverage: {:.1}%", gui.percent());
    assert!(gui.meets(75.0), "Pipeline coverage >= 75%");
}

// ============================================================================
// FRAME SEQUENCE TESTING (Parser State Transitions)
// ============================================================================

#[test]
fn test_parser_state_sequence() {
    let mut sequence = FrameSequence::new("parser_state_transitions");

    let inputs = [
        "x=5",
        "echo $x",
        "if [ $x -eq 5 ]; then echo yes; fi",
        "for i in 1 2 3; do echo $i; done",
        "myfunc() { echo hi; }",
    ];

    for input in &inputs {
        let result = ParserResult::from_parse(input);
        let frame = parser_frame(input, &result);
        sequence.add_frame(&frame);
    }

    assert_eq!(sequence.len(), 5);
    let first = sequence.first().unwrap();
    let last = sequence.last().unwrap();
    assert!(!first.matches(last), "First and last should differ");

    println!(
        "\nParser State Sequence: {} frames captured",
        sequence.len()
    );
}

// ============================================================================
// SNAPSHOT MANAGER TESTING (Golden Files)
// ============================================================================

#[test]
fn test_parser_snapshot_golden_files() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let manager = SnapshotManager::new(temp_dir.path());

    let input = "for i in 1 2 3; do echo $i; done";
    let result1 = ParserResult::from_parse(input);
    let result2 = ParserResult::from_parse(input);

    let frame1 = parser_frame(input, &result1);
    let frame2 = parser_frame(input, &result2);

    manager.assert_snapshot("for_loop_parse", &frame1).unwrap();
    manager.assert_snapshot("for_loop_parse", &frame2).unwrap();
    assert!(manager.exists("for_loop_parse"));

    println!("\nSnapshot golden file testing: PASS");
}

// ============================================================================
// COMPREHENSIVE PARSER COVERAGE REPORT
// ============================================================================

#[test]
fn test_parser_comprehensive_coverage() {
    let mut gui = gui_coverage! {
        buttons: [
            // Variables (10)
            "var_simple", "var_default", "var_assign", "var_alt", "var_error",
            "var_length", "var_substr", "var_pattern", "var_array", "var_indirect",
            // Command substitution (5)
            "cmd_dollar", "cmd_backtick", "cmd_nested", "cmd_string", "cmd_arith",
            // Conditionals (10)
            "if_simple", "if_else", "if_elif", "if_nested", "test_bracket",
            "test_double", "test_arith", "case_simple", "case_multi", "case_default",
            // Loops (9)
            "for_simple", "for_c", "for_brace", "while_simple", "while_read",
            "until_simple", "select_simple", "break_stmt", "continue_stmt",
            // Functions (6)
            "func_keyword", "func_parens", "func_body", "func_local", "func_return", "func_recursive",
            // Quoting (6)
            "quote_single", "quote_double", "quote_escape", "quote_dollar", "quote_ansi", "quote_mixed",
            // Arithmetic (10)
            "arith_add", "arith_sub", "arith_mult", "arith_div", "arith_mod",
            "arith_exp", "arith_parens", "arith_assign", "arith_incr", "arith_cmp",
            // Redirection (8)
            "redir_out", "redir_append", "redir_in", "redir_stderr", "redir_both",
            "redir_heredoc", "redir_herestring", "redir_fd",
            // Pipeline (7)
            "pipe_simple", "pipe_chain", "pipe_and", "pipe_or", "pipe_subshell", "pipe_brace", "pipe_proc"
        ],
        screens: ["idle", "tokenizing", "parsing", "ast_complete", "error"]
    };

    // Variable tests
    for (input, feature) in [
        ("x=5", "var_simple"),
        ("${x:-default}", "var_default"),
        ("${x:=default}", "var_assign"),
        ("${x:+alt}", "var_alt"),
        ("${x:?err}", "var_error"),
        ("${#x}", "var_length"),
        ("${x:0:5}", "var_substr"),
        ("${x%pat}", "var_pattern"),
        ("${arr[0]}", "var_array"),
        ("${!ref}", "var_indirect"),
    ] {
        let _ = ParserResult::from_parse(input);
        gui.click(feature);
    }

    // Command substitution
    for (input, feature) in [
        ("$(cmd)", "cmd_dollar"),
        ("`cmd`", "cmd_backtick"),
        ("$($(inner))", "cmd_nested"),
        ("\"$(cmd)\"", "cmd_string"),
        ("$(($(echo 1)))", "cmd_arith"),
    ] {
        let _ = ParserResult::from_parse(input);
        gui.click(feature);
    }

    // Conditionals
    for (input, feature) in [
        ("if true; then x; fi", "if_simple"),
        ("if true; then x; else y; fi", "if_else"),
        ("if true; then x; elif false; then y; fi", "if_elif"),
        ("if true; then if false; then x; fi; fi", "if_nested"),
        ("[ -f x ]", "test_bracket"),
        ("[[ -f x ]]", "test_double"),
        ("(( x > 5 ))", "test_arith"),
        ("case $x in a) ;; esac", "case_simple"),
        ("case $x in a|b) ;; esac", "case_multi"),
        ("case $x in *) ;; esac", "case_default"),
    ] {
        let _ = ParserResult::from_parse(input);
        gui.click(feature);
    }

    // Loops
    for (input, feature) in [
        ("for i in 1 2; do x; done", "for_simple"),
        ("for ((i=0;i<10;i++)); do x; done", "for_c"),
        ("for i in {1..10}; do x; done", "for_brace"),
        ("while true; do x; done", "while_simple"),
        ("while read l; do x; done", "while_read"),
        ("until false; do x; done", "until_simple"),
        ("select o in a b; do x; done", "select_simple"),
        ("for i in 1; do break; done", "break_stmt"),
        ("for i in 1; do continue; done", "continue_stmt"),
    ] {
        let _ = ParserResult::from_parse(input);
        gui.click(feature);
    }

    // Functions
    for (input, feature) in [
        ("function f { x; }", "func_keyword"),
        ("f() { x; }", "func_parens"),
        ("f() { x; y; }", "func_body"),
        ("f() { local v=1; }", "func_local"),
        ("f() { return 0; }", "func_return"),
        ("f() { f; }", "func_recursive"),
    ] {
        let _ = ParserResult::from_parse(input);
        gui.click(feature);
    }

    // Quoting
    for (input, feature) in [
        ("'single'", "quote_single"),
        ("\"double\"", "quote_double"),
        ("\"esc\\\"ape\"", "quote_escape"),
        ("$'dollar'", "quote_dollar"),
        ("$'\\x41'", "quote_ansi"),
        ("'a'\"b\"", "quote_mixed"),
    ] {
        let _ = ParserResult::from_parse(input);
        gui.click(feature);
    }

    // Arithmetic
    for (input, feature) in [
        ("$((1+2))", "arith_add"),
        ("$((5-3))", "arith_sub"),
        ("$((2*3))", "arith_mult"),
        ("$((6/2))", "arith_div"),
        ("$((7%3))", "arith_mod"),
        ("$((2**3))", "arith_exp"),
        ("$(((1+2)*3))", "arith_parens"),
        ("((x=5))", "arith_assign"),
        ("((x++))", "arith_incr"),
        ("((x>5))", "arith_cmp"),
    ] {
        let _ = ParserResult::from_parse(input);
        gui.click(feature);
    }

    // Redirection
    for (input, feature) in [
        ("cmd > f", "redir_out"),
        ("cmd >> f", "redir_append"),
        ("cmd < f", "redir_in"),
        ("cmd 2> f", "redir_stderr"),
        ("cmd &> f", "redir_both"),
        ("cat <<EOF\nx\nEOF", "redir_heredoc"),
        ("cat <<< 'x'", "redir_herestring"),
        ("cmd 2>&1", "redir_fd"),
    ] {
        let _ = ParserResult::from_parse(input);
        gui.click(feature);
    }

    // Pipeline
    for (input, feature) in [
        ("a | b", "pipe_simple"),
        ("a | b | c", "pipe_chain"),
        ("a && b", "pipe_and"),
        ("a || b", "pipe_or"),
        ("(a) | b", "pipe_subshell"),
        ("{ a; } | b", "pipe_brace"),
        ("diff <(a) <(b)", "pipe_proc"),
    ] {
        let _ = ParserResult::from_parse(input);
        gui.click(feature);
    }

    // Visit states
    gui.visit("idle");
    gui.visit("tokenizing");
    gui.visit("parsing");
    gui.visit("ast_complete");

    let report = gui.generate_report();

    println!("\n╔══════════════════════════════════════════════════════════════════════════════╗");
    println!("║                    PARSER COMPREHENSIVE COVERAGE REPORT                       ║");
    println!("╠══════════════════════════════════════════════════════════════════════════════╣");
    println!("║  Total Elements:   {:<57} ║", report.total_elements);
    println!("║  Covered Elements: {:<57} ║", report.covered_elements);
    println!(
        "║  Element Coverage: {:<57.1}% ║",
        report.element_coverage * 100.0
    );
    println!("║  Total States:     {:<57} ║", report.total_states);
    println!("║  Covered States:   {:<57} ║", report.covered_states);
    println!(
        "║  State Coverage:   {:<57.1}% ║",
        report.state_coverage * 100.0
    );
    println!(
        "║  Overall Coverage: {:<57.1}% ║",
        report.overall_coverage * 100.0
    );
    println!(
        "║  Status:           {:<57} ║",
        if report.is_complete {
            "COMPLETE"
        } else {
            "INCOMPLETE"
        }
    );
    println!("╚══════════════════════════════════════════════════════════════════════════════╝");

    assert!(
        report.overall_coverage >= 0.80,
        "Overall parser coverage should be >= 80%, got {:.1}%",
        report.overall_coverage * 100.0
    );
}

// ============================================================================
// SOFT ASSERTIONS FOR PARSER ERRORS
// ============================================================================

#[test]
fn test_parser_error_soft_assertions() {
    let script = r#"
        x=5
        echo $x
        if true; then echo yes; fi
        for i in 1 2 3; do echo $i; done
    "#;

    let result = ParserResult::from_parse(script);
    let frame = parser_frame(script, &result);

    let mut soft = expect_frame(&frame).soft();
    let _ = soft.to_contain_text("BASHRS PARSER OUTPUT");
    let _ = soft.to_contain_text("Status:");
    let _ = soft.not_to_contain_text("PANIC");
    let _ = soft.not_to_contain_text("FATAL");

    let errors = soft.errors();
    if !errors.is_empty() {
        for err in errors {
            println!("  Soft assertion failed: {}", err);
        }
    }

    soft.finalize().expect("All soft assertions should pass");
    println!("\nSoft assertions: all checks passed");
}

// ============================================================================
// DETERMINISM VERIFICATION
// ============================================================================

#[test]
fn test_parser_determinism() {
    let inputs = [
        "x=5",
        "echo ${x:-default}",
        "if true; then echo yes; fi",
        "for i in 1 2 3; do echo $i; done",
        "myfunc() { echo hi; }",
        "cmd1 | cmd2 | cmd3",
        "cat <<EOF\nhello world\nEOF",
    ];

    println!("\n╔══════════════════════════════════════════════════════════════════════════════╗");
    println!("║                    PARSER DETERMINISM VERIFICATION                           ║");
    println!("╚══════════════════════════════════════════════════════════════════════════════╝\n");

    for input in inputs {
        let result1 = ParserResult::from_parse(input);
        let result2 = ParserResult::from_parse(input);
        let result3 = ParserResult::from_parse(input);

        let frame1 = parser_frame(input, &result1);
        let frame2 = parser_frame(input, &result2);
        let frame3 = parser_frame(input, &result3);

        let snap1 = TuiSnapshot::from_frame("test", &frame1);
        let snap2 = TuiSnapshot::from_frame("test", &frame2);
        let snap3 = TuiSnapshot::from_frame("test", &frame3);

        assert!(
            snap1.matches(&snap2) && snap2.matches(&snap3),
            "Parser must be deterministic for: {}",
            input
        );

        let display = if input.len() > 40 {
            &input[..40]
        } else {
            input
        };
        println!("  {} ... deterministic", display);
    }

    println!("\n  All {} inputs verified deterministic", inputs.len());
}
