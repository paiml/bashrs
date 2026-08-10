//! Corpus visualization, diagnostics, and tier-breakdown subcommands.
//!
//! Split out of `CorpusCommands` (`args_corpus.rs`) so no single clap-derive
//! `augment_subcommands` body exceeds ~24 variants (GH-215: one generated stack
//! frame per enum, ~12KB per variant at opt-level 0). Re-attached with
//! `#[command(flatten)]`, so the user-visible CLI surface is unchanged.

use clap::Subcommand;

/// Corpus visualization, diagnostics, and tier-breakdown subcommands (flattened into `CorpusCommands`)
#[derive(Subcommand)]
pub enum CorpusDiagnosticsCommands {
    /// Auto-generate improvement suggestions from current state
    Todo,

    /// Scatter view: entries on a timing x failure-count grid
    Scatter,

    /// Grade distribution histogram across all entries
    GradeDist,

    /// Pivot table: tier x format cross-tabulation with pass rates
    Pivot,

    /// Dimension correlation matrix (which failures co-occur)
    Corr,

    /// Schema enforcement layer status per format (spec §11.8)
    Schema,

    /// ASCII chart of score over iterations from convergence log
    HistoryChart,

    /// Detect potentially flaky entries (high timing variance)
    Flaky {
        /// Minimum coefficient of variation for flakiness (default: 0.5)
        #[arg(long, default_value = "0.5")]
        threshold: f64,
    },

    /// Corpus composition profile: tier, format, category breakdown
    Profile,

    /// Find quality gaps: dimensions where specific formats underperform
    Gaps,

    /// Compact JSON summary for CI/script consumption
    SummaryJson,

    /// Full audit trail: entries, tests, build, lint status
    Audit,

    /// Per-tier detailed breakdown with pass rates
    TierDetail,

    /// ID range info per format (first, last, count)
    IdRange,

    /// Compact tier summary table
    Tiers,

    /// Map of failing entries with dimension failures
    FailMap,

    /// Score range analysis: min, max, median, IQR per format
    ScoreRange,

    /// Top-K entries by number of passing dimensions
    Topk {
        /// Number of entries to show
        #[arg(short = 'n', long, default_value = "10")]
        limit: usize,
    },

    /// Side-by-side format comparison
    FormatCmp,

    /// Stability index: ratio of entries never failing across iterations
    Stability,

    /// Simple pass rate display per format
    Rate,

    /// Distribution of entries by timing buckets
    Dist,

    /// Show decision trace for a single corpus entry (§11.10.1)
    Trace {
        /// Entry ID (e.g., B-001)
        #[arg(value_name = "ID")]
        id: String,
    },
}
