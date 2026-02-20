// CLI Logic Module - Extracted for testability
//
// This module contains pure functions that return structured results
// instead of printing directly. The commands.rs file acts as a thin shim
// that calls these functions and handles I/O.
//
// Architecture:
// - logic.rs: Pure functions (no I/O, no printing, no file access)
// - commands.rs: Thin I/O shim (reads files, calls logic, prints output)
//
// This separation enables:
// - Unit testing of all business logic
// - High test coverage (95%+ target)
// - Clear separation of concerns

use crate::cli::args::{LintLevel, LintProfileArg};
use crate::linter::{LintResult, Severity};
use crate::models::{Error, Result};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

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
        if is_makefile(filename) {
            FileType::Makefile
        } else if is_dockerfile(filename) {
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
                    info_count += 1
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
pub fn process_purify_bash(source: &str) -> Result<PurifyProcessResult> {
    use crate::bash_parser::codegen::generate_purified_bash;
    use crate::bash_parser::BashParser;
    use std::time::Instant;

    let start = Instant::now();
    let input_lines = source.lines().count();
    let input_bytes = source.len();

    // Parse and purify
    let mut parser = BashParser::new(source).map_err(|e| {
        let diag = crate::bash_parser::parser::format_parse_diagnostic(&e, source, None);
        Error::CommandFailed { message: format!("{diag}") }
    })?;
    let ast = parser.parse().map_err(|e| {
        let diag = crate::bash_parser::parser::format_parse_diagnostic(&e, parser.source(), None);
        Error::CommandFailed { message: format!("{diag}") }
    })?;
    let purified_source = generate_purified_bash(&ast);

    let total_time = start.elapsed();

    // Calculate transformations
    let transformations = generate_diff_lines(source, &purified_source)
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

// =============================================================================
// TEST COMMAND LOGIC
// =============================================================================

/// Result of test processing
#[derive(Debug, Clone)]
pub struct TestProcessResult {
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub total: usize,
    pub test_results: Vec<TestCaseResult>,
    pub duration_ms: u64,
}

/// Result of a single test case
#[derive(Debug, Clone)]
pub struct TestCaseResult {
    pub name: String,
    pub passed: bool,
    pub message: Option<String>,
    pub duration_ms: u64,
}

impl TestProcessResult {
    pub fn success_rate(&self) -> f64 {
        if self.total == 0 {
            100.0
        } else {
            (self.passed as f64 / self.total as f64) * 100.0
        }
    }

    pub fn all_passed(&self) -> bool {
        self.failed == 0
    }
}

// =============================================================================
// SCORE COMMAND LOGIC
// =============================================================================

/// Result of score processing
#[derive(Debug, Clone)]
pub struct ScoreProcessResult {
    pub overall_score: f64,
    pub grade: Grade,
    pub dimensions: Vec<ScoreDimension>,
}

/// A dimension of the quality score
#[derive(Debug, Clone)]
pub struct ScoreDimension {
    pub name: String,
    pub score: f64,
    pub max_score: f64,
    pub status: &'static str,
}

// =============================================================================
// AUDIT COMMAND LOGIC
// =============================================================================

/// Result of audit processing
#[derive(Debug, Clone)]
pub struct AuditProcessResult {
    pub parse_success: bool,
    pub parse_error: Option<String>,
    pub lint_errors: usize,
    pub lint_warnings: usize,
    pub test_passed: usize,
    pub test_failed: usize,
    pub test_total: usize,
    pub score: Option<ScoreProcessResult>,
    pub overall_pass: bool,
    pub failure_reason: Option<String>,
}

impl AuditProcessResult {
    pub fn passed(&self) -> bool {
        self.overall_pass
    }
}

// =============================================================================
// COVERAGE COMMAND LOGIC
// =============================================================================

/// Result of coverage processing
#[derive(Debug, Clone)]
pub struct CoverageProcessResult {
    pub line_coverage: f64,
    pub function_coverage: f64,
    pub total_lines: usize,
    pub covered_lines: usize,
    pub total_functions: usize,
    pub covered_functions: usize,
    pub uncovered_lines: Vec<usize>,
    pub uncovered_functions: Vec<String>,
}

impl CoverageProcessResult {
    pub fn meets_threshold(&self, min_percent: f64) -> bool {
        self.line_coverage >= min_percent
    }
}

// =============================================================================
// FORMAT COMMAND LOGIC
// =============================================================================

/// Result of format processing
#[derive(Debug, Clone)]
pub struct FormatProcessResult {
    pub original: String,
    pub formatted: String,
    pub changed: bool,
    pub diff_lines: Vec<(usize, String, String)>,
}

// ===== SHELL SCRIPT DETECTION =====

/// Detect if a file is a shell script based on extension and shebang (Issue #84)
///
/// Returns true if the file:
/// - Has a shell extension (.sh, .bash, .ksh, .zsh)
/// - Has a shell shebang (#!/bin/sh, #!/bin/bash, etc.)
pub fn is_shell_script_file(path: &Path, content: &str) -> bool {
    // Check file extension
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        let ext_lower = ext.to_lowercase();
        if matches!(ext_lower.as_str(), "sh" | "bash" | "ksh" | "zsh" | "ash") {
            return true;
        }
    }

    // Check shebang
    let first_line = content.lines().next().unwrap_or("");
    if first_line.starts_with("#!") {
        let shebang_lower = first_line.to_lowercase();
        // Check for common shell interpreters
        if shebang_lower.contains("/sh")
            || shebang_lower.contains("/bash")
            || shebang_lower.contains("/zsh")
            || shebang_lower.contains("/ksh")
            || shebang_lower.contains("/ash")
            || shebang_lower.contains("/dash")
            || shebang_lower.contains("env sh")
            || shebang_lower.contains("env bash")
        {
            return true;
        }
    }

    false
}

/// Normalize a shell script for comparison
/// Removes comments and normalizes whitespace
pub fn normalize_shell_script(script: &str) -> String {
    script
        .lines()
        .filter(|line| !line.trim().starts_with('#'))
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

// ===== LINT FILTERING =====

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

// ===== FILE TYPE DETECTION =====

/// Detect if a file is a Makefile
pub fn is_makefile(filename: &str) -> bool {
    filename == "Makefile"
        || filename == "makefile"
        || filename == "GNUmakefile"
        || filename.ends_with(".mk")
        || filename.ends_with(".make")
}

/// Detect if a file is a Dockerfile
pub fn is_dockerfile(filename: &str) -> bool {
    let filename_lower = filename.to_lowercase();
    filename_lower == "dockerfile"
        || filename_lower.starts_with("dockerfile.")
        || filename_lower.ends_with(".dockerfile")
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

// ===== GATE EXECUTION LOGIC =====

/// Result of a gate check
#[derive(Debug, Clone, PartialEq)]
pub enum GateResult {
    /// Gate passed
    Pass,
    /// Gate failed
    Fail,
    /// Gate skipped (tool not found)
    Skipped(String),
    /// Unknown gate
    Unknown,
}

impl GateResult {
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Pass | Self::Skipped(_))
    }

    pub fn format(&self) -> &'static str {
        match self {
            Self::Pass => "✅ PASS",
            Self::Fail => "❌ FAIL",
            Self::Skipped(_) => "⚠️  SKIP",
            Self::Unknown => "⚠️  Unknown gate",
        }
    }
}

/// Validate gate tier
pub fn validate_gate_tier(tier: u8) -> Result<()> {
    if !(1..=3).contains(&tier) {
        Err(Error::Validation(format!(
            "Invalid tier: {}. Must be 1, 2, or 3.",
            tier
        )))
    } else {
        Ok(())
    }
}

// ===== BUILD/CHECK COMMAND LOGIC =====

/// Result of check command
#[derive(Debug, Clone)]
pub enum CheckResult {
    /// File is compatible
    Compatible,
    /// File is a shell script (not Rash source)
    IsShellScript { path: String },
    /// Check failed
    Error(String),
}

impl CheckResult {
    pub fn format(&self) -> String {
        match self {
            Self::Compatible => "✓ File is compatible with Rash".to_string(),
            Self::IsShellScript { path } => {
                format!(
                    "File '{}' appears to be a shell script, not Rash source.\n\n\
                     The 'check' command is for verifying Rash (.rs) source files that will be\n\
                     transpiled to shell scripts.\n\n\
                     For linting existing shell scripts, use:\n\
                       bashrs lint {}\n\n\
                     For purifying shell scripts (adding determinism/idempotency):\n\
                       bashrs purify {}",
                    path, path, path
                )
            }
            Self::Error(e) => format!("Error: {}", e),
        }
    }
}

/// Process check command logic
pub fn process_check(path: &Path, content: &str) -> CheckResult {
    if is_shell_script_file(path, content) {
        return CheckResult::IsShellScript {
            path: path.display().to_string(),
        };
    }

    // Actual check logic would go here
    CheckResult::Compatible
}

// ===== VERIFY COMMAND LOGIC =====

/// Result of verification
#[derive(Debug, Clone, PartialEq)]
pub enum VerifyResult {
    /// Scripts match
    Match,
    /// Scripts don't match
    Mismatch,
}

impl VerifyResult {
    pub fn format(&self) -> &'static str {
        match self {
            Self::Match => "✓ Shell script matches Rust source",
            Self::Mismatch => "✗ Shell script does not match Rust source",
        }
    }
}

/// Compare generated shell script with expected
pub fn verify_scripts(generated: &str, expected: &str) -> VerifyResult {
    if normalize_shell_script(generated) == normalize_shell_script(expected) {
        VerifyResult::Match
    } else {
        VerifyResult::Mismatch
    }
}

// ===== PURIFICATION REPORT LOGIC =====

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

// ===== SCORE/AUDIT LOGIC =====

/// Grade calculation based on score
/// Note: Higher grades (A) are "better" than lower grades (F)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Grade {
    A,
    B,
    C,
    D,
    F,
}

impl PartialOrd for Grade {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Grade {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // A is the best (highest), F is the worst (lowest)
        // So A > B > C > D > F
        let self_val = match self {
            Grade::A => 4,
            Grade::B => 3,
            Grade::C => 2,
            Grade::D => 1,
            Grade::F => 0,
        };
        let other_val = match other {
            Grade::A => 4,
            Grade::B => 3,
            Grade::C => 2,
            Grade::D => 1,
            Grade::F => 0,
        };
        self_val.cmp(&other_val)
    }
}

impl Grade {
    pub fn from_score(score: f64) -> Self {
        if score >= 90.0 {
            Grade::A
        } else if score >= 80.0 {
            Grade::B
        } else if score >= 70.0 {
            Grade::C
        } else if score >= 60.0 {
            Grade::D
        } else {
            Grade::F
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Grade::A => "A",
            Grade::B => "B",
            Grade::C => "C",
            Grade::D => "D",
            Grade::F => "F",
        }
    }

    pub fn meets_minimum(&self, min: &str) -> bool {
        let min_grade = match min.to_uppercase().as_str() {
            "A" => Grade::A,
            "B" => Grade::B,
            "C" => Grade::C,
            "D" => Grade::D,
            _ => Grade::F,
        };
        *self >= min_grade
    }
}

// ===== FORMAT COMMAND LOGIC =====

/// Result of format check
#[derive(Debug, Clone)]
pub struct FormatCheckResult {
    pub files_checked: usize,
    pub files_formatted: usize,
    pub files_unchanged: usize,
}

impl FormatCheckResult {
    pub fn all_formatted(&self) -> bool {
        self.files_formatted == 0
    }
}

// ===== DOCKERFILE PURIFICATION LOGIC =====

/// Convert ADD to COPY for local files (DOCKER006)
///
/// Keep ADD for:
/// - URLs (http://, https://)
/// - Tarballs (.tar, .tar.gz, .tgz, .tar.bz2, .tar.xz) which ADD auto-extracts
pub fn convert_add_to_copy_if_local(line: &str) -> String {
    let trimmed = line.trim();

    // Skip comment lines
    if trimmed.starts_with('#') {
        return line.to_string();
    }

    // Must be an ADD directive
    if !trimmed.starts_with("ADD ") {
        return line.to_string();
    }

    // Extract the source path (first argument after ADD)
    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    let source = match parts.get(1) {
        Some(s) => *s,
        None => return line.to_string(), // Malformed ADD directive
    };

    // Check if source is a URL
    if source.starts_with("http://") || source.starts_with("https://") {
        return line.to_string(); // Keep ADD for URLs
    }

    // Check if source is a tarball (which ADD auto-extracts)
    let is_tarball = source.ends_with(".tar")
        || source.ends_with(".tar.gz")
        || source.ends_with(".tgz")
        || source.ends_with(".tar.bz2")
        || source.ends_with(".tar.xz")
        || source.ends_with(".tar.Z");

    if is_tarball {
        return line.to_string(); // Keep ADD for tarballs
    }

    // It's a local file - convert ADD to COPY
    line.replacen("ADD ", "COPY ", 1)
}

/// Add --no-install-recommends flag to apt-get install commands (DOCKER005)
pub fn add_no_install_recommends(line: &str) -> String {
    let trimmed = line.trim();

    // Skip comment lines
    if trimmed.starts_with('#') {
        return line.to_string();
    }

    // Check if already has --no-install-recommends
    if line.contains("--no-install-recommends") {
        return line.to_string();
    }

    // Must contain apt-get install
    if !line.contains("apt-get install") {
        return line.to_string();
    }

    let mut result = line.to_string();

    // Replace "apt-get install -y " (with -y flag)
    result = result.replace(
        "apt-get install -y ",
        "apt-get install -y --no-install-recommends ",
    );

    // Replace remaining "apt-get install "
    if !result.contains("--no-install-recommends") {
        result = result.replace(
            "apt-get install ",
            "apt-get install --no-install-recommends ",
        );
    }

    // Handle edge case: "apt-get install" at end of line (no trailing space)
    if !result.contains("--no-install-recommends") && result.trim_end().ends_with("apt-get install")
    {
        result = result.trim_end().to_string() + " --no-install-recommends ";
    }

    result
}

/// Add cleanup commands for package managers (DOCKER003)
pub fn add_package_manager_cleanup(line: &str) -> String {
    let trimmed = line.trim();

    // Skip comment lines
    if trimmed.starts_with('#') {
        return line.to_string();
    }

    // Check if cleanup already present
    if line.contains("/var/lib/apt/lists") || line.contains("/var/cache/apk") {
        return line.to_string();
    }

    // Detect apt/apt-get commands
    if line.contains("apt-get install") || line.contains("apt install") {
        return format!("{} && rm -rf /var/lib/apt/lists/*", line.trim_end());
    }

    // Detect apk commands
    if line.contains("apk add") {
        return format!("{} && rm -rf /var/cache/apk/*", line.trim_end());
    }

    line.to_string()
}

/// Pin unpinned base images to stable versions (DOCKER002)
pub fn pin_base_image_version(line: &str) -> String {
    let trimmed = line.trim();

    // Must be a FROM line
    if !trimmed.starts_with("FROM ") {
        return line.to_string();
    }

    // Parse FROM line
    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    let image_part = match parts.get(1) {
        Some(img) => *img,
        None => return line.to_string(), // Malformed FROM line
    };

    // Parse registry prefix
    let (registry_prefix, image_with_tag) = if let Some(slash_pos) = image_part.find('/') {
        let prefix_part = &image_part[..slash_pos];
        if prefix_part.contains('.') || prefix_part == "localhost" {
            (Some(prefix_part), &image_part[slash_pos + 1..])
        } else {
            (None, image_part)
        }
    } else {
        (None, image_part)
    };

    // Split image into name and tag
    let (image_name, tag) = if let Some(colon_pos) = image_with_tag.find(':') {
        let name = &image_with_tag[..colon_pos];
        let tag = &image_with_tag[colon_pos + 1..];
        (name, Some(tag))
    } else {
        (image_with_tag, None)
    };

    // Determine if pinning is needed
    let needs_pinning = tag.is_none() || tag == Some("latest");

    if !needs_pinning {
        return line.to_string(); // Already has specific version
    }

    // Map common images to stable versions
    let pinned_tag = match image_name {
        "ubuntu" => "22.04",
        "debian" => "12-slim",
        "alpine" => "3.19",
        "node" => "20-alpine",
        "python" => "3.11-slim",
        "rust" => "1.75-alpine",
        "nginx" => "1.25-alpine",
        "postgres" => "16-alpine",
        "redis" => "7-alpine",
        _ => return line.to_string(), // Unknown image
    };

    // Reconstruct FROM line with pinned version
    let pinned_image = if let Some(prefix) = registry_prefix {
        format!("{}/{}:{}", prefix, image_name, pinned_tag)
    } else {
        format!("{}:{}", image_name, pinned_tag)
    };

    // Preserve "AS <name>" if present
    if parts.len() > 2 {
        let rest = parts.get(2..).map(|s| s.join(" ")).unwrap_or_default();
        format!("FROM {} {}", pinned_image, rest)
    } else {
        format!("FROM {}", pinned_image)
    }
}

/// Find devcontainer.json in standard locations
pub fn find_devcontainer_json(path: &Path) -> Result<PathBuf> {
    // If path is a file, use it directly
    if path.is_file() {
        return Ok(path.to_path_buf());
    }

    // If path is a directory, search standard locations
    let candidates = [
        path.join(".devcontainer/devcontainer.json"),
        path.join(".devcontainer.json"),
    ];

    for candidate in &candidates {
        if candidate.exists() {
            return Ok(candidate.clone());
        }
    }

    // Check for .devcontainer/<folder>/devcontainer.json
    let devcontainer_dir = path.join(".devcontainer");
    if devcontainer_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&devcontainer_dir) {
            for entry in entries.flatten() {
                let subdir = entry.path();
                if subdir.is_dir() {
                    let candidate = subdir.join("devcontainer.json");
                    if candidate.exists() {
                        return Ok(candidate);
                    }
                }
            }
        }
    }

    Err(Error::Validation(format!(
        "No devcontainer.json found in {}. Expected locations:\n  \
         - .devcontainer/devcontainer.json\n  \
         - .devcontainer.json\n  \
         - .devcontainer/<folder>/devcontainer.json",
        path.display()
    )))
}

/// Parse custom size limit from string (e.g., "2GB", "500MB")
pub fn parse_size_limit(s: &str) -> Option<u64> {
    let s = s.to_uppercase();
    if s.ends_with("GB") {
        s[..s.len() - 2]
            .trim()
            .parse::<f64>()
            .ok()
            .map(|n| (n * 1_000_000_000.0) as u64)
    } else if s.ends_with("MB") {
        s[..s.len() - 2]
            .trim()
            .parse::<f64>()
            .ok()
            .map(|n| (n * 1_000_000.0) as u64)
    } else if s.ends_with("KB") {
        s[..s.len() - 2]
            .trim()
            .parse::<f64>()
            .ok()
            .map(|n| (n * 1_000.0) as u64)
    } else {
        s.parse::<u64>().ok()
    }
}

/// Estimate build time based on layer complexity
pub fn estimate_build_time_seconds(
    layer_count: usize,
    total_size: u64,
    has_apt: bool,
    has_pip: bool,
    has_npm: bool,
) -> u64 {
    let mut total_seconds = 0u64;

    // Base time for each layer
    total_seconds += layer_count as u64;

    // Add time based on size (1 second per 100MB)
    total_seconds += total_size / 100_000_000;

    // Add extra time for known slow operations
    if has_apt {
        total_seconds += 10;
    }
    if has_pip {
        total_seconds += 5;
    }
    if has_npm {
        total_seconds += 5;
    }

    total_seconds
}

/// Format build time as human-readable string
pub fn format_build_time(seconds: u64) -> String {
    if seconds < 60 {
        format!("~{}s", seconds)
    } else {
        format!("~{}m {}s", seconds / 60, seconds % 60)
    }
}

// ===== UTILITY FUNCTIONS =====

/// Hex encode bytes to string
pub fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Format timestamp as relative time
pub fn format_timestamp(timestamp: u64) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let diff = now.saturating_sub(timestamp);

    if diff < 60 {
        "just now".to_string()
    } else if diff < 3600 {
        format!("{}m ago", diff / 60)
    } else if diff < 86400 {
        format!("{}h ago", diff / 3600)
    } else {
        format!("{}d ago", diff / 86400)
    }
}

/// Truncate string to max length with ellipsis (delegates to batuta-common).
pub fn truncate_str(s: &str, max_len: usize) -> String {
    batuta_common::display::truncate_str(s, max_len)
}

/// Generate diff lines between original and purified content
pub fn generate_diff_lines(original: &str, purified: &str) -> Vec<(usize, String, String)> {
    let original_lines: Vec<&str> = original.lines().collect();
    let purified_lines: Vec<&str> = purified.lines().collect();

    original_lines
        .iter()
        .zip(purified_lines.iter())
        .enumerate()
        .filter_map(|(i, (orig, pure))| {
            if orig != pure {
                Some((i + 1, orig.to_string(), pure.to_string()))
            } else {
                None
            }
        })
        .collect()
}

/// Helper to get status emoji for dimension score
pub fn score_status(score: f64) -> &'static str {
    if score >= 8.0 {
        "✅"
    } else if score >= 6.0 {
        "⚠️"
    } else {
        "❌"
    }
}

/// Helper to get status emoji for coverage percent
pub fn coverage_status(percent: f64) -> &'static str {
    if percent >= 80.0 {
        "✅"
    } else if percent >= 50.0 {
        "⚠️"
    } else {
        "❌"
    }
}

/// Helper to get CSS class for coverage percent
pub fn coverage_class(percent: f64) -> &'static str {
    if percent >= 90.0 {
        "excellent"
    } else if percent >= 80.0 {
        "good"
    } else if percent >= 70.0 {
        "fair"
    } else {
        "poor"
    }
}

// =============================================================================
// EXTRACTED PURE LOGIC FUNCTIONS
// =============================================================================

/// Extract exit code from error message (pure function)
pub fn extract_exit_code(error: &str) -> i32 {
    // Common patterns for exit codes in error messages
    let patterns = [
        ("exit code ", 10),
        ("exited with ", 12),
        ("returned ", 9),
        ("status ", 7),
    ];

    for (pattern, prefix_len) in patterns {
        if let Some(idx) = error.to_lowercase().find(pattern) {
            let start = idx + prefix_len;
            let code_str: String = error[start..]
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect();
            if let Ok(code) = code_str.parse::<i32>() {
                return code;
            }
        }
    }

    // Check for well-known exit codes in error messages
    if error.contains("command not found") {
        return 127;
    }
    if error.contains("Permission denied") || error.contains("permission denied") {
        return 126;
    }

    // Default to generic failure
    1
}

/// Check if output path indicates stdout (pure function)
pub fn should_output_to_stdout(output_path: &Path) -> bool {
    output_path == Path::new("-") || output_path == Path::new("/dev/null")
}

/// Detect the current platform (pure function)
pub fn detect_platform() -> &'static str {
    #[cfg(target_os = "linux")]
    {
        "linux"
    }
    #[cfg(target_os = "macos")]
    {
        "macos"
    }
    #[cfg(target_os = "windows")]
    {
        "windows"
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        "unknown"
    }
}

/// Count duplicate entries in a list of paths (pure function)
pub fn count_duplicate_path_entries(entries: &[String]) -> usize {
    use std::collections::HashSet;
    let unique: HashSet<&String> = entries.iter().collect();
    entries.len().saturating_sub(unique.len())
}

// =============================================================================
// DOCKERFILE PURIFICATION LOGIC (extracted from commands.rs)
// =============================================================================

/// Purify a Dockerfile source (pure function - no I/O)
///
/// Applies the following transformations:
/// - DOCKER002: Pin unpinned base images to stable versions
/// - DOCKER003: Add package manager cleanup
/// - DOCKER005: Add --no-install-recommends to apt-get
/// - DOCKER006: Convert ADD to COPY for local files
/// - Add non-root USER directive before CMD/ENTRYPOINT
pub fn purify_dockerfile_source(source: &str, skip_user: bool) -> String {
    let lines: Vec<&str> = source.lines().collect();
    let mut purified = Vec::new();

    // Check if USER directive already exists
    let has_user = lines.iter().any(|line| line.trim().starts_with("USER "));
    let is_scratch = lines
        .iter()
        .any(|line| line.trim().starts_with("FROM scratch"));

    // Find CMD/ENTRYPOINT position
    let cmd_pos = lines.iter().position(|line| {
        let trimmed = line.trim();
        trimmed.starts_with("CMD ") || trimmed.starts_with("ENTRYPOINT ")
    });

    // Build purified Dockerfile
    for (i, line) in lines.iter().enumerate() {
        // Check if we should add USER before CMD/ENTRYPOINT
        if !skip_user && !has_user && !is_scratch && Some(i) == cmd_pos {
            purified.push(String::new());
            purified.push("# Security: Run as non-root user".to_string());
            purified.push("RUN groupadd -r appuser && useradd -r -g appuser appuser".to_string());
            purified.push("USER appuser".to_string());
            purified.push(String::new());
        }

        // DOCKER002: Pin unpinned base images
        let mut processed_line = if line.trim().starts_with("FROM ") {
            pin_base_image_version(line)
        } else {
            line.to_string()
        };

        // DOCKER006: Convert ADD to COPY for local files
        if line.trim().starts_with("ADD ") {
            processed_line = convert_add_to_copy_if_local(&processed_line);
        }

        // DOCKER005: Add --no-install-recommends to apt-get install
        if line.trim().starts_with("RUN ") && processed_line.contains("apt-get install") {
            processed_line = add_no_install_recommends(&processed_line);
        }

        // DOCKER003: Add apt/apk cleanup
        if line.trim().starts_with("RUN ") {
            processed_line = add_package_manager_cleanup(&processed_line);
        }

        purified.push(processed_line);
    }

    purified.join("\n")
}

/// Check if Dockerfile has USER directive
pub fn dockerfile_has_user_directive(source: &str) -> bool {
    source.lines().any(|line| line.trim().starts_with("USER "))
}

/// Check if Dockerfile uses scratch base
pub fn dockerfile_is_scratch(source: &str) -> bool {
    source
        .lines()
        .any(|line| line.trim().starts_with("FROM scratch"))
}

/// Find CMD or ENTRYPOINT line number (0-indexed)
pub fn dockerfile_find_cmd_line(source: &str) -> Option<usize> {
    source.lines().position(|line| {
        let trimmed = line.trim();
        trimmed.starts_with("CMD ") || trimmed.starts_with("ENTRYPOINT ")
    })
}

// =============================================================================
// LINT FILTERING LOGIC (extracted from commands.rs)
// =============================================================================

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

// =============================================================================
// GRADE INTERPRETATION (extracted from commands.rs)
// =============================================================================

/// Get human-readable grade interpretation
pub fn grade_interpretation(grade: &str) -> &'static str {
    match grade {
        "A+" => "Excellent! Near-perfect code quality.",
        "A" => "Great! Very good code quality.",
        "B+" | "B" => "Good code quality with room for improvement.",
        "C+" | "C" => "Average code quality. Consider addressing suggestions.",
        "D" => "Below average. Multiple improvements needed.",
        "F" => "Poor code quality. Significant improvements required.",
        _ => "Unknown grade.",
    }
}

/// Get grade emoji/symbol
pub fn grade_symbol(grade: &str) -> &'static str {
    match grade {
        "A+" | "A" | "B+" | "B" => "✓",
        "C+" | "C" | "D" => "⚠",
        "F" => "✗",
        _ => "?",
    }
}

// =============================================================================
// REPORT FORMATTING (extracted from commands.rs)
// =============================================================================

/// Format purification report as human text
pub fn format_purify_report_human(
    transformations_applied: usize,
    issues_fixed: usize,
    manual_fixes_needed: usize,
    report_items: &[String],
) -> String {
    let mut output = String::new();
    output.push_str("Makefile Purification Report\n");
    output.push_str("============================\n");
    output.push_str(&format!(
        "Transformations Applied: {}\n",
        transformations_applied
    ));
    output.push_str(&format!("Issues Fixed: {}\n", issues_fixed));
    output.push_str(&format!("Manual Fixes Needed: {}\n", manual_fixes_needed));
    output.push('\n');
    for (i, item) in report_items.iter().enumerate() {
        output.push_str(&format!("{}: {}\n", i + 1, item));
    }
    output
}

/// Format purification report as JSON
pub fn format_purify_report_json(
    transformations_applied: usize,
    issues_fixed: usize,
    manual_fixes_needed: usize,
    report_items: &[String],
) -> String {
    let mut output = String::new();
    output.push_str("{\n");
    output.push_str(&format!(
        "  \"transformations_applied\": {},\n",
        transformations_applied
    ));
    output.push_str(&format!("  \"issues_fixed\": {},\n", issues_fixed));
    output.push_str(&format!(
        "  \"manual_fixes_needed\": {},\n",
        manual_fixes_needed
    ));
    output.push_str("  \"report\": [\n");
    for (i, item) in report_items.iter().enumerate() {
        let comma = if i < report_items.len() - 1 { "," } else { "" };
        output.push_str(&format!("    \"{}\"{}\n", item.replace('"', "\\\""), comma));
    }
    output.push_str("  ]\n");
    output.push_str("}\n");
    output
}

/// Format purification report as Markdown
pub fn format_purify_report_markdown(
    transformations_applied: usize,
    issues_fixed: usize,
    manual_fixes_needed: usize,
    report_items: &[String],
) -> String {
    let mut output = String::new();
    output.push_str("# Makefile Purification Report\n\n");
    output.push_str(&format!(
        "**Transformations**: {}\n",
        transformations_applied
    ));
    output.push_str(&format!("**Issues Fixed**: {}\n", issues_fixed));
    output.push_str(&format!(
        "**Manual Fixes Needed**: {}\n\n",
        manual_fixes_needed
    ));
    for (i, item) in report_items.iter().enumerate() {
        output.push_str(&format!("{}. {}\n", i + 1, item));
    }
    output
}

// =============================================================================
// SCORE FORMATTING (extracted from commands.rs)
// =============================================================================

/// Format quality score as human text
#[allow(clippy::too_many_arguments)]
pub fn format_score_human(
    grade: &str,
    score: f64,
    complexity: f64,
    safety: f64,
    maintainability: f64,
    testing: f64,
    documentation: f64,
    suggestions: &[String],
    detailed: bool,
) -> String {
    let mut output = String::new();
    output.push('\n');
    output.push_str("Bash Script Quality Score\n");
    output.push_str("=========================\n\n");
    output.push_str(&format!("Overall Grade: {}\n", grade));
    output.push_str(&format!("Overall Score: {:.1}/10.0\n\n", score));

    if detailed {
        output.push_str("Dimension Scores:\n");
        output.push_str("-----------------\n");
        output.push_str(&format!("Complexity:      {:.1}/10.0\n", complexity));
        output.push_str(&format!("Safety:          {:.1}/10.0\n", safety));
        output.push_str(&format!("Maintainability: {:.1}/10.0\n", maintainability));
        output.push_str(&format!("Testing:         {:.1}/10.0\n", testing));
        output.push_str(&format!("Documentation:   {:.1}/10.0\n\n", documentation));
    }

    if !suggestions.is_empty() {
        output.push_str("Improvement Suggestions:\n");
        output.push_str("------------------------\n");
        for (i, suggestion) in suggestions.iter().enumerate() {
            output.push_str(&format!("{}. {}\n", i + 1, suggestion));
        }
        output.push('\n');
    }

    output.push_str(&format!(
        "{} {}\n",
        grade_symbol(grade),
        grade_interpretation(grade)
    ));
    output
}

/// Validate proof format data
pub fn validate_proof_data(source_hash: &str, verification_level: &str, target: &str) -> bool {
    // Hash should be non-empty hex
    !source_hash.is_empty()
        && source_hash.chars().all(|c| c.is_ascii_hexdigit())
        && !verification_level.is_empty()
        && !target.is_empty()
}

/// Parse shell dialect from string
pub fn parse_shell_dialect(s: &str) -> Option<&'static str> {
    match s.to_lowercase().as_str() {
        "posix" | "sh" => Some("posix"),
        "bash" => Some("bash"),
        "zsh" => Some("zsh"),
        "dash" => Some("dash"),
        _ => None,
    }
}

/// Calculate percentage with bounds
pub fn calculate_percentage(value: usize, total: usize) -> f64 {
    if total == 0 {
        100.0
    } else {
        (value as f64 / total as f64) * 100.0
    }
}

/// Format bytes as human readable size
pub fn format_bytes_human(bytes: u64) -> String {
    if bytes >= 1_000_000_000 {
        format!("{:.2} GB", bytes as f64 / 1_000_000_000.0)
    } else if bytes >= 1_000_000 {
        format!("{:.2} MB", bytes as f64 / 1_000_000.0)
    } else if bytes >= 1_000 {
        format!("{:.2} KB", bytes as f64 / 1_000.0)
    } else {
        format!("{} B", bytes)
    }
}

/// Format duration in seconds as human readable
pub fn format_duration_human(seconds: u64) -> String {
    if seconds >= 3600 {
        format!(
            "{}h {}m {}s",
            seconds / 3600,
            (seconds % 3600) / 60,
            seconds % 60
        )
    } else if seconds >= 60 {
        format!("{}m {}s", seconds / 60, seconds % 60)
    } else {
        format!("{}s", seconds)
    }
}

/// Check if path looks like stdin/stdout marker
pub fn is_stdio_path(path: &Path) -> bool {
    path == Path::new("-") || path == Path::new("/dev/stdin") || path == Path::new("/dev/stdout")
}

// =============================================================================
// SIZE PARSING (extracted from commands.rs)
// =============================================================================

/// Parse size string like "10GB" or "500MB" into bytes
pub fn parse_size_string(s: &str) -> Option<u64> {
    let s = s.trim().to_uppercase();
    if s.ends_with("GB") {
        s[..s.len() - 2]
            .trim()
            .parse::<f64>()
            .ok()
            .map(|n| (n * 1_000_000_000.0) as u64)
    } else if s.ends_with("MB") {
        s[..s.len() - 2]
            .trim()
            .parse::<f64>()
            .ok()
            .map(|n| (n * 1_000_000.0) as u64)
    } else if s.ends_with("KB") {
        s[..s.len() - 2]
            .trim()
            .parse::<f64>()
            .ok()
            .map(|n| (n * 1_000.0) as u64)
    } else if s.ends_with('B') {
        s[..s.len() - 1].trim().parse::<u64>().ok()
    } else {
        // Try parsing as raw bytes
        s.parse::<u64>().ok()
    }
}

/// Format build time estimate from layer data (pure function)
pub fn format_build_time_estimate(
    layer_count: usize,
    total_size_bytes: u64,
    has_apt: bool,
    has_pip: bool,
    has_npm: bool,
) -> String {
    let seconds =
        estimate_build_time_seconds(layer_count, total_size_bytes, has_apt, has_pip, has_npm);
    if seconds < 60 {
        format!("~{}s", seconds)
    } else {
        format!("~{}m {}s", seconds / 60, seconds % 60)
    }
}

/// Check if size exceeds limit
pub fn size_exceeds_limit(size_bytes: u64, limit_bytes: u64) -> bool {
    size_bytes > limit_bytes
}

/// Calculate size percentage of limit
pub fn size_percentage_of_limit(size_bytes: u64, limit_bytes: u64) -> f64 {
    if limit_bytes == 0 {
        100.0
    } else {
        (size_bytes as f64 / limit_bytes as f64) * 100.0
    }
}

/// Determine if layer contains slow operation
pub fn layer_has_slow_operation(content: &str) -> (bool, bool, bool) {
    let lower = content.to_lowercase();
    (
        lower.contains("apt-get install") || lower.contains("apt install"),
        lower.contains("pip install") || lower.contains("pip3 install"),
        lower.contains("npm install") || lower.contains("yarn install"),
    )
}

/// Format size comparison for display
pub fn format_size_comparison(actual_bytes: u64, limit_bytes: u64) -> String {
    let actual_gb = actual_bytes as f64 / 1_000_000_000.0;
    let limit_gb = limit_bytes as f64 / 1_000_000_000.0;
    let percentage = size_percentage_of_limit(actual_bytes, limit_bytes);

    if actual_bytes > limit_bytes {
        format!("✗ EXCEEDS LIMIT: {:.2}GB > {:.0}GB", actual_gb, limit_gb)
    } else {
        format!(
            "✓ Within limit: {:.2}GB / {:.0}GB ({:.0}%)",
            actual_gb, limit_gb, percentage
        )
    }
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

/// Classify test result status
pub fn test_result_status(passed: usize, failed: usize, total: usize) -> &'static str {
    if failed > 0 {
        "FAILED"
    } else if passed == total && total > 0 {
        "PASSED"
    } else if total == 0 {
        "NO TESTS"
    } else {
        "PARTIAL"
    }
}

/// Calculate test pass rate
pub fn test_pass_rate(passed: usize, total: usize) -> f64 {
    if total == 0 {
        100.0
    } else {
        (passed as f64 / total as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== SHELL SCRIPT DETECTION TESTS =====

    #[test]
    fn test_is_shell_script_by_extension() {
        assert!(is_shell_script_file(Path::new("script.sh"), "echo hello"));
        assert!(is_shell_script_file(Path::new("script.bash"), "echo hello"));
        assert!(is_shell_script_file(Path::new("script.zsh"), "echo hello"));
        assert!(is_shell_script_file(Path::new("script.ksh"), "echo hello"));
        assert!(is_shell_script_file(Path::new("script.ash"), "echo hello"));
        assert!(!is_shell_script_file(
            Path::new("script.rs"),
            "fn main() {}"
        ));
        assert!(!is_shell_script_file(
            Path::new("script.py"),
            "print('hello')"
        ));
    }

    #[test]
    fn test_is_shell_script_by_shebang() {
        assert!(is_shell_script_file(
            Path::new("script"),
            "#!/bin/sh\necho hello"
        ));
        assert!(is_shell_script_file(
            Path::new("script"),
            "#!/bin/bash\necho hello"
        ));
        assert!(is_shell_script_file(
            Path::new("script"),
            "#!/usr/bin/env bash\necho hello"
        ));
        assert!(is_shell_script_file(
            Path::new("script"),
            "#!/usr/bin/env sh\necho hello"
        ));
        assert!(!is_shell_script_file(
            Path::new("script"),
            "#!/usr/bin/python\nprint('hello')"
        ));
    }

    #[test]
    fn test_is_shell_script_case_insensitive() {
        assert!(is_shell_script_file(Path::new("script.SH"), "echo hello"));
        assert!(is_shell_script_file(Path::new("script.BASH"), "echo hello"));
    }

    // ===== NORMALIZE SHELL SCRIPT TESTS =====

    #[test]
    fn test_normalize_shell_script_removes_comments() {
        let script = "# comment\necho hello\n# another comment\necho world";
        let normalized = normalize_shell_script(script);
        assert_eq!(normalized, "echo hello\necho world");
    }

    #[test]
    fn test_normalize_shell_script_trims_whitespace() {
        let script = "  echo hello  \n  echo world  ";
        let normalized = normalize_shell_script(script);
        assert_eq!(normalized, "echo hello\necho world");
    }

    #[test]
    fn test_normalize_shell_script_removes_empty_lines() {
        let script = "echo hello\n\n\necho world";
        let normalized = normalize_shell_script(script);
        assert_eq!(normalized, "echo hello\necho world");
    }

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

    // ===== FILE TYPE DETECTION TESTS =====

    #[test]
    fn test_is_makefile() {
        assert!(is_makefile("Makefile"));
        assert!(is_makefile("makefile"));
        assert!(is_makefile("GNUmakefile"));
        assert!(is_makefile("rules.mk"));
        assert!(is_makefile("build.make"));
        assert!(!is_makefile("script.sh"));
        assert!(!is_makefile("Makefile.md"));
    }

    #[test]
    fn test_is_dockerfile() {
        assert!(is_dockerfile("Dockerfile"));
        assert!(is_dockerfile("dockerfile"));
        assert!(is_dockerfile("DOCKERFILE"));
        assert!(is_dockerfile("Dockerfile.dev"));
        assert!(is_dockerfile("app.dockerfile"));
        assert!(!is_dockerfile("Makefile"));
        assert!(!is_dockerfile("script.sh"));
    }

    // ===== GATE RESULT TESTS =====

    #[test]
    fn test_gate_result_is_success() {
        assert!(GateResult::Pass.is_success());
        assert!(GateResult::Skipped("tool not found".to_string()).is_success());
        assert!(!GateResult::Fail.is_success());
        assert!(!GateResult::Unknown.is_success());
    }

    #[test]
    fn test_validate_gate_tier() {
        assert!(validate_gate_tier(1).is_ok());
        assert!(validate_gate_tier(2).is_ok());
        assert!(validate_gate_tier(3).is_ok());
        assert!(validate_gate_tier(0).is_err());
        assert!(validate_gate_tier(4).is_err());
    }

    // ===== VERIFY SCRIPTS TESTS =====

    #[test]
    fn test_verify_scripts_match() {
        let script1 = "#!/bin/sh\necho hello";
        let script2 = "#!/bin/sh\necho hello";
        assert_eq!(verify_scripts(script1, script2), VerifyResult::Match);
    }

    #[test]
    fn test_verify_scripts_match_ignores_comments() {
        let script1 = "# comment\necho hello";
        let script2 = "echo hello";
        assert_eq!(verify_scripts(script1, script2), VerifyResult::Match);
    }

    #[test]
    fn test_verify_scripts_match_ignores_whitespace() {
        let script1 = "  echo hello  ";
        let script2 = "echo hello";
        assert_eq!(verify_scripts(script1, script2), VerifyResult::Match);
    }

    #[test]
    fn test_verify_scripts_mismatch() {
        let script1 = "echo hello";
        let script2 = "echo world";
        assert_eq!(verify_scripts(script1, script2), VerifyResult::Mismatch);
    }

    // ===== CHECK RESULT TESTS =====

    #[test]
    fn test_check_result_shell_script() {
        let result = process_check(Path::new("script.sh"), "echo hello");
        assert!(matches!(result, CheckResult::IsShellScript { .. }));
    }

    #[test]
    fn test_check_result_rust_file() {
        let result = process_check(Path::new("main.rs"), "fn main() {}");
        assert!(matches!(result, CheckResult::Compatible));
    }

    // ===== GRADE TESTS =====

    #[test]
    fn test_grade_from_score() {
        assert_eq!(Grade::from_score(95.0), Grade::A);
        assert_eq!(Grade::from_score(90.0), Grade::A);
        assert_eq!(Grade::from_score(85.0), Grade::B);
        assert_eq!(Grade::from_score(80.0), Grade::B);
        assert_eq!(Grade::from_score(75.0), Grade::C);
        assert_eq!(Grade::from_score(65.0), Grade::D);
        assert_eq!(Grade::from_score(55.0), Grade::F);
    }

    #[test]
    fn test_grade_meets_minimum() {
        assert!(Grade::A.meets_minimum("A"));
        assert!(Grade::A.meets_minimum("B"));
        assert!(Grade::A.meets_minimum("C"));
        assert!(!Grade::B.meets_minimum("A"));
        assert!(Grade::B.meets_minimum("B"));
        assert!(Grade::C.meets_minimum("D"));
        assert!(!Grade::D.meets_minimum("C"));
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

    // ===== FORMAT CHECK RESULT TESTS =====

    #[test]
    fn test_format_check_result_all_formatted() {
        let result = FormatCheckResult {
            files_checked: 5,
            files_formatted: 0,
            files_unchanged: 5,
        };
        assert!(result.all_formatted());
    }

    #[test]
    fn test_format_check_result_needs_formatting() {
        let result = FormatCheckResult {
            files_checked: 5,
            files_formatted: 2,
            files_unchanged: 3,
        };
        assert!(!result.all_formatted());
    }

    // ===== DOCKERFILE LOGIC TESTS =====

    #[test]
    fn test_convert_add_to_copy_local_file() {
        assert_eq!(
            convert_add_to_copy_if_local("ADD file.txt /app/"),
            "COPY file.txt /app/"
        );
    }

    #[test]
    fn test_convert_add_to_copy_preserves_url() {
        let line = "ADD https://example.com/file.tar.gz /app/";
        assert_eq!(convert_add_to_copy_if_local(line), line);
    }

    #[test]
    fn test_convert_add_to_copy_preserves_tarball() {
        let line = "ADD archive.tar.gz /app/";
        assert_eq!(convert_add_to_copy_if_local(line), line);

        let line2 = "ADD data.tgz /app/";
        assert_eq!(convert_add_to_copy_if_local(line2), line2);
    }

    #[test]
    fn test_convert_add_to_copy_preserves_comment() {
        let line = "# ADD file.txt /app/";
        assert_eq!(convert_add_to_copy_if_local(line), line);
    }

    #[test]
    fn test_convert_add_to_copy_non_add_line() {
        let line = "COPY file.txt /app/";
        assert_eq!(convert_add_to_copy_if_local(line), line);
    }

    #[test]
    fn test_add_no_install_recommends() {
        assert_eq!(
            add_no_install_recommends("RUN apt-get install -y curl"),
            "RUN apt-get install -y --no-install-recommends curl"
        );
    }

    #[test]
    fn test_add_no_install_recommends_already_present() {
        let line = "RUN apt-get install -y --no-install-recommends curl";
        assert_eq!(add_no_install_recommends(line), line);
    }

    #[test]
    fn test_add_no_install_recommends_without_y() {
        assert_eq!(
            add_no_install_recommends("RUN apt-get install curl"),
            "RUN apt-get install --no-install-recommends curl"
        );
    }

    #[test]
    fn test_add_no_install_recommends_comment() {
        let line = "# apt-get install curl";
        assert_eq!(add_no_install_recommends(line), line);
    }

    #[test]
    fn test_add_no_install_recommends_non_apt() {
        let line = "RUN yum install curl";
        assert_eq!(add_no_install_recommends(line), line);
    }

    #[test]
    fn test_add_package_manager_cleanup_apt() {
        assert_eq!(
            add_package_manager_cleanup("RUN apt-get install -y curl"),
            "RUN apt-get install -y curl && rm -rf /var/lib/apt/lists/*"
        );
    }

    #[test]
    fn test_add_package_manager_cleanup_apk() {
        assert_eq!(
            add_package_manager_cleanup("RUN apk add curl"),
            "RUN apk add curl && rm -rf /var/cache/apk/*"
        );
    }

    #[test]
    fn test_add_package_manager_cleanup_already_present() {
        let line = "RUN apt-get install curl && rm -rf /var/lib/apt/lists/*";
        assert_eq!(add_package_manager_cleanup(line), line);
    }

    #[test]
    fn test_add_package_manager_cleanup_comment() {
        let line = "# apt-get install curl";
        assert_eq!(add_package_manager_cleanup(line), line);
    }

    #[test]
    fn test_add_package_manager_cleanup_other_command() {
        let line = "RUN echo hello";
        assert_eq!(add_package_manager_cleanup(line), line);
    }

    #[test]
    fn test_pin_base_image_ubuntu() {
        assert_eq!(pin_base_image_version("FROM ubuntu"), "FROM ubuntu:22.04");
    }

    #[test]
    fn test_pin_base_image_latest() {
        assert_eq!(
            pin_base_image_version("FROM alpine:latest"),
            "FROM alpine:3.19"
        );
    }

    #[test]
    fn test_pin_base_image_already_pinned() {
        let line = "FROM python:3.9";
        assert_eq!(pin_base_image_version(line), line);
    }

    #[test]
    fn test_pin_base_image_with_as() {
        assert_eq!(
            pin_base_image_version("FROM node AS builder"),
            "FROM node:20-alpine AS builder"
        );
    }

    #[test]
    fn test_pin_base_image_with_registry() {
        assert_eq!(
            pin_base_image_version("FROM docker.io/ubuntu"),
            "FROM docker.io/ubuntu:22.04"
        );
    }

    #[test]
    fn test_pin_base_image_unknown() {
        let line = "FROM mycompany/myimage";
        assert_eq!(pin_base_image_version(line), line);
    }

    #[test]
    fn test_pin_base_image_non_from_line() {
        let line = "RUN echo hello";
        assert_eq!(pin_base_image_version(line), line);
    }

    #[test]
    fn test_parse_size_limit_gb() {
        assert_eq!(parse_size_limit("2GB"), Some(2_000_000_000));
        assert_eq!(parse_size_limit("1.5gb"), Some(1_500_000_000));
    }

    #[test]
    fn test_parse_size_limit_mb() {
        assert_eq!(parse_size_limit("500MB"), Some(500_000_000));
        assert_eq!(parse_size_limit("100mb"), Some(100_000_000));
    }

    #[test]
    fn test_parse_size_limit_kb() {
        assert_eq!(parse_size_limit("1000KB"), Some(1_000_000));
    }

    #[test]
    fn test_parse_size_limit_invalid() {
        assert_eq!(parse_size_limit("invalid"), None);
        assert_eq!(parse_size_limit(""), None);
    }

    #[test]
    fn test_estimate_build_time_basic() {
        let seconds = estimate_build_time_seconds(5, 0, false, false, false);
        assert_eq!(seconds, 5); // 1 second per layer
    }

    #[test]
    fn test_estimate_build_time_with_size() {
        let seconds = estimate_build_time_seconds(2, 500_000_000, false, false, false);
        assert_eq!(seconds, 2 + 5); // 2 layers + 5 seconds for 500MB
    }

    #[test]
    fn test_estimate_build_time_with_package_managers() {
        let seconds = estimate_build_time_seconds(1, 0, true, true, true);
        assert_eq!(seconds, 1 + 10 + 5 + 5); // 1 layer + apt + pip + npm
    }

    #[test]
    fn test_format_build_time_seconds() {
        assert_eq!(format_build_time(30), "~30s");
        assert_eq!(format_build_time(59), "~59s");
    }

    #[test]
    fn test_format_build_time_minutes() {
        assert_eq!(format_build_time(60), "~1m 0s");
        assert_eq!(format_build_time(90), "~1m 30s");
        assert_eq!(format_build_time(125), "~2m 5s");
    }

    #[test]
    fn test_find_devcontainer_json_not_found() {
        let result = find_devcontainer_json(Path::new("/nonexistent/path"));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("No devcontainer.json found"));
    }

    // ===== UTILITY FUNCTION TESTS =====

    #[test]
    fn test_hex_encode_empty() {
        assert_eq!(hex_encode(&[]), "");
    }

    #[test]
    fn test_hex_encode_single_byte() {
        assert_eq!(hex_encode(&[0x00]), "00");
        assert_eq!(hex_encode(&[0xff]), "ff");
        assert_eq!(hex_encode(&[0x0a]), "0a");
    }

    #[test]
    fn test_hex_encode_multiple_bytes() {
        assert_eq!(hex_encode(&[0xde, 0xad, 0xbe, 0xef]), "deadbeef");
        assert_eq!(hex_encode(&[0x01, 0x23, 0x45, 0x67]), "01234567");
    }

    #[test]
    fn test_format_timestamp_just_now() {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let result = format_timestamp(now - 30);
        assert_eq!(result, "just now");
    }

    #[test]
    fn test_format_timestamp_minutes_ago() {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let result = format_timestamp(now - 120);
        assert_eq!(result, "2m ago");
    }

    #[test]
    fn test_format_timestamp_hours_ago() {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let result = format_timestamp(now - 7200);
        assert_eq!(result, "2h ago");
    }

    #[test]
    fn test_format_timestamp_days_ago() {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let result = format_timestamp(now - 172800);
        assert_eq!(result, "2d ago");
    }

    #[test]
    fn test_truncate_str_short() {
        assert_eq!(truncate_str("hello", 10), "hello");
    }

    #[test]
    fn test_truncate_str_exact() {
        assert_eq!(truncate_str("hello", 5), "hello");
    }

    #[test]
    fn test_truncate_str_long() {
        assert_eq!(truncate_str("hello world", 8), "hello...");
    }

    #[test]
    fn test_truncate_str_empty() {
        assert_eq!(truncate_str("", 10), "");
    }

    #[test]
    fn test_generate_diff_lines_identical() {
        let diff = generate_diff_lines("a\nb\nc", "a\nb\nc");
        assert!(diff.is_empty());
    }

    #[test]
    fn test_generate_diff_lines_one_change() {
        let diff = generate_diff_lines("a\nb\nc", "a\nB\nc");
        assert_eq!(diff.len(), 1);
        assert_eq!(diff[0], (2, "b".to_string(), "B".to_string()));
    }

    #[test]
    fn test_generate_diff_lines_multiple_changes() {
        let diff = generate_diff_lines("a\nb\nc", "A\nb\nC");
        assert_eq!(diff.len(), 2);
        assert_eq!(diff[0], (1, "a".to_string(), "A".to_string()));
        assert_eq!(diff[1], (3, "c".to_string(), "C".to_string()));
    }

    #[test]
    fn test_generate_diff_lines_empty() {
        let diff = generate_diff_lines("", "");
        assert!(diff.is_empty());
    }

    #[test]
    fn test_score_status_excellent() {
        assert_eq!(score_status(10.0), "✅");
        assert_eq!(score_status(8.0), "✅");
    }

    #[test]
    fn test_score_status_warning() {
        assert_eq!(score_status(7.0), "⚠️");
        assert_eq!(score_status(6.0), "⚠️");
    }

    #[test]
    fn test_score_status_poor() {
        assert_eq!(score_status(5.0), "❌");
        assert_eq!(score_status(0.0), "❌");
    }

    #[test]
    fn test_coverage_status_good() {
        assert_eq!(coverage_status(90.0), "✅");
        assert_eq!(coverage_status(80.0), "✅");
    }

    #[test]
    fn test_coverage_status_medium() {
        assert_eq!(coverage_status(75.0), "⚠️");
        assert_eq!(coverage_status(50.0), "⚠️");
    }

    #[test]
    fn test_coverage_status_poor() {
        assert_eq!(coverage_status(49.0), "❌");
        assert_eq!(coverage_status(0.0), "❌");
    }

    #[test]
    fn test_coverage_class_excellent() {
        assert_eq!(coverage_class(100.0), "excellent");
        assert_eq!(coverage_class(95.0), "excellent");
        assert_eq!(coverage_class(90.0), "excellent");
    }

    #[test]
    fn test_coverage_class_good() {
        assert_eq!(coverage_class(89.9), "good");
        assert_eq!(coverage_class(85.0), "good");
        assert_eq!(coverage_class(80.0), "good");
    }

    #[test]
    fn test_coverage_class_fair() {
        assert_eq!(coverage_class(79.9), "fair");
        assert_eq!(coverage_class(75.0), "fair");
        assert_eq!(coverage_class(70.0), "fair");
    }

    #[test]
    fn test_coverage_class_poor() {
        assert_eq!(coverage_class(69.9), "poor");
        assert_eq!(coverage_class(50.0), "poor");
        assert_eq!(coverage_class(0.0), "poor");
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
        let mut options = LintOptions::default();
        options.quiet = true;
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

    // ===== TEST PROCESS RESULT TESTS =====

    #[test]
    fn test_test_process_result_success_rate() {
        let result = TestProcessResult {
            passed: 8,
            failed: 2,
            skipped: 0,
            total: 10,
            test_results: vec![],
            duration_ms: 100,
        };

        assert_eq!(result.success_rate(), 80.0);
        assert!(!result.all_passed());
    }

    #[test]
    fn test_test_process_result_all_passed() {
        let result = TestProcessResult {
            passed: 10,
            failed: 0,
            skipped: 0,
            total: 10,
            test_results: vec![],
            duration_ms: 100,
        };

        assert_eq!(result.success_rate(), 100.0);
        assert!(result.all_passed());
    }

    #[test]
    fn test_test_process_result_empty() {
        let result = TestProcessResult {
            passed: 0,
            failed: 0,
            skipped: 0,
            total: 0,
            test_results: vec![],
            duration_ms: 0,
        };

        assert_eq!(result.success_rate(), 100.0);
        assert!(result.all_passed());
    }

    // ===== COVERAGE PROCESS RESULT TESTS =====

    #[test]
    fn test_coverage_process_result_threshold() {
        let result = CoverageProcessResult {
            line_coverage: 85.0,
            function_coverage: 90.0,
            total_lines: 100,
            covered_lines: 85,
            total_functions: 10,
            covered_functions: 9,
            uncovered_lines: vec![1, 2, 3],
            uncovered_functions: vec!["foo".to_string()],
        };

        assert!(result.meets_threshold(80.0));
        assert!(result.meets_threshold(85.0));
        assert!(!result.meets_threshold(90.0));
    }

    // ===== AUDIT PROCESS RESULT TESTS =====

    #[test]
    fn test_audit_process_result_passed() {
        let result = AuditProcessResult {
            parse_success: true,
            parse_error: None,
            lint_errors: 0,
            lint_warnings: 2,
            test_passed: 10,
            test_failed: 0,
            test_total: 10,
            score: None,
            overall_pass: true,
            failure_reason: None,
        };

        assert!(result.passed());
    }

    #[test]
    fn test_audit_process_result_failed() {
        let result = AuditProcessResult {
            parse_success: true,
            parse_error: None,
            lint_errors: 5,
            lint_warnings: 2,
            test_passed: 8,
            test_failed: 2,
            test_total: 10,
            score: None,
            overall_pass: false,
            failure_reason: Some("Lint errors found".to_string()),
        };

        assert!(!result.passed());
    }

    // ===== EXTRACT EXIT CODE TESTS =====

    #[test]
    fn test_extract_exit_code_exit_code_pattern() {
        assert_eq!(extract_exit_code("Process failed with exit code 1"), 1);
        assert_eq!(extract_exit_code("exit code 127"), 127);
        assert_eq!(extract_exit_code("Error: exit code 255"), 255);
    }

    #[test]
    fn test_extract_exit_code_exited_with_pattern() {
        assert_eq!(extract_exit_code("Command exited with 42"), 42);
        assert_eq!(extract_exit_code("Process exited with 0"), 0);
    }

    #[test]
    fn test_extract_exit_code_returned_pattern() {
        assert_eq!(extract_exit_code("Function returned 5"), 5);
        assert_eq!(extract_exit_code("returned 100"), 100);
    }

    #[test]
    fn test_extract_exit_code_status_pattern() {
        assert_eq!(extract_exit_code("status 2"), 2);
        assert_eq!(extract_exit_code("Exit status 128"), 128);
    }

    #[test]
    fn test_extract_exit_code_command_not_found() {
        assert_eq!(extract_exit_code("bash: foo: command not found"), 127);
        assert_eq!(extract_exit_code("command not found: xyz"), 127);
    }

    #[test]
    fn test_extract_exit_code_permission_denied() {
        assert_eq!(extract_exit_code("Permission denied"), 126);
        assert_eq!(extract_exit_code("Error: permission denied"), 126);
    }

    #[test]
    fn test_extract_exit_code_default() {
        assert_eq!(extract_exit_code("Unknown error"), 1);
        assert_eq!(extract_exit_code("Something went wrong"), 1);
        assert_eq!(extract_exit_code(""), 1);
    }

    // ===== SHOULD OUTPUT TO STDOUT TESTS =====

    #[test]
    fn test_should_output_to_stdout_dash() {
        use std::path::Path;
        assert!(should_output_to_stdout(Path::new("-")));
    }

    #[test]
    fn test_should_output_to_stdout_devnull() {
        use std::path::Path;
        assert!(should_output_to_stdout(Path::new("/dev/null")));
    }

    #[test]
    fn test_should_output_to_stdout_regular_file() {
        use std::path::Path;
        assert!(!should_output_to_stdout(Path::new("output.txt")));
        assert!(!should_output_to_stdout(Path::new("/tmp/file.sh")));
    }

    // ===== DETECT PLATFORM TESTS =====

    #[test]
    fn test_detect_platform_returns_valid() {
        let platform = detect_platform();
        let valid_platforms = ["linux", "macos", "windows", "unknown"];
        assert!(valid_platforms.contains(&platform));
    }

    // ===== COUNT DUPLICATE PATH ENTRIES TESTS =====

    #[test]
    fn test_count_duplicate_path_entries_none() {
        let entries = vec![
            "/usr/bin".to_string(),
            "/usr/local/bin".to_string(),
            "/home/user/bin".to_string(),
        ];
        assert_eq!(count_duplicate_path_entries(&entries), 0);
    }

    #[test]
    fn test_count_duplicate_path_entries_some() {
        let entries = vec![
            "/usr/bin".to_string(),
            "/usr/local/bin".to_string(),
            "/usr/bin".to_string(), // duplicate
        ];
        assert_eq!(count_duplicate_path_entries(&entries), 1);
    }

    #[test]
    fn test_count_duplicate_path_entries_multiple() {
        let entries = vec![
            "/usr/bin".to_string(),
            "/usr/bin".to_string(),
            "/usr/bin".to_string(),
            "/home/user/bin".to_string(),
            "/home/user/bin".to_string(),
        ];
        assert_eq!(count_duplicate_path_entries(&entries), 3); // 2 extra /usr/bin + 1 extra /home/user/bin
    }

    #[test]
    fn test_count_duplicate_path_entries_empty() {
        let entries: Vec<String> = vec![];
        assert_eq!(count_duplicate_path_entries(&entries), 0);
    }

    // ===== ESTIMATE BUILD TIME TESTS =====

    #[test]
    fn test_estimate_build_time_small() {
        // 10 layers, 100MB, no package managers
        let time = estimate_build_time_seconds(10, 100_000_000, false, false, false);
        assert!(time >= 10); // at least 10 seconds for layers
    }

    #[test]
    fn test_estimate_build_time_large() {
        // 20 layers, 1GB, with package managers
        let time = estimate_build_time_seconds(20, 1_000_000_000, true, true, true);
        assert!(time > 30); // layers + size + package manager overhead
    }

    #[test]
    fn test_estimate_build_time_with_apt() {
        let no_apt = estimate_build_time_seconds(10, 100_000_000, false, false, false);
        let with_apt = estimate_build_time_seconds(10, 100_000_000, true, false, false);
        assert!(with_apt > no_apt);
    }

    #[test]
    fn test_estimate_build_time_with_pip() {
        let no_pip = estimate_build_time_seconds(10, 100_000_000, false, false, false);
        let with_pip = estimate_build_time_seconds(10, 100_000_000, false, true, false);
        assert!(with_pip > no_pip);
    }

    #[test]
    fn test_estimate_build_time_with_npm() {
        let no_npm = estimate_build_time_seconds(10, 100_000_000, false, false, false);
        let with_npm = estimate_build_time_seconds(10, 100_000_000, false, false, true);
        assert!(with_npm > no_npm);
    }

    // ===== DOCKERFILE PURIFICATION TESTS =====

    #[test]
    fn test_purify_dockerfile_source_basic() {
        let source = "FROM ubuntu\nRUN apt-get install -y curl\nCMD [\"bash\"]";
        let purified = purify_dockerfile_source(source, false);

        // Should pin ubuntu version
        assert!(purified.contains("ubuntu:22.04"));
        // Should add --no-install-recommends
        assert!(purified.contains("--no-install-recommends"));
        // Should add cleanup
        assert!(purified.contains("rm -rf /var/lib/apt/lists"));
        // Should add USER directive
        assert!(purified.contains("USER appuser"));
    }

    #[test]
    fn test_purify_dockerfile_source_skip_user() {
        let source = "FROM ubuntu\nCMD [\"bash\"]";
        let purified = purify_dockerfile_source(source, true);

        // Should NOT add USER directive when skip_user is true
        assert!(!purified.contains("USER appuser"));
    }

    #[test]
    fn test_purify_dockerfile_source_existing_user() {
        let source = "FROM ubuntu\nUSER myuser\nCMD [\"bash\"]";
        let purified = purify_dockerfile_source(source, false);

        // Should NOT add another USER directive
        assert!(!purified.contains("USER appuser"));
        assert!(purified.contains("USER myuser"));
    }

    #[test]
    fn test_purify_dockerfile_source_scratch() {
        let source = "FROM scratch\nCOPY binary /\nCMD [\"/binary\"]";
        let purified = purify_dockerfile_source(source, false);

        // Should NOT add USER directive for scratch images
        assert!(!purified.contains("USER appuser"));
    }

    #[test]
    fn test_dockerfile_has_user_directive() {
        assert!(dockerfile_has_user_directive(
            "FROM ubuntu\nUSER root\nCMD bash"
        ));
        assert!(!dockerfile_has_user_directive("FROM ubuntu\nCMD bash"));
        assert!(dockerfile_has_user_directive("USER nobody"));
    }

    #[test]
    fn test_dockerfile_is_scratch() {
        assert!(dockerfile_is_scratch("FROM scratch\nCOPY app /"));
        assert!(!dockerfile_is_scratch("FROM ubuntu\nCOPY app /"));
        assert!(dockerfile_is_scratch("  FROM scratch  "));
    }

    #[test]
    fn test_dockerfile_find_cmd_line() {
        assert_eq!(
            dockerfile_find_cmd_line("FROM ubuntu\nRUN apt update\nCMD bash"),
            Some(2)
        );
        assert_eq!(
            dockerfile_find_cmd_line("FROM ubuntu\nENTRYPOINT [\"app\"]"),
            Some(1)
        );
        assert_eq!(
            dockerfile_find_cmd_line("FROM ubuntu\nRUN apt update"),
            None
        );
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

    // ===== GRADE INTERPRETATION TESTS =====

    #[test]
    fn test_grade_interpretation_excellent() {
        assert!(grade_interpretation("A+").contains("Excellent"));
        assert!(grade_interpretation("A").contains("Great"));
    }

    #[test]
    fn test_grade_interpretation_good() {
        assert!(grade_interpretation("B+").contains("Good"));
        assert!(grade_interpretation("B").contains("Good"));
    }

    #[test]
    fn test_grade_interpretation_average() {
        assert!(grade_interpretation("C+").contains("Average"));
        assert!(grade_interpretation("C").contains("Average"));
    }

    #[test]
    fn test_grade_interpretation_poor() {
        assert!(grade_interpretation("D").contains("Below"));
        assert!(grade_interpretation("F").contains("Poor"));
    }

    #[test]
    fn test_grade_interpretation_unknown() {
        assert!(grade_interpretation("X").contains("Unknown"));
    }

    #[test]
    fn test_grade_symbol() {
        assert_eq!(grade_symbol("A+"), "✓");
        assert_eq!(grade_symbol("A"), "✓");
        assert_eq!(grade_symbol("B+"), "✓");
        assert_eq!(grade_symbol("B"), "✓");
        assert_eq!(grade_symbol("C+"), "⚠");
        assert_eq!(grade_symbol("C"), "⚠");
        assert_eq!(grade_symbol("D"), "⚠");
        assert_eq!(grade_symbol("F"), "✗");
        assert_eq!(grade_symbol("X"), "?");
    }

    // ===== REPORT FORMATTING TESTS =====

    #[test]
    fn test_format_purify_report_human() {
        let items = vec!["Fixed tabs".to_string(), "Added phony".to_string()];
        let report = format_purify_report_human(5, 3, 2, &items);

        assert!(report.contains("Makefile Purification Report"));
        assert!(report.contains("Transformations Applied: 5"));
        assert!(report.contains("Issues Fixed: 3"));
        assert!(report.contains("Manual Fixes Needed: 2"));
        assert!(report.contains("1: Fixed tabs"));
        assert!(report.contains("2: Added phony"));
    }

    #[test]
    fn test_format_purify_report_json() {
        let items = vec!["Fix1".to_string()];
        let report = format_purify_report_json(1, 1, 0, &items);

        assert!(report.contains("\"transformations_applied\": 1"));
        assert!(report.contains("\"issues_fixed\": 1"));
        assert!(report.contains("\"manual_fixes_needed\": 0"));
        assert!(report.contains("\"Fix1\""));
    }

    #[test]
    fn test_format_purify_report_markdown() {
        let items = vec!["Item1".to_string()];
        let report = format_purify_report_markdown(2, 1, 1, &items);

        assert!(report.contains("# Makefile Purification Report"));
        assert!(report.contains("**Transformations**: 2"));
        assert!(report.contains("1. Item1"));
    }

    // ===== SCORE FORMATTING TESTS =====

    #[test]
    fn test_format_score_human_basic() {
        let suggestions = vec!["Add tests".to_string()];
        let report = format_score_human("A", 9.0, 9.0, 9.0, 9.0, 8.0, 8.0, &suggestions, false);

        assert!(report.contains("Overall Grade: A"));
        assert!(report.contains("9.0/10.0"));
        assert!(report.contains("Add tests"));
    }

    #[test]
    fn test_format_score_human_detailed() {
        let report = format_score_human("B", 8.0, 7.0, 8.0, 9.0, 6.0, 7.0, &[], true);

        assert!(report.contains("Dimension Scores:"));
        assert!(report.contains("Complexity:"));
        assert!(report.contains("Safety:"));
        assert!(report.contains("Maintainability:"));
    }

    #[test]
    fn test_format_score_human_no_suggestions() {
        let report = format_score_human("A+", 10.0, 10.0, 10.0, 10.0, 10.0, 10.0, &[], false);

        assert!(!report.contains("Improvement Suggestions:"));
    }

    // ===== VALIDATION TESTS =====

    #[test]
    fn test_validate_proof_data_valid() {
        assert!(validate_proof_data("deadbeef", "strict", "posix"));
        assert!(validate_proof_data("0123456789abcdef", "minimal", "bash"));
    }

    #[test]
    fn test_validate_proof_data_invalid_hash() {
        assert!(!validate_proof_data("", "strict", "posix"));
        assert!(!validate_proof_data("xyz123", "strict", "posix")); // non-hex
    }

    #[test]
    fn test_validate_proof_data_empty_fields() {
        assert!(!validate_proof_data("deadbeef", "", "posix"));
        assert!(!validate_proof_data("deadbeef", "strict", ""));
    }

    // ===== SHELL DIALECT TESTS =====

    #[test]
    fn test_parse_shell_dialect_posix() {
        assert_eq!(parse_shell_dialect("posix"), Some("posix"));
        assert_eq!(parse_shell_dialect("sh"), Some("posix"));
        assert_eq!(parse_shell_dialect("POSIX"), Some("posix"));
        assert_eq!(parse_shell_dialect("SH"), Some("posix"));
    }

    #[test]
    fn test_parse_shell_dialect_bash() {
        assert_eq!(parse_shell_dialect("bash"), Some("bash"));
        assert_eq!(parse_shell_dialect("BASH"), Some("bash"));
    }

    #[test]
    fn test_parse_shell_dialect_zsh() {
        assert_eq!(parse_shell_dialect("zsh"), Some("zsh"));
        assert_eq!(parse_shell_dialect("ZSH"), Some("zsh"));
    }

    #[test]
    fn test_parse_shell_dialect_dash() {
        assert_eq!(parse_shell_dialect("dash"), Some("dash"));
    }

    #[test]
    fn test_parse_shell_dialect_unknown() {
        assert_eq!(parse_shell_dialect("fish"), None);
        assert_eq!(parse_shell_dialect("invalid"), None);
    }

    // ===== PERCENTAGE CALCULATION TESTS =====

    #[test]
    fn test_calculate_percentage_normal() {
        assert_eq!(calculate_percentage(50, 100), 50.0);
        assert_eq!(calculate_percentage(75, 100), 75.0);
        assert_eq!(calculate_percentage(1, 4), 25.0);
    }

    #[test]
    fn test_calculate_percentage_zero_total() {
        assert_eq!(calculate_percentage(0, 0), 100.0);
        assert_eq!(calculate_percentage(5, 0), 100.0);
    }

    #[test]
    fn test_calculate_percentage_full() {
        assert_eq!(calculate_percentage(100, 100), 100.0);
    }

    // ===== BYTES FORMATTING TESTS =====

    #[test]
    fn test_format_bytes_human_bytes() {
        assert_eq!(format_bytes_human(0), "0 B");
        assert_eq!(format_bytes_human(512), "512 B");
        assert_eq!(format_bytes_human(999), "999 B");
    }

    #[test]
    fn test_format_bytes_human_kb() {
        assert_eq!(format_bytes_human(1_000), "1.00 KB");
        assert_eq!(format_bytes_human(1_500), "1.50 KB");
        assert_eq!(format_bytes_human(999_999), "1000.00 KB");
    }

    #[test]
    fn test_format_bytes_human_mb() {
        assert_eq!(format_bytes_human(1_000_000), "1.00 MB");
        assert_eq!(format_bytes_human(500_000_000), "500.00 MB");
    }

    #[test]
    fn test_format_bytes_human_gb() {
        assert_eq!(format_bytes_human(1_000_000_000), "1.00 GB");
        assert_eq!(format_bytes_human(2_500_000_000), "2.50 GB");
    }

    // ===== DURATION FORMATTING TESTS =====

    #[test]
    fn test_format_duration_human_seconds() {
        assert_eq!(format_duration_human(0), "0s");
        assert_eq!(format_duration_human(30), "30s");
        assert_eq!(format_duration_human(59), "59s");
    }

    #[test]
    fn test_format_duration_human_minutes() {
        assert_eq!(format_duration_human(60), "1m 0s");
        assert_eq!(format_duration_human(90), "1m 30s");
        assert_eq!(format_duration_human(3599), "59m 59s");
    }

    #[test]
    fn test_format_duration_human_hours() {
        assert_eq!(format_duration_human(3600), "1h 0m 0s");
        assert_eq!(format_duration_human(3661), "1h 1m 1s");
        assert_eq!(format_duration_human(7325), "2h 2m 5s");
    }

    // ===== STDIO PATH TESTS =====

    #[test]
    fn test_is_stdio_path_stdin_stdout() {
        assert!(is_stdio_path(Path::new("-")));
        assert!(is_stdio_path(Path::new("/dev/stdin")));
        assert!(is_stdio_path(Path::new("/dev/stdout")));
    }

    #[test]
    fn test_is_stdio_path_regular_files() {
        assert!(!is_stdio_path(Path::new("output.txt")));
        assert!(!is_stdio_path(Path::new("/tmp/file.sh")));
        assert!(!is_stdio_path(Path::new("/dev/null")));
    }

    // ===== SIZE PARSING TESTS =====

    #[test]
    fn test_parse_size_string_gb() {
        assert_eq!(parse_size_string("1GB"), Some(1_000_000_000));
        assert_eq!(parse_size_string("2.5GB"), Some(2_500_000_000));
        assert_eq!(parse_size_string("10GB"), Some(10_000_000_000));
    }

    #[test]
    fn test_parse_size_string_mb() {
        assert_eq!(parse_size_string("100MB"), Some(100_000_000));
        assert_eq!(parse_size_string("500MB"), Some(500_000_000));
        assert_eq!(parse_size_string("1.5MB"), Some(1_500_000));
    }

    #[test]
    fn test_parse_size_string_kb() {
        assert_eq!(parse_size_string("1KB"), Some(1_000));
        assert_eq!(parse_size_string("500KB"), Some(500_000));
    }

    #[test]
    fn test_parse_size_string_bytes() {
        assert_eq!(parse_size_string("1000B"), Some(1000));
        assert_eq!(parse_size_string("1000"), Some(1000));
    }

    #[test]
    fn test_parse_size_string_case_insensitive() {
        assert_eq!(parse_size_string("1gb"), Some(1_000_000_000));
        assert_eq!(parse_size_string("1Gb"), Some(1_000_000_000));
        assert_eq!(parse_size_string("1mb"), Some(1_000_000));
    }

    #[test]
    fn test_parse_size_string_with_spaces() {
        assert_eq!(parse_size_string("  1GB  "), Some(1_000_000_000));
        // "500 MB" works because the number part gets trimmed after extracting
        assert_eq!(parse_size_string("500 MB"), Some(500_000_000));
    }

    #[test]
    fn test_parse_size_string_invalid() {
        assert_eq!(parse_size_string("invalid"), None);
        assert_eq!(parse_size_string(""), None);
        assert_eq!(parse_size_string("GB"), None);
    }

    // ===== BUILD TIME ESTIMATE TESTS =====

    #[test]
    fn test_format_build_time_estimate_seconds() {
        let result = format_build_time_estimate(5, 100_000_000, false, false, false);
        assert!(result.starts_with("~"));
        assert!(result.contains('s'));
    }

    #[test]
    fn test_format_build_time_estimate_minutes() {
        // Large enough to be over 60 seconds
        let result = format_build_time_estimate(50, 5_000_000_000, true, true, true);
        assert!(result.contains('m'));
    }

    // ===== SIZE LIMIT TESTS =====

    #[test]
    fn test_size_exceeds_limit() {
        assert!(size_exceeds_limit(2_000_000_000, 1_000_000_000));
        assert!(!size_exceeds_limit(500_000_000, 1_000_000_000));
        assert!(!size_exceeds_limit(1_000_000_000, 1_000_000_000));
    }

    #[test]
    fn test_size_percentage_of_limit() {
        assert_eq!(size_percentage_of_limit(500_000_000, 1_000_000_000), 50.0);
        assert_eq!(
            size_percentage_of_limit(1_000_000_000, 1_000_000_000),
            100.0
        );
        assert_eq!(size_percentage_of_limit(250_000_000, 1_000_000_000), 25.0);
    }

    #[test]
    fn test_size_percentage_of_limit_zero() {
        assert_eq!(size_percentage_of_limit(100, 0), 100.0);
    }

    // ===== LAYER OPERATION TESTS =====

    #[test]
    fn test_layer_has_slow_operation_apt() {
        let (apt, pip, npm) = layer_has_slow_operation("RUN apt-get install -y curl");
        assert!(apt);
        assert!(!pip);
        assert!(!npm);
    }

    #[test]
    fn test_layer_has_slow_operation_pip() {
        let (apt, pip, npm) = layer_has_slow_operation("RUN pip install requests");
        assert!(!apt);
        assert!(pip);
        assert!(!npm);
    }

    #[test]
    fn test_layer_has_slow_operation_npm() {
        let (apt, pip, npm) = layer_has_slow_operation("RUN npm install express");
        assert!(!apt);
        assert!(!pip);
        assert!(npm);
    }

    #[test]
    fn test_layer_has_slow_operation_yarn() {
        let (apt, pip, npm) = layer_has_slow_operation("RUN yarn install");
        assert!(!apt);
        assert!(!pip);
        assert!(npm); // yarn counts as npm-like
    }

    #[test]
    fn test_layer_has_slow_operation_none() {
        let (apt, pip, npm) = layer_has_slow_operation("RUN echo hello");
        assert!(!apt);
        assert!(!pip);
        assert!(!npm);
    }

    #[test]
    fn test_layer_has_slow_operation_multiple() {
        let (apt, pip, npm) =
            layer_has_slow_operation("RUN apt-get install && pip install && npm install");
        assert!(apt);
        assert!(pip);
        assert!(npm);
    }

    // ===== SIZE COMPARISON TESTS =====

    #[test]
    fn test_format_size_comparison_within_limit() {
        let result = format_size_comparison(500_000_000, 1_000_000_000);
        assert!(result.contains("✓"));
        assert!(result.contains("Within limit"));
        assert!(result.contains("50%"));
    }

    #[test]
    fn test_format_size_comparison_exceeds() {
        let result = format_size_comparison(2_000_000_000, 1_000_000_000);
        assert!(result.contains("✗"));
        assert!(result.contains("EXCEEDS"));
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

    // ===== TEST RESULT STATUS TESTS =====

    #[test]
    fn test_test_result_status_passed() {
        assert_eq!(test_result_status(10, 0, 10), "PASSED");
    }

    #[test]
    fn test_test_result_status_failed() {
        assert_eq!(test_result_status(8, 2, 10), "FAILED");
    }

    #[test]
    fn test_test_result_status_no_tests() {
        assert_eq!(test_result_status(0, 0, 0), "NO TESTS");
    }

    #[test]
    fn test_test_result_status_partial() {
        assert_eq!(test_result_status(5, 0, 10), "PARTIAL");
    }

    // ===== TEST PASS RATE TESTS =====

    #[test]
    fn test_test_pass_rate_all_passed() {
        assert_eq!(test_pass_rate(10, 10), 100.0);
    }

    #[test]
    fn test_test_pass_rate_half() {
        assert_eq!(test_pass_rate(5, 10), 50.0);
    }

    #[test]
    fn test_test_pass_rate_none() {
        assert_eq!(test_pass_rate(0, 10), 0.0);
    }

    #[test]
    fn test_test_pass_rate_no_tests() {
        assert_eq!(test_pass_rate(0, 0), 100.0);
    }
}
