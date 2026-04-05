use clap::Subcommand;
use std::path::PathBuf;

use super::args::{
    CfgOutputFormat, ClassifyFormat, MutateFormat, PlaybookFormat, SimulateFormat,
};

#[cfg(feature = "oracle")]
use super::args::ExplainErrorFormat;

use super::args::InstallerCommands;

#[derive(Subcommand)]
pub enum CommandsExt {
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

    /// Control flow graph analysis for bash scripts (Sprint 5: Formal CFG)
    Cfg {
        /// Input bash script
        #[arg(value_name = "FILE")]
        input: PathBuf,
        /// Output format
        #[arg(long, value_enum, default_value = "human")]
        format: CfgOutputFormat,
        /// Show per-function CFG breakdown
        #[arg(long)]
        per_function: bool,
    },

    /// Generate adversarial training data for shell safety classifier
    GenerateAdversarial {
        /// Output JSONL file path
        #[arg(short, long, default_value = "adversarial.jsonl")]
        output: PathBuf,

        /// RNG seed for reproducible generation
        #[arg(long, default_value = "42")]
        seed: u64,

        /// Number of samples per minority class (classes 2, 3, 4)
        #[arg(long, default_value = "2500")]
        count_per_class: usize,

        /// Extra needs-quoting (class 1) samples
        #[arg(long, default_value = "500")]
        extra_needs_quoting: usize,

        /// Verify each script against derive_safety_label for self-consistency
        #[arg(long)]
        verify: bool,

        /// Show generation statistics
        #[arg(long)]
        stats: bool,
    },
}
