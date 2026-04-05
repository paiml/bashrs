/// Output format for cfg command (Sprint 5)
#[derive(Clone, Debug, Default, ValueEnum)]
pub enum CfgOutputFormat {
    #[default]
    Human,
    Json,
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

/// Script format for classify command (SSC-022)
#[derive(Clone, Debug, ValueEnum)]
pub enum ClassifyFormat {
    /// Bash / shell script
    Bash,
    /// Makefile (GNU Make)
    Makefile,
    /// Dockerfile
    Dockerfile,
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
#[derive(Clone, Copy, Debug, ValueEnum)]
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
