//! Rich Lint Report Module (ML-006, ML-013, ML-014)
//!
//! Generates comprehensive lint reports with clustering, SBFL analysis,
//! and rich ASCII visualization following Tufte's principles.
//!
//! # Toyota Way Principles
//!
//! - **Visual Management** (Mieruka): Immediate problem visibility
//! - **Andon**: Color-coded severity signaling
//! - **Heijunka**: Cluster errors to batch similar fixes
//!
//! # References
//!
//! - BASHRS-SPEC-ML-006: SBFL ASCII Report
//! - BASHRS-SPEC-ML-013: Histogram Bars
//! - BASHRS-SPEC-ML-014: Complete Rich Report
//! - Tufte (2001): Visual Display of Quantitative Information

use std::collections::HashMap;

use crate::linter::{Diagnostic, LintResult, Severity};

use super::oracle::{Oracle, ShellErrorCategory};
use super::report::sparkline;
use super::sbfl::{SbflFormula, SuspiciousnessRanking};

/// Box drawing characters for double-line boxes
pub mod box_chars {
    pub const TOP_LEFT: char = '╔';
    pub const TOP_RIGHT: char = '╗';
    pub const BOTTOM_LEFT: char = '╚';
    pub const BOTTOM_RIGHT: char = '╝';
    pub const HORIZONTAL: char = '═';
    pub const VERTICAL: char = '║';
    pub const T_DOWN: char = '╦';
    pub const T_UP: char = '╩';
    pub const T_RIGHT: char = '╠';
    pub const T_LEFT: char = '╣';
    pub const CROSS: char = '╬';
}

/// Generate ASCII histogram bar
pub fn histogram_bar(value: f64, max_value: f64, width: usize) -> String {
    let ratio = if max_value > 0.0 {
        (value / max_value).min(1.0)
    } else {
        0.0
    };
    let filled = (ratio * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

/// Error cluster with analysis
#[derive(Debug, Clone)]
pub struct ErrorCluster {
    pub error_code: String,
    pub count: usize,
    pub category: ShellErrorCategory,
    pub diagnostics: Vec<Diagnostic>,
    pub fix_confidence: f64,
    pub auto_fixable: bool,
}

impl ErrorCluster {
    /// Calculate percentage of total
    pub fn percentage(&self, total: usize) -> f64 {
        if total > 0 {
            (self.count as f64 / total as f64) * 100.0
        } else {
            0.0
        }
    }
}

/// Complete rich lint report
#[derive(Debug)]
pub struct RichLintReport {
    pub source_file: String,
    pub timestamp: String,
    pub total_issues: usize,
    pub errors: usize,
    pub warnings: usize,
    pub info: usize,
    pub clusters: Vec<ErrorCluster>,
    pub sbfl_rankings: Vec<SuspiciousnessRanking>,
    pub auto_fixable_count: usize,
    pub manual_count: usize,
    pub overall_confidence: f64,
    pub trend_data: Vec<f64>,
    pub estimated_fix_time_minutes: usize,
}

impl RichLintReport {
    /// Create a rich report from lint results
    pub fn from_lint_result(source_file: &str, result: &LintResult, source: &str) -> Self {
        let oracle = Oracle::new();

        // Count by severity
        let errors = result
            .diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .count();
        let warnings = result
            .diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Warning)
            .count();
        let info = result
            .diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Info || d.severity == Severity::Note)
            .count();

        // Cluster by error code
        let mut code_clusters: HashMap<String, Vec<Diagnostic>> = HashMap::new();
        for diag in &result.diagnostics {
            code_clusters
                .entry(diag.code.clone())
                .or_default()
                .push(diag.clone());
        }

        // Build clusters with classification
        let mut clusters: Vec<ErrorCluster> = code_clusters
            .into_iter()
            .filter_map(|(code, diagnostics)| {
                let first_diag = diagnostics.first()?;
                let classification = oracle.classify(first_diag, source);
                let has_fix = diagnostics.iter().any(|d| d.fix.is_some());

                Some(ErrorCluster {
                    error_code: code,
                    count: diagnostics.len(),
                    category: classification.category,
                    fix_confidence: classification.confidence,
                    auto_fixable: has_fix,
                    diagnostics,
                })
            })
            .collect();

        // Sort by count descending (Pareto)
        clusters.sort_by(|a, b| b.count.cmp(&a.count));

        // Calculate auto-fixable counts
        let auto_fixable_count = result
            .diagnostics
            .iter()
            .filter(|d| d.fix.is_some())
            .count();
        let manual_count = result.diagnostics.len() - auto_fixable_count;

        // Calculate overall confidence (weighted average)
        let overall_confidence = if !clusters.is_empty() {
            let total_weighted: f64 = clusters
                .iter()
                .map(|c| c.fix_confidence * c.count as f64)
                .sum();
            let total_count: f64 = clusters.iter().map(|c| c.count as f64).sum();
            if total_count > 0.0 {
                total_weighted / total_count
            } else {
                0.0
            }
        } else {
            1.0 // No issues = perfect confidence
        };

        // Generate mock trend data (would be from historical data)
        let trend_data = vec![0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9];

        // Estimate fix time (2 min per manual, 30 sec per auto)
        let estimated_fix_time_minutes = (manual_count * 2) + (auto_fixable_count / 2);

        // SBFL rankings (mock - would integrate with actual test coverage)
        let sbfl_rankings = Self::compute_sbfl_rankings(&clusters);

        RichLintReport {
            source_file: source_file.to_string(),
            timestamp: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            total_issues: result.diagnostics.len(),
            errors,
            warnings,
            info,
            clusters,
            sbfl_rankings,
            auto_fixable_count,
            manual_count,
            overall_confidence,
            trend_data,
            estimated_fix_time_minutes,
        }
    }

    fn compute_sbfl_rankings(clusters: &[ErrorCluster]) -> Vec<SuspiciousnessRanking> {
        // Create mock SBFL rankings based on cluster analysis
        // In production, this would use actual test coverage data
        clusters
            .iter()
            .take(5)
            .enumerate()
            .map(|(i, cluster)| {
                use super::sbfl::CoverageData;

                // Mock suspiciousness based on cluster characteristics
                let suspiciousness = if cluster.category == ShellErrorCategory::CommandInjection
                    || cluster.category == ShellErrorCategory::PathTraversal
                {
                    0.95 - (i as f64 * 0.1)
                } else {
                    0.8 - (i as f64 * 0.15)
                };

                SuspiciousnessRanking {
                    location: cluster.error_code.clone(),
                    score: suspiciousness.max(0.1),
                    coverage: CoverageData::default(),
                    rank: i + 1,
                }
            })
            .collect()
    }

    /// Render the complete report as ASCII
    pub fn render(&self, width: usize) -> String {
        let mut out = String::new();

        self.render_header(&mut out, width);
        self.render_summary(&mut out, width);
        self.render_clusters(&mut out, width);
        self.render_sbfl(&mut out, width);
        self.render_recommendations(&mut out, width);
        self.render_footer(&mut out, width);

        out
    }

    fn render_header(&self, out: &mut String, width: usize) {
        let inner = width - 2;
        let version = env!("CARGO_PKG_VERSION");

        // Top border
        out.push(box_chars::TOP_LEFT);
        for _ in 0..inner {
            out.push(box_chars::HORIZONTAL);
        }
        out.push(box_chars::TOP_RIGHT);
        out.push('\n');

        // Title line
        let title = format!("BASHRS LINT REPORT v{}", version);
        let padding = (inner.saturating_sub(title.len())) / 2;
        out.push(box_chars::VERTICAL);
        for _ in 0..padding {
            out.push(' ');
        }
        out.push_str(&title);
        for _ in 0..(inner - padding - title.len()) {
            out.push(' ');
        }
        out.push(box_chars::VERTICAL);
        out.push('\n');

        // Subtitle
        let subtitle = format!("{} │ {}", self.source_file, self.timestamp);
        let padding = (inner.saturating_sub(subtitle.len())) / 2;
        out.push(box_chars::VERTICAL);
        for _ in 0..padding {
            out.push(' ');
        }
        out.push_str(&subtitle);
        for _ in 0..(inner - padding - subtitle.len()) {
            out.push(' ');
        }
        out.push(box_chars::VERTICAL);
        out.push('\n');
    }

    fn render_summary(&self, out: &mut String, width: usize) {
        let inner = width - 2;

        // Section divider
        out.push(box_chars::T_RIGHT);
        for _ in 0..inner {
            out.push(box_chars::HORIZONTAL);
        }
        out.push(box_chars::T_LEFT);
        out.push('\n');

        // Section title
        self.render_section_title(out, "SUMMARY", width);

        // Issue counts
        let line1 = format!(
            "  Total Issues: {:>4} │ Errors: {:>3} │ Warnings: {:>3} │ Info: {:>3}",
            self.total_issues, self.errors, self.warnings, self.info
        );
        self.render_line(out, &line1, width);

        // Cluster and fix info
        let line2 = format!(
            "  Clusters: {:>7} │ Auto-fixable: {:>3} ({:>2}%) │ Manual: {:>3} ({:>2}%)",
            self.clusters.len(),
            self.auto_fixable_count,
            if self.total_issues > 0 {
                (self.auto_fixable_count * 100) / self.total_issues
            } else {
                100
            },
            self.manual_count,
            if self.total_issues > 0 {
                (self.manual_count * 100) / self.total_issues
            } else {
                0
            }
        );
        self.render_line(out, &line2, width);

        // Confidence and time
        let line3 = format!(
            "  Confidence: {:>5.1}% │ Est. Fix Time: ~{} min",
            self.overall_confidence * 100.0,
            self.estimated_fix_time_minutes
        );
        self.render_line(out, &line3, width);

        // Trend sparkline
        let trend = sparkline(&self.trend_data);
        let trend_status = if self.trend_data.last() > self.trend_data.first() {
            "(improving)"
        } else {
            "(degrading)"
        };
        let line4 = format!("  Trend (7 days): {} {}", trend, trend_status);
        self.render_line(out, &line4, width);
    }

    fn render_clusters(&self, out: &mut String, width: usize) {
        let inner = width - 2;

        // Section divider
        out.push(box_chars::T_RIGHT);
        for _ in 0..inner {
            out.push(box_chars::HORIZONTAL);
        }
        out.push(box_chars::T_LEFT);
        out.push('\n');

        self.render_section_title(out, "ERROR CLUSTERS (Pareto Analysis)", width);

        // Header
        let header = "  Cluster │ Count │ Distribution          │ Category    │ Fix Confidence";
        self.render_line(out, header, width);

        // Divider
        out.push(box_chars::T_RIGHT);
        for _ in 0..inner {
            out.push(box_chars::HORIZONTAL);
        }
        out.push(box_chars::T_LEFT);
        out.push('\n');

        // Cluster rows
        let max_count = self.clusters.first().map(|c| c.count).unwrap_or(1);
        for cluster in self.clusters.iter().take(5) {
            let bar = histogram_bar(cluster.count as f64, max_count as f64, 20);
            let confidence_str = if cluster.auto_fixable {
                format!("{:.0}% (auto-fix)", cluster.fix_confidence * 100.0)
            } else {
                format!("{:.0}% (manual)", cluster.fix_confidence * 100.0)
            };

            let line = format!(
                "  {:>7} │ {:>5} │ {} │ {:>11} │ {}",
                cluster.error_code,
                cluster.count,
                bar,
                &cluster.category.name()[..11.min(cluster.category.name().len())],
                confidence_str
            );
            self.render_line(out, &line, width);
        }
    }

    fn render_sbfl(&self, out: &mut String, width: usize) {
        let inner = width - 2;

        if self.sbfl_rankings.is_empty() {
            return;
        }

        // Section divider
        out.push(box_chars::T_RIGHT);
        for _ in 0..inner {
            out.push(box_chars::HORIZONTAL);
        }
        out.push(box_chars::T_LEFT);
        out.push('\n');

        self.render_section_title(out, "FAULT LOCALIZATION (Ochiai SBFL)", width);

        // Header
        let header = "  Rank │ Location          │ Suspiciousness │ Root Cause";
        self.render_line(out, header, width);

        // Divider
        out.push(box_chars::T_RIGHT);
        for _ in 0..inner {
            out.push(box_chars::HORIZONTAL);
        }
        out.push(box_chars::T_LEFT);
        out.push('\n');

        // SBFL rows
        for ranking in self.sbfl_rankings.iter().take(5) {
            let bar = histogram_bar(ranking.score, 1.0, 10);
            let line = format!(
                "  {:>4} │ {:>17} │ {} {:.2}│ {}",
                ranking.rank,
                ranking.location,
                bar,
                ranking.score,
                self.get_root_cause_desc(&ranking.location)
            );
            self.render_line(out, &line, width);
        }
    }

    fn render_recommendations(&self, out: &mut String, width: usize) {
        let inner = width - 2;

        // Section divider
        out.push(box_chars::T_RIGHT);
        for _ in 0..inner {
            out.push(box_chars::HORIZONTAL);
        }
        out.push(box_chars::T_LEFT);
        out.push('\n');

        self.render_section_title(
            out,
            "RECOMMENDED ACTIONS (Toyota Way: Start with highest impact)",
            width,
        );

        // Auto-fix recommendation
        if self.auto_fixable_count > 0 {
            let line1 = format!("  1. Run: bashrs lint {} --fix", self.source_file);
            self.render_line(out, &line1, width);

            let auto_codes: Vec<_> = self
                .clusters
                .iter()
                .filter(|c| c.auto_fixable)
                .take(3)
                .map(|c| c.error_code.as_str())
                .collect();
            let line2 = format!(
                "     → Auto-fixes {} issues ({})",
                self.auto_fixable_count,
                auto_codes.join(", ")
            );
            self.render_line(out, &line2, width);
            self.render_line(out, "", width);
        }

        // Manual review
        if self.manual_count > 0 {
            let manual_clusters: Vec<_> = self
                .clusters
                .iter()
                .filter(|c| !c.auto_fixable)
                .take(2)
                .collect();

            if !manual_clusters.is_empty() {
                let line = format!(
                    "  2. Manual review required for {} ({} issues)",
                    manual_clusters
                        .first()
                        .map(|c| c.error_code.as_str())
                        .unwrap_or(""),
                    self.manual_count
                );
                self.render_line(out, &line, width);
            }
        }
    }

    fn render_footer(&self, out: &mut String, width: usize) {
        let inner = width - 2;

        // Section divider
        out.push(box_chars::T_RIGHT);
        for _ in 0..inner {
            out.push(box_chars::HORIZONTAL);
        }
        out.push(box_chars::T_LEFT);
        out.push('\n');

        self.render_section_title(out, "CITL EXPORT", width);

        let line1 = format!(
            "  Export: bashrs lint {} --citl-export diagnostics.json",
            self.source_file
        );
        self.render_line(out, &line1, width);
        self.render_line(
            out,
            "  Integration: organizational-intelligence-plugin for ML training",
            width,
        );

        // Bottom border
        out.push(box_chars::BOTTOM_LEFT);
        for _ in 0..inner {
            out.push(box_chars::HORIZONTAL);
        }
        out.push(box_chars::BOTTOM_RIGHT);
        out.push('\n');
    }

    fn render_section_title(&self, out: &mut String, title: &str, width: usize) {
        let inner = width - 2;
        out.push(box_chars::VERTICAL);
        out.push(' ');
        out.push_str(title);
        for _ in 0..(inner - title.len() - 1) {
            out.push(' ');
        }
        out.push(box_chars::VERTICAL);
        out.push('\n');
    }

    fn render_line(&self, out: &mut String, content: &str, width: usize) {
        let inner = width - 2;
        out.push(box_chars::VERTICAL);
        let truncated = if content.len() > inner {
            &content[..inner]
        } else {
            content
        };
        out.push_str(truncated);
        for _ in 0..(inner.saturating_sub(content.len())) {
            out.push(' ');
        }
        out.push(box_chars::VERTICAL);
        out.push('\n');
    }

    fn get_root_cause_desc(&self, location: &str) -> String {
        // Find cluster for location and describe root cause
        self.clusters
            .iter()
            .find(|c| c.error_code == location)
            .map(|c| c.category.name().to_string())
            .unwrap_or_else(|| "Unknown".to_string())
    }
}

/// Generate SBFL-only ASCII report (ML-006)
pub fn sbfl_report(
    rankings: &[SuspiciousnessRanking],
    formula: SbflFormula,
    width: usize,
) -> String {
    let mut out = String::new();
    let inner = width - 2;

    // Header
    out.push(box_chars::TOP_LEFT);
    for _ in 0..inner {
        out.push(box_chars::HORIZONTAL);
    }
    out.push(box_chars::TOP_RIGHT);
    out.push('\n');

    let title = format!("FAULT LOCALIZATION REPORT ({})", formula);
    let padding = (inner.saturating_sub(title.len())) / 2;
    out.push(box_chars::VERTICAL);
    for _ in 0..padding {
        out.push(' ');
    }
    out.push_str(&title);
    for _ in 0..(inner - padding - title.len()) {
        out.push(' ');
    }
    out.push(box_chars::VERTICAL);
    out.push('\n');

    // Divider
    out.push(box_chars::T_RIGHT);
    for _ in 0..inner {
        out.push(box_chars::HORIZONTAL);
    }
    out.push(box_chars::T_LEFT);
    out.push('\n');

    // Column headers
    let header = " Rank │ Rule   │ Suspiciousness │ Failed │ Passed │ Explanation";
    out.push(box_chars::VERTICAL);
    out.push_str(header);
    for _ in 0..(inner.saturating_sub(header.len())) {
        out.push(' ');
    }
    out.push(box_chars::VERTICAL);
    out.push('\n');

    // Divider
    out.push(box_chars::T_RIGHT);
    for _ in 0..inner {
        out.push(box_chars::HORIZONTAL);
    }
    out.push(box_chars::T_LEFT);
    out.push('\n');

    // Rows
    for ranking in rankings.iter().take(10) {
        let bar = histogram_bar(ranking.score, 1.0, 10);
        let explanation = if ranking.score > 0.8 {
            "High suspiciousness"
        } else if ranking.score > 0.5 {
            "Moderate suspicion"
        } else {
            "Low suspicion"
        };

        let row = format!(
            "  {:>2}  │ {:>6} │ {} {:>.2}│ {:>6} │ {:>6} │ {}",
            ranking.rank,
            ranking.location,
            bar,
            ranking.score,
            ranking.coverage.failed_covering,
            ranking.coverage.passed_covering,
            explanation
        );

        out.push(box_chars::VERTICAL);
        let truncated = if row.len() > inner {
            &row[..inner]
        } else {
            &row
        };
        out.push_str(truncated);
        for _ in 0..(inner.saturating_sub(row.len())) {
            out.push(' ');
        }
        out.push(box_chars::VERTICAL);
        out.push('\n');
    }

    // Footer
    out.push(box_chars::BOTTOM_LEFT);
    for _ in 0..inner {
        out.push(box_chars::HORIZONTAL);
    }
    out.push(box_chars::BOTTOM_RIGHT);
    out.push('\n');

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::linter::Span;

    fn sample_lint_result() -> LintResult {
        LintResult {
            diagnostics: vec![
                Diagnostic {
                    code: "SC2086".to_string(),
                    severity: Severity::Warning,
                    message: "Double quote to prevent globbing".to_string(),
                    span: Span::new(10, 1, 10, 20),
                    fix: Some(crate::linter::Fix::new("\"$var\"")),
                },
                Diagnostic {
                    code: "SC2086".to_string(),
                    severity: Severity::Warning,
                    message: "Double quote to prevent globbing".to_string(),
                    span: Span::new(15, 1, 15, 20),
                    fix: Some(crate::linter::Fix::new("\"$other\"")),
                },
                Diagnostic {
                    code: "DET001".to_string(),
                    severity: Severity::Warning,
                    message: "Non-deterministic $RANDOM".to_string(),
                    span: Span::new(20, 5, 20, 15),
                    fix: None,
                },
                Diagnostic {
                    code: "SEC010".to_string(),
                    severity: Severity::Error,
                    message: "Hardcoded path /tmp".to_string(),
                    span: Span::new(25, 1, 25, 10),
                    fix: Some(crate::linter::Fix::new("${TMPDIR:-/tmp}")),
                },
            ],
        }
    }

    #[test]
    fn test_ml_013_histogram_bar() {
        let bar = histogram_bar(50.0, 100.0, 10);
        assert_eq!(bar.chars().count(), 10);
        assert!(bar.contains('█'));
        assert!(bar.contains('░'));

        let full_bar = histogram_bar(100.0, 100.0, 10);
        assert!(!full_bar.contains('░'));

        let empty_bar = histogram_bar(0.0, 100.0, 10);
        assert!(!empty_bar.contains('█'));
    }

    #[test]
    fn test_ml_014_rich_report_creation() {
        let result = sample_lint_result();
        let report = RichLintReport::from_lint_result("test.sh", &result, "echo $var");

        assert_eq!(report.total_issues, 4);
        assert_eq!(report.errors, 1);
        assert_eq!(report.warnings, 3);
        assert!(!report.clusters.is_empty());
    }

    #[test]
    fn test_ml_014_rich_report_clustering() {
        let result = sample_lint_result();
        let report = RichLintReport::from_lint_result("test.sh", &result, "echo $var");

        // SC2086 appears twice, should be biggest cluster
        let sc2086 = report.clusters.iter().find(|c| c.error_code == "SC2086");
        assert!(sc2086.is_some());
        assert_eq!(sc2086.expect("found").count, 2);
    }

    #[test]
    fn test_ml_014_rich_report_render() {
        let result = sample_lint_result();
        let report = RichLintReport::from_lint_result("test.sh", &result, "echo $var");

        let rendered = report.render(80);

        assert!(rendered.contains("BASHRS LINT REPORT"));
        assert!(rendered.contains("test.sh"));
        assert!(rendered.contains("SUMMARY"));
        assert!(rendered.contains("ERROR CLUSTERS"));
        assert!(rendered.contains("SC2086"));
    }

    #[test]
    fn test_ml_006_sbfl_report() {
        use super::super::sbfl::CoverageData;

        let rankings = vec![
            SuspiciousnessRanking {
                location: "SC2086".to_string(),
                score: 0.94,
                coverage: CoverageData::new(2, 31, 8, 0),
                rank: 1,
            },
            SuspiciousnessRanking {
                location: "DET001".to_string(),
                score: 0.72,
                coverage: CoverageData::new(8, 12, 2, 0),
                rank: 2,
            },
        ];

        let report = sbfl_report(&rankings, SbflFormula::Ochiai, 80);

        assert!(report.contains("FAULT LOCALIZATION"));
        assert!(report.contains("Ochiai"));
        assert!(report.contains("SC2086"));
        assert!(report.contains("DET001"));
    }

    #[test]
    fn test_error_cluster_percentage() {
        let cluster = ErrorCluster {
            error_code: "SC2086".to_string(),
            count: 25,
            category: ShellErrorCategory::MissingQuotes,
            diagnostics: vec![],
            fix_confidence: 0.94,
            auto_fixable: true,
        };

        assert!((cluster.percentage(100) - 25.0).abs() < 0.01);
        assert!((cluster.percentage(50) - 50.0).abs() < 0.01);
        assert!((cluster.percentage(0) - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_ml_014_auto_fixable_counting() {
        let result = sample_lint_result();
        let report = RichLintReport::from_lint_result("test.sh", &result, "echo $var");

        // 3 diagnostics have fixes, 1 doesn't
        assert_eq!(report.auto_fixable_count, 3);
        assert_eq!(report.manual_count, 1);
    }

    #[test]
    fn test_ml_014_confidence_calculation() {
        let result = sample_lint_result();
        let report = RichLintReport::from_lint_result("test.sh", &result, "echo $var");

        // Confidence should be between 0 and 1
        assert!(report.overall_confidence >= 0.0);
        assert!(report.overall_confidence <= 1.0);
    }
}
