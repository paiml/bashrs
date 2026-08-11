use clap::{Subcommand, ValueEnum};

pub use super::args_corpus_analysis::CorpusAnalysisCommands;
pub use super::args_corpus_diagnostics::CorpusDiagnosticsCommands;
pub use super::args_corpus_operations::CorpusOperationsCommands;
pub use super::args_corpus_scoring::CorpusScoringCommands;

/// Corpus scoring subcommands (V2 quality measurement)
///
/// A shell of `#[command(flatten)]` groups: clap-derive emits one enormous stack
/// frame per `augment_subcommands` body (~12KB per variant at opt-level 0), and a
/// single 73-variant body overflowed the 2MB test thread stack (GH-215). Flattened
/// children are augmented by sequential calls, so peak stack is
/// `parent + max(child)` rather than `parent + sum(children)`.
///
/// `#[command(flatten)]` keeps every subcommand at the same CLI level, so the
/// user-visible command surface is byte-identical to the unsplit enum.
#[derive(Subcommand)]
pub enum CorpusCommands {
    /// Scoring, reporting, and failure-analysis subcommands (flattened from CorpusScoringCommands)
    #[command(flatten)]
    Scoring(CorpusScoringCommands),

    /// Operations, benchmark, comparison, and weighting subcommands (flattened from CorpusOperationsCommands)
    #[command(flatten)]
    Operations(CorpusOperationsCommands),

    /// Visualization, diagnostics, and tier-breakdown subcommands (flattened from CorpusDiagnosticsCommands)
    #[command(flatten)]
    Diagnostics(CorpusDiagnosticsCommands),

    /// Analysis, SSC, and dataset subcommands (flattened from CorpusAnalysisCommands)
    #[command(flatten)]
    Analysis(CorpusAnalysisCommands),

    /// Corpus version and metadata info
    Version,
}

include!("args_corpus_datasetexpor.rs");
