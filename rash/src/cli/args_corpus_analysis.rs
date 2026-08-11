//! Corpus analysis, SSC, and dataset subcommands.
//!
//! GH-215: `clap_derive` emits one `let __clap_app = __clap_app.subcommand({..});`
//! binding per variant inside `augment_subcommands`. At opt-level 0 each binding
//! gets its own stack slot (~12KB), so a single 63-variant enum needed ~768KB of
//! frame and overflowed the 2MB stack `cargo test` gives each test thread.
//! `CorpusAnalysisCommands` is now a shell of `#[command(flatten)]` groups; clap
//! calls each group's `augment_subcommands` as a sequential statement, so peak
//! stack is parent + MAX(group) rather than parent + SUM(variants).
//!
//! `#[command(flatten)]` keeps every subcommand at the same CLI level: this is a
//! pure code-organisation change with zero effect on the user-visible surface.
//! Variant order across the groups MUST match the original declaration order so
//! `--help` ordering is unchanged.

use clap::Subcommand;

pub use super::args_corpus_analysis_diag::CorpusAnalysisDiagCommands;
pub use super::args_corpus_analysis_gates::CorpusAnalysisGatesCommands;
pub use super::args_corpus_analysis_pipeline::CorpusAnalysisPipelineCommands;

/// Corpus analysis, SSC, and dataset subcommands (split from CorpusCommands for file health)
#[derive(Subcommand)]
pub enum CorpusAnalysisCommands {
    /// Group 1 of 3: `suspicious` .. `export-dataset` (see module docs for why)
    #[command(flatten)]
    Diag(CorpusAnalysisDiagCommands),

    /// Group 2 of 3: `dataset-info` .. `pipeline-check` (see module docs for why)
    #[command(flatten)]
    Gates(CorpusAnalysisGatesCommands),

    /// Group 3 of 3: `merge-data` .. `batch-eval` (see module docs for why)
    #[command(flatten)]
    Pipeline(CorpusAnalysisPipelineCommands),
}
