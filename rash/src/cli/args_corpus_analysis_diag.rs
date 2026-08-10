//! Corpus analysis subcommands, group 1 of 3 (original variants 1-21:
//! `suspicious` .. `export-dataset`).
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

use super::args_corpus::{CorpusFormatArg, DatasetExportFormat};

/// Corpus analysis subcommands, group 1 of 3 (flattened into `CorpusAnalysisCommands`).
#[derive(Subcommand)]
pub enum CorpusAnalysisDiagCommands {
    /// Tarantula suspiciousness ranking across all decisions (§11.10.1)
    Suspicious {
        /// Maximum entries to show
        #[arg(short = 'n', long, default_value = "20")]
        limit: usize,
    },
    /// Decision frequency and pass/fail correlation summary (§11.10.1)
    Decisions,

    /// Mine CITL fix patterns from corpus failures (§11.10.2)
    Patterns,

    /// Query CITL patterns for a specific error signal (§11.10.2)
    PatternQuery {
        /// Error signal to query (e.g. B3_behavioral_fail, D_lint_fail, G_cross_shell_fail)
        #[arg(value_name = "SIGNAL")]
        signal: String,
    },
    /// Suggest fixes for a failing corpus entry (§11.10.2)
    FixSuggest {
        /// Entry ID (e.g. B-143)
        #[arg(value_name = "ID")]
        id: String,
    },
    /// Show decision connectivity graph with usage counts (§11.10.3)
    Graph,

    /// Impact-weighted decision priority (suspiciousness × connectivity) (§11.10.3)
    Impact {
        /// Maximum entries to show
        #[arg(short = 'n', long, default_value = "20")]
        limit: usize,
    },
    /// Show blast radius of fixing a specific decision (§11.10.3)
    BlastRadius {
        /// Decision key (e.g. assignment_value:bool_literal)
        #[arg(value_name = "DECISION")]
        decision: String,
    },

    /// Deduplicated error view with counts and risk classification (§11.10.4)
    Dedup,

    /// Risk-prioritized fix backlog with weak supervision labels (§11.10.4)
    Triage,

    /// Show programmatic labeling rules and match counts (§11.10.4)
    LabelRules,

    /// Full iteration x format convergence table (§11.10.5)
    ConvergeTable,

    /// Per-format delta between two iterations (§11.10.5)
    ConvergeDiff {
        /// First iteration number (default: second-to-last)
        #[arg(long)]
        from: Option<u32>,
        /// Second iteration number (default: last)
        #[arg(long)]
        to: Option<u32>,
    },

    /// Per-format convergence status with trend (§11.10.5)
    ConvergeStatus,

    /// Mine fix patterns from git history (§11.9.1)
    Mine {
        /// Maximum number of commits to analyze
        #[arg(short = 'n', long, default_value = "100")]
        limit: usize,
    },

    /// Find fix commits without regression corpus entries (§11.9.3)
    FixGaps {
        /// Maximum number of commits to analyze
        #[arg(short = 'n', long, default_value = "100")]
        limit: usize,
    },

    /// Cross-project defect pattern analysis (§11.9.4)
    OrgPatterns,

    /// Validate all corpus entries against formal grammar (§11.8)
    SchemaValidate,

    /// Categorize grammar violations by GRAM-001..GRAM-008 (§11.8.5)
    GrammarErrors,

    /// Display formal grammar specification for a format (§11.8.1-11.8.3)
    FormatGrammar {
        /// Target format to show grammar for
        #[arg(value_enum)]
        format: CorpusFormatArg,
    },

    /// Export corpus as dataset (JSON/CSV/JSONL) for HF publishing (§10.3)
    ExportDataset {
        /// Export format
        #[arg(long, default_value = "json")]
        format: DatasetExportFormat,

        /// Output file path (stdout if not specified)
        #[arg(short, long)]
        output: Option<std::path::PathBuf>,
    },
}
