fn hunt_malformed_syntax_recovery() {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║           BUG HUNT: Malformed Syntax Recovery                ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let mut bugs: Vec<BugReport> = vec![];

    // Malformed inputs that should NOT crash
    let cases = [
        ("M001", "echo ${", "Unclosed brace"),
        ("M002", "echo $((1+", "Unclosed arithmetic"),
        ("M003", "if true; then", "Missing fi"),
        ("M004", "case x in", "Missing esac"),
        ("M005", "while true; do", "Missing done"),
        ("M006", "echo \"unclosed", "Unclosed double quote"),
        ("M007", "echo 'unclosed", "Unclosed single quote"),
        ("M008", "echo $(unclosed", "Unclosed command sub"),
        ("M009", "}", "Unmatched brace"),
        ("M010", "fi", "Unexpected fi"),
        ("M011", "done", "Unexpected done"),
        ("M012", "esac", "Unexpected esac"),
        ("M013", ";;", "Unexpected ;;"),
        ("M014", "<<<", "Triple less-than"),
        ("M015", ">>>", "Triple greater-than"),
        ("M016", "||| ", "Triple pipe"),
        ("M017", "&&& ", "Triple ampersand"),
        ("M018", "echo $((()))", "Empty nested arithmetic"),
        ("M019", "echo ${}", "Empty brace expansion"),
        ("M020", "echo $[]", "Empty bracket"),
    ];

    for (id, code, desc) in cases {
        match lint_safely(code) {
            Ok(_) => println!("  [OK] {}: {} - Handled gracefully", id, desc),
            Err(e) => {
                println!("  [PANIC] {}: {} - {}", id, desc, e);
                bugs.push(BugReport {
                    category: "Malformed",
                    id: id.to_string(),
                    description: desc.to_string(),
                    code: code.to_string(),
                    issue: e,
                });
            }
        }
    }

    print_bugs(&bugs);
    println!("\n  Malformed syntax bugs found: {}", bugs.len());
}

// ============================================================================
// BUG HUNT: Escape Sequence Edge Cases
// ============================================================================

#[test]
fn hunt_escape_sequences() {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║           BUG HUNT: Escape Sequences                         ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let mut bugs: Vec<BugReport> = vec![];

    let cases = [
        ("E001", r#"echo "\\""#, "Escaped backslash"),
        ("E002", r#"echo "\$""#, "Escaped dollar"),
        ("E003", r#"echo "\`""#, "Escaped backtick"),
        ("E004", r#"echo "\"hello\"""#, "Escaped quotes"),
        ("E005", "echo $'\\n\\t\\r'", "ANSI-C escapes"),
        ("E006", "echo $'\\x41\\x42'", "Hex escapes"),
        ("E007", "echo $'\\101\\102'", "Octal escapes"),
        ("E008", "echo $'\\u0041'", "Unicode 4-digit"),
        ("E009", "echo $'\\U0001F600'", "Unicode 8-digit"),
        ("E010", "echo $'\\e[31m'", "ANSI color escape"),
        (
            "E011",
            "echo $'\\a\\b\\f\\v'",
            "Bell/backspace/form/vertical",
        ),
        ("E012", "echo $'\\0'", "Null byte"),
        ("E013", "echo $'\\c@'", "Control character"),
        ("E014", "echo $'\\'", "Single backslash in $''"),
        ("E015", "echo '\\n'", "Literal backslash-n"),
    ];

    for (id, code, desc) in cases {
        match lint_safely(code) {
            Ok(_) => println!("  [OK] {}: {}", id, desc),
            Err(e) => {
                println!("  [PANIC] {}: {} - {}", id, desc, e);
                bugs.push(BugReport {
                    category: "Escape",
                    id: id.to_string(),
                    description: desc.to_string(),
                    code: code.to_string(),
                    issue: e,
                });
            }
        }
    }

    print_bugs(&bugs);
    println!("\n  Escape sequence bugs found: {}", bugs.len());
}

// ============================================================================
// COMPREHENSIVE BUG HUNTING REPORT
// ============================================================================

#[test]
fn generate_bug_hunting_report() {
    println!("\n");
    println!("╔══════════════════════════════════════════════════════════════════════════════╗");
    println!("║                    LINTER BUG HUNTING - COMPREHENSIVE REPORT                 ║");
    println!("╚══════════════════════════════════════════════════════════════════════════════╝\n");

    let mut total_bugs = 0;
    let mut categories: Vec<(&str, usize)> = vec![];

    // Run all hunts and collect results
    let hunts = [
        ("Unicode Edge Cases", hunt_category_unicode()),
        ("Extreme Nesting", hunt_category_nesting()),
        ("Large Inputs", hunt_category_large_inputs()),
        ("False Positives", hunt_category_false_positives()),
        ("Malformed Syntax", hunt_category_malformed()),
        ("Escape Sequences", hunt_category_escapes()),
    ];

    println!("┌─────────────────────────────┬──────────┬──────────┐");
    println!("│ Category                    │ Tested   │ Bugs     │");
    println!("├─────────────────────────────┼──────────┼──────────┤");

    for (name, (tested, bugs)) in &hunts {
        println!("│ {:<27} │ {:>8} │ {:>8} │", name, tested, bugs);
        total_bugs += bugs;
        categories.push((name, *bugs));
    }

    println!("├─────────────────────────────┼──────────┼──────────┤");
    let total_tested: usize = hunts.iter().map(|(_, (t, _))| t).sum();
    println!(
        "│ {:<27} │ {:>8} │ {:>8} │",
        "TOTAL", total_tested, total_bugs
    );
    println!("└─────────────────────────────┴──────────┴──────────┘");

    println!();
    if total_bugs == 0 {
        println!("✅ No bugs found! Linter is robust.");
    } else {
        println!(
            "❌ Found {} bugs across {} categories.",
            total_bugs,
            categories.iter().filter(|(_, b)| *b > 0).count()
        );
        println!("\nPriority categories for fixing:");
        let mut sorted = categories.clone();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        for (name, bugs) in sorted.iter().filter(|(_, b)| *b > 0) {
            println!("  • {}: {} bugs", name, bugs);
        }
    }
    println!();
}

// Helper functions for comprehensive report
fn hunt_category_unicode() -> (usize, usize) {
    let cases = [
        "echo '日本語'",
        "echo '🚀🔥💻'",
        "echo 'مرحبا'",
        "x='Ω≈ç√∫'; echo $x",
        "echo 'A\u{0308}'",
    ];
    let mut bugs = 0;
    for code in cases {
        if lint_safely(code).is_err() {
            bugs += 1;
        }
    }
    (cases.len(), bugs)
}

fn hunt_category_nesting() -> (usize, usize) {
    let mut bugs = 0;
    let mut tested = 0;
    for depth in [5, 10, 20] {
        let mut code = "echo hi".to_string();
        for _ in 0..depth {
            code = format!("echo $({})", code);
        }
        tested += 1;
        if lint_safely(&code).is_err() {
            bugs += 1;
        }
    }
    (tested, bugs)
}

fn hunt_category_large_inputs() -> (usize, usize) {
    let mut bugs = 0;
    let mut tested = 0;
    for size in [100, 1000, 10000] {
        let code = format!("x={}", "a".repeat(size));
        tested += 1;
        if lint_safely(&code).is_err() {
            bugs += 1;
        }
    }
    (tested, bugs)
}

fn hunt_category_false_positives() -> (usize, usize) {
    let cases: [(&str, &str); 10] = [
        ("[[ $var ]]", "SC2086"),
        ("(( $x + $y ))", "SC2086"),
        ("echo ${#arr[@]}", "SC2086"),
        ("echo $RANDOM", "SC2154"),
        ("echo $EUID", "SC2154"),
        ("export VAR=1", "SC2034"),
        ("find . -name '*.txt'", "SC2035"),
        ("trap 'rm $f' EXIT", "SC2064"),
        ("sudo sh -c 'echo 1 > /f'", "SC2024"),
        ("echo 1 | sudo tee /f", "SC2024"),
    ];
    let mut bugs = 0;
    for (code, forbidden) in cases {
        if let Ok(result) = lint_safely(code) {
            if has_rule(&result, forbidden) {
                bugs += 1;
            }
        } else {
            bugs += 1;
        }
    }
    (cases.len(), bugs)
}

fn hunt_category_malformed() -> (usize, usize) {
    let cases = [
        "echo ${",
        "echo $((1+",
        "if true; then",
        "case x in",
        "while true; do",
    ];
    let mut bugs = 0;
    for code in cases {
        if lint_safely(code).is_err() {
            bugs += 1;
        }
    }
    (cases.len(), bugs)
}

fn hunt_category_escapes() -> (usize, usize) {
    let cases = [
        r#"echo "\\""#,
        r#"echo "\$""#,
        "echo $'\\n\\t\\r'",
        "echo $'\\x41'",
        "echo $'\\u0041'",
    ];
    let mut bugs = 0;
    for code in cases {
        if lint_safely(code).is_err() {
            bugs += 1;
        }
    }
    (cases.len(), bugs)
}
