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

    /// Launch interactive playground REPL
    #[cfg(feature = "playground")]
    Playground {
        /// Initial file to load
        #[arg(short, long)]
        file: Option<PathBuf>,

        /// Session state URL to restore
        #[arg(long)]
        restore: Option<String>,

        /// Disable VI-style key bindings
        #[arg(long)]
        no_vi: bool,
    },

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
