//! Core corpus command dispatchers.
//!
//! Routes corpus subcommands to their implementation modules.

use crate::cli::args::{
    CorpusAnalysisCommands, CorpusCommands, CorpusDiagnosticsCommands, CorpusFormatArg,
    CorpusOperationsCommands, CorpusOutputFormat, CorpusScoringCommands,
};
use crate::models::{Config, Error, Result};

pub(crate) fn handle_corpus_command(command: CorpusCommands) -> Result<()> {
    match command {
        CorpusCommands::Scoring(scoring) => handle_corpus_scoring(scoring),
        CorpusCommands::Operations(operations) => handle_corpus_operations(operations),
        CorpusCommands::Diagnostics(diagnostics) => handle_corpus_diagnostics(diagnostics),
        // All analysis commands (flattened from CorpusAnalysisCommands)
        CorpusCommands::Analysis(analysis) => handle_corpus_analysis_ops(analysis),
        CorpusCommands::Version => super::corpus_metrics_commands::corpus_version(),
    }
}

pub(crate) fn handle_corpus_run(
    format: CorpusOutputFormat,
    filter: Option<CorpusFormatArg>,
    min_score: Option<f64>,
    log: bool,
) -> Result<()> {
    use crate::corpus::registry::{CorpusFormat, CorpusRegistry};
    use crate::corpus::runner::CorpusRunner;

    let config = Config::default();
    let registry = CorpusRegistry::load_full();
    let runner = CorpusRunner::new(config);

    let score = match filter {
        Some(CorpusFormatArg::Bash) => runner.run_format(&registry, CorpusFormat::Bash),
        Some(CorpusFormatArg::Makefile) => runner.run_format(&registry, CorpusFormat::Makefile),
        Some(CorpusFormatArg::Dockerfile) => runner.run_format(&registry, CorpusFormat::Dockerfile),
        None => runner.run(&registry),
    };

    super::corpus_score_print_commands::corpus_print_score(&score, &format)?;
    super::corpus_score_print_commands::corpus_save_last_run(&score);

    if log {
        super::corpus_score_print_commands::corpus_write_convergence_log(&runner, &score)?;
    }

    if let Some(threshold) = min_score {
        if score.score < threshold {
            return Err(Error::Validation(format!(
                "Score {:.1} is below minimum threshold {:.1}",
                score.score, threshold
            )));
        }
    }

    Ok(())
}

/// Scoring, reporting, and failure-analysis commands (from CorpusScoringCommands).
pub(crate) fn handle_corpus_scoring(command: CorpusScoringCommands) -> Result<()> {
    match command {
        // Core commands: run, show, check
        CorpusScoringCommands::Run {
            format,
            filter,
            min_score,
            log,
        } => handle_corpus_run(format, filter, min_score, log),
        CorpusScoringCommands::Show { id, format } => {
            super::corpus_report_commands::corpus_show_entry(&id, &format)
        }
        // Reports and history
        CorpusScoringCommands::History { format, last } => {
            super::corpus_report_commands::corpus_show_history(&format, last)
        }
        CorpusScoringCommands::Failures {
            format,
            filter,
            dimension,
        } => super::corpus_report_commands::corpus_show_failures(
            &format,
            filter.as_ref(),
            dimension.as_deref(),
        ),
        CorpusScoringCommands::Report { output } => {
            super::corpus_diff_commands::corpus_generate_report(output.as_deref())
        }
        CorpusScoringCommands::Diff { format, from, to } => {
            super::corpus_diff_commands::corpus_show_diff(&format, from, to)
        }
        CorpusScoringCommands::Export { output, filter } => {
            super::corpus_report_commands::corpus_export(output.as_deref(), filter.as_ref())
        }
        CorpusScoringCommands::Stats { format } => {
            super::corpus_score_print_commands::corpus_show_stats(&format)
        }
        CorpusScoringCommands::Check { id, format } => {
            super::corpus_entry_commands::corpus_check_entry(&id, &format)
        }
        CorpusScoringCommands::Difficulty { id, format } => {
            super::corpus_entry_commands::corpus_classify_difficulty(&id, &format)
        }
        CorpusScoringCommands::Summary => super::corpus_analysis_commands::corpus_summary(),
        // Coverage and quality
        CorpusScoringCommands::Growth { format } => {
            super::corpus_analysis_commands::corpus_growth(&format)
        }
        CorpusScoringCommands::Coverage { format } => {
            super::corpus_analysis_commands::corpus_coverage(&format)
        }
        CorpusScoringCommands::Validate { format } => {
            super::corpus_analysis_commands::corpus_validate(&format)
        }
        // Risk and failure analysis
        CorpusScoringCommands::Pareto {
            format,
            filter,
            top,
        } => super::corpus_failure_commands::corpus_pareto_analysis(&format, filter.as_ref(), top),
        CorpusScoringCommands::Risk { format, level } => {
            super::corpus_entry_commands::corpus_risk_analysis(&format, level.as_deref())
        }
        CorpusScoringCommands::WhyFailed { id, format } => {
            super::corpus_failure_commands::corpus_why_failed(&id, &format)
        }
        CorpusScoringCommands::Regressions { format } => {
            super::corpus_failure_commands::corpus_regressions(&format)
        }
        CorpusScoringCommands::Heatmap { limit, filter } => {
            super::corpus_display_commands::corpus_heatmap(limit, filter.as_ref())
        }
        CorpusScoringCommands::Dashboard => super::corpus_display_commands::corpus_dashboard(),
        // Search, filter, ranking
        CorpusScoringCommands::Search {
            pattern,
            format,
            filter,
        } => super::corpus_display_commands::corpus_search(&pattern, &format, filter.as_ref()),
        CorpusScoringCommands::Sparkline => super::corpus_ranking_commands::corpus_sparkline(),
        CorpusScoringCommands::Top {
            limit,
            worst,
            filter,
        } => super::corpus_ranking_commands::corpus_top(limit, worst, filter.as_ref()),
        CorpusScoringCommands::Categories { format } => {
            super::corpus_ranking_commands::corpus_categories(&format)
        }
    }
}

/// Operations, benchmark, comparison, and weighting commands (from CorpusOperationsCommands).
pub(crate) fn handle_corpus_operations(command: CorpusOperationsCommands) -> Result<()> {
    match command {
        // Categorization and structure
        CorpusOperationsCommands::Dimensions { format, filter } => {
            super::corpus_ranking_commands::corpus_dimensions(&format, filter.as_ref())
        }
        CorpusOperationsCommands::Dupes => super::corpus_ops_commands::corpus_dupes(),
        // Convergence
        CorpusOperationsCommands::Converged {
            min_rate,
            max_delta,
            min_stable,
        } => super::corpus_ops_commands::corpus_converged(min_rate, max_delta, min_stable),
        // Performance
        CorpusOperationsCommands::Benchmark { max_ms, filter } => {
            super::corpus_ops_commands::corpus_benchmark(max_ms, filter.as_ref())
        }
        CorpusOperationsCommands::Errors { format, filter } => {
            super::corpus_gate_commands::corpus_errors(&format, filter.as_ref())
        }
        CorpusOperationsCommands::Sample { count, filter } => {
            super::corpus_gate_commands::corpus_sample(count, filter.as_ref())
        }
        CorpusOperationsCommands::Completeness => {
            super::corpus_gate_commands::corpus_completeness()
        }
        // Gates (non-analysis)
        CorpusOperationsCommands::Gate { min_score, max_ms } => {
            super::corpus_gate_commands::corpus_gate(min_score, max_ms)
        }
        CorpusOperationsCommands::Outliers { threshold, filter } => {
            super::corpus_gate_commands::corpus_outliers(threshold, filter.as_ref())
        }
        CorpusOperationsCommands::Matrix => super::corpus_gate_commands::corpus_matrix(),
        // Stability and drift
        CorpusOperationsCommands::Timeline => super::corpus_time_commands::corpus_timeline(),
        CorpusOperationsCommands::Drift => super::corpus_time_commands::corpus_drift(),
        CorpusOperationsCommands::Slow { limit, filter } => {
            super::corpus_time_commands::corpus_slow(limit, filter.as_ref())
        }
        CorpusOperationsCommands::Tags => super::corpus_time_commands::corpus_tags(),
        // Comparison and diagnostics
        CorpusOperationsCommands::Health => super::corpus_compare_commands::corpus_health(),
        CorpusOperationsCommands::Compare { id1, id2 } => {
            super::corpus_compare_commands::corpus_compare(&id1, &id2)
        }
        CorpusOperationsCommands::Density => super::corpus_compare_commands::corpus_density(),
        CorpusOperationsCommands::Perf { filter } => {
            super::corpus_compare_commands::corpus_perf(filter.as_ref())
        }
        CorpusOperationsCommands::Citl { filter } => {
            super::corpus_compare_commands::corpus_citl(filter.as_ref())
        }
        CorpusOperationsCommands::Streak => super::corpus_compare_commands::corpus_streak(),
        // Weights, formats, scoring
        CorpusOperationsCommands::Weight => super::corpus_weight_commands::corpus_weight(),
        CorpusOperationsCommands::Format { format } => {
            super::corpus_weight_commands::corpus_format_report(&format)
        }
        CorpusOperationsCommands::Budget => super::corpus_weight_commands::corpus_budget(),
        CorpusOperationsCommands::Entropy => super::corpus_weight_commands::corpus_entropy(),
    }
}

/// Visualization, diagnostics, and tier-breakdown commands (from CorpusDiagnosticsCommands).
pub(crate) fn handle_corpus_diagnostics(command: CorpusDiagnosticsCommands) -> Result<()> {
    match command {
        CorpusDiagnosticsCommands::Todo => super::corpus_weight_commands::corpus_todo(),
        CorpusDiagnosticsCommands::Scatter => super::corpus_viz_commands::corpus_scatter(),
        CorpusDiagnosticsCommands::GradeDist => super::corpus_viz_commands::corpus_grade_dist(),
        CorpusDiagnosticsCommands::Pivot => super::corpus_viz_commands::corpus_pivot(),
        CorpusDiagnosticsCommands::Corr => super::corpus_viz_commands::corpus_corr(),
        CorpusDiagnosticsCommands::Schema => super::corpus_viz_commands::corpus_schema(),
        CorpusDiagnosticsCommands::HistoryChart => {
            super::corpus_viz_commands::corpus_history_chart()
        }
        CorpusDiagnosticsCommands::Flaky { threshold } => {
            super::corpus_diag_commands::corpus_flaky(threshold)
        }
        CorpusDiagnosticsCommands::Profile => super::corpus_diag_commands::corpus_profile(),
        CorpusDiagnosticsCommands::Gaps => super::corpus_diag_commands::corpus_gaps(),
        CorpusDiagnosticsCommands::SummaryJson => {
            super::corpus_diag_commands::corpus_summary_json()
        }
        CorpusDiagnosticsCommands::Audit => super::corpus_diag_commands::corpus_audit(),
        CorpusDiagnosticsCommands::TierDetail => super::corpus_tier_commands::corpus_tier_detail(),
        CorpusDiagnosticsCommands::IdRange => super::corpus_tier_commands::corpus_id_range(),
        CorpusDiagnosticsCommands::Tiers => super::corpus_tier_commands::corpus_tiers(),
        CorpusDiagnosticsCommands::FailMap => super::corpus_tier_commands::corpus_fail_map(),
        CorpusDiagnosticsCommands::ScoreRange => super::corpus_tier_commands::corpus_score_range(),
        CorpusDiagnosticsCommands::Topk { limit } => {
            super::corpus_metrics_commands::corpus_topk(limit)
        }
        CorpusDiagnosticsCommands::FormatCmp => super::corpus_metrics_commands::corpus_format_cmp(),
        CorpusDiagnosticsCommands::Stability => super::corpus_metrics_commands::corpus_stability(),
        CorpusDiagnosticsCommands::Rate => super::corpus_metrics_commands::corpus_rate(),
        CorpusDiagnosticsCommands::Dist => super::corpus_metrics_commands::corpus_dist(),
        CorpusDiagnosticsCommands::Trace { id } => {
            super::corpus_metrics_commands::corpus_trace(&id)
        }
    }
}

/// Analysis, SSC, dataset, domain, and tier commands (from CorpusAnalysisCommands).
///
/// GH-215: the 63 analysis variants now live in three `#[command(flatten)]` groups,
/// each dispatched by its own handler in `corpus_analysis_dispatch`.
pub(crate) fn handle_corpus_analysis_ops(command: CorpusAnalysisCommands) -> Result<()> {
    match command {
        CorpusAnalysisCommands::Diag(cmd) => {
            super::corpus_analysis_dispatch::handle_corpus_analysis_diag_ops(cmd)
        }
        CorpusAnalysisCommands::Gates(cmd) => {
            super::corpus_analysis_dispatch::handle_corpus_analysis_gates_ops(cmd)
        }
        CorpusAnalysisCommands::Pipeline(cmd) => {
            super::corpus_analysis_dispatch::handle_corpus_analysis_pipeline_ops(cmd)
        }
    }
}
