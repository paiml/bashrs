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

include!("linter_tui_bug_hunting_tests_cont.rs");
