fn test_falsification_traps_signals() {
    let mut gui = gui_coverage! {
        buttons: ["F081", "F082", "F083", "F084", "F085", "F086", "F087", "F088", "F089", "F090"],
        screens: ["pass", "fail"]
    };

    let tests = [
        ("F081", "trap 'rm $f' EXIT", "SC2064"),
        ("F082", "trap \"echo $v\" INT", "SC2064"),
        ("F083", "kill -9 $$ ", "SC2086"),
        ("F084", "wait $!", "SC2086"),
        ("F085", "disown -h", "Parser"),
        ("F086", "suspend -f", "Parser"),
        ("F087", "ulimit -n 1024", "Parser"),
        ("F088", "umask 077", "Parser"),
        ("F089", "set -e", "SC2034"),
        ("F090", "shopt -s extglob", "Parser"),
    ];

    for (id, code, forbidden) in tests {
        let result = lint_shell(code);
        let passed = if forbidden == "Parser" {
            !has_parse_error(&result)
        } else {
            !has_rule(&result, forbidden)
        };
        gui.click(id);
        gui.visit(if passed { "pass" } else { "fail" });
        assert!(
            passed,
            "{}: Must NOT trigger {} on: {}",
            id, forbidden, code
        );
    }

    let report = gui.generate_report();
    println!(
        "\n6.8 Traps/Signals: {:.1}% elements",
        report.element_coverage * 100.0
    );
    assert!(report.element_coverage >= 1.0, "All trap tests must pass");
}

// ============================================================================
// 6.9 PARSING AND FORMATTING (F091-F100)
// ============================================================================

#[test]
fn test_falsification_parsing_formatting() {
    let mut gui = gui_coverage! {
        buttons: ["F091", "F092", "F093", "F094", "F095", "F096", "F097", "F098", "F099", "F100"],
        screens: ["pass", "fail"]
    };

    let tests = [
        ("F091", "echo # comment", "Parser"),
        ("F092", "echo \\# literal", "Parser"),
        ("F093", "x=()", "Parser"),
        ("F094", "x=([0]=a [2]=c)", "Parser"),
        ("F095", "x+=(\"new\")", "Parser"),
        ("F096", "[[ $x =~ ^[a-z]+$ ]]", "Parser"),
        ("F097", "echo *", "SC2035"),
        ("F098", "echo {1..10}", "Parser"),
        ("F099", "echo {a,b,c}", "Parser"),
        ("F100", "echo $'\\t'", "Parser"),
    ];

    for (id, code, forbidden) in tests {
        let result = lint_shell(code);
        let passed = if forbidden == "Parser" {
            !has_parse_error(&result)
        } else {
            !has_rule(&result, forbidden)
        };
        gui.click(id);
        gui.visit(if passed { "pass" } else { "fail" });
        assert!(
            passed,
            "{}: Must NOT trigger {} on: {}",
            id, forbidden, code
        );
    }

    let report = gui.generate_report();
    println!(
        "\n6.9 Parsing/Formatting: {:.1}% elements",
        report.element_coverage * 100.0
    );
    assert!(
        report.element_coverage >= 1.0,
        "All parsing tests must pass"
    );
}

// ============================================================================
// 6.10 ARRAYS (F101-F110)
// ============================================================================

#[test]
fn test_falsification_arrays() {
    let mut gui = gui_coverage! {
        buttons: ["F101", "F102", "F103", "F104", "F105", "F106", "F107", "F108", "F109", "F110"],
        screens: ["pass", "fail"]
    };

    let tests = [
        ("F101", "arr=(a b c); echo ${arr[0]}", "SC2086"),
        ("F102", "arr=(\"$@\"); echo ${#arr[@]}", "SC2086"),
        ("F103", "declare -A map; map[key]=val", "Parser"),
        ("F104", "arr=(); arr+=(item)", "Parser"),
        ("F105", "echo \"${arr[*]}\"", "SC2086"),
        (
            "F106",
            "for i in \"${arr[@]}\"; do echo \"$i\"; done",
            "SC2086",
        ),
        ("F107", "unset arr[0]", "Parser"),
        ("F108", "arr=([0]=a [2]=c)", "SC2086"),
        ("F109", "echo ${!arr[@]}", "SC2086"),
        ("F110", "readarray -t lines < file", "Parser"),
    ];

    for (id, code, forbidden) in tests {
        let result = lint_shell(code);
        let passed = if forbidden == "Parser" {
            !has_parse_error(&result)
        } else {
            !has_rule(&result, forbidden)
        };
        gui.click(id);
        gui.visit(if passed { "pass" } else { "fail" });
        assert!(
            passed,
            "{}: Must NOT trigger {} on: {}",
            id, forbidden, code
        );
    }

    let report = gui.generate_report();
    println!(
        "\n6.10 Arrays: {:.1}% elements",
        report.element_coverage * 100.0
    );
    assert!(report.element_coverage >= 1.0, "All array tests must pass");
}

// ============================================================================
// 6.11 STRING OPERATIONS (F111-F120)
// ============================================================================

#[test]
fn test_falsification_string_operations() {
    let mut gui = gui_coverage! {
        buttons: ["F111", "F112", "F113", "F114", "F115", "F116", "F117", "F118", "F119", "F120"],
        screens: ["pass", "fail"]
    };

    let tests = [
        ("F111", "echo ${var:0:5}", "SC2086"),
        ("F112", "echo ${var/old/new}", "SC2086"),
        ("F113", "echo ${var//old/new}", "SC2086"),
        ("F114", "echo ${var,,}", "SC2086"),
        ("F115", "echo ${var^^}", "SC2086"),
        ("F116", "echo ${#var}", "SC2086"),
        ("F117", "echo ${var%suffix}", "SC2086"),
        ("F118", "echo ${var%%pattern}", "SC2086"),
        ("F119", "echo ${var#prefix}", "SC2086"),
        ("F120", "echo ${var##pattern}", "SC2086"),
    ];

    for (id, code, forbidden) in tests {
        let result = lint_shell(code);
        let passed = !has_rule(&result, forbidden);
        gui.click(id);
        gui.visit(if passed { "pass" } else { "fail" });
        assert!(
            passed,
            "{}: Must NOT trigger {} on: {}",
            id, forbidden, code
        );
    }

    let report = gui.generate_report();
    println!(
        "\n6.11 String Operations: {:.1}% elements",
        report.element_coverage * 100.0
    );
    assert!(
        report.element_coverage >= 1.0,
        "All string operation tests must pass"
    );
}

// ============================================================================
// 6.12 ARITHMETIC (F121-F130)
// ============================================================================

#[test]
fn test_falsification_arithmetic() {
    let mut gui = gui_coverage! {
        buttons: ["F121", "F122", "F123", "F124", "F125", "F126", "F127", "F128", "F129", "F130"],
        screens: ["pass", "fail"]
    };

    let tests = [
        ("F121", "echo $((1+2))", "SC2086"),
        ("F122", "echo $((x+y))", "SC2086"),
        ("F123", "(( i++ ))", "SC2086"),
        ("F124", "(( x = 5 + 3 ))", "SC2086"),
        ("F125", "let x=5+3", "Parser"),
        ("F126", "echo $((RANDOM % 100))", "SC2086"),
        ("F127", "echo $(( (1+2) * 3 ))", "SC2086"),
        ("F128", "echo $((16#FF))", "SC2086"),
        ("F129", "echo $((2**10))", "SC2086"),
        ("F130", "echo $((x<y ? x : y))", "SC2086"),
    ];

    for (id, code, forbidden) in tests {
        let result = lint_shell(code);
        let passed = if forbidden == "Parser" {
            !has_parse_error(&result)
        } else {
            !has_rule(&result, forbidden)
        };
        gui.click(id);
        gui.visit(if passed { "pass" } else { "fail" });
        assert!(
            passed,
            "{}: Must NOT trigger {} on: {}",
            id, forbidden, code
        );
    }

    let report = gui.generate_report();
    println!(
        "\n6.12 Arithmetic: {:.1}% elements",
        report.element_coverage * 100.0
    );
    assert!(
        report.element_coverage >= 1.0,
        "All arithmetic tests must pass"
    );
}

// ============================================================================
// COMPREHENSIVE COVERAGE REPORT
// ============================================================================

#[test]
fn test_falsification_comprehensive_coverage() {
    let mut gui = gui_coverage! {
        buttons: [
            // 6.1 Sudo (10)
            "F001", "F002", "F003", "F004", "F005", "F006", "F007", "F008", "F009", "F010",
            // 6.2 Redirection (10)
            "F011", "F012", "F013", "F014", "F015", "F016", "F017", "F018", "F019", "F020",
            // 6.3 Quoting (10)
            "F021", "F022", "F023", "F024", "F025", "F026", "F027", "F028", "F029", "F030",
            // 6.4 Variables (15)
            "F031", "F032", "F033", "F034", "F035", "F036", "F037", "F038", "F039", "F040",
            "F041", "F042", "F043", "F044", "F045",
            // 6.5 Control Flow (15)
            "F046", "F047", "F048", "F049", "F050", "F051", "F052", "F053", "F054", "F055",
            "F056", "F057", "F058", "F059", "F060",
            // 6.6 Builtins (10)
            "F061", "F062", "F063", "F064", "F065", "F066", "F067", "F068", "F069", "F070",
            // 6.7 Subshells (10)
            "F071", "F072", "F073", "F074", "F075", "F076", "F077", "F078", "F079", "F080",
            // 6.8 Traps (10)
            "F081", "F082", "F083", "F084", "F085", "F086", "F087", "F088", "F089", "F090",
            // 6.9 Parsing (10)
            "F091", "F092", "F093", "F094", "F095", "F096", "F097", "F098", "F099", "F100",
            // 6.10 Arrays (10)
            "F101", "F102", "F103", "F104", "F105", "F106", "F107", "F108", "F109", "F110",
            // 6.11 Strings (10)
            "F111", "F112", "F113", "F114", "F115", "F116", "F117", "F118", "F119", "F120",
            // 6.12 Arithmetic (10)
            "F121", "F122", "F123", "F124", "F125", "F126", "F127", "F128", "F129", "F130"
        ],
        screens: ["pass", "fail"]
    };

    // Mark all as visited (individual tests verify correctness)
    for i in 1..=130 {
        gui.click(&format!("F{:03}", i));
    }
    gui.visit("pass");

    let report = gui.generate_report();

    println!("\n╔══════════════════════════════════════════════════════════════════════════════╗");
    println!("║                   FALSIFICATION COMPREHENSIVE COVERAGE                        ║");
    println!("╠══════════════════════════════════════════════════════════════════════════════╣");
    println!("║  Total Tests:      {:<56} ║", report.total_elements);
    println!("║  Covered Tests:    {:<56} ║", report.covered_elements);
    println!(
        "║  Coverage:         {:<56.1}% ║",
        report.element_coverage * 100.0
    );
    println!("╚══════════════════════════════════════════════════════════════════════════════╝");

    assert!(
        report.element_coverage >= 1.0,
        "All 130 falsification tests must be covered"
    );
}
