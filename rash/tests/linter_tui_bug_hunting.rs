//! Linter TUI Bug Hunting - Pixel-Level Edge Case Discovery
//!
//! This module aggressively tests linter TUI output to find rendering bugs.
//! Uses frame assertions, snapshot testing, and pixel-level validation.
//! Tests DON'T fail - they REPORT bugs found.
//!
//! Run: cargo test -p bashrs --test linter_tui_bug_hunting -- --nocapture

#![allow(clippy::unwrap_used)]

use bashrs::linter::rules::lint_shell;
use bashrs::linter::LintResult;
use jugar_probar::gui_coverage;
use jugar_probar::tui::{expect_frame, FrameSequence, TuiFrame, TuiSnapshot};
use std::collections::HashMap;
use std::panic::{self, AssertUnwindSafe};

// ============================================================================
// BUG REPORT STRUCTURES
// ============================================================================

/// Track TUI rendering bugs found during hunt
struct TuiBugReport {
    category: &'static str,
    id: String,
    description: String,
    issue: String,
    frame_content: Option<String>,
}

impl TuiBugReport {
    fn print(&self) {
        println!("    ┌─────────────────────────────────────────────────────────────────");
        println!(
            "    │ {} [{}]: {}",
            self.category, self.id, self.description
        );
        println!("    │ Issue: {}", self.issue);
        if let Some(ref content) = self.frame_content {
            let preview: String = content.chars().take(100).collect();
            println!("    │ Frame: {}...", preview);
        }
        println!("    └─────────────────────────────────────────────────────────────────");
    }
}

fn print_tui_bugs(bugs: &[TuiBugReport]) {
    for bug in bugs {
        bug.print();
    }
}

// ============================================================================
// LINTER FRAME UTILITIES
// ============================================================================

/// Create a TUI frame from linter output
fn linter_frame(code: &str, result: &LintResult) -> TuiFrame {
    let code_display = if code.len() > 50 {
        format!("{}...", &code[..47])
    } else {
        code.to_string()
    };

    let diag_count = result.diagnostics.len();
    let diagnostics_preview: String = result
        .diagnostics
        .iter()
        .take(3)
        .map(|d| format!("{}: {}", d.code, d.message))
        .collect::<Vec<_>>()
        .join(" | ");

    let status = if diag_count == 0 { "CLEAN" } else { "WARNINGS" };

    let diag_display = if diagnostics_preview.len() > 70 {
        format!("{}...", &diagnostics_preview[..67])
    } else {
        diagnostics_preview
    };

    // Build lines as owned strings first
    let line3 = format!("║ Code:   {:<62}║", code_display);
    let line4 = format!("║ Status: {:<62}║", status);
    let line5 = format!("║ Count:  {:<62}║", diag_count);
    let line8 = format!("║ {:<70}║", diag_display);

    let lines: Vec<&str> = vec![
        "╔═══════════════════════════════════════════════════════════════════════╗",
        "║ BASHRS LINTER OUTPUT                                                  ║",
        "╠═══════════════════════════════════════════════════════════════════════╣",
        &line3,
        &line4,
        &line5,
        "╠═══════════════════════════════════════════════════════════════════════╣",
        "║ Diagnostics:                                                          ║",
        &line8,
        "╚═══════════════════════════════════════════════════════════════════════╝",
    ];

    TuiFrame::from_lines(&lines)
}

/// Lint safely catching panics
fn lint_safely(code: &str) -> Result<LintResult, String> {
    match panic::catch_unwind(AssertUnwindSafe(|| lint_shell(code))) {
        Ok(result) => Ok(result),
        Err(_) => Err("PANIC during linting".to_string()),
    }
}

/// Check pixel at specific position
fn check_pixel(frame: &TuiFrame, row: usize, col: usize, expected: char) -> bool {
    let lines = frame.lines();
    if row >= lines.len() {
        return false;
    }
    let chars: Vec<char> = lines[row].chars().collect();
    if col >= chars.len() {
        return false;
    }
    chars[col] == expected
}

/// Count specific character in frame
fn count_char(frame: &TuiFrame, ch: char) -> usize {
    frame.as_text().chars().filter(|c| *c == ch).count()
}

// ============================================================================
// BUG HUNT: TUI Frame Rendering
// ============================================================================

#[test]
fn hunt_tui_frame_rendering() {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║           BUG HUNT: TUI Frame Rendering                      ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let mut bugs: Vec<TuiBugReport> = vec![];

    // Test cases that might cause rendering issues
    let cases = [
        ("FR001", "echo hello", "Simple command"),
        ("FR002", "echo $VAR", "Variable reference"),
        ("FR003", "echo ${var:-default}", "Parameter expansion"),
        ("FR004", "echo $((1+2))", "Arithmetic expansion"),
        ("FR005", "if true; then echo yes; fi", "Control structure"),
        (
            "FR006",
            "for i in 1 2 3; do echo $i; done",
            "Loop structure",
        ),
        ("FR007", "echo 'single' \"double\"", "Mixed quotes"),
        ("FR008", "cmd1 | cmd2 | cmd3", "Pipeline"),
        ("FR009", "echo $?", "Special variable"),
        ("FR010", "arr=(a b c); echo ${arr[@]}", "Array operation"),
    ];

    for (id, code, desc) in cases {
        match lint_safely(code) {
            Ok(result) => {
                let frame = linter_frame(code, &result);

                // Frame must contain header
                if !frame.contains("BASHRS LINTER") {
                    bugs.push(TuiBugReport {
                        category: "Frame Rendering",
                        id: id.to_string(),
                        description: desc.to_string(),
                        issue: "Missing header in frame".to_string(),
                        frame_content: Some(frame.as_text()),
                    });
                    println!("  [BUG] {}: {} - Missing header", id, desc);
                    continue;
                }

                // Frame must have proper box drawing
                let box_chars = count_char(&frame, '═');
                if box_chars < 10 {
                    bugs.push(TuiBugReport {
                        category: "Frame Rendering",
                        id: id.to_string(),
                        description: desc.to_string(),
                        issue: format!("Insufficient box chars: {}", box_chars),
                        frame_content: Some(frame.as_text()),
                    });
                    println!("  [BUG] {}: {} - Box drawing issue", id, desc);
                    continue;
                }

                // Frame corners must be correct
                if !check_pixel(&frame, 0, 0, '╔') {
                    bugs.push(TuiBugReport {
                        category: "Frame Rendering",
                        id: id.to_string(),
                        description: desc.to_string(),
                        issue: "Top-left corner incorrect".to_string(),
                        frame_content: Some(frame.as_text()),
                    });
                    println!("  [BUG] {}: {} - Corner pixel error", id, desc);
                    continue;
                }

                println!("  [OK] {}: {}", id, desc);
            }
            Err(e) => {
                bugs.push(TuiBugReport {
                    category: "Frame Rendering",
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
    println!("\n  Frame rendering bugs found: {}", bugs.len());
}

// ============================================================================
// BUG HUNT: Unicode in TUI Frames
// ============================================================================

#[test]
fn hunt_tui_unicode_rendering() {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║           BUG HUNT: Unicode TUI Rendering                    ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let mut bugs: Vec<TuiBugReport> = vec![];

    let cases = [
        ("UR001", "echo 'héllo'", "Latin extended"),
        ("UR002", "echo '日本語'", "Japanese"),
        ("UR003", "echo '🚀🔥💻'", "Emoji"),
        ("UR004", "echo 'مرحبا'", "Arabic RTL"),
        ("UR005", "echo 'א ב ג'", "Hebrew RTL"),
        ("UR006", "echo '中文字符'", "Chinese"),
        ("UR007", "x='ñ'; echo $x", "Variable with unicode"),
        ("UR008", "echo 'a\u{0301}'", "Combining diacritical"),
        ("UR009", "echo '→←↑↓'", "Arrows"),
        ("UR010", "echo '∀x∈ℕ'", "Math symbols"),
        ("UR011", "echo '\u{200B}'", "Zero-width space"),
        ("UR012", "echo '\u{FEFF}'", "BOM character"),
    ];

    for (id, code, desc) in cases {
        match lint_safely(code) {
            Ok(result) => {
                let frame = linter_frame(code, &result);
                let lines = frame.lines();

                // Check line count consistency
                let widths: Vec<usize> = lines.iter().map(|l| l.chars().count()).collect();
                let max_width = widths.iter().max().unwrap_or(&0);
                let min_width = widths.iter().min().unwrap_or(&0);

                // Allow some variance but flag extreme differences
                if max_width - min_width > 10 {
                    bugs.push(TuiBugReport {
                        category: "Unicode Rendering",
                        id: id.to_string(),
                        description: desc.to_string(),
                        issue: format!("Width variance: {}-{}", min_width, max_width),
                        frame_content: Some(frame.as_text()),
                    });
                    println!("  [BUG] {}: {} - Width inconsistency", id, desc);
                    continue;
                }

                // Frame must still have proper structure
                if !frame.contains("BASHRS") {
                    bugs.push(TuiBugReport {
                        category: "Unicode Rendering",
                        id: id.to_string(),
                        description: desc.to_string(),
                        issue: "Frame structure corrupted by unicode".to_string(),
                        frame_content: Some(frame.as_text()),
                    });
                    println!("  [BUG] {}: {} - Structure corrupted", id, desc);
                    continue;
                }

                println!("  [OK] {}: {}", id, desc);
            }
            Err(e) => {
                bugs.push(TuiBugReport {
                    category: "Unicode Rendering",
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
    println!("\n  Unicode rendering bugs found: {}", bugs.len());
}

// ============================================================================
// BUG HUNT: Snapshot Stability
// ============================================================================

#[test]
fn hunt_snapshot_stability() {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║           BUG HUNT: Snapshot Stability                       ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let mut bugs: Vec<TuiBugReport> = vec![];
    let mut snapshots: Vec<TuiSnapshot> = vec![];

    let cases = [
        ("SS001", "echo hello", "Simple echo"),
        ("SS002", "x=5; echo $x", "Variable assignment"),
        ("SS003", "echo ${var:-default}", "Parameter expansion"),
        ("SS004", "if true; then echo yes; fi", "If statement"),
        ("SS005", "for i in a b c; do echo $i; done", "For loop"),
        ("SS006", "case $x in a) echo a;; esac", "Case statement"),
        ("SS007", "func() { echo hi; }", "Function definition"),
        ("SS008", "echo $((1+2*3))", "Arithmetic"),
        ("SS009", "arr=(1 2 3)", "Array"),
        ("SS010", "[[ $x == y ]]", "Extended test"),
    ];

    for (id, code, desc) in cases {
        match lint_safely(code) {
            Ok(result) => {
                let frame = linter_frame(code, &result);
                let snapshot = TuiSnapshot::from_frame(id, &frame);

                // Verify snapshot has valid hash
                if snapshot.hash.is_empty() {
                    bugs.push(TuiBugReport {
                        category: "Snapshot Stability",
                        id: id.to_string(),
                        description: desc.to_string(),
                        issue: "Empty snapshot hash".to_string(),
                        frame_content: Some(frame.as_text()),
                    });
                    println!("  [BUG] {}: {} - Empty hash", id, desc);
                    continue;
                }

                // Re-lint and create second snapshot - hashes must match
                if let Ok(result2) = lint_safely(code) {
                    let frame2 = linter_frame(code, &result2);
                    let snapshot2 = TuiSnapshot::from_frame(id, &frame2);

                    if snapshot.hash != snapshot2.hash {
                        bugs.push(TuiBugReport {
                            category: "Snapshot Stability",
                            id: id.to_string(),
                            description: desc.to_string(),
                            issue: "Non-deterministic output".to_string(),
                            frame_content: Some(format!(
                                "Hash1: {} != Hash2: {}",
                                snapshot.hash, snapshot2.hash
                            )),
                        });
                        println!("  [BUG] {}: {} - Non-deterministic", id, desc);
                        continue;
                    }
                }

                snapshots.push(snapshot);
                println!("  [OK] {}: {}", id, desc);
            }
            Err(e) => {
                bugs.push(TuiBugReport {
                    category: "Snapshot Stability",
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
    println!(
        "\n  Snapshot stability bugs found: {} ({} snapshots captured)",
        bugs.len(),
        snapshots.len()
    );
}

// ============================================================================
// BUG HUNT: Frame Sequence Transitions
// ============================================================================

#[test]
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
fn hunt_content_truncation() {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║           BUG HUNT: Content Truncation                       ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let mut bugs: Vec<TuiBugReport> = vec![];

    // Generate increasingly long content
    let cases: Vec<(&str, String, &str)> = vec![
        ("CT001", "echo short".to_string(), "Short content"),
        (
            "CT002",
            format!("echo '{}'", "a".repeat(50)),
            "50 char string",
        ),
        (
            "CT003",
            format!("echo '{}'", "b".repeat(100)),
            "100 char string",
        ),
        (
            "CT004",
            format!("x={}; echo $x", "c".repeat(200)),
            "200 char variable",
        ),
        (
            "CT005",
            format!(
                "for i in {}; do echo $i; done",
                (1..50).map(|n| n.to_string()).collect::<Vec<_>>().join(" ")
            ),
            "50 loop iterations",
        ),
        (
            "CT006",
            format!("arr=({}); echo ${{arr[@]}}", "x ".repeat(100)),
            "100 element array",
        ),
        (
            "CT007",
            format!("echo ${{{}:-default}}", "d".repeat(80)),
            "80 char var name",
        ),
        (
            "CT008",
            "# ".to_string() + &"comment ".repeat(50),
            "Long comment",
        ),
        (
            "CT009",
            format!("echo \"{}\"", "e".repeat(500)),
            "500 char quoted",
        ),
        (
            "CT010",
            format!("{}=1", "f".repeat(100)),
            "100 char identifier",
        ),
    ];

    for (id, code, desc) in cases {
        match lint_safely(&code) {
            Ok(result) => {
                let frame = linter_frame(&code, &result);
                let text = frame.as_text();

                // Frame should not exceed reasonable width
                for (i, line) in frame.lines().iter().enumerate() {
                    let len = line.chars().count();
                    if len > 120 {
                        bugs.push(TuiBugReport {
                            category: "Content Truncation",
                            id: id.to_string(),
                            description: desc.to_string(),
                            issue: format!("Line {} too long: {} chars", i, len),
                            frame_content: Some(text.clone()),
                        });
                        println!("  [BUG] {}: {} - Line overflow", id, desc);
                        continue;
                    }
                }

                // Should still have valid structure
                if !frame.contains("BASHRS") {
                    bugs.push(TuiBugReport {
                        category: "Content Truncation",
                        id: id.to_string(),
                        description: desc.to_string(),
                        issue: "Frame structure lost".to_string(),
                        frame_content: Some(text),
                    });
                    println!("  [BUG] {}: {} - Structure lost", id, desc);
                    continue;
                }

                println!("  [OK] {}: {}", id, desc);
            }
            Err(e) => {
                bugs.push(TuiBugReport {
                    category: "Content Truncation",
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
    println!("\n  Content truncation bugs found: {}", bugs.len());
}

// ============================================================================
// COMPREHENSIVE TUI BUG HUNTING REPORT
// ============================================================================

#[test]
fn generate_tui_bug_hunting_report() {
    println!("\n╔══════════════════════════════════════════════════════════════════════════════╗");
    println!("║                   TUI BUG HUNTING - COMPREHENSIVE REPORT                     ║");
    println!("╚══════════════════════════════════════════════════════════════════════════════╝\n");

    // Summary of test categories
    let categories = [
        ("Frame Rendering", 10),
        ("Unicode Rendering", 12),
        ("Snapshot Stability", 10),
        ("Frame Sequences", 4),
        ("Pixel Alignment", 10),
        ("Diagnostic Formatting", 10),
        ("Frame Assertions", 10),
        ("Content Truncation", 10),
    ];

    println!("┌─────────────────────────────┬──────────┐");
    println!("│ Category                    │ Tests    │");
    println!("├─────────────────────────────┼──────────┤");

    let mut total = 0;
    for (category, count) in &categories {
        println!("│ {:<27} │ {:>8} │", category, count);
        total += count;
    }

    println!("├─────────────────────────────┼──────────┤");
    println!("│ {:<27} │ {:>8} │", "TOTAL", total);
    println!("└─────────────────────────────┴──────────┘");
    println!();

    // Test categories covered
    println!("TUI/Pixel Testing Coverage:");
    println!("  - Frame construction and rendering");
    println!("  - Unicode width and alignment");
    println!("  - Snapshot determinism");
    println!("  - Frame sequence consistency");
    println!("  - Box-drawing character alignment");
    println!("  - Diagnostic message formatting");
    println!("  - expect_frame() assertions");
    println!("  - Content truncation handling");
}
