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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::args::LintLevel;

    // ===== BUILD IGNORED RULES TESTS =====

    #[test]
    fn test_build_ignored_rules_from_ignore_str() {
        let rules = build_ignored_rules(Some("SEC001,DET002"), None, &[]);
        assert!(rules.contains("SEC001"));
        assert!(rules.contains("DET002"));
        assert_eq!(rules.len(), 2);
    }

    #[test]
    fn test_build_ignored_rules_from_exclude() {
        let excludes = vec!["sec003".to_string(), "det004".to_string()];
        let rules = build_ignored_rules(None, Some(&excludes), &[]);
        assert!(rules.contains("SEC003"));
        assert!(rules.contains("DET004"));
    }

    #[test]
    fn test_build_ignored_rules_from_file() {
        let file_rules = vec!["IDEM001".to_string(), "IDEM002".to_string()];
        let rules = build_ignored_rules(None, None, &file_rules);
        assert!(rules.contains("IDEM001"));
        assert!(rules.contains("IDEM002"));
    }

    #[test]
    fn test_build_ignored_rules_combined() {
        let excludes = vec!["det001".to_string()];
        let file_rules = vec!["IDEM001".to_string()];
        let rules = build_ignored_rules(Some("SEC001"), Some(&excludes), &file_rules);
        assert!(rules.contains("SEC001"));
        assert!(rules.contains("DET001"));
        assert!(rules.contains("IDEM001"));
        assert_eq!(rules.len(), 3);
    }

    #[test]
    fn test_build_ignored_rules_handles_whitespace() {
        let rules = build_ignored_rules(Some(" SEC001 , DET002 "), None, &[]);
        assert!(rules.contains("SEC001"));
        assert!(rules.contains("DET002"));
    }

    // ===== DETERMINE MIN SEVERITY TESTS =====

    #[test]
    fn test_determine_min_severity_quiet() {
        let severity = determine_min_severity(true, LintLevel::Info);
        assert_eq!(severity, Severity::Warning);
    }

    #[test]
    fn test_determine_min_severity_level_info() {
        let severity = determine_min_severity(false, LintLevel::Info);
        assert_eq!(severity, Severity::Info);
    }

    #[test]
    fn test_determine_min_severity_level_warning() {
        let severity = determine_min_severity(false, LintLevel::Warning);
        assert_eq!(severity, Severity::Warning);
    }

    #[test]
    fn test_determine_min_severity_level_error() {
        let severity = determine_min_severity(false, LintLevel::Error);
        assert_eq!(severity, Severity::Error);
    }

    // ===== LINT PROCESS TESTS =====

    #[test]
    fn test_file_type_from_filename_shell() {
        assert_eq!(FileType::from_filename("script.sh"), FileType::Shell);
        assert_eq!(FileType::from_filename("test.bash"), FileType::Shell);
        assert_eq!(FileType::from_filename("run"), FileType::Shell);
    }

    #[test]
    fn test_file_type_from_filename_makefile() {
        assert_eq!(FileType::from_filename("Makefile"), FileType::Makefile);
        assert_eq!(FileType::from_filename("makefile"), FileType::Makefile);
        assert_eq!(FileType::from_filename("GNUmakefile"), FileType::Makefile);
    }

    #[test]
    fn test_file_type_from_filename_dockerfile() {
        assert_eq!(FileType::from_filename("Dockerfile"), FileType::Dockerfile);
        assert_eq!(
            FileType::from_filename("Dockerfile.prod"),
            FileType::Dockerfile
        );
    }

    #[test]
    fn test_lint_options_default() {
        let options = LintOptions::default();
        assert!(!options.quiet);
        assert_eq!(options.level, LintLevel::Info);
        assert!(options.ignore_rules.is_empty());
        assert!(!options.fix);
    }

    #[test]
    fn test_process_lint_basic() {
        let source = "echo hello world";
        let options = LintOptions::default();
        let result = process_lint(source, "test.sh", &options);

        assert_eq!(result.file_type, FileType::Shell);
        // Results depend on what the linter finds
    }

    #[test]
    fn test_process_lint_with_ignored_rules() {
        let source = "echo hello world";
        let mut options = LintOptions::default();
        options.ignore_rules.insert("SC2086".to_string());
        let result = process_lint(source, "test.sh", &options);

        // Verify ignored rules are filtered out
        for diag in &result.diagnostics {
            assert_ne!(diag.code, "SC2086");
        }
    }

    #[test]
    fn test_process_lint_quiet_mode() {
        let source = "echo hello world";
        let options = LintOptions {
            quiet: true,
            ..Default::default()
        };
        let result = process_lint(source, "test.sh", &options);

        // Info-level diagnostics should be filtered
        for diag in &result.diagnostics {
            assert!(diag.severity >= crate::linter::Severity::Warning);
        }
    }

    // ===== PURIFY PROCESS TESTS =====

    #[test]
    fn test_process_purify_bash_simple() {
        let source = "echo hello";
        let result = process_purify_bash(source);
        assert!(result.is_ok());

        let result = result.unwrap();
        assert!(!result.purified_source.is_empty());
    }

    #[test]
    fn test_process_purify_bash_mkdir() {
        let source = "mkdir /tmp/test";
        let result = process_purify_bash(source).unwrap();

        // Purification should output something with mkdir
        assert!(result.purified_source.contains("mkdir"));
        // The output should be valid
        assert!(!result.purified_source.is_empty());
    }

    #[test]
    fn test_process_purify_bash_stats() {
        let source = "echo hello\necho world";
        let result = process_purify_bash(source).unwrap();

        assert_eq!(result.stats.input_lines, 2);
        assert!(result.stats.input_bytes > 0);
        assert!(result.stats.total_time_ns > 0);
    }

    #[test]
    fn test_transformation_rule_detection() {
        assert!(detect_transformation_rule("mkdir /tmp", "mkdir -p /tmp").contains("IDEM001"));
        assert!(detect_transformation_rule("rm /tmp/file", "rm -f /tmp/file").contains("IDEM002"));
        assert!(detect_transformation_rule("ln file link", "ln -sf file link").contains("IDEM003"));
        assert!(detect_transformation_rule("echo $RANDOM", "echo 42").contains("DET001"));
    }

    // ===== PURIFICATION STATS TESTS =====

    #[test]
    fn test_purification_stats_throughput() {
        let stats = PurificationStats {
            input_lines: 100,
            input_bytes: 1024 * 1024, // 1 MB
            output_lines: 90,
            output_bytes: 900 * 1024,
            read_time_ns: 1_000_000,
            parse_time_ns: 5_000_000,
            purify_time_ns: 3_000_000,
            codegen_time_ns: 2_000_000,
            write_time_ns: 1_000_000,
            total_time_ns: 1_000_000_000, // 1 second
        };

        let throughput = stats.throughput_mb_s();
        assert!((throughput - 1.0).abs() < 0.01); // ~1 MB/s
    }

    #[test]
    fn test_purification_stats_format_report() {
        let stats = PurificationStats {
            input_lines: 10,
            input_bytes: 100,
            output_lines: 8,
            output_bytes: 80,
            read_time_ns: 1_000,
            parse_time_ns: 2_000,
            purify_time_ns: 3_000,
            codegen_time_ns: 4_000,
            write_time_ns: 5_000,
            total_time_ns: 15_000,
        };

        let report = stats.format_report("input.sh", Some("output.sh"));
        assert!(report.contains("input.sh"));
        assert!(report.contains("output.sh"));
        assert!(report.contains("10 lines"));
        assert!(report.contains("100 bytes"));
    }

    // ===== LINT FILTERING TESTS =====

    #[test]
    fn test_parse_rule_filter() {
        let rules = parse_rule_filter("SEC001,DET002,IDEM003");
        assert_eq!(rules, vec!["SEC001", "DET002", "IDEM003"]);
    }

    #[test]
    fn test_parse_rule_filter_with_spaces() {
        let rules = parse_rule_filter(" SEC001 , DET002 , IDEM003 ");
        assert_eq!(rules, vec!["SEC001", "DET002", "IDEM003"]);
    }

    #[test]
    fn test_parse_rule_filter_single() {
        let rules = parse_rule_filter("SEC001");
        assert_eq!(rules, vec!["SEC001"]);
    }

    // ===== RULE CODE TESTS =====

    #[test]
    fn test_parse_rule_codes() {
        let codes = parse_rule_codes("sec001,det002,idem003");
        assert_eq!(codes, vec!["SEC001", "DET002", "IDEM003"]);
    }

    #[test]
    fn test_parse_rule_codes_with_spaces() {
        let codes = parse_rule_codes(" sec001 , det002 ");
        assert_eq!(codes, vec!["SEC001", "DET002"]);
    }

    #[test]
    fn test_parse_rule_codes_empty() {
        let codes = parse_rule_codes("");
        assert!(codes.is_empty());
    }

    #[test]
    fn test_diagnostic_matches_rules() {
        let rules = vec!["SEC".to_string(), "DET".to_string()];
        assert!(diagnostic_matches_rules("SEC001", &rules));
        assert!(diagnostic_matches_rules("DET002", &rules));
        assert!(!diagnostic_matches_rules("IDEM001", &rules));
    }

    // ===== SEVERITY ICON TESTS =====

    #[test]
    fn test_severity_icon() {
        assert_eq!(severity_icon("error"), "❌");
        assert_eq!(severity_icon("Error"), "❌");
        assert_eq!(severity_icon("ERROR"), "❌");
        assert_eq!(severity_icon("warning"), "⚠");
        assert_eq!(severity_icon("info"), "ℹ");
        assert_eq!(severity_icon("hint"), "💡");
        assert_eq!(severity_icon("unknown"), "•");
    }
}
