//! `bashrs explain` command (SSC v11 Section 8.1).
//!
//! Generates natural-language safety explanations from linter findings.
//! Stage 0 implementation: pure rule-based analysis.
//! Future: Stage 2 (Qwen-1.5B chat) will provide richer explanations.
//!
//! ```text
//! bashrs explain script.sh
//!     ├── lint (<1ms)
//!     ├── classify findings by category
//!     ├── generate explanation per finding
//!     v
//!     Output: structured safety explanation
//! ```

use crate::cli::args::ClassifyFormat;
use crate::linter::{lint_dockerfile_with_profile, lint_makefile, lint_shell, LintProfile};
use crate::models::{Error, Result};
use serde::Serialize;
use std::path::Path;

/// A complete safety explanation report.
#[derive(Debug, Serialize)]
struct ExplainReport {
    /// Overall safety verdict
    verdict: String,
    /// Risk level: "safe", "low", "medium", "high", "critical"
    risk_level: String,
    /// Detected script format
    format: String,
    /// Natural-language summary (1-2 sentences)
    summary: String,
    /// Categorized explanations
    categories: Vec<CategoryExplanation>,
    /// Suggested next steps
    recommendations: Vec<String>,
}

/// Explanation for a category of findings.
#[derive(Debug, Serialize)]
struct CategoryExplanation {
    /// Category name (e.g., "Security", "Determinism")
    category: String,
    /// Number of findings in this category
    count: usize,
    /// Natural-language explanation of the category risk
    explanation: String,
    /// Individual finding details
    findings: Vec<FindingExplanation>,
}

/// Explanation for a single finding.
#[derive(Debug, Serialize)]
struct FindingExplanation {
    /// Rule code
    code: String,
    /// Line number
    line: usize,
    /// What the issue is (plain English)
    what: String,
    /// Why it matters
    why: String,
    /// How to fix it
    fix: String,
}

/// Detect format from file path (shared logic with safety_check).
fn detect_format(path: &Path) -> ClassifyFormat {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase()
        .as_str()
    {
        "sh" | "bash" | "zsh" | "ksh" | "dash" => ClassifyFormat::Bash,
        _ => {
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_lowercase();
            if name == "makefile" || name == "gnumakefile" || name.ends_with(".mk") {
                ClassifyFormat::Makefile
            } else if name == "dockerfile"
                || name.starts_with("dockerfile.")
                || name.ends_with(".dockerfile")
            {
                ClassifyFormat::Dockerfile
            } else {
                ClassifyFormat::Bash
            }
        }
    }
}

/// Entry point for `bashrs explain`.
pub(crate) fn explain_command(
    input: &Path,
    json: bool,
    forced_format: Option<&ClassifyFormat>,
) -> Result<()> {
    let source = std::fs::read_to_string(input)
        .map_err(|e| Error::Validation(format!("Cannot read {}: {e}", input.display())))?;

    let fmt = forced_format
        .cloned()
        .unwrap_or_else(|| detect_format(input));

    let report = generate_explanation(&source, &fmt);

    if json {
        let json_str = serde_json::to_string_pretty(&report)
            .map_err(|e| Error::Validation(format!("JSON serialization failed: {e}")))?;
        println!("{json_str}");
    } else {
        print_explanation(&report);
    }

    Ok(())
}

/// Generate a full explanation report from source code.
fn generate_explanation(source: &str, fmt: &ClassifyFormat) -> ExplainReport {
    let diagnostics = match fmt {
        ClassifyFormat::Bash => lint_shell(source).diagnostics,
        ClassifyFormat::Makefile => lint_makefile(source).diagnostics,
        ClassifyFormat::Dockerfile => {
            lint_dockerfile_with_profile(source, LintProfile::Standard).diagnostics
        }
    };

    let categories = partition_into_categories(&diagnostics);
    let (risk_level, verdict) = assess_risk(&categories, diagnostics.is_empty());
    let summary = build_summary(&categories, diagnostics.len());

    let has_security = categories.iter().any(|c| c.category == "Security");
    let has_determinism = categories.iter().any(|c| c.category == "Determinism");
    let has_idempotency = categories.iter().any(|c| c.category == "Idempotency");
    let recommendations =
        build_recommendations(&categories, has_security, has_determinism, has_idempotency);

    let format_name = match fmt {
        ClassifyFormat::Bash => "bash",
        ClassifyFormat::Makefile => "makefile",
        ClassifyFormat::Dockerfile => "dockerfile",
    };

    ExplainReport {
        verdict,
        risk_level,
        format: format_name.to_string(),
        summary,
        categories,
        recommendations,
    }
}

/// Classify a diagnostic code into a category bucket.
fn classify_code(code: &str) -> &'static str {
    if code.starts_with("SEC") || code == "DOCKER001" || code == "DOCKER006" || code == "MAKE003" {
        "Security"
    } else if code.starts_with("DET") || code == "DOCKER002" || code == "MAKE001" {
        "Determinism"
    } else if code.starts_with("IDEM") || code == "MAKE002" {
        "Idempotency"
    } else {
        "Style"
    }
}

/// Partition diagnostics into categorized explanations.
fn partition_into_categories(
    diagnostics: &[crate::linter::Diagnostic],
) -> Vec<CategoryExplanation> {
    let mut sec = Vec::new();
    let mut det = Vec::new();
    let mut idem = Vec::new();
    let mut other = Vec::new();

    for d in diagnostics {
        let explanation = FindingExplanation {
            code: d.code.clone(),
            line: d.span.start_line,
            what: d.message.clone(),
            why: explain_why(&d.code),
            fix: explain_fix(&d.code),
        };

        match classify_code(&d.code) {
            "Security" => sec.push(explanation),
            "Determinism" => det.push(explanation),
            "Idempotency" => idem.push(explanation),
            _ => other.push(explanation),
        }
    }

    let mut categories = Vec::new();
    push_category(&mut categories, "Security", sec,
        "These patterns can allow attackers to execute arbitrary commands, read sensitive files, or escalate privileges.");
    push_category(&mut categories, "Determinism", det,
        "These patterns produce different results on each run, making the script unreliable for automation and CI/CD.");
    push_category(&mut categories, "Idempotency", idem,
        "These operations are not safe to re-run — running the script twice may produce errors or unintended side effects.");
    push_category(&mut categories, "Style & Best Practices", other,
        "While not security-critical, fixing these improves readability and maintainability.");
    categories
}

/// Push a category if it has findings.
fn push_category(
    categories: &mut Vec<CategoryExplanation>,
    name: &str,
    findings: Vec<FindingExplanation>,
    description: &str,
) {
    if findings.is_empty() {
        return;
    }
    let count = findings.len();
    let noun = name.to_lowercase();
    categories.push(CategoryExplanation {
        category: name.to_string(),
        count,
        explanation: format!(
            "Found {count} {noun} issue{}. {description}",
            if count == 1 { "" } else { "s" }
        ),
        findings,
    });
}

/// Determine risk level and verdict from categories.
fn assess_risk(categories: &[CategoryExplanation], no_diagnostics: bool) -> (String, String) {
    let has_security = categories.iter().any(|c| c.category == "Security");
    let has_determinism = categories.iter().any(|c| c.category == "Determinism");
    let has_idempotency = categories.iter().any(|c| c.category == "Idempotency");
    let sec_count = categories
        .iter()
        .find(|c| c.category == "Security")
        .map_or(0, |c| c.count);

    let risk_level = if sec_count >= 3 {
        "critical"
    } else if has_security {
        "high"
    } else if has_determinism {
        "medium"
    } else if has_idempotency || !no_diagnostics {
        "low"
    } else {
        "safe"
    };

    let verdict = if no_diagnostics {
        "No safety issues detected."
    } else if has_security {
        "This script has security vulnerabilities that should be fixed before deployment."
    } else if has_determinism {
        "This script has non-deterministic behavior that may cause inconsistent results."
    } else if has_idempotency {
        "This script has idempotency issues — it may not be safe to run multiple times."
    } else {
        "This script has minor quality issues but no critical safety problems."
    };

    (risk_level.to_string(), verdict.to_string())
}

fn build_summary(categories: &[CategoryExplanation], total: usize) -> String {
    if total == 0 {
        return "This script passes all safety checks. No security vulnerabilities, \
                non-deterministic patterns, or idempotency issues were detected."
            .to_string();
    }

    let parts: Vec<String> = categories
        .iter()
        .map(|c| format!("{} {}", c.count, c.category.to_lowercase()))
        .collect();

    format!(
        "Analysis found {total} issue{}: {}.",
        if total == 1 { "" } else { "s" },
        parts.join(", ")
    )
}

fn build_recommendations(
    categories: &[CategoryExplanation],
    has_security: bool,
    has_determinism: bool,
    has_idempotency: bool,
) -> Vec<String> {
    let mut recs = Vec::new();

    if has_security {
        recs.push(
            "Fix security issues first — they represent the highest risk. \
             Run `bashrs lint --fix` to apply automatic fixes where available."
                .to_string(),
        );
    }

    if has_determinism {
        recs.push(
            "Replace non-deterministic patterns with parameters or fixed values. \
             Use `bashrs purify` to automatically apply determinism transformations."
                .to_string(),
        );
    }

    if has_idempotency {
        recs.push(
            "Add idempotency guards (mkdir -p, rm -f, ln -sf) so the script \
             is safe to re-run. Use `bashrs purify` to apply these automatically."
                .to_string(),
        );
    }

    if categories.is_empty() {
        recs.push("No issues found. This script is ready for deployment.".to_string());
    } else {
        recs.push(
            "Run `bashrs safety-check` for a machine-readable safety classification.".to_string(),
        );
    }

    recs
}

/// Return a "why this matters" explanation for a rule code.
fn explain_why(code: &str) -> String {
    match code {
        "SEC001" => "eval() executes arbitrary strings as code, enabling command injection attacks.",
        "SEC002" => "Unquoted variables expand unsafely — spaces and glob characters can alter command behavior.",
        "SEC003" => "Executing code downloaded from the internet (curl|sh) bypasses all review and verification.",
        "SEC004" => "Hardcoded credentials in scripts can be extracted from version control history.",
        "SEC005" => "Temporary files with predictable names enable symlink attacks and race conditions.",
        "SEC006" => "World-writable permissions (chmod 777) allow any user to modify files.",
        "SEC007" => "Running as root without checks risks destructive operations affecting the entire system.",
        "SEC008" => "Unsanitized input in SQL or command strings enables injection attacks.",
        "SEC010" => "Source/dot-sourcing external files executes untrusted code in the current shell.",
        "SEC016" => "Passing unvalidated positional parameters to dangerous commands enables injection.",
        "SEC019" => "Unquoted variable in command position can execute arbitrary commands.",
        "SEC020" => "Passing variables to awk/sed system() calls enables command injection.",
        "SEC021" => "Destructive system operations (disk wipe, fork bomb, rm -rf /) can destroy data.",
        "SEC022" => "Privilege escalation (setuid, chmod +s, sudoers) grants elevated access.",
        "SEC023" => "Data exfiltration (reverse shells, DNS exfil, curl POST) leaks sensitive data.",
        "SEC024" => "Race conditions (TOCTOU, symlink attacks) enable privilege escalation.",
        "DET001" => "$RANDOM produces different values on each run, making output unpredictable.",
        "DET002" => "date/time commands produce different output on each run.",
        "DET003" => "$$ (process ID) changes on each invocation, breaking reproducibility.",
        "DET004" => "System state commands (df, free, ps, etc.) return different values each time.",
        "IDEM001" => "mkdir without -p fails if the directory already exists.",
        "IDEM002" => "rm without -f fails if the file doesn't exist.",
        "IDEM003" => "ln without -sf fails if the link already exists.",
        _ => "This pattern may cause unexpected behavior in certain environments.",
    }
    .to_string()
}

/// Return a "how to fix" suggestion for a rule code.
fn explain_fix(code: &str) -> String {
    match code {
        "SEC001" => "Replace eval with direct command execution or a safer alternative like a case statement.",
        "SEC002" => "Quote all variable expansions: use \"$var\" instead of $var.",
        "SEC003" => "Download scripts to a file first, review them, then execute.",
        "SEC004" => "Use environment variables or a secrets manager instead of hardcoded values.",
        "SEC005" => "Use mktemp to create temporary files with unpredictable names.",
        "SEC006" => "Use specific permissions (e.g., chmod 644 for files, 755 for executables).",
        "SEC007" => "Add a root check: [ \"$(id -u)\" -eq 0 ] || exit 1",
        "SEC008" => "Use parameterized queries or properly escape/validate all inputs.",
        "SEC010" => "Verify the sourced file's integrity (checksum) before sourcing.",
        "SEC016" => "Validate positional parameters before passing to commands like eval, exec, or su.",
        "SEC019" => "Quote the variable or use a case statement to restrict allowed commands.",
        "SEC020" => "Pass data to awk/sed via variables, not through shell interpolation.",
        "SEC021" => "Remove destructive commands or add confirmation prompts and dry-run modes.",
        "SEC022" => "Use minimal required privileges. Avoid setuid/chmod +s on untrusted binaries.",
        "SEC023" => "Remove exfiltration vectors. Use firewall rules to restrict outbound connections.",
        "SEC024" => "Use atomic operations (mv, flock) instead of check-then-act sequences.",
        "DET001" => "Accept randomness as a parameter: ${SEED:-42} instead of $RANDOM.",
        "DET002" => "Use a fixed timestamp parameter: ${BUILD_TIME:-$(date +%s)}",
        "DET003" => "Use a fixed identifier instead of $$: ${RUN_ID:-default}",
        "DET004" => "Pass system state as parameters instead of querying at runtime.",
        "IDEM001" => "Use mkdir -p to create directories idempotently.",
        "IDEM002" => "Use rm -f to remove files without failing if absent.",
        "IDEM003" => "Use ln -sf to create symlinks idempotently.",
        _ => "Review the flagged line and apply the suggested fix from `bashrs lint --fix`.",
    }
    .to_string()
}

/// Print human-readable explanation.
fn print_explanation(report: &ExplainReport) {
    use crate::cli::color::*;

    let risk_color = match report.risk_level.as_str() {
        "safe" => GREEN,
        "low" => YELLOW,
        "medium" => YELLOW,
        "high" | "critical" => RED,
        _ => RESET,
    };

    println!(
        "{BOLD}Safety Explanation{RESET} [{risk_color}{}{RESET}]\n",
        report.risk_level.to_uppercase()
    );
    println!("{}", report.verdict);
    println!();
    println!("{}", report.summary);

    for cat in &report.categories {
        println!(
            "\n{BOLD}--- {} ({} issue{}) ---{RESET}",
            cat.category,
            cat.count,
            if cat.count == 1 { "" } else { "s" }
        );
        println!("{}", cat.explanation);

        for f in &cat.findings {
            println!("\n  {BOLD}L{} [{}]{RESET}", f.line, f.code);
            println!("  What: {}", f.what);
            println!("  Why:  {}", f.why);
            println!("  Fix:  {}", f.fix);
        }
    }

    if !report.recommendations.is_empty() {
        println!("\n{BOLD}Recommendations:{RESET}");
        for rec in &report.recommendations {
            println!("  - {rec}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_explain_safe_script() {
        let report = generate_explanation("#!/bin/sh\necho hello\n", &ClassifyFormat::Bash);
        assert_eq!(report.risk_level, "safe");
        assert!(report.categories.is_empty());
        assert!(report.summary.contains("passes all safety checks"));
    }

    #[test]
    fn test_explain_eval_security() {
        let report =
            generate_explanation("#!/bin/bash\neval \"$user_input\"\n", &ClassifyFormat::Bash);
        assert_eq!(report.risk_level, "high");
        assert!(report.categories.iter().any(|c| c.category == "Security"));
        let sec = report
            .categories
            .iter()
            .find(|c| c.category == "Security")
            .expect("should have security category");
        assert!(sec.findings.iter().any(|f| f.code.starts_with("SEC")));
    }

    #[test]
    fn test_explain_nondeterministic() {
        let report =
            generate_explanation("#!/bin/bash\necho $RANDOM\n", &ClassifyFormat::Bash);
        assert!(report.risk_level == "medium" || report.risk_level == "high");
        assert!(report
            .categories
            .iter()
            .any(|c| c.category == "Determinism"));
    }

    #[test]
    fn test_explain_idempotency() {
        let report =
            generate_explanation("#!/bin/sh\nmkdir /tmp/test\n", &ClassifyFormat::Bash);
        assert!(report
            .categories
            .iter()
            .any(|c| c.category == "Idempotency"));
    }

    #[test]
    fn test_explain_json_serializable() {
        let report = generate_explanation("#!/bin/sh\necho ok\n", &ClassifyFormat::Bash);
        let json = serde_json::to_string_pretty(&report);
        assert!(json.is_ok());
        let s = json.expect("should serialize");
        assert!(s.contains("\"verdict\""));
        assert!(s.contains("\"risk_level\""));
        assert!(s.contains("\"recommendations\""));
    }

    #[test]
    fn test_explain_makefile() {
        let report = generate_explanation(
            ".PHONY: build\nbuild:\n\techo ok\n",
            &ClassifyFormat::Makefile,
        );
        assert_eq!(report.format, "makefile");
    }

    #[test]
    fn test_explain_dockerfile() {
        let report = generate_explanation(
            "FROM alpine:3.18\nUSER nobody\nCOPY app /app\n",
            &ClassifyFormat::Dockerfile,
        );
        assert_eq!(report.format, "dockerfile");
    }

    #[test]
    fn test_explain_critical_risk() {
        // Multiple security issues should escalate to critical
        let report = generate_explanation(
            "#!/bin/bash\neval \"$1\"\neval \"$2\"\neval \"$3\"\n",
            &ClassifyFormat::Bash,
        );
        assert_eq!(report.risk_level, "critical");
    }

    #[test]
    fn test_explain_why_coverage() {
        // Verify common rule codes return meaningful explanations
        for code in &[
            "SEC001", "SEC002", "SEC003", "SEC004", "SEC005", "SEC006", "SEC007", "SEC008",
            "SEC010", "SEC016", "SEC019", "SEC020", "SEC021", "SEC022", "SEC023", "SEC024",
            "DET001", "DET002", "DET003", "DET004", "IDEM001", "IDEM002", "IDEM003",
        ] {
            let why = explain_why(code);
            assert!(
                !why.contains("unexpected behavior"),
                "{code} should have a specific explanation"
            );
        }
    }

    #[test]
    fn test_explain_fix_coverage() {
        for code in &[
            "SEC001", "SEC002", "SEC003", "SEC004", "SEC005", "SEC006", "SEC007", "SEC008",
            "SEC010", "SEC016", "SEC019", "SEC020", "SEC021", "SEC022", "SEC023", "SEC024",
            "DET001", "DET002", "DET003", "DET004", "IDEM001", "IDEM002", "IDEM003",
        ] {
            let fix = explain_fix(code);
            assert!(
                !fix.contains("Review the flagged line"),
                "{code} should have a specific fix"
            );
        }
    }

    #[test]
    fn test_recommendations_include_purify() {
        let report =
            generate_explanation("#!/bin/bash\necho $RANDOM\n", &ClassifyFormat::Bash);
        assert!(report
            .recommendations
            .iter()
            .any(|r| r.contains("purify")));
    }

    #[test]
    fn test_detect_format_extensions() {
        assert!(matches!(
            detect_format(Path::new("test.sh")),
            ClassifyFormat::Bash
        ));
        assert!(matches!(
            detect_format(Path::new("Makefile")),
            ClassifyFormat::Makefile
        ));
        assert!(matches!(
            detect_format(Path::new("Dockerfile")),
            ClassifyFormat::Dockerfile
        ));
    }
}
