use std::path::PathBuf;
use clap::{Parser, Subcommand, ValueEnum};
use crate::models::{VerificationLevel, ShellDialect};

#[derive(Parser)]
#[command(name = "rash")]
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