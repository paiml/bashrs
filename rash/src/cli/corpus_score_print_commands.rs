//! Corpus score printing, convergence logging, statistics display, and run caching.

use crate::cli::args::CorpusOutputFormat;
use crate::cli::logic::truncate_str;
use crate::models::{Config, Error, Result};
use std::path::PathBuf;

pub(crate) fn corpus_print_score(
    score: &crate::corpus::runner::CorpusScore,
    format: &CorpusOutputFormat,
) -> Result<()> {
    use crate::cli::color::*;

    match format {
        CorpusOutputFormat::Human => {
            let grade_str = score.grade.to_string();
            let gc = grade_color(&grade_str);
            let fail_color = if score.failed == 0 { GREEN } else { BRIGHT_RED };

            // Header box
            let score_str = format!("{:.1}", score.score);
            let pad_len = 18_usize.saturating_sub(score_str.len() + grade_str.len());
            println!("{DIM}╭──────────────────────────────────────────────╮{RESET}");
            println!(
                "{DIM}│{RESET}  V2 Corpus Score: {WHITE}{}/100{RESET} ({gc}{grade_str}{RESET}){:>pad$}{DIM}│{RESET}",
                score_str, "",
                pad = pad_len
            );
            println!(
                "{DIM}│{RESET}  Entries: {} total, {GREEN}{} passed{RESET}, {fail_color}{} failed{RESET} ({:.1}%)  {DIM}│{RESET}",
                score.total, score.passed, score.failed, score.rate * 100.0
            );
            println!("{DIM}╰──────────────────────────────────────────────╯{RESET}");
            println!();

            // Format breakdown
            for fs in &score.format_scores {
                let fgs = fs.grade.to_string();
                let fgc = grade_color(&fgs);
                let pc = pct_color(fs.passed as f64 / fs.total.max(1) as f64 * 100.0);
                println!(
                    "  {CYAN}{:<12}{RESET} {WHITE}{:.1}/100{RESET} ({fgc}{fgs}{RESET}) — {pc}{}/{} passed{RESET}",
                    format!("{}:", fs.format), fs.score, fs.passed, fs.total
                );
            }

            // V2 component breakdown (spec §11.4, §11.12)
            if !score.results.is_empty() {
                let n = score.results.len();
                let a_pass = score.results.iter().filter(|r| r.transpiled).count();
                let b1_pass = score.results.iter().filter(|r| r.output_contains).count();
                let b2_pass = score.results.iter().filter(|r| r.output_exact).count();
                let b3_pass = score.results.iter().filter(|r| r.output_behavioral).count();
                let d_pass = score.results.iter().filter(|r| r.lint_clean).count();
                let e_pass = score.results.iter().filter(|r| r.deterministic).count();
                let f_pass = score
                    .results
                    .iter()
                    .filter(|r| r.metamorphic_consistent)
                    .count();
                let g_pass = score.results.iter().filter(|r| r.cross_shell_agree).count();
                let c_avg: f64 =
                    score.results.iter().map(|r| r.coverage_ratio).sum::<f64>() / n as f64;

                let pct_val = |pass: usize| -> f64 { pass as f64 / n as f64 * 100.0 };
                let pts = |pass: usize, max: f64| -> f64 { pass as f64 / n as f64 * max };

                println!();
                println!("{BOLD}V2 Component Breakdown:{RESET}");

                let print_dim = |label: &str, pass: usize, max_pts: f64| {
                    let p = pct_val(pass);
                    let pc = pct_color(p);
                    let bar = progress_bar(pass, n, 16);
                    println!(
                        "  {WHITE}{:<2} {:<14}{RESET} {pc}{:>4}/{}{RESET} ({pc}{:.1}%{RESET}) {bar} {WHITE}{:.1}/{}{RESET} pts",
                        label.split_whitespace().next().unwrap_or(""),
                        label.split_whitespace().skip(1).collect::<Vec<_>>().join(" "),
                        pass, n, p, pts(pass, max_pts), max_pts as u32
                    );
                };

                print_dim("A  Transpilation", a_pass, 30.0);
                print_dim("B1 Containment", b1_pass, 10.0);
                print_dim("B2 Exact match", b2_pass, 8.0);
                print_dim("B3 Behavioral", b3_pass, 7.0);

                // Coverage is special (average, not pass/fail)
                let c_pct = c_avg * 100.0;
                let cc = pct_color(c_pct);
                let c_bar = progress_bar((c_avg * n as f64) as usize, n, 16);
                println!(
                    "  {WHITE}C  Coverage       {RESET} {cc}avg {:.1}%{RESET}        {c_bar} {WHITE}{:.1}/15{RESET} pts",
                    c_pct, c_avg * 15.0
                );

                print_dim("D  Lint clean", d_pass, 10.0);
                print_dim("E  Deterministic", e_pass, 10.0);
                print_dim("F  Metamorphic", f_pass, 5.0);
                print_dim("G  Cross-shell", g_pass, 5.0);
            }

            // Failures section
            let failures: Vec<_> = score.results.iter().filter(|r| !r.transpiled).collect();
            if !failures.is_empty() {
                println!();
                println!("{BRIGHT_RED}Failed entries ({}):{RESET}", failures.len());
                for f in &failures {
                    let err = f.error.as_deref().unwrap_or("unknown error");
                    println!(
                        "  {CYAN}{}{RESET} — {DIM}{}{RESET}",
                        f.id,
                        truncate_str(err, 80)
                    );
                }
            }
        }
        CorpusOutputFormat::Json => {
            let json = serde_json::to_string_pretty(score)
                .map_err(|e| Error::Internal(format!("JSON serialization failed: {e}")))?;
            println!("{json}");
        }
    }
    Ok(())
}

pub(crate) fn corpus_write_convergence_log(
    runner: &crate::corpus::runner::CorpusRunner,
    score: &crate::corpus::runner::CorpusScore,
) -> Result<()> {
    use crate::corpus::runner::CorpusRunner;

    let log_path = PathBuf::from(".quality/convergence.log");
    let previous = CorpusRunner::load_convergence_log(&log_path).unwrap_or_default();
    let iteration = previous.len() as u32 + 1;
    let prev_rate = previous.last().map_or(0.0, |e| e.rate);
    let date = super::corpus_diff_commands::chrono_free_date();
    let entry = runner.convergence_entry(score, iteration, &date, prev_rate, "CLI corpus run");
    CorpusRunner::append_convergence_log(&entry, &log_path)
        .map_err(|e| Error::Internal(format!("Failed to write convergence log: {e}")))?;
    use crate::cli::color::*;
    println!();
    let dc = delta_color(entry.delta);
    let sc = pct_color(entry.score);
    println!(
        "{DIM}Convergence log:{RESET} iteration {}, {sc}{:.1}/100 {}{RESET}, delta {dc}",
        iteration, entry.score, entry.grade
    );
    // Per-format breakdown (spec §11.10.5)
    if entry.bash_total > 0 || entry.makefile_total > 0 || entry.dockerfile_total > 0 {
        let fmt_part = |name: &str, passed: usize, total: usize| -> String {
            if total > 0 {
                format!("{name} {passed}/{total}")
            } else {
                String::new()
            }
        };
        let parts: Vec<String> = [
            fmt_part("Bash", entry.bash_passed, entry.bash_total),
            fmt_part("Make", entry.makefile_passed, entry.makefile_total),
            fmt_part("Docker", entry.dockerfile_passed, entry.dockerfile_total),
        ]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect();
        if !parts.is_empty() {
            println!("{DIM}  Per-format:{RESET} {}", parts.join(", "));
        }
    }
    // Lint pass rate (spec §7.5)
    if entry.lint_passed > 0 {
        let lint_pct = entry.lint_rate * 100.0;
        let lc = pct_color(lint_pct);
        let gap = ((entry.rate - entry.lint_rate) * 100.0).abs();
        let gap_str = if gap > 0.1 {
            format!(" {DIM}(gap: {gap:.1}%){RESET}")
        } else {
            String::new()
        };
        println!(
            "{DIM}  Lint rate:{RESET} {lc}{lint_pct:.1}%{RESET} ({}/{}){}",
            entry.lint_passed, entry.total, gap_str
        );
    }
    // Regression detection (spec §5.3 — Jidoka)
    if let Some(prev) = previous.last() {
        let report = entry.detect_regressions(prev);
        if report.has_regressions() {
            println!();
            println!("{BRIGHT_RED}ANDON CORD: Corpus regression detected!{RESET}");
            for r in &report.regressions {
                println!("  {BRIGHT_RED}• {}{RESET}", r.message);
            }
            println!("{BRIGHT_RED}STOP THE LINE — investigate before proceeding.{RESET}");
        }
    }
    Ok(())
}

/// Format a bar chart for a percentage value.
pub(crate) fn stats_bar(pct: f64, width: usize) -> String {
    let filled = ((pct / 100.0) * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

/// Show per-format statistics and convergence trends (spec §11.10).
pub(crate) fn corpus_show_stats(format: &CorpusOutputFormat) -> Result<()> {
    use crate::corpus::registry::CorpusRegistry;
    use crate::corpus::runner::CorpusRunner;

    let config = Config::default();
    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(config);
    let score = runner.run(&registry);

    match format {
        CorpusOutputFormat::Human => {
            use crate::cli::color::*;

            println!("{BOLD}Corpus Statistics{RESET}");
            println!("{DIM}═══════════════════════════════════════════════════{RESET}");

            // Per-format table
            println!(
                "{DIM}{:<12} {:>7} {:>10} {:>5} {:>16}{RESET}",
                "Format", "Entries", "Pass Rate", "Grade", "Bar"
            );
            println!("{DIM}───────────────────────────────────────────────────{RESET}");

            for fs in &score.format_scores {
                let pct = fs.rate * 100.0;
                let rc = pct_color(pct);
                let gc = grade_color(&fs.grade.to_string());
                let bar = stats_bar(pct, 16);
                println!(
                    "{:<12} {:>7} {rc}{:>9.1}%{RESET} {gc}{:>5}{RESET} {rc}{bar}{RESET}",
                    fs.format, fs.total, pct, fs.grade,
                );
            }

            println!("{DIM}───────────────────────────────────────────────────{RESET}");
            let total_pct = score.rate * 100.0;
            let tc = pct_color(total_pct);
            let tg = grade_color(&score.grade.to_string());
            let tbar = stats_bar(total_pct, 16);
            println!(
                "{BOLD}{:<12}{RESET} {:>7} {tc}{:>9.1}%{RESET} {tg}{:>5}{RESET} {tc}{tbar}{RESET}",
                "Total", score.total, total_pct, score.grade,
            );

            // V2 score
            let sc = pct_color(score.score);
            println!();
            println!(
                "{BOLD}V2 Score:{RESET} {sc}{:.1}/100{RESET} ({tg}{}{RESET})",
                score.score, score.grade
            );

            // Convergence trend from log
            let log_path = PathBuf::from(".quality/convergence.log");
            if let Ok(entries) = CorpusRunner::load_convergence_log(&log_path) {
                if entries.len() >= 2 {
                    println!();
                    println!(
                        "{BOLD}Convergence Trend{RESET} (last {} runs):",
                        entries.len().min(10)
                    );
                    let recent: &[_] = if entries.len() > 10 {
                        &entries[entries.len() - 10..]
                    } else {
                        &entries
                    };
                    corpus_stats_sparkline(recent);
                }
            }

            // Failure summary
            let failures: Vec<_> = score.results.iter().filter(|r| !r.transpiled).collect();
            if !failures.is_empty() {
                println!();
                println!("{BOLD}Failing Entries{RESET} ({}):", failures.len());
                for r in failures.iter().take(10) {
                    println!("  {BRIGHT_RED}• {}{RESET}", r.id);
                }
                if failures.len() > 10 {
                    println!("  {DIM}... and {} more{RESET}", failures.len() - 10);
                }
            }
        }
        CorpusOutputFormat::Json => {
            #[derive(serde::Serialize)]
            struct StatsJson {
                total: usize,
                passed: usize,
                failed: usize,
                rate: f64,
                score: f64,
                grade: String,
                formats: Vec<FormatStats>,
            }
            #[derive(serde::Serialize)]
            struct FormatStats {
                format: String,
                total: usize,
                passed: usize,
                rate: f64,
                score: f64,
                grade: String,
            }
            let stats = StatsJson {
                total: score.total,
                passed: score.passed,
                failed: score.failed,
                rate: score.rate,
                score: score.score,
                grade: score.grade.to_string(),
                formats: score
                    .format_scores
                    .iter()
                    .map(|fs| FormatStats {
                        format: fs.format.to_string(),
                        total: fs.total,
                        passed: fs.passed,
                        rate: fs.rate,
                        score: fs.score,
                        grade: fs.grade.to_string(),
                    })
                    .collect(),
            };
            let json = serde_json::to_string_pretty(&stats)
                .map_err(|e| Error::Internal(format!("JSON: {e}")))?;
            println!("{json}");
        }
    }
    Ok(())
}

/// Print sparkline of score trend from convergence entries.
pub(crate) fn corpus_stats_sparkline(entries: &[crate::corpus::runner::ConvergenceEntry]) {
    use crate::cli::color::*;
    let bars = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    let scores: Vec<f64> = entries.iter().map(|e| e.score).collect();
    let min = scores.iter().copied().fold(f64::INFINITY, f64::min);
    let max = scores.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let range = (max - min).max(0.1);
    let sparkline: String = scores
        .iter()
        .map(|&s| {
            let idx = (((s - min) / range) * 7.0).round() as usize;
            bars[idx.min(7)]
        })
        .collect();
    let first = scores.first().copied().unwrap_or(0.0);
    let last = scores.last().copied().unwrap_or(0.0);
    let trend = if last > first {
        GREEN
    } else if last < first {
        BRIGHT_RED
    } else {
        DIM
    };
    println!(
        "  {DIM}Score:{RESET} {trend}{sparkline}{RESET}  ({:.1} → {:.1})",
        first, last
    );
}

/// Run metamorphic relation checks on a single corpus entry (spec §11.2).

pub(crate) const CORPUS_CACHE_PATH: &str = ".quality/last-corpus-run.json";

/// Save corpus run results to cache file for instant diagnosis.
pub(crate) fn corpus_save_last_run(score: &crate::corpus::runner::CorpusScore) {
    let path = std::path::Path::new(CORPUS_CACHE_PATH);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string(score) {
        let _ = std::fs::write(path, json);
    }
}

/// Load cached corpus results. Returns None if no cache exists.
pub(crate) fn corpus_load_last_run() -> Option<crate::corpus::runner::CorpusScore> {
    let data = std::fs::read_to_string(CORPUS_CACHE_PATH).ok()?;
    serde_json::from_str(&data).ok()
}
