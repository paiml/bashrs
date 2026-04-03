use super::args::{AuditOutputFormat, LintFormat, LintProfileArg, MakeOutputFormat, ReportFormat};
use clap::{Subcommand, ValueEnum};
use std::path::PathBuf;

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

    /// Generate compliance report (Phase 2)
    Report {
        /// Project path
        #[arg(short, long, default_value = ".")]
        path: PathBuf,

        /// Output format
        #[arg(short, long, value_enum, default_value = "markdown")]
        format: ComplyFormat,

        /// Write output to file
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Scope to report
        #[arg(long, value_enum)]
        scope: Option<ComplyScopeArg>,
    },

    /// Install pre-commit compliance hooks (Phase 2)
    Enforce {
        /// Enforcement tier (1=fast, 2=standard, 3=strict)
        #[arg(long, default_value = "1")]
        tier: u8,

        /// Remove enforcement hooks
        #[arg(long)]
        uninstall: bool,
    },

    /// Show compliance delta since last check (Phase 2)
    Diff {
        /// Project path
        #[arg(short, long, default_value = ".")]
        path: PathBuf,

        /// Compare against last comply check
        #[arg(long)]
        since_last: bool,
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

/// Output format for config commands
#[derive(Clone, Debug, ValueEnum)]
pub enum ConfigOutputFormat {
    /// Human-readable format
    Human,
    /// JSON format
    Json,
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
