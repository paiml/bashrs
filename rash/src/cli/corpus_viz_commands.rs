//! Corpus visualization: scatter plots, grade distribution, pivot tables, correlation, schema, and history charts.

use crate::models::{Config, Error, Result};
use super::corpus_failure_commands::result_fail_dims;

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
    use crate::cli::color::*;
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;
    use std::time::Instant;

    let registry = CorpusRegistry::load_full();
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
    use crate::cli::color::*;
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::registry::Grade;
    use crate::corpus::runner::CorpusRunner;

    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());
    let score = runner.run(&registry);

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
    use crate::cli::color::*;
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;

    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());
    let score = runner.run(&registry);

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
    use crate::cli::color::*;
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;

    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());
    let score = runner.run(&registry);

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

/// Schema enforcement layer status per format (spec §11.8).
pub(crate) fn corpus_schema() -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;

    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());
    let score = runner.run(&registry);

    let formats = ["Bash", "Makefile", "Dockerfile"];

    println!("{BOLD}Schema Enforcement Layers{RESET} (spec §11.8)");
    println!();
    println!(
        "  {BOLD}{:<12} {:>8} {:>8} {:>8} {:>8} {:>8}{RESET}",
        "Format", "Total", "L1:Lex", "L2:Syn", "L3:Sem", "L4:Beh"
    );

    for fmt_name in &formats {
        let fmt_filter = match *fmt_name {
            "Bash" => crate::corpus::registry::CorpusFormat::Bash,
            "Makefile" => crate::corpus::registry::CorpusFormat::Makefile,
            _ => crate::corpus::registry::CorpusFormat::Dockerfile,
        };

        let entries: Vec<_> = registry
            .entries
            .iter()
            .enumerate()
            .filter(|(_, e)| e.format == fmt_filter)
            .collect();

        let total = entries.len();
        let (l1, l2, l3, l4) = schema_layer_counts(&score.results, &entries);

        let l1c = pct_color(l1 as f64 / total.max(1) as f64 * 100.0);
        let l2c = pct_color(l2 as f64 / total.max(1) as f64 * 100.0);
        let l3c = pct_color(l3 as f64 / total.max(1) as f64 * 100.0);
        let l4c = pct_color(l4 as f64 / total.max(1) as f64 * 100.0);

        println!("  {CYAN}{:<12}{RESET} {:>8} {l1c}{:>8}{RESET} {l2c}{:>8}{RESET} {l3c}{:>8}{RESET} {l4c}{:>8}{RESET}",
            fmt_name, total, l1, l2, l3, l4);
    }

    println!();
    println!("  {DIM}L1=Lexical(transpile) L2=Syntactic(lint) L3=Semantic(det+meta) L4=Behavioral(exec+cross-shell){RESET}");

    Ok(())
}

/// ASCII chart of score over iterations from convergence log.
pub(crate) fn corpus_history_chart() -> Result<()> {
    use crate::cli::color::*;

    let log_path = std::path::Path::new(".quality/convergence.log");
    if !log_path.exists() {
        let log_path2 = std::path::Path::new("../.quality/convergence.log");
        if !log_path2.exists() {
            println!("{YELLOW}No convergence log found{RESET}");
            return Ok(());
        }
        return corpus_history_chart_from(log_path2);
    }
    corpus_history_chart_from(log_path)
}

/// Render a single chart cell for history chart.
pub(crate) fn history_chart_cell(score: f64, row: usize, min_score: f64, range: f64, height: usize) {
    use crate::cli::color::*;
    if score <= 0.0 {
        print!(" ");
    } else {
        let normalized = (score - min_score) / range * height as f64;
        if normalized >= row as f64 {
            let color = if score >= 99.0 {
                GREEN
            } else if score >= 95.0 {
                YELLOW
            } else {
                RED
            };
            print!("{color}█{RESET}");
        } else {
            print!(" ");
        }
    }
}


pub(crate) fn corpus_history_chart_from(path: &std::path::Path) -> Result<()> {
    use crate::cli::color::*;

    let content = std::fs::read_to_string(path).map_err(Error::Io)?;

    let entries: Vec<crate::corpus::runner::ConvergenceEntry> = content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect();

    if entries.is_empty() {
        println!("{YELLOW}No convergence entries found{RESET}");
        return Ok(());
    }

    println!(
        "{BOLD}Score History Chart{RESET} ({} iterations)",
        entries.len()
    );
    println!();

    let scores: Vec<f64> = entries
        .iter()
        .filter_map(|e| if e.score > 0.0 { Some(e.score) } else { None })
        .collect();

    let min_score = scores.iter().copied().fold(f64::MAX, f64::min);
    let max_score = scores.iter().copied().fold(0.0f64, f64::max);
    let chart_height = 10usize;
    let range = (max_score - min_score).max(0.1);

    for row in (0..chart_height).rev() {
        let threshold = min_score + range * row as f64 / chart_height as f64;
        print!("  {DIM}{:>6.1}{RESET} │", threshold);
        for entry in &entries {
            history_chart_cell(entry.score, row, min_score, range, chart_height);
        }
        println!();
    }

    print!("  {DIM}{:>6}{RESET} └", "");
    for _ in &entries {
        print!("─");
    }
    println!();
    print!("  {DIM}{:>6}  ", "");
    for entry in &entries {
        print!("{}", entry.iteration % 10);
    }
    println!("{RESET}");

    println!();
    if let Some(last) = entries.last() {
        println!(
            "  {DIM}Latest: iter {} | {:.1}/100 | {} entries{RESET}",
            last.iteration, last.score, last.total
        );
    }

    Ok(())
}
