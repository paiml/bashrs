fn hunt_frame_sequences() {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║           BUG HUNT: Frame Sequence Transitions               ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let mut bugs: Vec<TuiBugReport> = vec![];

    // Test sequences that build on each other
    let sequences = [
        (
            "SEQ01",
            vec!["x=1", "y=2", "z=$((x+y))", "echo $z"],
            "Variable chain",
        ),
        (
            "SEQ02",
            vec!["arr=()", "arr+=(a)", "arr+=(b)", "echo ${arr[@]}"],
            "Array building",
        ),
        (
            "SEQ03",
            vec![
                "if true; then echo a; fi",
                "if false; then echo b; fi",
                "if true; then if false; then echo c; fi; fi",
            ],
            "Nested if",
        ),
        (
            "SEQ04",
            vec![
                "func1() { echo 1; }",
                "func2() { echo 2; }",
                "func1",
                "func2",
            ],
            "Function sequence",
        ),
    ];

    for (id, codes, desc) in sequences {
        let mut sequence = FrameSequence::new(id);
        let mut had_error = false;

        for (i, code) in codes.iter().enumerate() {
            match lint_safely(code) {
                Ok(result) => {
                    let frame = linter_frame(code, &result);
                    sequence.add_frame(&frame);

                    // Each frame in sequence must have valid structure
                    if !frame.contains("BASHRS") {
                        bugs.push(TuiBugReport {
                            category: "Frame Sequence",
                            id: format!("{}_step{}", id, i),
                            description: desc.to_string(),
                            issue: format!("Frame {} corrupted in sequence", i),
                            frame_content: Some(frame.as_text()),
                        });
                        had_error = true;
                    }
                }
                Err(e) => {
                    bugs.push(TuiBugReport {
                        category: "Frame Sequence",
                        id: format!("{}_step{}", id, i),
                        description: desc.to_string(),
                        issue: e,
                        frame_content: None,
                    });
                    had_error = true;
                }
            }
        }

        // Verify sequence is complete
        if sequence.len() != codes.len() && !had_error {
            bugs.push(TuiBugReport {
                category: "Frame Sequence",
                id: id.to_string(),
                description: desc.to_string(),
                issue: format!("Incomplete sequence: {} of {}", sequence.len(), codes.len()),
                frame_content: None,
            });
            println!("  [BUG] {}: {} - Incomplete", id, desc);
        } else if !had_error {
            println!("  [OK] {}: {} ({} frames)", id, desc, sequence.len());
        } else {
            println!("  [BUG] {}: {} - Errors in sequence", id, desc);
        }
    }

    print_tui_bugs(&bugs);
    println!("\n  Frame sequence bugs found: {}", bugs.len());
}

// ============================================================================
// BUG HUNT: Pixel-Level Alignment
// ============================================================================

#[test]
fn hunt_pixel_alignment() {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║           BUG HUNT: Pixel-Level Alignment                    ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let mut bugs: Vec<TuiBugReport> = vec![];

    let cases = [
        ("PA001", "echo short", "Short output"),
        (
            "PA002",
            "echo 'this is a longer string with more content'",
            "Long output",
        ),
        (
            "PA003",
            "echo ${very_long_variable_name:-default}",
            "Long variable",
        ),
        ("PA004", "echo $((123456789 + 987654321))", "Large numbers"),
        (
            "PA005",
            "for i in a b c d e f g h i j; do echo $i; done",
            "Many items",
        ),
        ("PA006", "echo 'a\tb\tc'", "Tabs in output"),
        ("PA007", "echo 'line1'", "Single line"),
        ("PA008", "echo ''", "Empty string"),
        ("PA009", "echo '        '", "Spaces only"),
        (
            "PA010",
            "x='aaaaaaaaaaaaaaaaaaaaaaaaaaaa'; echo $x",
            "30+ char variable",
        ),
    ];

    for (id, code, desc) in cases {
        match lint_safely(code) {
            Ok(result) => {
                let frame = linter_frame(code, &result);
                let lines = frame.lines();

                // Check vertical alignment - all ║ chars should be at same columns
                let mut pipe_positions: HashMap<usize, Vec<usize>> = HashMap::new();
                for (row, line) in lines.iter().enumerate() {
                    for (col, ch) in line.chars().enumerate() {
                        if ch == '║' {
                            pipe_positions.entry(col).or_default().push(row);
                        }
                    }
                }

                // Find columns with most pipes - those are the alignment columns
                // Note: Rows 0, 2, 6, 9 have box corners (╔╠╚) not pipes, so we expect
                // pipes only on content rows (1, 3, 4, 5, 7, 8) = 6 rows for a 10-line frame
                let content_row_count = lines.len().saturating_sub(4); // Exclude corner rows
                let mut alignment_issues = false;
                for (col, rows) in &pipe_positions {
                    // Only flag if we have significantly fewer pipes than expected content rows
                    // and it's not a corner column issue
                    if rows.len() > 2 && rows.len() < content_row_count.saturating_sub(1) {
                        alignment_issues = true;
                        bugs.push(TuiBugReport {
                            category: "Pixel Alignment",
                            id: id.to_string(),
                            description: desc.to_string(),
                            issue: format!("Column {} has inconsistent pipes: {:?}", col, rows),
                            frame_content: Some(frame.as_text()),
                        });
                    }
                }

                if !alignment_issues {
                    println!("  [OK] {}: {}", id, desc);
                } else {
                    println!("  [BUG] {}: {} - Misaligned", id, desc);
                }
            }
            Err(e) => {
                bugs.push(TuiBugReport {
                    category: "Pixel Alignment",
                    id: id.to_string(),
                    description: desc.to_string(),
                    issue: e,
                    frame_content: None,
                });
                println!("  [BUG] {}: {} - PANIC", id, desc);
            }
        }
    }

    print_tui_bugs(&bugs);
    println!("\n  Pixel alignment bugs found: {}", bugs.len());
}

// ============================================================================
// BUG HUNT: Diagnostic Formatting
// ============================================================================

#[test]
fn hunt_diagnostic_formatting() {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║           BUG HUNT: Diagnostic Formatting                    ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let mut bugs: Vec<TuiBugReport> = vec![];

    // Cases that should produce diagnostics
    let cases = [
        ("DF001", "echo $unquoted", "SC2086 - unquoted variable"),
        ("DF002", "x=`cmd`", "SC2006 - backticks"),
        ("DF003", "echo $UNDEFINED_VAR", "SC2154 - undefined"),
        ("DF004", "rm -rf $path", "SEC001 - dangerous rm"),
        ("DF005", "chmod 777 file", "SEC003 - world writable"),
        ("DF006", "eval $user_input", "SEC007 - eval"),
        ("DF007", "x=$RANDOM", "DET001 - non-deterministic"),
        ("DF008", "echo $$", "DET002 - process ID"),
        ("DF009", "date +%s", "DET003 - timestamp"),
        ("DF010", "mkdir /tmp/dir", "IDEM001 - non-idempotent"),
    ];

    for (id, code, desc) in cases {
        match lint_safely(code) {
            Ok(result) => {
                let frame = linter_frame(code, &result);

                // Frame should contain diagnostic info
                if !frame.contains("Status:") {
                    bugs.push(TuiBugReport {
                        category: "Diagnostic Formatting",
                        id: id.to_string(),
                        description: desc.to_string(),
                        issue: "Missing Status field".to_string(),
                        frame_content: Some(frame.as_text()),
                    });
                    println!("  [BUG] {}: {} - No status", id, desc);
                    continue;
                }

                // If diagnostics exist, verify they're formatted
                if !result.diagnostics.is_empty() && !frame.contains("Diagnostics:") {
                    bugs.push(TuiBugReport {
                        category: "Diagnostic Formatting",
                        id: id.to_string(),
                        description: desc.to_string(),
                        issue: "Missing Diagnostics section".to_string(),
                        frame_content: Some(frame.as_text()),
                    });
                    println!("  [BUG] {}: {} - No diagnostics section", id, desc);
                    continue;
                }

                println!(
                    "  [OK] {}: {} ({} diagnostics)",
                    id,
                    desc,
                    result.diagnostics.len()
                );
            }
            Err(e) => {
                bugs.push(TuiBugReport {
                    category: "Diagnostic Formatting",
                    id: id.to_string(),
                    description: desc.to_string(),
                    issue: e,
                    frame_content: None,
                });
                println!("  [BUG] {}: {} - PANIC", id, desc);
            }
        }
    }

    print_tui_bugs(&bugs);
    println!("\n  Diagnostic formatting bugs found: {}", bugs.len());
}

// ============================================================================
// BUG HUNT: Coverage with expect_frame assertions
// ============================================================================

#[test]
fn hunt_frame_assertions() {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║           BUG HUNT: Frame Assertions                         ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let mut bugs: Vec<TuiBugReport> = vec![];

    let mut gui = gui_coverage! {
        buttons: [
            "FA001", "FA002", "FA003", "FA004", "FA005",
            "FA006", "FA007", "FA008", "FA009", "FA010"
        ],
        screens: ["valid", "invalid"]
    };

    let cases = [
        ("FA001", "echo hello", "Basic echo"),
        ("FA002", "x=5", "Assignment"),
        ("FA003", "if true; then :; fi", "If statement"),
        ("FA004", "while :; do break; done", "While loop"),
        ("FA005", "case $x in *) :;; esac", "Case"),
        ("FA006", "func() { :; }", "Function"),
        ("FA007", "echo $((1+2))", "Arithmetic"),
        ("FA008", "(subshell)", "Subshell"),
        ("FA009", "{ group; }", "Brace group"),
        ("FA010", "cmd1 && cmd2 || cmd3", "Boolean ops"),
    ];

    for (id, code, desc) in cases {
        gui.click(id);

        match lint_safely(code) {
            Ok(result) => {
                let frame = linter_frame(code, &result);

                // Use expect_frame assertions
                match expect_frame(&frame).to_contain_text("BASHRS LINTER") {
                    Ok(_) => {
                        gui.visit("valid");
                        println!("  [OK] {}: {}", id, desc);
                    }
                    Err(e) => {
                        gui.visit("invalid");
                        bugs.push(TuiBugReport {
                            category: "Frame Assertions",
                            id: id.to_string(),
                            description: desc.to_string(),
                            issue: format!("Assertion failed: {:?}", e),
                            frame_content: Some(frame.as_text()),
                        });
                        println!("  [BUG] {}: {} - Assertion failed", id, desc);
                    }
                }
            }
            Err(e) => {
                gui.visit("invalid");
                bugs.push(TuiBugReport {
                    category: "Frame Assertions",
                    id: id.to_string(),
                    description: desc.to_string(),
                    issue: e,
                    frame_content: None,
                });
                println!("  [BUG] {}: {} - PANIC", id, desc);
            }
        }
    }

    let report = gui.generate_report();
    print_tui_bugs(&bugs);
    println!(
        "\n  Frame assertion bugs found: {} (coverage: {:.1}%)",
        bugs.len(),
        report.element_coverage * 100.0
    );
}

// ============================================================================
// BUG HUNT: Long Content Truncation
// ============================================================================

#[test]

include!("linter_tui_bug_hunting_incl2_incl2.rs");
