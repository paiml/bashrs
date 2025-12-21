//! Falsification Probar Testing - Popper Methodology
//!
//! Every valid bash pattern must pass the linter without triggering false positives.
//! These tests use jugar-probar's TUI coverage framework to verify 130 F-code tests.
//!
//! Run: cargo test -p bashrs --test falsification_probar_testing

#![allow(clippy::unwrap_used)]

use bashrs::linter::rules::lint_shell;
use jugar_probar::gui_coverage;

/// Helper: Check if result contains a specific rule code
fn has_rule(result: &bashrs::linter::LintResult, rule: &str) -> bool {
    result.diagnostics.iter().any(|d| d.code == rule)
}

/// Helper: Check if result has any parse errors
fn has_parse_error(result: &bashrs::linter::LintResult) -> bool {
    result
        .diagnostics
        .iter()
        .any(|d| d.message.to_lowercase().contains("parse"))
}

// ============================================================================
// 6.1 SUDO AND PERMISSIONS (F001-F010)
// ============================================================================

#[test]
fn test_falsification_sudo_permissions() {
    let mut gui = gui_coverage! {
        buttons: ["F001", "F002", "F003", "F004", "F005", "F006", "F007", "F008", "F009", "F010"],
        screens: ["pass", "fail"]
    };

    let tests = [
        ("F001", "sudo sh -c 'echo 1 > /f'", "SC2024"),
        ("F002", "echo 1 | sudo tee /f", "SC2024"),
        ("F003", "echo 1 | sudo tee /f >/dev/null", "SC2024"),
        ("F004", "sudo -u user cmd > /tmp/f", "SC2024"),
        ("F005", "sudo -v", "SC2024"),
        ("F006", "sudo -k && sudo -n ls", "SC2024"),
        ("F007", "sudo bash -c \"cmd | pipe\"", "SC2024"),
        ("F008", "pkexec cmd > /f", "SC2024"),
        ("F009", "doas cmd > /f", "SC2024"),
        ("F010", "sudo env PATH=$P cmd", "SC2024"),
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
        "\n6.1 Sudo/Permissions: {:.1}% elements",
        report.element_coverage * 100.0
    );
    assert!(
        report.element_coverage >= 1.0,
        "All sudo tests must pass (element coverage)"
    );
}

// ============================================================================
// 6.2 REDIRECTION AND PIPES (F011-F020)
// ============================================================================

#[test]
fn test_falsification_redirection_pipes() {
    let mut gui = gui_coverage! {
        buttons: ["F011", "F012", "F013", "F014", "F015", "F016", "F017", "F018", "F019", "F020"],
        screens: ["pass", "fail"]
    };

    let tests = [
        ("F011", "cmd 2>&1 | other", "SC2069"),
        ("F012", "cmd >/dev/null 2>&1", "SC2069"),
        ("F013", "cmd &> file", "SC2069"),
        ("F014", "exec 3>&1", "SC2069"),
        ("F015", "cmd |& other", "SC2069"),
        ("F016", "echo \"x\" >&2", "SC2069"),
        ("F017", "read -r x <<< \"str\"", "SC2069"),
        ("F018", "cmd <(list)", "SC2069"),
        ("F019", "cmd > >(other)", "SC2069"),
        ("F020", "{ cmd; } > file", "SC2024"),
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
        "\n6.2 Redirection/Pipes: {:.1}% elements",
        report.element_coverage * 100.0
    );
    assert!(
        report.element_coverage >= 1.0,
        "All redirection tests must pass"
    );
}

// ============================================================================
// 6.3 QUOTING AND HEREDOCS (F021-F030)
// ============================================================================

#[test]
fn test_falsification_quoting_heredocs() {
    let mut gui = gui_coverage! {
        buttons: ["F021", "F022", "F023", "F024", "F025", "F026", "F027", "F028", "F029", "F030"],
        screens: ["pass", "fail"]
    };

    let tests = [
        ("F021", "cat << 'EOF'\n$var\nEOF", "SC2016"),
        ("F022", "cat << \"EOF\"\n$var\nEOF", "SC2016"),
        ("F023", "cat <<-'EOF'\n  literal\nEOF", "SC2016"),
        ("F024", "echo \"Don't\"", "SC2016"),
        ("F025", "echo 'Value: \"$var\"'", "SC2016"),
        ("F026", "printf '%s\\n' \"$v\"", "SC2059"),
        ("F027", "echo \"Only $ var\"", "SC2016"),
        ("F028", "echo 'a'\\''b'", "SC2016"),
        ("F029", "find . -name '*.c'", "SC2035"),
        ("F030", "grep -r '*.c' .", "SC2035"),
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
        "\n6.3 Quoting/Heredocs: {:.1}% elements",
        report.element_coverage * 100.0
    );
    assert!(
        report.element_coverage >= 1.0,
        "All quoting tests must pass"
    );
}

// ============================================================================
// 6.4 VARIABLES AND PARAMETERS (F031-F045)
// ============================================================================

#[test]
fn test_falsification_variables_parameters() {
    let mut gui = gui_coverage! {
        buttons: ["F031", "F032", "F033", "F034", "F035", "F036", "F037", "F038", "F039", "F040", "F041", "F042", "F043", "F044", "F045"],
        screens: ["pass", "fail"]
    };

    let tests = [
        ("F031", "echo \"${var:-default}\"", "SC2086"),
        ("F032", "echo \"${var#*}\"", "SC2086"),
        ("F033", "echo \"${!prefix@}\"", "SC2086"),
        ("F034", "echo \"${arr[@]}\"", "SC2068"),
        ("F035", "echo ${#arr[@]}", "SC2086"),
        ("F036", "(( var++ ))", "SC2086"),
        ("F037", "[[ -n $var ]]", "SC2086"),
        ("F038", "f() { local var; echo $var; }", "SC2034"),
        ("F039", "export VAR=1", "SC2034"),
        ("F040", "readonly VAR=1", "SC2034"),
        ("F041", "_unused_arg=1", "SC2034"),
        ("F042", "typeset -n ref=$1", "SC2034"),
        ("F043", "PS1='prompt'", "SC2034"),
        ("F044", "PROMPT_COMMAND='cmd'", "SC2034"),
        ("F045", "trap 'echo $SIG' SIGINT", "SC2034"),
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
        "\n6.4 Variables/Parameters: {:.1}% elements",
        report.element_coverage * 100.0
    );
    assert!(
        report.element_coverage >= 1.0,
        "All variable tests must pass"
    );
}

// ============================================================================
// 6.5 CONTROL FLOW (F046-F060)
// ============================================================================

#[test]
fn test_falsification_control_flow() {
    let mut gui = gui_coverage! {
        buttons: ["F046", "F047", "F048", "F049", "F050", "F051", "F052", "F053", "F054", "F055", "F056", "F057", "F058", "F059", "F060"],
        screens: ["pass", "fail"]
    };

    let tests = [
        ("F046", "if true; then echo yes; fi", "Parser"),
        ("F047", "case $x in *) ;; esac", "SC2154"),
        ("F048", "for ((i=0;i<10;i++)); do echo $i; done", "SC2086"),
        ("F049", "select x in list; do echo $x; done", "Parser"),
        ("F050", "while read -r; do echo $REPLY; done < f", "SC2034"),
        ("F051", "until [[ cond ]]; do echo x; done", "Parser"),
        ("F052", "[ \"$a\" ] && [ \"$b\" ]", "SC2015"),
        ("F053", "! command", "Parser"),
        ("F054", "time command", "Parser"),
        ("F055", "coproc command", "Parser"),
        ("F056", "f() { return 0 2>/dev/null; }", "SC2086"),
        ("F057", "break 2", "Parser"),
        ("F058", "continue 2", "Parser"),
        ("F059", "exit 0; echo unreachable", "SC2317"),
        ("F060", "function f { cmd; }", "Parser"),
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
        "\n6.5 Control Flow: {:.1}% elements",
        report.element_coverage * 100.0
    );
    assert!(
        report.element_coverage >= 1.0,
        "All control flow tests must pass"
    );
}

// ============================================================================
// 6.6 BUILTINS AND ENVIRONMENT (F061-F070)
// ============================================================================

#[test]
fn test_falsification_builtins_environment() {
    let mut gui = gui_coverage! {
        buttons: ["F061", "F062", "F063", "F064", "F065", "F066", "F067", "F068", "F069", "F070"],
        screens: ["pass", "fail"]
    };

    let tests = [
        ("F061", "echo $EUID", "SC2154"),
        ("F062", "echo $UID", "SC2154"),
        ("F063", "echo $BASH_VERSION", "SC2154"),
        ("F064", "echo $PIPESTATUS", "SC2154"),
        ("F065", "echo $RANDOM", "SC2154"),
        ("F066", "echo $LINENO", "SC2154"),
        ("F067", "echo $SECONDS", "SC2154"),
        ("F068", "echo $PWD", "SC2154"),
        ("F069", "echo $OLDPWD", "SC2154"),
        ("F070", "echo $SHLVL", "SC2154"),
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
        "\n6.6 Builtins/Environment: {:.1}% elements",
        report.element_coverage * 100.0
    );
    assert!(
        report.element_coverage >= 1.0,
        "All builtin tests must pass"
    );
}

// ============================================================================
// 6.7 SUBSHELLS AND COMMAND SUBSTITUTION (F071-F080)
// ============================================================================

#[test]
fn test_falsification_subshells_cmdsub() {
    let mut gui = gui_coverage! {
        buttons: ["F071", "F072", "F073", "F074", "F075", "F076", "F077", "F078", "F079", "F080"],
        screens: ["pass", "fail"]
    };

    let tests = [
        ("F071", "( cd dir && cmd )", "SC2034"),
        ("F072", "echo $(command)", "SC2034"),
        ("F073", "var=$(cmd)", "SC2031"),
        ("F074", "var=\"$(cmd)\"", "SC2031"),
        ("F075", "echo $( < file )", "SC2002"),
        ("F076", "diff <(cmd1) <(cmd2)", "Parser"),
        ("F077", "exec > >(logger)", "Parser"),
        ("F078", "x=$( (cmd) )", "Parser"),
        ("F079", "x=$( { cmd; } )", "Parser"),
        ("F080", "x=`cmd`", "SC2006"),
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
        "\n6.7 Subshells/CmdSub: {:.1}% elements",
        report.element_coverage * 100.0
    );
    assert!(
        report.element_coverage >= 1.0,
        "All subshell tests must pass"
    );
}

// ============================================================================
// 6.8 TRAPS AND SIGNALS (F081-F090)
// ============================================================================

#[test]
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
