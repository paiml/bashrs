//! Corpus visualization: scatter plots, grade distribution, pivot tables, correlation, schema, and history charts.

use super::corpus_failure_commands::result_fail_dims;
use crate::models::{Config, Error, Result};

/// Map a failure dimension count to a letter grade
pub(super) fn grade_from_fail_count(fail_count: usize) -> &'static str {
    match fail_count {
        0 => "A+",
        1 => "A",
        2 => "B",
        3..=4 => "C",
        5..=6 => "D",
        _ => "F",
    }
}

pub(crate) fn corpus_scatter() -> Result<()> {
    use crate::corpus::registry::CorpusRegistry;

    let registry = CorpusRegistry::load_full();
    corpus_scatter_with(&registry)
}

/// PMAT-257: body of `corpus_scatter`, split so a test can pass a small
/// synthetic registry instead of running the full corpus through a
/// `CorpusRunner`.
pub(crate) fn corpus_scatter_with(
    registry: &crate::corpus::registry::CorpusRegistry,
) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::runner::CorpusRunner;
    use std::time::Instant;

    let runner = CorpusRunner::new(Config::default());

    // Collect timing and failure counts
    let mut data: Vec<(&str, f64, usize)> = Vec::new();
    for entry in &registry.entries {
        let start = Instant::now();
        let result = runner.run_single(entry);
        let ms = start.elapsed().as_secs_f64() * 1000.0;
        let fails = result_fail_dims(&result).len();
        data.push((&entry.id, ms, fails));
    }

    // Bucket into timing ranges
    let ranges = [
        ("< 1ms", 0.0, 1.0),
        ("1-10ms", 1.0, 10.0),
        ("10-50ms", 10.0, 50.0),
        ("50-100ms", 50.0, 100.0),
        ("100-500ms", 100.0, 500.0),
        ("> 500ms", 500.0, f64::MAX),
    ];

    println!(
        "{BOLD}Timing × Failure Scatter{RESET} ({} entries)",
        data.len()
    );
    println!();

    println!(
        "  {BOLD}{:<12} {:>6} {:>6} {:>6}{RESET}",
        "Timing", "0 fail", "1 fail", "2+ fail"
    );

    for (label, lo, hi) in &ranges {
        let in_range: Vec<_> = data
            .iter()
            .filter(|(_, ms, _)| *ms >= *lo && *ms < *hi)
            .collect();
        if in_range.is_empty() {
            continue;
        }
        let f0 = in_range.iter().filter(|(_, _, f)| *f == 0).count();
        let f1 = in_range.iter().filter(|(_, _, f)| *f == 1).count();
        let f2 = in_range.iter().filter(|(_, _, f)| *f >= 2).count();
        let f0c = if f0 > 0 { GREEN } else { DIM };
        let f1c = if f1 > 0 { YELLOW } else { DIM };
        let f2c = if f2 > 0 { RED } else { DIM };
        println!(
            "  {CYAN}{:<12}{RESET} {f0c}{:>6}{RESET} {f1c}{:>6}{RESET} {f2c}{:>6}{RESET}",
            label, f0, f1, f2
        );
    }

    // Summary
    let total_pass = data.iter().filter(|(_, _, f)| *f == 0).count();
    let total_fail = data.len() - total_pass;
    println!();
    println!(
        "  {DIM}Pass: {total_pass} | Fail: {total_fail} | Entries: {}{RESET}",
        data.len()
    );
    Ok(())
}

/// Grade distribution histogram across all entries.
pub(crate) fn corpus_grade_dist() -> Result<()> {
    use crate::corpus::registry::CorpusRegistry;

    let registry = CorpusRegistry::load_full();
    corpus_grade_dist_with(&registry)
}

/// PMAT-257: body of `corpus_grade_dist`, split so a test can pass a small
/// synthetic registry instead of running the full corpus through a
/// `CorpusRunner`.
pub(crate) fn corpus_grade_dist_with(
    registry: &crate::corpus::registry::CorpusRegistry,
) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::registry::Grade;
    use crate::corpus::runner::CorpusRunner;

    let runner = CorpusRunner::new(Config::default());
    let score = runner.run(registry);

    // Count per-entry grades
    let mut counts: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    for result in &score.results {
        let fail_count = result_fail_dims(result).len();
        let entry_grade = grade_from_fail_count(fail_count);
        *counts.entry(entry_grade.to_string()).or_default() += 1;
    }

    println!(
        "{BOLD}Grade Distribution{RESET} ({} entries)",
        score.results.len()
    );
    println!();

    let max_count = counts.values().copied().max().unwrap_or(1);
    let bar_width = 40;
    let grade_order = ["A+", "A", "B", "C", "D", "F"];

    for grade in &grade_order {
        let count = counts.get(*grade).copied().unwrap_or(0);
        let bar_len = if max_count > 0 {
            count * bar_width / max_count
        } else {
            0
        };
        let bar: String = "█".repeat(bar_len);
        let pct = if score.results.is_empty() {
            0.0
        } else {
            count as f64 / score.results.len() as f64 * 100.0
        };
        let color = match *grade {
            "A+" | "A" => GREEN,
            "B" => YELLOW,
            "C" => BRIGHT_YELLOW,
            _ => RED,
        };
        println!(
            "  {color}{:<3}{RESET} {color}{bar}{RESET} {BOLD}{count:>4}{RESET} ({pct:.1}%)",
            grade
        );
    }

    println!();
    println!(
        "  {DIM}Overall: {:.1}/100 {}{RESET}",
        score.score,
        Grade::from_score(score.score)
    );
    Ok(())
}

/// Pivot table: tier × format cross-tabulation with pass rates.
pub(crate) fn corpus_pivot() -> Result<()> {
    use crate::corpus::registry::CorpusRegistry;

    let registry = CorpusRegistry::load_full();
    corpus_pivot_with(&registry)
}

/// PMAT-257: body of `corpus_pivot`, split so a test can pass a small
/// synthetic registry instead of running the full corpus through a
/// `CorpusRunner`.
pub(crate) fn corpus_pivot_with(registry: &crate::corpus::registry::CorpusRegistry) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::runner::CorpusRunner;

    let runner = CorpusRunner::new(Config::default());
    let score = runner.run(registry);

    // Build tier × format grid
    let tiers = [
        "Trivial",
        "Standard",
        "Complex",
        "Adversarial",
        "Production",
    ];
    let formats = ["Bash", "Makefile", "Dockerfile"];

    // Collect data: (tier_idx, format_str) -> (passed, total)
    let mut grid: std::collections::HashMap<(usize, &str), (usize, usize)> =
        std::collections::HashMap::new();

    for (i, entry) in registry.entries.iter().enumerate() {
        let tier_idx = match entry.tier {
            crate::corpus::registry::CorpusTier::Trivial => 0,
            crate::corpus::registry::CorpusTier::Standard => 1,
            crate::corpus::registry::CorpusTier::Complex => 2,
            crate::corpus::registry::CorpusTier::Adversarial => 3,
            crate::corpus::registry::CorpusTier::Production => 4,
        };
        let fmt_str = match entry.format {
            crate::corpus::registry::CorpusFormat::Bash => "Bash",
            crate::corpus::registry::CorpusFormat::Makefile => "Makefile",
            crate::corpus::registry::CorpusFormat::Dockerfile => "Dockerfile",
        };
        if let Some(result) = score.results.get(i) {
            let passed = result_fail_dims(result).is_empty();
            let cell = grid.entry((tier_idx, fmt_str)).or_insert((0, 0));
            cell.1 += 1;
            if passed {
                cell.0 += 1;
            }
        }
    }

    println!("{BOLD}Tier × Format Pivot{RESET}");
    println!();

    // Header
    print!("  {BOLD}{:<14}", "Tier");
    for fmt in &formats {
        print!("{:>14}", fmt);
    }
    println!("{:>12}{RESET}", "Total");

    // Rows
    for (t_idx, tier) in tiers.iter().enumerate() {
        print!("  {CYAN}{:<14}{RESET}", tier);
        let mut row_pass = 0usize;
        let mut row_total = 0usize;
        for fmt in &formats {
            let (p, t) = grid.get(&(t_idx, *fmt)).copied().unwrap_or((0, 0));
            row_pass += p;
            row_total += t;
            if t == 0 {
                print!("{DIM}{:>14}{RESET}", "-");
            } else {
                let rate = p as f64 / t as f64 * 100.0;
                let color = pct_color(rate);
                print!("{color}{:>5}/{:<4} {:>4.1}%{RESET}", p, t, rate);
            }
        }
        if row_total == 0 {
            println!("{DIM}{:>12}{RESET}", "-");
        } else {
            let rate = row_pass as f64 / row_total as f64 * 100.0;
            let color = pct_color(rate);
            println!("{color}{:>5}/{:<4}{RESET}", row_pass, row_total);
        }
    }

    // Footer totals
    print!("  {BOLD}{:<14}", "Total");
    let mut grand_pass = 0usize;
    let mut grand_total = 0usize;
    for fmt in &formats {
        let (p, t): (usize, usize) = (0..5)
            .map(|ti| grid.get(&(ti, *fmt)).copied().unwrap_or((0, 0)))
            .fold((0, 0), |acc, (p, t)| (acc.0 + p, acc.1 + t));
        grand_pass += p;
        grand_total += t;
        if t == 0 {
            print!("{DIM}{:>14}{RESET}", "-");
        } else {
            let rate = p as f64 / t as f64 * 100.0;
            let color = pct_color(rate);
            print!("{color}{:>5}/{:<4} {:>4.1}%{RESET}", p, t, rate);
        }
    }
    println!("{BOLD}{:>5}/{:<4}{RESET}", grand_pass, grand_total);

    Ok(())
}

/// Dimension correlation matrix: which failures co-occur.
pub(crate) fn corpus_corr() -> Result<()> {
    use crate::corpus::registry::CorpusRegistry;

    let registry = CorpusRegistry::load_full();
    corpus_corr_with(&registry)
}

/// PMAT-257: body of `corpus_corr`, split so a test can pass a small
/// synthetic registry instead of running the full corpus through a
/// `CorpusRunner`.
pub(crate) fn corpus_corr_with(registry: &crate::corpus::registry::CorpusRegistry) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::runner::CorpusRunner;

    let runner = CorpusRunner::new(Config::default());
    let score = runner.run(registry);

    let dims = ["A", "B1", "B2", "B3", "D", "E", "F", "G"];

    // For each result, extract failure bitmask per dimension
    let mut fail_vecs: Vec<[bool; 8]> = Vec::new();
    for result in &score.results {
        let fails = [
            !result.transpiled,
            !result.output_contains,
            !result.output_exact,
            !result.output_behavioral,
            !result.lint_clean,
            !result.deterministic,
            !result.metamorphic_consistent,
            !result.cross_shell_agree,
        ];
        fail_vecs.push(fails);
    }

    println!(
        "{BOLD}Dimension Failure Correlation{RESET} ({} entries)",
        fail_vecs.len()
    );
    println!();
    println!("  {DIM}Shows how often two dimensions fail together (co-occurrence count).{RESET}");
    println!();

    // Header
    print!("  {BOLD}{:<5}", "");
    for dim in &dims {
        print!("{:>5}", dim);
    }
    println!("{RESET}");

    // Correlation matrix (co-occurrence counts)
    for (i, dim_i) in dims.iter().enumerate() {
        print!("  {CYAN}{:<5}{RESET}", dim_i);
        for (j, _) in dims.iter().enumerate() {
            let co = fail_vecs.iter().filter(|f| f[i] && f[j]).count();
            if i == j {
                // Diagonal: total failures for this dimension
                let color = if co == 0 { DIM } else { BRIGHT_RED };
                print!("{color}{co:>5}{RESET}");
            } else if co == 0 {
                print!("{DIM}{:>5}{RESET}", "·");
            } else {
                print!("{YELLOW}{co:>5}{RESET}");
            }
        }
        println!();
    }

    // Summary of entries with multi-dimension failures
    let multi_fail = fail_vecs
        .iter()
        .filter(|f| f.iter().filter(|&&x| x).count() >= 2)
        .count();
    println!();
    println!("  {DIM}Multi-dimension failures: {multi_fail} entries{RESET}");

    Ok(())
}

/// Count entries passing each schema enforcement layer (L1-L4).
pub(crate) fn schema_layer_counts(
    results: &[crate::corpus::runner::CorpusResult],
    indices: &[(usize, &crate::corpus::registry::CorpusEntry)],
) -> (usize, usize, usize, usize) {
    let mut l1 = 0usize;
    let mut l2 = 0usize;
    let mut l3 = 0usize;
    let mut l4 = 0usize;
    for (i, _) in indices {
        if let Some(r) = results.get(*i) {
            if r.transpiled {
                l1 += 1;
            }
            if r.lint_clean {
                l2 += 1;
            }
            if r.deterministic && r.metamorphic_consistent {
                l3 += 1;
            }
            if r.output_behavioral && r.cross_shell_agree {
                l4 += 1;
            }
        }
    }
    (l1, l2, l3, l4)
}

// PMAT-257: coverage for the corpus visualization handlers. These live
// inside the library, because the coverage gate measures `cargo llvm-cov
// --lib -p bashrs` and a test under rash/tests/ does not move that number
// at all. `corpus_scatter`, `corpus_grade_dist`, `corpus_pivot`, and
// `corpus_corr` all build a `CorpusRunner` and score entries out of the
// real `CorpusRegistry::load_full()` (18,000+ entries) -- too slow for a
// unit test as written. Each was split into a `*_with(registry)` twin that
// takes the registry as a parameter, matching `corpus_diag_commands.rs`.
#[cfg(test)]
mod pmat257_cov_tests {
    use super::*;
    use crate::corpus::registry::{CorpusEntry, CorpusFormat, CorpusRegistry, CorpusTier};

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
            CorpusTier::Standard,
            "all:\n\techo hello\n",
            "all:",
        ));
        registry.add(CorpusEntry::new(
            "D-001",
            "hello-dockerfile",
            "PMAT-257 fixture",
            CorpusFormat::Dockerfile,
            CorpusTier::Production,
            "FROM alpine:3.18\nWORKDIR /app\n",
            "FROM alpine:3.18",
        ));
        registry
    }

    #[test]
    fn test_PMAT257_cov_grade_from_fail_count_every_bucket() {
        assert_eq!(grade_from_fail_count(0), "A+");
        assert_eq!(grade_from_fail_count(1), "A");
        assert_eq!(grade_from_fail_count(2), "B");
        assert_eq!(grade_from_fail_count(3), "C");
        assert_eq!(grade_from_fail_count(4), "C");
        assert_eq!(grade_from_fail_count(5), "D");
        assert_eq!(grade_from_fail_count(6), "D");
        assert_eq!(grade_from_fail_count(7), "F");
        assert_eq!(grade_from_fail_count(100), "F");
    }

    #[test]
    fn test_PMAT257_cov_scatter_with_tiny_registry() {
        corpus_scatter_with(&tiny_registry()).expect("scatter must render over a tiny registry");
    }

    #[test]
    fn test_PMAT257_cov_scatter_with_empty_registry() {
        corpus_scatter_with(&CorpusRegistry::new())
            .expect("scatter must not fail on an empty registry");
    }

    #[test]
    fn test_PMAT257_cov_grade_dist_with_tiny_registry() {
        corpus_grade_dist_with(&tiny_registry())
            .expect("grade distribution must render over a tiny registry");
    }

    #[test]
    fn test_PMAT257_cov_grade_dist_with_empty_registry() {
        corpus_grade_dist_with(&CorpusRegistry::new())
            .expect("grade distribution must not fail on an empty registry");
    }

    #[test]
    fn test_PMAT257_cov_pivot_with_tiny_registry() {
        corpus_pivot_with(&tiny_registry()).expect("pivot table must render over a tiny registry");
    }

    #[test]
    fn test_PMAT257_cov_pivot_with_empty_registry() {
        corpus_pivot_with(&CorpusRegistry::new())
            .expect("pivot table must not fail on an empty registry");
    }

    #[test]
    fn test_PMAT257_cov_corr_with_tiny_registry() {
        corpus_corr_with(&tiny_registry())
            .expect("correlation matrix must render over a tiny registry");
    }

    #[test]
    fn test_PMAT257_cov_schema_layer_counts_all_layers_pass() {
        use crate::corpus::runner::CorpusResult;
        let entry = tiny_registry().entries[0].clone();
        let result = CorpusResult {
            transpiled: true,
            lint_clean: true,
            deterministic: true,
            metamorphic_consistent: true,
            output_behavioral: true,
            cross_shell_agree: true,
            ..Default::default()
        };
        let results = vec![result];
        let indices = vec![(0usize, &entry)];
        let (l1, l2, l3, l4) = schema_layer_counts(&results, &indices);
        assert_eq!((l1, l2, l3, l4), (1, 1, 1, 1));
    }

    #[test]
    fn test_PMAT257_cov_schema_layer_counts_missing_index_is_skipped() {
        let entry = tiny_registry().entries[0].clone();
        let results: Vec<crate::corpus::runner::CorpusResult> = Vec::new();
        let indices = vec![(0usize, &entry)];
        let (l1, l2, l3, l4) = schema_layer_counts(&results, &indices);
        assert_eq!((l1, l2, l3, l4), (0, 0, 0, 0));
    }
}

include!("corpus_viz_commands_corpus.rs");
