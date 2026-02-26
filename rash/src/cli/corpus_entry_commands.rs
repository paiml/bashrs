//! Corpus entry checking, difficulty classification, and risk analysis commands.

use super::corpus_failure_commands::result_fail_dims;
use super::corpus_score_print_commands::stats_bar;
use crate::cli::args::CorpusOutputFormat;
use crate::models::{Config, Error, Result};

pub(crate) fn corpus_check_entry(id: &str, format: &CorpusOutputFormat) -> Result<()> {
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;

    let registry = CorpusRegistry::load_full();
    let entry = registry
        .entries
        .iter()
        .find(|e| e.id == id)
        .ok_or_else(|| Error::Validation(format!("Entry {id} not found")))?;

    let config = Config::default();
    let runner = CorpusRunner::new(config);
    let result = runner.run_single(entry);

    match format {
        CorpusOutputFormat::Human => {
            use crate::cli::color::*;
            println!("{BOLD}Metamorphic Check: {id}{RESET}");
            println!("{DIM}Input:{RESET} {}", truncate_line(&entry.input, 60));
            println!();

            let mr_pass = |name: &str, ok: bool, desc: &str| {
                let mark = if ok {
                    format!("{GREEN}PASS{RESET}")
                } else {
                    format!("{BRIGHT_RED}FAIL{RESET}")
                };
                println!("  {name:<22} {mark}  {DIM}{desc}{RESET}");
            };

            // MR-1: Determinism — transpile twice, same output
            let result2 = runner.run_single(entry);
            let mr1 = result.actual_output == result2.actual_output;
            mr_pass("MR-1 Determinism", mr1, "transpile(X) == transpile(X)");

            // MR-2: Transpilation success
            mr_pass(
                "MR-2 Transpilation",
                result.transpiled,
                "transpile(X) succeeds",
            );

            // MR-3: Containment
            mr_pass(
                "MR-3 Containment",
                result.output_contains,
                "output ⊇ expected_contains",
            );

            // MR-4: Exact match
            mr_pass(
                "MR-4 Exact match",
                result.output_exact,
                "output lines == expected_contains",
            );

            // MR-5: Behavioral execution
            mr_pass(
                "MR-5 Behavioral",
                result.output_behavioral,
                "sh -c output terminates",
            );

            // MR-6: Lint clean
            mr_pass(
                "MR-6 Lint clean",
                result.lint_clean,
                "shellcheck/make -n passes",
            );

            // MR-7: Cross-shell agree
            mr_pass(
                "MR-7 Cross-shell",
                result.cross_shell_agree,
                "sh + dash agree",
            );

            let total = 7u32;
            let passed = [
                mr1,
                result.transpiled,
                result.output_contains,
                result.output_exact,
                result.output_behavioral,
                result.lint_clean,
                result.cross_shell_agree,
            ]
            .iter()
            .filter(|&&b| b)
            .count() as u32;
            let pct = (passed as f64 / total as f64) * 100.0;
            let pc = pct_color(pct);
            println!();
            println!("{BOLD}MR Pass Rate:{RESET} {pc}{passed}/{total} ({pct:.0}%){RESET}");
            println!(
                "{BOLD}V2 Score:{RESET} {pc}{:.1}/100{RESET}",
                result.score()
            );
        }
        CorpusOutputFormat::Json => {
            #[derive(serde::Serialize)]
            struct MrCheck {
                name: String,
                passed: bool,
                description: String,
            }
            #[derive(serde::Serialize)]
            struct CheckResult {
                id: String,
                checks: Vec<MrCheck>,
                passed: u32,
                total: u32,
                score: f64,
            }
            let result2 = runner.run_single(entry);
            let mr1 = result.actual_output == result2.actual_output;
            let checks = vec![
                MrCheck {
                    name: "MR-1 Determinism".into(),
                    passed: mr1,
                    description: "transpile(X) == transpile(X)".into(),
                },
                MrCheck {
                    name: "MR-2 Transpilation".into(),
                    passed: result.transpiled,
                    description: "transpile(X) succeeds".into(),
                },
                MrCheck {
                    name: "MR-3 Containment".into(),
                    passed: result.output_contains,
                    description: "output ⊇ expected".into(),
                },
                MrCheck {
                    name: "MR-4 Exact match".into(),
                    passed: result.output_exact,
                    description: "output == expected".into(),
                },
                MrCheck {
                    name: "MR-5 Behavioral".into(),
                    passed: result.output_behavioral,
                    description: "sh -c terminates".into(),
                },
                MrCheck {
                    name: "MR-6 Lint clean".into(),
                    passed: result.lint_clean,
                    description: "linter passes".into(),
                },
                MrCheck {
                    name: "MR-7 Cross-shell".into(),
                    passed: result.cross_shell_agree,
                    description: "sh + dash agree".into(),
                },
            ];
            let passed = checks.iter().filter(|c| c.passed).count() as u32;
            let cr = CheckResult {
                id: id.to_string(),
                checks,
                passed,
                total: 7,
                score: result.score(),
            };
            let json = serde_json::to_string_pretty(&cr)
                .map_err(|e| Error::Internal(format!("JSON: {e}")))?;
            println!("{json}");
        }
    }
    Ok(())
}

/// Truncate a string to max_len, adding "..." if truncated.
pub(crate) fn truncate_line(s: &str, max_len: usize) -> String {
    let line = s.lines().next().unwrap_or(s);
    if line.len() <= max_len {
        line.to_string()
    } else {
        format!("{}...", &line[..max_len])
    }
}

/// Classify a corpus entry's difficulty based on input features (spec §2.3).
/// Returns tier 1-5 with factor breakdown.
pub(crate) fn classify_difficulty(input: &str) -> (u8, Vec<(&'static str, bool)>) {
    let lines: Vec<&str> = input.lines().collect();
    let line_count = lines.len();
    let has_fn = input.contains("fn ") && input.matches("fn ").count() > 1;
    let has_loop = input.contains("for ") || input.contains("while ") || input.contains("loop ");
    let has_pipe = input.contains('|');
    let has_if = input.contains("if ");
    let has_match = input.contains("match ");
    let has_nested = input.matches('{').count() > 3;
    let has_special = input.contains('\\') || input.contains("\\n") || input.contains("\\t");
    let has_unicode = !input.is_ascii();
    let has_unsafe = input.contains("unsafe") || input.contains("exec") || input.contains("eval");

    let mut factors = vec![
        (
            "Simple (single construct)",
            line_count <= 3 && !has_loop && !has_fn,
        ),
        ("Has loops", has_loop),
        ("Has multiple functions", has_fn),
        ("Has pipes/redirects", has_pipe),
        ("Has conditionals", has_if || has_match),
        ("Has deep nesting (>3)", has_nested),
        ("Has special chars/escapes", has_special),
        ("Has Unicode", has_unicode),
        ("Has unsafe/exec patterns", has_unsafe),
    ];

    // Score based on complexity indicators
    let complexity: u32 = [
        has_loop as u32,
        has_fn as u32 * 2,
        has_pipe as u32,
        (has_if || has_match) as u32,
        has_nested as u32 * 2,
        has_special as u32,
        has_unicode as u32 * 2,
        has_unsafe as u32 * 3,
        (line_count > 10) as u32,
        (line_count > 30) as u32 * 2,
    ]
    .iter()
    .sum();

    let tier = match complexity {
        0..=1 => 1,
        2..=3 => 2,
        4..=6 => 3,
        7..=9 => 4,
        _ => 5,
    };

    // Add tier-specific reason
    factors.push(("POSIX-safe (no bashisms)", !has_unsafe && !has_unicode));

    (tier, factors)
}

/// Tier description string.
pub(crate) fn tier_label(tier: u8) -> &'static str {
    match tier {
        1 => "Trivial",
        2 => "Standard",
        3 => "Complex",
        4 => "Adversarial",
        5 => "Production",
        _ => "Unknown",
    }
}

/// Classify corpus entry difficulty (spec §2.3).
pub(crate) fn corpus_classify_difficulty(id: &str, format: &CorpusOutputFormat) -> Result<()> {
    use crate::corpus::registry::CorpusRegistry;

    let registry = CorpusRegistry::load_full();

    if id == "all" {
        return corpus_classify_all(&registry, format);
    }

    let entry = registry
        .entries
        .iter()
        .find(|e| e.id == id)
        .ok_or_else(|| Error::Validation(format!("Entry {id} not found")))?;

    let (tier, factors) = classify_difficulty(&entry.input);

    match format {
        CorpusOutputFormat::Human => {
            use crate::cli::color::*;
            println!("{BOLD}Difficulty: {id}{RESET}");
            println!("{DIM}Input:{RESET} {}", truncate_line(&entry.input, 60));
            println!();
            let tc = match tier {
                1 => GREEN,
                2 => CYAN,
                3 => YELLOW,
                4 => BRIGHT_RED,
                _ => BRIGHT_CYAN,
            };
            println!(
                "{BOLD}Predicted Tier:{RESET} {tc}{tier} ({}){RESET}",
                tier_label(tier)
            );
            println!();
            println!("{BOLD}Complexity Factors:{RESET}");
            for (label, present) in &factors {
                let mark = if *present {
                    format!("{GREEN}+{RESET}")
                } else {
                    format!("{DIM}-{RESET}")
                };
                println!("  {mark} {label}");
            }
        }
        CorpusOutputFormat::Json => {
            #[derive(serde::Serialize)]
            struct DiffResult {
                id: String,
                tier: u8,
                label: String,
                factors: Vec<Factor>,
            }
            #[derive(serde::Serialize)]
            struct Factor {
                name: String,
                present: bool,
            }
            let dr = DiffResult {
                id: id.to_string(),
                tier,
                label: tier_label(tier).to_string(),
                factors: factors
                    .iter()
                    .map(|(n, p)| Factor {
                        name: n.to_string(),
                        present: *p,
                    })
                    .collect(),
            };
            let json = serde_json::to_string_pretty(&dr)
                .map_err(|e| Error::Internal(format!("JSON: {e}")))?;
            println!("{json}");
        }
    }
    Ok(())
}

/// Classify all corpus entries and show tier distribution.
pub(crate) fn corpus_classify_all(
    registry: &crate::corpus::registry::CorpusRegistry,
    format: &CorpusOutputFormat,
) -> Result<()> {
    let mut tier_counts = [0u32; 6]; // index 0 unused, 1-5
    let mut format_tiers: std::collections::HashMap<String, [u32; 6]> =
        std::collections::HashMap::new();

    for entry in &registry.entries {
        let (tier, _) = classify_difficulty(&entry.input);
        tier_counts[tier as usize] += 1;
        let fmt_key = entry.id.chars().next().unwrap_or('?').to_string();
        let ft = format_tiers.entry(fmt_key).or_insert([0u32; 6]);
        ft[tier as usize] += 1;
    }

    match format {
        CorpusOutputFormat::Human => {
            use crate::cli::color::*;
            println!(
                "{BOLD}Corpus Tier Distribution{RESET} ({} entries)",
                registry.entries.len()
            );
            println!("{DIM}════════════════════════════════════════{RESET}");
            println!(
                "{DIM}{:>6}  {:<15} {:>7} {:>16}{RESET}",
                "Tier", "Label", "Count", "Bar"
            );
            for t in 1..=5u8 {
                let count = tier_counts[t as usize];
                let pct = if registry.entries.is_empty() {
                    0.0
                } else {
                    count as f64 / registry.entries.len() as f64 * 100.0
                };
                let bar = stats_bar(pct, 16);
                let tc = match t {
                    1 => GREEN,
                    2 => CYAN,
                    3 => YELLOW,
                    4 => BRIGHT_RED,
                    _ => BRIGHT_CYAN,
                };
                println!(
                    "  {tc}{t:>4}{RESET}  {:<15} {:>7} {tc}{bar}{RESET}",
                    tier_label(t),
                    count
                );
            }

            // Per-format breakdown
            println!();
            println!("{BOLD}Per-Format Breakdown:{RESET}");
            for (key, label) in [("B", "Bash"), ("M", "Makefile"), ("D", "Dockerfile")] {
                if let Some(ft) = format_tiers.get(key) {
                    let parts: Vec<String> = (1..=5u8)
                        .filter(|&t| ft[t as usize] > 0)
                        .map(|t| format!("T{t}:{}", ft[t as usize]))
                        .collect();
                    if !parts.is_empty() {
                        println!("  {DIM}{label}:{RESET} {}", parts.join(", "));
                    }
                }
            }
        }
        CorpusOutputFormat::Json => {
            #[derive(serde::Serialize)]
            struct AllResult {
                total: usize,
                tiers: Vec<TierCount>,
            }
            #[derive(serde::Serialize)]
            struct TierCount {
                tier: u8,
                label: String,
                count: u32,
            }
            let result = AllResult {
                total: registry.entries.len(),
                tiers: (1..=5u8)
                    .map(|t| TierCount {
                        tier: t,
                        label: tier_label(t).to_string(),
                        count: tier_counts[t as usize],
                    })
                    .collect(),
            };
            let json = serde_json::to_string_pretty(&result)
                .map_err(|e| Error::Internal(format!("JSON: {e}")))?;
            println!("{json}");
        }
    }
    Ok(())
}

/// Classify a V2 dimension failure by risk level (spec §11.10.4).
pub(crate) fn dimension_risk(dim: &str) -> &'static str {
    match dim {
        "A" => "HIGH",   // transpilation failure = can't use output at all
        "B3" => "HIGH",  // behavioral = execution fails/hangs
        "E" => "HIGH",   // non-deterministic = unreliable output
        "D" => "MEDIUM", // lint violations = quality issue
        "G" => "MEDIUM", // cross-shell = portability issue
        "F" => "MEDIUM", // metamorphic = consistency issue
        "B1" => "LOW",   // containment = output semantics
        "B2" => "LOW",   // exact match = cosmetic
        _ => "LOW",
    }
}

/// Collect classified failures from corpus results, optionally filtered by risk level.
pub(crate) fn collect_risk_failures<'a>(
    results: &'a [crate::corpus::runner::CorpusResult],
    level_filter: Option<&str>,
) -> Vec<(&'a str, &'static str, &'static str)> {
    let mut classified = Vec::new();
    for r in results {
        for dim in result_fail_dims(r) {
            let risk = dimension_risk(dim);
            if level_filter.is_none_or(|f| risk.eq_ignore_ascii_case(f)) {
                classified.push((r.id.as_str(), dim, risk));
            }
        }
    }
    classified
}

/// Print risk group for a given level.
pub(crate) fn risk_print_group(
    classified: &[(&str, &str, &str)],
    label: &str,
    color: &str,
    count: usize,
) {
    use crate::cli::color::*;
    if count == 0 {
        return;
    }
    println!("  {color}{BOLD}{label}{RESET} ({count}):");
    for (id, dim, risk) in classified {
        if *risk == label {
            println!("    {color}{id}{RESET} — {dim}");
        }
    }
    println!();
}

/// Risk analysis: classify corpus failures by HIGH/MEDIUM/LOW risk (spec §11.10.4).
pub(crate) fn corpus_risk_analysis(
    format: &CorpusOutputFormat,
    level_filter: Option<&str>,
) -> Result<()> {
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;

    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());
    let score = runner.run(&registry);

    let classified = collect_risk_failures(&score.results, level_filter);
    let high = classified.iter().filter(|(_, _, r)| *r == "HIGH").count();
    let medium = classified.iter().filter(|(_, _, r)| *r == "MEDIUM").count();
    let low = classified.iter().filter(|(_, _, r)| *r == "LOW").count();

    match format {
        CorpusOutputFormat::Human => {
            use crate::cli::color::*;
            println!("{BOLD}Risk Classification: Corpus Failures{RESET}");
            println!(
                "{DIM}Total failures: {} (HIGH: {high}, MEDIUM: {medium}, LOW: {low}){RESET}",
                classified.len()
            );
            println!();
            if classified.is_empty() {
                println!("  {GREEN}No failures to classify.{RESET}");
                return Ok(());
            }
            risk_print_group(&classified, "HIGH", BRIGHT_RED, high);
            risk_print_group(&classified, "MEDIUM", YELLOW, medium);
            risk_print_group(&classified, "LOW", DIM, low);
        }
        CorpusOutputFormat::Json => {
            let result = serde_json::json!({
                "total": classified.len(),
                "high": high, "medium": medium, "low": low,
                "failures": classified.iter().map(|(id, dim, risk)| serde_json::json!({
                    "id": id, "dimension": dim, "risk": risk,
                })).collect::<Vec<_>>(),
            });
            let json = serde_json::to_string_pretty(&result)
                .map_err(|e| Error::Internal(format!("JSON: {e}")))?;
            println!("{json}");
        }
    }
    Ok(())
}
