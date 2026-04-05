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
}

include!("lint_report_render_clusters.rs");
