//! Linter Bug Hunting - Aggressive Edge Case Discovery
//!
//! This module aggressively tests linter edge cases to find bugs.
//! Uses property-based testing patterns and corner cases.
//! Tests DON'T fail - they REPORT bugs found.
//!
//! Run: cargo test -p bashrs --test linter_bug_hunting -- --nocapture

#![allow(clippy::unwrap_used)]

use bashrs::linter::rules::lint_shell;
use std::panic::{self, AssertUnwindSafe};

/// Track bugs found during hunt
struct BugReport {
    category: &'static str,
    id: String,
    description: String,
    code: String,
    issue: String,
}

impl BugReport {
    /// Print bug report details
    fn print(&self) {
        println!("    ┌─────────────────────────────────────────────────────────");
        println!(
            "    │ {} [{}]: {}",
            self.category, self.id, self.description
        );
        println!(
            "    │ Code: {}",
            if self.code.len() > 60 {
                format!("{}...", &self.code[..60])
            } else {
                self.code.clone()
            }
        );
        println!("    │ Issue: {}", self.issue);
        println!("    └─────────────────────────────────────────────────────────");
    }
}

/// Print all bugs found in a category
fn print_bugs(bugs: &[BugReport]) {
    for bug in bugs {
        bug.print();
    }
}

/// Lint and check for panics
fn lint_safely(code: &str) -> Result<bashrs::linter::LintResult, String> {
    match panic::catch_unwind(AssertUnwindSafe(|| lint_shell(code))) {
        Ok(result) => Ok(result),
        Err(_) => Err("PANIC during linting".to_string()),
    }
}

/// Check if a specific rule was triggered
fn has_rule(result: &bashrs::linter::LintResult, rule: &str) -> bool {
    result.diagnostics.iter().any(|d| d.code == rule)
}

// ============================================================================
// BUG HUNT: Unicode Edge Cases
// ============================================================================

#[test]
fn hunt_unicode_edge_cases() {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║           BUG HUNT: Unicode Edge Cases                       ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let mut bugs: Vec<BugReport> = vec![];

    let cases = [
        ("U001", "echo '日本語'", "Japanese text"),
        ("U002", "echo '🚀🔥💻'", "Emoji string"),
        ("U003", "echo 'مرحبا'", "RTL Arabic"),
        ("U004", "x='Ω≈ç√∫'; echo $x", "Math symbols in var"),
        ("U005", "echo 'A\u{0308}'", "Combining diacritical"),
        ("U006", "echo '\u{200B}'", "Zero-width space"),
        ("U007", "echo '\u{FEFF}'", "BOM character"),
        ("U008", "echo '᠎'", "Mongolian vowel separator"),
        ("U009", "echo '⁣'", "Invisible separator"),
        ("U010", "var_日本語=1", "Unicode in variable name"),
        ("U011", "echo \"$日本語\"", "Unicode var expansion"),
        ("U012", "# 日本語コメント", "Unicode comment"),
        ("U013", "'日本語'() { echo hi; }", "Unicode function name"),
        ("U014", "echo ${x:-日本語}", "Unicode in default"),
        ("U015", "case $x in 日本語) ;; esac", "Unicode case pattern"),
    ];

    for (id, code, desc) in cases {
        match lint_safely(code) {
            Ok(_) => println!("  [OK] {}: {}", id, desc),
            Err(e) => {
                println!("  [BUG] {}: {} - {}", id, desc, e);
                bugs.push(BugReport {
                    category: "Unicode",
                    id: id.to_string(),
                    description: desc.to_string(),
                    code: code.to_string(),
                    issue: e,
                });
            }
        }
    }

    print_bugs(&bugs);
    println!("\n  Unicode bugs found: {}", bugs.len());
}

// ============================================================================
// BUG HUNT: Extreme Nesting
// ============================================================================

#[test]
fn hunt_extreme_nesting() {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║           BUG HUNT: Extreme Nesting                          ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let mut bugs: Vec<BugReport> = vec![];

    // Test increasingly deep nesting levels
    for depth in [5, 10, 20, 50, 100] {
        // Nested command substitution
        let mut code = "echo hi".to_string();
        for _ in 0..depth {
            code = format!("echo $({})", code);
        }

        let id = format!("N{:03}_cmdsub", depth);
        match lint_safely(&code) {
            Ok(_) => println!("  [OK] {}: {} levels of $()", id, depth),
            Err(e) => {
                println!("  [BUG] {}: {} levels of $() - {}", id, depth, e);
                bugs.push(BugReport {
                    category: "Nesting",
                    id,
                    description: format!("{} nested $()", depth),
                    code: code.chars().take(100).collect(),
                    issue: e,
                });
            }
        }

        // Nested parameter expansion
        let mut code2 = "default".to_string();
        for i in 0..depth {
            code2 = format!("${{x{}:-{}}}", i, code2);
        }
        code2 = format!("echo {}", code2);

        let id2 = format!("N{:03}_param", depth);
        match lint_safely(&code2) {
            Ok(_) => println!("  [OK] {}: {} levels of ${{}}", id2, depth),
            Err(e) => {
                println!("  [BUG] {}: {} levels of ${{}} - {}", id2, depth, e);
                bugs.push(BugReport {
                    category: "Nesting",
                    id: id2,
                    description: format!("{} nested ${{}}", depth),
                    code: code2.chars().take(100).collect(),
                    issue: e,
                });
            }
        }
    }

    print_bugs(&bugs);
    println!("\n  Nesting bugs found: {}", bugs.len());
}

// ============================================================================
// BUG HUNT: Large Inputs
// ============================================================================

#[test]
fn hunt_large_inputs() {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║           BUG HUNT: Large Inputs                             ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let mut bugs: Vec<BugReport> = vec![];

    // Test increasingly large inputs
    for size in [100, 1000, 10000, 50000] {
        // Large variable assignment
        let code = format!("x={}", "a".repeat(size));
        let id = format!("L{:05}_var", size);
        match lint_safely(&code) {
            Ok(_) => println!("  [OK] {}: {} char variable", id, size),
            Err(e) => {
                println!("  [BUG] {}: {} char variable - {}", id, size, e);
                bugs.push(BugReport {
                    category: "LargeInput",
                    id,
                    description: format!("{} char variable", size),
                    code: format!("x={}...", &"a".repeat(20)),
                    issue: e,
                });
            }
        }

        // Large number of commands
        let code2 = (0..size / 10)
            .map(|_| "echo x")
            .collect::<Vec<_>>()
            .join("; ");
        let id2 = format!("L{:05}_cmd", size / 10);
        match lint_safely(&code2) {
            Ok(_) => println!("  [OK] {}: {} chained commands", id2, size / 10),
            Err(e) => {
                println!("  [BUG] {}: {} chained commands - {}", id2, size / 10, e);
                bugs.push(BugReport {
                    category: "LargeInput",
                    id: id2,
                    description: format!("{} chained commands", size / 10),
                    code: "echo x; echo x; ...".to_string(),
                    issue: e,
                });
            }
        }

        // Large array
        let code3 = format!(
            "arr=({})",
            (0..size / 10)
                .map(|i| format!("elem{}", i))
                .collect::<Vec<_>>()
                .join(" ")
        );
        let id3 = format!("L{:05}_arr", size / 10);
        match lint_safely(&code3) {
            Ok(_) => println!("  [OK] {}: {} element array", id3, size / 10),
            Err(e) => {
                println!("  [BUG] {}: {} element array - {}", id3, size / 10, e);
                bugs.push(BugReport {
                    category: "LargeInput",
                    id: id3,
                    description: format!("{} element array", size / 10),
                    code: "arr=(elem0 elem1 ...)".to_string(),
                    issue: e,
                });
            }
        }
    }

    print_bugs(&bugs);
    println!("\n  Large input bugs found: {}", bugs.len());
}

// ============================================================================
// BUG HUNT: False Positive Edge Cases
// ============================================================================

#[test]
fn hunt_false_positive_edge_cases() {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║           BUG HUNT: False Positive Edge Cases                ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let mut bugs: Vec<BugReport> = vec![];

    // Tricky cases that might trigger false positives
    let cases = [
        // SC2086 edge cases
        ("FP001", "[[ $var ]]", "SC2086", "Unquoted in [[]]"),
        ("FP002", "(( $x + $y ))", "SC2086", "Unquoted in (())"),
        ("FP003", "echo ${#arr[@]}", "SC2086", "Array length"),
        ("FP004", "echo $((x + y))", "SC2086", "Arithmetic expansion"),
        (
            "FP005",
            "for ((i=0;i<10;i++)); do echo $i; done",
            "SC2086",
            "C-style for var",
        ),
        // SC2154 edge cases
        ("FP006", "echo $RANDOM", "SC2154", "Builtin RANDOM"),
        ("FP007", "echo $EUID", "SC2154", "Builtin EUID"),
        ("FP008", "echo $$", "SC2154", "PID variable"),
        ("FP009", "echo $!", "SC2154", "Background PID"),
        (
            "FP010",
            "case $x in *) ;; esac",
            "SC2154",
            "Case expression var",
        ),
        // SC2034 edge cases
        ("FP011", "export VAR=1", "SC2034", "Exported variable"),
        ("FP012", "readonly VAR=1", "SC2034", "Readonly variable"),
        ("FP013", "_unused=1", "SC2034", "Underscore prefix"),
        ("FP014", "PS1='prompt'", "SC2034", "Shell variable"),
        ("FP015", "PROMPT_COMMAND='cmd'", "SC2034", "Hook variable"),
        // SC2035 edge cases
        (
            "FP016",
            "find . -name '*.txt'",
            "SC2035",
            "Find -name pattern",
        ),
        ("FP017", "grep '*.txt' file", "SC2035", "Grep regex"),
        (
            "FP018",
            "ls *.txt 2>/dev/null",
            "SC2035",
            "Glob with redirect",
        ),
        // SC2064 edge cases
        ("FP019", "trap 'rm $f' EXIT", "SC2064", "Trap single quote"),
        (
            "FP020",
            "trap \"echo $v\" INT",
            "SC2064",
            "Trap double quote intentional",
        ),
        // SC2024 edge cases
        (
            "FP021",
            "sudo sh -c 'echo 1 > /f'",
            "SC2024",
            "Sudo wrapped",
        ),
        ("FP022", "echo 1 | sudo tee /f", "SC2024", "Sudo tee"),
        ("FP023", "sudo cmd > /tmp/f", "SC2024", "Sudo writable path"),
        // Complex combinations
        (
            "FP024",
            "echo \"${arr[@]/#/prefix}\"",
            "SC2086",
            "Array transform",
        ),
        ("FP025", "[[ $x =~ ^[0-9]+$ ]]", "SC2086", "Regex match"),
    ];

    for (id, code, forbidden_rule, desc) in cases {
        match lint_safely(code) {
            Ok(result) => {
                if has_rule(&result, forbidden_rule) {
                    println!("  [FP] {}: {} - Triggered {}", id, desc, forbidden_rule);
                    bugs.push(BugReport {
                        category: "FalsePositive",
                        id: id.to_string(),
                        description: desc.to_string(),
                        code: code.to_string(),
                        issue: format!("Incorrectly triggered {}", forbidden_rule),
                    });
                } else {
                    println!("  [OK] {}: {}", id, desc);
                }
            }
            Err(e) => {
                println!("  [PANIC] {}: {} - {}", id, desc, e);
                bugs.push(BugReport {
                    category: "FalsePositive",
                    id: id.to_string(),
                    description: desc.to_string(),
                    code: code.to_string(),
                    issue: e,
                });
            }
        }
    }

    print_bugs(&bugs);
    println!("\n  False positive bugs found: {}", bugs.len());
}

// ============================================================================
// BUG HUNT: Malformed Syntax Recovery
// ============================================================================

#[test]

include!("linter_bug_hunting_incl2.rs");
