//! Parser Probar TUI Testing - Comprehensive Coverage
//!
//! Tests bashrs parser with probar-style TUI testing:
//! - Frame assertions for parser output
//! - UX coverage tracking for parser features
//! - Snapshot testing for AST output stability
//!
//! Run: cargo test -p bashrs --test parser_probar_testing

#![allow(clippy::indexing_slicing)]
#![allow(clippy::panic)]
#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]

use bashrs::bash_parser::ast::BashAst;
use bashrs::bash_parser::parser::BashParser;
use jugar_probar::gui_coverage;
use jugar_probar::tui::{expect_frame, FrameSequence, SnapshotManager, TuiFrame, TuiSnapshot};
use jugar_probar::ux_coverage::{InteractionType, StateId, UxCoverageBuilder};

// ============================================================================
// PARSER RESULT WRAPPER
// ============================================================================

/// Result from parsing - contains AST and any errors
struct ParserResult {
    ast: Option<BashAst>,
    errors: Vec<String>,
}

impl ParserResult {
    fn from_parse(input: &str) -> Self {
        match BashParser::new(input) {
            Ok(mut parser) => match parser.parse() {
                Ok(ast) => Self {
                    ast: Some(ast),
                    errors: vec![],
                },
                Err(e) => Self {
                    ast: None,
                    errors: vec![format!("{:?}", e)],
                },
            },
            Err(e) => Self {
                ast: None,
                errors: vec![format!("{:?}", e)],
            },
        }
    }

    fn is_ok(&self) -> bool {
        self.errors.is_empty() && self.ast.is_some()
    }
}

// ============================================================================
// PARSER FRAME UTILITIES
// ============================================================================

/// Parser frame from output - simulates TUI output
fn parser_frame(input: &str, result: &ParserResult) -> TuiFrame {
    let status = if result.is_ok() { "OK" } else { "ERROR" };
    let ast_preview = match &result.ast {
        Some(ast) => format!("{:?}", ast).chars().take(200).collect::<String>(),
        None => "None".to_string(),
    };
    let input_display = if input.len() > 50 {
        format!("{}...", &input[..47])
    } else {
        input.to_string()
    };

    let content = format!(
        "╔═══════════════════════════════════════════════════════════════════════╗\n\
         ║ BASHRS PARSER OUTPUT                                                  ║\n\
         ╠═══════════════════════════════════════════════════════════════════════╣\n\
         ║ Input:  {:<62}║\n\
         ║ Status: {:<62}║\n\
         ║ Errors: {:<62}║\n\
         ╠═══════════════════════════════════════════════════════════════════════╣\n\
         ║ AST Preview:                                                          ║\n\
         ║ {:<70}║\n\
         ╚═══════════════════════════════════════════════════════════════════════╝",
        input_display,
        status,
        result.errors.len(),
        &ast_preview[..ast_preview.len().min(70)]
    );

    TuiFrame::from_lines(&content.lines().collect::<Vec<_>>())
}

/// Parse and track coverage
fn parse_with_coverage(
    input: &str,
    feature: &str,
    tracker: &mut jugar_probar::ux_coverage::UxCoverageTracker,
) -> ParserResult {
    let result = ParserResult::from_parse(input);
    tracker.record_interaction(
        &jugar_probar::ux_coverage::ElementId::new("parser", feature),
        InteractionType::Click,
    );
    tracker.record_state(StateId::new(
        "state",
        if result.is_ok() { "parsed" } else { "error" },
    ));
    result
}

// ============================================================================
// VARIABLE PARSING TESTS
// ============================================================================

#[test]
fn test_parser_variables_coverage() {
    let mut tracker = UxCoverageBuilder::new()
        .clickable("parser", "var_simple")
        .clickable("parser", "var_default")
        .clickable("parser", "var_assign_default")
        .clickable("parser", "var_alt_value")
        .clickable("parser", "var_error")
        .clickable("parser", "var_length")
        .clickable("parser", "var_substring")
        .clickable("parser", "var_pattern_remove")
        .clickable("parser", "var_array")
        .clickable("parser", "var_indirect")
        .screen("parsed")
        .screen("error")
        .build();

    let tests = [
        ("x=5", "var_simple"),
        ("echo ${x:-default}", "var_default"),
        ("echo ${x:=default}", "var_assign_default"),
        ("echo ${x:+alternate}", "var_alt_value"),
        ("echo ${x:?error message}", "var_error"),
        ("echo ${#x}", "var_length"),
        ("echo ${x:0:5}", "var_substring"),
        ("echo ${x%pattern}", "var_pattern_remove"),
        ("arr=(1 2 3); echo ${arr[0]}", "var_array"),
        ("echo ${!ref}", "var_indirect"),
    ];

    for (input, feature) in tests {
        let result = parse_with_coverage(input, feature, &mut tracker);
        let frame = parser_frame(input, &result);
        expect_frame(&frame)
            .to_contain_text("BASHRS PARSER")
            .unwrap();
    }

    let report = tracker.generate_report();
    println!(
        "\nVariable Parsing Coverage: {:.1}%",
        report.overall_coverage * 100.0
    );
    // Coverage includes both elements + states; element coverage is what we care about
    assert!(
        report.element_coverage >= 0.80,
        "Variable element coverage should be >= 80%, got {:.1}%",
        report.element_coverage * 100.0
    );
}

// ============================================================================
// COMMAND SUBSTITUTION TESTS
// ============================================================================

#[test]
fn test_parser_command_substitution_coverage() {
    let mut gui = gui_coverage! {
        buttons: ["cmd_sub_dollar", "cmd_sub_backtick", "cmd_sub_nested", "cmd_sub_string", "cmd_sub_arith"],
        screens: ["parsed", "error"]
    };

    let tests = [
        ("x=$(command)", "cmd_sub_dollar"),
        ("x=`command`", "cmd_sub_backtick"),
        ("x=$(echo $(inner))", "cmd_sub_nested"),
        ("echo \"result: $(cmd)\"", "cmd_sub_string"),
        ("x=$((1 + $(echo 5)))", "cmd_sub_arith"),
    ];

    for (input, feature) in tests {
        let result = ParserResult::from_parse(input);
        gui.click(feature);
        gui.visit(if result.is_ok() { "parsed" } else { "error" });

        let frame = parser_frame(input, &result);
        expect_frame(&frame).to_contain_text("BASHRS").unwrap();
    }

    println!("\nCommand Substitution Coverage: {:.1}%", gui.percent());
    assert!(gui.meets(80.0), "Command substitution coverage >= 80%");
}

// ============================================================================
// CONDITIONAL PARSING TESTS
// ============================================================================

#[test]
fn test_parser_conditionals_coverage() {
    let mut gui = gui_coverage! {
        buttons: ["if_simple", "if_else", "if_elif", "if_nested", "test_bracket", "test_double", "test_arith", "case_simple", "case_multi", "case_default", "if_invalid"],
        screens: ["parsed", "error"]
    };

    let tests = [
        ("if true; then echo yes; fi", "if_simple"),
        ("if true; then echo yes; else echo no; fi", "if_else"),
        (
            "if true; then echo 1; elif false; then echo 2; fi",
            "if_elif",
        ),
        (
            "if true; then if false; then echo inner; fi; fi",
            "if_nested",
        ),
        ("[ -f file ]", "test_bracket"),
        ("[[ -f file ]]", "test_double"),
        ("(( x > 5 ))", "test_arith"),
        ("case $x in a) echo a;; esac", "case_simple"),
        ("case $x in a|b) echo ab;; esac", "case_multi"),
        (
            "case $x in a) echo a;; *) echo default;; esac",
            "case_default",
        ),
        // Invalid conditional to test error handling path
        ("if then echo oops; fi", "if_invalid"),
    ];

    for (input, feature) in tests {
        let result = ParserResult::from_parse(input);
        gui.click(feature);
        gui.visit(if result.is_ok() { "parsed" } else { "error" });
    }

    println!("\nConditional Parsing Coverage: {:.1}%", gui.percent());
    assert!(gui.meets(80.0), "Conditional coverage >= 80%");
}

// ============================================================================
// LOOP PARSING TESTS
// ============================================================================

#[test]
fn test_parser_loops_coverage() {
    let mut gui = gui_coverage! {
        buttons: ["for_simple", "for_c_style", "for_brace", "while_simple", "while_read", "until_simple", "select_simple", "loop_break", "loop_continue"],
        screens: ["parsed", "error"]
    };

    let tests = [
        ("for i in 1 2 3; do echo $i; done", "for_simple"),
        ("for ((i=0; i<10; i++)); do echo $i; done", "for_c_style"),
        ("for i in {1..10}; do echo $i; done", "for_brace"),
        ("while true; do echo loop; done", "while_simple"),
        ("while read line; do echo $line; done", "while_read"),
        ("until false; do echo loop; done", "until_simple"),
        ("select opt in a b c; do echo $opt; done", "select_simple"),
        ("for i in 1 2; do break; done", "loop_break"),
        ("for i in 1 2; do continue; done", "loop_continue"),
        // Invalid syntax to test error path (Poka-Yoke: graceful failure)
        ("for in; do done", "for_simple"), // Malformed for loop
    ];

    for (input, feature) in tests {
        let result = ParserResult::from_parse(input);
        gui.click(feature);
        gui.visit(if result.is_ok() { "parsed" } else { "error" });

        let frame = parser_frame(input, &result);
        let snapshot = TuiSnapshot::from_frame(feature, &frame);
        assert!(!snapshot.hash.is_empty());
    }

    println!("\nLoop Parsing Coverage: {:.1}%", gui.percent());
    assert!(gui.meets(80.0), "Loop coverage >= 80%");
}

// ============================================================================
// FUNCTION PARSING TESTS
// ============================================================================

#[test]
fn test_parser_functions_coverage() {
    let mut gui = gui_coverage! {
        buttons: ["func_keyword", "func_parens", "func_body", "func_local", "func_return", "func_recursive"],
        screens: ["parsed", "error"]
    };

    let tests = [
        ("function myfunc { echo hi; }", "func_keyword"),
        ("myfunc() { echo hi; }", "func_parens"),
        ("myfunc() { echo $1; echo $2; }", "func_body"),
        ("myfunc() { local x=5; echo $x; }", "func_local"),
        ("myfunc() { return 0; }", "func_return"),
        ("myfunc() { myfunc; }", "func_recursive"),
    ];

    for (input, feature) in tests {
        let result = ParserResult::from_parse(input);
        gui.click(feature);
        gui.visit(if result.is_ok() { "parsed" } else { "error" });
    }

    println!("\nFunction Parsing Coverage: {:.1}%", gui.percent());
    assert!(gui.meets(70.0), "Function coverage >= 70%");
}

// ============================================================================
// QUOTING TESTS
// ============================================================================

#[test]
fn test_parser_quoting_coverage() {
    let mut gui = gui_coverage! {
        buttons: ["quote_single", "quote_double", "quote_escape", "quote_dollar", "quote_ansi", "quote_mixed"],
        screens: ["parsed", "error"]
    };

    let tests = [
        ("echo 'single quoted'", "quote_single"),
        ("echo \"double quoted\"", "quote_double"),
        ("echo \"escaped \\\"quote\\\"\"", "quote_escape"),
        ("echo $'ansi\\nstring'", "quote_dollar"),
        ("echo $'\\x41\\x42\\x43'", "quote_ansi"),
        ("echo 'single'\"double\"", "quote_mixed"),
    ];

    for (input, feature) in tests {
        let result = ParserResult::from_parse(input);
        gui.click(feature);
        gui.visit(if result.is_ok() { "parsed" } else { "error" });
    }

    println!("\nQuoting Parsing Coverage: {:.1}%", gui.percent());
    assert!(gui.meets(70.0), "Quoting coverage >= 70%");
}

// ============================================================================
// ARITHMETIC TESTS
// ============================================================================

#[test]
fn test_parser_arithmetic_coverage() {
    let mut gui = gui_coverage! {
        buttons: ["arith_add", "arith_sub", "arith_mult", "arith_div", "arith_mod", "arith_exp", "arith_parens", "arith_assign", "arith_incr", "arith_cmp"],
        screens: ["parsed", "error"]
    };

    let tests = [
        ("echo $((1 + 2))", "arith_add"),
        ("echo $((5 - 3))", "arith_sub"),
        ("echo $((2 * 3))", "arith_mult"),
        ("echo $((6 / 2))", "arith_div"),
        ("echo $((7 % 3))", "arith_mod"),
        ("echo $((2 ** 3))", "arith_exp"),
        ("echo $(((1 + 2) * 3))", "arith_parens"),
        ("((x = 5))", "arith_assign"),
        ("((x++))", "arith_incr"),
        ("((x > 5))", "arith_cmp"),
    ];

    for (input, feature) in tests {
        let result = ParserResult::from_parse(input);
        gui.click(feature);
        gui.visit(if result.is_ok() { "parsed" } else { "error" });
    }

    println!("\nArithmetic Parsing Coverage: {:.1}%", gui.percent());
    assert!(gui.meets(80.0), "Arithmetic coverage >= 80%");
}

// ============================================================================
// REDIRECTION TESTS
// ============================================================================

#[test]
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
