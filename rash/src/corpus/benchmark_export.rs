//! ShellSafetyBench DPO-compatible benchmark export (SSC v12 S14.4).
//!
//! Exports corpus entries in the ShellSafetyBench schema:
//! ```json
//! {
//!   "id": "SSB-00142",
//!   "lang": "bash",
//!   "cwe": "CWE-78",
//!   "rule": "SEC001",
//!   "severity": "error",
//!   "script": "#!/bin/bash\nrm -rf $USER_INPUT/tmp",
//!   "chosen": "Classification: unsafe\n\n**SEC001** ...",
//!   "rejected": "This script looks fine.",
//!   "source": "bashrs-corpus",
//!   "conversation_type": "classify-explain"
//! }
//! ```

use crate::corpus::cwe_mapping;
use crate::corpus::registry::{CorpusEntry, CorpusFormat, CorpusRegistry};
use crate::linter::{self, LintProfile};
use crate::models::Config;
use serde::Serialize;

/// A single ShellSafetyBench entry in DPO-compatible format.
#[derive(Debug, Serialize)]
pub struct BenchmarkEntry {
    /// Unique identifier (SSB-NNNNN)
    pub id: String,
    /// Language: bash, makefile, or dockerfile
    pub lang: String,
    /// Primary CWE ID (e.g., "CWE-78") or "none" if safe
    pub cwe: String,
    /// Primary bashrs rule ID (e.g., "SEC001") or "none" if safe
    pub rule: String,
    /// Severity: error, warning, info, or safe
    pub severity: String,
    /// The raw script (shell code, not Rust DSL)
    pub script: String,
    /// Chosen (correct) response for DPO
    pub chosen: String,
    /// Rejected (incorrect) response for DPO
    pub rejected: String,
    /// Data source
    pub source: String,
    /// Conversation type
    pub conversation_type: String,
}

/// Export results summary.
#[derive(Debug)]
pub struct ExportSummary {
    pub total: usize,
    pub safe: usize,
    pub unsafe_count: usize,
    pub by_lang: std::collections::HashMap<String, usize>,
    pub by_cwe: std::collections::HashMap<String, usize>,
}

/// Export corpus entries as ShellSafetyBench DPO-compatible JSONL.
pub fn export_benchmark(
    registry: &CorpusRegistry,
    limit: Option<usize>,
) -> (Vec<BenchmarkEntry>, ExportSummary) {
    let config = Config::default();
    let max = limit.unwrap_or(registry.entries.len());
    let mut entries = Vec::new();
    let mut summary = ExportSummary {
        total: 0,
        safe: 0,
        unsafe_count: 0,
        by_lang: std::collections::HashMap::new(),
        by_cwe: std::collections::HashMap::new(),
    };

    for (idx, corpus_entry) in registry.entries.iter().take(max).enumerate() {
        // Transpile Rust DSL → shell code
        let shell_code = transpile_entry(corpus_entry, &config);
        if shell_code.is_empty() {
            continue;
        }

        // Lint the shell code to find violations
        let diagnostics = lint_entry(&shell_code, corpus_entry.format);

        let lang = format_to_lang(corpus_entry.format);
        let id = format!("SSB-{:05}", idx + 1);

        // Only SEC/DET/IDEM rules are "unsafe" indicators (same as baselines module).
        // SC/REL rules fire on preamble and are false positives for safety labeling.
        let security_diags: Vec<&LintDiagnostic> = diagnostics
            .iter()
            .filter(|d| {
                d.rule.starts_with("SEC") || d.rule.starts_with("DET") || d.rule.starts_with("IDEM")
            })
            .collect();

        let (rule, cwe, severity, is_unsafe) = if let Some(first_diag) = security_diags.first() {
            let rule_id = &first_diag.rule;
            let cwe_str = cwe_mapping::lookup_rule(rule_id)
                .map(|m| m.cwe.to_string())
                .unwrap_or_else(|| "CWE-unknown".to_string());
            (rule_id.clone(), cwe_str, first_diag.severity.clone(), true)
        } else {
            (
                "none".to_string(),
                "none".to_string(),
                "safe".to_string(),
                false,
            )
        };

        let sec_diag_refs: Vec<&LintDiagnostic> = security_diags.into_iter().collect();
        let chosen = build_chosen_response(&sec_diag_refs, &shell_code, is_unsafe);
        let rejected = build_rejected_response(is_unsafe);

        summary.total += 1;
        if is_unsafe {
            summary.unsafe_count += 1;
        } else {
            summary.safe += 1;
        }
        *summary.by_lang.entry(lang.clone()).or_insert(0) += 1;
        if cwe != "none" {
            *summary.by_cwe.entry(cwe.clone()).or_insert(0) += 1;
        }

        entries.push(BenchmarkEntry {
            id,
            lang,
            cwe,
            rule,
            severity,
            script: shell_code,
            chosen,
            rejected,
            source: "bashrs-corpus".to_string(),
            conversation_type: if is_unsafe {
                "classify-explain".to_string()
            } else {
                "confirm-safe".to_string()
            },
        });
    }

    (entries, summary)
}

fn transpile_entry(entry: &CorpusEntry, config: &Config) -> String {
    match entry.format {
        CorpusFormat::Bash => crate::transpile(&entry.input, config).unwrap_or_default(),
        CorpusFormat::Makefile => {
            crate::transpile_makefile(&entry.input, config).unwrap_or_default()
        }
        CorpusFormat::Dockerfile => {
            crate::transpile_dockerfile(&entry.input, config).unwrap_or_default()
        }
    }
}

/// Lint result for a single diagnostic.
#[derive(Debug)]
pub struct LintDiagnostic {
    pub rule: String,
    pub severity: String,
    pub message: String,
    pub line: usize,
}

fn lint_entry(shell_code: &str, format: CorpusFormat) -> Vec<LintDiagnostic> {
    let result = match format {
        CorpusFormat::Bash => linter::lint_shell(shell_code),
        CorpusFormat::Makefile => linter::lint_makefile(shell_code),
        CorpusFormat::Dockerfile => {
            linter::lint_dockerfile_with_profile(shell_code, LintProfile::Standard)
        }
    };

    result
        .diagnostics
        .iter()
        .map(|d| LintDiagnostic {
            rule: d.code.clone(),
            severity: format!("{:?}", d.severity).to_lowercase(),
            message: d.message.clone(),
            line: d.span.start_line,
        })
        .collect()
}

fn format_to_lang(format: CorpusFormat) -> String {
    match format {
        CorpusFormat::Bash => "bash".to_string(),
        CorpusFormat::Makefile => "makefile".to_string(),
        CorpusFormat::Dockerfile => "dockerfile".to_string(),
    }
}

fn build_chosen_response(
    diagnostics: &[&LintDiagnostic],
    shell_code: &str,
    is_unsafe: bool,
) -> String {
    if !is_unsafe {
        return "Classification: safe\n\nThis script does not contain known unsafe patterns. \
             It follows good practices for shell scripting."
            .to_string();
    }

    let mut response = String::from("Classification: unsafe\n\n");

    for d in diagnostics {
        response.push_str(&format!(
            "**{}** (line {}): {}\n",
            d.rule, d.line, d.message
        ));

        // Add CWE reference if available
        if let Some(mapping) = cwe_mapping::lookup_rule(&d.rule) {
            response.push_str(&format!(
                "  CWE: {} — {} (CVSS {:.1} {})\n",
                mapping.cwe, mapping.owasp, mapping.cvss_score, mapping.cvss_severity
            ));
        }
    }

    // Add a simple fix suggestion
    response.push_str(&format!(
        "\nOriginal:\n```bash\n{}\n```",
        shell_code.lines().take(10).collect::<Vec<_>>().join("\n")
    ));

    response
}

fn build_rejected_response(is_unsafe: bool) -> String {
    if is_unsafe {
        // Plausibly wrong response (not a strawman)
        "This script looks fine. It performs standard operations without any obvious issues. \
         I don't see any security concerns."
            .to_string()
    } else {
        // Plausibly wrong for safe scripts: false alarm
        "This script has potential security issues. The use of shell commands \
         could be dangerous and should be reviewed carefully."
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::corpus::registry::CorpusRegistry;

    #[test]
    fn test_export_produces_entries() {
        let registry = CorpusRegistry::load_full();
        let (entries, summary) = export_benchmark(&registry, Some(500));

        assert!(entries.len() > 100, "Should have >100 entries");
        assert!(summary.total > 100);
        assert!(summary.safe > 0, "Should have some safe entries");
        assert!(summary.unsafe_count > 0, "Should have some unsafe entries");
    }

    #[test]
    fn test_entry_has_required_fields() {
        let registry = CorpusRegistry::load_full();
        let (entries, _) = export_benchmark(&registry, Some(200));

        for entry in entries.iter().take(10) {
            assert!(entry.id.starts_with("SSB-"), "ID must start with SSB-");
            assert!(
                ["bash", "makefile", "dockerfile"].contains(&entry.lang.as_str()),
                "Invalid lang: {}",
                entry.lang
            );
            assert!(!entry.script.is_empty(), "Script must not be empty");
            assert!(!entry.chosen.is_empty(), "Chosen must not be empty");
            assert!(!entry.rejected.is_empty(), "Rejected must not be empty");
            assert!(!entry.source.is_empty(), "Source must not be empty");
            assert!(!entry.conversation_type.is_empty());
        }
    }

    #[test]
    fn test_unsafe_entry_has_cwe() {
        let registry = CorpusRegistry::load_full();
        let (entries, _) = export_benchmark(&registry, Some(200));

        // Find first unsafe entry
        let unsafe_entry = entries
            .iter()
            .find(|e| e.conversation_type == "classify-explain");

        if let Some(entry) = unsafe_entry {
            assert_ne!(entry.cwe, "none", "Unsafe entry must have CWE");
            assert_ne!(entry.rule, "none", "Unsafe entry must have rule");
            assert!(
                entry.chosen.contains("Classification: unsafe"),
                "Chosen must start with unsafe classification"
            );
        }
    }

    #[test]
    fn test_safe_entry_is_correct() {
        let registry = CorpusRegistry::load_full();
        let (entries, _) = export_benchmark(&registry, Some(200));

        let safe_entry = entries
            .iter()
            .find(|e| e.conversation_type == "confirm-safe");

        if let Some(entry) = safe_entry {
            assert_eq!(entry.cwe, "none");
            assert_eq!(entry.rule, "none");
            assert_eq!(entry.severity, "safe");
            assert!(entry.chosen.contains("Classification: safe"));
        }
    }

    #[test]
    fn test_dpo_format_serializable() {
        let entry = BenchmarkEntry {
            id: "SSB-00001".to_string(),
            lang: "bash".to_string(),
            cwe: "CWE-78".to_string(),
            rule: "SEC001".to_string(),
            severity: "error".to_string(),
            script: "#!/bin/bash\nrm -rf $USER_INPUT".to_string(),
            chosen: "Classification: unsafe".to_string(),
            rejected: "Looks fine".to_string(),
            source: "bashrs-corpus".to_string(),
            conversation_type: "classify-explain".to_string(),
        };

        let json = serde_json::to_string(&entry).expect("serialize");
        assert!(json.contains("SSB-00001"));
        assert!(json.contains("CWE-78"));
    }

    #[test]
    fn test_summary_by_lang() {
        let registry = CorpusRegistry::load_full();
        let (_, summary) = export_benchmark(&registry, Some(500));

        assert!(
            summary.by_lang.contains_key("bash"),
            "Should have bash entries"
        );
    }

    /// FALSIFY-SSB-003: Benchmark export has DPO-compatible schema.
    /// All entries must have: id, lang, cwe, rule, severity, script, chosen, rejected.
    #[test]
    fn test_benchmark_dpo_schema() {
        let registry = CorpusRegistry::load_full();
        let (entries, _) = export_benchmark(&registry, Some(200));

        assert!(
            entries.len() >= 17000,
            "Expected >=17000, got {}",
            entries.len()
        );

        for entry in &entries {
            // Required fields must be non-empty
            assert!(!entry.id.is_empty(), "id must not be empty");
            assert!(!entry.lang.is_empty(), "lang must not be empty");
            assert!(!entry.cwe.is_empty(), "cwe must not be empty");
            assert!(!entry.rule.is_empty(), "rule must not be empty");
            assert!(!entry.severity.is_empty(), "severity must not be empty");
            assert!(!entry.script.is_empty(), "script must not be empty");
            assert!(!entry.chosen.is_empty(), "chosen must not be empty");
            assert!(!entry.rejected.is_empty(), "rejected must not be empty");

            // DPO format: chosen and rejected must be different
            assert_ne!(
                entry.chosen, entry.rejected,
                "Chosen and rejected must differ for {}",
                entry.id
            );
        }
    }

    /// FALSIFY-SSB-002: Conversations contain shell code, not Rust.
    /// Instruction fields must contain shell patterns, not fn main().
    #[test]
    fn test_conversations_contain_shell() {
        use crate::corpus::conversations::generate_batch;
        use crate::corpus::dataset::strip_shell_preamble;
        use crate::corpus::registry::CorpusRegistry;

        let registry = CorpusRegistry::load_full();
        let config = crate::Config::default();

        // Transpile first 50 entries
        let transpiled: Vec<(String, String)> = registry
            .entries
            .iter()
            .take(50)
            .map(|e| {
                let shell = match e.format {
                    crate::corpus::registry::CorpusFormat::Bash => {
                        crate::transpile(&e.input, &config)
                            .map(|s| strip_shell_preamble(&s))
                            .unwrap_or_else(|_| e.input.clone())
                    }
                    crate::corpus::registry::CorpusFormat::Makefile => {
                        crate::transpile_makefile(&e.input, &config)
                            .unwrap_or_else(|_| e.input.clone())
                    }
                    crate::corpus::registry::CorpusFormat::Dockerfile => {
                        crate::transpile_dockerfile(&e.input, &config)
                            .unwrap_or_else(|_| e.input.clone())
                    }
                };
                (e.id.clone(), shell)
            })
            .collect();

        let batch: Vec<(&str, &str)> = transpiled
            .iter()
            .map(|(id, s)| (id.as_str(), s.as_str()))
            .collect();

        let (conversations, _) = generate_batch(&batch, 42);

        let mut rust_count = 0;
        for conv in &conversations {
            for turn in &conv.turns {
                if turn.role == "user" && turn.content.contains("fn main") {
                    rust_count += 1;
                }
            }
        }

        assert_eq!(
            rust_count, 0,
            "Found {} conversations containing Rust code (fn main). Expected 0.",
            rust_count
        );
    }
}
