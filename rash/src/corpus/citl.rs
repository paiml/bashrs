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
pub fn lint_pipeline(
    registry: &CorpusRegistry,
    score: &CorpusScore,
) -> Vec<LintPipelineEntry> {
    let mut suggestions = Vec::new();
    let max_id = find_max_corpus_id(registry);
    let mut next_id = max_id + 1;

    for result in &score.results {
        if !result.transpiled || result.lint_clean {
            continue;
        }
        // This entry transpiled but failed lint — generate suggestion
        let entry = registry.entries.iter().find(|e| e.id == result.id);
        let format = entry.map(|e| e.format).unwrap_or(CorpusFormat::Bash);
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
                .last()
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
pub fn check_regressions(
    score: &CorpusScore,
    history: &[ConvergenceEntry],
) -> RegressionReport {
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
pub fn check_convergence(
    score: &CorpusScore,
    history: &[ConvergenceEntry],
) -> ConvergenceCriteria {
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
    out.push_str(&format!("{} suggestion(s) from CITL lint pipeline\n", suggestions.len()));

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
        out.push_str(&format!("\nImprovements:\n"));
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
        let names: Vec<&str> = failing.iter().filter(|(f, _)| *f).map(|(_, n)| *n).collect();
        out.push_str(&format!(
            "NOT CONVERGED: {} criteria failing ({})\n",
            names.len(),
            names.join(", "),
        ));
    }

    out
}

fn status_str(passed: bool) -> &'static str {
    if passed { "\u{2713} PASS" } else { "\u{2717} FAIL" }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::corpus::registry::{CorpusEntry, CorpusTier};

    fn make_entry(id: &str, format: CorpusFormat) -> CorpusEntry {
        CorpusEntry {
            id: id.to_string(),
            name: format!("test-{id}"),
            description: "Test entry".to_string(),
            format,
            tier: CorpusTier::Trivial,
            input: String::new(),
            expected_output: "#!/bin/sh\necho ok\n".to_string(),
            shellcheck: true,
            deterministic: true,
            idempotent: true,
        }
    }

    fn make_result(id: &str, transpiled: bool, lint_clean: bool) -> CorpusResult {
        CorpusResult {
            id: id.to_string(),
            transpiled,
            output_contains: transpiled,
            output_exact: transpiled,
            output_behavioral: transpiled,
            has_test: true,
            coverage_ratio: 0.95,
            schema_valid: true,
            lint_clean,
            deterministic: transpiled,
            metamorphic_consistent: transpiled,
            cross_shell_agree: transpiled,
            actual_output: if transpiled { Some("#!/bin/sh\n".into()) } else { None },
            error: if !lint_clean {
                Some("SEC003: unquoted variable".into())
            } else if !transpiled {
                Some("transpilation failed".into())
            } else {
                None
            },
            error_category: None,
            error_confidence: None,
            decision_trace: None,
        }
    }

    fn make_convergence(iter: u32, rate: f64, delta: f64, total: usize, passed: usize) -> ConvergenceEntry {
        ConvergenceEntry {
            iteration: iter,
            date: "2026-02-09".into(),
            total,
            passed,
            failed: total - passed,
            rate,
            delta,
            notes: String::new(),
            ..Default::default()
        }
    }

    #[test]
    fn test_extract_lint_rule_sec() {
        assert_eq!(extract_lint_rule("SEC003: unquoted variable"), "SEC003");
    }

    #[test]
    fn test_extract_lint_rule_det() {
        assert_eq!(extract_lint_rule("DET001: non-deterministic"), "DET001");
    }

    #[test]
    fn test_extract_lint_rule_make() {
        assert_eq!(extract_lint_rule("MAKE005 tab issue"), "MAKE005");
    }

    #[test]
    fn test_extract_lint_rule_docker() {
        assert_eq!(extract_lint_rule("DOCKER003: no USER"), "DOCKER003");
    }

    #[test]
    fn test_extract_lint_rule_unknown() {
        assert_eq!(extract_lint_rule("some other error"), "LINT-UNKNOWN");
    }

    #[test]
    fn test_generate_entry_name_security() {
        let name = generate_entry_name("SEC003", "B-001");
        assert_eq!(name, "security-violation-from-B-001");
    }

    #[test]
    fn test_generate_entry_name_determinism() {
        let name = generate_entry_name("DET001", "B-050");
        assert_eq!(name, "determinism-violation-from-B-050");
    }

    #[test]
    fn test_generate_entry_name_dockerfile() {
        let name = generate_entry_name("DOCKER003", "D-010");
        assert_eq!(name, "dockerfile-lint-violation-from-D-010");
    }

    #[test]
    fn test_find_max_corpus_id() {
        let registry = CorpusRegistry {
            entries: vec![
                make_entry("B-001", CorpusFormat::Bash),
                make_entry("B-500", CorpusFormat::Bash),
                make_entry("M-200", CorpusFormat::Makefile),
            ],
            ..Default::default()
        };
        assert_eq!(find_max_corpus_id(&registry), 500);
    }

    #[test]
    fn test_find_max_corpus_id_empty() {
        let registry = CorpusRegistry::default();
        assert_eq!(find_max_corpus_id(&registry), 0);
    }

    #[test]
    fn test_lint_pipeline_clean() {
        let registry = CorpusRegistry {
            entries: vec![make_entry("B-001", CorpusFormat::Bash)],
            ..Default::default()
        };
        let score = CorpusScore {
            total: 1,
            passed: 1,
            failed: 0,
            rate: 1.0,
            score: 100.0,
            grade: crate::corpus::registry::Grade::APlus,
            format_scores: vec![],
            results: vec![make_result("B-001", true, true)],
        };

        let suggestions = lint_pipeline(&registry, &score);
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_lint_pipeline_violation() {
        let registry = CorpusRegistry {
            entries: vec![make_entry("B-001", CorpusFormat::Bash)],
            ..Default::default()
        };
        let score = CorpusScore {
            total: 1,
            passed: 1,
            failed: 0,
            rate: 1.0,
            score: 80.0,
            grade: crate::corpus::registry::Grade::B,
            format_scores: vec![],
            results: vec![make_result("B-001", true, false)],
        };

        let suggestions = lint_pipeline(&registry, &score);
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].source_id, "B-001");
        assert_eq!(suggestions[0].rule, "SEC003");
        assert!(suggestions[0].suggested_id.starts_with("B-"));
    }

    #[test]
    fn test_lint_pipeline_not_transpiled() {
        let registry = CorpusRegistry {
            entries: vec![make_entry("B-001", CorpusFormat::Bash)],
            ..Default::default()
        };
        let score = CorpusScore {
            total: 1,
            passed: 0,
            failed: 1,
            rate: 0.0,
            score: 0.0,
            grade: crate::corpus::registry::Grade::F,
            format_scores: vec![],
            results: vec![make_result("B-001", false, false)],
        };

        // Entries that didn't transpile shouldn't generate lint suggestions
        let suggestions = lint_pipeline(&registry, &score);
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_check_regressions_clean() {
        let score = CorpusScore {
            total: 10,
            passed: 10,
            failed: 0,
            rate: 1.0,
            score: 99.9,
            grade: crate::corpus::registry::Grade::APlus,
            format_scores: vec![],
            results: vec![],
        };
        let history = vec![make_convergence(1, 1.0, 0.0, 10, 10)];

        let report = check_regressions(&score, &history);
        assert!(report.regressions.is_empty());
        assert!(!report.andon_triggered);
    }

    #[test]
    fn test_check_regressions_detected() {
        let score = CorpusScore {
            total: 10,
            passed: 8,
            failed: 2,
            rate: 0.8,
            score: 80.0,
            grade: crate::corpus::registry::Grade::B,
            format_scores: vec![],
            results: vec![
                make_result("B-001", false, false),
                make_result("B-002", false, false),
            ],
        };
        let history = vec![make_convergence(1, 1.0, 0.0, 10, 10)];

        let report = check_regressions(&score, &history);
        assert_eq!(report.regressions.len(), 2);
        assert!(report.andon_triggered);
    }

    #[test]
    fn test_check_regressions_no_history() {
        let score = CorpusScore {
            total: 10,
            passed: 9,
            failed: 1,
            rate: 0.9,
            score: 90.0,
            grade: crate::corpus::registry::Grade::A,
            format_scores: vec![],
            results: vec![make_result("B-001", false, false)],
        };

        let report = check_regressions(&score, &[]);
        assert!(!report.andon_triggered);
    }

    #[test]
    fn test_convergence_criteria_met() {
        let score = CorpusScore {
            total: 900,
            passed: 900,
            failed: 0,
            rate: 1.0,
            score: 99.9,
            grade: crate::corpus::registry::Grade::APlus,
            format_scores: vec![],
            results: vec![],
        };
        let history = vec![
            make_convergence(1, 1.0, 0.0, 900, 900),
            make_convergence(2, 1.0, 0.0, 900, 900),
            make_convergence(3, 1.0, 0.0, 900, 900),
        ];

        let criteria = check_convergence(&score, &history);
        assert!(criteria.rate_met);
        assert!(criteria.stability_met);
        assert!(criteria.growth_met);
        assert!(criteria.no_regressions);
        assert!(criteria.converged);
    }

    #[test]
    fn test_convergence_criteria_rate_not_met() {
        let score = CorpusScore {
            total: 900,
            passed: 890,
            failed: 10,
            rate: 0.989,
            score: 90.0,
            grade: crate::corpus::registry::Grade::A,
            format_scores: vec![],
            results: vec![],
        };
        let history = vec![
            make_convergence(1, 0.95, -0.05, 900, 855),
            make_convergence(2, 0.98, 0.03, 900, 882),
            make_convergence(3, 0.989, 0.009, 900, 890),
        ];

        let criteria = check_convergence(&score, &history);
        assert!(!criteria.rate_met);
        assert!(!criteria.converged);
    }

    #[test]
    fn test_convergence_criteria_growth_not_met() {
        let score = CorpusScore {
            total: 500,
            passed: 500,
            failed: 0,
            rate: 1.0,
            score: 99.9,
            grade: crate::corpus::registry::Grade::APlus,
            format_scores: vec![],
            results: vec![],
        };
        let history = vec![
            make_convergence(1, 1.0, 0.0, 500, 500),
            make_convergence(2, 1.0, 0.0, 500, 500),
            make_convergence(3, 1.0, 0.0, 500, 500),
        ];

        let criteria = check_convergence(&score, &history);
        assert!(criteria.rate_met);
        assert!(criteria.stability_met);
        assert!(!criteria.growth_met);
        assert!(!criteria.converged);
    }

    #[test]
    fn test_convergence_insufficient_history() {
        let score = CorpusScore {
            total: 900,
            passed: 900,
            failed: 0,
            rate: 1.0,
            score: 99.9,
            grade: crate::corpus::registry::Grade::APlus,
            format_scores: vec![],
            results: vec![],
        };
        let history = vec![make_convergence(1, 1.0, 0.0, 900, 900)];

        let criteria = check_convergence(&score, &history);
        assert!(!criteria.rate_met); // need 3 entries
        assert!(!criteria.converged);
    }

    #[test]
    fn test_format_lint_pipeline_clean() {
        let table = format_lint_pipeline(&[]);
        assert!(table.contains("No lint violations"));
        assert!(table.contains("CITL loop clean"));
    }

    #[test]
    fn test_format_lint_pipeline_with_suggestions() {
        let suggestions = vec![LintPipelineEntry {
            source_id: "B-001".into(),
            rule: "SEC003".into(),
            message: "unquoted variable".into(),
            suggested_id: "B-501".into(),
            suggested_name: "security-violation-from-B-001".into(),
            format: CorpusFormat::Bash,
        }];
        let table = format_lint_pipeline(&suggestions);
        assert!(table.contains("B-001"));
        assert!(table.contains("SEC003"));
        assert!(table.contains("B-501"));
        assert!(table.contains("1 suggestion(s)"));
    }

    #[test]
    fn test_format_regression_report_clean() {
        let report = RegressionReport {
            regressions: vec![],
            improvements: vec![],
            total: 900,
            andon_triggered: false,
        };
        let table = format_regression_report(&report);
        assert!(table.contains("No regressions"));
        assert!(table.contains("OK"));
    }

    #[test]
    fn test_format_regression_report_with_regressions() {
        let report = RegressionReport {
            regressions: vec![RegressionEntry {
                id: "B-143".into(),
                format: CorpusFormat::Bash,
                error: "behavioral check failed".into(),
            }],
            improvements: vec![],
            total: 900,
            andon_triggered: true,
        };
        let table = format_regression_report(&report);
        assert!(table.contains("REGRESSIONS DETECTED"));
        assert!(table.contains("B-143"));
        assert!(table.contains("ANDON CORD"));
    }

    #[test]
    fn test_format_convergence_criteria_converged() {
        let criteria = ConvergenceCriteria {
            rate_met: true,
            rate_values: vec![1.0, 1.0, 1.0],
            stability_met: true,
            delta_values: vec![0.0, 0.0, 0.0],
            growth_met: true,
            corpus_size: 900,
            target_size: 900,
            no_regressions: true,
            converged: true,
        };
        let table = format_convergence_criteria(&criteria);
        assert!(table.contains("CONVERGED"));
        assert!(table.contains("Shewhart"));
    }

    #[test]
    fn test_format_convergence_criteria_not_converged() {
        let criteria = ConvergenceCriteria {
            rate_met: true,
            rate_values: vec![1.0, 1.0, 1.0],
            stability_met: false,
            delta_values: vec![0.01, -0.02, 0.03],
            growth_met: true,
            corpus_size: 900,
            target_size: 900,
            no_regressions: true,
            converged: false,
        };
        let table = format_convergence_criteria(&criteria);
        assert!(table.contains("NOT CONVERGED"));
        assert!(table.contains("stability"));
    }

    #[test]
    fn test_guess_format() {
        assert_eq!(guess_format("B-001"), CorpusFormat::Bash);
        assert_eq!(guess_format("M-042"), CorpusFormat::Makefile);
        assert_eq!(guess_format("D-015"), CorpusFormat::Dockerfile);
    }

    #[test]
    fn test_format_prefix() {
        assert_eq!(format_prefix(CorpusFormat::Bash), "B");
        assert_eq!(format_prefix(CorpusFormat::Makefile), "M");
        assert_eq!(format_prefix(CorpusFormat::Dockerfile), "D");
    }

    #[test]
    fn test_status_str() {
        assert!(status_str(true).contains("PASS"));
        assert!(status_str(false).contains("FAIL"));
    }
}
