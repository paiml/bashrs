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
