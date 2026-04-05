use clap::{Subcommand, ValueEnum};

pub use super::args_corpus_analysis::CorpusAnalysisCommands;

/// Corpus scoring subcommands (V2 quality measurement)
#[derive(Subcommand)]
pub enum CorpusCommands {
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

    /// Analysis, SSC, and dataset subcommands (flattened from CorpusAnalysisCommands)
    #[command(flatten)]
    Analysis(CorpusAnalysisCommands),
}

include!("args_corpus_datasetexpor.rs");
