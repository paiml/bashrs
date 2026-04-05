
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
