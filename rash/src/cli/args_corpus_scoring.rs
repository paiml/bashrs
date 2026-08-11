//! Corpus scoring, reporting, and failure-analysis subcommands.
//!
//! Split out of `CorpusCommands` (`args_corpus.rs`) so no single clap-derive
//! `augment_subcommands` body exceeds ~24 variants (GH-215: one generated stack
//! frame per enum, ~12KB per variant at opt-level 0). Re-attached with
//! `#[command(flatten)]`, so the user-visible CLI surface is unchanged.

use clap::Subcommand;

use super::args_corpus::{CorpusFormatArg, CorpusOutputFormat};

/// Corpus scoring, reporting, and failure-analysis subcommands (flattened into `CorpusCommands`)
#[derive(Subcommand)]
pub enum CorpusScoringCommands {
    /// Run V2 corpus scoring on all 500 entries
    Run {
        /// Output format
        #[arg(short, long, value_enum, default_value = "human")]
        format: CorpusOutputFormat,

        /// Filter by format (bash, makefile, dockerfile)
        #[arg(long, value_enum)]
        filter: Option<CorpusFormatArg>,

        /// Minimum score threshold (exit 1 if below)
        #[arg(long)]
        min_score: Option<f64>,

        /// Write convergence log entry to .quality/convergence.log
        #[arg(long)]
        log: bool,
    },

    /// Show detailed scoring for a single corpus entry
    Show {
        /// Entry ID (e.g., B-001, M-042, D-100)
        #[arg(value_name = "ID")]
        id: String,

        /// Output format
        #[arg(short, long, value_enum, default_value = "human")]
        format: CorpusOutputFormat,
    },

    /// Show convergence history from .quality/convergence.log
    History {
        /// Output format
        #[arg(short, long, value_enum, default_value = "human")]
        format: CorpusOutputFormat,

        /// Show last N entries
        #[arg(short = 'n', long)]
        last: Option<usize>,
    },

    /// List corpus entries with failures (any V2 dimension)
    Failures {
        /// Output format
        #[arg(short, long, value_enum, default_value = "human")]
        format: CorpusOutputFormat,

        /// Filter by format (bash, makefile, dockerfile)
        #[arg(long, value_enum)]
        filter: Option<CorpusFormatArg>,

        /// Filter by failing dimension (a, b1, b2, b3, d, e, f, g)
        #[arg(long)]
        dimension: Option<String>,
    },

    /// Generate comprehensive markdown quality report
    Report {
        /// Write to file instead of stdout
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Compare two convergence log snapshots
    Diff {
        /// Output format
        #[arg(short, long, value_enum, default_value = "human")]
        format: CorpusOutputFormat,

        /// First iteration number (default: second-to-last)
        #[arg(long)]
        from: Option<u32>,

        /// Second iteration number (default: last)
        #[arg(long)]
        to: Option<u32>,
    },

    /// Export per-entry results as structured JSON (spec §10.3)
    Export {
        /// Output file (default: stdout)
        #[arg(short, long)]
        output: Option<String>,

        /// Filter by format (bash, makefile, dockerfile)
        #[arg(long, value_enum)]
        filter: Option<CorpusFormatArg>,
    },

    /// Show per-format statistics and convergence trends (spec §11.10)
    Stats {
        /// Output format
        #[arg(short, long, value_enum, default_value = "human")]
        format: CorpusOutputFormat,
    },

    /// Run metamorphic relation checks on a corpus entry (spec §11.2)
    Check {
        /// Entry ID (e.g., B-001, M-042, D-100)
        #[arg(value_name = "ID")]
        id: String,

        /// Output format
        #[arg(short, long, value_enum, default_value = "human")]
        format: CorpusOutputFormat,
    },

    /// Classify corpus entry difficulty as Tier 1-5 (spec §2.3)
    Difficulty {
        /// Entry ID (e.g., B-001) or "all" for full corpus
        #[arg(value_name = "ID")]
        id: String,

        /// Output format
        #[arg(short, long, value_enum, default_value = "human")]
        format: CorpusOutputFormat,
    },

    /// One-line corpus summary for CI and scripts (spec §10)
    Summary,

    /// Show corpus size growth over time from convergence log (spec §4)
    Growth {
        /// Output format
        #[arg(short, long, value_enum, default_value = "human")]
        format: CorpusOutputFormat,
    },

    /// Show tier x format coverage matrix (spec §2.3)
    Coverage {
        /// Output format
        #[arg(short, long, value_enum, default_value = "human")]
        format: CorpusOutputFormat,
    },

    /// Validate corpus entries for metadata correctness (spec §2.3)
    Validate {
        /// Output format
        #[arg(short, long, value_enum, default_value = "human")]
        format: CorpusOutputFormat,
    },

    /// Pareto analysis of corpus failures by dimension (spec §11.10.4)
    Pareto {
        /// Output format
        #[arg(short, long, value_enum, default_value = "human")]
        format: CorpusOutputFormat,

        /// Filter by format (bash, makefile, dockerfile)
        #[arg(long, value_enum)]
        filter: Option<CorpusFormatArg>,

        /// Show top N dimensions only
        #[arg(short = 'n', long)]
        top: Option<usize>,
    },

    /// Risk classification of corpus failures by severity (spec §11.10.4)
    Risk {
        /// Output format
        #[arg(short, long, value_enum, default_value = "human")]
        format: CorpusOutputFormat,

        /// Filter by risk level (high, medium, low)
        #[arg(long)]
        level: Option<String>,
    },

    /// Generate Five Whys root cause template for a failing entry (spec §11.10.3)
    WhyFailed {
        /// Entry ID (e.g., B-143)
        #[arg(value_name = "ID")]
        id: String,

        /// Output format
        #[arg(short, long, value_enum, default_value = "human")]
        format: CorpusOutputFormat,
    },

    /// Detect regressions between convergence log iterations (spec §5.3 Jidoka)
    Regressions {
        /// Output format
        #[arg(short, long, value_enum, default_value = "human")]
        format: CorpusOutputFormat,
    },

    /// Visual heatmap of entries x V2 dimensions (pass/fail matrix)
    Heatmap {
        /// Maximum entries to show (default: 20, failures first)
        #[arg(short = 'n', long, default_value = "20")]
        limit: usize,

        /// Filter by format (bash, makefile, dockerfile)
        #[arg(long, value_enum)]
        filter: Option<CorpusFormatArg>,
    },

    /// Compact multi-corpus convergence dashboard (spec §11.10.5)
    Dashboard,

    /// Search corpus entries by ID, name, or description pattern
    Search {
        /// Search pattern (substring match, case-insensitive)
        #[arg(value_name = "PATTERN")]
        pattern: String,

        /// Output format
        #[arg(short, long, value_enum, default_value = "human")]
        format: CorpusOutputFormat,

        /// Filter by format (bash, makefile, dockerfile)
        #[arg(long, value_enum)]
        filter: Option<CorpusFormatArg>,
    },

    /// Show score trend as Unicode sparkline from convergence log
    Sparkline,

    /// Show top/bottom entries ranked by failure count
    Top {
        /// Number of entries to show
        #[arg(short = 'n', long, default_value = "10")]
        limit: usize,

        /// Show bottom (most failures) instead of top (fewest)
        #[arg(long)]
        worst: bool,

        /// Filter by format (bash, makefile, dockerfile)
        #[arg(long, value_enum)]
        filter: Option<CorpusFormatArg>,
    },

    /// Show entries grouped by domain-specific category (spec §11.11)
    Categories {
        /// Output format
        #[arg(short, long, value_enum, default_value = "human")]
        format: CorpusOutputFormat,
    },
}
