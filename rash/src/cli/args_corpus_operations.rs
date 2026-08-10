//! Corpus operations, benchmark, comparison, and weighting subcommands.
//!
//! Split out of `CorpusCommands` (`args_corpus.rs`) so no single clap-derive
//! `augment_subcommands` body exceeds ~24 variants (GH-215: one generated stack
//! frame per enum, ~12KB per variant at opt-level 0). Re-attached with
//! `#[command(flatten)]`, so the user-visible CLI surface is unchanged.

use clap::Subcommand;

use super::args_corpus::{CorpusFormatArg, CorpusOutputFormat};

/// Corpus operations, benchmark, comparison, and weighting subcommands (flattened into `CorpusCommands`)
#[derive(Subcommand)]
pub enum CorpusOperationsCommands {
    /// Show per-dimension pass rates, weights, and point contributions
    Dimensions {
        /// Output format
        #[arg(short, long, value_enum, default_value = "human")]
        format: CorpusOutputFormat,

        /// Filter by format (bash, makefile, dockerfile)
        #[arg(long, value_enum)]
        filter: Option<CorpusFormatArg>,
    },

    /// Find potential duplicate or similar corpus entries
    Dupes,

    /// Check convergence criteria from spec §5.2 (exit 1 if not converged)
    Converged {
        /// Minimum rate threshold (default: 99.0%)
        #[arg(long, default_value = "99.0")]
        min_rate: f64,

        /// Minimum consecutive stable iterations (default: 3)
        #[arg(long, default_value = "3")]
        min_stable: usize,

        /// Maximum delta for stability (default: 0.5%)
        #[arg(long, default_value = "0.5")]
        max_delta: f64,
    },

    /// Benchmark transpilation time per entry (spec §8.2)
    Benchmark {
        /// Maximum allowed ms per entry (flag violations)
        #[arg(long, default_value = "100")]
        max_ms: u64,

        /// Filter by format (bash, makefile, dockerfile)
        #[arg(long, value_enum)]
        filter: Option<CorpusFormatArg>,
    },

    /// Group failures by error category and message pattern
    Errors {
        /// Output format
        #[arg(short, long, value_enum, default_value = "human")]
        format: CorpusOutputFormat,

        /// Filter by format (bash, makefile, dockerfile)
        #[arg(long, value_enum)]
        filter: Option<CorpusFormatArg>,
    },

    /// Random sample of N entries with results (spot-check)
    Sample {
        /// Number of entries to sample
        #[arg(short = 'n', long, default_value = "5")]
        count: usize,

        /// Filter by format (bash, makefile, dockerfile)
        #[arg(long, value_enum)]
        filter: Option<CorpusFormatArg>,
    },

    /// Check corpus construct completeness by tier
    Completeness,

    /// CI quality gate: score + regressions + benchmark in one check
    Gate {
        /// Minimum score threshold (default: 99.0)
        #[arg(long, default_value = "99.0")]
        min_score: f64,

        /// Maximum ms per entry for benchmark (default: 200)
        #[arg(long, default_value = "200")]
        max_ms: u64,
    },

    /// Find statistical outliers by transpilation timing (z-score detection)
    Outliers {
        /// Z-score threshold for outlier detection (default: 2.0)
        #[arg(long, default_value = "2.0")]
        threshold: f64,

        /// Filter by format (bash, makefile, dockerfile)
        #[arg(long, value_enum)]
        filter: Option<CorpusFormatArg>,
    },

    /// Cross-category x quality property matrix (spec §11.11.9)
    Matrix,

    /// Timeline visualization of corpus growth from convergence log
    Timeline,

    /// Detect per-dimension score drift across convergence iterations
    Drift,

    /// Show entries sorted by transpilation time (slowest first)
    Slow {
        /// Number of entries to show
        #[arg(short = 'n', long, default_value = "20")]
        limit: usize,

        /// Filter by format (bash, makefile, dockerfile)
        #[arg(long, value_enum)]
        filter: Option<CorpusFormatArg>,
    },

    /// Show entries grouped by shell construct type (variable, loop, pipe, etc.)
    Tags,

    /// Compact one-line health check for CI status reporting
    Health,

    /// Compare two corpus entries side-by-side
    Compare {
        /// First entry ID (e.g., B-001)
        #[arg(value_name = "ID1")]
        id1: String,

        /// Second entry ID (e.g., B-002)
        #[arg(value_name = "ID2")]
        id2: String,
    },

    /// Show entry density by ID range (detect numbering gaps)
    Density,

    /// Performance percentile breakdown (P50, P90, P95, P99) per format
    Perf {
        /// Filter by format (bash, makefile, dockerfile)
        #[arg(long, value_enum)]
        filter: Option<CorpusFormatArg>,
    },

    /// CITL lint violation summary from transpiled output (spec §7.3)
    Citl {
        /// Filter by format (bash, makefile, dockerfile)
        #[arg(long, value_enum)]
        filter: Option<CorpusFormatArg>,
    },

    /// Show longest streak of consecutive passing entries
    Streak,

    /// Show V2 scoring weight contributions per dimension
    Weight,

    /// Detailed per-format quality report with dimension breakdown
    Format {
        /// Output format
        #[arg(short, long, value_enum, default_value = "human")]
        format: CorpusOutputFormat,
    },

    /// Time budget analysis: time spent per format and per tier
    Budget,

    /// Information entropy of construct distribution (diversity metric)
    Entropy,
}
