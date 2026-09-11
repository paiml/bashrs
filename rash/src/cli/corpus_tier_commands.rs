//! Corpus tier management: tier detail, ID ranges, tier listing, fail maps, and score ranges.

use super::corpus_diag_commands::result_dim_pass;
use super::corpus_failure_commands::result_fail_dims;
use crate::models::{Config, Result};

pub(crate) fn corpus_tier_detail() -> Result<()> {
    use crate::corpus::registry::CorpusRegistry;
    corpus_tier_detail_with(&CorpusRegistry::load_full())
}

/// PMAT-257: body of `corpus_tier_detail`, split so a test can pass a small
/// synthetic registry instead of running the full corpus through a
/// `CorpusRunner`.
pub(crate) fn corpus_tier_detail_with(
    registry: &crate::corpus::registry::CorpusRegistry,
) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::runner::CorpusRunner;

    let runner = CorpusRunner::new(Config::default());
    let score = runner.run(registry);

    let tiers = [
        ("Trivial", crate::corpus::registry::CorpusTier::Trivial),
        ("Standard", crate::corpus::registry::CorpusTier::Standard),
        ("Complex", crate::corpus::registry::CorpusTier::Complex),
        (
            "Adversarial",
            crate::corpus::registry::CorpusTier::Adversarial,
        ),
        (
            "Production",
            crate::corpus::registry::CorpusTier::Production,
        ),
    ];

    println!("{BOLD}Tier Detail{RESET}");
    println!();

    for (name, tier) in &tiers {
        let entries: Vec<_> = registry
            .entries
            .iter()
            .enumerate()
            .filter(|(_, e)| e.tier == *tier)
            .collect();

        let total = entries.len();
        let passed = entries
            .iter()
            .filter(|(i, _)| {
                score
                    .results
                    .get(*i)
                    .is_some_and(|r| result_fail_dims(r).is_empty())
            })
            .count();
        let rate = if total > 0 {
            passed as f64 / total as f64 * 100.0
        } else {
            100.0
        };
        let color = pct_color(rate);

        println!("  {BOLD}{name}{RESET} ({total} entries)");
        println!(
            "    Pass rate: {color}{:.1}%{RESET} ({passed}/{total})",
            rate
        );

        // Per-dimension breakdown for this tier
        let dims = ["A", "B1", "B2", "B3", "D", "E", "F", "G"];
        print!("    Dims:     ");
        for (d_idx, dim) in dims.iter().enumerate() {
            let dim_pass = entries
                .iter()
                .filter(|(i, _)| {
                    score
                        .results
                        .get(*i)
                        .is_some_and(|r| result_dim_pass(r, d_idx))
                })
                .count();
            let dc = if dim_pass == total { GREEN } else { YELLOW };
            print!("{dc}{dim}:{dim_pass}{RESET} ");
        }
        println!();

        // Show failures if any
        let failures: Vec<_> = entries
            .iter()
            .filter(|(i, _)| {
                !score
                    .results
                    .get(*i)
                    .is_none_or(|r| result_fail_dims(r).is_empty())
            })
            .collect();
        if !failures.is_empty() {
            for (i, entry) in &failures {
                let fails = score
                    .results
                    .get(*i)
                    .map_or_else(Vec::new, result_fail_dims);
                println!("    {RED}FAIL{RESET}: {} [{}]", entry.id, fails.join(","));
            }
        }
        println!();
    }

    Ok(())
}

/// ID range info per format (first, last, count).
pub(crate) fn corpus_id_range() -> Result<()> {
    use crate::corpus::registry::CorpusRegistry;
    corpus_id_range_with(&CorpusRegistry::load_full())
}

/// PMAT-257: body of `corpus_id_range`, split so a test can pass a small
/// synthetic registry instead of the full corpus.
pub(crate) fn corpus_id_range_with(
    registry: &crate::corpus::registry::CorpusRegistry,
) -> Result<()> {
    use crate::cli::color::*;

    println!("{BOLD}ID Range by Format{RESET}");
    println!();

    let formats = [
        ("Bash", "B-", crate::corpus::registry::CorpusFormat::Bash),
        (
            "Makefile",
            "M-",
            crate::corpus::registry::CorpusFormat::Makefile,
        ),
        (
            "Dockerfile",
            "D-",
            crate::corpus::registry::CorpusFormat::Dockerfile,
        ),
    ];

    println!(
        "  {BOLD}{:<12} {:>6} {:>8} {:>8} {:>8}{RESET}",
        "Format", "Count", "First", "Last", "Max#"
    );

    for (name, prefix, fmt) in &formats {
        let ids: Vec<&str> = registry
            .entries
            .iter()
            .filter(|e| e.format == *fmt)
            .map(|e| e.id.as_str())
            .collect();

        let count = ids.len();
        let nums: Vec<usize> = ids
            .iter()
            .filter_map(|id| id.strip_prefix(prefix).and_then(|n| n.parse().ok()))
            .collect();

        let first = ids.first().copied().unwrap_or("-");
        let last = ids.last().copied().unwrap_or("-");
        let max_num = nums.iter().copied().max().unwrap_or(0);

        println!(
            "  {CYAN}{:<12}{RESET} {:>6} {:>8} {:>8} {:>8}",
            name, count, first, last, max_num
        );
    }

    println!();
    println!("  {DIM}Max# = highest numeric ID in range{RESET}");

    Ok(())
}

/// Compact tier summary table.
pub(crate) fn corpus_tiers() -> Result<()> {
    use crate::corpus::registry::CorpusRegistry;
    corpus_tiers_with(&CorpusRegistry::load_full())
}

/// PMAT-257: body of `corpus_tiers`, split so a test can pass a small
/// synthetic registry instead of running the full corpus through a
/// `CorpusRunner`.
pub(crate) fn corpus_tiers_with(registry: &crate::corpus::registry::CorpusRegistry) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::runner::CorpusRunner;

    let runner = CorpusRunner::new(Config::default());
    let score = runner.run(registry);

    let tiers = [
        ("Trivial", crate::corpus::registry::CorpusTier::Trivial, 1.0),
        (
            "Standard",
            crate::corpus::registry::CorpusTier::Standard,
            1.5,
        ),
        ("Complex", crate::corpus::registry::CorpusTier::Complex, 2.0),
        (
            "Adversarial",
            crate::corpus::registry::CorpusTier::Adversarial,
            2.5,
        ),
        (
            "Production",
            crate::corpus::registry::CorpusTier::Production,
            3.0,
        ),
    ];

    println!("{BOLD}Tier Summary{RESET}");
    println!();
    println!(
        "  {BOLD}{:<14} {:>6} {:>6} {:>7} {:>6}{RESET}",
        "Tier", "Count", "Pass", "Rate", "Weight"
    );

    for (name, tier, weight) in &tiers {
        let entries: Vec<_> = registry
            .entries
            .iter()
            .enumerate()
            .filter(|(_, e)| e.tier == *tier)
            .collect();
        let total = entries.len();
        let passed = entries
            .iter()
            .filter(|(i, _)| {
                score
                    .results
                    .get(*i)
                    .is_some_and(|r| result_fail_dims(r).is_empty())
            })
            .count();
        let rate = if total > 0 {
            passed as f64 / total as f64 * 100.0
        } else {
            100.0
        };
        let color = pct_color(rate);
        println!(
            "  {CYAN}{:<14}{RESET} {:>6} {color}{:>6}{RESET} {color}{:>6.1}%{RESET} {:>6.1}x",
            name, total, passed, rate, weight
        );
    }

    Ok(())
}

/// Map of failing entries with dimension failures.
pub(crate) fn corpus_fail_map() -> Result<()> {
    use crate::corpus::registry::CorpusRegistry;
    corpus_fail_map_with(&CorpusRegistry::load_full())
}

/// PMAT-257: body of `corpus_fail_map`, split so a test can pass a small
/// synthetic registry instead of running the full corpus through a
/// `CorpusRunner`.
pub(crate) fn corpus_fail_map_with(
    registry: &crate::corpus::registry::CorpusRegistry,
) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::runner::CorpusRunner;

    let runner = CorpusRunner::new(Config::default());
    let score = runner.run(registry);

    let failures: Vec<_> = registry
        .entries
        .iter()
        .enumerate()
        .filter_map(|(i, entry)| {
            score.results.get(i).and_then(|r| {
                let fails = result_fail_dims(r);
                if fails.is_empty() {
                    None
                } else {
                    Some((entry, fails))
                }
            })
        })
        .collect();

    println!("{BOLD}Failure Map{RESET}");
    println!();

    if failures.is_empty() {
        println!(
            "  {GREEN}No failures across all {} entries{RESET}",
            score.total
        );
        println!("  {DIM}(Note: B-143 shows 0 dimension failures because pass/fail{RESET}");
        println!("  {DIM} is evaluated per entry, not per dimension for this view){RESET}");
    } else {
        println!(
            "  {BOLD}{:<10} {:<12} {:<14} Failed Dimensions{RESET}",
            "ID", "Format", "Tier"
        );
        for (entry, fails) in &failures {
            let fail_str = fails.join(", ");
            println!(
                "  {RED}{:<10}{RESET} {:<12} {:<14} {YELLOW}{fail_str}{RESET}",
                entry.id,
                format!("{}", entry.format),
                format!("{:?}", entry.tier)
            );
        }
    }

    println!();
    println!(
        "  {DIM}Total: {} failures out of {} entries{RESET}",
        failures.len(),
        score.total
    );

    Ok(())
}

/// Score range analysis: min, max, median, IQR per format.
pub(crate) fn corpus_score_range() -> Result<()> {
    use crate::corpus::registry::CorpusRegistry;
    corpus_score_range_with(&CorpusRegistry::load_full())
}

/// PMAT-257: body of `corpus_score_range`, split so a test can pass a small
/// synthetic registry instead of running the full corpus through a
/// `CorpusRunner`.
pub(crate) fn corpus_score_range_with(
    registry: &crate::corpus::registry::CorpusRegistry,
) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::runner::CorpusRunner;

    let runner = CorpusRunner::new(Config::default());
    let score = runner.run(registry);

    println!("{BOLD}Score Range Analysis{RESET}");
    println!();

    // Per-format score ranges (using coverage_ratio as a proxy for per-entry quality)
    let formats = [
        ("Bash", crate::corpus::registry::CorpusFormat::Bash),
        ("Makefile", crate::corpus::registry::CorpusFormat::Makefile),
        (
            "Dockerfile",
            crate::corpus::registry::CorpusFormat::Dockerfile,
        ),
    ];

    println!(
        "  {BOLD}{:<12} {:>8} {:>8} {:>8} {:>8} {:>8}{RESET}",
        "Format", "Score", "Pass%", "Dims/9", "Min", "Max"
    );

    for (name, fmt) in &formats {
        let entries: Vec<_> = registry
            .entries
            .iter()
            .enumerate()
            .filter(|(_, e)| e.format == *fmt)
            .collect();

        let total = entries.len();
        let passed = entries
            .iter()
            .filter(|(i, _)| {
                score
                    .results
                    .get(*i)
                    .is_some_and(|r| result_fail_dims(r).is_empty())
            })
            .count();

        // Per-entry dimension pass count
        let dim_passes: Vec<usize> = entries
            .iter()
            .filter_map(|(i, _)| score.results.get(*i).map(|r| 9 - result_fail_dims(r).len()))
            .collect();

        let min_dims = dim_passes.iter().copied().min().unwrap_or(9);
        let max_dims = dim_passes.iter().copied().max().unwrap_or(9);

        let rate = if total > 0 {
            passed as f64 / total as f64 * 100.0
        } else {
            100.0
        };
        let avg_dims = if dim_passes.is_empty() {
            9.0
        } else {
            dim_passes.iter().sum::<usize>() as f64 / dim_passes.len() as f64
        };

        let fmt_score = score
            .format_scores
            .iter()
            .find(|fs| fs.format == *fmt)
            .map_or(0.0, |fs| fs.score);

        let color = pct_color(rate);
        println!("  {CYAN}{:<12}{RESET} {color}{:>7.1}{RESET} {color}{:>7.1}%{RESET} {:>7.1} {:>8} {:>8}",
            name, fmt_score, rate, avg_dims, min_dims, max_dims);
    }

    // Overall
    println!();
    let total_dims: Vec<usize> = score
        .results
        .iter()
        .map(|r| 9 - result_fail_dims(r).len())
        .collect();
    let min_all = total_dims.iter().copied().min().unwrap_or(9);
    let max_all = total_dims.iter().copied().max().unwrap_or(9);
    let avg_all = if total_dims.is_empty() {
        9.0
    } else {
        total_dims.iter().sum::<usize>() as f64 / total_dims.len() as f64
    };
    println!(
        "  {BOLD}{:<12}{RESET} {:>7.1} {:>7.1}% {:>7.1} {:>8} {:>8}",
        "Overall",
        score.score,
        score.rate * 100.0,
        avg_all,
        min_all,
        max_all
    );

    Ok(())
}

// PMAT-257: coverage for the corpus tier handlers. These live inside the
// library, because the coverage gate measures `cargo llvm-cov --lib -p
// bashrs` and a test under rash/tests/ does not move that number at all.
// Every handler here builds a `CorpusRunner` and scores entries out of the
// real `CorpusRegistry::load_full()` (18,000+ entries) -- too slow for a unit
// test as written. Each was split into a `*_with(registry)` twin that takes
// the registry as a parameter, matching `corpus_compare_commands.rs`.
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
    fn test_PMAT257_cov_tier_detail_with_tiny_registry() {
        corpus_tier_detail_with(&tiny_registry()).expect("tier detail runs over a tiny registry");
    }

    #[test]
    fn test_PMAT257_cov_id_range_with_tiny_registry() {
        corpus_id_range_with(&tiny_registry()).expect("id range runs over a tiny registry");
    }

    #[test]
    fn test_PMAT257_cov_id_range_with_empty_registry() {
        corpus_id_range_with(&CorpusRegistry::new())
            .expect("id range must handle an empty registry");
    }

    #[test]
    fn test_PMAT257_cov_tiers_with_tiny_registry() {
        corpus_tiers_with(&tiny_registry()).expect("tier summary runs over a tiny registry");
    }

    #[test]
    fn test_PMAT257_cov_fail_map_with_tiny_registry() {
        corpus_fail_map_with(&tiny_registry()).expect("fail map runs over a tiny registry");
    }

    #[test]
    fn test_PMAT257_cov_score_range_with_tiny_registry() {
        corpus_score_range_with(&tiny_registry()).expect("score range runs over a tiny registry");
    }
}
