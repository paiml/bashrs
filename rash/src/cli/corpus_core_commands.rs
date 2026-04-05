//! Core corpus command dispatchers.
//!
//! Routes corpus subcommands to their implementation modules.

use crate::cli::args::{CorpusAnalysisCommands, CorpusCommands, CorpusFormatArg, CorpusOutputFormat};
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
        // Convergence
        CorpusCommands::Converged {
            min_rate,
            max_delta,
            min_stable,
        } => super::corpus_ops_commands::corpus_converged(min_rate, max_delta, min_stable),
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
        // Gates (non-analysis)
        CorpusCommands::Gate { min_score, max_ms } => {
            super::corpus_gate_commands::corpus_gate(min_score, max_ms)
        }
        // All analysis commands (flattened from CorpusAnalysisCommands)
        CorpusCommands::Analysis(analysis) => handle_corpus_analysis_ops(analysis),
        // Handled in parent dispatchers
        _ => unreachable!(),
    }
}

/// Analysis, SSC, dataset, domain, and tier commands (from CorpusAnalysisCommands).
pub(crate) fn handle_corpus_analysis_ops(command: CorpusAnalysisCommands) -> Result<()> {
    match command {
        // Patterns and decisions
        CorpusAnalysisCommands::Suspicious { limit } => {
            super::corpus_metrics_commands::corpus_suspicious(limit)
        }
        CorpusAnalysisCommands::Decisions => super::corpus_decision_commands::corpus_decisions(),
        CorpusAnalysisCommands::Patterns => super::corpus_decision_commands::corpus_patterns(),
        CorpusAnalysisCommands::PatternQuery { signal } => {
            super::corpus_decision_commands::corpus_pattern_query(&signal)
        }
        CorpusAnalysisCommands::FixSuggest { id } => {
            super::corpus_decision_commands::corpus_fix_suggest(&id)
        }
        CorpusAnalysisCommands::Graph => super::corpus_advanced_commands::corpus_graph(),
        CorpusAnalysisCommands::Impact { limit } => {
            super::corpus_advanced_commands::corpus_impact(limit)
        }
        CorpusAnalysisCommands::BlastRadius { decision } => {
            super::corpus_advanced_commands::corpus_blast_radius(&decision)
        }
        CorpusAnalysisCommands::Dedup => super::corpus_advanced_commands::corpus_dedup(),
        CorpusAnalysisCommands::Triage => super::corpus_advanced_commands::corpus_triage(),
        CorpusAnalysisCommands::LabelRules => super::corpus_advanced_commands::corpus_label_rules(),
        CorpusAnalysisCommands::OrgPatterns => {
            super::corpus_convergence_commands::corpus_org_patterns()
        }
        // Convergence
        CorpusAnalysisCommands::ConvergeTable => {
            super::corpus_convergence_commands::corpus_converge_table()
        }
        CorpusAnalysisCommands::ConvergeDiff { from, to } => {
            super::corpus_convergence_commands::corpus_converge_diff(from, to)
        }
        CorpusAnalysisCommands::ConvergeStatus => {
            super::corpus_convergence_commands::corpus_converge_status()
        }
        CorpusAnalysisCommands::ConvergenceCheck => {
            super::corpus_pipeline_commands::corpus_convergence_check()
        }
        // Gates and quality checks
        CorpusAnalysisCommands::GateStatus => {
            super::corpus_config_commands::corpus_gate_status_cmd()
        }
        CorpusAnalysisCommands::QualityGates => {
            super::corpus_config_commands::corpus_quality_gates()
        }
        CorpusAnalysisCommands::MetricsCheck => {
            super::corpus_config_commands::corpus_metrics_check()
        }
        CorpusAnalysisCommands::RegressionCheck => {
            super::corpus_pipeline_commands::corpus_regression_check()
        }
        CorpusAnalysisCommands::LintPipeline => {
            super::corpus_pipeline_commands::corpus_lint_pipeline()
        }
        CorpusAnalysisCommands::PublishCheck => {
            super::corpus_config_commands::corpus_publish_check()
        }
        // Mining and fixes
        CorpusAnalysisCommands::Mine { limit } => {
            super::corpus_convergence_commands::corpus_mine(limit)
        }
        CorpusAnalysisCommands::FixGaps { limit } => {
            super::corpus_convergence_commands::corpus_fix_gaps(limit)
        }
        CorpusAnalysisCommands::DiagnoseB2 { filter, limit } => {
            super::corpus_b2_commands::corpus_diagnose_b2(filter.as_ref(), limit)
        }
        CorpusAnalysisCommands::FixB2 { apply } => {
            super::corpus_b2_commands::corpus_fix_b2(apply)
        }
        // Grammar and dataset
        CorpusAnalysisCommands::SchemaValidate => {
            super::corpus_convergence_commands::corpus_schema_validate()
        }
        CorpusAnalysisCommands::GrammarErrors => {
            super::corpus_convergence_commands::corpus_grammar_errors()
        }
        CorpusAnalysisCommands::FormatGrammar { format } => {
            super::corpus_convergence_commands::corpus_format_grammar(format)
        }
        CorpusAnalysisCommands::ExportDataset { format, output } => {
            super::corpus_config_commands::corpus_export_dataset(format, output)
        }
        CorpusAnalysisCommands::DatasetInfo => {
            super::corpus_config_commands::corpus_dataset_info()
        }
        CorpusAnalysisCommands::GenerateConversations {
            output,
            seed,
            limit,
            entrenar,
        } => super::corpus_config_commands::corpus_generate_conversations(
            output, seed, limit, entrenar,
        ),
        // SSC v11 baselines and validation
        CorpusAnalysisCommands::Baselines => super::corpus_config_commands::corpus_baselines(),
        CorpusAnalysisCommands::CweMapping { json } => {
            super::corpus_config_commands::corpus_cwe_mapping(json)
        }
        CorpusAnalysisCommands::ExportBenchmark { output, limit } => {
            super::corpus_config_commands::corpus_export_benchmark(output, limit)
        }
        CorpusAnalysisCommands::PipelineCheck { json } => {
            super::corpus_ssb_commands::corpus_pipeline_check(json)
        }
        CorpusAnalysisCommands::MergeData {
            output,
            input,
            seed,
        } => super::corpus_ssb_commands::corpus_merge_data(output, input, seed),
        CorpusAnalysisCommands::ShellcheckValidate {
            samples,
            seed,
            json,
        } => super::corpus_ssb_commands::corpus_shellcheck_validate(samples, seed, json),
        CorpusAnalysisCommands::EvalBenchmark { predictions, json } => {
            super::corpus_ssb_commands::corpus_eval_benchmark(predictions, json)
        }
        CorpusAnalysisCommands::Label {
            input,
            output,
            format: _,
        } => super::corpus_config_commands::corpus_label(input, output),
        CorpusAnalysisCommands::LabelAudit { limit } => {
            super::corpus_config_commands::corpus_label_audit(limit)
        }
        CorpusAnalysisCommands::GeneralizationTests => {
            super::corpus_config_commands::corpus_generalization_tests()
        }
        CorpusAnalysisCommands::TokenizerValidation => {
            super::corpus_config_commands::corpus_tokenizer_validation()
        }
        CorpusAnalysisCommands::ValidateContracts => {
            super::corpus_config_commands::corpus_validate_contracts()
        }
        CorpusAnalysisCommands::ExportSplits { output, input } => {
            super::corpus_config_commands::corpus_export_splits(output, input)
        }
        CorpusAnalysisCommands::SscReport { json, gate } => {
            super::corpus_config_commands::corpus_ssc_report(json, gate)
        }
        CorpusAnalysisCommands::ModelCard { output } => {
            super::corpus_config_commands::corpus_model_card(output)
        }
        CorpusAnalysisCommands::TrainingConfig { output, json } => {
            super::corpus_config_commands::corpus_training_config(output, json)
        }
        CorpusAnalysisCommands::PublishDataset { output } => {
            super::corpus_config_commands::corpus_publish_dataset(output)
        }
        CorpusAnalysisCommands::PublishBenchmark {
            input,
            output,
            version,
        } => super::corpus_expansion_commands::corpus_publish_benchmark(input, output, version),
        CorpusAnalysisCommands::GenerateExpansion {
            format,
            count,
            output,
            seed,
        } => {
            super::corpus_expansion_commands::corpus_generate_expansion(format, count, output, seed)
        }
        CorpusAnalysisCommands::PublishConversations { output, seed } => {
            super::corpus_config_commands::corpus_publish_conversations(output, seed)
        }
        CorpusAnalysisCommands::ConvertSsb {
            input,
            output,
            limit,
        } => super::corpus_ssb_commands::corpus_convert_ssb(input, output, limit),
        CorpusAnalysisCommands::ExtractEmbeddings {
            model,
            output,
            limit,
            input_jsonl,
        } => {
            super::corpus_ml_commands::corpus_extract_embeddings(model, output, limit, input_jsonl)
        }
        CorpusAnalysisCommands::TrainClassifier {
            embeddings,
            output,
            epochs,
            learning_rate,
            seed,
            max_entries,
            augment,
            mlp,
            mlp_hidden,
        } => super::corpus_ml_commands::corpus_train_classifier(
            embeddings,
            output,
            epochs,
            learning_rate,
            seed,
            max_entries,
            augment,
            mlp,
            mlp_hidden,
        ),
        CorpusAnalysisCommands::RunClassifier {
            model,
            output,
            epochs,
            learning_rate,
            seed,
        } => super::corpus_ml_commands::corpus_run_classifier(
            model,
            output,
            epochs,
            learning_rate,
            seed,
        ),
        // Domain analysis
        CorpusAnalysisCommands::DomainCategories => {
            super::corpus_config_commands::corpus_domain_categories()
        }
        CorpusAnalysisCommands::DomainCoverage => {
            super::corpus_config_commands::corpus_domain_coverage()
        }
        CorpusAnalysisCommands::DomainMatrix => {
            super::corpus_config_commands::corpus_domain_matrix()
        }
        // Tier configuration
        CorpusAnalysisCommands::TierWeights => {
            super::corpus_config_commands::corpus_tier_weights()
        }
        CorpusAnalysisCommands::TierAnalysis => {
            super::corpus_config_commands::corpus_tier_analysis()
        }
        CorpusAnalysisCommands::TierTargets => {
            super::corpus_config_commands::corpus_tier_targets()
        }
        CorpusAnalysisCommands::Version => super::corpus_metrics_commands::corpus_version(),
        CorpusAnalysisCommands::BatchEval {
            model,
            test_data,
            output,
            max_tokens,
        } => super::corpus_ssb_commands::corpus_batch_eval(model, test_data, output, max_tokens),
    }
}
