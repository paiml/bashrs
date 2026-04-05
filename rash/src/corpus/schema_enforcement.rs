//! # Formal Schema Enforcement for Output Formats (§11.8)
//!
//! Implements grammar validation layers (L1-L4) for each target format:
//! - POSIX Shell: IEEE Std 1003.1-2017, Section 2
//! - GNU Make: GNU Make Manual 4.4, Section 3.7
//! - Dockerfile: Docker Engine v25+ reference
//!
//! Grammar violations are classified as GRAM-001..GRAM-008 categories
//! following the taxonomy in §11.8.5.

use crate::corpus::registry::{CorpusEntry, CorpusFormat, CorpusRegistry};
use std::fmt;

/// Grammar violation category (§11.8.5)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GrammarCategory {
    /// GRAM-001: Missing quoting in expansion
    MissingQuoting,
    /// GRAM-002: Bashism in POSIX output
    Bashism,
    /// GRAM-003: Tab/space confusion in Makefile
    TabSpaceConfusion,
    /// GRAM-004: Shell form in Dockerfile CMD
    ShellFormCmd,
    /// GRAM-005: Undefined variable reference
    UndefinedVariable,
    /// GRAM-006: Invalid POSIX arithmetic
    InvalidArithmetic,
    /// GRAM-007: Missing FROM in Dockerfile
    MissingFrom,
    /// GRAM-008: Circular Make dependency
    CircularDependency,
}

impl GrammarCategory {
    /// Return the GRAM-NNN code
    pub fn code(&self) -> &'static str {
        match self {
            Self::MissingQuoting => "GRAM-001",
            Self::Bashism => "GRAM-002",
            Self::TabSpaceConfusion => "GRAM-003",
            Self::ShellFormCmd => "GRAM-004",
            Self::UndefinedVariable => "GRAM-005",
            Self::InvalidArithmetic => "GRAM-006",
            Self::MissingFrom => "GRAM-007",
            Self::CircularDependency => "GRAM-008",
        }
    }

    /// Human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            Self::MissingQuoting => "Missing quoting in expansion",
            Self::Bashism => "Bashism in POSIX output",
            Self::TabSpaceConfusion => "Tab/space confusion in Makefile recipe",
            Self::ShellFormCmd => "Shell form in Dockerfile CMD/ENTRYPOINT",
            Self::UndefinedVariable => "Undefined variable reference",
            Self::InvalidArithmetic => "Invalid POSIX arithmetic",
            Self::MissingFrom => "Missing FROM in Dockerfile",
            Self::CircularDependency => "Circular Make dependency",
        }
    }

    /// Suggested fix pattern
    pub fn fix_pattern(&self) -> &'static str {
        match self {
            Self::MissingQuoting => "Add double quotes around ${}",
            Self::Bashism => "Replace [[ ]] with [ ]",
            Self::TabSpaceConfusion => "Ensure recipe lines use \\t",
            Self::ShellFormCmd => "Convert to exec form [\"cmd\", \"arg\"]",
            Self::UndefinedVariable => "Add := assignment before use",
            Self::InvalidArithmetic => "Replace (( )) with $(( ))",
            Self::MissingFrom => "Add FROM as first instruction",
            Self::CircularDependency => "Reorder targets to break cycle",
        }
    }

    /// Which format this violation applies to
    pub fn applicable_format(&self) -> CorpusFormat {
        match self {
            Self::MissingQuoting | Self::Bashism | Self::InvalidArithmetic => CorpusFormat::Bash,
            Self::TabSpaceConfusion | Self::UndefinedVariable | Self::CircularDependency => {
                CorpusFormat::Makefile
            }
            Self::ShellFormCmd | Self::MissingFrom => CorpusFormat::Dockerfile,
        }
    }

    /// All categories
    pub fn all() -> &'static [GrammarCategory] {
        &[
            Self::MissingQuoting,
            Self::Bashism,
            Self::TabSpaceConfusion,
            Self::ShellFormCmd,
            Self::UndefinedVariable,
            Self::InvalidArithmetic,
            Self::MissingFrom,
            Self::CircularDependency,
        ]
    }
}

impl fmt::Display for GrammarCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code())
    }
}

/// Validation layer (L1-L4 from §11.8)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationLayer {
    /// L1: Lexical — token stream is valid
    Lexical,
    /// L2: Syntactic — grammar compliance
    Syntactic,
    /// L3: Semantic — security/determinism/idempotency rules
    Semantic,
    /// L4: Behavioral — runtime equivalence across shells
    Behavioral,
}

impl fmt::Display for ValidationLayer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lexical => write!(f, "L1:Lexical"),
            Self::Syntactic => write!(f, "L2:Syntactic"),
            Self::Semantic => write!(f, "L3:Semantic"),
            Self::Behavioral => write!(f, "L4:Behavioral"),
        }
    }
}

/// A grammar violation found during schema validation
#[derive(Debug, Clone)]
pub struct GrammarViolation {
    pub category: GrammarCategory,
    pub layer: ValidationLayer,
    pub entry_id: String,
    pub line: usize,
    pub message: String,
}

/// Result of schema validation for a single entry
#[derive(Debug, Clone)]
pub struct SchemaResult {
    pub entry_id: String,
    pub format: CorpusFormat,
    pub valid: bool,
    pub violations: Vec<GrammarViolation>,
    pub layers_passed: Vec<ValidationLayer>,
}

/// Aggregate schema validation report
#[derive(Debug, Clone)]
pub struct SchemaReport {
    pub results: Vec<SchemaResult>,
    pub total_entries: usize,
    pub valid_entries: usize,
    pub total_violations: usize,
    pub violations_by_category: Vec<(GrammarCategory, usize)>,
}

impl SchemaReport {
    /// Percentage of entries that pass schema validation
    pub fn pass_rate(&self) -> f64 {
        if self.total_entries == 0 {
            return 0.0;
        }
        (self.valid_entries as f64 / self.total_entries as f64) * 100.0
    }
}

/// Validate a single corpus entry against its format's grammar (§11.8.1-11.8.3)
pub fn validate_entry(entry: &CorpusEntry) -> SchemaResult {
    let mut violations = Vec::new();
    let mut layers_passed = Vec::new();

    match entry.format {
        CorpusFormat::Bash => validate_bash_entry(entry, &mut violations, &mut layers_passed),
        CorpusFormat::Makefile => {
            validate_makefile_entry(entry, &mut violations, &mut layers_passed);
        }
        CorpusFormat::Dockerfile => {
            validate_dockerfile_entry(entry, &mut violations, &mut layers_passed);
        }
    }

    SchemaResult {
        entry_id: entry.id.clone(),
        format: entry.format,
        valid: violations.is_empty(),
        violations,
        layers_passed,
    }
}

/// Validate POSIX shell output (§11.8.1)
fn validate_bash_entry(
    entry: &CorpusEntry,
    violations: &mut Vec<GrammarViolation>,
    layers_passed: &mut Vec<ValidationLayer>,
) {
    let output = &entry.expected_output;

    // L1: Lexical — basic token validity
    let l1_pass = !output.is_empty();
    if l1_pass {
        layers_passed.push(ValidationLayer::Lexical);
    }

    // L2: Syntactic — check for bashisms in POSIX output
    for (i, line) in output.lines().enumerate() {
        let trimmed = line.trim();

        // GRAM-002: [[ ]] bashism
        if trimmed.contains("[[") && trimmed.contains("]]") {
            violations.push(GrammarViolation {
                category: GrammarCategory::Bashism,
                layer: ValidationLayer::Syntactic,
                entry_id: entry.id.clone(),
                line: i + 1,
                message: "Double bracket [[ ]] is a bashism; use [ ] for POSIX".into(),
            });
        }

        // GRAM-006: bash-specific (( )) arithmetic (not $((  )))
        if trimmed.contains("(( ") && !trimmed.contains("$((") {
            violations.push(GrammarViolation {
                category: GrammarCategory::InvalidArithmetic,
                layer: ValidationLayer::Syntactic,
                entry_id: entry.id.clone(),
                line: i + 1,
                message: "(( )) is bash-specific; use $(( )) for POSIX arithmetic".into(),
            });
        }
    }

    // GRAM-001: Unquoted variable expansions (simple heuristic)
    for (i, line) in output.lines().enumerate() {
        let trimmed = line.trim();
        // Skip comments, shebangs, and assignments
        if trimmed.starts_with('#') || trimmed.starts_with("#!/") {
            continue;
        }
        // Look for unquoted $VAR or ${VAR} in arguments (not in assignments)
        if check_unquoted_expansion(trimmed) {
            violations.push(GrammarViolation {
                category: GrammarCategory::MissingQuoting,
                layer: ValidationLayer::Semantic,
                entry_id: entry.id.clone(),
                line: i + 1,
                message: "Unquoted variable expansion; wrap in double quotes".into(),
            });
        }
    }

    if violations
        .iter()
        .all(|v| v.layer != ValidationLayer::Syntactic)
    {
        layers_passed.push(ValidationLayer::Syntactic);
    }
    if violations
        .iter()
        .all(|v| v.layer != ValidationLayer::Semantic)
    {
        layers_passed.push(ValidationLayer::Semantic);
    }
}

/// Check for unquoted expansions in a shell line (heuristic)
/// Check if a line is a simple shell assignment (VAR=value)
fn is_shell_assignment(line: &str) -> bool {
    line.find('=').is_some_and(|eq_pos| {
        line[..eq_pos]
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_')
    })
}

/// Check if a `$` at position `i` is an unquoted variable expansion

include!("schema_enforcement_incl2.rs");
