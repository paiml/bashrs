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
