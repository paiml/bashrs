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

    let registry = CorpusRegistry::load_full();
    corpus_risk_analysis_with(&registry, format, level_filter)
}

/// PMAT-257: body of `corpus_risk_analysis()` split out so a test can pass a
/// small synthetic registry instead of running the full corpus through a
/// `CorpusRunner`.
pub(crate) fn corpus_risk_analysis_with(
    registry: &crate::corpus::registry::CorpusRegistry,
    format: &CorpusOutputFormat,
    level_filter: Option<&str>,
) -> Result<()> {
    use crate::corpus::runner::CorpusRunner;

    let runner = CorpusRunner::new(Config::default());
    let score = runner.run(registry);

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

// PMAT-257: coverage for the difficulty classification and risk-analysis
// handlers. `corpus_classify_difficulty`/`corpus_classify_all` call
// `CorpusRegistry::load_full()` but never build a `CorpusRunner` (no
// transpilation), so they're cheap enough to exercise against the real
// registry directly -- same approach as `corpus_density` in
// corpus_compare_commands.rs. `corpus_risk_analysis` does build a
// `CorpusRunner`, so it was split into `corpus_risk_analysis_with(registry,
// ...)` and is tested against a tiny synthetic registry instead.
#[cfg(test)]
mod pmat257_cov_tests {
    use super::*;
    use crate::corpus::registry::{CorpusEntry, CorpusFormat, CorpusRegistry, CorpusTier};
    use crate::corpus::runner::CorpusResult;

    fn tiny_registry() -> CorpusRegistry {
        let mut registry = CorpusRegistry::new();
        registry.add(CorpusEntry::new(
            "B-001",
            "hello-bash",
            "PMAT-257 fixture",
            CorpusFormat::Bash,
            CorpusTier::Trivial,
            r#"fn main() { let greeting = "hello"; }"#,
            "greeting='hello'",
        ));
        registry.add(CorpusEntry::new(
            "M-001",
            "hello-makefile",
            "PMAT-257 fixture",
            CorpusFormat::Makefile,
            CorpusTier::Trivial,
            "all:\n\techo hello\n",
            "all:",
        ));
        registry.add(CorpusEntry::new(
            "D-001",
            "hello-dockerfile",
            "PMAT-257 fixture",
            CorpusFormat::Dockerfile,
            CorpusTier::Trivial,
            "FROM alpine:3.18\nWORKDIR /app\n",
            "FROM alpine:3.18",
        ));
        registry
    }

    #[test]
    fn test_PMAT257_cov_classify_difficulty_every_tier_factor() {
        // Tier 1: trivial, no loop/fn/nesting.
        let (tier, factors) = classify_difficulty("echo hi");
        assert_eq!(tier, 1);
        assert!(!factors.is_empty());

        // Tier bumped up by loops, multiple functions, pipes, conditionals,
        // deep nesting, escapes, unicode, and unsafe/exec patterns.
        let complex = r#"
fn a() {}
fn b() {}
for i in 1 2 3; do
    if [ "$i" = "1" ]; then
        echo "\n\t special ☃" | eval "unsafe $i"
    fi
done
{ { { { nested } } } }
"#;
        let (tier2, factors2) = classify_difficulty(complex);
        assert!(tier2 >= 3, "complex input should score a higher tier");
        assert!(factors2
            .iter()
            .any(|(label, present)| *label == "Has loops" && *present));
        assert!(factors2
            .iter()
            .any(|(label, present)| *label == "Has multiple functions" && *present));
        assert!(factors2
            .iter()
            .any(|(label, present)| *label == "Has unsafe/exec patterns" && *present));
    }

    #[test]
    fn test_PMAT257_cov_tier_label_all_branches() {
        assert_eq!(tier_label(1), "Trivial");
        assert_eq!(tier_label(2), "Standard");
        assert_eq!(tier_label(3), "Complex");
        assert_eq!(tier_label(4), "Adversarial");
        assert_eq!(tier_label(5), "Production");
        assert_eq!(tier_label(99), "Unknown");
    }

    #[test]
    fn test_PMAT257_cov_classify_difficulty_missing_id_is_an_error() {
        assert!(corpus_classify_difficulty("does-not-exist", &CorpusOutputFormat::Human).is_err());
    }

    #[test]
    fn test_PMAT257_cov_classify_difficulty_all_both_formats() {
        corpus_classify_difficulty("all", &CorpusOutputFormat::Human)
            .expect("classify all (human) over the real registry");
        corpus_classify_difficulty("all", &CorpusOutputFormat::Json)
            .expect("classify all (json) over the real registry");
    }

    #[test]
    fn test_PMAT257_cov_classify_difficulty_single_entry_both_formats() {
        let registry = crate::corpus::registry::CorpusRegistry::load_full();
        let some_id = registry
            .entries
            .first()
            .expect("registry has entries")
            .id
            .clone();
        corpus_classify_difficulty(&some_id, &CorpusOutputFormat::Human)
            .expect("classify a real entry id (human)");
        corpus_classify_difficulty(&some_id, &CorpusOutputFormat::Json)
            .expect("classify a real entry id (json)");
    }

    #[test]
    fn test_PMAT257_cov_classify_all_on_tiny_registry() {
        let registry = tiny_registry();
        corpus_classify_all(&registry, &CorpusOutputFormat::Human)
            .expect("classify all over a tiny registry (human)");
        corpus_classify_all(&registry, &CorpusOutputFormat::Json)
            .expect("classify all over a tiny registry (json)");
    }

    #[test]
    fn test_PMAT257_cov_classify_all_empty_registry() {
        let registry = CorpusRegistry::new();
        corpus_classify_all(&registry, &CorpusOutputFormat::Human)
            .expect("classify all must not divide by zero on an empty registry");
    }

    #[test]
    fn test_PMAT257_cov_dimension_risk_all_branches() {
        assert_eq!(dimension_risk("A"), "HIGH");
        assert_eq!(dimension_risk("B3"), "HIGH");
        assert_eq!(dimension_risk("E"), "HIGH");
        assert_eq!(dimension_risk("D"), "MEDIUM");
        assert_eq!(dimension_risk("G"), "MEDIUM");
        assert_eq!(dimension_risk("F"), "MEDIUM");
        assert_eq!(dimension_risk("B1"), "LOW");
        assert_eq!(dimension_risk("B2"), "LOW");
        assert_eq!(dimension_risk("Z"), "LOW");
    }

    fn failing_result(id: &str) -> CorpusResult {
        CorpusResult {
            id: id.to_string(),
            transpiled: true,
            output_contains: true,
            output_exact: true,
            output_behavioral: true,
            lint_clean: false,
            deterministic: false,
            metamorphic_consistent: true,
            cross_shell_agree: true,
            ..Default::default()
        }
    }

    #[test]
    fn test_PMAT257_cov_collect_risk_failures_no_filter_and_filtered() {
        let results = vec![failing_result("B-1"), failing_result("B-2")];
        let all = collect_risk_failures(&results, None);
        // Two failing dims (D, E) per result => 4 classified entries.
        assert_eq!(all.len(), 4);

        let medium_only = collect_risk_failures(&results, Some("medium"));
        assert!(medium_only.iter().all(|(_, _, risk)| *risk == "MEDIUM"));
        assert!(!medium_only.is_empty());

        let high_only = collect_risk_failures(&results, Some("HIGH"));
        assert!(!high_only.is_empty());
        assert!(high_only.iter().all(|(_, _, risk)| *risk == "HIGH"));

        let low_only = collect_risk_failures(&results, Some("LOW"));
        assert!(low_only.is_empty());
    }

    #[test]
    fn test_PMAT257_cov_risk_print_group_zero_and_nonzero() {
        let classified = vec![("B-1", "D", "MEDIUM"), ("B-2", "E", "HIGH")];
        // count == 0 takes the early-return branch.
        risk_print_group(&classified, "MEDIUM", crate::cli::color::YELLOW, 0);
        // count > 0 walks the loop and prints matching rows.
        risk_print_group(&classified, "HIGH", crate::cli::color::BRIGHT_RED, 1);
    }

    #[test]
    fn test_PMAT257_cov_risk_analysis_with_every_filter_and_format() {
        let registry = tiny_registry();
        for format in [CorpusOutputFormat::Human, CorpusOutputFormat::Json] {
            for filter in [None, Some("HIGH"), Some("MEDIUM"), Some("LOW")] {
                corpus_risk_analysis_with(&registry, &format, filter)
                    .expect("risk analysis runs for every format/filter combination");
            }
        }
    }
}
