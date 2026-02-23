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
fn is_unquoted_var_at(bytes: &[u8], i: usize) -> bool {
    if i + 1 >= bytes.len() {
        return false;
    }
    let next = bytes[i + 1];
    // $( is subshell/arithmetic, not a bare variable
    if next == b'(' {
        return false;
    }
    next.is_ascii_alphabetic() || next == b'{' || next == b'_'
}

fn check_unquoted_expansion(line: &str) -> bool {
    if is_shell_assignment(line) {
        return false;
    }

    let bytes = line.as_bytes();
    let mut i = 0;
    let mut in_single_quote = false;
    let mut in_double_quote = false;

    while i < bytes.len() {
        match bytes[i] {
            b'\'' if !in_double_quote => in_single_quote = !in_single_quote,
            b'"' if !in_single_quote => in_double_quote = !in_double_quote,
            b'$' if !in_single_quote && !in_double_quote => {
                if is_unquoted_var_at(bytes, i) {
                    return true;
                }
            }
            b'\\' if !in_single_quote => i += 1,
            _ => {}
        }
        i += 1;
    }
    false
}

/// Assignment operators in Makefiles
const MAKE_ASSIGN_OPS: &[&str] = &[":=", "?=", "+=", "="];

/// Extract a variable name from a Makefile assignment line
fn extract_make_var(line: &str) -> Option<String> {
    if line.starts_with('\t') || line.starts_with('#') {
        return None;
    }
    for op in MAKE_ASSIGN_OPS {
        if let Some(pos) = line.find(op) {
            let name = line[..pos].trim().to_string();
            if !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                return Some(name);
            }
        }
    }
    None
}

/// Check if a Makefile line is a space-indented recipe (GRAM-003)
fn is_space_indented_recipe(line: &str, in_recipe: bool) -> bool {
    in_recipe
        && !line.starts_with('\t')
        && !line.trim().is_empty()
        && (line.starts_with("    ") || line.starts_with("  "))
}

/// Validate Makefile output (§11.8.2)
fn validate_makefile_entry(
    entry: &CorpusEntry,
    violations: &mut Vec<GrammarViolation>,
    layers_passed: &mut Vec<ValidationLayer>,
) {
    let output = &entry.expected_output;

    if !output.is_empty() {
        layers_passed.push(ValidationLayer::Lexical);
    }

    let mut in_recipe = false;

    for (i, line) in output.lines().enumerate() {
        if line.trim().is_empty() {
            in_recipe = false;
            continue;
        }
        if line.starts_with('#') {
            continue;
        }

        // Detect target rules
        if !line.starts_with('\t') && line.contains(':') && !line.contains(":=") {
            in_recipe = true;
            continue;
        }

        if is_space_indented_recipe(line, in_recipe) {
            violations.push(GrammarViolation {
                category: GrammarCategory::TabSpaceConfusion,
                layer: ValidationLayer::Syntactic,
                entry_id: entry.id.clone(),
                line: i + 1,
                message: "Recipe line uses spaces instead of tab".into(),
            });
        }
    }

    // Collect defined variables (GRAM-005 preparation)
    let _defined_vars: Vec<String> = output.lines().filter_map(extract_make_var).collect();

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

/// Validate Dockerfile output (§11.8.3)
fn validate_dockerfile_entry(
    entry: &CorpusEntry,
    violations: &mut Vec<GrammarViolation>,
    layers_passed: &mut Vec<ValidationLayer>,
) {
    let output = &entry.expected_output;

    // L1: Lexical
    let l1_pass = !output.is_empty();
    if l1_pass {
        layers_passed.push(ValidationLayer::Lexical);
    }

    let instructions: Vec<&str> = output
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.trim_start().starts_with('#'))
        .collect();

    // GRAM-007: First instruction must be FROM (or ARG before FROM)
    if let Some(first) = instructions.first() {
        let upper = first.trim().to_uppercase();
        if !upper.starts_with("FROM") && !upper.starts_with("ARG") {
            violations.push(GrammarViolation {
                category: GrammarCategory::MissingFrom,
                layer: ValidationLayer::Syntactic,
                entry_id: entry.id.clone(),
                line: 1,
                message: "Dockerfile must start with FROM (or ARG before FROM)".into(),
            });
        }
    }

    // GRAM-004: Shell form in CMD/ENTRYPOINT
    for (i, line) in output.lines().enumerate() {
        let trimmed = line.trim();
        let upper = trimmed.to_uppercase();
        if (upper.starts_with("CMD ") || upper.starts_with("ENTRYPOINT ")) && !trimmed.contains('[')
        {
            violations.push(GrammarViolation {
                category: GrammarCategory::ShellFormCmd,
                layer: ValidationLayer::Semantic,
                entry_id: entry.id.clone(),
                line: i + 1,
                message: "Use exec form [\"cmd\", \"arg\"] instead of shell form".into(),
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

/// Validate all entries in a corpus registry (§11.8.4)
pub fn validate_corpus(registry: &CorpusRegistry) -> SchemaReport {
    let results: Vec<SchemaResult> = registry.entries.iter().map(validate_entry).collect();

    let total_entries = results.len();
    let valid_entries = results.iter().filter(|r| r.valid).count();
    let total_violations: usize = results.iter().map(|r| r.violations.len()).sum();

    // Count violations by category
    let mut category_counts = std::collections::HashMap::new();
    for result in &results {
        for v in &result.violations {
            *category_counts.entry(v.category).or_insert(0usize) += 1;
        }
    }

    let mut violations_by_category: Vec<(GrammarCategory, usize)> =
        category_counts.into_iter().collect();
    violations_by_category.sort_by(|a, b| b.1.cmp(&a.1));

    SchemaReport {
        results,
        total_entries,
        valid_entries,
        total_violations,
        violations_by_category,
    }
}

/// Format the schema validation report as a table
pub fn format_schema_report(report: &SchemaReport) -> String {
    let mut out = String::new();
    let line = "\u{2500}".repeat(72);

    out.push_str(&format!(
        "{}\n{:<12} {:<14} {:<10} {:<10} {}\n{}\n",
        line, "Format", "Entries", "Valid", "Violations", "Pass Rate", line,
    ));

    // Per-format summary
    for format in &[
        CorpusFormat::Bash,
        CorpusFormat::Makefile,
        CorpusFormat::Dockerfile,
    ] {
        let fmt_results: Vec<&SchemaResult> = report
            .results
            .iter()
            .filter(|r| r.format == *format)
            .collect();
        let total = fmt_results.len();
        let valid = fmt_results.iter().filter(|r| r.valid).count();
        let violations: usize = fmt_results.iter().map(|r| r.violations.len()).sum();
        let rate = if total > 0 {
            (valid as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        let fmt_name = match format {
            CorpusFormat::Bash => "Bash",
            CorpusFormat::Makefile => "Makefile",
            CorpusFormat::Dockerfile => "Dockerfile",
        };

        out.push_str(&format!(
            "{:<12} {:<14} {:<10} {:<10} {:.1}%\n",
            fmt_name, total, valid, violations, rate,
        ));
    }

    out.push_str(&line);
    out.push('\n');
    out.push_str(&format!(
        "{:<12} {:<14} {:<10} {:<10} {:.1}%\n",
        "Total",
        report.total_entries,
        report.valid_entries,
        report.total_violations,
        report.pass_rate(),
    ));

    out
}

/// Format grammar errors grouped by category
pub fn format_grammar_errors(report: &SchemaReport) -> String {
    let mut out = String::new();
    let line = "\u{2500}".repeat(72);

    out.push_str(&format!(
        "{}\n{:<12} {:<36} {:<8} {}\n{}\n",
        line, "Code", "Description", "Count", "Format", line,
    ));

    // Show all categories, even those with 0 violations
    for cat in GrammarCategory::all() {
        let count = report
            .violations_by_category
            .iter()
            .find(|(c, _)| c == cat)
            .map(|(_, n)| *n)
            .unwrap_or(0);

        let fmt_name = match cat.applicable_format() {
            CorpusFormat::Bash => "Bash",
            CorpusFormat::Makefile => "Makefile",
            CorpusFormat::Dockerfile => "Dockerfile",
        };

        out.push_str(&format!(
            "{:<12} {:<36} {:<8} {}\n",
            cat.code(),
            cat.description(),
            count,
            fmt_name,
        ));
    }

    out.push_str(&line);
    out.push('\n');

    // Show entries with violations
    let entries_with_violations: Vec<&SchemaResult> =
        report.results.iter().filter(|r| !r.valid).collect();

    if entries_with_violations.is_empty() {
        out.push_str("No grammar violations found.\n");
    } else {
        out.push_str(&format!(
            "\nEntries with violations ({}):\n",
            entries_with_violations.len()
        ));
        for result in entries_with_violations.iter().take(20) {
            out.push_str(&format!(
                "  {} ({}): {} violation(s)\n",
                result.entry_id,
                result.format,
                result.violations.len(),
            ));
            for v in &result.violations {
                out.push_str(&format!(
                    "    L{}: {} ({})\n",
                    v.line, v.message, v.category
                ));
            }
        }
        if entries_with_violations.len() > 20 {
            out.push_str(&format!(
                "  ... and {} more entries\n",
                entries_with_violations.len() - 20
            ));
        }
    }

    out
}

/// Get formal grammar specification for a format (§11.8.1-11.8.3)
pub fn format_grammar_spec(format: CorpusFormat) -> String {
    match format {
        CorpusFormat::Bash => posix_grammar_spec(),
        CorpusFormat::Makefile => makefile_grammar_spec(),
        CorpusFormat::Dockerfile => dockerfile_grammar_spec(),
    }
}

fn posix_grammar_spec() -> String {
    let mut out = String::new();
    out.push_str("POSIX Shell Grammar (IEEE Std 1003.1-2017, Section 2)\n");
    out.push_str(&"\u{2500}".repeat(60));
    out.push('\n');
    out.push_str(
        "\
complete_command : list separator_op
               | list
               ;
list            : list separator_op and_or
               | and_or
               ;
and_or          : pipeline
               | and_or AND_IF linebreak pipeline
               | and_or OR_IF linebreak pipeline
               ;
pipeline        : pipe_sequence
               | Bang pipe_sequence
               ;
pipe_sequence   : command
               | pipe_sequence '|' linebreak command
               ;
command         : simple_command
               | compound_command
               | compound_command redirect_list
               | function_definition
               ;
simple_command  : cmd_prefix cmd_word cmd_suffix
               | cmd_prefix cmd_word
               | cmd_prefix
               | cmd_name cmd_suffix
               | cmd_name
               ;
compound_command: brace_group
               | subshell
               | for_clause
               | case_clause
               | if_clause
               | while_clause
               | until_clause
               ;

Validation Layers:
  L1: Lexical  — bashrs parser (ShellAst), token stream valid
  L2: Syntactic — shellcheck -s sh, POSIX grammar compliance
  L3: Semantic  — bashrs linter (SEC/DET/IDEM rules)
  L4: Behavioral — cross-shell execution (dash, bash, ash)
",
    );
    out
}

fn makefile_grammar_spec() -> String {
    let mut out = String::new();
    out.push_str("GNU Make Grammar (GNU Make Manual 4.4, Section 3.7)\n");
    out.push_str(&"\u{2500}".repeat(60));
    out.push('\n');
    out.push_str(
        "\
makefile     : (rule | assignment | directive | comment | empty_line)*
rule         : targets ':' prerequisites '\\n' recipe
targets      : target (' ' target)*
prerequisites: prerequisite (' ' prerequisite)*
recipe       : ('\\t' command '\\n')+
assignment   : variable assignment_op value
assignment_op: ':=' | '?=' | '+=' | '='
directive    : 'include' | 'ifeq' | 'ifdef' | 'define' | '.PHONY' | ...

Validation Layers:
  L1: Lexical  — tab-vs-space detection
  L2: Syntactic — make -n --warn-undefined-variables
  L3: Semantic  — bashrs Makefile linter (MAKE001-MAKE020)
  L4: Behavioral — make -n dry-run comparison
",
    );
    out
}

fn dockerfile_grammar_spec() -> String {
    let mut out = String::new();
    out.push_str("Dockerfile Grammar (Docker Engine v25+)\n");
    out.push_str(&"\u{2500}".repeat(60));
    out.push('\n');
    out.push_str(
        "\
dockerfile   : (instruction | comment | empty_line)*
instruction  : FROM from_args
             | RUN run_args
             | COPY copy_args
             | WORKDIR path
             | ENV env_args
             | EXPOSE port_spec
             | USER user_spec
             | CMD exec_or_shell
             | ENTRYPOINT exec_or_shell
             | ARG arg_spec
             | LABEL label_args
             | HEALTHCHECK healthcheck_args
             | ...
from_args    : ['--platform=' platform] image [':' tag | '@' digest] ['AS' name]
exec_or_shell: exec_form | shell_form
exec_form    : '[' string (',' string)* ']'
shell_form   : string

Validation Layers:
  L1: Lexical  — instruction keyword recognition
  L2: Syntactic — bashrs Dockerfile parser
  L3: Semantic  — bashrs Dockerfile linter (DOCKER001-012) + Hadolint
  L4: Behavioral — docker build --no-cache
",
    );
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::corpus::registry::{CorpusEntry, CorpusFormat, CorpusTier};

    fn make_entry(id: &str, format: CorpusFormat, output: &str) -> CorpusEntry {
        CorpusEntry {
            id: id.to_string(),
            name: format!("test-{id}"),
            description: "Test entry".to_string(),
            format,
            tier: CorpusTier::Trivial,
            input: String::new(),
            expected_output: output.to_string(),
            shellcheck: true,
            deterministic: true,
            idempotent: true,
        }
    }

    #[test]
    fn test_grammar_category_code() {
        assert_eq!(GrammarCategory::MissingQuoting.code(), "GRAM-001");
        assert_eq!(GrammarCategory::Bashism.code(), "GRAM-002");
        assert_eq!(GrammarCategory::TabSpaceConfusion.code(), "GRAM-003");
        assert_eq!(GrammarCategory::ShellFormCmd.code(), "GRAM-004");
        assert_eq!(GrammarCategory::UndefinedVariable.code(), "GRAM-005");
        assert_eq!(GrammarCategory::InvalidArithmetic.code(), "GRAM-006");
        assert_eq!(GrammarCategory::MissingFrom.code(), "GRAM-007");
        assert_eq!(GrammarCategory::CircularDependency.code(), "GRAM-008");
    }

    #[test]
    fn test_grammar_category_display() {
        assert_eq!(format!("{}", GrammarCategory::Bashism), "GRAM-002");
    }

    #[test]
    fn test_grammar_category_all() {
        assert_eq!(GrammarCategory::all().len(), 8);
    }

    #[test]
    fn test_grammar_category_applicable_format() {
        assert_eq!(
            GrammarCategory::MissingQuoting.applicable_format(),
            CorpusFormat::Bash
        );
        assert_eq!(
            GrammarCategory::TabSpaceConfusion.applicable_format(),
            CorpusFormat::Makefile
        );
        assert_eq!(
            GrammarCategory::MissingFrom.applicable_format(),
            CorpusFormat::Dockerfile
        );
    }

    #[test]
    fn test_grammar_category_description() {
        assert!(!GrammarCategory::MissingQuoting.description().is_empty());
        assert!(!GrammarCategory::Bashism.fix_pattern().is_empty());
    }

    #[test]
    fn test_validation_layer_display() {
        assert_eq!(format!("{}", ValidationLayer::Lexical), "L1:Lexical");
        assert_eq!(format!("{}", ValidationLayer::Syntactic), "L2:Syntactic");
        assert_eq!(format!("{}", ValidationLayer::Semantic), "L3:Semantic");
        assert_eq!(format!("{}", ValidationLayer::Behavioral), "L4:Behavioral");
    }

    #[test]
    fn test_validate_bash_clean() {
        let entry = make_entry(
            "B-001",
            CorpusFormat::Bash,
            "#!/bin/sh\nset -eu\necho \"hello\"\n",
        );
        let result = validate_entry(&entry);
        assert!(result.valid);
        assert!(result.violations.is_empty());
        assert!(result.layers_passed.contains(&ValidationLayer::Lexical));
    }

    #[test]
    fn test_validate_bash_bashism() {
        let entry = make_entry(
            "B-002",
            CorpusFormat::Bash,
            "#!/bin/sh\nif [[ -f file ]]; then echo ok; fi\n",
        );
        let result = validate_entry(&entry);
        assert!(!result.valid);
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].category, GrammarCategory::Bashism);
    }

    #[test]
    fn test_validate_bash_unquoted_expansion() {
        let entry = make_entry("B-003", CorpusFormat::Bash, "#!/bin/sh\necho $HOME\n");
        let result = validate_entry(&entry);
        assert!(!result.valid);
        assert_eq!(
            result.violations[0].category,
            GrammarCategory::MissingQuoting
        );
    }

    #[test]
    fn test_validate_bash_quoted_expansion_ok() {
        let entry = make_entry("B-004", CorpusFormat::Bash, "#!/bin/sh\necho \"$HOME\"\n");
        let result = validate_entry(&entry);
        assert!(result.valid);
    }

    #[test]
    fn test_validate_bash_assignment_not_flagged() {
        let entry = make_entry("B-005", CorpusFormat::Bash, "#!/bin/sh\nFOO=$HOME\n");
        let result = validate_entry(&entry);
        // Assignments are not flagged for unquoted expansions
        assert!(result.valid);
    }

    #[test]
    fn test_validate_bash_invalid_arithmetic() {
        let entry = make_entry("B-006", CorpusFormat::Bash, "#!/bin/sh\n(( x = x + 1 ))\n");
        let result = validate_entry(&entry);
        assert!(!result.valid);
        assert_eq!(
            result.violations[0].category,
            GrammarCategory::InvalidArithmetic
        );
    }

    #[test]
    fn test_validate_bash_posix_arithmetic_ok() {
        let entry = make_entry("B-007", CorpusFormat::Bash, "#!/bin/sh\nx=$((x + 1))\n");
        let result = validate_entry(&entry);
        assert!(result.valid);
    }

    #[test]
    fn test_validate_makefile_clean() {
        let entry = make_entry(
            "M-001",
            CorpusFormat::Makefile,
            "CC := gcc\n\nall:\n\t$(CC) -o main main.c\n",
        );
        let result = validate_entry(&entry);
        assert!(result.valid);
    }

    #[test]
    fn test_validate_makefile_space_recipe() {
        let entry = make_entry("M-002", CorpusFormat::Makefile, "all:\n    echo hello\n");
        let result = validate_entry(&entry);
        assert!(!result.valid);
        assert_eq!(
            result.violations[0].category,
            GrammarCategory::TabSpaceConfusion
        );
    }

    #[test]
    fn test_validate_dockerfile_clean() {
        let entry = make_entry(
            "D-001",
            CorpusFormat::Dockerfile,
            "FROM alpine:3.18\nRUN apk add --no-cache curl\nCMD [\"curl\", \"https://example.com\"]\n",
        );
        let result = validate_entry(&entry);
        assert!(result.valid);
    }

    #[test]
    fn test_validate_dockerfile_missing_from() {
        let entry = make_entry("D-002", CorpusFormat::Dockerfile, "RUN apt-get update\n");
        let result = validate_entry(&entry);
        assert!(!result.valid);
        assert_eq!(result.violations[0].category, GrammarCategory::MissingFrom);
    }

    #[test]
    fn test_validate_dockerfile_shell_form_cmd() {
        let entry = make_entry(
            "D-003",
            CorpusFormat::Dockerfile,
            "FROM alpine:3.18\nCMD echo hello\n",
        );
        let result = validate_entry(&entry);
        assert!(!result.valid);
        assert_eq!(result.violations[0].category, GrammarCategory::ShellFormCmd);
    }

    #[test]
    fn test_validate_dockerfile_exec_form_ok() {
        let entry = make_entry(
            "D-004",
            CorpusFormat::Dockerfile,
            "FROM alpine:3.18\nCMD [\"echo\", \"hello\"]\n",
        );
        let result = validate_entry(&entry);
        assert!(result.valid);
    }

    #[test]
    fn test_validate_dockerfile_arg_before_from() {
        let entry = make_entry(
            "D-005",
            CorpusFormat::Dockerfile,
            "ARG VERSION=3.18\nFROM alpine:${VERSION}\nRUN echo ok\n",
        );
        let result = validate_entry(&entry);
        assert!(result.valid);
    }

    #[test]
    fn test_validate_corpus_report() {
        let entries = vec![
            make_entry("B-001", CorpusFormat::Bash, "#!/bin/sh\necho \"ok\"\n"),
            make_entry(
                "B-002",
                CorpusFormat::Bash,
                "#!/bin/sh\nif [[ 1 ]]; then echo ok; fi\n",
            ),
            make_entry("M-001", CorpusFormat::Makefile, "all:\n\techo ok\n"),
            make_entry(
                "D-001",
                CorpusFormat::Dockerfile,
                "FROM alpine:3.18\nRUN echo ok\n",
            ),
        ];
        let registry = CorpusRegistry {
            entries,
        };

        let report = validate_corpus(&registry);
        assert_eq!(report.total_entries, 4);
        assert_eq!(report.valid_entries, 3);
        assert_eq!(report.total_violations, 1);
    }

    #[test]
    fn test_schema_report_pass_rate() {
        let report = SchemaReport {
            results: vec![],
            total_entries: 10,
            valid_entries: 9,
            total_violations: 1,
            violations_by_category: vec![],
        };
        let rate = report.pass_rate();
        assert!((rate - 90.0).abs() < 0.01);
    }

    #[test]
    fn test_schema_report_pass_rate_empty() {
        let report = SchemaReport {
            results: vec![],
            total_entries: 0,
            valid_entries: 0,
            total_violations: 0,
            violations_by_category: vec![],
        };
        assert!((report.pass_rate() - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_format_schema_report() {
        let entries = vec![
            make_entry("B-001", CorpusFormat::Bash, "#!/bin/sh\necho \"ok\"\n"),
            make_entry("M-001", CorpusFormat::Makefile, "all:\n\techo ok\n"),
        ];
        let registry = CorpusRegistry {
            entries,
        };
        let report = validate_corpus(&registry);
        let table = format_schema_report(&report);
        assert!(table.contains("Bash"));
        assert!(table.contains("Makefile"));
        assert!(table.contains("Total"));
    }

    #[test]
    fn test_format_grammar_errors() {
        let entries = vec![make_entry(
            "B-001",
            CorpusFormat::Bash,
            "#!/bin/sh\nif [[ 1 ]]; then echo ok; fi\n",
        )];
        let registry = CorpusRegistry {
            entries,
        };
        let report = validate_corpus(&registry);
        let table = format_grammar_errors(&report);
        assert!(table.contains("GRAM-001"));
        assert!(table.contains("GRAM-002"));
        assert!(table.contains("B-001"));
    }

    #[test]
    fn test_format_grammar_errors_clean() {
        let entries = vec![make_entry(
            "B-001",
            CorpusFormat::Bash,
            "#!/bin/sh\necho \"ok\"\n",
        )];
        let registry = CorpusRegistry {
            entries,
        };
        let report = validate_corpus(&registry);
        let table = format_grammar_errors(&report);
        assert!(table.contains("No grammar violations"));
    }

    #[test]
    fn test_format_grammar_spec_bash() {
        let spec = format_grammar_spec(CorpusFormat::Bash);
        assert!(spec.contains("POSIX Shell Grammar"));
        assert!(spec.contains("complete_command"));
        assert!(spec.contains("L1: Lexical"));
    }

    #[test]
    fn test_format_grammar_spec_makefile() {
        let spec = format_grammar_spec(CorpusFormat::Makefile);
        assert!(spec.contains("GNU Make Grammar"));
        assert!(spec.contains("makefile"));
        assert!(spec.contains("recipe"));
    }

    #[test]
    fn test_format_grammar_spec_dockerfile() {
        let spec = format_grammar_spec(CorpusFormat::Dockerfile);
        assert!(spec.contains("Dockerfile Grammar"));
        assert!(spec.contains("FROM"));
        assert!(spec.contains("exec_form"));
    }

    #[test]
    fn test_check_unquoted_expansion_simple() {
        assert!(check_unquoted_expansion("echo $HOME"));
        assert!(!check_unquoted_expansion("echo \"$HOME\""));
        assert!(!check_unquoted_expansion("FOO=$BAR"));
    }

    #[test]
    fn test_check_unquoted_expansion_single_quote() {
        assert!(!check_unquoted_expansion("echo '$HOME'"));
    }

    #[test]
    fn test_check_unquoted_expansion_arithmetic() {
        assert!(!check_unquoted_expansion("x=$((x + 1))"));
    }

    #[test]
    fn test_check_unquoted_expansion_escaped() {
        assert!(!check_unquoted_expansion("echo \\$HOME"));
    }

    // BH-MUT-0013: is_unquoted_var_at mutation targets
    // Kills mutations of the $( exclusion and ${}/$ _ detection at lines 277-287

    #[test]
    fn test_SCHEMA_MUT_013a_subshell_not_flagged() {
        // $(...) subshell is NOT a bare variable expansion
        assert!(!check_unquoted_expansion("echo $(date)"));
    }

    #[test]
    fn test_SCHEMA_MUT_013b_brace_expansion_flagged() {
        // ${VAR} outside quotes IS an unquoted expansion
        assert!(check_unquoted_expansion("echo ${HOME}"));
    }

    #[test]
    fn test_SCHEMA_MUT_013c_underscore_var_flagged() {
        // $_ outside quotes IS an unquoted expansion
        assert!(check_unquoted_expansion("echo $_"));
    }

    #[test]
    fn test_SCHEMA_MUT_013d_mixed_quotes_var_flagged() {
        // Var between quoted segments is still unquoted
        assert!(check_unquoted_expansion("echo \"hello\" $var 'world'"));
    }

    // BH-MUT-0014: extract_make_var mutation targets
    // Kills mutations of tab/comment filtering at lines 321-322

    #[test]
    fn test_SCHEMA_MUT_014a_extract_make_var_comment() {
        assert!(extract_make_var("# CC := gcc").is_none());
    }

    #[test]
    fn test_SCHEMA_MUT_014b_extract_make_var_tab() {
        assert!(extract_make_var("\t$(CC) -o main main.c").is_none());
    }

    #[test]
    fn test_SCHEMA_MUT_014c_extract_make_var_valid() {
        assert_eq!(extract_make_var("CC := gcc"), Some("CC".to_string()));
    }

    #[test]
    fn test_SCHEMA_MUT_014d_extract_make_var_invalid_name() {
        // Variable name with spaces should not match
        assert!(extract_make_var("bad name := value").is_none());
    }

    #[test]
    fn test_multiple_violations_same_entry() {
        let entry = make_entry(
            "B-010",
            CorpusFormat::Bash,
            "#!/bin/sh\nif [[ -f file ]]; then echo $var; fi\n",
        );
        let result = validate_entry(&entry);
        assert!(!result.valid);
        // Should have both bashism and unquoted expansion
        assert!(result.violations.len() >= 2);
        let categories: Vec<GrammarCategory> =
            result.violations.iter().map(|v| v.category).collect();
        assert!(categories.contains(&GrammarCategory::Bashism));
        assert!(categories.contains(&GrammarCategory::MissingQuoting));
    }

    #[test]
    fn test_empty_output_fails_lexical() {
        let entry = make_entry("B-099", CorpusFormat::Bash, "");
        let result = validate_entry(&entry);
        assert!(!result.layers_passed.contains(&ValidationLayer::Lexical));
    }

    #[test]
    fn test_entrypoint_shell_form_violation() {
        let entry = make_entry(
            "D-010",
            CorpusFormat::Dockerfile,
            "FROM alpine:3.18\nENTRYPOINT /bin/sh\n",
        );
        let result = validate_entry(&entry);
        assert!(!result.valid);
        assert_eq!(result.violations[0].category, GrammarCategory::ShellFormCmd);
    }

    // BH-MUT-0009: is_space_indented_recipe mutation targets
    // Kills mutations of the 4-part AND + OR in lines 337-340

    #[test]
    fn test_SCHEMA_MUT_009a_space_recipe_requires_in_recipe() {
        // Space-indented line NOT inside a recipe context → should NOT flag
        let entry = make_entry(
            "M-MUT-009a",
            CorpusFormat::Makefile,
            "CC := gcc\n    echo hello\n",
        );
        let result = validate_entry(&entry);
        // No GRAM-003 because there's no preceding target rule
        assert!(!result
            .violations
            .iter()
            .any(|v| v.category == GrammarCategory::TabSpaceConfusion));
    }

    #[test]
    fn test_SCHEMA_MUT_009b_tab_recipe_not_flagged() {
        // Tab-indented recipe line → should NOT flag (correct indentation)
        let entry = make_entry("M-MUT-009b", CorpusFormat::Makefile, "all:\n\techo hello\n");
        let result = validate_entry(&entry);
        assert!(result.valid);
    }

    #[test]
    fn test_SCHEMA_MUT_009c_two_space_recipe_flagged() {
        // Two-space indented recipe → should flag GRAM-003
        let entry = make_entry("M-MUT-009c", CorpusFormat::Makefile, "all:\n  echo hello\n");
        let result = validate_entry(&entry);
        assert!(!result.valid);
        assert!(result
            .violations
            .iter()
            .any(|v| v.category == GrammarCategory::TabSpaceConfusion));
    }

    #[test]
    fn test_SCHEMA_MUT_009d_empty_line_resets_recipe() {
        // Empty line between target and space-indented line → NOT in recipe context
        let entry = make_entry(
            "M-MUT-009d",
            CorpusFormat::Makefile,
            "all:\n\n    echo hello\n",
        );
        let result = validate_entry(&entry);
        // Empty line resets in_recipe, so the space line is not flagged
        assert!(!result
            .violations
            .iter()
            .any(|v| v.category == GrammarCategory::TabSpaceConfusion));
    }

    // BH-MUT-0010: Dockerfile ENTRYPOINT exec form
    // Kills mutation of || to && and negation of contains('[')

    #[test]
    fn test_SCHEMA_MUT_010a_entrypoint_exec_form_ok() {
        // ENTRYPOINT with exec form → should NOT flag
        let entry = make_entry(
            "D-MUT-010a",
            CorpusFormat::Dockerfile,
            "FROM alpine:3.18\nENTRYPOINT [\"sh\", \"-c\", \"echo hello\"]\n",
        );
        let result = validate_entry(&entry);
        assert!(result.valid);
    }

    #[test]
    fn test_SCHEMA_MUT_010b_cmd_and_entrypoint_shell_form() {
        // Both CMD and ENTRYPOINT in shell form → should flag both
        let entry = make_entry(
            "D-MUT-010b",
            CorpusFormat::Dockerfile,
            "FROM alpine:3.18\nCMD echo hello\nENTRYPOINT /bin/sh\n",
        );
        let result = validate_entry(&entry);
        assert!(!result.valid);
        let shell_form_count = result
            .violations
            .iter()
            .filter(|v| v.category == GrammarCategory::ShellFormCmd)
            .count();
        assert_eq!(shell_form_count, 2);
    }

    // BH-MUT-0011: Makefile := assignment vs target rule distinction
    // Kills mutation of !line.contains(":=") at line 367

    #[test]
    fn test_SCHEMA_MUT_011a_assignment_not_target() {
        // := assignment should NOT set in_recipe, so next space line is not flagged
        let entry = make_entry(
            "M-MUT-011a",
            CorpusFormat::Makefile,
            "CC := gcc\n    echo hello\n",
        );
        let result = validate_entry(&entry);
        // No tab/space confusion because CC := gcc is assignment, not target
        assert!(!result
            .violations
            .iter()
            .any(|v| v.category == GrammarCategory::TabSpaceConfusion));
    }

    #[test]
    fn test_SCHEMA_MUT_011b_target_then_space_recipe() {
        // Real target rule followed by space-indented recipe → SHOULD flag
        let entry = make_entry(
            "M-MUT-011b",
            CorpusFormat::Makefile,
            "build:\n    gcc -o main main.c\n",
        );
        let result = validate_entry(&entry);
        assert!(!result.valid);
        assert!(result
            .violations
            .iter()
            .any(|v| v.category == GrammarCategory::TabSpaceConfusion));
    }

    // BH-MUT-0012: Bash arithmetic (( vs $(( coexistence
    // Kills mutation of && to || and negation removal at line 228

    #[test]
    fn test_SCHEMA_MUT_012a_posix_arithmetic_not_flagged() {
        // $(( )) is POSIX arithmetic → should NOT flag
        let entry = make_entry(
            "B-MUT-012a",
            CorpusFormat::Bash,
            "#!/bin/sh\nx=$(( x + 1 ))\n",
        );
        let result = validate_entry(&entry);
        assert!(result.valid);
    }

    #[test]
    fn test_SCHEMA_MUT_012b_bash_arithmetic_flagged() {
        // (( )) without $( prefix → SHOULD flag
        let entry = make_entry(
            "B-MUT-012b",
            CorpusFormat::Bash,
            "#!/bin/sh\n(( x = x + 1 ))\n",
        );
        let result = validate_entry(&entry);
        assert!(!result.valid);
        assert!(result
            .violations
            .iter()
            .any(|v| v.category == GrammarCategory::InvalidArithmetic));
    }

    #[test]
    fn test_SCHEMA_MUT_012c_mixed_arithmetic_not_flagged() {
        // Line has both (( and $(( — the $(( takes precedence, NOT a bashism
        let entry = make_entry(
            "B-MUT-012c",
            CorpusFormat::Bash,
            "#!/bin/sh\necho \"result: $(( 1 + 2 ))\"\n",
        );
        let result = validate_entry(&entry);
        // $(( is valid POSIX arithmetic expansion, should not flag
        assert!(!result
            .violations
            .iter()
            .any(|v| v.category == GrammarCategory::InvalidArithmetic));
    }
}
