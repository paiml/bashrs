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
    let mut parser =
        BashParser::new(source).map_err(|e| Error::Internal(format!("Parse: {}", e)))?;
    let ast = parser
        .parse()
        .map_err(|e| Error::Internal(format!("Parse: {}", e)))?;
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

/// Truncate string to max length with ellipsis
pub fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
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
    if percent >= 80.0 {
        "good"
    } else if percent >= 50.0 {
        "medium"
    } else {
        "poor"
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
    fn test_coverage_class_good() {
        assert_eq!(coverage_class(90.0), "good");
        assert_eq!(coverage_class(80.0), "good");
    }

    #[test]
    fn test_coverage_class_medium() {
        assert_eq!(coverage_class(75.0), "medium");
        assert_eq!(coverage_class(50.0), "medium");
    }

    #[test]
    fn test_coverage_class_poor() {
        assert_eq!(coverage_class(49.0), "poor");
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
}
