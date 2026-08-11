//! Dispatch for the three `CorpusAnalysisCommands` flatten groups.
//!
//! GH-215 split the 63-variant `CorpusAnalysisCommands` enum into three
//! `#[command(flatten)]` groups to bound the stack frame `clap_derive` generates
//! for `augment_subcommands`. The match arms below are the original arms of
//! `corpus_core_commands::handle_corpus_analysis_ops` verbatim, partitioned by
//! group; only the enum path on each pattern changed (plus the rustfmt reflow
//! that the longer type names force on a few one-line arms).

use crate::cli::args_corpus_analysis::{
    CorpusAnalysisDiagCommands, CorpusAnalysisGatesCommands, CorpusAnalysisPipelineCommands,
};
use crate::models::Result;

/// Group 1 of `CorpusAnalysisCommands` (original variants 1-21).
pub(super) fn handle_corpus_analysis_diag_ops(command: CorpusAnalysisDiagCommands) -> Result<()> {
    match command {
        // Patterns and decisions
        CorpusAnalysisDiagCommands::Suspicious { limit } => {
            super::corpus_metrics_commands::corpus_suspicious(limit)
        }
        CorpusAnalysisDiagCommands::Decisions => {
            super::corpus_decision_commands::corpus_decisions()
        }
        CorpusAnalysisDiagCommands::Patterns => super::corpus_decision_commands::corpus_patterns(),
        CorpusAnalysisDiagCommands::PatternQuery { signal } => {
            super::corpus_decision_commands::corpus_pattern_query(&signal)
        }
        CorpusAnalysisDiagCommands::FixSuggest { id } => {
            super::corpus_decision_commands::corpus_fix_suggest(&id)
        }
        CorpusAnalysisDiagCommands::Graph => super::corpus_advanced_commands::corpus_graph(),
        CorpusAnalysisDiagCommands::Impact { limit } => {
            super::corpus_advanced_commands::corpus_impact(limit)
        }
        CorpusAnalysisDiagCommands::BlastRadius { decision } => {
            super::corpus_advanced_commands::corpus_blast_radius(&decision)
        }
        CorpusAnalysisDiagCommands::Dedup => super::corpus_advanced_commands::corpus_dedup(),
        CorpusAnalysisDiagCommands::Triage => super::corpus_advanced_commands::corpus_triage(),
        CorpusAnalysisDiagCommands::LabelRules => {
            super::corpus_advanced_commands::corpus_label_rules()
        }
        CorpusAnalysisDiagCommands::OrgPatterns => {
            super::corpus_convergence_commands::corpus_org_patterns()
        }
        CorpusAnalysisDiagCommands::ConvergeTable => {
            super::corpus_convergence_commands::corpus_converge_table()
        }
        CorpusAnalysisDiagCommands::ConvergeDiff { from, to } => {
            super::corpus_convergence_commands::corpus_converge_diff(from, to)
        }
        CorpusAnalysisDiagCommands::ConvergeStatus => {
            super::corpus_convergence_commands::corpus_converge_status()
        }
        // Mining and fixes
        CorpusAnalysisDiagCommands::Mine { limit } => {
            super::corpus_convergence_commands::corpus_mine(limit)
        }
        CorpusAnalysisDiagCommands::FixGaps { limit } => {
            super::corpus_convergence_commands::corpus_fix_gaps(limit)
        }
        // Grammar and dataset
        CorpusAnalysisDiagCommands::SchemaValidate => {
            super::corpus_convergence_commands::corpus_schema_validate()
        }
        CorpusAnalysisDiagCommands::GrammarErrors => {
            super::corpus_convergence_commands::corpus_grammar_errors()
        }
        CorpusAnalysisDiagCommands::FormatGrammar { format } => {
            super::corpus_convergence_commands::corpus_format_grammar(format)
        }
        CorpusAnalysisDiagCommands::ExportDataset { format, output } => {
            super::corpus_config_commands::corpus_export_dataset(format, output)
        }
    }
}

/// Group 2 of `CorpusAnalysisCommands` (original variants 22-42).
pub(super) fn handle_corpus_analysis_gates_ops(command: CorpusAnalysisGatesCommands) -> Result<()> {
    match command {
        CorpusAnalysisGatesCommands::ConvergenceCheck => {
            super::corpus_pipeline_commands::corpus_convergence_check()
        }
        // Gates and quality checks
        CorpusAnalysisGatesCommands::GateStatus => {
            super::corpus_config_commands::corpus_gate_status_cmd()
        }
        CorpusAnalysisGatesCommands::QualityGates => {
            super::corpus_config_commands::corpus_quality_gates()
        }
        CorpusAnalysisGatesCommands::MetricsCheck => {
            super::corpus_config_commands::corpus_metrics_check()
        }
        CorpusAnalysisGatesCommands::RegressionCheck => {
            super::corpus_pipeline_commands::corpus_regression_check()
        }
        CorpusAnalysisGatesCommands::LintPipeline => {
            super::corpus_pipeline_commands::corpus_lint_pipeline()
        }
        CorpusAnalysisGatesCommands::PublishCheck => {
            super::corpus_config_commands::corpus_publish_check()
        }
        CorpusAnalysisGatesCommands::DiagnoseB2 { filter, limit } => {
            super::corpus_b2_commands::corpus_diagnose_b2(filter.as_ref(), limit)
        }
        CorpusAnalysisGatesCommands::FixB2 { apply } => {
            super::corpus_b2_commands::corpus_fix_b2(apply)
        }
        CorpusAnalysisGatesCommands::DatasetInfo => {
            super::corpus_config_commands::corpus_dataset_info()
        }
        CorpusAnalysisGatesCommands::GenerateConversations {
            output,
            seed,
            limit,
            entrenar,
        } => super::corpus_config_commands::corpus_generate_conversations(
            output, seed, limit, entrenar,
        ),
        // SSC v11 baselines and validation
        CorpusAnalysisGatesCommands::Baselines => super::corpus_config_commands::corpus_baselines(),
        CorpusAnalysisGatesCommands::CweMapping { json } => {
            super::corpus_config_commands::corpus_cwe_mapping(json)
        }
        CorpusAnalysisGatesCommands::ExportBenchmark { output, limit } => {
            super::corpus_config_commands::corpus_export_benchmark(output, limit)
        }
        CorpusAnalysisGatesCommands::PipelineCheck { json } => {
            super::corpus_ssb_commands::corpus_pipeline_check(json)
        }
        // Domain analysis
        CorpusAnalysisGatesCommands::DomainCategories => {
            super::corpus_config_commands::corpus_domain_categories()
        }
        CorpusAnalysisGatesCommands::DomainCoverage => {
            super::corpus_config_commands::corpus_domain_coverage()
        }
        CorpusAnalysisGatesCommands::DomainMatrix => {
            super::corpus_config_commands::corpus_domain_matrix()
        }
        // Tier configuration
        CorpusAnalysisGatesCommands::TierWeights => {
            super::corpus_config_commands::corpus_tier_weights()
        }
        CorpusAnalysisGatesCommands::TierAnalysis => {
            super::corpus_config_commands::corpus_tier_analysis()
        }
        CorpusAnalysisGatesCommands::TierTargets => {
            super::corpus_config_commands::corpus_tier_targets()
        }
    }
}

/// Group 3 of `CorpusAnalysisCommands` (original variants 43-63).
pub(super) fn handle_corpus_analysis_pipeline_ops(
    command: CorpusAnalysisPipelineCommands,
) -> Result<()> {
    match command {
        CorpusAnalysisPipelineCommands::MergeData {
            output,
            input,
            seed,
        } => super::corpus_ssb_commands::corpus_merge_data(output, input, seed),
        CorpusAnalysisPipelineCommands::ShellcheckValidate {
            samples,
            seed,
            json,
        } => super::corpus_ssb_commands::corpus_shellcheck_validate(samples, seed, json),
        CorpusAnalysisPipelineCommands::EvalBenchmark { predictions, json } => {
            super::corpus_ssb_commands::corpus_eval_benchmark(predictions, json)
        }
        CorpusAnalysisPipelineCommands::Label {
            input,
            output,
            format: _,
        } => super::corpus_config_commands::corpus_label(input, output),
        CorpusAnalysisPipelineCommands::LabelAudit { limit } => {
            super::corpus_config_commands::corpus_label_audit(limit)
        }
        CorpusAnalysisPipelineCommands::GeneralizationTests => {
            super::corpus_config_commands::corpus_generalization_tests()
        }
        CorpusAnalysisPipelineCommands::TokenizerValidation => {
            super::corpus_config_commands::corpus_tokenizer_validation()
        }
        CorpusAnalysisPipelineCommands::ValidateContracts => {
            super::corpus_config_commands::corpus_validate_contracts()
        }
        CorpusAnalysisPipelineCommands::ExportSplits { output, input } => {
            super::corpus_config_commands::corpus_export_splits(output, input)
        }
        CorpusAnalysisPipelineCommands::SscReport { json, gate } => {
            super::corpus_config_commands::corpus_ssc_report(json, gate)
        }
        CorpusAnalysisPipelineCommands::ModelCard { output } => {
            super::corpus_config_commands::corpus_model_card(output)
        }
        CorpusAnalysisPipelineCommands::TrainingConfig { output, json } => {
            super::corpus_config_commands::corpus_training_config(output, json)
        }
        CorpusAnalysisPipelineCommands::PublishDataset { output } => {
            super::corpus_config_commands::corpus_publish_dataset(output)
        }
        CorpusAnalysisPipelineCommands::PublishBenchmark {
            input,
            output,
            version,
        } => super::corpus_expansion_commands::corpus_publish_benchmark(input, output, version),
        CorpusAnalysisPipelineCommands::GenerateExpansion {
            format,
            count,
            output,
            seed,
        } => {
            super::corpus_expansion_commands::corpus_generate_expansion(format, count, output, seed)
        }
        CorpusAnalysisPipelineCommands::PublishConversations { output, seed } => {
            super::corpus_config_commands::corpus_publish_conversations(output, seed)
        }
        CorpusAnalysisPipelineCommands::ConvertSsb {
            input,
            output,
            limit,
        } => super::corpus_ssb_commands::corpus_convert_ssb(input, output, limit),
        CorpusAnalysisPipelineCommands::ExtractEmbeddings {
            model,
            output,
            limit,
            input_jsonl,
        } => {
            super::corpus_ml_commands::corpus_extract_embeddings(model, output, limit, input_jsonl)
        }
        CorpusAnalysisPipelineCommands::TrainClassifier {
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
        CorpusAnalysisPipelineCommands::RunClassifier {
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
        CorpusAnalysisPipelineCommands::BatchEval {
            model,
            test_data,
            output,
            max_tokens,
        } => super::corpus_ssb_commands::corpus_batch_eval(model, test_data, output, max_tokens),
    }
}
