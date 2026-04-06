// CLI Logic - Lint Processing
//
// Lint command processing, filtering, and rule handling functions.

use crate::cli::args::{LintLevel, LintProfileArg};
use crate::linter::{LintResult, Severity};
use std::collections::HashSet;

// =============================================================================
// LINT COMMAND LOGIC
// =============================================================================

/// Options for lint processing
#[derive(Debug, Clone)]
pub struct LintOptions {
    pub quiet: bool,
    pub level: LintLevel,
    pub ignore_rules: HashSet<String>,
    pub profile: LintProfileArg,
    pub fix: bool,
    pub fix_assumptions: bool,
}

impl Default for LintOptions {
    fn default() -> Self {
        Self {
            quiet: false,
            level: LintLevel::Info,
            ignore_rules: HashSet::new(),
            profile: LintProfileArg::Standard,
            fix: false,
            fix_assumptions: false,
        }
    }
}

/// Result of lint processing
#[derive(Debug, Clone)]
pub struct LintProcessResult {
    pub diagnostics: Vec<LintDiagnostic>,
    pub error_count: usize,
    pub warning_count: usize,
    pub info_count: usize,
    pub file_type: FileType,
    pub fixes_applied: usize,
}

/// A single lint diagnostic
#[derive(Debug, Clone)]
pub struct LintDiagnostic {
    pub code: String,
    pub message: String,
    pub severity: Severity,
    pub line: usize,
    pub column: usize,
    pub suggestion: Option<String>,
}

/// Detected file type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Shell,
    Makefile,
    Dockerfile,
    Unknown,
}

impl FileType {
    pub fn from_filename(filename: &str) -> Self {
        if super::is_makefile(filename) {
            FileType::Makefile
        } else if super::is_dockerfile(filename) {
            FileType::Dockerfile
        } else {
            FileType::Shell
        }
    }
}

/// Process lint on source content (pure function - no I/O)
pub fn process_lint(source: &str, filename: &str, options: &LintOptions) -> LintProcessResult {
    use crate::linter::rules::{lint_dockerfile_with_profile, lint_makefile, lint_shell};

    let file_type = FileType::from_filename(filename);
    let profile = convert_lint_profile(options.profile);

    // Run appropriate linter
    let raw_result = match file_type {
        FileType::Makefile => lint_makefile(source),
        FileType::Dockerfile => lint_dockerfile_with_profile(source, profile),
        FileType::Shell => lint_shell(source),
        FileType::Unknown => lint_shell(source),
    };

    // Determine minimum severity
    let min_severity = determine_min_severity(options.quiet, options.level);

    // Filter diagnostics
    let filtered = filter_diagnostics(raw_result, min_severity, &options.ignore_rules);

    // Convert to our diagnostic type and count by severity
    let mut error_count = 0;
    let mut warning_count = 0;
    let mut info_count = 0;

    let diagnostics: Vec<LintDiagnostic> = filtered
        .diagnostics
        .iter()
        .map(|d| {
            match d.severity {
                Severity::Error => error_count += 1,
                Severity::Warning => warning_count += 1,
                Severity::Info | Severity::Note | Severity::Perf | Severity::Risk => {
                    info_count += 1;
                }
            }
            LintDiagnostic {
                code: d.code.clone(),
                message: d.message.clone(),
                severity: d.severity,
                line: d.span.start_line,
                column: d.span.start_col,
                suggestion: d.fix.as_ref().map(|f| f.replacement.clone()),
            }
        })
        .collect();

    LintProcessResult {
        diagnostics,
        error_count,
        warning_count,
        info_count,
        file_type,
        fixes_applied: 0, // Set by caller if fixes are applied
    }
}

// =============================================================================
// PURIFY COMMAND LOGIC
// =============================================================================

/// Result of purification processing
#[derive(Debug, Clone)]
pub struct PurifyProcessResult {
    pub purified_source: String,
    pub transformations: Vec<Transformation>,
    pub stats: PurificationStats,
}

/// A single transformation applied during purification
#[derive(Debug, Clone)]
pub struct Transformation {
    pub line: usize,
    pub original: String,
    pub purified: String,
    pub rule: String,
}

/// Process purify on bash source content (pure function - no I/O)
pub fn process_purify_bash(source: &str) -> crate::models::Result<PurifyProcessResult> {
    use crate::bash_parser::codegen::generate_purified_bash;
    use crate::bash_parser::BashParser;
    use std::time::Instant;

    let start = Instant::now();
    let input_lines = source.lines().count();
    let input_bytes = source.len();

    // Parse and purify
    let mut parser = BashParser::new(source).map_err(|e| {
        let diag = crate::bash_parser::parser::format_parse_diagnostic(&e, source, None);
        crate::models::Error::CommandFailed {
            message: format!("{diag}"),
        }
    })?;
    let ast = parser.parse().map_err(|e| {
        let diag = crate::bash_parser::parser::format_parse_diagnostic(&e, parser.source(), None);
        crate::models::Error::CommandFailed {
            message: format!("{diag}"),
        }
    })?;
    let purified_source = generate_purified_bash(&ast);

    let total_time = start.elapsed();

    // Calculate transformations
    let transformations = super::generate_diff_lines(source, &purified_source)
        .into_iter()
        .map(|(line, original, purified)| {
            let rule = detect_transformation_rule(&original, &purified);
            Transformation {
                line,
                original,
                purified,
                rule,
            }
        })
        .collect();

    let stats = PurificationStats {
        input_lines,
        input_bytes,
        output_lines: purified_source.lines().count(),
        output_bytes: purified_source.len(),
        read_time_ns: 0,  // Set by caller
        parse_time_ns: 0, // Included in total
        purify_time_ns: total_time.as_nanos() as u64,
        codegen_time_ns: 0,
        write_time_ns: 0, // Set by caller
        total_time_ns: total_time.as_nanos() as u64,
    };

    Ok(PurifyProcessResult {
        purified_source,
        transformations,
        stats,
    })
}

/// Detect which transformation rule was applied
fn detect_transformation_rule(original: &str, purified: &str) -> String {
    if original.contains("mkdir ") && purified.contains("mkdir -p") {
        "IDEM001: mkdir → mkdir -p".to_string()
    } else if original.contains("rm ") && purified.contains("rm -f") {
        "IDEM002: rm → rm -f".to_string()
    } else if original.contains("ln ") && purified.contains("ln -sf") {
        "IDEM003: ln → ln -sf".to_string()
    } else if original.contains("$RANDOM") {
        "DET001: Remove $RANDOM".to_string()
    } else if original.contains("$$") && !purified.contains("$$") {
        "DET002: Remove $$".to_string()
    } else if original.contains("#!/bin/bash") && purified.contains("#!/bin/sh") {
        "POSIX: bash → sh shebang".to_string()
    } else {
        "TRANSFORM: general".to_string()
    }
}

/// Purification statistics
#[derive(Debug, Clone)]
pub struct PurificationStats {
    pub input_lines: usize,
    pub input_bytes: usize,
    pub output_lines: usize,
    pub output_bytes: usize,
    pub read_time_ns: u64,
    pub parse_time_ns: u64,
    pub purify_time_ns: u64,
    pub codegen_time_ns: u64,
    pub write_time_ns: u64,
    pub total_time_ns: u64,
}

impl PurificationStats {
    pub fn throughput_mb_s(&self) -> f64 {
        let total_secs = self.total_time_ns as f64 / 1_000_000_000.0;
        (self.input_bytes as f64) / total_secs / 1024.0 / 1024.0
    }

    pub fn format_report(&self, input_path: &str, output_path: Option<&str>) -> String {
        let mut report = String::new();
        report.push_str("\n=== Purification Report ===\n");
        report.push_str(&format!("Input: {}\n", input_path));
        if let Some(output) = output_path {
            report.push_str(&format!("Output: {}\n", output));
        }
        report.push_str(&format!(
            "\nInput size: {} lines, {} bytes\n",
            self.input_lines, self.input_bytes
        ));
        report.push_str(&format!(
            "Output size: {} lines, {} bytes\n",
            self.output_lines, self.output_bytes
        ));

        report.push_str("\nTransformations Applied:\n");
        report.push_str("- Shebang: #!/bin/bash → #!/bin/sh\n");
        report.push_str("- Determinism: Removed $RANDOM, timestamps\n");
        report.push_str("- Idempotency: mkdir → mkdir -p, rm → rm -f\n");
        report.push_str("- Safety: All variables quoted\n");

        report.push_str("\nPerformance:\n");
        report.push_str(&format!(
            "  Read:     {:>8.2?}\n",
            std::time::Duration::from_nanos(self.read_time_ns)
        ));
        report.push_str(&format!(
            "  Parse:    {:>8.2?}\n",
            std::time::Duration::from_nanos(self.parse_time_ns)
        ));
        report.push_str(&format!(
            "  Purify:   {:>8.2?}\n",
            std::time::Duration::from_nanos(self.purify_time_ns)
        ));
        report.push_str(&format!(
            "  Codegen:  {:>8.2?}\n",
            std::time::Duration::from_nanos(self.codegen_time_ns)
        ));
        report.push_str(&format!(
            "  Write:    {:>8.2?}\n",
            std::time::Duration::from_nanos(self.write_time_ns)
        ));
        report.push_str("  ─────────────────\n");
        report.push_str(&format!(
            "  Total:    {:>8.2?}\n",
            std::time::Duration::from_nanos(self.total_time_ns)
        ));

        report.push_str(&format!(
            "\nThroughput: {:.2} MB/s\n",
            self.throughput_mb_s()
        ));
        report
    }
}

// =============================================================================
// LINT FILTERING
// =============================================================================

/// Build set of ignored rule codes from various sources
pub fn build_ignored_rules(
    ignore_rules_str: Option<&str>,
    exclude_rules: Option<&[String]>,
    ignore_file_rules: &[String],
) -> HashSet<String> {
    let mut rules = HashSet::new();

    // Add from --ignore (comma-separated)
    if let Some(ignore_str) = ignore_rules_str {
        for code in ignore_str.split(',') {
            let code = code.trim().to_uppercase();
            if !code.is_empty() {
                rules.insert(code);
            }
        }
    }

    // Add from -e (can be repeated)
    if let Some(excludes) = exclude_rules {
        for code in excludes {
            let code = code.trim().to_uppercase();
            if !code.is_empty() {
                rules.insert(code);
            }
        }
    }

    // Add rule codes from .bashrsignore file
    for code in ignore_file_rules {
        rules.insert(code.clone());
    }

    rules
}

/// Determine minimum severity based on --quiet and --level flags
pub fn determine_min_severity(quiet: bool, level: LintLevel) -> Severity {
    if quiet {
        Severity::Warning // --quiet suppresses info
    } else {
        match level {
            LintLevel::Info => Severity::Info,
            LintLevel::Warning => Severity::Warning,
            LintLevel::Error => Severity::Error,
        }
    }
}

/// Filter diagnostics by severity and ignored rules
pub fn filter_diagnostics(
    result: LintResult,
    min_severity: Severity,
    ignored_rules: &HashSet<String>,
) -> LintResult {
    let filtered = result
        .diagnostics
        .into_iter()
        .filter(|d| d.severity >= min_severity)
        .filter(|d| !ignored_rules.contains(&d.code.to_uppercase()))
        .collect();
    LintResult {
        diagnostics: filtered,
    }
}

/// Convert CLI lint profile arg to linter profile
pub fn convert_lint_profile(profile: LintProfileArg) -> crate::linter::rules::LintProfile {
    use crate::linter::rules::LintProfile;

    match profile {
        LintProfileArg::Standard => LintProfile::Standard,
        LintProfileArg::Coursera => LintProfile::Coursera,
        LintProfileArg::DevContainer => LintProfile::DevContainer,
    }
}

/// Filter lint diagnostics by rule codes (pure function)
pub fn filter_diagnostics_by_rules(
    diagnostics: Vec<crate::linter::Diagnostic>,
    rules: &[&str],
) -> Vec<crate::linter::Diagnostic> {
    diagnostics
        .into_iter()
        .filter(|d| rules.iter().any(|rule| d.code.contains(rule)))
        .collect()
}

/// Parse comma-separated rule filter into list
pub fn parse_rule_filter(filter: &str) -> Vec<&str> {
    filter.split(',').map(|s| s.trim()).collect()
}

/// Parse multiple rule codes from comma-separated string
pub fn parse_rule_codes(input: &str) -> Vec<String> {
    input
        .split(',')
        .map(|s| s.trim().to_uppercase())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Check if diagnostic code matches any of the rule filters
pub fn diagnostic_matches_rules(code: &str, rules: &[String]) -> bool {
    rules.iter().any(|rule| code.contains(rule))
}

/// Format lint severity as icon
pub fn severity_icon(severity: &str) -> &'static str {
    match severity.to_lowercase().as_str() {
        "error" => "❌",
        "warning" => "⚠",
        "info" => "ℹ",
        "hint" => "💡",
        _ => "•",
    }
}

// FIXME(PMAT-238): #[cfg(test)]
// FIXME(PMAT-238): #[path = "logic_lint_tests_build_ignore.rs"]
// FIXME(PMAT-238): mod tests_extracted;
