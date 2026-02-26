//! Corpus comparison commands: health, compare, density, performance, CITL, and streak.

use super::corpus_failure_commands::result_fail_dims;
use crate::cli::args::CorpusFormatArg;
use crate::models::{Config, Error, Result};

pub(crate) fn corpus_health() -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;

    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());
    let score = runner.run(&registry);

    let grade_str = score.grade.to_string();
    let gc = grade_color(&grade_str);

    // Per-format compact
    let fmt_parts: Vec<String> = score
        .format_scores
        .iter()
        .map(|fs| format!("{}:{}/{}", fs.format, fs.passed, fs.total))
        .collect();

    let status = if score.failed == 0 {
        "HEALTHY"
    } else {
        "DEGRADED"
    };
    let status_color = if score.failed == 0 { GREEN } else { YELLOW };

    println!("{status_color}{status}{RESET} | {WHITE}{:.1}/100{RESET} {gc}{grade_str}{RESET} | {}/{} passed | {} | {DIM}failures:{}{RESET}",
        score.score, score.passed, score.total, fmt_parts.join(" "), score.failed);
    Ok(())
}

/// Compare two corpus entries side-by-side.
pub(crate) fn corpus_compare(id1: &str, id2: &str) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;
    use std::time::Instant;

    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());

    let entry1 = registry
        .entries
        .iter()
        .find(|e| e.id == id1)
        .ok_or_else(|| Error::Validation(format!("Entry '{id1}' not found")))?;
    let entry2 = registry
        .entries
        .iter()
        .find(|e| e.id == id2)
        .ok_or_else(|| Error::Validation(format!("Entry '{id2}' not found")))?;

    let start1 = Instant::now();
    let r1 = runner.run_single(entry1);
    let t1 = start1.elapsed().as_secs_f64() * 1000.0;

    let start2 = Instant::now();
    let r2 = runner.run_single(entry2);
    let t2 = start2.elapsed().as_secs_f64() * 1000.0;

    let dims1 = result_fail_dims(&r1);
    let dims2 = result_fail_dims(&r2);

    println!("{BOLD}Corpus Entry Comparison{RESET}");
    println!();
    println!("  {BOLD}{:<18} {:<20} {:<20}{RESET}", "", id1, id2);
    println!("  {:<18} {:<20} {:<20}", "Name", entry1.name, entry2.name);
    println!(
        "  {:<18} {:<20} {:<20}",
        "Format",
        format!("{}", entry1.format),
        format!("{}", entry2.format)
    );
    println!(
        "  {:<18} {:<20} {:<20}",
        "Tier",
        format!("{:?}", entry1.tier),
        format!("{:?}", entry2.tier)
    );
    println!(
        "  {:<18} {:<20} {:<20}",
        "Time",
        format!("{t1:.1}ms"),
        format!("{t2:.1}ms")
    );

    let s1 = if dims1.is_empty() {
        format!("{GREEN}PASS{RESET}")
    } else {
        format!("{RED}FAIL{RESET}")
    };
    let s2 = if dims2.is_empty() {
        format!("{GREEN}PASS{RESET}")
    } else {
        format!("{RED}FAIL{RESET}")
    };
    println!("  {:<18} {:<20} {:<20}", "Status", s1, s2);
    println!(
        "  {:<18} {:<20} {:<20}",
        "Pass Dims",
        format!("{}/9", 9 - dims1.len()),
        format!("{}/9", 9 - dims2.len())
    );

    // Per-dimension comparison
    println!();
    println!("  {BOLD}Dimension Comparison:{RESET}");
    let dim_names = [
        "A-Transpile",
        "B1-Contain",
        "B2-Exact",
        "B3-Behav",
        "D-Lint",
        "E-Determ",
        "F-Meta",
        "G-XShell",
    ];
    let bools1 = [
        r1.transpiled,
        r1.output_contains,
        r1.output_exact,
        r1.output_behavioral,
        r1.lint_clean,
        r1.deterministic,
        r1.metamorphic_consistent,
        r1.cross_shell_agree,
    ];
    let bools2 = [
        r2.transpiled,
        r2.output_contains,
        r2.output_exact,
        r2.output_behavioral,
        r2.lint_clean,
        r2.deterministic,
        r2.metamorphic_consistent,
        r2.cross_shell_agree,
    ];

    for i in 0..dim_names.len() {
        let v1 = if bools1[i] {
            format!("{GREEN}\u{2713}{RESET}")
        } else {
            format!("{RED}\u{2717}{RESET}")
        };
        let v2 = if bools2[i] {
            format!("{GREEN}\u{2713}{RESET}")
        } else {
            format!("{RED}\u{2717}{RESET}")
        };
        let diff = if bools1[i] != bools2[i] {
            format!(" {YELLOW}<-{RESET}")
        } else {
            String::new()
        };
        println!("    {:<14} {:>6}  {:>6}{diff}", dim_names[i], v1, v2);
    }
    Ok(())
}

/// Show entry density by ID range (detect numbering gaps).
pub(crate) fn corpus_density() -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::registry::{CorpusFormat, CorpusRegistry};

    let registry = CorpusRegistry::load_full();

    let formats = [
        ("B", CorpusFormat::Bash, 500),
        ("M", CorpusFormat::Makefile, 200),
        ("D", CorpusFormat::Dockerfile, 200),
    ];

    println!("{BOLD}Corpus Entry Density{RESET}");
    println!();

    for (prefix, fmt, max_id) in &formats {
        let ids: std::collections::BTreeSet<usize> = registry
            .entries
            .iter()
            .filter(|e| e.format == *fmt)
            .filter_map(|e| {
                e.id.strip_prefix(&format!("{prefix}-"))
                    .and_then(|n| n.parse::<usize>().ok())
            })
            .collect();

        let count = ids.len();
        let min_id = ids.iter().next().copied().unwrap_or(1);
        let max_found = ids.iter().next_back().copied().unwrap_or(0);
        let expected_range = max_found - min_id + 1;
        let gaps: Vec<usize> = (min_id..=max_found).filter(|n| !ids.contains(n)).collect();

        let density = if expected_range > 0 {
            count as f64 / expected_range as f64 * 100.0
        } else {
            0.0
        };
        let dc = if density >= 99.0 {
            GREEN
        } else if density >= 90.0 {
            YELLOW
        } else {
            RED
        };

        println!("  {CYAN}{prefix}{RESET}-{min_id:03}..{prefix}-{max_found:03} ({count}/{max_id} target)");
        println!(
            "    Density: {dc}{density:.1}%{RESET}  ({count} present / {expected_range} in range)"
        );
        if gaps.is_empty() {
            println!("    {GREEN}No gaps detected.{RESET}");
        } else if gaps.len() <= 10 {
            let gap_strs: Vec<String> = gaps.iter().map(|g| format!("{prefix}-{g:03}")).collect();
            println!(
                "    {YELLOW}Gaps ({}):{RESET} {}",
                gaps.len(),
                gap_strs.join(", ")
            );
        } else {
            let first_gaps: Vec<String> = gaps
                .iter()
                .take(5)
                .map(|g| format!("{prefix}-{g:03}"))
                .collect();
            println!(
                "    {YELLOW}Gaps ({}):{RESET} {}... +{} more",
                gaps.len(),
                first_gaps.join(", "),
                gaps.len() - 5
            );
        }
        println!();
    }
    Ok(())
}

/// Compute percentile from sorted data.
pub(crate) fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = (p / 100.0 * (sorted.len() - 1) as f64).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

/// Performance percentile breakdown (P50, P90, P95, P99) per format.
fn perf_ms_color(ms: f64) -> &'static str {
    use crate::cli::color::*;
    if ms > 1000.0 {
        BRIGHT_RED
    } else if ms > 100.0 {
        YELLOW
    } else {
        GREEN
    }
}

fn print_perf_row(label: &str, label_color: &str, timings: &[f64]) {
    use crate::cli::color::RESET;
    let pcts = [50.0, 90.0, 95.0, 99.0];
    let mean = timings.iter().sum::<f64>() / timings.len().max(1) as f64;
    let max = timings.last().copied().unwrap_or(0.0);
    print!("  {label_color}{:<12}{RESET}", label);
    for p in &pcts {
        let v = percentile(timings, *p);
        let color = perf_ms_color(v);
        print!(" {color}{:>7.1}ms{RESET}", v);
    }
    let mc = perf_ms_color(max);
    println!(" {mc}{:>7.1}ms{RESET} {:>7.1}ms", max, mean);
}

pub(crate) fn corpus_perf(filter: Option<&CorpusFormatArg>) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::registry::{CorpusFormat, CorpusRegistry};
    use crate::corpus::runner::CorpusRunner;
    use std::time::Instant;

    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());

    let entries: Vec<_> = registry
        .entries
        .iter()
        .filter(|e| match filter {
            Some(CorpusFormatArg::Bash) => e.format == CorpusFormat::Bash,
            Some(CorpusFormatArg::Makefile) => e.format == CorpusFormat::Makefile,
            Some(CorpusFormatArg::Dockerfile) => e.format == CorpusFormat::Dockerfile,
            None => true,
        })
        .collect();

    let mut all_timings: Vec<f64> = Vec::new();
    let mut format_timings: std::collections::HashMap<String, Vec<f64>> =
        std::collections::HashMap::new();

    for entry in &entries {
        let start = Instant::now();
        let _ = runner.run_single(entry);
        let ms = start.elapsed().as_secs_f64() * 1000.0;
        all_timings.push(ms);
        format_timings
            .entry(format!("{}", entry.format))
            .or_default()
            .push(ms);
    }

    all_timings.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    println!(
        "{BOLD}Performance Percentile Breakdown{RESET} ({} entries)",
        entries.len()
    );
    println!();
    println!(
        "  {BOLD}{:<12} {:>8} {:>8} {:>8} {:>8} {:>8} {:>8}{RESET}",
        "Format", "P50", "P90", "P95", "P99", "Max", "Mean"
    );

    print_perf_row("ALL", WHITE, &all_timings);

    let mut fmt_keys: Vec<_> = format_timings.keys().cloned().collect();
    fmt_keys.sort();
    for key in &fmt_keys {
        let mut ts = format_timings[key].clone();
        ts.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        print_perf_row(key, CYAN, &ts);
    }
    Ok(())
}

/// CITL lint violation summary from transpiled output (spec §7.3).
pub(crate) fn corpus_citl(filter: Option<&CorpusFormatArg>) -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::registry::{CorpusFormat, CorpusRegistry};
    use crate::corpus::runner::CorpusRunner;

    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());

    let entries: Vec<_> = registry
        .entries
        .iter()
        .filter(|e| match filter {
            Some(CorpusFormatArg::Bash) => e.format == CorpusFormat::Bash,
            Some(CorpusFormatArg::Makefile) => e.format == CorpusFormat::Makefile,
            Some(CorpusFormatArg::Dockerfile) => e.format == CorpusFormat::Dockerfile,
            None => true,
        })
        .collect();

    let mut lint_pass = 0usize;
    let mut lint_fail = 0usize;
    let mut fail_entries: Vec<(&str, String)> = Vec::new();

    for entry in &entries {
        let result = runner.run_single(entry);
        if result.lint_clean {
            lint_pass += 1;
        } else {
            lint_fail += 1;
            let err = result.error.as_deref().unwrap_or("lint violation");
            fail_entries.push((&entry.id, err.to_string()));
        }
    }

    let total = entries.len();
    let rate = lint_pass as f64 / total.max(1) as f64 * 100.0;
    let rc = pct_color(rate);

    println!("{BOLD}CITL Lint Compliance{RESET} (spec §7.3)");
    println!();
    println!("  Entries: {total}  Pass: {GREEN}{lint_pass}{RESET}  Fail: {}{}{}  Rate: {rc}{rate:.1}%{RESET}",
        if lint_fail > 0 { RED } else { GREEN }, lint_fail, RESET);
    println!();

    if fail_entries.is_empty() {
        println!("  {GREEN}All transpiled outputs pass CITL lint gate.{RESET}");
    } else {
        println!("  {BOLD}Lint Violations:{RESET}");
        for (id, err) in &fail_entries {
            let short_err = if err.len() > 60 { &err[..60] } else { err };
            println!("    {CYAN}{id}{RESET}  {DIM}{short_err}{RESET}");
        }
    }

    println!();
    println!("  {DIM}CITL loop: transpile → lint → score → feedback{RESET}");
    Ok(())
}

/// Show longest streak of consecutive passing entries.
pub(crate) fn corpus_streak() -> Result<()> {
    use crate::cli::color::*;
    use crate::corpus::registry::{CorpusFormat, CorpusRegistry};
    use crate::corpus::runner::CorpusRunner;

    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(Config::default());

    let formats = [
        ("Bash", CorpusFormat::Bash),
        ("Makefile", CorpusFormat::Makefile),
        ("Dockerfile", CorpusFormat::Dockerfile),
    ];

    println!("{BOLD}Consecutive Pass Streaks{RESET}");
    println!();

    for (name, fmt) in &formats {
        let mut entries: Vec<_> = registry
            .entries
            .iter()
            .filter(|e| e.format == *fmt)
            .collect();
        entries.sort_by(|a, b| a.id.cmp(&b.id));

        let mut current_streak = 0usize;
        let mut max_streak = 0usize;
        let mut max_start = "";
        let mut max_end = "";
        let mut cur_start = "";

        for entry in &entries {
            let result = runner.run_single(entry);
            let pass = result_fail_dims(&result).is_empty();
            if pass {
                if current_streak == 0 {
                    cur_start = &entry.id;
                }
                current_streak += 1;
                if current_streak > max_streak {
                    max_streak = current_streak;
                    max_start = cur_start;
                    max_end = &entry.id;
                }
            } else {
                current_streak = 0;
            }
        }

        let total = entries.len();
        let pct = max_streak as f64 / total.max(1) as f64 * 100.0;
        let sc = if max_streak == total {
            GREEN
        } else if pct >= 90.0 {
            YELLOW
        } else {
            RED
        };
        println!("  {CYAN}{name:<12}{RESET} {sc}{max_streak}{RESET}/{total} ({sc}{pct:.1}%{RESET})  {DIM}{max_start}..{max_end}{RESET}");
    }
    Ok(())
}
