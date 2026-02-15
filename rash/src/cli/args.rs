use crate::models::{ShellDialect, VerificationLevel};
use crate::validation::ValidationLevel;
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "bashrs")]
#[command(about = "Rust-to-Shell transpiler for deterministic bootstrap scripts")]
#[command(version = env!("CARGO_PKG_VERSION"))]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Verification stringency level
    #[arg(long, default_value = "strict")]
    pub verify: VerificationLevel,

    /// Target shell dialect
    #[arg(long, default_value = "posix")]
    pub target: ShellDialect,

    /// ShellCheck-compatible validation level
    #[arg(long, default_value = "minimal")]
    pub validation: ValidationLevel,

    /// Enable strict mode (fail on warnings)
    #[arg(long)]
    pub strict: bool,

    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Transpile Rust source to shell script
    Build {
        /// Input Rust file
        #[arg(value_name = "FILE")]
        input: PathBuf,

        /// Output shell script file
        #[arg(short, long, default_value = "install.sh")]
        output: PathBuf,

        /// Emit verification proof
        #[arg(long)]
        emit_proof: bool,

        /// Disable optimizations
        #[arg(long)]
        no_optimize: bool,
    },

    /// Check Rust source for Rash compatibility
    Check {
        /// Input Rust file
        #[arg(value_name = "FILE")]
        input: PathBuf,
    },

    /// Initialize new Rash project
    Init {
        /// Project directory
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Project name
        #[arg(long)]
        name: Option<String>,
    },

    /// Verify shell script matches Rust source
    Verify {
        /// Rust source file
        rust_source: PathBuf,

        /// Shell script file
        shell_script: PathBuf,
    },

    /// Generate formal verification inspection report
    Inspect {
        /// Input AST file (JSON) or inline AST specification
        #[arg(value_name = "AST")]
        input: String,

        /// Output format
        #[arg(long, default_value = "markdown")]
        format: InspectionFormat,

        /// Output file (defaults to stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Include detailed traces
        #[arg(long)]
        detailed: bool,
    },

    // Playground feature removed in v1.0 - will be moved to separate rash-playground crate in v1.1
    /// Compile to standalone binary
    Compile {
        /// Input Rust source file
        rust_source: PathBuf,

        /// Output binary path
        #[arg(short, long)]
        output: PathBuf,

        /// Runtime type
        #[arg(long, value_enum, default_value = "dash")]
        runtime: CompileRuntime,

        /// Create self-extracting script instead of binary
        #[arg(long)]
        self_extracting: bool,

        /// Build distroless container
        #[arg(long)]
        container: bool,

        /// Container format
        #[arg(long, value_enum, default_value = "oci")]
        container_format: ContainerFormatArg,
    },

    /// Lint shell scripts or Rust source for safety issues
    Lint {
        /// Input file (shell script or Rust source)
        #[arg(value_name = "FILE")]
        input: PathBuf,

        /// Output format
        #[arg(long, value_enum, default_value = "human")]
        format: LintFormat,

        /// Enable auto-fix suggestions (SAFE fixes only)
        #[arg(long)]
        fix: bool,

        /// Apply fixes with assumptions (requires --fix, includes SAFE + SAFE-WITH-ASSUMPTIONS fixes)
        #[arg(long, requires = "fix")]
        fix_assumptions: bool,

        /// Output file for fixed content
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Disable .bashrsignore file processing (Issue #58)
        #[arg(long)]
        no_ignore: bool,

        /// Path to ignore file (defaults to .bashrsignore in project root)
        #[arg(long, value_name = "FILE")]
        ignore_file: Option<PathBuf>,

        /// Suppress info-level messages, show only warnings and errors (Issue #75)
        #[arg(short = 'q', long)]
        quiet: bool,

        /// Minimum severity level to display (info, warning, error)
        #[arg(long, value_enum, default_value = "info")]
        level: LintLevel,

        /// Ignore specific rule codes (comma-separated: SEC010,DET002)
        #[arg(long, value_name = "RULES")]
        ignore: Option<String>,

        /// Exclude specific rule (shellcheck-compatible, can be repeated)
        #[arg(short = 'e', value_name = "CODE", action = clap::ArgAction::Append)]
        exclude: Option<Vec<String>>,

        /// Export diagnostics in CITL format for OIP integration (Issue #83)
        #[arg(long, value_name = "FILE")]
        citl_export: Option<PathBuf>,

        /// Lint profile for specialized validation (standard, coursera, devcontainer)
        #[arg(long, value_enum, default_value = "standard")]
        profile: LintProfileArg,

        /// Enable graded output mode (educational scoring with pass/fail criteria)
        #[arg(long)]
        graded: bool,
    },

    /// Purify bash scripts (determinism + idempotency + safety)
    Purify {
        /// Input bash script file
        #[arg(value_name = "FILE")]
        input: PathBuf,

        /// Output file (defaults to stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Show detailed transformation report
        #[arg(long)]
        report: bool,

        /// Generate test suite for purified script
        #[arg(long)]
        with_tests: bool,

        /// Generate property-based tests (100+ cases)
        #[arg(long)]
        property_tests: bool,

        /// Enable gradual type checking (check type annotations in comments)
        #[arg(long)]
        type_check: bool,

        /// Emit runtime type guards in purified output (implies --type-check)
        #[arg(long)]
        emit_guards: bool,

        /// Treat type warnings as errors
        #[arg(long)]
        type_strict: bool,
    },

    /// Makefile parsing, purification, and transformation
    Make {
        #[command(subcommand)]
        command: MakeCommands,
    },

    /// Dockerfile purification and linting (NEW in v6.36.0)
    Dockerfile {
        #[command(subcommand)]
        command: DockerfileCommands,
    },

    /// Dev Container validation (devcontainer.json) (NEW in v6.43.0)
    Devcontainer {
        #[command(subcommand)]
        command: DevContainerCommands,
    },

    /// Shell artifact compliance system (NEW in v7.1.0 - SPEC-COMPLY-2026-001)
    Comply {
        #[command(subcommand)]
        command: ComplyCommands,
    },

    /// V2 corpus scoring and quality measurement (NEW in v7.2.0)
    Corpus {
        #[command(subcommand)]
        command: CorpusCommands,
    },

    /// Shell configuration file management (NEW in v7.0)
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },

    /// Interactive REPL with integrated debugger (NEW in v7.0)
    Repl {
        /// Enable debug mode
        #[arg(long)]
        debug: bool,

        /// Enable sandboxed execution
        #[arg(long)]
        sandboxed: bool,

        /// Maximum memory usage in MB (default: 100)
        #[arg(long)]
        max_memory: Option<usize>,

        /// Timeout in seconds (default: 30)
        #[arg(long)]
        timeout: Option<u64>,

        /// Maximum recursion depth (default: 100)
        #[arg(long)]
        max_depth: Option<usize>,
    },

    /// Terminal User Interface with multi-panel layout (NEW)
    #[cfg(feature = "tui")]
    Tui,

    /// Enforce quality gates (NEW in v6.42.0)
    Gate {
        /// Quality gate tier (1=fast, 2=pre-commit, 3=nightly)
        #[arg(long, default_value = "1")]
        tier: u8,

        /// Report format
        #[arg(long, value_enum, default_value = "human")]
        report: ReportFormat,
    },

    /// Run bash script tests (NEW in v6.10.0 - Bash Quality Tools)
    Test {
        /// Input bash script file
        #[arg(value_name = "FILE")]
        input: PathBuf,

        /// Output format
        #[arg(long, value_enum, default_value = "human")]
        format: TestOutputFormat,

        /// Show detailed test results
        #[arg(long)]
        detailed: bool,

        /// Run only tests matching pattern
        #[arg(long)]
        pattern: Option<String>,
    },

    /// Score bash script quality (NEW in v6.11.0 - Bash Quality Tools)
    Score {
        /// Input bash script file
        #[arg(value_name = "FILE")]
        input: PathBuf,

        /// Output format
        #[arg(long, value_enum, default_value = "human")]
        format: ScoreOutputFormat,

        /// Show detailed dimension scores
        #[arg(long)]
        detailed: bool,

        /// Use Dockerfile-specific quality scoring
        #[arg(long)]
        dockerfile: bool,

        /// Include runtime performance metrics in score (requires Docker daemon for Dockerfiles)
        #[arg(long)]
        runtime: bool,

        /// Show letter grade (A+, A, B+, B, C+, C, D, F)
        #[arg(long)]
        grade: bool,

        /// Apply platform-specific scoring profile (coursera)
        #[arg(long, value_enum)]
        profile: Option<LintProfileArg>,
    },

    /// Run comprehensive quality audit (NEW in v6.12.0 - Bash Quality Tools)
    Audit {
        /// Input bash script file
        #[arg(value_name = "FILE")]
        input: PathBuf,

        /// Output format
        #[arg(long, value_enum, default_value = "human")]
        format: AuditOutputFormat,

        /// Enable strict mode (fail on warnings)
        #[arg(long)]
        strict: bool,

        /// Show detailed check results
        #[arg(long)]
        detailed: bool,

        /// Minimum grade required (A+, A, B+, B, C+, C, D, F)
        #[arg(long)]
        min_grade: Option<String>,
    },

    /// Generate coverage report (NEW in v6.13.0 - Bash Quality Tools)
    Coverage {
        /// Input bash script file
        #[arg(value_name = "FILE")]
        input: PathBuf,

        /// Output format
        #[arg(long, value_enum, default_value = "terminal")]
        format: CoverageOutputFormat,

        /// Minimum coverage percentage required
        #[arg(long)]
        min: Option<u8>,

        /// Show detailed coverage breakdown
        #[arg(long)]
        detailed: bool,

        /// Output file for HTML/LCOV format
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Format bash scripts (NEW in v6.14.0 - Bash Quality Tools)
    Format {
        /// Input bash script file(s)
        #[arg(value_name = "FILE", required = true)]
        inputs: Vec<PathBuf>,

        /// Check if files are formatted without applying changes
        #[arg(long)]
        check: bool,

        /// Show diff without applying changes
        #[arg(long)]
        dry_run: bool,

        /// Output file (for single input file)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Benchmark shell script(s) with scientific rigor (NEW in v6.25.0)
    Bench {
        /// Shell script(s) to benchmark
        #[arg(value_name = "SCRIPT", required = true)]
        scripts: Vec<PathBuf>,

        /// Number of warmup iterations
        #[arg(short = 'w', long, default_value = "3")]
        warmup: usize,

        /// Number of measured iterations
        #[arg(short = 'i', long, default_value = "10")]
        iterations: usize,

        /// Output results to JSON file
        #[arg(short = 'o', long)]
        output: Option<PathBuf>,

        /// Enable quality gates (lint + determinism checks)
        #[arg(short = 's', long)]
        strict: bool,

        /// Verify script produces identical output
        #[arg(long)]
        verify_determinism: bool,

        /// Show raw iteration times
        #[arg(long)]
        show_raw: bool,

        /// Suppress progress output
        #[arg(short = 'q', long)]
        quiet: bool,

        /// Measure memory usage (requires /usr/bin/time)
        #[arg(short = 'm', long)]
        measure_memory: bool,

        /// Output results in CSV format (Issue #77)
        #[arg(long)]
        csv: bool,

        /// Disable ANSI colors in output (Issue #77)
        #[arg(long)]
        no_color: bool,
    },

    /// Explain shell error using ML classification (NEW in v6.40.0)
    #[cfg(feature = "oracle")]
    ExplainError {
        /// Error message to classify
        #[arg(value_name = "ERROR")]
        error: String,

        /// Command that produced the error (optional, improves accuracy)
        #[arg(short = 'c', long)]
        command: Option<String>,

        /// Shell type (bash, sh, zsh)
        #[arg(long, default_value = "bash")]
        shell: String,

        /// Output format
        #[arg(long, value_enum, default_value = "human")]
        format: ExplainErrorFormat,

        /// Show confidence scores and related patterns
        #[arg(long)]
        detailed: bool,
    },

    /// Execute playbook-driven state machine tests (NEW in v6.46.0 - Probar Integration)
    Playbook {
        /// Playbook YAML file
        #[arg(value_name = "FILE")]
        input: PathBuf,

        /// Run the playbook (default: validate only)
        #[arg(long)]
        run: bool,

        /// Output format
        #[arg(long, value_enum, default_value = "human")]
        format: PlaybookFormat,

        /// Verbose output showing state transitions
        #[arg(long)]
        verbose: bool,

        /// Dry run (show what would be executed)
        #[arg(long)]
        dry_run: bool,
    },

    /// Mutation testing for shell scripts (NEW in v6.46.0 - Probar Integration)
    Mutate {
        /// Shell script to mutate
        #[arg(value_name = "FILE")]
        input: PathBuf,

        /// Mutation configuration file
        #[arg(long)]
        config: Option<PathBuf>,

        /// Output format
        #[arg(long, value_enum, default_value = "human")]
        format: MutateFormat,

        /// Number of mutants to generate
        #[arg(long, default_value = "10")]
        count: usize,

        /// Show surviving mutants (test quality issues)
        #[arg(long)]
        show_survivors: bool,

        /// Output directory for mutant files
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Deterministic simulation replay (NEW in v6.46.0 - Probar Integration)
    Simulate {
        /// Shell script to simulate
        #[arg(value_name = "FILE")]
        input: PathBuf,

        /// Random seed for deterministic replay
        #[arg(long, default_value = "42")]
        seed: u64,

        /// Verify determinism (run twice, compare outputs)
        #[arg(long)]
        verify: bool,

        /// Mock external commands
        #[arg(long)]
        mock_externals: bool,

        /// Output format
        #[arg(long, value_enum, default_value = "human")]
        format: SimulateFormat,

        /// Record execution trace
        #[arg(long)]
        trace: bool,
    },

    /// TDD-first installer framework (NEW in v7.0 - Issue #104)
    Installer {
        #[command(subcommand)]
        command: InstallerCommands,
    },
}

/// Output format for playbook command
#[derive(Clone, Debug, Default, ValueEnum)]
pub enum PlaybookFormat {
    /// Human-readable output
    #[default]
    Human,
    /// JSON output
    Json,
    /// JUnit XML for CI integration
    Junit,
}

/// Output format for mutate command
#[derive(Clone, Debug, Default, ValueEnum)]
pub enum MutateFormat {
    /// Human-readable output
    #[default]
    Human,
    /// JSON output
    Json,
    /// CSV for analysis
    Csv,
}

/// Output format for simulate command
#[derive(Clone, Debug, Default, ValueEnum)]
pub enum SimulateFormat {
    /// Human-readable output
    #[default]
    Human,
    /// JSON output
    Json,
    /// Detailed trace format
    Trace,
}

/// Output format for explain-error command
#[derive(Clone, Debug, Default, ValueEnum)]
pub enum ExplainErrorFormat {
    /// Human-readable output
    #[default]
    Human,
    /// JSON output
    Json,
}

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

/// Comply subcommands (SPEC-COMPLY-2026-001)
#[derive(Subcommand)]
pub enum ComplyCommands {
    /// Initialize .bashrs/comply.toml manifest
    Init {
        /// Scopes to track
        #[arg(long, value_enum, default_value = "project")]
        scope: ComplyScopeArg,

        /// Enable pzsh integration
        #[arg(long)]
        pzsh: bool,

        /// Strict mode (all rules enforced, zero tolerance)
        #[arg(long)]
        strict: bool,
    },

    /// Layer 1 (Jidoka): Automated compliance verification
    Check {
        /// Project path
        #[arg(short, long, default_value = ".")]
        path: PathBuf,

        /// Scope to check
        #[arg(long, value_enum)]
        scope: Option<ComplyScopeArg>,

        /// Exit with error if non-compliant (grade F)
        #[arg(long)]
        strict: bool,

        /// Show only non-compliant artifacts
        #[arg(long)]
        failures_only: bool,

        /// Minimum acceptable score (exit non-zero if below)
        #[arg(long)]
        min_score: Option<u32>,

        /// Output format
        #[arg(short, long, value_enum, default_value = "text")]
        format: ComplyFormat,
    },

    /// Show current compliance status (alias for check)
    Status {
        /// Project path
        #[arg(short, long, default_value = ".")]
        path: PathBuf,

        /// Output format
        #[arg(short, long, value_enum, default_value = "text")]
        format: ComplyFormat,
    },

    /// Manage tracked artifacts
    Track {
        #[command(subcommand)]
        command: ComplyTrackCommands,
    },

    /// List all compliance rules with descriptions and weights
    Rules {
        /// Output format
        #[arg(short, long, value_enum, default_value = "text")]
        format: ComplyFormat,
    },
}

/// Track subcommands
#[derive(Subcommand)]
pub enum ComplyTrackCommands {
    /// Auto-discover artifacts in project
    Discover {
        /// Project path
        #[arg(short, long, default_value = ".")]
        path: PathBuf,

        /// Scope to discover
        #[arg(long, value_enum, default_value = "project")]
        scope: ComplyScopeArg,
    },

    /// List tracked artifacts
    List {
        /// Project path
        #[arg(short, long, default_value = ".")]
        path: PathBuf,

        /// Scope to list
        #[arg(long, value_enum)]
        scope: Option<ComplyScopeArg>,
    },
}

/// Scope argument for comply commands
#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum ComplyScopeArg {
    /// Project artifacts (*.sh, Makefile, Dockerfile)
    #[default]
    Project,
    /// User config files (~/.zshrc, ~/.bashrc)
    User,
    /// System config files (/etc/profile, read-only)
    System,
    /// All scopes
    All,
}

/// Output format for comply commands
#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum ComplyFormat {
    /// Human-readable text
    #[default]
    Text,
    /// JSON format for CI/CD
    Json,
    /// Markdown report
    Markdown,
}

#[derive(Subcommand)]
pub enum MakeCommands {
    /// Transpile Rust DSL to Makefile
    Build {
        /// Input Rust file with Makefile DSL
        #[arg(value_name = "FILE")]
        input: PathBuf,

        /// Output Makefile path
        #[arg(short, long, default_value = "Makefile")]
        output: PathBuf,
    },

    /// Parse Makefile to AST
    Parse {
        /// Input Makefile
        #[arg(value_name = "FILE")]
        input: PathBuf,

        /// Output format
        #[arg(long, value_enum, default_value = "text")]
        format: MakeOutputFormat,
    },

    /// Purify Makefile (determinism + idempotency)
    Purify {
        /// Input Makefile
        #[arg(value_name = "FILE")]
        input: PathBuf,

        /// Output file (defaults to stdout or in-place with --fix)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Apply fixes in-place (creates .bak backup)
        #[arg(long)]
        fix: bool,

        /// Show detailed transformation report
        #[arg(long)]
        report: bool,

        /// Report format
        #[arg(long, value_enum, default_value = "human")]
        format: ReportFormat,

        /// Generate test suite for purified Makefile
        #[arg(long)]
        with_tests: bool,

        /// Generate property-based tests (100+ cases)
        #[arg(long)]
        property_tests: bool,

        /// Preserve formatting (keep blank lines, multi-line format)
        #[arg(long)]
        preserve_formatting: bool,

        /// Maximum line length (default: unlimited)
        #[arg(long)]
        max_line_length: Option<usize>,

        /// Skip blank line removal transformation
        #[arg(long)]
        skip_blank_line_removal: bool,

        /// Skip multi-line consolidation transformation
        #[arg(long)]
        skip_consolidation: bool,
    },

    /// Lint Makefile for quality issues
    Lint {
        /// Input Makefile
        #[arg(value_name = "FILE")]
        input: PathBuf,

        /// Output format
        #[arg(long, value_enum, default_value = "human")]
        format: LintFormat,

        /// Apply automatic fixes
        #[arg(long)]
        fix: bool,

        /// Output file (defaults to in-place with --fix)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Filter by specific rules (comma-separated: MAKE001,MAKE003)
        #[arg(long)]
        rules: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum DockerfileCommands {
    /// Transpile Rust DSL to Dockerfile
    Build {
        /// Input Rust file with Dockerfile DSL
        #[arg(value_name = "FILE")]
        input: PathBuf,

        /// Output Dockerfile path
        #[arg(short, long, default_value = "Dockerfile")]
        output: PathBuf,

        /// Base image (e.g., "rust:1.75-alpine")
        #[arg(long)]
        base_image: Option<String>,
    },

    /// Purify Dockerfile (auto-fix security and best practices issues)
    Purify {
        /// Input Dockerfile
        #[arg(value_name = "FILE")]
        input: PathBuf,

        /// Output file (defaults to stdout or in-place with --fix)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Apply fixes in-place (creates .bak backup)
        #[arg(long)]
        fix: bool,

        /// Don't create backup with --fix (dangerous!)
        #[arg(long)]
        no_backup: bool,

        /// Show changes without applying (dry-run mode)
        #[arg(long)]
        dry_run: bool,

        /// Show detailed transformation report
        #[arg(long)]
        report: bool,

        /// Report format
        #[arg(long, value_enum, default_value = "human")]
        format: ReportFormat,

        /// Don't add USER directive (for special cases)
        #[arg(long)]
        skip_user: bool,

        /// Don't purify bash in RUN commands
        #[arg(long)]
        skip_bash_purify: bool,
    },

    /// Lint Dockerfile for issues (existing functionality)
    Lint {
        /// Input Dockerfile
        #[arg(value_name = "FILE")]
        input: PathBuf,

        /// Output format
        #[arg(long, value_enum, default_value = "human")]
        format: LintFormat,

        /// Filter by specific rules (comma-separated: DOCKER001,DOCKER003)
        #[arg(long)]
        rules: Option<String>,
    },

    /// Profile Docker image runtime performance (requires Docker daemon)
    Profile {
        /// Input Dockerfile
        #[arg(value_name = "FILE")]
        input: PathBuf,

        /// Measure build time and layer cache efficiency
        #[arg(long)]
        build: bool,

        /// Show layer-by-layer timing analysis
        #[arg(long)]
        layers: bool,

        /// Measure container startup time to healthy state
        #[arg(long)]
        startup: bool,

        /// Measure container memory usage during runtime
        #[arg(long)]
        memory: bool,

        /// Measure container CPU usage during runtime
        #[arg(long)]
        cpu: bool,

        /// Run custom workload script for profiling
        #[arg(long, value_name = "SCRIPT")]
        workload: Option<PathBuf>,

        /// Duration for runtime profiling (e.g., "30s", "1m")
        #[arg(long, default_value = "30s")]
        duration: String,

        /// Apply platform-specific constraints (coursera)
        #[arg(long, value_enum)]
        profile: Option<LintProfileArg>,

        /// Simulate platform resource limits during profiling
        #[arg(long)]
        simulate_limits: bool,

        /// Run full runtime validation suite
        #[arg(long)]
        full: bool,

        /// Output format
        #[arg(long, value_enum, default_value = "human")]
        format: ReportFormat,
    },

    /// Check Docker image size and detect bloat patterns
    SizeCheck {
        /// Input Dockerfile
        #[arg(value_name = "FILE")]
        input: PathBuf,

        /// Show verbose size breakdown by layer
        #[arg(long)]
        verbose: bool,

        /// Show per-layer size analysis
        #[arg(long)]
        layers: bool,

        /// Detect common size bloat patterns
        #[arg(long)]
        detect_bloat: bool,

        /// Verify estimate against actual built image
        #[arg(long)]
        verify: bool,

        /// Build image and verify size (requires Docker)
        #[arg(long)]
        docker_verify: bool,

        /// Apply platform-specific size constraints (coursera = 10GB)
        #[arg(long, value_enum)]
        profile: Option<LintProfileArg>,

        /// Exit with error if estimated size exceeds limit
        #[arg(long)]
        strict: bool,

        /// Custom maximum size limit (e.g., "5GB", "500MB")
        #[arg(long, value_name = "SIZE")]
        max_size: Option<String>,

        /// Show compression opportunities
        #[arg(long)]
        compression_analysis: bool,

        /// Output format
        #[arg(long, value_enum, default_value = "human")]
        format: ReportFormat,
    },

    /// Run full validation pipeline (lint + size + optional runtime profiling)
    FullValidate {
        /// Input Dockerfile
        #[arg(value_name = "FILE")]
        input: PathBuf,

        /// Apply platform-specific validation (coursera)
        #[arg(long, value_enum)]
        profile: Option<LintProfileArg>,

        /// Include size verification
        #[arg(long)]
        size_check: bool,

        /// Include graded lab validation (for Coursera)
        #[arg(long)]
        graded: bool,

        /// Include runtime profiling (requires Docker daemon)
        #[arg(long)]
        runtime: bool,

        /// Exit with error on any warning
        #[arg(long)]
        strict: bool,

        /// Output format
        #[arg(long, value_enum, default_value = "human")]
        format: ReportFormat,
    },
}

#[derive(Subcommand)]
pub enum DevContainerCommands {
    /// Validate devcontainer.json file (JSONC support)
    Validate {
        /// Path to devcontainer.json or directory containing .devcontainer
        #[arg(value_name = "PATH")]
        path: PathBuf,

        /// Output format
        #[arg(long, value_enum, default_value = "human")]
        format: LintFormat,

        /// Also lint referenced Dockerfile (if build.dockerfile specified)
        #[arg(long)]
        lint_dockerfile: bool,

        /// List all available DEVCONTAINER rules
        #[arg(long)]
        list_rules: bool,
    },
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Analyze shell configuration file for issues
    Analyze {
        /// Input config file (.bashrc, .zshrc, .profile, etc.)
        #[arg(value_name = "FILE")]
        input: PathBuf,

        /// Output format
        #[arg(long, value_enum, default_value = "human")]
        format: ConfigOutputFormat,
    },

    /// Lint shell configuration file
    Lint {
        /// Input config file
        #[arg(value_name = "FILE")]
        input: PathBuf,

        /// Output format
        #[arg(long, value_enum, default_value = "human")]
        format: ConfigOutputFormat,
    },

    /// Purify shell configuration file (fix issues automatically)
    Purify {
        /// Input config file
        #[arg(value_name = "FILE")]
        input: PathBuf,

        /// Output file (defaults to stdout, or in-place with --fix)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Apply fixes in-place (creates timestamped backup)
        #[arg(long)]
        fix: bool,

        /// Don't create backup (dangerous!)
        #[arg(long)]
        no_backup: bool,

        /// Dry run (show what would be changed)
        #[arg(long)]
        dry_run: bool,
    },
}

/// Installer subcommands (NEW in v7.0 - Issue #104)
#[derive(Subcommand)]
pub enum InstallerCommands {
    /// Initialize new installer project with TDD-first test harness
    Init {
        /// Project name/directory
        #[arg(value_name = "NAME")]
        name: PathBuf,

        /// Project description
        #[arg(long)]
        description: Option<String>,
    },

    /// Convert bash script to installer.toml format
    FromBash {
        /// Input bash script
        #[arg(value_name = "FILE")]
        input: PathBuf,

        /// Output directory
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Run installer (with optional resume, dry-run, etc.)
    Run {
        /// Installer directory or installer.toml path
        #[arg(value_name = "PATH")]
        path: PathBuf,

        /// Checkpoint directory for resuming
        #[arg(long)]
        checkpoint_dir: Option<PathBuf>,

        /// Dry-run without making changes
        #[arg(long)]
        dry_run: bool,

        /// Show unified diff of changes
        #[arg(long)]
        diff: bool,

        /// Enable hermetic mode (reproducible builds)
        #[arg(long)]
        hermetic: bool,

        /// Verify artifact signatures
        #[arg(long)]
        verify_signatures: bool,

        /// Enable parallel execution
        #[arg(long)]
        parallel: bool,

        /// Enable OpenTelemetry tracing
        #[arg(long)]
        trace: bool,

        /// Export traces to file (JSON format)
        #[arg(long)]
        trace_file: Option<PathBuf>,
    },

    /// Resume installer from checkpoint
    Resume {
        /// Installer directory
        #[arg(value_name = "PATH")]
        path: PathBuf,

        /// Step to resume from
        #[arg(long)]
        from: Option<String>,
    },

    /// Validate installer without executing
    Validate {
        /// Installer directory or installer.toml path
        #[arg(value_name = "PATH")]
        path: PathBuf,
    },

    /// Run installer test suite
    Test {
        /// Installer directory
        #[arg(value_name = "PATH")]
        path: PathBuf,

        /// Test matrix (platforms to test, comma-separated)
        #[arg(long)]
        matrix: Option<String>,

        /// Enable coverage reporting
        #[arg(long)]
        coverage: bool,
    },

    /// Generate lockfile for hermetic builds
    Lock {
        /// Installer directory
        #[arg(value_name = "PATH")]
        path: PathBuf,

        /// Update existing lockfile
        #[arg(long)]
        update: bool,

        /// Verify lockfile matches current state
        #[arg(long)]
        verify: bool,
    },

    /// Visualize installer build graph
    Graph {
        /// Installer directory
        #[arg(value_name = "PATH")]
        path: PathBuf,

        /// Output format (mermaid, dot, json)
        #[arg(long, value_enum, default_value = "mermaid")]
        format: InstallerGraphFormat,
    },

    /// Capture golden trace baseline
    GoldenCapture {
        /// Installer directory
        #[arg(value_name = "PATH")]
        path: PathBuf,

        /// Trace name
        #[arg(long)]
        trace: String,
    },

    /// Compare execution against golden trace
    GoldenCompare {
        /// Installer directory
        #[arg(value_name = "PATH")]
        path: PathBuf,

        /// Trace name to compare against
        #[arg(long)]
        trace: String,
    },

    /// Audit installer for security, quality, and best practices
    Audit {
        /// Installer directory or installer.toml path
        #[arg(value_name = "PATH")]
        path: PathBuf,

        /// Output format (human, json)
        #[arg(long, value_enum, default_value = "human")]
        format: AuditOutputFormat,

        /// Security-only audit
        #[arg(long)]
        security_only: bool,

        /// Minimum severity to report (info, suggestion, warning, error, critical)
        #[arg(long)]
        min_severity: Option<String>,

        /// Issue #110: Ignore specific rules (can be specified multiple times)
        /// Example: --ignore SEC001 --ignore QUAL002
        #[arg(long, value_name = "RULE")]
        ignore: Vec<String>,
    },

    /// Initialize or manage keyring for artifact verification
    Keyring {
        #[command(subcommand)]
        command: KeyringCommands,
    },
}

/// Keyring management subcommands
#[derive(Subcommand)]
pub enum KeyringCommands {
    /// Initialize a new keyring
    Init {
        /// Import keys from files
        #[arg(long, action = clap::ArgAction::Append)]
        import: Vec<PathBuf>,
    },

    /// Add a key to the keyring
    Add {
        /// Key file to add
        #[arg(value_name = "FILE")]
        key: PathBuf,

        /// Key ID
        #[arg(long)]
        id: String,
    },

    /// List keys in the keyring
    List,

    /// Remove a key from the keyring
    Remove {
        /// Key ID to remove
        #[arg(value_name = "ID")]
        id: String,
    },
}

/// Output format for installer graph command
#[derive(Clone, Debug, Default, ValueEnum)]
pub enum InstallerGraphFormat {
    /// Mermaid diagram
    #[default]
    Mermaid,
    /// Graphviz DOT format
    Dot,
    /// JSON format
    Json,
}

/// Output format for config commands
#[derive(Clone, Debug, ValueEnum)]
pub enum ConfigOutputFormat {
    /// Human-readable format
    Human,
    /// JSON format
    Json,
}

/// Runtime options for compilation
#[derive(Clone, Debug, ValueEnum)]
pub enum CompileRuntime {
    /// Dash shell (180KB)
    Dash,
    /// Busybox (900KB)
    Busybox,
    /// Minimal interpreter (50KB)
    Minimal,
}

/// Container format options
#[derive(Clone, Debug, ValueEnum)]
pub enum ContainerFormatArg {
    /// OCI format
    Oci,
    /// Docker format
    Docker,
}

/// Output format for inspection reports
#[derive(Clone, Debug, ValueEnum)]
pub enum InspectionFormat {
    /// Markdown report
    Markdown,
    /// JSON report
    Json,
    /// HTML report
    Html,
}

/// Output format for lint results
#[derive(Clone, Debug, ValueEnum)]
pub enum LintFormat {
    /// Human-readable format
    Human,
    /// JSON format
    Json,
    /// SARIF format
    Sarif,
}

/// Output format for Makefile parse results
#[derive(Clone, Debug, ValueEnum)]
pub enum MakeOutputFormat {
    /// Human-readable text
    Text,
    /// JSON AST
    Json,
    /// Debug format
    Debug,
}

/// Output format for purification reports
#[derive(Clone, Debug, ValueEnum)]
pub enum ReportFormat {
    /// Human-readable report
    Human,
    /// JSON format
    Json,
    /// Markdown format
    Markdown,
}

/// Output format for test results
#[derive(Clone, Debug, ValueEnum)]
pub enum TestOutputFormat {
    /// Human-readable format
    Human,
    /// JSON format
    Json,
    /// JUnit XML format
    Junit,
}

/// Output format for score results
#[derive(Clone, Debug, ValueEnum)]
pub enum ScoreOutputFormat {
    /// Human-readable format
    Human,
    /// JSON format
    Json,
    /// Markdown report
    Markdown,
}

/// Output format for audit results
#[derive(Clone, Debug, ValueEnum)]
pub enum AuditOutputFormat {
    /// Human-readable format
    Human,
    /// JSON format
    Json,
    /// SARIF format (for GitHub Code Scanning)
    Sarif,
}

/// Output format for coverage results
#[derive(Clone, Debug, ValueEnum)]
pub enum CoverageOutputFormat {
    /// Terminal output with colors
    Terminal,
    /// JSON format
    Json,
    /// HTML report
    Html,
    /// LCOV format
    Lcov,
}

/// Minimum severity level for lint output (Issue #75)
#[derive(Clone, Copy, Debug, Default, ValueEnum, PartialEq, Eq, PartialOrd, Ord)]
pub enum LintLevel {
    /// Show info, warning, and error messages
    #[default]
    Info,
    /// Show only warning and error messages
    Warning,
    /// Show only error messages
    Error,
}

/// Lint profile for specialized validation rules
#[derive(Clone, Copy, Debug, Default, ValueEnum, PartialEq, Eq)]
pub enum LintProfileArg {
    /// Standard Dockerfile linting (default)
    #[default]
    Standard,
    /// Coursera Labs image validation (single port, 10GB limit, HEALTHCHECK required)
    Coursera,
    /// Dev Container validation (devcontainer.json + Dockerfile compatibility)
    DevContainer,
}

impl ValueEnum for VerificationLevel {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            VerificationLevel::None,
            VerificationLevel::Basic,
            VerificationLevel::Strict,
            VerificationLevel::Paranoid,
        ]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        Some(match self {
            VerificationLevel::None => clap::builder::PossibleValue::new("none"),
            VerificationLevel::Basic => clap::builder::PossibleValue::new("basic"),
            VerificationLevel::Strict => clap::builder::PossibleValue::new("strict"),
            VerificationLevel::Paranoid => clap::builder::PossibleValue::new("paranoid"),
        })
    }
}

impl ValueEnum for ShellDialect {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            ShellDialect::Posix,
            ShellDialect::Bash,
            ShellDialect::Dash,
            ShellDialect::Ash,
        ]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        Some(match self {
            ShellDialect::Posix => clap::builder::PossibleValue::new("posix"),
            ShellDialect::Bash => clap::builder::PossibleValue::new("bash"),
            ShellDialect::Dash => clap::builder::PossibleValue::new("dash"),
            ShellDialect::Ash => clap::builder::PossibleValue::new("ash"),
        })
    }
}

impl ValueEnum for ValidationLevel {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            ValidationLevel::None,
            ValidationLevel::Minimal,
            ValidationLevel::Strict,
            ValidationLevel::Paranoid,
        ]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        Some(match self {
            ValidationLevel::None => clap::builder::PossibleValue::new("none"),
            ValidationLevel::Minimal => clap::builder::PossibleValue::new("minimal"),
            ValidationLevel::Strict => clap::builder::PossibleValue::new("strict"),
            ValidationLevel::Paranoid => clap::builder::PossibleValue::new("paranoid"),
        })
    }
}
