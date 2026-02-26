//! Core corpus command dispatchers.
//!
//! Routes corpus subcommands to their implementation modules.

use crate::cli::args::{CorpusCommands, CorpusFormatArg, CorpusOutputFormat};
use crate::models::{Config, Error, Result};

pub(crate) fn handle_corpus_command(command: CorpusCommands) -> Result<()> {
    match command {
        // Core commands: run, show, check
        CorpusCommands::Run {
            format,
            filter,
            min_score,
            log,
        } => handle_corpus_run(format, filter, min_score, log),
        CorpusCommands::Show { id, format } => {
            super::corpus_report_commands::corpus_show_entry(&id, &format)
        }
        CorpusCommands::Check { id, format } => {
            super::corpus_entry_commands::corpus_check_entry(&id, &format)
        }
        CorpusCommands::Validate { format } => {
            super::corpus_analysis_commands::corpus_validate(&format)
        }
        // Reporting and analysis
        _ => handle_corpus_analysis(command),
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

pub(crate) fn handle_corpus_analysis(command: CorpusCommands) -> Result<()> {
    match command {
        // Reports and history
        CorpusCommands::History { format, last } => {
            super::corpus_report_commands::corpus_show_history(&format, last)
        }
        CorpusCommands::Report { output } => {
            super::corpus_diff_commands::corpus_generate_report(output.as_deref())
        }
        CorpusCommands::Diff { format, from, to } => {
            super::corpus_diff_commands::corpus_show_diff(&format, from, to)
        }
        CorpusCommands::Export { output, filter } => {
            super::corpus_report_commands::corpus_export(output.as_deref(), filter.as_ref())
        }
        CorpusCommands::Stats { format } => {
            super::corpus_score_print_commands::corpus_show_stats(&format)
        }
        CorpusCommands::Summary => super::corpus_analysis_commands::corpus_summary(),
        CorpusCommands::SummaryJson => super::corpus_diag_commands::corpus_summary_json(),
        CorpusCommands::Dashboard => super::corpus_display_commands::corpus_dashboard(),
        CorpusCommands::Sparkline => super::corpus_ranking_commands::corpus_sparkline(),
        CorpusCommands::HistoryChart => super::corpus_viz_commands::corpus_history_chart(),
        // Failure analysis
        CorpusCommands::Failures {
            format,
            filter,
            dimension,
        } => super::corpus_report_commands::corpus_show_failures(
            &format,
            filter.as_ref(),
            dimension.as_deref(),
        ),
        CorpusCommands::WhyFailed { id, format } => {
            super::corpus_failure_commands::corpus_why_failed(&id, &format)
        }
        CorpusCommands::Regressions { format } => {
            super::corpus_failure_commands::corpus_regressions(&format)
        }
        CorpusCommands::FailMap => super::corpus_tier_commands::corpus_fail_map(),
        CorpusCommands::Errors { format, filter } => {
            super::corpus_gate_commands::corpus_errors(&format, filter.as_ref())
        }
        CorpusCommands::Flaky { threshold } => super::corpus_diag_commands::corpus_flaky(threshold),
        CorpusCommands::Suspicious { limit } => {
            super::corpus_metrics_commands::corpus_suspicious(limit)
        }
        // Metrics and scoring
        _ => handle_corpus_metrics(command),
    }
}

pub(crate) fn handle_corpus_metrics(command: CorpusCommands) -> Result<()> {
    match command {
        // Coverage and quality
        CorpusCommands::Growth { format } => {
            super::corpus_analysis_commands::corpus_growth(&format)
        }
        CorpusCommands::Coverage { format } => {
            super::corpus_analysis_commands::corpus_coverage(&format)
        }
        CorpusCommands::Difficulty { id, format } => {
            super::corpus_entry_commands::corpus_classify_difficulty(&id, &format)
        }
        CorpusCommands::Completeness => super::corpus_gate_commands::corpus_completeness(),
        CorpusCommands::Gaps => super::corpus_diag_commands::corpus_gaps(),
        CorpusCommands::Density => super::corpus_compare_commands::corpus_density(),
        CorpusCommands::Entropy => super::corpus_weight_commands::corpus_entropy(),
        // Search, filter, ranking
        CorpusCommands::Search {
            pattern,
            format,
            filter,
        } => super::corpus_display_commands::corpus_search(&pattern, &format, filter.as_ref()),
        CorpusCommands::Top {
            limit,
            worst,
            filter,
        } => super::corpus_ranking_commands::corpus_top(limit, worst, filter.as_ref()),
        CorpusCommands::Topk { limit } => super::corpus_metrics_commands::corpus_topk(limit),
        CorpusCommands::Sample { count, filter } => {
            super::corpus_gate_commands::corpus_sample(count, filter.as_ref())
        }
        CorpusCommands::Outliers { threshold, filter } => {
            super::corpus_gate_commands::corpus_outliers(threshold, filter.as_ref())
        }
        // Categorization and structure
        CorpusCommands::Categories { format } => {
            super::corpus_ranking_commands::corpus_categories(&format)
        }
        CorpusCommands::Dimensions { format, filter } => {
            super::corpus_ranking_commands::corpus_dimensions(&format, filter.as_ref())
        }
        CorpusCommands::Tiers => super::corpus_tier_commands::corpus_tiers(),
        CorpusCommands::TierDetail => super::corpus_tier_commands::corpus_tier_detail(),
        CorpusCommands::Tags => super::corpus_time_commands::corpus_tags(),
        CorpusCommands::IdRange => super::corpus_tier_commands::corpus_id_range(),
        // Performance and benchmarks
        _ => handle_corpus_ops(command),
    }
}

pub(crate) fn handle_corpus_ops(command: CorpusCommands) -> Result<()> {
    match command {
        // Risk and analysis
        CorpusCommands::Pareto {
            format,
            filter,
            top,
        } => super::corpus_failure_commands::corpus_pareto_analysis(&format, filter.as_ref(), top),
        CorpusCommands::Risk { format, level } => {
            super::corpus_entry_commands::corpus_risk_analysis(&format, level.as_deref())
        }
        CorpusCommands::Heatmap { limit, filter } => {
            super::corpus_display_commands::corpus_heatmap(limit, filter.as_ref())
        }
        CorpusCommands::Dupes => super::corpus_ops_commands::corpus_dupes(),
        CorpusCommands::Dedup => super::corpus_advanced_commands::corpus_dedup(),
        // Convergence
        CorpusCommands::Converged {
            min_rate,
            max_delta,
            min_stable,
        } => super::corpus_ops_commands::corpus_converged(min_rate, max_delta, min_stable),
        CorpusCommands::ConvergeTable => {
            super::corpus_convergence_commands::corpus_converge_table()
        }
        CorpusCommands::ConvergeDiff { from, to } => {
            super::corpus_convergence_commands::corpus_converge_diff(from, to)
        }
        CorpusCommands::ConvergeStatus => {
            super::corpus_convergence_commands::corpus_converge_status()
        }
        CorpusCommands::ConvergenceCheck => {
            super::corpus_pipeline_commands::corpus_convergence_check()
        }
        // Performance
        CorpusCommands::Benchmark { max_ms, filter } => {
            super::corpus_ops_commands::corpus_benchmark(max_ms, filter.as_ref())
        }
        CorpusCommands::Slow { limit, filter } => {
            super::corpus_time_commands::corpus_slow(limit, filter.as_ref())
        }
        CorpusCommands::Perf { filter } => {
            super::corpus_compare_commands::corpus_perf(filter.as_ref())
        }
        CorpusCommands::Profile => super::corpus_diag_commands::corpus_profile(),
        // Stability and drift
        CorpusCommands::Drift => super::corpus_time_commands::corpus_drift(),
        CorpusCommands::Stability => super::corpus_metrics_commands::corpus_stability(),
        CorpusCommands::Timeline => super::corpus_time_commands::corpus_timeline(),
        CorpusCommands::Matrix => super::corpus_gate_commands::corpus_matrix(),
        CorpusCommands::Streak => super::corpus_compare_commands::corpus_streak(),
        // Comparison and diagnostics
        CorpusCommands::Compare { id1, id2 } => {
            super::corpus_compare_commands::corpus_compare(&id1, &id2)
        }
        CorpusCommands::Trace { id } => super::corpus_metrics_commands::corpus_trace(&id),
        CorpusCommands::Health => super::corpus_compare_commands::corpus_health(),
        CorpusCommands::Citl { filter } => {
            super::corpus_compare_commands::corpus_citl(filter.as_ref())
        }
        // Weights, formats, scoring
        CorpusCommands::Weight => super::corpus_weight_commands::corpus_weight(),
        CorpusCommands::Format { format } => {
            super::corpus_weight_commands::corpus_format_report(&format)
        }
        CorpusCommands::FormatCmp => super::corpus_metrics_commands::corpus_format_cmp(),
        CorpusCommands::Budget => super::corpus_weight_commands::corpus_budget(),
        CorpusCommands::ScoreRange => super::corpus_tier_commands::corpus_score_range(),
        CorpusCommands::Rate => super::corpus_metrics_commands::corpus_rate(),
        CorpusCommands::Dist => super::corpus_metrics_commands::corpus_dist(),
        // Remaining ops
        _ => handle_corpus_quality_ops(command),
    }
}

pub(crate) fn handle_corpus_quality_ops(command: CorpusCommands) -> Result<()> {
    match command {
        CorpusCommands::GradeDist => super::corpus_viz_commands::corpus_grade_dist(),
        CorpusCommands::Scatter => super::corpus_viz_commands::corpus_scatter(),
        CorpusCommands::Pivot => super::corpus_viz_commands::corpus_pivot(),
        CorpusCommands::Corr => super::corpus_viz_commands::corpus_corr(),
        CorpusCommands::Schema => super::corpus_viz_commands::corpus_schema(),
        CorpusCommands::Todo => super::corpus_weight_commands::corpus_todo(),
        CorpusCommands::Audit => super::corpus_diag_commands::corpus_audit(),
        // Patterns and decisions
        CorpusCommands::Decisions => super::corpus_decision_commands::corpus_decisions(),
        CorpusCommands::Patterns => super::corpus_decision_commands::corpus_patterns(),
        CorpusCommands::PatternQuery { signal } => {
            super::corpus_decision_commands::corpus_pattern_query(&signal)
        }
        CorpusCommands::FixSuggest { id } => {
            super::corpus_decision_commands::corpus_fix_suggest(&id)
        }
        CorpusCommands::Graph => super::corpus_advanced_commands::corpus_graph(),
        CorpusCommands::Impact { limit } => super::corpus_advanced_commands::corpus_impact(limit),
        CorpusCommands::BlastRadius { decision } => {
            super::corpus_advanced_commands::corpus_blast_radius(&decision)
        }
        CorpusCommands::Triage => super::corpus_advanced_commands::corpus_triage(),
        CorpusCommands::LabelRules => super::corpus_advanced_commands::corpus_label_rules(),
        CorpusCommands::OrgPatterns => super::corpus_convergence_commands::corpus_org_patterns(),
        // Gates and quality checks
        CorpusCommands::Gate { min_score, max_ms } => {
            super::corpus_gate_commands::corpus_gate(min_score, max_ms)
        }
        CorpusCommands::GateStatus => super::corpus_config_commands::corpus_gate_status_cmd(),
        CorpusCommands::QualityGates => super::corpus_config_commands::corpus_quality_gates(),
        CorpusCommands::MetricsCheck => super::corpus_config_commands::corpus_metrics_check(),
        CorpusCommands::RegressionCheck => {
            super::corpus_pipeline_commands::corpus_regression_check()
        }
        CorpusCommands::LintPipeline => super::corpus_pipeline_commands::corpus_lint_pipeline(),
        CorpusCommands::PublishCheck => super::corpus_config_commands::corpus_publish_check(),
        // Mining and fixes
        CorpusCommands::Mine { limit } => super::corpus_convergence_commands::corpus_mine(limit),
        CorpusCommands::FixGaps { limit } => {
            super::corpus_convergence_commands::corpus_fix_gaps(limit)
        }
        CorpusCommands::DiagnoseB2 { filter, limit } => {
            super::corpus_b2_commands::corpus_diagnose_b2(filter.as_ref(), limit)
        }
        CorpusCommands::FixB2 { apply } => super::corpus_b2_commands::corpus_fix_b2(apply),
        // Grammar and dataset
        CorpusCommands::SchemaValidate => {
            super::corpus_convergence_commands::corpus_schema_validate()
        }
        CorpusCommands::GrammarErrors => {
            super::corpus_convergence_commands::corpus_grammar_errors()
        }
        CorpusCommands::FormatGrammar { format } => {
            super::corpus_convergence_commands::corpus_format_grammar(format)
        }
        CorpusCommands::ExportDataset { format, output } => {
            super::corpus_config_commands::corpus_export_dataset(format, output)
        }
        CorpusCommands::DatasetInfo => super::corpus_config_commands::corpus_dataset_info(),
        // Domain analysis
        CorpusCommands::DomainCategories => {
            super::corpus_config_commands::corpus_domain_categories()
        }
        CorpusCommands::DomainCoverage => super::corpus_config_commands::corpus_domain_coverage(),
        CorpusCommands::DomainMatrix => super::corpus_config_commands::corpus_domain_matrix(),
        // Tier configuration
        CorpusCommands::TierWeights => super::corpus_config_commands::corpus_tier_weights(),
        CorpusCommands::TierAnalysis => super::corpus_config_commands::corpus_tier_analysis(),
        CorpusCommands::TierTargets => super::corpus_config_commands::corpus_tier_targets(),
        CorpusCommands::Version => super::corpus_metrics_commands::corpus_version(),
        // Handled in parent dispatchers
        _ => unreachable!(),
    }
}
