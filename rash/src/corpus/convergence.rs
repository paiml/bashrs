//! Multi-corpus convergence analysis (spec §11.10.5).
//!
//! Provides per-format convergence tracking across iterations,
//! enabling detection of format-specific regressions and trend analysis.

use super::runner::ConvergenceEntry;

/// A format extractor: (format_name, function to extract (passed, total) from an entry).
type FormatExtractor = [(&'static str, fn(&ConvergenceEntry) -> (usize, usize)); 3];

/// Convergence trend for a single format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trend {
    /// Pass rate increasing across recent iterations.
    Improving,
    /// Pass rate stable (no change) across recent iterations.
    Stable,
    /// Pass rate decreasing across recent iterations.
    Regressing,
}

impl std::fmt::Display for Trend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Trend::Improving => write!(f, "Improving"),
            Trend::Stable => write!(f, "Stable"),
            Trend::Regressing => write!(f, "Regressing"),
        }
    }
}

/// Per-format convergence status with trend detection.
#[derive(Debug, Clone)]
pub struct FormatConvergenceStatus {
    /// Format name (e.g. "Bash", "Makefile", "Dockerfile").
    pub format: &'static str,
    /// Current pass rate (0.0-1.0).
    pub current_rate: f64,
    /// Detected trend across recent iterations.
    pub trend: Trend,
    /// Number of consecutive stable iterations.
    pub iterations_stable: usize,
}

/// Delta between two iterations across all formats.
#[derive(Debug, Clone)]
pub struct IterationDiff {
    /// Source iteration number.
    pub from_iter: u32,
    /// Target iteration number.
    pub to_iter: u32,
    /// Bash pass-rate delta (percentage points).
    pub bash_delta: f64,
    /// Makefile pass-rate delta (percentage points).
    pub makefile_delta: f64,
    /// Dockerfile pass-rate delta (percentage points).
    pub dockerfile_delta: f64,
    /// Total pass-rate delta (percentage points).
    pub total_delta: f64,
    /// Score delta (0-100 scale).
    pub score_delta: f64,
    /// Per-format passed/total snapshots for display.
    pub from_bash: (usize, usize),
    pub to_bash: (usize, usize),
    pub from_makefile: (usize, usize),
    pub to_makefile: (usize, usize),
    pub from_dockerfile: (usize, usize),
    pub to_dockerfile: (usize, usize),
    pub from_total: (usize, usize),
    pub to_total: (usize, usize),
    pub from_score: f64,
    pub to_score: f64,
}

/// Compute pass rate from (passed, total), returning 0.0 if total is 0.
fn pass_rate(passed: usize, total: usize) -> f64 {
    if total == 0 {
        return 0.0;
    }
    passed as f64 / total as f64
}

/// Format a full iteration x format convergence table.
///
/// Renders all iterations with per-format pass counts and overall score/grade.
pub fn format_convergence_table(entries: &[ConvergenceEntry]) -> String {
    use std::fmt::Write;
    let mut out = String::new();

    let _ = writeln!(out, "Multi-Corpus Convergence Table (\u{00a7}11.10.5)");
    let divider = "\u{2500}".repeat(78);
    let _ = writeln!(out, "{divider}");
    let _ = writeln!(
        out,
        "{:<6}{:<12}{:<14}{:<16}{:<18}{:<9}Grade",
        "Iter", "Date", "Bash (500)", "Makefile (200)", "Dockerfile (200)", "Score"
    );
    let _ = writeln!(out, "{divider}");

    for e in entries {
        // Skip entries without per-format data (legacy iterations)
        if e.bash_total == 0 && e.makefile_total == 0 && e.dockerfile_total == 0 {
            let _ = writeln!(
                out,
                "{:<6}{:<12}{:<14}{:<16}{:<18}{:<9.1}{}",
                format!("#{}", e.iteration),
                &e.date,
                format!("{}/{}", e.passed, e.total),
                "-",
                "-",
                e.score,
                &e.grade,
            );
        } else {
            let _ = writeln!(
                out,
                "{:<6}{:<12}{:<14}{:<16}{:<18}{:<9.1}{}",
                format!("#{}", e.iteration),
                &e.date,
                format!("{}/{}", e.bash_passed, e.bash_total),
                format!("{}/{}", e.makefile_passed, e.makefile_total),
                format!("{}/{}", e.dockerfile_passed, e.dockerfile_total),
                e.score,
                &e.grade,
            );
        }
    }

    let _ = writeln!(out, "{divider}");
    let _ = writeln!(out, "{} iterations total", entries.len());
    out
}

/// Compute per-format convergence status with trend detection.
///
/// Examines the last few iterations to determine if each format is
/// improving, stable, or regressing.
pub fn convergence_status(entries: &[ConvergenceEntry]) -> Vec<FormatConvergenceStatus> {
    let formats: FormatExtractor = [
        ("Bash", |e| (e.bash_passed, e.bash_total)),
        ("Makefile", |e| (e.makefile_passed, e.makefile_total)),
        ("Dockerfile", |e| (e.dockerfile_passed, e.dockerfile_total)),
    ];

    formats
        .iter()
        .map(|(name, extract)| {
            let rates: Vec<f64> = entries
                .iter()
                .filter(|e| {
                    let (_, total) = extract(e);
                    total > 0
                })
                .map(|e| {
                    let (passed, total) = extract(e);
                    pass_rate(passed, total)
                })
                .collect();

            let current_rate = rates.last().copied().unwrap_or(0.0);
            let (trend, iterations_stable) = detect_trend(&rates);

            FormatConvergenceStatus {
                format: name,
                current_rate,
                trend,
                iterations_stable,
            }
        })
        .collect()
}

/// Detect trend from a series of rates.
///
/// Returns (Trend, consecutive_stable_count).
fn detect_trend(rates: &[f64]) -> (Trend, usize) {
    if rates.len() < 2 {
        return (Trend::Stable, rates.len());
    }

    let epsilon = 1e-9;
    let mut stable_count = 0usize;

    // Count consecutive stable iterations from the end
    for pair in rates.windows(2).rev() {
        let delta = pair[1] - pair[0];
        if delta.abs() < epsilon {
            stable_count += 1;
        } else {
            break;
        }
    }

    if stable_count > 0 {
        return (Trend::Stable, stable_count + 1); // +1 for the anchor iteration
    }

    // Check last delta for trend direction
    let last_delta = rates[rates.len() - 1] - rates[rates.len() - 2];
    if last_delta > epsilon {
        (Trend::Improving, 1)
    } else if last_delta < -epsilon {
        (Trend::Regressing, 1)
    } else {
        (Trend::Stable, 1)
    }
}

/// Compare two convergence entries, computing per-format deltas.
pub fn compare_iterations(from: &ConvergenceEntry, to: &ConvergenceEntry) -> IterationDiff {
    let bash_from_rate = pass_rate(from.bash_passed, from.bash_total) * 100.0;
    let bash_to_rate = pass_rate(to.bash_passed, to.bash_total) * 100.0;
    let make_from_rate = pass_rate(from.makefile_passed, from.makefile_total) * 100.0;
    let make_to_rate = pass_rate(to.makefile_passed, to.makefile_total) * 100.0;
    let dock_from_rate = pass_rate(from.dockerfile_passed, from.dockerfile_total) * 100.0;
    let dock_to_rate = pass_rate(to.dockerfile_passed, to.dockerfile_total) * 100.0;
    let total_from_rate = pass_rate(from.passed, from.total) * 100.0;
    let total_to_rate = pass_rate(to.passed, to.total) * 100.0;

    IterationDiff {
        from_iter: from.iteration,
        to_iter: to.iteration,
        bash_delta: bash_to_rate - bash_from_rate,
        makefile_delta: make_to_rate - make_from_rate,
        dockerfile_delta: dock_to_rate - dock_from_rate,
        total_delta: total_to_rate - total_from_rate,
        score_delta: to.score - from.score,
        from_bash: (from.bash_passed, from.bash_total),
        to_bash: (to.bash_passed, to.bash_total),
        from_makefile: (from.makefile_passed, from.makefile_total),
        to_makefile: (to.makefile_passed, to.makefile_total),
        from_dockerfile: (from.dockerfile_passed, from.dockerfile_total),
        to_dockerfile: (to.dockerfile_passed, to.dockerfile_total),
        from_total: (from.passed, from.total),
        to_total: (to.passed, to.total),
        from_score: from.score,
        to_score: to.score,
    }
}

/// Format a delta value with arrow indicator.
pub fn format_delta(delta: f64) -> String {
    if delta.abs() < 1e-9 {
        "\u{2192} 0.0%".to_string()
    } else if delta > 0.0 {
        format!("\u{2191} +{delta:.1}%")
    } else {
        format!("\u{2193} {delta:.1}%")
    }
}

/// Format a score delta with arrow indicator.
pub fn format_score_delta(delta: f64) -> String {
    if delta.abs() < 1e-9 {
        "\u{2192} 0.0".to_string()
    } else if delta > 0.0 {
        format!("\u{2191} +{delta:.1}")
    } else {
        format!("\u{2193} {delta:.1}")
    }
}

/// Format the convergence diff as a human-readable table.
pub fn format_iteration_diff(diff: &IterationDiff) -> String {
    use std::fmt::Write;
    let mut out = String::new();

    let _ = writeln!(
        out,
        "Convergence Diff: Iteration #{} \u{2192} #{}",
        diff.from_iter, diff.to_iter
    );
    let divider = "\u{2500}".repeat(56);
    let _ = writeln!(out, "{divider}");
    let _ = writeln!(out, "{:<13}{:<11}{:<11}Delta", "Format", "Before", "After");
    let _ = writeln!(out, "{divider}");

    let _ = writeln!(
        out,
        "{:<13}{:<11}{:<11}{}",
        "Bash",
        format!("{}/{}", diff.from_bash.0, diff.from_bash.1),
        format!("{}/{}", diff.to_bash.0, diff.to_bash.1),
        format_delta(diff.bash_delta),
    );
    let _ = writeln!(
        out,
        "{:<13}{:<11}{:<11}{}",
        "Makefile",
        format!("{}/{}", diff.from_makefile.0, diff.from_makefile.1),
        format!("{}/{}", diff.to_makefile.0, diff.to_makefile.1),
        format_delta(diff.makefile_delta),
    );
    let _ = writeln!(
        out,
        "{:<13}{:<11}{:<11}{}",
        "Dockerfile",
        format!("{}/{}", diff.from_dockerfile.0, diff.from_dockerfile.1),
        format!("{}/{}", diff.to_dockerfile.0, diff.to_dockerfile.1),
        format_delta(diff.dockerfile_delta),
    );
    let _ = writeln!(
        out,
        "{:<13}{:<11}{:<11}{}",
        "Total",
        format!("{}/{}", diff.from_total.0, diff.from_total.1),
        format!("{}/{}", diff.to_total.0, diff.to_total.1),
        format_delta(diff.total_delta),
    );
    let _ = writeln!(
        out,
        "{:<13}{:<11.1}{:<11.1}{}",
        "Score",
        diff.from_score,
        diff.to_score,
        format_score_delta(diff.score_delta),
    );
    let _ = writeln!(out, "{divider}");
    out
}

/// Format the convergence status as a human-readable table.
pub fn format_convergence_status(statuses: &[FormatConvergenceStatus]) -> String {
    use std::fmt::Write;
    let mut out = String::new();

    let _ = writeln!(out, "Per-Format Convergence Status");
    let divider = "\u{2500}".repeat(56);
    let _ = writeln!(out, "{divider}");
    let _ = writeln!(
        out,
        "{:<13}{:<10}{:<14}Stable Iters",
        "Format", "Rate", "Trend"
    );
    let _ = writeln!(out, "{divider}");

    for s in statuses {
        let trend_arrow = match s.trend {
            Trend::Improving => "\u{2191}",
            Trend::Stable => "\u{2192}",
            Trend::Regressing => "\u{2193}",
        };
        let rate = format!("{:.1}%", s.current_rate * 100.0);
        let trend = format!("{trend_arrow} {}", s.trend);
        let iters = format!("{} iterations", s.iterations_stable);
        let _ = writeln!(out, "{:<13}{rate:<10}{trend:<14}{iters}", s.format,);
    }

    let _ = writeln!(out, "{divider}");

    // Overall convergence assessment
    let all_stable = statuses
        .iter()
        .all(|s| s.trend == Trend::Stable && s.iterations_stable >= 2);
    if all_stable {
        let min_stable = statuses
            .iter()
            .map(|s| s.iterations_stable)
            .min()
            .unwrap_or(0);
        let _ = writeln!(
            out,
            "\nOverall: CONVERGED (all formats stable for \u{2265}{min_stable} iterations)"
        );
    } else {
        let regressing: Vec<&str> = statuses
            .iter()
            .filter(|s| s.trend == Trend::Regressing)
            .map(|s| s.format)
            .collect();
        if regressing.is_empty() {
            let _ = writeln!(out, "\nOverall: IMPROVING (not yet converged)");
        } else {
            let _ = writeln!(out, "\nOverall: REGRESSING ({})", regressing.join(", "));
        }
    }

    out
}

// FIXME(PMAT-238): #[cfg(test)]
// FIXME(PMAT-238): #[path = "convergence_tests_make_entry.rs"]
// FIXME(PMAT-238): mod tests_ext;
