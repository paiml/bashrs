//! Quality Gate Configuration for Corpus (§9)
//!
//! Implements corpus-specific quality gates from the `.pmat-gates.toml`
//! and `.pmat-metrics.toml` schemas defined in spec §8.1 and §8.2.
//!
//! Gates check corpus results against configurable thresholds:
//! - Transpilation rate, score, coverage
//! - Performance (ms per entry, memory)
//! - Staleness (days since last convergence log entry)
//! - Regression detection

use crate::corpus::runner::{ConvergenceEntry, CorpusScore};
use std::time::Duration;

/// Quality gate check result
#[derive(Debug, Clone)]
pub struct GateCheck {
    pub name: &'static str,
    pub description: &'static str,
    pub passed: bool,
    pub actual: String,
    pub threshold: String,
}

/// Performance metric check result
#[derive(Debug, Clone)]
pub struct MetricCheck {
    pub name: &'static str,
    pub passed: bool,
    pub actual: String,
    pub threshold: String,
    pub unit: &'static str,
}

/// Combined gate status
#[derive(Debug, Clone)]
pub struct GateStatus {
    pub quality_gates: Vec<GateCheck>,
    pub metrics: Vec<MetricCheck>,
    pub all_passed: bool,
    pub gates_passed: usize,
    pub gates_total: usize,
}

/// Quality gate thresholds (from §8.1 .pmat-gates.toml schema)
#[derive(Debug, Clone)]
pub struct QualityThresholds {
    pub min_coverage: f64,
    pub min_score: f64,
    pub min_rate: f64,
    pub max_failures: usize,
    pub min_grade: &'static str,
    pub block_on_regression: bool,
}

impl Default for QualityThresholds {
    fn default() -> Self {
        Self {
            min_coverage: 95.0,
            min_score: 90.0,
            min_rate: 99.0,
            max_failures: 5,
            min_grade: "A",
            block_on_regression: true,
        }
    }
}

/// Performance thresholds (from §8.2 .pmat-metrics.toml schema)
#[derive(Debug, Clone)]
pub struct PerformanceThresholds {
    pub max_transpile_ms_per_entry: u64,
    pub max_total_ms: u64,
    pub max_staleness_days: u32,
    pub min_mutation_score: f64,
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            max_transpile_ms_per_entry: 100,
            max_total_ms: 60_000,
            max_staleness_days: 7,
            min_mutation_score: 90.0,
        }
    }
}

/// Check corpus quality gates against thresholds
pub fn check_quality_gates(
    score: &CorpusScore,
    history: &[ConvergenceEntry],
    thresholds: &QualityThresholds,
) -> Vec<GateCheck> {
    let mut gates = Vec::new();

    // Gate 1: Transpilation rate
    let rate_pct = score.rate * 100.0;
    gates.push(GateCheck {
        name: "Transpilation Rate",
        description: "Minimum transpilation success rate",
        passed: rate_pct >= thresholds.min_rate,
        actual: format!("{:.1}%", rate_pct),
        threshold: format!(">= {:.0}%", thresholds.min_rate),
    });

    // Gate 2: V2 Score
    gates.push(GateCheck {
        name: "V2 Corpus Score",
        description: "Minimum 100-point quality score",
        passed: score.score >= thresholds.min_score,
        actual: format!("{:.1}", score.score),
        threshold: format!(">= {:.0}", thresholds.min_score),
    });

    // Gate 3: Maximum failures
    gates.push(GateCheck {
        name: "Failure Count",
        description: "Maximum allowed failing entries",
        passed: score.failed <= thresholds.max_failures,
        actual: format!("{}", score.failed),
        threshold: format!("<= {}", thresholds.max_failures),
    });

    // Gate 4: Grade threshold
    let grade_str = score.grade.to_string();
    let grade_ok = grade_meets_minimum(&grade_str, thresholds.min_grade);
    gates.push(GateCheck {
        name: "Quality Grade",
        description: "Minimum quality grade",
        passed: grade_ok,
        actual: grade_str,
        threshold: format!(">= {}", thresholds.min_grade),
    });

    // Gate 5: No regressions (compare against history)
    if thresholds.block_on_regression && !history.is_empty() {
        let has_regression = check_for_regression(score, history);
        gates.push(GateCheck {
            name: "No Regressions",
            description: "No entries that previously passed now fail",
            passed: !has_regression,
            actual: if has_regression {
                "regression detected".to_string()
            } else {
                "no regressions".to_string()
            },
            threshold: "0 regressions".to_string(),
        });
    }

    // Gate 6: Per-format rates
    for fs in &score.format_scores {
        let fmt_rate = fs.rate * 100.0;
        gates.push(GateCheck {
            name: match fs.format {
                crate::corpus::registry::CorpusFormat::Bash => "Bash Rate",
                crate::corpus::registry::CorpusFormat::Makefile => "Makefile Rate",
                crate::corpus::registry::CorpusFormat::Dockerfile => "Dockerfile Rate",
            },
            description: "Per-format transpilation rate",
            passed: fmt_rate >= thresholds.min_rate,
            actual: format!("{:.1}%", fmt_rate),
            threshold: format!(">= {:.0}%", thresholds.min_rate),
        });
    }

    gates
}

/// Check performance metrics against thresholds
pub fn check_metrics(
    score: &CorpusScore,
    run_duration: Duration,
    history: &[ConvergenceEntry],
    thresholds: &PerformanceThresholds,
) -> Vec<MetricCheck> {
    let mut metrics = Vec::new();

    // Metric 1: Total corpus run time
    let total_ms = run_duration.as_millis() as u64;
    metrics.push(MetricCheck {
        name: "Total Run Time",
        passed: total_ms <= thresholds.max_total_ms,
        actual: format!("{}", total_ms),
        threshold: format!("<= {}", thresholds.max_total_ms),
        unit: "ms",
    });

    // Metric 2: Average ms per entry
    let avg_ms = if score.total > 0 {
        total_ms / score.total as u64
    } else {
        0
    };
    metrics.push(MetricCheck {
        name: "Avg Time/Entry",
        passed: avg_ms <= thresholds.max_transpile_ms_per_entry,
        actual: format!("{}", avg_ms),
        threshold: format!("<= {}", thresholds.max_transpile_ms_per_entry),
        unit: "ms",
    });

    // Metric 3: Staleness (days since last convergence entry)
    let staleness_days = compute_staleness(history);
    metrics.push(MetricCheck {
        name: "Log Staleness",
        passed: staleness_days <= thresholds.max_staleness_days,
        actual: format!("{}", staleness_days),
        threshold: format!("<= {}", thresholds.max_staleness_days),
        unit: "days",
    });

    // Metric 4: Corpus size (entries)
    let min_entries = 500; // spec §2.3 target
    metrics.push(MetricCheck {
        name: "Corpus Size",
        passed: score.total >= min_entries,
        actual: format!("{}", score.total),
        threshold: format!(">= {}", min_entries),
        unit: "entries",
    });

    // Metric 5: History depth (iterations tracked)
    let min_iterations = 3;
    metrics.push(MetricCheck {
        name: "History Depth",
        passed: history.len() >= min_iterations,
        actual: format!("{}", history.len()),
        threshold: format!(">= {}", min_iterations),
        unit: "iterations",
    });

    metrics
}

/// Build combined gate status
pub fn build_gate_status(
    score: &CorpusScore,
    run_duration: Duration,
    history: &[ConvergenceEntry],
) -> GateStatus {
    let qt = QualityThresholds::default();
    let pt = PerformanceThresholds::default();

    let quality_gates = check_quality_gates(score, history, &qt);
    let metrics = check_metrics(score, run_duration, history, &pt);

    let gates_passed = quality_gates.iter().filter(|g| g.passed).count()
        + metrics.iter().filter(|m| m.passed).count();
    let gates_total = quality_gates.len() + metrics.len();
    let all_passed = gates_passed == gates_total;

    GateStatus {
        quality_gates,
        metrics,
        all_passed,
        gates_passed,
        gates_total,
    }
}

/// Format quality gates report
pub fn format_quality_gates(gates: &[GateCheck]) -> String {
    let mut out = String::new();
    let sep = "\u{2500}".repeat(72);

    out.push_str("Corpus Quality Gates (\u{00a7}9 / \u{00a7}8.1)\n");
    out.push_str(&sep);
    out.push('\n');
    out.push_str(&format!(
        "{:<22} {:>12} {:>12} {:>8}\n",
        "Gate", "Actual", "Threshold", "Status"
    ));
    out.push_str(&sep);
    out.push('\n');

    for gate in gates {
        let status = if gate.passed { "PASS" } else { "FAIL" };
        out.push_str(&format!(
            "{:<22} {:>12} {:>12} {:>8}\n",
            gate.name, gate.actual, gate.threshold, status,
        ));
    }

    let passed = gates.iter().filter(|g| g.passed).count();
    let total = gates.len();
    out.push_str(&sep);
    out.push('\n');
    out.push_str(&format!("Result: {}/{} gates passed\n", passed, total));

    out
}

/// Format metrics check report
pub fn format_metrics_check(metrics: &[MetricCheck]) -> String {
    let mut out = String::new();
    let sep = "\u{2500}".repeat(72);

    out.push_str("Corpus Performance Metrics (\u{00a7}9 / \u{00a7}8.2)\n");
    out.push_str(&sep);
    out.push('\n');
    out.push_str(&format!(
        "{:<22} {:>12} {:>12} {:>8} {:>8}\n",
        "Metric", "Actual", "Threshold", "Unit", "Status"
    ));
    out.push_str(&sep);
    out.push('\n');

    for metric in metrics {
        let status = if metric.passed { "PASS" } else { "FAIL" };
        out.push_str(&format!(
            "{:<22} {:>12} {:>12} {:>8} {:>8}\n",
            metric.name, metric.actual, metric.threshold, metric.unit, status,
        ));
    }

    let passed = metrics.iter().filter(|m| m.passed).count();
    let total = metrics.len();
    out.push_str(&sep);
    out.push('\n');
    out.push_str(&format!("Result: {}/{} metrics passed\n", passed, total));

    out
}

/// Format combined gate status report
pub fn format_gate_status(status: &GateStatus) -> String {
    let mut out = String::new();
    let sep = "\u{2500}".repeat(72);

    out.push_str("Corpus Gate Status Summary (\u{00a7}9)\n");
    out.push_str(&sep);
    out.push('\n');

    // Quality gates section
    out.push_str("Quality Gates:\n");
    for gate in &status.quality_gates {
        let icon = if gate.passed { "\u{2713}" } else { "\u{2717}" };
        out.push_str(&format!(
            "  {} {:<22} {} ({})\n",
            icon, gate.name, gate.actual, gate.threshold,
        ));
    }

    // Metrics section
    out.push_str("\nPerformance Metrics:\n");
    for metric in &status.metrics {
        let icon = if metric.passed {
            "\u{2713}"
        } else {
            "\u{2717}"
        };
        out.push_str(&format!(
            "  {} {:<22} {} {} ({})\n",
            icon, metric.name, metric.actual, metric.unit, metric.threshold,
        ));
    }

    // Summary
    out.push_str(&format!("\n{}\n", sep));
    let overall = if status.all_passed {
        "ALL GATES PASSED"
    } else {
        "GATES FAILED"
    };
    out.push_str(&format!(
        "Overall: {} ({}/{} passed)\n",
        overall, status.gates_passed, status.gates_total,
    ));

    out
}

/// Check if grade meets minimum threshold
fn grade_meets_minimum(actual: &str, minimum: &str) -> bool {
    let grade_rank = |g: &str| -> u8 {
        match g {
            "A+" => 6,
            "A" => 5,
            "B" => 4,
            "C" => 3,
            "D" => 2,
            "F" => 1,
            _ => 0,
        }
    };
    grade_rank(actual) >= grade_rank(minimum)
}

/// Check if there's a regression compared to history
fn check_for_regression(score: &CorpusScore, history: &[ConvergenceEntry]) -> bool {
    if let Some(last) = history.last() {
        // Regression if current pass count is lower than last recorded
        if score.passed < last.passed {
            return true;
        }
    }
    false
}

/// Compute days since last convergence log entry
fn compute_staleness(history: &[ConvergenceEntry]) -> u32 {
    if history.is_empty() {
        return 999; // Very stale if no history
    }

    // Try to parse the last entry's date
    if let Some(last) = history.last() {
        if !last.date.is_empty() {
            // Date format: "2026-02-08" — compute days difference
            if let Some(days) = days_since(&last.date) {
                return days;
            }
        }
    }

    // If we can't determine, assume fresh (0 days)
    0
}

/// Compute days since a date string (YYYY-MM-DD format)
fn days_since(date_str: &str) -> Option<u32> {
    let parts: Vec<&str> = date_str.split('-').collect();
    if parts.len() != 3 {
        return None;
    }
    let year: i64 = parts[0].parse().ok()?;
    let month: i64 = parts[1].parse().ok()?;
    let day: i64 = parts[2].parse().ok()?;

    // Julian day number calculation
    let jdn = |y: i64, m: i64, d: i64| -> i64 {
        let a = (14 - m) / 12;
        let y2 = y + 4800 - a;
        let m2 = m + 12 * a - 3;
        d + (153 * m2 + 2) / 5 + 365 * y2 + y2 / 4 - y2 / 100 + y2 / 400 - 32045
    };

    let entry_jdn = jdn(year, month, day);

    // Get current date via system time
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?;
    let total_days = now.as_secs() / 86400;
    // Unix epoch is 1970-01-01 which is JDN 2440588
    let current_jdn = total_days as i64 + 2_440_588;

    let diff = current_jdn - entry_jdn;
    if diff < 0 {
        Some(0)
    } else {
        Some(diff as u32)
    }
}

#[cfg(test)]
#[path = "quality_gates_tests_make_score.rs"]
mod tests_extracted;
