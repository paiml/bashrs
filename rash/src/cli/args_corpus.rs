use clap::{Subcommand, ValueEnum};

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

    /// Show tier × format coverage matrix (spec §2.3)
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

    /// Visual heatmap of entries × V2 dimensions (pass/fail matrix)
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

    /// Cross-category × quality property matrix (spec §11.11.9)
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

    /// Scatter view: entries on a timing × failure-count grid
    Scatter,

    /// Grade distribution histogram across all entries
    GradeDist,

    /// Pivot table: tier × format cross-tabulation with pass rates
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

    /// Corpus version and metadata info
    Version,

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

    /// Show dataset schema and metadata (§10.3)
    DatasetInfo,

    /// Verify corpus is ready for Hugging Face publishing (§10.3)
    PublishCheck,

    /// CITL lint pipeline: violations → corpus entry suggestions (§7.3)
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

    /// Merge corpus + verificar data into unified training JSONL (SSC v12 S14 Step 7.4)
    MergeData {
        /// Output file
        #[arg(short, long)]
        output: std::path::PathBuf,

        /// Additional JSONL input files to merge (e.g., verificar-labeled.jsonl)
        #[arg(short, long)]
        input: Vec<std::path::PathBuf>,

        /// Random seed for shuffling
        #[arg(long, default_value = "42")]
        seed: u64,
    },

    /// Cross-validate bashrs labels against ShellCheck (SSC v12 S14.9 Step 7.4e)
    ShellcheckValidate {
        /// Number of samples to validate
        #[arg(long, default_value = "500")]
        samples: usize,

        /// Random seed
        #[arg(long, default_value = "42")]
        seed: u64,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Run eval harness on benchmark predictions (SSC v12 S14.5)
    EvalBenchmark {
        /// Predictions JSONL file
        #[arg(short, long)]
        predictions: std::path::PathBuf,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Label external JSONL with linter findings + CWE mappings (SSC v12 pipeline)
    Label {
        /// Input JSONL file (one shell script per line, field: "script" or "text")
        #[arg(short, long)]
        input: std::path::PathBuf,

        /// Output JSONL file (labeled entries)
        #[arg(short, long)]
        output: Option<std::path::PathBuf>,

        /// Output format
        #[arg(long, default_value = "json")]
        format: String,
    },

    /// Audit safety label accuracy (SSC v11 S5.3, C-LABEL-001)
    LabelAudit {
        /// Maximum unsafe entries to audit (default: 100)
        #[arg(short = 'n', long, default_value = "100")]
        limit: usize,
    },

    /// Run out-of-distribution generalization tests (SSC v11 S5.6)
    GeneralizationTests,

    /// Validate tokenizer quality on shell constructs (SSC v11 S5.2, C-TOK-001)
    TokenizerValidation,

    /// Run all SSC contract validations (pre-training gate)
    ValidateContracts,

    /// Export dataset with train/val/test splits for ML training
    ExportSplits {
        /// Output directory for split JSONL files
        #[arg(short, long)]
        output: Option<std::path::PathBuf>,

        /// Input merged JSONL (if omitted, uses corpus transpilation)
        #[arg(long)]
        input: Option<std::path::PathBuf>,
    },

    /// Show comprehensive SSC v11 readiness report
    SscReport {
        /// Output as JSON
        #[arg(long)]
        json: bool,
        /// CI gate mode: exit 1 if any section fails
        #[arg(long)]
        gate: bool,
    },

    /// Generate HuggingFace model card for SSC dataset/classifier (S6.5, S9)
    ModelCard {
        /// Output file (default: stdout)
        #[arg(short, long)]
        output: Option<std::path::PathBuf>,
    },

    /// Export entrenar-compatible training configuration (S9 CLF-001)
    TrainingConfig {
        /// Output file (default: stdout)
        #[arg(short, long)]
        output: Option<std::path::PathBuf>,

        /// Output as JSON instead of YAML
        #[arg(long)]
        json: bool,
    },

    /// Export complete HuggingFace-ready dataset directory (S9 GEN-003)
    PublishDataset {
        /// Output directory (required)
        #[arg(short, long)]
        output: std::path::PathBuf,
    },

    /// Publish ShellSafetyBench to HuggingFace (SSC v12 S14.7, Phase 10)
    PublishBenchmark {
        /// Directory containing SSB split files (train.jsonl, val.jsonl, test.jsonl)
        #[arg(short = 'i', long)]
        input: std::path::PathBuf,

        /// Output directory for HuggingFace-ready repository
        #[arg(short, long)]
        output: std::path::PathBuf,

        /// Version tag (e.g., "1.0.0")
        #[arg(long, default_value = "1.0.0")]
        version: String,
    },

    /// Generate expansion entries for ShellSafetyBench (Phase 9 #10: 27K → 50K+)
    GenerateExpansion {
        /// Script format to generate
        #[arg(short, long, value_parser = ["bash", "makefile", "dockerfile"])]
        format: String,

        /// Number of entries to generate
        #[arg(short, long, default_value = "5000")]
        count: usize,

        /// Output JSONL file
        #[arg(short, long)]
        output: std::path::PathBuf,

        /// Random seed for reproducibility
        #[arg(short, long, default_value = "42")]
        seed: u64,
    },

    /// Export HuggingFace-ready conversation dataset (S6.6 paiml/shell-safety-conversations)
    PublishConversations {
        /// Output directory (required)
        #[arg(short, long)]
        output: std::path::PathBuf,

        /// Random seed for variant selection
        #[arg(short, long, default_value = "42")]
        seed: u64,
    },

    /// Convert SSB splits to entrenar ChatML JSONL for chat model training (PMAT-167)
    ConvertSsb {
        /// Input JSONL file (SSB format: {"input":"...","label":0|1})
        #[arg(short, long)]
        input: std::path::PathBuf,

        /// Output file (stdout if not specified)
        #[arg(short, long)]
        output: Option<std::path::PathBuf>,

        /// Maximum entries to convert
        #[arg(long)]
        limit: Option<usize>,
    },

    /// Extract [CLS] embeddings from CodeBERT for all corpus entries (CLF-RUN step 1)
    ExtractEmbeddings {
        /// Path to CodeBERT model directory (must contain model.safetensors)
        #[arg(short, long)]
        model: std::path::PathBuf,

        /// Output file for cached embeddings (JSONL)
        #[arg(short, long)]
        output: std::path::PathBuf,

        /// Maximum number of entries to process (for testing)
        #[arg(short, long)]
        limit: Option<usize>,

        /// Extract from JSONL file instead of corpus (format: {"input":"...","label":N})
        #[arg(long)]
        input_jsonl: Option<std::path::PathBuf>,
    },

    /// Train linear probe classifier on cached embeddings (CLF-RUN step 2-3)
    TrainClassifier {
        /// Path to cached embeddings JSONL (from extract-embeddings)
        #[arg(short, long)]
        embeddings: std::path::PathBuf,

        /// Output directory for probe weights and evaluation report
        #[arg(short, long)]
        output: std::path::PathBuf,

        /// Training epochs
        #[arg(long, default_value = "30")]
        epochs: usize,

        /// Learning rate
        #[arg(long, default_value = "0.01")]
        learning_rate: f32,

        /// Random seed for train/test split
        #[arg(short, long, default_value = "42")]
        seed: u64,

        /// Maximum entries to use (caps training data to avoid data labeling gaps)
        #[arg(long)]
        max_entries: Option<usize>,

        /// Additional embedding JSONL files to augment training data (e.g. adversarial entries)
        #[arg(long)]
        augment: Vec<std::path::PathBuf>,

        /// Use MLP probe (2-layer with ReLU) instead of linear probe
        #[arg(long)]
        mlp: bool,

        /// MLP hidden layer size (only with --mlp)
        #[arg(long, default_value = "128")]
        mlp_hidden: usize,
    },

    /// Run full CLF-RUN pipeline: extract embeddings → train → evaluate (requires --features ml)
    RunClassifier {
        /// Path to CodeBERT model directory
        #[arg(short, long)]
        model: std::path::PathBuf,

        /// Output directory for all artifacts
        #[arg(short, long)]
        output: std::path::PathBuf,

        /// Training epochs
        #[arg(long, default_value = "30")]
        epochs: usize,

        /// Learning rate
        #[arg(long, default_value = "0.01")]
        learning_rate: f32,

        /// Random seed
        #[arg(short, long, default_value = "42")]
        seed: u64,
    },

    /// Run batch inference on test split using a trained model checkpoint (SSC v12 S14)
    BatchEval {
        /// Path to model directory (config.json + safetensors + optional LoRA adapter)
        #[arg(short, long)]
        model: std::path::PathBuf,

        /// Path to test JSONL file (entries with "input" and "label" fields)
        #[arg(short, long)]
        test_data: std::path::PathBuf,

        /// Output predictions JSONL path (compatible with eval-benchmark)
        #[arg(short, long)]
        output: std::path::PathBuf,

        /// Maximum tokens to generate per entry (default: 128)
        #[arg(long, default_value = "128")]
        max_tokens: usize,
    },
}

/// Dataset export format
#[derive(Clone, Debug, Default, ValueEnum)]
pub enum DatasetExportFormat {
    /// JSON array (pretty-printed)
    #[default]
    Json,
    /// JSON Lines (one object per line)
    Jsonl,
    /// CSV with headers
    Csv,
    /// Classification JSONL for ML fine-tuning ({"input":"...","label":N})
    Classification,
    /// Multi-label classification JSONL ({"input":"...","labels":[0.0, 1.0, ...]})
    MultiLabelClassification,
}

/// Corpus output format
#[derive(Clone, Debug, Default, ValueEnum)]
pub enum CorpusOutputFormat {
    /// Human-readable report
    #[default]
    Human,
    /// JSON output
    Json,
}

/// Corpus format filter
#[derive(Clone, Debug, ValueEnum)]
pub enum CorpusFormatArg {
    /// Bash shell scripts
    Bash,
    /// Makefiles
    Makefile,
    /// Dockerfiles
    Dockerfile,
}
