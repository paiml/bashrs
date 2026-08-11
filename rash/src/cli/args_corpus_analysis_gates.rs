//! Corpus analysis subcommands, group 2 of 3 (original variants 22-42:
//! `dataset-info` .. `pipeline-check`).
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

use super::args_corpus::CorpusFormatArg;

/// Corpus analysis subcommands, group 2 of 3 (flattened into `CorpusAnalysisCommands`).
#[derive(Subcommand)]
pub enum CorpusAnalysisGatesCommands {
    /// Show dataset schema and metadata (§10.3)
    DatasetInfo,

    /// Verify corpus is ready for Hugging Face publishing (§10.3)
    PublishCheck,

    /// CITL lint pipeline: violations -> corpus entry suggestions (§7.3)
    LintPipeline,

    /// Jidoka regression detection: compare against last known good (§5.3)
    RegressionCheck,

    /// Verify 4 convergence criteria from §5.2
    ConvergenceCheck,

    /// Classify entries into domain categories A-H (§11.11)
    DomainCategories,

    /// Per-category coverage analysis and gap identification (§11.11)
    DomainCoverage,

    /// Cross-category quality requirements matrix (§11.11.9)
    DomainMatrix,

    /// Per-tier weighted pass rates and scoring breakdown (§4.3)
    TierWeights,

    /// Tier difficulty analysis with weighted vs unweighted comparison (§4.3)
    TierAnalysis,

    /// Per-tier actual vs target rate comparison with risk ranking (§2.3/§4.3)
    TierTargets,

    /// Check corpus against quality gate thresholds (§9 / §8.1)
    QualityGates,

    /// Check corpus performance metrics against thresholds (§9 / §8.2)
    MetricsCheck,

    /// Combined quality gate + metrics status overview (§9)
    GateStatus,

    /// Diagnose B2 exact match failures: show expected vs actual line mismatches
    DiagnoseB2 {
        /// Filter by format (bash, makefile, dockerfile)
        #[arg(long, value_enum)]
        filter: Option<CorpusFormatArg>,

        /// Maximum entries to show (default: 50)
        #[arg(long, default_value = "50")]
        limit: usize,
    },

    /// Fix B2 expected_contains values (reads from cached corpus run results).
    /// Without --apply, outputs JSON fixes. With --apply, updates registry.rs directly.
    FixB2 {
        /// Apply fixes directly to registry.rs instead of outputting JSON
        #[arg(long)]
        apply: bool,
    },

    /// Generate synthetic conversations for chat model training (SSC v11 S6)
    GenerateConversations {
        /// Output file path (stdout if not specified)
        #[arg(short, long)]
        output: Option<std::path::PathBuf>,

        /// Random seed for variant selection (default: 42)
        #[arg(long, default_value = "42")]
        seed: u64,

        /// Maximum entries to process (default: all)
        #[arg(long)]
        limit: Option<usize>,

        /// Output in entrenar-compatible JSONL format (instruction/response/system)
        #[arg(long)]
        entrenar: bool,
    },

    /// Run baseline classifiers: majority, keyword regex, linter (SSC v11 S5.5)
    Baselines,

    /// Show CWE taxonomy mapping for all linter rules (SSC v12 S14.2)
    CweMapping {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Export corpus as ShellSafetyBench DPO-compatible JSONL (SSC v12 S14.4)
    ExportBenchmark {
        /// Output file (stdout if not specified)
        #[arg(short, long)]
        output: Option<std::path::PathBuf>,

        /// Maximum entries to export
        #[arg(long)]
        limit: Option<usize>,
    },

    /// Validate pipeline tooling availability (SSC v12 S14 pipeline preflight)
    PipelineCheck {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}
