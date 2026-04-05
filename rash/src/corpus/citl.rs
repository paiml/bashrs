//! # Compiler-in-the-Loop (CITL) Integration (§7)
//!
//! Implements the self-improving corpus pipeline (§7.3), regression
//! detection / Jidoka Andon cord (§5.3), and convergence criteria
//! verification (§5.2).
//!
//! Key insight from §7: the bashrs linter IS the compiler. CITL is
//! the combination of transpilation + linting + testing on every entry.

use crate::corpus::registry::{CorpusFormat, CorpusRegistry};
use crate::corpus::runner::{ConvergenceEntry, CorpusResult, CorpusScore};

/// A lint violation found in transpiled output that suggests a new corpus entry (§7.3)
#[derive(Debug, Clone)]
pub struct LintPipelineEntry {
    /// Source entry that produced the violation
    pub source_id: String,
    /// Lint rule that was violated
    pub rule: String,
    /// Violation description
    pub message: String,
    /// Suggested corpus entry ID
    pub suggested_id: String,
    /// Suggested corpus entry name
    pub suggested_name: String,
    /// Format of the source entry
    pub format: CorpusFormat,
}

/// Result of regression check (§5.3 Jidoka)
#[derive(Debug, Clone)]
pub struct RegressionReport {
    /// Entries that regressed (previously passed, now fail)
    pub regressions: Vec<RegressionEntry>,
    /// Entries that improved (previously failed, now pass)
    pub improvements: Vec<String>,
    /// Total entries checked
    pub total: usize,
    /// Whether Andon cord should be pulled
    pub andon_triggered: bool,
}

/// A single regression (§5.3)
#[derive(Debug, Clone)]
pub struct RegressionEntry {
    pub id: String,
    pub format: CorpusFormat,
    pub error: String,
}

/// Convergence criteria status (§5.2)
#[derive(Debug, Clone)]
pub struct ConvergenceCriteria {
    /// Rate threshold: >= 99% for 3 consecutive iterations
    pub rate_met: bool,
    pub rate_values: Vec<f64>,
    /// Stability: delta < 0.5% for 3 consecutive iterations
    pub stability_met: bool,
    pub delta_values: Vec<f64>,
    /// Corpus growth: size >= initial target (500 bash + 200 makefile + 200 dockerfile)
    pub growth_met: bool,
    pub corpus_size: usize,
    pub target_size: usize,
    /// No regressions: no entry that previously passed has started failing
    pub no_regressions: bool,
    /// Overall: all 4 criteria met
    pub converged: bool,
}

/// Scan transpiled output for lint violations and suggest new corpus entries (§7.3)
pub fn lint_pipeline(registry: &CorpusRegistry, score: &CorpusScore) -> Vec<LintPipelineEntry> {
    let mut suggestions = Vec::new();
    let max_id = find_max_corpus_id(registry);
    let mut next_id = max_id + 1;

    for result in &score.results {
        if !result.transpiled || result.lint_clean {
            continue;
        }
        // This entry transpiled but failed lint — generate suggestion
        let entry = registry.entries.iter().find(|e| e.id == result.id);
        let format = entry.map_or(CorpusFormat::Bash, |e| e.format);
        let prefix = format_prefix(format);

        let error_msg = result
            .error
            .as_deref()
            .unwrap_or("lint violation in transpiled output");

        let rule = extract_lint_rule(error_msg);
        let name = generate_entry_name(&rule, &result.id);

        suggestions.push(LintPipelineEntry {
            source_id: result.id.clone(),
            rule: rule.clone(),
            message: error_msg.to_string(),
            suggested_id: format!("{prefix}-{next_id:03}"),
            suggested_name: name,
            format,
        });
        next_id += 1;
    }

    suggestions
}

/// Find the highest numeric ID across all corpus entries
fn find_max_corpus_id(registry: &CorpusRegistry) -> usize {
    registry
        .entries
        .iter()
        .filter_map(|e| {
            e.id.split('-')
                .next_back()
                .and_then(|n| n.parse::<usize>().ok())
        })
        .max()
        .unwrap_or(0)
}

fn format_prefix(format: CorpusFormat) -> &'static str {
    match format {
        CorpusFormat::Bash => "B",
        CorpusFormat::Makefile => "M",
        CorpusFormat::Dockerfile => "D",
    }
}

/// Extract a lint rule code from an error message
fn extract_lint_rule(message: &str) -> String {
    // Look for patterns like SEC001, DET003, IDEM002, MAKE005, DOCKER003
    let prefixes = ["SEC", "DET", "IDEM", "MAKE", "DOCKER"];
    for prefix in &prefixes {
        if let Some(pos) = message.find(prefix) {
            let rest = &message[pos..];
            let end = rest
                .find(|c: char| !c.is_alphanumeric())
                .unwrap_or(rest.len());
            return rest[..end].to_string();
        }
    }
    "LINT-UNKNOWN".to_string()
}

/// Generate a descriptive entry name from a lint rule
fn generate_entry_name(rule: &str, source_id: &str) -> String {
    let description = match rule {
        r if r.starts_with("SEC") => "security-violation",
        r if r.starts_with("DET") => "determinism-violation",
        r if r.starts_with("IDEM") => "idempotency-violation",
        r if r.starts_with("MAKE") => "makefile-lint-violation",
        r if r.starts_with("DOCKER") => "dockerfile-lint-violation",
        _ => "lint-violation",
    };
    format!("{description}-from-{source_id}")
}

/// Check for regressions by comparing current results against the last convergence entry (§5.3)
pub fn check_regressions(score: &CorpusScore, history: &[ConvergenceEntry]) -> RegressionReport {
    let total = score.total;

    // If no history, no regressions possible
    if history.is_empty() {
        return RegressionReport {
            regressions: Vec::new(),
            improvements: Vec::new(),
            total,
            andon_triggered: false,
        };
    }

    let last = &history[history.len() - 1];

    // Compare current pass count against last known
    let current_failed: Vec<&CorpusResult> =
        score.results.iter().filter(|r| !r.transpiled).collect();

    let regressions: Vec<RegressionEntry> = current_failed
        .iter()
        .map(|r| RegressionEntry {
            id: r.id.clone(),
            format: guess_format(&r.id),
            error: r.error.clone().unwrap_or_else(|| "unknown".to_string()),
        })
        .collect();

    // Check if we went backwards
    let andon_triggered = score.passed < last.passed && !regressions.is_empty();

    // Improvements: entries beyond last total that pass (corpus growth)
    let improvements = if score.total > last.total {
        vec![format!(
            "{} new entries added ({} → {})",
            score.total - last.total,
            last.total,
            score.total
        )]
    } else {
        Vec::new()
    };

    RegressionReport {
        regressions,
        improvements,
        total,
        andon_triggered,
    }
}

fn guess_format(id: &str) -> CorpusFormat {
    if id.starts_with("M-") {
        CorpusFormat::Makefile
    } else if id.starts_with("D-") {
        CorpusFormat::Dockerfile
    } else {
        CorpusFormat::Bash
    }
}

/// Verify convergence criteria (§5.2)
pub fn check_convergence(score: &CorpusScore, history: &[ConvergenceEntry]) -> ConvergenceCriteria {
    let target_size = 900; // 500 bash + 200 makefile + 200 dockerfile

    // Criterion 1: Rate >= 99% for last 3 iterations
    let rate_values: Vec<f64> = history
        .iter()
        .rev()
        .take(3)
        .map(|e| e.rate)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    let rate_met = rate_values.len() >= 3 && rate_values.iter().all(|r| *r >= 0.99);

    // Criterion 2: Stability (delta < 0.5% for 3 consecutive iterations)
    let delta_values: Vec<f64> = history
        .iter()
        .rev()
        .take(3)
        .map(|e| e.delta)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    let stability_met = delta_values.len() >= 3 && delta_values.iter().all(|d| d.abs() < 0.005);

    // Criterion 3: Corpus size >= target
    let growth_met = score.total >= target_size;

    // Criterion 4: No regressions (current failed == 0 or stable)
    let no_regressions = score.failed == 0;

    let converged = rate_met && stability_met && growth_met && no_regressions;

    ConvergenceCriteria {
        rate_met,
        rate_values,
        stability_met,
        delta_values,
        growth_met,
        corpus_size: score.total,
        target_size,
        no_regressions,
        converged,
    }
}

/// Format lint pipeline suggestions as a table
pub fn format_lint_pipeline(suggestions: &[LintPipelineEntry]) -> String {
    let mut out = String::new();
    let line = "\u{2500}".repeat(72);

    if suggestions.is_empty() {
        out.push_str(&format!("{}\n", line));
        out.push_str("No lint violations in transpiled output. CITL loop clean.\n");
        out.push_str(&format!("{}\n", line));
        return out;
    }

    out.push_str(&format!(
        "{}\n{:<10} {:<12} {:<14} {}\n{}\n",
        line, "Source", "Rule", "Suggested ID", "Name", line,
    ));

    for s in suggestions {
        out.push_str(&format!(
            "{:<10} {:<12} {:<14} {}\n",
            s.source_id, s.rule, s.suggested_id, s.suggested_name,
        ));
    }

    out.push_str(&format!("{}\n", line));
    out.push_str(&format!(
        "{} suggestion(s) from CITL lint pipeline\n",
        suggestions.len()
    ));

    out
}

/// Format regression report
pub fn format_regression_report(report: &RegressionReport) -> String {
    let mut out = String::new();
    let line = "\u{2500}".repeat(56);

    out.push_str(&format!("{}\n", line));

    if report.regressions.is_empty() {
        out.push_str("No regressions detected. All entries stable.\n");
    } else {
        out.push_str(&format!(
            "REGRESSIONS DETECTED: {} entries\n{}\n",
            report.regressions.len(),
            line,
        ));
        out.push_str(&format!("{:<10} {:<12} {}\n", "ID", "Format", "Error"));
        out.push_str(&format!("{}\n", line));
        for r in &report.regressions {
            let error = if r.error.len() > 40 {
                format!("{}...", &r.error[..37])
            } else {
                r.error.clone()
            };
            out.push_str(&format!("{:<10} {:<12} {}\n", r.id, r.format, error));
        }
    }

    if !report.improvements.is_empty() {
        out.push_str("\nImprovements:\n");
        for imp in &report.improvements {
            out.push_str(&format!("  + {}\n", imp));
        }
    }

    out.push_str(&format!("{}\n", line));

    if report.andon_triggered {
        out.push_str("ANDON CORD: STOP THE LINE - Regressions detected!\n");
    } else {
        out.push_str("Status: OK - No Andon cord trigger\n");
    }

    out
}

/// Format convergence criteria check
pub fn format_convergence_criteria(criteria: &ConvergenceCriteria) -> String {
    let mut out = String::new();
    let line = "\u{2500}".repeat(60);

    out.push_str(&format!(
        "{}\n{:<40} {:<10} {}\n{}\n",
        line, "Criterion", "Status", "Detail", line,
    ));

    // 1. Rate threshold
    let rate_detail = if criteria.rate_values.is_empty() {
        "no history".to_string()
    } else {
        criteria
            .rate_values
            .iter()
            .map(|r| format!("{:.1}%", r * 100.0))
            .collect::<Vec<_>>()
            .join(", ")
    };
    out.push_str(&format!(
        "{:<40} {:<10} {}\n",
        "Rate \u{2265} 99% (3 consecutive)",
        status_str(criteria.rate_met),
        rate_detail,
    ));

    // 2. Stability
    let delta_detail = if criteria.delta_values.is_empty() {
        "no history".to_string()
    } else {
        criteria
            .delta_values
            .iter()
            .map(|d| format!("{:+.2}%", d * 100.0))
            .collect::<Vec<_>>()
            .join(", ")
    };
    out.push_str(&format!(
        "{:<40} {:<10} {}\n",
        "Stability (delta < 0.5%, 3 consec.)",
        status_str(criteria.stability_met),
        delta_detail,
    ));

    // 3. Growth
    out.push_str(&format!(
        "{:<40} {:<10} {}/{}\n",
        "Corpus size \u{2265} target",
        status_str(criteria.growth_met),
        criteria.corpus_size,
        criteria.target_size,
    ));

    // 4. No regressions
    out.push_str(&format!(
        "{:<40} {:<10} {}\n",
        "No regressions (Jidoka)",
        status_str(criteria.no_regressions),
        if criteria.no_regressions {
            "clean"
        } else {
            "regressions found"
        },
    ));

    out.push_str(&format!("{}\n", line));

    if criteria.converged {
        out.push_str("CONVERGED: All 4 criteria met (Shewhart 1931)\n");
        out.push_str("Action: Add harder entries to challenge the transpiler (\u{00a7}1.3)\n");
    } else {
        let failing = [
            (!criteria.rate_met, "rate"),
            (!criteria.stability_met, "stability"),
            (!criteria.growth_met, "growth"),
            (!criteria.no_regressions, "regressions"),
        ];
        let names: Vec<&str> = failing
            .iter()
            .filter(|(f, _)| *f)
            .map(|(_, n)| *n)
            .collect();
        out.push_str(&format!(
            "NOT CONVERGED: {} criteria failing ({})\n",
            names.len(),
            names.join(", "),
        ));
    }

    out
}

fn status_str(passed: bool) -> &'static str {
    if passed {
        "\u{2713} PASS"
    } else {
        "\u{2717} FAIL"
    }
}

#[cfg(test)]
#[path = "citl_tests_extracted.rs"]
mod tests_extracted;
